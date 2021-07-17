use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::ShapeOps;

#[derive(Debug, Copy, Clone)]
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
}

impl ShapeOps for Segment {
    fn bounds(&self) -> Rt {
        Rt::enclosing(self.st, self.en)
    }

    fn shape(self) -> Shape {
        Shape::Segment(self)
    }
}
