use std::collections::HashMap;

use eyre::Result;

use crate::model::geom::quadtree::{Query, ShapeIdx, Tag, NO_TAG};
use crate::model::pcb::{
    LayerId, LayerSet, LayerShape, Net, Padstack, Pcb, Pin, PinRef, Via, Wire,
};
use crate::model::primitive::compound::Compound;
use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::{path, ShapeOps};
use crate::model::tf::Tf;
use crate::name::Id;

pub type PlaceId = (LayerId, ShapeIdx);

// Need to handle:
// but also keeping them for hole drils
#[derive(Debug, Default, Clone)]
pub struct PlaceModel {
    pcb: Pcb,
    boundary: HashMap<LayerId, Compound>,
    blocked: HashMap<LayerId, Compound>,
    pins: HashMap<PinRef, Vec<PlaceId>>, // Record which pins correspond to which place ids in |blocked|.
    bounds: Rt,
}

impl PlaceModel {
    pub fn new(pcb: Pcb) -> Self {
        let mut m = Self {
            pcb: Pcb::default(), // Initially set as empty since we will initialise.
            boundary: HashMap::new(),
            blocked: HashMap::new(),
            pins: HashMap::new(),
            bounds: Rt::empty(),
        };
        m.init(pcb);
        m
    }

    pub fn debug_rts(&self) -> Vec<Rt> {
        // 0 = F.Cu, 1 = B.Cu
        self.blocked.get(&1).unwrap().quadtree().rts()
    }

    pub fn pcb(&self) -> &Pcb {
        &self.pcb
    }

    // Creates a wire for a given net, but doesn't add it.
    pub fn create_wire(&self, net_id: Id, layer: u8, pts: &[Pt]) -> Wire {
        let rs = self.pcb.net_ruleset(net_id);
        let shape =
            LayerShape { layers: LayerSet::one(layer), shape: path(pts, rs.radius()).shape() };
        Wire { shape, net_id }
    }

    pub fn add_wire(&mut self, wire: &Wire) -> Vec<PlaceId> {
        Self::add_shape(self.bounds, &mut self.blocked, &Tf::identity(), &wire.shape, wire.net_id)
    }

    // Creates a via for a given net, but doesn't add it.
    pub fn create_via(&self, net_id: Id, p: Pt) -> Via {
        // TODO: consult ruleset.
        Via { padstack: self.pcb.via_padstacks()[0].clone(), p, net_id }
    }

    pub fn add_via(&mut self, via: &Via) -> Vec<PlaceId> {
        self.add_padstack(&via.tf(), &via.padstack, via.net_id)
    }

    // Adds all pins in the given net.
    pub fn add_net(&mut self, pcb: &Pcb, net: &Net) -> Result<()> {
        for p in net.pins.iter() {
            let (component, pin) = pcb.pin_ref(p)?;
            self.add_pin(&component.tf(), p.clone(), pin, net.id);
        }
        Ok(())
    }

    // Removes all pins in the given net.
    pub fn remove_net(&mut self, net: &Net) {
        for p in net.pins.iter() {
            self.remove_pin(p);
        }
    }

    pub fn is_wire_blocked(&self, wire: &Wire, q: Query) -> bool {
        self.is_shape_blocked(&Tf::identity(), &wire.shape, q)
    }

    pub fn is_via_blocked(&self, via: &Via, q: Query) -> bool {
        self.is_padstack_blocked(&via.tf(), &via.padstack, q)
    }

    pub fn is_shape_blocked(&self, tf: &Tf, ls: &LayerShape, q: Query) -> bool {
        let s = tf.shape(&ls.shape);

        for layer in ls.layers.iter() {
            if let Some(boundary) = self.boundary.get(&layer) {
                if !boundary.contains(&s, q) {
                    return true;
                }
            }
        }

        for layer in ls.layers.iter() {
            if let Some(blocked) = self.blocked.get(&layer) {
                if blocked.intersects(&s, q) {
                    return true;
                }
            }
        }

        false
    }

    fn init(&mut self, pcb: Pcb) {
        let tf = Tf::identity();

        self.bounds = self.bounds.united(&pcb.bounds());
        for boundary in pcb.boundaries() {
            Self::add_shape(self.bounds, &mut self.boundary, &tf, boundary, NO_TAG);
        }

        for wire in pcb.wires() {
            self.add_wire(wire);
        }
        for via in pcb.vias() {
            self.add_via(via);
        }
        for keepout in pcb.keepouts() {
            Self::add_shape(self.bounds, &mut self.blocked, &tf, &keepout.shape, NO_TAG);
        }

        for c in pcb.components() {
            let tf = tf * c.tf();
            for pin in c.pins() {
                let r = PinRef::new(c, pin);
                let tag = pcb.pin_ref_net(&r).unwrap_or(NO_TAG);
                self.add_pin(&tf, r, pin, tag);
            }
            for keepout in c.keepouts.iter() {
                Self::add_shape(self.bounds, &mut self.blocked, &tf, &keepout.shape, NO_TAG);
            }
        }
        self.pcb = pcb;
    }

    fn add_shape(
        bounds: Rt,
        map: &mut HashMap<LayerId, Compound>,
        tf: &Tf,
        ls: &LayerShape,
        tag: Tag,
    ) -> Vec<PlaceId> {
        let s = tf.shape(&ls.shape);
        let mut idxs = Vec::new();

        for layer in ls.layers.iter() {
            idxs.extend(
                map.entry(layer)
                    .or_insert_with(|| Compound::with_bounds(&bounds))
                    .add_shape(s.clone(), tag)
                    .iter()
                    .map(|&v| (layer, v)),
            );
        }

        idxs
    }

    fn add_padstack(&mut self, tf: &Tf, padstack: &Padstack, tag: Tag) -> Vec<PlaceId> {
        padstack
            .shapes
            .iter()
            .map(|shape| Self::add_shape(self.bounds, &mut self.blocked, tf, shape, tag))
            .flatten()
            .collect()
    }

    fn add_pin(&mut self, tf: &Tf, pinref: PinRef, pin: &Pin, tag: Tag) -> Vec<PlaceId> {
        let ids = self.add_padstack(&(tf * pin.tf()), &pin.padstack, tag);
        let e = self.pins.entry(pinref).or_insert_with(Vec::new);
        for &id in ids.iter() {
            e.push(id);
        }
        ids
    }

    fn remove_pin(&mut self, p: &PinRef) {
        if let Some(ids) = self.pins.remove(p) {
            for id in ids {
                self.remove_shape(id);
            }
        }
    }

    fn remove_shape(&mut self, id: PlaceId) {
        self.blocked.get_mut(&id.0).unwrap().remove_shape(id.1);
    }

    fn is_padstack_blocked(&self, tf: &Tf, padstack: &Padstack, q: Query) -> bool {
        padstack.shapes.iter().any(|shape| self.is_shape_blocked(tf, shape, q))
    }
}
