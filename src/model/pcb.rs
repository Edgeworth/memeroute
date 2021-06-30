use rust_decimal::Decimal;

use crate::model::geom::{PtF, RtF};

// File-format independent representation of a PCB.

pub type Id = String;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Circle {
    r: Decimal,
    p: PtF,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Polygon {
    width: Decimal,
    pts: Vec<PtF>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Path {
    width: Decimal,
    pts: Vec<PtF>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Arc {
    width: Decimal,
    start: PtF,
    end: PtF,
    center: PtF,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeType {
    Rect(RtF),
    Circle(Circle),
    Polygon(Polygon),
    Path(Path),
    Arc(Arc),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    layer: Id,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Side {
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
    side: Side, // TODO: replace with layer?
}

// Describes a route.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Wire {}

// Describes a via.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Via {}

// Describes an overall PCB.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Pcb {
    wires: Vec<Wire>,
    vias: Vec<Via>,
}
