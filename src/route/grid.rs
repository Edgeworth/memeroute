use std::collections::HashMap;

use eyre::{eyre, Result};
use memegeom::geom::math::f64_cmp;
use memegeom::geom::qt::query::TagQuery;
use memegeom::primitive::point::{Pt, PtI};
use memegeom::primitive::rect::{Rt, RtI};
use memegeom::primitive::{circ, pt, pti, ShapeOps};
use memegeom::tf::Tf;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

use crate::model::pcb::{LayerSet, LayerShape, ObjectKind, Pcb, PinRef, Via, Wire};
use crate::name::{Id, NO_ID};
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

#[must_use]
#[derive(Debug, Default, Hash, Copy, Clone, PartialEq, Eq)]
pub struct State {
    pub p: PtI,
    pub layers: LayerSet,
    pub net_id: Id,
}

#[must_use]
#[derive(Debug, Clone, PartialEq)]
struct NodeData {
    prev: State,
    cost: f64,
    seen: bool,
}

impl Default for NodeData {
    fn default() -> Self {
        Self { seen: false, cost: f64::MAX / 10.0, prev: State::default() }
    }
}

pub type BlockMap = HashMap<State, i64>;

#[must_use]
#[derive(Debug, Clone)]
pub struct GridRouter {
    resolution: f64,
    place: PlaceModel,
    net_order: Vec<Id>,
}

impl GridRouter {
    pub fn new(pcb: Pcb, net_order: Vec<Id>) -> Self {
        let place = PlaceModel::new(pcb);
        Self { resolution: 0.4, place, net_order }
    }

    fn pin_ref_state(&self, pin_ref: &PinRef) -> Result<State> {
        let (component, pin) = self.place.pcb().pin_ref(pin_ref)?;
        let p = self.grid_pt((component.tf() * pin.tf()).pt(Pt::zero()));
        // TODO: Assumes connect to the center of the pin. Look at padstack instead.
        let layers = pin.padstack.shapes.iter().map(|v| v.layers).collect();
        let net_id =
            self.place.pcb().pin_ref_net(pin_ref).ok_or_else(|| eyre!("missing net id"))?;
        Ok(State { p, layers, net_id })
    }

    fn wire_from_states(&self, states: &[State]) -> Wire {
        let pts: Vec<_> = states.iter().map(|s| self.world_pt_mid(s.p)).collect();
        self.place.create_wire(states[0].net_id, states[0].layers.id().unwrap(), &pts)
    }

    fn via_from_state(&self, state: &State) -> Via {
        self.place.create_via(state.net_id, self.world_pt_mid(state.p))
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
                node_data.insert(s, NodeData { prev: State::default(), cost: 0.0, seen: true });
            }
        }

        let mut dst = None;
        while let Some((cur, _)) = q.pop() {
            let cur_cost = node_data.get(&cur).unwrap().cost;

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
                    let next = State {
                        p: cur.p + dp,
                        layers: LayerSet::one(layer),
                        net_id: srcs[0].net_id,
                    };
                    let cost = cur_cost + edge_cost;
                    let data = node_data.entry(next).or_insert_with(Default::default);

                    if data.seen {
                        continue;
                    }

                    let wire = self.wire_from_states(&[cur, next]);
                    // Wire is blocked if anything other than its net is there.
                    if !is_via && self.place.is_wire_blocked(&wire) {
                        continue;
                    }

                    // Vias are blocked by anything since they create a hole.
                    let via = self.via_from_state(&next);
                    if is_via && (self.place.is_via_blocked(&via)) {
                        continue;
                    }

                    if cost <= data.cost {
                        data.cost = cost;
                        data.prev = cur;

                        // A* heuristic. Minimum distance to a destination.
                        let dist_fn =
                            |d: &State| self.world_pt_mid(d.p).dist(self.world_pt_mid(next.p));
                        let heuristic = dsts.iter().map(dist_fn).min_by(f64_cmp).unwrap();
                        q.push(next, OrderedFloat(-(cost + heuristic)));
                    }
                }
            }

            let data = node_data.get_mut(&cur).unwrap();
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
            assert_eq!(cur, State::default());
            path.reverse();
            path
        } else {
            vec![]
        }
    }

    // Connect the given states together and return a route result doing that.
    fn connect(&mut self, mut srcs: Vec<State>) -> RouteResult {
        let mut res = RouteResult::default();
        if srcs.len() <= 1 {
            return res;
        }
        let mut dsts = srcs.split_off(1);
        while !dsts.is_empty() {
            let path = self.dijkstra(&srcs, &dsts);
            if path.is_empty() {
                res.failed = true;
                return res;
            }
            let (wires, vias) = self.create_path(&path);
            for wire in &wires {
                self.place.add_wire(wire);
            }
            for via in &vias {
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

        res
    }

    fn _draw_debug(&mut self, res: &mut RouteResult) {
        let bounds = self.place.pcb().bounds();
        // let bounds = rt(77.0495, -125.1745, 79.099, -120.75);
        let bounds =
            RtI::enclosing(self.grid_pt(bounds.bl()), self.grid_pt(bounds.tr()) + pti(1, 1));
        for l in bounds.l()..bounds.r() {
            for b in bounds.b()..bounds.t() {
                let p = pti(l, b);
                let shape = circ(self.world_pt_mid(p), self.resolution / 2.0).shape();
                let shape = LayerShape { layers: LayerSet::one(0), shape };
                if self.place.is_shape_blocked(
                    &Tf::identity(),
                    &shape,
                    TagQuery::All,
                    ObjectKind::Wire,
                    &[],
                ) {
                    continue;
                }
                res.wires.push(Wire { shape, net_id: NO_ID });
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
        for net_id in self.net_order.clone() {
            let net = self
                .place
                .pcb()
                .net(net_id)
                .ok_or_else(|| eyre!("missing net {}", net_id))?
                .clone();
            let states = net.pins.iter().map(|p| self.pin_ref_state(p)).collect::<Result<_>>()?;

            let sub_result = self.connect(states);
            println!("done {}, failed {}", self.place.pcb().to_name(net_id), sub_result.failed);
            // Mark wires and vias.
            for wire in &sub_result.wires {
                self.place.add_wire(wire);
            }
            for via in &sub_result.vias {
                self.place.add_via(via);
            }
            res.merge(sub_result);
        }

        // self.draw_debug(&mut res);
        Ok(res)
    }
}
