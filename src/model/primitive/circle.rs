use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::{rt, ShapeOps};

#[derive(Debug, Copy, Clone)]
pub struct Circle {
    p: Pt,
    r: f64,
}

impl Circle {
    pub const fn new(p: Pt, r: f64) -> Self {
        Self { p, r }
    }

    pub const fn r(&self) -> f64 {
        self.r
    }

    pub const fn p(&self) -> Pt {
        self.p
    }
}

impl ShapeOps for Circle {
    fn bounds(&self) -> Rt {
        rt(self.p.x - self.r, self.p.y - self.r, 2.0 * self.r, 2.0 * self.r)
    }

    fn shape(self) -> Shape {
        Shape::Circle(self)
    }
}
