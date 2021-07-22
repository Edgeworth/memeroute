use std::collections::HashMap;

use eyre::Result;

use crate::model::geom::quadtree::ShapeIdx;
use crate::model::pcb::{Id, LayerShape, Net, Padstack, Pcb, Pin, Via, Wire, ANY_LAYER};
use crate::model::primitive::compound::Compound;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::ShapeOps;
use crate::model::tf::Tf;

pub type PlaceId = (String, ShapeIdx);

// Need to handle:
// but also keeping them for hole drils
#[derive(Debug, Clone)]
pub struct PlaceModel {
    boundary: HashMap<Id, Compound>,
    blocked: HashMap<Id, Compound>,
    bounds: Rt,
}

impl PlaceModel {
    pub fn new() -> Self {
        Self { boundary: HashMap::new(), blocked: HashMap::new(), bounds: Rt::empty() }
    }

    pub fn add_pcb(&mut self, pcb: &Pcb) {
        let tf = Tf::identity();

        self.bounds = self.bounds.united(&pcb.bounds());
        for boundary in pcb.boundaries() {
            Self::add_shape(self.bounds, &mut self.boundary, &tf, boundary);
        }

        for wire in pcb.wires() {
            self.add_wire(wire);
        }
        for via in pcb.vias() {
            self.add_via(via);
        }
        for keepout in pcb.keepouts() {
            // TODO: Handle only via vs only wire keepout.
            Self::add_shape(self.bounds, &mut self.blocked, &tf, &keepout.shape);
        }

        for c in pcb.components() {
            let tf = tf * c.tf();
            for pin in c.pins() {
                self.add_pin(&tf, pin);
            }
            for keepout in c.keepouts.iter() {
                // TODO: Handle only via vs only wire keepout.
                Self::add_shape(self.bounds, &mut self.blocked, &tf, &keepout.shape);
            }
        }
    }

    fn add_shape(bounds: Rt, map: &mut HashMap<Id, Compound>, tf: &Tf, ls: &LayerShape) -> PlaceId {
        let s = tf.shape(&ls.shape);
        let idx = map
            .entry(ls.layer.clone())
            .or_insert_with(|| Compound::with_bounds(&bounds))
            .add_shape(s);
        (ls.layer.clone(), idx)
    }

    pub fn add_padstack(&mut self, tf: &Tf, padstack: &Padstack) -> Vec<PlaceId> {
        padstack
            .shapes
            .iter()
            .map(|shape| Self::add_shape(self.bounds, &mut self.blocked, tf, shape))
            .collect()
    }

    pub fn add_wire(&mut self, wire: &Wire) -> PlaceId {
        Self::add_shape(self.bounds, &mut self.blocked, &Tf::identity(), &wire.shape)
    }

    pub fn add_via(&mut self, via: &Via) -> Vec<PlaceId> {
        self.add_padstack(&via.tf(), &via.padstack)
    }

    pub fn add_pin(&mut self, tf: &Tf, pin: &Pin) -> Vec<PlaceId> {
        self.add_padstack(&(tf * pin.tf()), &pin.padstack)
    }

    // Marks all pins in the given net.
    pub fn add_net(&mut self, pcb: &Pcb, net: &Net) -> Result<Vec<PlaceId>> {
        let mut ids = Vec::new();
        for p in net.pins.iter() {
            let (component, pin) = pcb.pin_ref(p)?;
            ids.extend(self.add_pin(&component.tf(), pin));
        }
        Ok(ids)
    }

    pub fn remove_shape(&mut self, id: PlaceId) {
        self.blocked.get_mut(&id.0).unwrap().remove_shape(id.1);
    }

    pub fn is_shape_blocked(&self, tf: &Tf, ls: &LayerShape) -> bool {
        let s = tf.shape(&ls.shape);

        for layer in [&ls.layer, "pcb"] {
            if let Some(boundary) = self.boundary.get(layer) {
                if !boundary.contains_shape(&s) {
                    return true;
                }
            }
        }

        for layer in [&ls.layer, ANY_LAYER] {
            if let Some(blocked) = self.blocked.get(layer) {
                if blocked.intersects_shape(&s) {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_padstack_blocked(&self, tf: &Tf, padstack: &Padstack) -> bool {
        padstack.shapes.iter().any(|shape| self.is_shape_blocked(tf, shape))
    }

    pub fn is_wire_blocked(&self, wire: &Wire) -> bool {
        self.is_shape_blocked(&Tf::identity(), &wire.shape)
    }

    pub fn is_via_blocked(&self, via: &Via) -> bool {
        self.is_padstack_blocked(&via.tf(), &via.padstack)
    }
}
