use derive_more::Display;

use crate::model::geom::math::is_collinear;
use crate::model::primitive::line_shape::Line;
use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::{line, ShapeOps};

#[derive(Debug, Display, Copy, Clone)]
#[display(fmt = "Seg[{}, {}]", st, en)]
pub struct Segment {
    st: Pt,
    en: Pt,
}

impl Segment {
    pub const fn new(st: Pt, en: Pt) -> Self {
        Self { st, en }
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

    pub const fn line(&self) -> Line {
        line(self.st, self.en)
    }

    pub fn contains(&self, p: Pt) -> bool {
        Rt::enclosing(self.st, self.en).contains(p) && is_collinear(self.st, self.en, p)
    }
}

impl ShapeOps for Segment {
    fn bounds(&self) -> Rt {
        Rt::enclosing(self.st, self.en)
    }

    fn shape(self) -> Shape {
        Shape::Segment(self)
    }
}
