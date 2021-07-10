use crate::model::pt::Pt;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Circle {
    pub r: f64, // Radius
    pub p: Pt,  // Center
}
