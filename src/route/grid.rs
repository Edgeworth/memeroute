use std::collections::HashSet;

use eyre::{eyre, Result};
use parry2d_f64::math::Isometry;
use parry2d_f64::query::intersection_test;

use crate::model::pcb::{Id, Pcb, Pin, PinRef, Shape, Side, Wire, ANY_LAYER};
use crate::model::pt::{Pt, PtI};
use crate::model::shape::circle::Circle;
use crate::model::shape::identity;
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
    blocked: HashSet<State>,
    wire_test: Vec<Wire>,
}

impl GridRouter {
    pub fn new(pcb: Pcb, net_order: Vec<Id>) -> Result<Self> {
        let mut s = Self {
            pcb,
            net_order,
            resolution: 1.0,
            blocked: HashSet::new(),
            wire_test: Vec::new(),
        };
        s.mark_blocked()?;
        Ok(s)
    }

    fn mark_blocked_shape(&self, blocked: &mut HashSet<State>, tf: &Tf, s: &Shape) -> Result<()> {
        println!("shape: {:?}", s.shape);
        let shape = tf.shape(&s.shape);
        let bounds = self.grid_rt(shape.bounds());

        for l in bounds.l()..bounds.r() {
            for b in bounds.b()..bounds.t() {
                let p = PtI::new(l, b);
                // Collision rectangle test
                let r = Rt::enclosing(self.world_pt(&p), self.world_pt(&PtI::new(l + 1, b + 1)));
                if intersection_test(&identity(), &shape, &identity(), &r)? {
                    blocked.insert(State { p, layer: s.layer.clone() });
                }
            }
        }
        Ok(())
    }

    // need to unmark for pins we are currently looking at?
    fn mark_blocked_pin(&self, blocked: &mut HashSet<State>, tf: &Tf, pin: &Pin) {}

    fn mark_blocked(&mut self) -> Result<()> {
        let mut blocked = HashSet::new();
        for keepout in self.pcb.keepouts() {
            // TODO: Handle only via vs only wire keepout.
            self.mark_blocked_shape(&mut blocked, &Tf::identity(), &keepout.shape)?;
        }

        let mut wires = Vec::new();
        for c in self.pcb.components() {
            let tf = c.tf();
            // for pin in c.pins() {
            //     self.mark_blocked_shape(&mut blocked, &(tf * pin.tf()), pin.)
            // }
            for keepout in c.keepouts.iter() {
                // TODO: Handle only via vs only wire keepout.
                self.mark_blocked_shape(&mut blocked, &tf, &keepout.shape)?;
            }
        }
        self.blocked = blocked;
        self.wire_test = wires;
        Ok(())
    }

    // Checks if the state |s| is routable (inside boundary, outside of
    // keepouts, etc).
    fn is_blocked(&self, s: &State) -> bool {
        self.blocked.contains(s)
            || self.blocked.contains(&State { p: s.p, layer: ANY_LAYER.to_owned() })
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

    fn grid_rt(&self, r: Rt) -> RtI {
        RtI::enclosing(self.grid_pt(r.bl()), self.grid_pt(r.tr()) + PtI::new(1, 1))
    }

    fn world_pt(&self, p: &PtI) -> Pt {
        Pt::new(p.x as f64 * self.resolution, p.y as f64 * self.resolution)
    }

    fn world_pt_mid(&self, p: &PtI) -> Pt {
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
        let bounds = self.grid_rt(self.pcb.bounds());
        for l in bounds.l()..bounds.r() {
            for b in bounds.b()..bounds.t() {
                let p = PtI::new(l, b);
                if self.is_blocked(&State { p, layer: "F.Cu".to_owned() }) {
                    continue;
                }
                let shape =
                    ShapeType::Circle(Circle::new(self.world_pt_mid(&p), self.resolution / 2.0));
                res.wires.push(Wire { shape: Shape { layer: "F.Cu".to_owned(), shape } })
            }
        }
        res.wires.extend(self.wire_test.clone());
        Ok(res)
    }
}
