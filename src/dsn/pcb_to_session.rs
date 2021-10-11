use eyre::Result;

use crate::model::pcb::Pcb;

#[derive(Debug, Clone)]
pub struct PcbToSession {
    pcb: Pcb,
}

impl PcbToSession {
    pub fn new(pcb: Pcb) -> Self {
        Self { pcb }
    }


    pub fn convert(self) -> Result<String> {
        let s = String::new();
        Ok(s)
    }
}
