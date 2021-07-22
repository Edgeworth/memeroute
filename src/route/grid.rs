use std::collections::HashMap;

use eyre::{eyre, Result};
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

use crate::model::pcb::{Id, LayerShape, Pcb, PinRef, Via, Wire};
use crate::model::primitive::point::{Pt, PtI};
use crate::model::primitive::{path, pt, pti, ShapeOps};
use crate::route::place_model::PlaceModel;
use crate::route::router::{RouteResult, RouteStrategy};

const VIA_COST: f64 = 10.0;

const DIR: [(PtI, f64); 9] = [
    (pti(-1, 0), 1.0),
    (pti(1, 0), 1.0),
    (pti(0, -1), 1.0),
    (pti(0, 1), 1.0),
    (pti(1, 1), 1.414),
    (pti(1, -1), 1.414),
    (pti(-1, 1), 1.414),
    (pti(-1, -1), 1.414),
    (pti(0, 0), VIA_COST),
];

#[derive(Debug, Default, Hash, Clone, PartialEq, Eq)]
pub struct State {
    pub p: PtI,
    pub layer: Id,
}

#[derive(Debug, Clone, PartialEq)]
struct NodeData {
    seen: bool,
    cost: f64,
    prev: State,
}

impl Default for NodeData {
    fn default() -> Self {
        Self { seen: false, cost: f64::MAX / 10.0, prev: Default::default() }
    }
}

pub type BlockMap = HashMap<State, i64>;

#[derive(Debug, Clone)]
pub struct GridRouter {
    pcb: Pcb,
    resolution: f64,
    place: PlaceModel,
    net_order: Vec<Id>,
}

impl GridRouter {
    pub fn new(pcb: Pcb, net_order: Vec<Id>) -> Self {
        let mut place = PlaceModel::new();
        place.add_pcb(&pcb);
        Self { pcb, resolution: 0.8, place, net_order }
    }

    // TODO: Assumes connect to the center of the pin. Look at padstack instead.
    fn pin_ref_state(&self, pin_ref: &PinRef) -> Result<State> {
        let (component, pin) = self.pcb.pin_ref(pin_ref)?;
        let p = self.grid_pt((component.tf() * pin.tf()).pt(Pt::zero()));

        // TODO: return all layers in padstack somehow.
        let mut layer = "F.Cu".to_owned();
        for shape in pin.padstack.shapes.iter() {
            layer = shape.layer.clone();
        }
        Ok(State { p, layer })
    }

    fn wire_from_states(&self, states: &[State]) -> Wire {
        let pts: Vec<_> = states.iter().map(|s| self.world_pt_mid(s.p)).collect();
        Wire {
            shape: LayerShape {
                layer: states[0].layer.clone(),
                shape: path(&pts, self.resolution * 0.4).shape(),
            },
        }
    }

    fn via_from_state(&self, state: &State) -> Via {
        // TODO: Uses only one type of via.
        Via { padstack: self.pcb.via_padstacks()[0].clone(), p: self.world_pt_mid(state.p) }
    }

    fn grid_pt(&self, p: Pt) -> PtI {
        // Map points to the lower left corner.
        pti((p.x / self.resolution).floor() as i64, (p.y / self.resolution).floor() as i64)
    }

    fn world_pt(&self, p: PtI) -> Pt {
        pt(p.x as f64 * self.resolution, p.y as f64 * self.resolution)
    }

    fn world_pt_mid(&self, p: PtI) -> Pt {
        self.world_pt(p) + pt(self.resolution / 2.0, self.resolution / 2.0)
    }

    fn push_path(
        &self,
        wires: &mut Vec<Wire>,
        vias: &mut Vec<Via>,
        cur: &mut Vec<State>,
        last: bool,
    ) {
        let l = cur.len();
        if cur.is_empty() {
            return;
        }
        let is_via = l >= 2 && cur[l - 1].layer != cur[l - 2].layer;
        // Add the via.
        if is_via {
            vias.push(self.via_from_state(&cur[l - 1]));
        }
        // Add the wire, if it exists.
        if is_via || last {
            // TODO: Assumes wire width some proportion of resolution.
            // Keeps duplicated last point if we made a via. That allows for
            // wires that only take up one square.
            wires.push(self.wire_from_states(cur));
            // Only keep the last element - for starting next wire in the same
            // location as the via.
            cur.swap(0, l - 1);
            cur.truncate(1);
        }
    }

    fn create_path(&self, path: &[State]) -> (Vec<Wire>, Vec<Via>) {
        let mut wires = Vec::new();
        let mut vias = Vec::new();
        let mut cur_states = Vec::new();
        for cur in path {
            self.push_path(&mut wires, &mut vias, &mut cur_states, false);
            cur_states.push(cur.clone());
        }
        self.push_path(&mut wires, &mut vias, &mut cur_states, true);
        (wires, vias)
    }

    fn dijkstra(&self, srcs: &[State], dsts: &[State]) -> Vec<State> {
        let mut q: PriorityQueue<State, OrderedFloat<f64>> = PriorityQueue::new();
        let mut node_data: HashMap<State, NodeData> = HashMap::new();

        for src in srcs {
            q.push(src.clone(), OrderedFloat(0.0));
        }

        let mut dst = None;
        while let Some((cur, cur_cost)) = q.pop() {
            let cur_cost = -cur_cost.0;
            for (dp, edge_cost) in DIR {
                let is_via = dp.is_zero();
                let layers = if is_via {
                    let via_padstack = self.via_from_state(&cur).padstack;
                    let layers = via_padstack.shapes.iter().map(|s| s.layer.clone());
                    layers.filter(|v| v != &cur.layer).collect::<Vec<_>>()
                } else {
                    vec![cur.layer.clone()]
                };
                for layer in layers.into_iter() {
                    let next = State { p: cur.p + dp, layer };
                    let cost = cur_cost + edge_cost;
                    let data = node_data.entry(next.clone()).or_insert_with(Default::default);

                    if data.seen {
                        continue;
                    }

                    // TODO: Don't check if wire with thickness is blocked
                    // because wires already mark a 2x2 area around them (at
                    // least).
                    let wire = self.wire_from_states(&[cur.clone(), next.clone()]);
                    if !is_via && self.place.is_wire_blocked(&wire) {
                        continue;
                    }
                    // Don't put vias through pins.
                    let via = self.via_from_state(&next);
                    if is_via
                        && (self.place.is_via_blocked(&via) || self.place.is_via_blocked(&via))
                    {
                        continue;
                    }
                    if cost <= data.cost {
                        data.cost = cost;
                        data.prev = cur.clone();
                        q.push(next, OrderedFloat(-cost));
                    }
                }
            }

            let data = node_data.entry(cur.clone()).or_insert_with(Default::default);
            data.seen = true;
            if dsts.contains(&cur) {
                dst = Some(cur);
                break;
            }
        }

        if let Some(dst) = dst {
            // Recover path.
            let mut path = Vec::new();
            let mut cur = dst;
            while let Some(data) = node_data.get(&cur) {
                path.push(cur);
                cur = data.prev.clone();
            }
            // Should reach the end of the path.
            assert_eq!(cur, Default::default());
            path.reverse();
            path
        } else {
            vec![]
        }
    }

    // Connect the given states together and return a route result doing that.
    fn connect(&mut self, mut srcs: Vec<State>) -> Result<RouteResult> {
        let mut res = RouteResult::default();
        if srcs.len() <= 1 {
            return Ok(res);
        }
        let mut dsts = srcs.split_off(1);
        while !dsts.is_empty() {
            let path = self.dijkstra(&srcs, &dsts);
            if path.is_empty() {
                res.failed = true;
                return Ok(res);
            }
            let (wires, vias) = self.create_path(&path);
            for wire in wires.iter() {
                self.place.add_wire(wire);
            }
            for via in vias.iter() {
                self.place.add_via(via);
            }
            res.wires.extend(wires);
            res.vias.extend(vias);
            // Assume the last state in the path is a destination.
            let idx = dsts.iter().position(|v| v == path.last().unwrap()).unwrap();
            srcs.push(dsts.swap_remove(idx));
        }

        Ok(res)
    }
}

impl RouteStrategy for GridRouter {
    fn route(&mut self) -> Result<RouteResult> {
        let mut res = RouteResult::default();
        for net_id in self.net_order.clone().into_iter() {
            let net = self.pcb.net(&net_id).ok_or_else(|| eyre!("missing net {}", net_id))?.clone();
            let states = net.pins.iter().map(|p| self.pin_ref_state(p)).collect::<Result<_>>()?;

            //self.place.add_net(&self.pcb, &net)?; // Temporarily remove pins as blocking.
            let sub_result = self.connect(states)?;
            // Mark wires and vias.
            for wire in sub_result.wires.iter() {
                self.place.add_wire(wire);
            }
            for via in sub_result.vias.iter() {
                self.place.add_via(via);
            }
            res.merge(sub_result);
            //self.place.remove_shape(&net)?; // Add pins back.
        }

        // let bounds = self.grid_rt(&self.pcb.bounds());
        // for l in bounds.l()..bounds.r() {
        //     for b in bounds.b()..bounds.t() {
        //         let p = pti(l, b);
        //         if self.is_state_blocked(&State { p, layer: "F.Cu".to_owned() }) {
        //             continue;
        //         }
        //         let shape = circ(self.world_pt_mid(p), self.resolution / 2.0).shape();
        //         res.wires.push(Wire { shape: LayerShape { layer: "F.Cu".to_owned(), shape } })
        //     }
        // }
        Ok(res)
    }
}
