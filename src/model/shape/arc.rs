use crate::model::pt::Pt;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Arc {
    pub width: f64,
    pub pt: Pt,
    pub start: f64,
    pub end: f64,
}
