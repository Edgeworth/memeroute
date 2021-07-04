use eyre::Result;

use crate::model::pcb::{Via, Wire};

pub trait RouteStrategy {
    fn route() -> RouteResult;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteResult {
    wires: Vec<Wire>,
    vias: Vec<Via>,
    failed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Router {}

impl Router {
    pub fn new() -> Self {
        Self {}
    }

    pub fn route() -> Result<RouteResult> {
        todo!()
    }
}
