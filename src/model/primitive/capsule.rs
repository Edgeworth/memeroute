use derive_more::Display;

use crate::model::geom::intersects::cap_intersect_rt;
use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::segment::Segment;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::{line, seg, ShapeOps};

#[derive(Debug, Display, Copy, Clone)]
#[display(fmt = "Cap[{}, {}; {}]", st, en, r)]
pub struct Capsule {
    st: Pt,
    en: Pt,
    r: f64,
}

impl Capsule {
    pub const fn new(st: Pt, en: Pt, r: f64) -> Self {
        Self { st, en, r }
    }

    pub const fn r(&self) -> f64 {
        self.r
    }

    pub const fn st(&self) -> Pt {
        self.st
    }

    pub const fn en(&self) -> Pt {
        self.en
    }

    pub fn seg(&self) -> Segment {
        seg(self.st, self.en)
    }
}

impl ShapeOps for Capsule {
    fn bounds(&self) -> Rt {
        let r = line(self.st(), self.en()).bounds();
        r.inset(-self.r(), -self.r())
    }

    fn shape(self) -> Shape {
        Shape::Capsule(self)
    }

    fn intersects_shape(&self, s: &Shape) -> bool {
        match s {
            Shape::Capsule(_) => todo!(),
            Shape::Circle(_) => todo!(),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(_) => todo!(),
            Shape::Point(_) => todo!(),
            Shape::Polygon(_) => todo!(),
            Shape::Rect(s) => cap_intersect_rt(self, s),
            Shape::Segment(_) => todo!(),
            Shape::Tri(_) => todo!(),
        }
    }

    fn contains_shape(&self, s: &Shape) -> bool {
        match s {
            Shape::Capsule(_) => todo!(),
            Shape::Circle(_) => todo!(),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(_) => todo!(),
            Shape::Point(_) => todo!(),
            Shape::Polygon(_) => todo!(),
            Shape::Rect(_) => todo!(),
            Shape::Segment(_) => todo!(),
            Shape::Tri(_) => todo!(),
        }
    }

    fn dist_to_shape(&self, s: &Shape) -> f64 {
        match s {
            Shape::Capsule(_) => todo!(),
            Shape::Circle(_) => todo!(),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(_) => todo!(),
            Shape::Point(_) => todo!(),
            Shape::Polygon(_) => todo!(),
            Shape::Rect(_) => todo!(),
            Shape::Segment(_) => todo!(),
            Shape::Tri(_) => todo!(),
        }
    }
}
