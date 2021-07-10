use eyre::{eyre, Result};

use crate::model::pcb::{Id, Pcb};
use crate::route::router::{RouteResult, RouteStrategy};

type GridIdx = (i32, i32);

const DIR: [GridIdx; 9] =
    [(-1, -0), (-1, 0), (-1, 0), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1), (0, 0)];

#[derive(Debug, Clone, PartialEq)]
pub struct GridRouter {
    pcb: Pcb,
    net_order: Vec<Id>,
}

impl GridRouter {
    pub fn new(pcb: Pcb, net_order: Vec<Id>) -> Self {
        Self { pcb, net_order }
    }
}

impl RouteStrategy for GridRouter {
    fn route(&mut self) -> Result<RouteResult> {
        for net_id in self.net_order.iter() {
            let net = self.pcb.net(net_id).ok_or_else(|| eyre!("missing net {}", net_id))?;
        }
        Ok(RouteResult { wires: Vec::new(), vias: Vec::new(), failed: false })
    }
}
