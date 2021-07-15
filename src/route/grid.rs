use std::collections::{HashMap, HashSet};

use eyre::{eyre, Result};
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

use crate::model::pcb::{Id, Pcb, Shape, Via, Wire};
use crate::model::pt::PtI;
use crate::model::shape::circle::Circle;
use crate::model::shape::path::Path;
use crate::model::shape::shape_type::ShapeType;
use crate::route::grid_model::GridModel;
use crate::route::router::{RouteResult, RouteStrategy};

const DIR: [(PtI, f64); 8] = [
    (PtI::new(-1, 0), 1.0),
    (PtI::new(1, 0), 1.0),
    (PtI::new(0, -1), 1.0),
    (PtI::new(0, 1), 1.0),
    (PtI::new(1, 1), 1.414),
    (PtI::new(1, -1), 1.414),
    (PtI::new(-1, 1), 1.414),
    (PtI::new(-1, -1), 1.414),
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
    model: GridModel,
    net_order: Vec<Id>,
    blk: BlockMap,
}

impl GridRouter {
    pub fn new(pcb: Pcb, net_order: Vec<Id>) -> Self {
        let mut s = Self { model: GridModel::new(pcb), net_order, blk: BlockMap::new() };
        s.model.mark_blocked(&mut s.blk);
        s
    }

    fn create_path(&self, path: &[State]) -> (Vec<Wire>, Vec<Via>) {
        let mut wires = Vec::new();
        let mut cur_wire = Vec::new();
        let mut prev = State::default();
        for (idx, cur) in path.iter().enumerate() {
            cur_wire.push(self.model.world_pt_mid(cur.p));

            // Add the wire.
            if !prev.layer.is_empty() && cur.layer != prev.layer || idx == path.len() - 1 {
                // TODO: Assumes wire width some proportion of resolution.
                wires.push(Wire {
                    shape: Shape {
                        layer: cur.layer.clone(),
                        shape: ShapeType::Path(Path::new(&cur_wire, self.model.resolution * 0.8)),
                    },
                });
                cur_wire.clear();
                // TODO: Add a via.
            }
            prev = cur.clone();
        }
        (wires, vec![])
    }

    // Returns
    fn dijkstra(&self, srcs: &[State], dsts: &[State]) -> Vec<State> {
        let mut q: PriorityQueue<State, OrderedFloat<f64>> = PriorityQueue::new();
        let mut node_data: HashMap<State, NodeData> = HashMap::new();

        for src in srcs.iter() {
            q.push(src.clone(), OrderedFloat(0.0));
        }

        let mut dst = None;
        while let Some((cur, cur_cost)) = q.pop() {
            let cur_cost = -cur_cost.0;
            for (dp, edge_cost) in DIR.iter() {
                let next = State {
                    p: cur.p + dp,
                    layer: cur.layer.clone(), // TODO: transition
                };
                let cost = cur_cost + edge_cost;
                let data = node_data.entry(next.clone()).or_insert_with(Default::default);
                if data.seen || self.model.is_blocked(&self.blk, &next) {
                    continue;
                }
                if cost <= data.cost {
                    data.cost = cost;
                    data.prev = cur.clone();
                    q.push(next, OrderedFloat(-cost));
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
                self.model.mark_wire(&mut self.blk, 1, wire);
            }
            for via in vias.iter() {
                self.model.mark_via(&mut self.blk, 1, via);
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
            let net =
                self.model.pcb.net(&net_id).ok_or_else(|| eyre!("missing net {}", net_id))?.clone();
            let states =
                net.pins.iter().map(|p| self.model.pin_ref_state(p)).collect::<Result<_>>()?;

            self.model.mark_net(&mut self.blk, -1, &net)?; // Temporarily remove pins as blocking.
            res.merge(self.connect(states)?);
            self.model.mark_net(&mut self.blk, 1, &net)?; // Add them back.
        }

        let bounds = self.model.grid_rt(&self.model.pcb.bounds());
        for l in bounds.l()..bounds.r() {
            for b in bounds.b()..bounds.t() {
                let p = PtI::new(l, b);
                if self.model.is_blocked(&self.blk, &State { p, layer: "F.Cu".to_owned() }) {
                    continue;
                }
                let shape = ShapeType::Circle(Circle::new(
                    self.model.world_pt_mid(p),
                    self.model.resolution / 2.0,
                ));
                res.wires.push(Wire { shape: Shape { layer: "F.Cu".to_owned(), shape } })
            }
        }
        Ok(res)
    }
}
