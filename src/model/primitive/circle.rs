use crate::model::geom::contains::{circ_contains_pt, circ_contains_rt};
use crate::model::geom::distance::{cap_circ_dist, circ_path_dist, circ_rt_dist};
use crate::model::geom::intersects::{
    circ_intersects_circ, circ_intersects_path, circ_intersects_poly, circ_intersects_rt,
    circ_intersects_tri,
};
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
        rt(self.p.x - self.r, self.p.y - self.r, self.p.x + self.r, self.p.y + self.r)
    }

    fn shape(self) -> Shape {
        Shape::Circle(self)
    }

    fn intersects_shape(&self, s: &Shape) -> bool {
        match s {
            Shape::Capsule(_) => todo!(),
            Shape::Circle(s) => circ_intersects_circ(self, s),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(s) => circ_intersects_path(self, s),
            Shape::Point(s) => circ_contains_pt(self, s),
            Shape::Polygon(s) => circ_intersects_poly(self, s),
            Shape::Rect(s) => circ_intersects_rt(self, s),
            Shape::Segment(_) => todo!(),
            Shape::Tri(s) => circ_intersects_tri(self, s),
        }
    }

    fn contains_shape(&self, s: &Shape) -> bool {
        match s {
            Shape::Capsule(_) => todo!(),
            Shape::Circle(_) => todo!(),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(_) => todo!(),
            Shape::Point(s) => circ_contains_pt(self, s),
            Shape::Polygon(_) => todo!(),
            Shape::Rect(s) => circ_contains_rt(self, s),
            Shape::Segment(_) => todo!(),
            Shape::Tri(_) => todo!(),
        }
    }

    fn dist_to_shape(&self, s: &Shape) -> f64 {
        match s {
            Shape::Capsule(s) => cap_circ_dist(s, self),
            Shape::Circle(_) => todo!(),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(s) => circ_path_dist(self, s),
            Shape::Point(_) => todo!(),
            Shape::Polygon(_) => todo!(),
            Shape::Rect(s) => circ_rt_dist(self, s),
            Shape::Segment(_) => todo!(),
            Shape::Tri(_) => todo!(),
        }
    }
}
