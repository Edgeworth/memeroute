use rust_decimal::Decimal;

use crate::model::geom::{Pt, Rt};

// File-format independent representation of a PCB.

pub type Id = String;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Circle {
    pub r: u64,
    pub p: Pt,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Polygon {
    pub width: u64,
    pub pts: Vec<Pt>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Path {
    pub width: u64,
    pub pts: Vec<Pt>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Arc {
    pub width: u64,
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
    pub layer: Id,
    pub shape: ShapeType,
}

// Keepout: No routing whatsoever.
// ViaKeepout: No vias.
// WireKeepout: No wires.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeepoutType {
    Keepout,
    ViaKeepout,
    WireKeepout,
}

// Describes a keepout area.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keepout {
    pub kind: KeepoutType,
    pub shape: Shape,
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

// Describes a pin.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Pin {
    pub id: Id,
    pub padstack: Padstack,
    pub rotation: Decimal,
    pub p: Pt,
}

// Describes a component at a location.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Component {
    pub id: Id,
    pub side: Side,
    pub outlines: Vec<Shape>,
    pub keepouts: Vec<Keepout>,
    pub pins: Vec<Pin>,
}

// Describes a padstack.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Padstack {
    pub id: Id,
    pub shapes: Vec<Shape>,
    pub attach: bool,
}

// Describes a layer in a PCB.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Layer {
    pub id: Id,
}

impl Layer {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_owned() }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PinRef {
    pub component: Id,
    pub pin: Id,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Net {
    pub id: Id,
    pub pins: Vec<PinRef>,
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
    id: Id,

    // Physical structure:
    layers: Vec<Layer>,
    boundaries: Vec<Shape>,
    keepouts: Vec<Keepout>,
    via_padstacks: Vec<Padstack>, // Types of vias available to use.
    components: Vec<Component>,

    // Routing:
    wires: Vec<Wire>,
    vias: Vec<Via>,
    nets: Vec<Net>,

    resolution: Decimal, // Stores how many millimetres each unit value represents.
}

impl Pcb {
    pub fn set_id(&mut self, id: &str) {
        self.id = id.to_owned();
    }

    pub fn set_resolution(&mut self, resolution: Decimal) {
        self.resolution = resolution;
    }

    pub fn add_layer(&mut self, l: Layer) {
        self.layers.push(l);
    }

    pub fn add_boundary(&mut self, s: Shape) {
        self.boundaries.push(s);
    }

    pub fn add_keepout(&mut self, k: Keepout) {
        self.keepouts.push(k);
    }

    pub fn add_via_padstack(&mut self, p: Padstack) {
        self.via_padstacks.push(p);
    }

    pub fn add_component(&mut self, c: Component) {
        self.components.push(c);
    }

    pub fn add_net(&mut self, n: Net) {
        self.nets.push(n);
    }
}
