use derive_more::Display;

use crate::model::geom::contains::{cap_contains_pt, cap_contains_rt};
use crate::model::geom::distance::{
    cap_cap_dist, cap_circ_dist, cap_path_dist, cap_poly_dist, cap_rt_dist, cap_seg_dist,
};
use crate::model::geom::intersects::{
    cap_intersects_cap, cap_intersects_circ, cap_intersects_path, cap_intersects_poly,
    cap_intersects_rt, cap_intersects_tri,
};
use crate::model::primitive::circle::Circle;
use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::segment::Segment;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::{circ, seg, ShapeOps};

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
        let r = seg(self.st(), self.en()).bounds();
        r.inset(-self.r(), -self.r())
    }

    fn shape(self) -> Shape {
        Shape::Capsule(self)
    }

    fn intersects_shape(&self, s: &Shape) -> bool {
        match s {
            Shape::Capsule(s) => cap_intersects_cap(self, s),
            Shape::Circle(s) => cap_intersects_circ(self, s),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(s) => cap_intersects_path(self, s),
            Shape::Point(s) => cap_contains_pt(self, s),
            Shape::Polygon(s) => cap_intersects_poly(self, s),
            Shape::Rect(s) => cap_intersects_rt(self, s),
            Shape::Segment(_) => todo!(),
            Shape::Tri(s) => cap_intersects_tri(self, s),
        }
    }

    fn contains_shape(&self, s: &Shape) -> bool {
        match s {
            Shape::Capsule(_) => todo!(),
            Shape::Circle(_) => todo!(),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(_) => todo!(),
            Shape::Point(s) => cap_contains_pt(self, s),
            Shape::Polygon(_) => todo!(),
            Shape::Rect(s) => cap_contains_rt(self, s),
            Shape::Segment(_) => todo!(),
            Shape::Tri(_) => todo!(),
        }
    }

    fn dist_to_shape(&self, s: &Shape) -> f64 {
        match s {
            Shape::Capsule(s) => cap_cap_dist(self, s),
            Shape::Circle(s) => cap_circ_dist(self, s),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(s) => cap_path_dist(self, s),
            Shape::Point(_) => todo!(),
            Shape::Polygon(s) => cap_poly_dist(self, s),
            Shape::Rect(s) => cap_rt_dist(self, s),
            Shape::Segment(s) => cap_seg_dist(self, s),
            Shape::Tri(_) => todo!(),
        }
    }
}
