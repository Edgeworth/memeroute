use std::collections::HashSet;

use eyre::{eyre, Result};
use petgraph::algo::dijkstra;

use crate::model::pcb::{Id, Pcb, PinRef, Side};
use crate::model::pt::Pt;
use crate::route::router::{RouteResult, RouteStrategy};

type GridIdx = (i32, i32);

const DIR: [(GridIdx, f32); 8] = [
    ((-1, 0), 1.0),
    ((1, 0), 1.0),
    ((0, -1), 1.0),
    ((0, 1), 1.0),
    ((1, 1), 1.414),
    ((1, -1), 1.414),
    ((-1, 1), 1.414),
    ((-1, -1), 1.414),
];

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct State {
    idx: GridIdx,
    layer: Id,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridRouter {
    pcb: Pcb,
    net_order: Vec<Id>,
    resolution: f64, // Resolution in mm.
    blocked: HashSet<State>,
}

impl GridRouter {
    pub fn new(pcb: Pcb, net_order: Vec<Id>) -> Self {
        Self { pcb, net_order, resolution: 0.1, blocked: HashSet::new() }
    }

    fn mark_blocked(&mut self) {}

    // Connect the given states together and return a route result doing that.
    fn connect(&mut self, states: Vec<State>) -> Result<RouteResult> {
        let res = RouteResult::default();
        // dijkstra(graph, start, goal, edge_cost)
        Ok(res)
    }

    // TODO: Assumes connect to the center of the pin. Look at padstack instead.
    fn pin_ref_state(&self, p: &PinRef) -> Result<State> {
        let (component, pin) = self.pcb.pin_ref(p)?;
        let idx = self.to_grid((component.tf() * pin.tf()).pt(pin.p));
        // TODO: Using component side for which layer is broken. Need to look at
        // padstack.
        let layer = match component.side {
            Side::Front => "F.Cu".to_owned(),
            Side::Back => "B.Cu".to_owned(),
        };
        Ok(State { idx, layer })
    }

    fn to_grid(&self, p: Pt) -> GridIdx {
        ((p.x / self.resolution).trunc() as i32, (p.y / self.resolution).trunc() as i32)
    }

    fn to_world(&self, p: &GridIdx) -> Pt {
        Pt::new(
            p.0 as f64 * self.resolution + self.resolution / 2.0,
            p.1 as f64 * self.resolution + self.resolution / 2.0,
        )
    }

    // Checks if the point |p| is routable (inside boundary, outside of
    // keepouts, etc).
    fn is_oob(&self, p: &GridIdx) -> bool {
        false
    }
}

impl RouteStrategy for GridRouter {
    fn route(&mut self) -> Result<RouteResult> {
        let mut res = RouteResult::default();
        for net_id in self.net_order.clone().into_iter() {
            let net = self.pcb.net(&net_id).ok_or_else(|| eyre!("missing net {}", net_id))?;
            let states = net.pins.iter().map(|p| self.pin_ref_state(p)).collect::<Result<_>>()?;
            res.merge(self.connect(states)?);
        }
        Ok(res)
    }
}
