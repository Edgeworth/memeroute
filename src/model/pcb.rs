use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::iter::FromIterator;

use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use eyre::{eyre, Result};
use rust_dense_bitset::{BitSet, DenseBitSet};

use crate::model::geom::bounds::rt_cloud_bounds;
use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::{pt, ShapeOps};
use crate::model::tf::Tf;

// File-format independent representation of a PCB.
// Units are in millimetres.
// All rotations are in degrees, counterclockwise from the positive x axis.

pub type LayerId = u8;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum LayerKind {
    All,
    Signal,
    Power,
    Mixed,
    Jumper,
}

// Support up to 64 layers.
#[derive(Debug, Default, Hash, PartialEq, Eq, Copy, Clone)]
pub struct LayerSet {
    l: DenseBitSet,
}

impl_op_ex!(| |a: &LayerSet, b: &LayerSet| -> LayerSet {LayerSet {l: a.l | b.l}});
impl_op_ex_commutative!(| |a: &LayerSet, b: &LayerId| -> LayerSet {let mut copy = *a; copy |= b; copy});
impl_op_ex!(|= |a: &mut LayerSet, b: &LayerSet| {a.l |= b.l;});
impl_op_ex!(|= |a: &mut LayerSet, b: &LayerId| {a.l.set_bit(*b as usize, true);});
impl_op_ex!(&|a: &LayerSet, b: &LayerSet| -> LayerSet { LayerSet { l: a.l & b.l } });
impl_op_ex!(&= |a: &mut LayerSet, b: &LayerSet| {a.l &= b.l;});

impl LayerSet {
    pub fn empty() -> Self {
        Self { l: DenseBitSet::new() }
    }

    pub fn one(id: LayerId) -> Self {
        Self { l: DenseBitSet::from_integer(1 << (id as u64)) }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.l.get_weight() as usize
    }

    pub fn id(&self) -> Option<LayerId> {
        if self.len() == 1 { Some(self.l.first_set() as LayerId) } else { None }
    }

    pub fn contains(&self, layer: LayerId) -> bool {
        self.l.get_bit(layer as usize)
    }

    pub fn contains_set(&self, layers: LayerSet) -> bool {
        (self.l | layers.l) == self.l
    }

    pub fn iter(&self) -> BitSetIterator {
        BitSetIterator::new(self.l)
    }

    pub fn remove(&mut self, layer: LayerId) {
        self.l.set_bit(layer as usize, false);
    }
}

impl FromIterator<LayerId> for LayerSet {
    fn from_iter<T: IntoIterator<Item = LayerId>>(iter: T) -> Self {
        iter.into_iter().fold(LayerSet::empty(), |a, b| a | b)
    }
}

impl FromIterator<LayerSet> for LayerSet {
    fn from_iter<T: IntoIterator<Item = LayerSet>>(iter: T) -> Self {
        iter.into_iter().fold(LayerSet::empty(), |a, b| a | b)
    }
}

pub struct BitSetIterator {
    l: DenseBitSet,
}

impl BitSetIterator {
    pub fn new(l: DenseBitSet) -> Self {
        Self { l }
    }
}

impl Iterator for BitSetIterator {
    type Item = LayerId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.l.any() {
            let id = self.l.first_set();
            self.l.set_bit(id as usize, false);
            Some(id as LayerId)
        } else {
            None
        }
    }
}

pub type Id = String;

#[derive(Debug, Clone)]
pub struct LayerShape {
    pub layers: LayerSet,
    pub shape: Shape,
}

// Keepout: No routing whatsoever.
// ViaKeepout: No vias.
// WireKeepout: No wires.
#[derive(Debug, Clone)]
pub enum KeepoutType {
    Keepout,
    ViaKeepout,
    WireKeepout,
}

// Describes a keepout area.
#[derive(Debug, Clone)]
pub struct Keepout {
    pub kind: KeepoutType,
    pub shape: LayerShape,
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
#[derive(Debug, Default, Clone)]
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
#[derive(Debug, Default, Clone)]
pub struct Component {
    pub id: Id,
    pub p: Pt,
    pub side: Side,
    pub rotation: f64,
    pub outlines: Vec<LayerShape>,
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
            if self.side == Side::Back { Tf::scale(pt(-1.0, -1.0)) } else { Tf::identity() };
        Tf::translate(self.p) * Tf::rotate(self.rotation) * side_tf
    }
}

// Describes a padstack.
#[derive(Debug, Default, Clone)]
pub struct Padstack {
    pub id: Id,
    pub shapes: Vec<LayerShape>,
    pub attach: bool,
}

impl Padstack {
    pub fn layers(&self) -> LayerSet {
        self.shapes.iter().map(|s| s.layers).collect()
    }
}

// Describes a layer in a PCB.
#[derive(Debug, Clone)]
pub struct Layer {
    pub name: Id,
    pub id: LayerId,
    pub kind: LayerKind,
}

#[derive(Debug, Hash, PartialEq, Eq, Default, Clone)]
pub struct PinRef {
    pub component: Id,
    pub pin: Id,
}

impl PinRef {
    pub fn new(component: &Component, pin: &Pin) -> Self {
        Self { component: component.id.clone(), pin: pin.id.clone() }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Net {
    pub id: Id,
    pub pins: Vec<PinRef>,
}

// Describes a route.
#[derive(Debug, Clone)]
pub struct Wire {
    pub shape: LayerShape,
    pub net_id: Id,
}

// Describes a via.
#[derive(Debug, Clone)]
pub struct Via {
    pub p: Pt,
    pub padstack: Padstack,
    pub net_id: Id,
}

impl Via {
    pub fn tf(&self) -> Tf {
        Tf::translate(self.p)
    }
}

// Describes an overall PCB.
#[derive(Debug, Default, Clone)]
pub struct Pcb {
    id: Id,

    // Physical structure:
    layers: Vec<Layer>,
    boundaries: Vec<LayerShape>,
    keepouts: Vec<Keepout>,
    via_padstacks: Vec<Padstack>, // Types of vias available to use.
    components: HashMap<Id, Component>,

    // Routing:
    wires: Vec<Wire>,
    vias: Vec<Via>,
    nets: HashMap<Id, Net>,
    pin_ref_net: HashMap<PinRef, Id>,

    // Debug:
    debug_rts: Vec<Rt>,
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

    pub fn layers_by_kind(&self, kind: LayerKind) -> LayerSet {
        if kind == LayerKind::All {
            self.layers().iter().map(|v| v.id).collect()
        } else {
            self.layers().iter().filter(|l| l.kind == kind).map(|v| v.id).collect()
        }
    }

    pub fn add_boundary(&mut self, s: LayerShape) {
        self.boundaries.push(s);
    }

    pub fn boundaries(&self) -> &[LayerShape] {
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

    pub fn add_wire(&mut self, w: Wire) {
        self.wires.push(w);
    }

    pub fn wires(&self) -> &[Wire] {
        &self.wires
    }

    pub fn add_via(&mut self, v: Via) {
        self.vias.push(v);
    }

    pub fn vias(&self) -> &[Via] {
        &self.vias
    }

    pub fn add_net(&mut self, n: Net) {
        for p in n.pins {
            self.pin_ref_net.insert(p.clone(), n.id.clone())
        }
        self.nets.insert(n.id.clone(), n);
    }

    pub fn nets(&self) -> Values<'_, Id, Net> {
        self.nets.values()
    }

    pub fn net(&self, id: &str) -> Option<&Net> {
        self.nets.get(id)
    }

    pub fn add_debug_rt(&mut self, r: Rt) {
        self.debug_rts.push(r);
    }

    pub fn debug_rts(&self) -> &[Rt] {
        &self.debug_rts
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

    pub fn pin_ref_net(&self, p: &PinRef) -> Option<Id> {
        self.pin_ref_net.get(p)
    }

    pub fn bounds(&self) -> Rt {
        // Assumes boundaries are valid.
        rt_cloud_bounds(self.boundaries().iter().map(|v| v.shape.bounds()))
    }
}
