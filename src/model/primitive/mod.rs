use crate::model::primitive::capsule::Capsule;
use crate::model::primitive::circle::Circle;
use crate::model::primitive::line_shape::Line;
use crate::model::primitive::path_shape::Path;
use crate::model::primitive::point::{Pt, PtI};
use crate::model::primitive::polygon::Polygon;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::segment::Segment;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::triangle::Tri;

pub mod circle;
pub mod line_shape;
pub mod path_shape;
pub mod point;
pub mod polygon;
pub mod rect;
pub mod segment;
pub mod shape;
pub mod triangle;
pub mod capsule;

pub trait ShapeOps {
    fn bounds(&self) -> Rt;
    fn shape(self) -> Shape;
}

pub fn cap(st: Pt, en: Pt, r: f64) -> Capsule {
    Capsule::new(st, en, r)
}

pub fn circ(p: Pt, r: f64) -> Circle {
    Circle::new(p, r)
}

pub const fn line(st: Pt, en: Pt) -> Line {
    Line::new(st, en)
}

pub fn path(pts: &[Pt], r: f64) -> Path {
    Path::new(pts, r)
}

pub const fn pt(x: f64, y: f64) -> Pt {
    Pt::new(x, y)
}

pub const fn pti(x: i64, y: i64) -> PtI {
    PtI::new(x, y)
}

pub fn poly(pts: &[Pt]) -> Polygon {
    Polygon::new(pts)
}

pub const fn rt(l: f64, b: f64, w: f64, h: f64) -> Rt {
    Rt::new(l, b, w, h)
}

pub const fn seg(st: Pt, en: Pt) -> Segment {
    Segment::new(st, en)
}

pub const fn tri(a: Pt, b: Pt, c: Pt) -> Tri {
    Tri::new([a, b, c])
}
