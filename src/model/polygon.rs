use crate::model::pt::Pt;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Polygon {
    pub width: f64,
    pub pts: Vec<Pt>,
}
