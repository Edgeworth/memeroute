use std::collections::HashMap;

use eyre::{eyre, Result};

use crate::model::pcb::{Id, Padstack, Pcb, Pin, PinRef, Shape, Side, Wire, ANY_LAYER};
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
pub struct GridRouter {
    pcb: Pcb,
    net_order: Vec<Id>,
    resolution: f64, // Resolution in mm.
    blocked: HashMap<State, i64>,
    wire_test: Vec<Wire>,
}

impl GridRouter {
    pub fn new(pcb: Pcb, net_order: Vec<Id>) -> Self {
        let mut s = Self {
            pcb,
            net_order,
            resolution: 0.8,
            blocked: HashMap::new(),
            wire_test: Vec::new(),
        };
        s.mark_blocked();
        s
    }

    fn mark_shape(&self, blocked: &mut HashMap<State, i64>, count: i64, tf: &Tf, s: &Shape) {
        let shape = tf.shape(&s.shape);
        let bounds = self.grid_rt(&shape.bounds());

        for l in bounds.l()..bounds.r() {
            for b in bounds.b()..bounds.t() {
                let p = PtI::new(l, b);
                let r = self.grid_square_in_world(p);
                if shape.intersects(&ShapeType::Rect(r)) {
                    *blocked.entry(State { p, layer: s.layer.clone() }).or_insert(0) += count;
                }
            }
        }
    }

    fn mark_padstack(
        &self,
        blocked: &mut HashMap<State, i64>,
        count: i64,
        tf: &Tf,
        padstack: &Padstack,
    ) {
        for shape in padstack.shapes.iter() {
            self.mark_shape(blocked, count, tf, shape);
        }
    }

    fn mark_pin(&self, blocked: &mut HashMap<State, i64>, count: i64, tf: &Tf, pin: &Pin) {
        self.mark_padstack(blocked, count, &(tf * pin.tf()), &pin.padstack);
    }

    fn mark_blocked(&mut self) {
        let mut blocked = HashMap::new();
        for keepout in self.pcb.keepouts() {
            // TODO: Handle only via vs only wire keepout.
            self.mark_shape(&mut blocked, 1, &Tf::identity(), &keepout.shape);
        }

        let mut wires = Vec::new();
        for c in self.pcb.components() {
            let tf = c.tf();
            for pin in c.pins() {
                self.mark_pin(&mut blocked, 1, &tf, pin);
            }
            for keepout in c.keepouts.iter() {
                // TODO: Handle only via vs only wire keepout.
                self.mark_shape(&mut blocked, 1, &tf, &keepout.shape);
            }
        }
        self.blocked = blocked;
        self.wire_test = wires;
    }

    // Checks if the state |s| is routable (inside boundary, outside of
    // keepouts, etc).
    fn is_blocked(&self, s: &State) -> Result<bool> {
        // TODO: Check which layer the boundary is for.
        let r = self.grid_square_in_world(s.p);
        if !self.pcb.rt_in_boundary(&r) {
            return Ok(true);
        }

        if *self.blocked.get(s).unwrap_or(&0) > 0 {
            return Ok(true);
        }

        if *self.blocked.get(&State { p: s.p, layer: ANY_LAYER.to_owned() }).unwrap_or(&0) > 0 {
            return Ok(true);
        }

        Ok(false)
    }

    // Connect the given states together and return a route result doing that.
    fn connect(&mut self, _states: Vec<State>) -> Result<RouteResult> {
        let res = RouteResult::default();
        // dijkstra(graph, start, goal, edge_cost)
        Ok(res)
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

impl RouteStrategy for GridRouter {
    fn route(&mut self) -> Result<RouteResult> {
        let mut res = RouteResult::default();
        for net_id in self.net_order.clone().into_iter() {
            let net = self.pcb.net(&net_id).ok_or_else(|| eyre!("missing net {}", net_id))?;
            let states = net.pins.iter().map(|p| self.pin_ref_state(p)).collect::<Result<_>>()?;
            res.merge(self.connect(states)?);
        }
        let bounds = self.grid_rt(&self.pcb.bounds());
        for l in bounds.l()..bounds.r() {
            for b in bounds.b()..bounds.t() {
                let p = PtI::new(l, b);
                if self.is_blocked(&State { p, layer: "F.Cu".to_owned() })? {
                    continue;
                }
                let shape =
                    ShapeType::Circle(Circle::new(self.world_pt_mid(p), self.resolution / 2.0));
                res.wires.push(Wire { shape: Shape { layer: "F.Cu".to_owned(), shape } })
            }
        }
        res.wires.extend(self.wire_test.clone());
        Ok(res)
    }
}
