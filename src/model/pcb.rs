use std::collections::hash_map::Values;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::RwLock;

use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use enumset::{enum_set, EnumSet, EnumSetType};
use eyre::{eyre, Result};
use rust_dense_bitset::{BitSet, DenseBitSet};
use strum::EnumIter;

use crate::model::geom::bounds::rt_cloud_bounds;
use crate::model::geom::qt::query::QueryKinds;
use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::{pt, ShapeOps};
use crate::model::tf::Tf;
use crate::name::{Id, NameMap};

// File-format independent representation of a PCB.
// Units are in millimetres.
// All rotations are in degrees, counterclockwise from the positive x axis.

pub type LayerId = usize;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, EnumIter)]
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

    pub fn first(&self) -> Option<LayerId> {
        if !self.is_empty() { Some(self.l.first_set() as LayerId) } else { None }
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

    // Flips layers, e.g. moving a component from front to back.
    // This is based on the assumption that layers are in physical order.
    pub fn flip(&mut self, num_layers: usize) {
        self.l = self.l.reverse();
        self.l >>= 64 - num_layers;
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

// Describes a layer in a PCB. Layers should be numbered from 0 up, contiguously.
// Layers should be in order of physical stackup.
#[derive(Debug, Clone)]
pub struct Layer {
    pub name_id: Id,
    pub layer_id: LayerId, // Should be less than 64.
    pub kind: LayerKind,
}

#[derive(Debug, Clone)]
pub struct LayerShape {
    pub layers: LayerSet,
    pub shape: Shape,
}

impl LayerShape {
    pub fn flip(&mut self, num_layers: usize) {
        self.layers.flip(num_layers);
    }
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

impl Keepout {
    pub fn flip(&mut self, num_layers: usize) {
        self.shape.flip(num_layers);
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

    pub fn flip(&mut self, num_layers: usize) {
        self.padstack.flip(num_layers);
    }
}

// Describes a component at a location.
#[derive(Debug, Default, Clone)]
pub struct Component {
    pub id: Id,
    // Id of the footprint for this component. Only used in exporting currently.
    pub footprint_id: Id,
    pub p: Pt,
    pub rotation: f64,
    pub outlines: Vec<LayerShape>,
    pub keepouts: Vec<Keepout>,
    pins: HashMap<Id, Pin>,
    flipped: bool,
}

impl Component {
    pub fn add_pin(&mut self, p: Pin) {
        self.pins.insert(p.id, p);
    }

    pub fn pins(&self) -> Values<'_, Id, Pin> {
        self.pins.values()
    }

    pub fn pin(&self, id: Id) -> Option<&Pin> {
        self.pins.get(&id)
    }

    pub fn tf(&self) -> Tf {
        // Being on the back mirrors, i.e. horizontal flip.
        let side_tf = if self.flipped { Tf::scale(pt(-1.0, 1.0)) } else { Tf::identity() };
        Tf::translate(self.p) * Tf::rotate(self.rotation) * side_tf
    }

    pub fn flip(&mut self, num_layers: usize) {
        self.flipped = !self.flipped;
        for v in self.outlines.iter_mut() {
            v.flip(num_layers);
        }
        for v in self.keepouts.iter_mut() {
            v.flip(num_layers);
        }
        for v in self.pins.values_mut() {
            v.flip(num_layers);
        }
    }

    pub fn flipped(&self) -> bool {
        self.flipped
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

    pub fn flip(&mut self, num_layers: usize) {
        for v in self.shapes.iter_mut() {
            v.flip(num_layers);
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Default, Clone)]
pub struct PinRef {
    pub component: Id,
    pub pin: Id,
}

impl PinRef {
    pub fn new(component: &Component, pin: &Pin) -> Self {
        Self { component: component.id, pin: pin.id }
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

// Object kinds
#[derive(Debug, EnumSetType, EnumIter)]
pub enum ObjectKind {
    Area, // Keepout, boundary, or conducting shapes (fills)
    Pin,  // Through hole pin objects
    Smd,  // Surface mount pad shapes
    Via,  // Vias
    Wire, // Wires
}

impl ObjectKind {
    pub fn query(&self) -> QueryKinds {
        QueryKinds(DenseBitSet::from_integer(enum_set!(self).as_u64()))
    }
}

// If there are multple clearances specified for a ClearanceType,
// take the most specific clearance, defined by the fewest number of ClearanceTypes.
// If there are multiple such clearances, take the one with the largest value.
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Clearance {
    amount: f64,
    area_kinds: EnumSet<ObjectKind>,
    pin_kinds: EnumSet<ObjectKind>,
    smd_kinds: EnumSet<ObjectKind>,
    via_kinds: EnumSet<ObjectKind>,
    wire_kinds: EnumSet<ObjectKind>,
}

impl Clearance {
    pub fn new(amount: f64, pairs: &[(ObjectKind, ObjectKind)]) -> Self {
        let mut c = Self { amount, ..Self::default() };
        for &(a, b) in pairs {
            c.subset_for_mut(a).insert(b);
            c.subset_for_mut(b).insert(a);
        }
        c
    }

    // Returns set of ObjectKind that |kind| has a clearance rule with.
    pub fn subset_for(&self, kind: ObjectKind) -> EnumSet<ObjectKind> {
        match kind {
            ObjectKind::Area => self.area_kinds,
            ObjectKind::Pin => self.pin_kinds,
            ObjectKind::Smd => self.smd_kinds,
            ObjectKind::Via => self.via_kinds,
            ObjectKind::Wire => self.wire_kinds,
        }
    }

    fn subset_for_mut(&mut self, kind: ObjectKind) -> &mut EnumSet<ObjectKind> {
        match kind {
            ObjectKind::Area => &mut self.area_kinds,
            ObjectKind::Pin => &mut self.pin_kinds,
            ObjectKind::Smd => &mut self.smd_kinds,
            ObjectKind::Via => &mut self.via_kinds,
            ObjectKind::Wire => &mut self.wire_kinds,
        }
    }
}

// Describes various rules for layout of tracks.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Rule {
    Radius(f64),          // e.g. Half-width of track
    Clearance(Clearance), // e.g. Minimum distance between track and via.
    UseVia(Id),           // Use the specified via if this rule applies.
}

// Collection of rules that e.g. may apply to a given net.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleSet {
    pub id: Id,
    rules: Vec<Rule>,
    radius: Option<f64>,
}

impl RuleSet {
    pub fn new(id: Id, rules: Vec<Rule>) -> Result<Self> {
        let mut rs = Self { id, rules, radius: None };
        // Check for consistency:
        for rule in rs.rules.iter() {
            match rule {
                Rule::Radius(r) => {
                    if rs.radius.is_some() {
                        return Err(eyre!("Multple width rules"));
                    } else {
                        rs.radius = Some(*r);
                    }
                }
                Rule::Clearance(_) => {}
                Rule::UseVia(_) => {}
            }
        }

        Ok(rs)
    }

    pub fn radius(&self) -> f64 {
        self.radius.unwrap()
    }
}

// Describes an overall PCB.
#[derive(Debug, Default)]
pub struct Pcb {
    id: Id,
    name_map: RwLock<NameMap>,

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
    pin_ref_to_net: HashMap<PinRef, Id>, // Map PinRef to net ID.

    // Rules:
    rulesets: HashMap<Id, RuleSet>,
    net_to_ruleset: HashMap<Id, Id>,
    default_net_ruleset: Id,

    // Debug:
    debug_rts: Vec<Rt>,
}

impl Clone for Pcb {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            name_map: RwLock::new(self.name_map.read().unwrap().clone()),
            layers: self.layers.clone(),
            boundaries: self.boundaries.clone(),
            keepouts: self.keepouts.clone(),
            via_padstacks: self.via_padstacks.clone(),
            components: self.components.clone(),
            wires: self.wires.clone(),
            vias: self.vias.clone(),
            nets: self.nets.clone(),
            pin_ref_to_net: self.pin_ref_to_net.clone(),
            rulesets: self.rulesets.clone(),
            net_to_ruleset: self.net_to_ruleset.clone(),
            default_net_ruleset: self.default_net_ruleset,
            debug_rts: self.debug_rts.clone(),
        }
    }
}

impl Pcb {
    pub fn to_name(&self, id: Id) -> String {
        self.name_map.read().unwrap().name(id).to_string()
    }

    pub fn to_id(&self, name: &str) -> Id {
        self.name_map.write().unwrap().name_to_id(name)
    }

    pub fn layers_by_kind(&self, kind: LayerKind) -> LayerSet {
        if kind == LayerKind::All {
            self.layers().iter().map(|v| v.layer_id).collect()
        } else {
            self.layers().iter().filter(|l| l.kind == kind).map(|v| v.layer_id).collect()
        }
    }

    pub fn layer_by_id(&self, lid: LayerId) -> &Layer {
        self.layers().iter().find(|l| l.layer_id == lid).unwrap()
    }

    pub fn pin_ref(&self, p: &PinRef) -> Result<(&Component, &Pin)> {
        let component = self
            .component(p.component)
            .ok_or_else(|| eyre!("unknown component id {}", p.component))?;
        let pin = component
            .pin(p.pin)
            .ok_or_else(|| eyre!("unknown pin id {} on component {}", p.pin, p.component))?;
        Ok((component, pin))
    }

    pub fn pin_ref_net(&self, p: &PinRef) -> Option<Id> {
        self.pin_ref_to_net.get(p).copied()
    }

    pub fn bounds(&self) -> Rt {
        // Assumes boundaries are valid.
        rt_cloud_bounds(self.boundaries().iter().map(|v| v.shape.bounds()))
    }
}

// Getting and setting
impl Pcb {
    pub fn set_pcb_name(&mut self, name: &str) {
        self.id = self.to_id(name);
    }

    pub fn pcb_id(&self) -> Id {
        self.id
    }

    pub fn add_ruleset(&mut self, r: RuleSet) {
        self.rulesets.insert(r.id, r);
    }

    pub fn set_default_net_ruleset(&mut self, id: Id) {
        self.default_net_ruleset = id;
    }

    pub fn set_net_ruleset(&mut self, net_id: Id, ruleset_id: Id) {
        self.net_to_ruleset.insert(net_id, ruleset_id);
    }

    pub fn net_ruleset(&self, net_id: Id) -> &RuleSet {
        let ruleset_id = self.net_to_ruleset.get(&net_id).unwrap_or(&self.default_net_ruleset);
        self.rulesets.get(ruleset_id).unwrap()
    }

    pub fn add_layer(&mut self, l: Layer) {
        self.layers.push(l);
    }

    pub fn layers(&self) -> &[Layer] {
        &self.layers
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
        self.components.insert(c.id, c);
    }

    pub fn components(&self) -> Values<'_, Id, Component> {
        self.components.values()
    }

    pub fn component(&self, id: Id) -> Option<&Component> {
        self.components.get(&id)
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
        for p in n.pins.iter() {
            self.pin_ref_to_net.insert(p.clone(), n.id);
        }
        self.nets.insert(n.id, n);
    }

    pub fn nets(&self) -> Values<'_, Id, Net> {
        self.nets.values()
    }

    pub fn net(&self, id: Id) -> Option<&Net> {
        self.nets.get(&id)
    }

    pub fn add_debug_rt(&mut self, r: Rt) {
        self.debug_rts.push(r);
    }

    pub fn debug_rts(&self) -> &[Rt] {
        &self.debug_rts
    }
}
