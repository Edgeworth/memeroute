use std::collections::hash_map::Values;
use std::collections::HashMap;

use eyre::{eyre, Result};

use crate::model::arc::Arc;
use crate::model::circle::Circle;
use crate::model::path::Path;
use crate::model::polygon::Polygon;
use crate::model::pt::Pt;
use crate::model::rt::Rt;
use crate::model::tf::Tf;

// File-format independent representation of a PCB.
// Units are in millimetres.
// All rotations are in degrees, counterclockwise from the positive x axis.

pub type Id = String;

#[derive(Debug, Clone, PartialEq)]
pub enum ShapeType {
    Rect(Rt),
    Circle(Circle),
    Polygon(Polygon),
    Path(Path),
    Arc(Arc),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Shape {
    pub layer: Id,
    pub shape: ShapeType,
}

// Keepout: No routing whatsoever.
// ViaKeepout: No vias.
// WireKeepout: No wires.
#[derive(Debug, Clone, PartialEq)]
pub enum KeepoutType {
    Keepout,
    ViaKeepout,
    WireKeepout,
}

// Describes a keepout area.
#[derive(Debug, Clone, PartialEq)]
pub struct Keepout {
    pub kind: KeepoutType,
    pub shape: Shape,
}

#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Pin {
    pub id: Id,
    pub padstack: Padstack,
    pub rotation: f64,
    pub p: Pt,
}

impl Pin {
    pub fn tf(&self) -> Tf {
        Tf::translate(self.p) * Tf::rotate(self.rotation)
    }
}

// Describes a component at a location.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Component {
    pub id: Id,
    pub p: Pt,
    pub side: Side,
    pub rotation: f64,
    pub outlines: Vec<Shape>,
    pub keepouts: Vec<Keepout>,
    pins: HashMap<Id, Pin>,
}

impl Component {
    pub fn add_pin(&mut self, p: Pin) {
        self.pins.insert(p.id.clone(), p);
    }

    pub fn pins(&self) -> Values<'_, Id, Pin> {
        self.pins.values()
    }

    pub fn pin(&self, id: &str) -> Option<&Pin> {
        self.pins.get(id)
    }

    pub fn tf(&self) -> Tf {
        let side_tf =
            if self.side == Side::Back { Tf::scale(Pt::new(-1.0, -1.0)) } else { Tf::identity() };
        Tf::translate(self.p) * Tf::rotate(self.rotation) * side_tf
    }
}

// Describes a padstack.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Padstack {
    pub id: Id,
    pub shapes: Vec<Shape>,
    pub attach: bool,
}

// Describes a layer in a PCB.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Layer {
    pub id: Id,
}

impl Layer {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_owned() }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PinRef {
    pub component: Id,
    pub pin: Id,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Net {
    pub id: Id,
    pub pins: Vec<PinRef>,
}

// Describes a route.
#[derive(Debug, Clone, PartialEq)]
pub struct Wire {
    pub shape: Shape,
}

// Describes a via.
#[derive(Debug, Clone, PartialEq)]
pub struct Via {}

// Describes an overall PCB.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Pcb {
    id: Id,

    // Physical structure:
    layers: Vec<Layer>,
    boundaries: Vec<Shape>,
    keepouts: Vec<Keepout>,
    via_padstacks: Vec<Padstack>, // Types of vias available to use.
    components: HashMap<Id, Component>,

    // Routing:
    wires: Vec<Wire>,
    vias: Vec<Via>,
    nets: HashMap<Id, Net>,
}

impl Pcb {
    pub fn set_id(&mut self, id: &str) {
        self.id = id.to_owned();
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn add_layer(&mut self, l: Layer) {
        self.layers.push(l);
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    pub fn add_boundary(&mut self, s: Shape) {
        self.boundaries.push(s);
    }

    pub fn boundaries(&self) -> &[Shape] {
        &self.boundaries
    }

    pub fn add_keepout(&mut self, k: Keepout) {
        self.keepouts.push(k);
    }

    pub fn keepouts(&self) -> &[Keepout] {
        &self.keepouts
    }

    pub fn add_via_padstack(&mut self, p: Padstack) {
        self.via_padstacks.push(p);
    }

    pub fn via_padstacks(&self) -> &[Padstack] {
        &self.via_padstacks
    }

    pub fn add_component(&mut self, c: Component) {
        self.components.insert(c.id.clone(), c);
    }

    pub fn components(&self) -> Values<'_, Id, Component> {
        self.components.values()
    }

    pub fn component(&self, id: &str) -> Option<&Component> {
        self.components.get(id)
    }

    pub fn wires(&self) -> &[Wire] {
        &self.wires
    }

    pub fn vias(&self) -> &[Via] {
        &self.vias
    }

    pub fn add_net(&mut self, n: Net) {
        self.nets.insert(n.id.clone(), n);
    }

    pub fn nets(&self) -> Values<'_, Id, Net> {
        self.nets.values()
    }

    pub fn net(&self, id: &str) -> Option<&Net> {
        self.nets.get(id)
    }

    pub fn add_wire(&mut self, w: Wire) {
        self.wires.push(w);
    }

    pub fn add_via(&mut self, v: Via) {
        self.vias.push(v);
    }

    pub fn pin_ref(&self, p: &PinRef) -> Result<(&Component, &Pin)> {
        let component = self
            .component(&p.component)
            .ok_or_else(|| eyre!("unknown component id {}", p.component))?;
        let pin = component
            .pin(&p.pin)
            .ok_or_else(|| eyre!("unknown pin id {} on component {}", p.pin, p.component))?;
        Ok((component, pin))
    }
}
