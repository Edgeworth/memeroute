use std::collections::HashMap;

use eyre::Result;

use crate::model::geom::quadtree::{ShapeIdx, Tag, EMPTY_TAG};
use crate::model::pcb::{LayerId, LayerShape, Net, Padstack, Pcb, Pin, PinRef, Via, Wire};
use crate::model::primitive::compound::Compound;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::ShapeOps;
use crate::model::tf::Tf;

pub type PlaceId = (LayerId, ShapeIdx);

// Need to handle:
// but also keeping them for hole drils
#[derive(Debug, Default, Clone)]
pub struct PlaceModel {
    boundary: HashMap<LayerId, Compound>,
    blocked: HashMap<LayerId, Compound>,
    pins: HashMap<PinRef, Vec<PlaceId>>, // Record which pins correspond to which place ids in |blocked|.
    bounds: Rt,
}

impl PlaceModel {
    pub fn new() -> Self {
        Self {
            boundary: HashMap::new(),
            blocked: HashMap::new(),
            pins: HashMap::new(),
            bounds: Rt::empty(),
        }
    }

    pub fn debug_rts(&self) -> Vec<Rt> {
        // 0 = F.Cu, 1 = B.Cu
        self.blocked.get(&1).unwrap().quadtree().rts()
    }

    pub fn add_pcb(&mut self, pcb: &Pcb) {
        let tf = Tf::identity();

        self.bounds = self.bounds.united(&pcb.bounds());
        for boundary in pcb.boundaries() {
            Self::add_shape(self.bounds, &mut self.boundary, &tf, boundary, EMPTY_TAG);
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
                let tag = pcb.pin_ref_net(pin).unwrap_or_else(|| EMPTY_TAG);
                self.add_pin(&tf, PinRef::new(c, pin), pin, &tag);
            }
            for keepout in c.keepouts.iter() {
                // TODO: Handle only via vs only wire keepout.
                Self::add_shape(self.bounds, &mut self.blocked, &tf, &keepout.shape);
            }
        }
    }

    pub fn add_wire(&mut self, wire: &Wire) -> Vec<PlaceId> {
        Self::add_shape(self.bounds, &mut self.blocked, &Tf::identity(), &wire.shape, &wire.net_id)
    }

    pub fn add_via(&mut self, via: &Via) -> Vec<PlaceId> {
        self.add_padstack(&via.tf(), &via.padstack, &via.net_id)
    }

    fn add_shape(
        bounds: Rt,
        map: &mut HashMap<LayerId, Compound>,
        tf: &Tf,
        ls: &LayerShape,
        tag: &Tag,
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

    fn add_padstack(&mut self, tf: &Tf, padstack: &Padstack, tag: &Tag) -> Vec<PlaceId> {
        padstack
            .shapes
            .iter()
            .map(|shape| Self::add_shape(self.bounds, &mut self.blocked, tf, shape, tag))
            .flatten()
            .collect()
    }

    fn add_pin(&mut self, tf: &Tf, pinref: PinRef, pin: &Pin, tag: &Tag) -> Vec<PlaceId> {
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

    // Adds all pins in the given net.
    pub fn add_net(&mut self, pcb: &Pcb, net: &Net) -> Result<()> {
        for p in net.pins.iter() {
            let (component, pin) = pcb.pin_ref(p)?;
            self.add_pin(&component.tf(), p.clone(), pin, &net.id);
        }
        Ok(())
    }

    // Removes all pins in the given net.
    pub fn remove_net(&mut self, net: &Net) {
        for p in net.pins.iter() {
            self.remove_pin(p);
        }
    }

    fn remove_shape(&mut self, id: PlaceId) {
        self.blocked.get_mut(&id.0).unwrap().remove_shape(id.1);
    }

    pub fn is_shape_blocked(&self, tf: &Tf, ls: &LayerShape, q: TagQuery) -> bool {
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

    pub fn is_wire_blocked(&self, wire: &Wire, q: TagQuery) -> bool {
        self.is_shape_blocked(&Tf::identity(), &wire.shape, q)
    }

    pub fn is_via_blocked(&self, via: &Via, q: TagQuery) -> bool {
        self.is_padstack_blocked(&via.tf(), &via.padstack, q)
    }

    fn is_padstack_blocked(&self, tf: &Tf, padstack: &Padstack, q: TagQuery) -> bool {
        padstack.shapes.iter().any(|shape| self.is_shape_blocked(tf, shape, q))
    }
}
