use eyre::Result;

use crate::model::pcb::{Id, Net, Padstack, Pcb, Pin, PinRef, Shape, Side, Wire, ANY_LAYER};
use crate::model::pt::{Pt, PtI};
use crate::model::shape::rt::{Rt, RtI};
use crate::model::shape::shape_type::ShapeType;
use crate::model::tf::Tf;
use crate::route::grid::{BlockMap, State};

#[derive(Debug, Clone)]
pub struct GridModel {
    pub pcb: Pcb,
    pub resolution: f64, // Resolution in mm.
}

impl GridModel {
    pub fn new(pcb: Pcb) -> Self {
        Self { pcb, resolution: 0.5 }
    }

    pub fn mark_shape(&self, blk: &mut BlockMap, count: i64, tf: &Tf, s: &Shape) {
        let shape = tf.shape(&s.shape);
        let bounds = self.grid_rt(&shape.bounds());

        for l in bounds.l()..bounds.r() {
            for b in bounds.b()..bounds.t() {
                let p = PtI::new(l, b);
                let r = self.grid_square_in_world(p);
                if shape.intersects(&ShapeType::Rect(r)) {
                    *blk.entry(State { p, layer: s.layer.clone() }).or_insert(0) += count;
                }
            }
        }
    }

    pub fn mark_padstack(&self, blk: &mut BlockMap, count: i64, tf: &Tf, padstack: &Padstack) {
        for shape in padstack.shapes.iter() {
            self.mark_shape(blk, count, tf, shape);
        }
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
        for keepout in self.pcb.keepouts() {
            // TODO: Handle only via vs only wire keepout.
            self.mark_shape(blk, 1, &Tf::identity(), &keepout.shape);
        }

        for c in self.pcb.components() {
            let tf = c.tf();
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
    pub fn is_blocked(&self, blk: &BlockMap, s: &State) -> Result<bool> {
        // TODO: Check which layer the boundary is for.
        let r = self.grid_square_in_world(s.p);
        if !self.pcb.boundary_contains_rt(&r) {
            return Ok(true);
        }

        if *blk.get(s).unwrap_or(&0) > 0 {
            return Ok(true);
        }

        if *blk.get(&State { p: s.p, layer: ANY_LAYER.to_owned() }).unwrap_or(&0) > 0 {
            return Ok(true);
        }

        Ok(false)
    }


    // TODO: Assumes connect to the center of the pin. Look at padstack instead.
    pub fn pin_ref_state(&self, pin_ref: &PinRef) -> Result<State> {
        let (component, pin) = self.pcb.pin_ref(pin_ref)?;
        let p = self.grid_pt((component.tf() * pin.tf()).pt(pin.p));
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
        PtI::new((p.x / self.resolution).floor() as i64, (p.y / self.resolution).floor() as i64)
    }

    pub fn grid_rt(&self, r: &Rt) -> RtI {
        RtI::enclosing(self.grid_pt(r.bl()), self.grid_pt(r.tr()) + PtI::new(1, 1))
    }

    pub fn grid_square_in_world(&self, p: PtI) -> Rt {
        Rt::enclosing(self.world_pt(p), self.world_pt(PtI::new(p.x + 1, p.y + 1)))
    }

    pub fn world_pt(&self, p: PtI) -> Pt {
        Pt::new(p.x as f64 * self.resolution, p.y as f64 * self.resolution)
    }

    pub fn world_pt_mid(&self, p: PtI) -> Pt {
        self.world_pt(p) + Pt::new(self.resolution / 2.0, self.resolution / 2.0)
    }
}
