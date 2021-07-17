use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::ShapeOps;

#[derive(Debug, Copy, Clone)]
pub struct Line {
    st: Pt,
    en: Pt,
}

impl Line {
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

impl ShapeOps for Line {
    fn bounds(&self) -> Rt {
        // Bounds kind of undefined for a Line.
        Rt::empty()
    }

    fn shape(self) -> Shape {
        Shape::Line(self)
    }
}
