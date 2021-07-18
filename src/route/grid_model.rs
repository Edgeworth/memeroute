use eyre::Result;

use crate::model::pcb::{LayerShape, Net, Padstack, Pcb, Pin, PinRef, Side, Via, Wire, ANY_LAYER};
use crate::model::primitive::point::{Pt, PtI};
use crate::model::primitive::rect::{Rt, RtI};
use crate::model::primitive::{pt, pti, ShapeOps};
use crate::model::tf::Tf;
use crate::route::grid::{BlockMap, State};

#[derive(Debug, Clone)]
pub struct GridModel {
    pub pcb: Pcb,
    pub resolution: f64, // Resolution in mm.
}

impl GridModel {
    pub fn new(pcb: Pcb) -> Self {
        Self { pcb, resolution: 0.8 }
    }

    pub fn mark_shape(&self, blk: &mut BlockMap, count: i64, tf: &Tf, ls: &LayerShape) {
        let s = tf.shape(&ls.shape);
        let bounds = self.grid_rt(&s.bounds());

        for l in bounds.l()..bounds.r() {
            for b in bounds.b()..bounds.t() {
                let p = pti(l, b);
                let r = self.grid_square_in_world(p);
                if s.intersects(&r.shape()) {
                    *blk.entry(State { p, layer: ls.layer.clone() }).or_insert(0) += count;
                }
            }
        }
    }

    pub fn mark_padstack(&self, blk: &mut BlockMap, count: i64, tf: &Tf, padstack: &Padstack) {
        for shape in padstack.shapes.iter() {
            self.mark_shape(blk, count, tf, shape);
        }
    }

    pub fn mark_wire(&self, blk: &mut BlockMap, count: i64, wire: &Wire) {
        self.mark_shape(blk, count, &Tf::identity(), &wire.shape);
    }

    pub fn mark_via(&self, blk: &mut BlockMap, count: i64, via: &Via) {
        self.mark_padstack(blk, count, &via.tf(), &via.padstack);
    }

    pub fn mark_pin(&self, blk: &mut BlockMap, count: i64, tf: &Tf, pin: &Pin) {
        self.mark_padstack(blk, count, &(tf * pin.tf()), &pin.padstack);
    }

    // Marks all pins in the given net.
    pub fn mark_net(&self, blk: &mut BlockMap, count: i64, net: &Net) -> Result<()> {
        for p in net.pins.iter() {
            let (component, pin) = self.pcb.pin_ref(p)?;
            self.mark_pin(blk, count, &component.tf(), pin);
        }
        Ok(())
    }

    pub fn mark_blocked(&self, blk: &mut BlockMap) {
        let tf = Tf::identity();
        for wire in self.pcb.wires() {
            self.mark_wire(blk, 1, wire);
        }
        for via in self.pcb.vias() {
            self.mark_via(blk, 1, via);
        }
        for keepout in self.pcb.keepouts() {
            // TODO: Handle only via vs only wire keepout.
            self.mark_shape(blk, 1, &tf, &keepout.shape);
        }

        for c in self.pcb.components() {
            let tf = tf * c.tf();
            for pin in c.pins() {
                self.mark_pin(blk, 1, &tf, pin);
            }
            for keepout in c.keepouts.iter() {
                // TODO: Handle only via vs only wire keepout.
                self.mark_shape(blk, 1, &tf, &keepout.shape);
            }
        }
    }

    // Checks if the state |s| is routable (inside boundary, outside of
    // keepouts, etc).
    pub fn is_state_blocked(&self, blk: &BlockMap, s: &State) -> bool {
        // TODO: Check which layer the boundary is for.
        let r = self.grid_square_in_world(s.p);
        if !self.pcb.boundary_contains_rt(&r) {
            return true;
        }

        if *blk.get(s).unwrap_or(&0) > 0 {
            return true;
        }

        // TODO: This hardcodes "signal" as applying to all layers. Might also
        // miss other layer names like pcb/power/etc.
        // Handle keepouts
        if *blk.get(&State { p: s.p, layer: ANY_LAYER.to_owned() }).unwrap_or(&0) > 0 {
            return true;
        }

        false
    }

    pub fn is_shape_blocked(&self, blk: &BlockMap, tf: &Tf, ls: &LayerShape) -> bool {
        let s = tf.shape(&ls.shape);
        let bounds = self.grid_rt(&s.bounds());

        for l in bounds.l()..bounds.r() {
            for b in bounds.b()..bounds.t() {
                let p = pti(l, b);
                let r = self.grid_square_in_world(p);
                if s.intersects(&r.shape())
                    && self.is_state_blocked(blk, &State { p, layer: ls.layer.clone() })
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_padstack_blocked(&self, blk: &BlockMap, tf: &Tf, padstack: &Padstack) -> bool {
        padstack.shapes.iter().any(|shape| self.is_shape_blocked(blk, tf, shape))
    }

    pub fn is_wire_blocked(&self, blk: &BlockMap, wire: &Wire) -> bool {
        self.is_shape_blocked(blk, &Tf::identity(), &wire.shape)
    }

    pub fn is_via_blocked(&self, blk: &BlockMap, via: &Via) -> bool {
        self.is_padstack_blocked(blk, &via.tf(), &via.padstack)
    }

    // TODO: Assumes connect to the center of the pin. Look at padstack instead.
    pub fn pin_ref_state(&self, pin_ref: &PinRef) -> Result<State> {
        let (component, pin) = self.pcb.pin_ref(pin_ref)?;
        let p = self.grid_pt((component.tf() * pin.tf()).pt(Pt::zero()));
        // TODO: Using component side for which layer is broken. Need to look at
        // padstack.
        let layer = match component.side {
            Side::Front => "F.Cu".to_owned(),
            Side::Back => "B.Cu".to_owned(),
        };
        Ok(State { p, layer })
    }

    pub fn grid_pt(&self, p: Pt) -> PtI {
        // Map points to the lower left corner.
        pti((p.x / self.resolution).floor() as i64, (p.y / self.resolution).floor() as i64)
    }

    pub fn grid_rt(&self, r: &Rt) -> RtI {
        RtI::enclosing(self.grid_pt(r.bl()), self.grid_pt(r.tr()) + pti(1, 1))
    }

    pub fn grid_square_in_world(&self, p: PtI) -> Rt {
        Rt::enclosing(self.world_pt(p), self.world_pt(pti(p.x + 1, p.y + 1)))
    }

    pub fn world_pt(&self, p: PtI) -> Pt {
        pt(p.x as f64 * self.resolution, p.y as f64 * self.resolution)
    }

    pub fn world_pt_mid(&self, p: PtI) -> Pt {
        self.world_pt(p) + pt(self.resolution / 2.0, self.resolution / 2.0)
    }
}
