use eyre::Result;

use crate::model::pcb::{Pcb, Via, Wire};
use crate::model::primitive::rect::Rt;
use crate::route::grid::GridRouter;

pub trait RouteStrategy {
    fn route(&mut self) -> Result<RouteResult>;
}

#[derive(Debug, Default, Clone)]
pub struct RouteResult {
    pub wires: Vec<Wire>,
    pub vias: Vec<Via>,
    pub debug_rts: Vec<Rt>,
    pub failed: bool,
}

impl RouteResult {
    pub fn merge(&mut self, r: RouteResult) {
        self.wires.extend(r.wires);
        self.vias.extend(r.vias);
        self.debug_rts.extend(r.debug_rts);
        self.failed |= r.failed;
    }
}

#[derive(Debug, Clone)]
pub struct Router {
    pcb: Pcb,
}

impl Router {
    pub fn new(pcb: Pcb) -> Self {
        Self { pcb }
    }

    pub fn route(&mut self) -> Result<RouteResult> {
        let mut net_order: Vec<_> = self.pcb.nets().map(|v| v.id).collect();
        net_order.sort_unstable(); // TODO remove
        let mut grid = GridRouter::new(self.pcb.clone(), net_order);
        grid.route()
    }
}

pub fn apply_route_result(pcb: &mut Pcb, r: &RouteResult) {
    for wire in r.wires.iter() {
        pcb.add_wire(wire.clone());
    }
    for via in r.vias.iter() {
        pcb.add_via(via.clone());
    }
    for rt in r.debug_rts.iter() {
        pcb.add_debug_rt(*rt);
    }
}
