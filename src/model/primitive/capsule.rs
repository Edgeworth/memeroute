use derive_more::Display;

use crate::model::geom::intersects::{cap_intersect_cap, cap_intersect_rt};
use crate::model::primitive::circle::Circle;
use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::segment::Segment;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::{circ, line, seg, ShapeOps};

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

    pub fn dir(&self) -> Pt {
        self.en - self.st
    }

    pub fn st_cap(&self) -> Circle {
        circ(self.st(), self.r())
    }

    pub fn en_cap(&self) -> Circle {
        circ(self.en(), self.r())
    }

    // Left wall of the capsule.
    pub fn left_seg(&self) -> Segment {
        let perp = -self.dir().perp() * self.r();
        seg(self.st + perp, self.en + perp)
    }

    // Right wall of the capsule.
    pub fn right_seg(&self) -> Segment {
        let perp = self.dir().perp() * self.r();
        seg(self.st + perp, self.en + perp)
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
            Shape::Capsule(s) => cap_intersect_cap(self, s),
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
