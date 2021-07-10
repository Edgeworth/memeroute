use eyre::Result;

use crate::model::pcb::{Pcb, Via, Wire};
use crate::route::grid::GridRouter;

pub trait RouteStrategy {
    fn route(&mut self) -> Result<RouteResult>;
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct RouteResult {
    pub wires: Vec<Wire>,
    pub vias: Vec<Via>,
    pub failed: bool,
}

impl RouteResult {
    pub fn merge(&mut self, r: RouteResult) {
        self.wires.extend(r.wires);
        self.vias.extend(r.vias);
        self.failed |= r.failed;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Router {
    pcb: Pcb,
}

impl Router {
    pub fn new(pcb: Pcb) -> Self {
        Self { pcb }
    }

    pub fn route(&mut self) -> Result<RouteResult> {
        let net_order = self.pcb.nets().map(|v| v.id.clone()).collect();
        let mut grid = GridRouter::new(self.pcb.clone(), net_order);
        grid.route()
    }
}
