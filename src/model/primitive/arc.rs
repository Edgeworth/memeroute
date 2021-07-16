use crate::model::pt::Pt;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Arc {
    pub pt: Pt,
    pub start: f64,
    pub end: f64,
    pub width: f64,
}
