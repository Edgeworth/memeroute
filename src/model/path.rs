use crate::model::pt::Pt;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Path {
    pub width: f64,
    pub pts: Vec<Pt>,
}
