use std::collections::HashMap;

use eyre::{eyre, Result};

use crate::model::pcb::{Id, Net, Padstack, Pcb, Pin, PinRef, Shape, Side, Wire, ANY_LAYER};
use crate::model::pt::PtI;
use crate::model::shape::circle::Circle;
use crate::model::shape::shape_type::ShapeType;
use crate::route::grid_model::GridModel;
use crate::route::router::{RouteResult, RouteStrategy};

const DIR: [(PtI, f32); 8] = [
    (PtI::new(-1, 0), 1.0),
    (PtI::new(1, 0), 1.0),
    (PtI::new(0, -1), 1.0),
    (PtI::new(0, 1), 1.0),
    (PtI::new(1, 1), 1.414),
    (PtI::new(1, -1), 1.414),
    (PtI::new(-1, 1), 1.414),
    (PtI::new(-1, -1), 1.414),
];

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct State {
    pub p: PtI,
    pub layer: Id,
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

    // Connect the given states together and return a route result doing that.
    fn connect(&self, _states: Vec<State>) -> Result<RouteResult> {
        let res = RouteResult::default();
        // dijkstra(graph, start, goal, edge_cost)
        Ok(res)
    }
}

impl RouteStrategy for GridRouter {
    fn route(&mut self) -> Result<RouteResult> {
        let mut res = RouteResult::default();
        for net_id in self.net_order.clone().into_iter() {
            let net = self.model.pcb.net(&net_id).ok_or_else(|| eyre!("missing net {}", net_id))?;
            let states =
                net.pins.iter().map(|p| self.model.pin_ref_state(p)).collect::<Result<_>>()?;

            self.model.mark_net(&mut self.blk, -1, net)?; // Temporarily remove pins as blocking.
            res.merge(self.connect(states)?);
            self.model.mark_net(&mut self.blk, 1, net)?; // Add them back.
        }

        let bounds = self.model.grid_rt(&self.model.pcb.bounds());
        for l in bounds.l()..bounds.r() {
            for b in bounds.b()..bounds.t() {
                let p = PtI::new(l, b);
                if self.model.is_blocked(&self.blk, &State { p, layer: "F.Cu".to_owned() })? {
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
