use rust_decimal::Decimal;

use crate::model::geom::{Pt, Rt};

// File-format independent representation of a PCB.

pub type Id = String;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Circle {
    pub r: Decimal,
    pub p: Pt,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Polygon {
    pub width: Decimal,
    pub pts: Vec<Pt>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Path {
    pub width: Decimal,
    pub pts: Vec<Pt>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Arc {
    pub width: Decimal,
    pub start: Pt,
    pub end: Pt,
    pub center: Pt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeType {
    Rect(Rt),
    Circle(Circle),
    Polygon(Polygon),
    Path(Path),
    Arc(Arc),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    layer: Id,
    shape: ShapeType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Side {
    Front,
    Back,
}

impl Default for Side {
    fn default() -> Self {
        Self::Front
    }
}

// Describes a component at a location.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Component {
    pub side: Side, // TODO: replace with layer?
}

// Describes a route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wire {
    pub shape: Shape,
}

// Describes a via.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Via {}

// Describes an overall PCB.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Pcb {
    pub name: String,
    pub wires: Vec<Wire>,
    pub vias: Vec<Via>,
    pub resolution: Decimal,  // 
}
