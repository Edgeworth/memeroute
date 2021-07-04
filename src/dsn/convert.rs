use eyre::Result;

use crate::dsn::types::DsnPcb;
use crate::model::pcb::Pcb;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Converter {
    dsn: DsnPcb,
    pcb: Pcb,
}

impl Converter {
    pub fn new(dsn: DsnPcb) -> Self {
        Self { dsn, pcb: Default::default() }
    }

    pub fn convert(mut self) -> Result<Pcb> {
        self.pcb.name = self.dsn.pcb_id;
        Ok(self.pcb)
    }
}
