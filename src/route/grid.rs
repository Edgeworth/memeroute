use std::collections::HashMap;

use eyre::{eyre, Result};
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

use crate::model::geom::math::f64_cmp;
use crate::model::pcb::{Id, LayerSet, LayerShape, Pcb, PinRef, Via, Wire};
use crate::model::primitive::point::{Pt, PtI};
use crate::model::primitive::rect::{Rt, RtI};
use crate::model::primitive::{circ, path, pt, pti, ShapeOps};
use crate::model::tf::Tf;
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

#[derive(Debug, Default, Hash, Copy, Clone, PartialEq, Eq)]
pub struct State {
    pub p: PtI,
    pub layers: LayerSet,
}

#[derive(Debug, Clone, PartialEq)]
struct NodeData {
    prev: State,
    cost: f64,
    seen: bool,
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
        Self { pcb, resolution: 0.4, place, net_order }
    }

    fn pin_ref_state(&self, pin_ref: &PinRef) -> Result<State> {
        let (component, pin) = self.pcb.pin_ref(pin_ref)?;
        let p = self.grid_pt((component.tf() * pin.tf()).pt(Pt::zero()));
        // TODO: Assumes connect to the center of the pin. Look at padstack instead.
        let layers = pin.padstack.shapes.iter().map(|v| v.layers).collect();
        Ok(State { p, layers })
    }

    fn wire_from_states(&self, states: &[State]) -> Wire {
        let pts: Vec<_> = states.iter().map(|s| self.world_pt_mid(s.p)).collect();
        Wire {
            shape: LayerShape {
                layers: LayerSet::one(states[0].layers.id().unwrap()),
                shape: path(&pts, self.resolution * 0.8).shape(),
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
        let is_via = l >= 2 && cur[l - 1].layers != cur[l - 2].layers;
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
            cur_states.push(*cur);
        }
        self.push_path(&mut wires, &mut vias, &mut cur_states, true);
        (wires, vias)
    }

    fn dijkstra(&self, srcs: &[State], dsts: &[State]) -> Vec<State> {
        let mut q: PriorityQueue<State, OrderedFloat<f64>> = PriorityQueue::new();
        let mut node_data: HashMap<State, NodeData> = HashMap::new();

        for src in srcs {
            // Try going from each of the valid layers in this state.
            for layer in src.layers.iter() {
                let s = State { layers: LayerSet::one(layer), ..*src };
                q.push(s, OrderedFloat(0.0));
            }
        }

        let mut dst = None;
        while let Some((cur, cur_cost)) = q.pop() {
            let cur_cost = -cur_cost.0;

            for (dp, edge_cost) in DIR {
                let is_via = dp.is_zero();
                let cur_layer = cur.layers.id().unwrap(); // Should only be one layer.
                let layers = if is_via {
                    let mut layers = self.via_from_state(&cur).padstack.layers();
                    // Try all layers from via except the current one.
                    layers.remove(cur_layer);
                    layers
                } else {
                    LayerSet::one(cur_layer)
                };
                for layer in layers.iter() {
                    let next = State { p: cur.p + dp, layers: LayerSet::one(layer) };
                    let cost = cur_cost + edge_cost;
                    let data = node_data.entry(next).or_insert_with(Default::default);

                    if data.seen {
                        continue;
                    }

                    let wire = self.wire_from_states(&[cur, next]);
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

                    // A* heuristic. Minimum distance to a destination.
                    // let heuristic = dsts.iter().map(|v| v.p.dist(next.p)).min_by(f64_cmp);

                    if cost <= data.cost {
                        data.cost = cost;
                        data.prev = cur;
                        q.push(next, OrderedFloat(-cost));
                    }
                }
            }

            let data = node_data.entry(cur).or_insert_with(Default::default);
            data.seen = true;
            // Check if we reached any destination.
            for d in dsts {
                if d.p == cur.p && d.layers.contains_set(cur.layers) {
                    dst = Some(cur);
                    break;
                }
            }
            if dst.is_some() {
                break;
            }
        }

        if let Some(dst) = dst {
            // Recover path.
            let mut path = Vec::new();
            let mut cur = dst;
            while let Some(data) = node_data.get(&cur) {
                path.push(cur);
                cur = data.prev;
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
            let dst = path.last().unwrap();
            let idx = dsts
                .iter()
                .position(|v| v.p == dst.p && v.layers.contains_set(dst.layers))
                .unwrap();
            srcs.push(dsts.swap_remove(idx));
        }

        Ok(res)
    }

    fn draw_debug(&mut self, res: &mut RouteResult) {
        let bounds = self.pcb.bounds();
        // let bounds = rt(77.0495, -125.1745, 79.099, -120.75);
        let bounds =
            RtI::enclosing(self.grid_pt(bounds.bl()), self.grid_pt(bounds.tr()) + pti(1, 1));
        for l in bounds.l()..bounds.r() {
            for b in bounds.b()..bounds.t() {
                let p = pti(l, b);
                let shape = circ(self.world_pt_mid(p), self.resolution / 2.0).shape();
                let shape = LayerShape { layers: LayerSet::one(0), shape };
                if self.place.is_shape_blocked(&Tf::identity(), &shape) {
                    continue;
                }
                res.wires.push(Wire { shape });
            }
        }

        let bounds = RtI::new(157, -116, 1, 1);
        res.debug_rts.push(
            Rt::enclosing(self.world_pt(bounds.bl()), self.world_pt(bounds.tr()))
                .inset(-10.0, -10.0),
        );
        res.debug_rts.extend(self.place.debug_rts());
    }
}

impl RouteStrategy for GridRouter {
    fn route(&mut self) -> Result<RouteResult> {
        let mut res = RouteResult::default();
        for net_id in self.net_order.clone().into_iter() {
            let net = self.pcb.net(&net_id).ok_or_else(|| eyre!("missing net {}", net_id))?.clone();
            let states = net.pins.iter().map(|p| self.pin_ref_state(p)).collect::<Result<_>>()?;

            self.place.remove_net(&net); // Temporarily remove pins as blocking.
            let sub_result = self.connect(states)?;
            // Mark wires and vias.
            for wire in sub_result.wires.iter() {
                self.place.add_wire(wire);
            }
            for via in sub_result.vias.iter() {
                self.place.add_via(via);
            }
            res.merge(sub_result);
            self.place.add_net(&self.pcb, &net)?; // Add pins back.
            println!("done");
        }

        // self.draw_debug(&mut res);
        Ok(res)
    }
}
