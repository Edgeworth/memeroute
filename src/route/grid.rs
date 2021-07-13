use std::collections::HashMap;

use eyre::{eyre, Result};

use crate::model::pcb::{Id, Net, Padstack, Pcb, Pin, PinRef, Shape, Side, Wire, ANY_LAYER};
use crate::model::pt::{Pt, PtI};
use crate::model::shape::circle::Circle;
use crate::model::shape::rt::{Rt, RtI};
use crate::model::shape::shape_type::ShapeType;
use crate::model::tf::Tf;
use crate::route::router::{RouteResult, RouteStrategy};

const DIR: [(PtI, f32); 8] = [
    (PtI::new(-1, 0), 1.0),
    (PtI::new(1, 0), 1.0),
    (PtI::new(0, -1), 1.0),
    (PtI::new(0, 1), 1.0),
    (PtI::new(1, 1), 1.414),
    (PtI::new(1, -1), 1.414),
    (PtI::new(-1, 1), 1.414),
    (PtI::new(-1, -1), 1.414),
];

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct State {
    p: PtI,
    layer: Id,
}

#[derive(Debug, Clone)]
struct GridModel {
    pcb: Pcb,
    resolution: f64, // Resolution in mm.
}

impl GridModel {
    pub fn new(pcb: Pcb) -> Self {
        Self { pcb, resolution: 0.5 }
    }

    fn mark_shape(&self, blk: &mut BlockMap, count: i64, tf: &Tf, s: &Shape) {
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

    fn mark_padstack(&self, blk: &mut BlockMap, count: i64, tf: &Tf, padstack: &Padstack) {
        for shape in padstack.shapes.iter() {
            self.mark_shape(blk, count, tf, shape);
        }
    }

    fn mark_pin(&self, blk: &mut BlockMap, count: i64, tf: &Tf, pin: &Pin) {
        self.mark_padstack(blk, count, &(tf * pin.tf()), &pin.padstack);
    }

    // Marks all pins in the given net.
    fn mark_net(&self, blk: &mut BlockMap, count: i64, net: &Net) -> Result<()> {
        for p in net.pins.iter() {
            let (component, pin) = self.pcb.pin_ref(p)?;
            self.mark_pin(blk, count, &component.tf(), pin);
        }
        Ok(())
    }

    fn mark_blocked(&self, blk: &mut BlockMap) {
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
    fn is_blocked(&self, blk: &BlockMap, s: &State) -> Result<bool> {
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
    fn pin_ref_state(&self, pin_ref: &PinRef) -> Result<State> {
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

    fn grid_pt(&self, p: Pt) -> PtI {
        // Map points to the lower left corner.
        PtI::new((p.x / self.resolution).floor() as i64, (p.y / self.resolution).floor() as i64)
    }

    fn grid_rt(&self, r: &Rt) -> RtI {
        RtI::enclosing(self.grid_pt(r.bl()), self.grid_pt(r.tr()) + PtI::new(1, 1))
    }

    fn grid_square_in_world(&self, p: PtI) -> Rt {
        Rt::enclosing(self.world_pt(p), self.world_pt(PtI::new(p.x + 1, p.y + 1)))
    }

    fn world_pt(&self, p: PtI) -> Pt {
        Pt::new(p.x as f64 * self.resolution, p.y as f64 * self.resolution)
    }

    fn world_pt_mid(&self, p: PtI) -> Pt {
        self.world_pt(p) + Pt::new(self.resolution / 2.0, self.resolution / 2.0)
    }
}

type BlockMap = HashMap<State, i64>;

#[derive(Debug, Clone)]
pub struct GridRouter {
    model: GridModel,
    net_order: Vec<Id>,
    blk: BlockMap,
}

impl GridRouter {
    pub fn new(pcb: Pcb, net_order: Vec<Id>) -> Self {
        let mut s = Self { model: GridModel::new(pcb), net_order, blk: BlockMap::new() };
        s.model.mark_blocked(&mut s.blk);
        s
    }

    // Connect the given states together and return a route result doing that.
    fn connect(&self, _states: Vec<State>) -> Result<RouteResult> {
        let res = RouteResult::default();
        // dijkstra(graph, start, goal, edge_cost)
        Ok(res)
    }
}

impl RouteStrategy for GridRouter {
    fn route(&mut self) -> Result<RouteResult> {
        let mut res = RouteResult::default();
        for net_id in self.net_order.clone().into_iter() {
            let net = self.model.pcb.net(&net_id).ok_or_else(|| eyre!("missing net {}", net_id))?;
            let states =
                net.pins.iter().map(|p| self.model.pin_ref_state(p)).collect::<Result<_>>()?;

            self.model.mark_net(&mut self.blk, -1, net)?; // Temporarily remove pins as blocking.
            res.merge(self.connect(states)?);
            self.model.mark_net(&mut self.blk, 1, net)?; // Add them back.
        }

        let bounds = self.model.grid_rt(&self.model.pcb.bounds());
        for l in bounds.l()..bounds.r() {
            for b in bounds.b()..bounds.t() {
                let p = PtI::new(l, b);
                if self.model.is_blocked(&self.blk, &State { p, layer: "F.Cu".to_owned() })? {
                    continue;
                }
                let shape = ShapeType::Circle(Circle::new(
                    self.model.world_pt_mid(p),
                    self.model.resolution / 2.0,
                ));
                res.wires.push(Wire { shape: Shape { layer: "F.Cu".to_owned(), shape } })
            }
        }
        Ok(res)
    }
}
