use crate::model::primitive::rt::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::pt::Pt;

#[derive(Debug, Copy, Clone)]
pub struct Circle {
    p: Pt,
    r: f64,
}

impl Circle {
    pub fn new(p: Pt, r: f64) -> Self {
        Self { p, r }
    }

    pub fn shape(self) -> Shape {
        Shape::Circle(self)
    }

    pub fn bounds(&self) -> Rt {
        Rt::new(self.p.x - self.r, self.p.y - self.r, 2.0 * self.r, 2.0 * self.r)
    }

    pub fn r(&self) -> f64 {
        self.r
    }

    pub fn p(&self) -> Pt {
        self.p
    }
}
