use crate::model::primitive::capsule::Capsule;
use crate::model::primitive::circle::Circle;
use crate::model::primitive::line_shape::Line;
use crate::model::primitive::path_shape::Path;
use crate::model::primitive::point::{Pt, PtI};
use crate::model::primitive::polygon::Poly;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::segment::Segment;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::triangle::Tri;

pub mod capsule;
pub mod circle;
pub mod compound;
pub mod line_shape;
pub mod path_shape;
pub mod point;
pub mod polygon;
pub mod rect;
pub mod segment;
pub mod shape;
pub mod triangle;

pub trait ShapeOps {
    fn bounds(&self) -> Rt;
    fn shape(self) -> Shape;
    // Returns true iff the two shapes have a non-zero intersection.
    fn intersects_shape(&self, s: &Shape) -> bool;
    // Returns true iff |s| is fully contained within this shape.
    fn contains_shape(&self, s: &Shape) -> bool;
    // Returns the minimum distance between the two shapes.
    fn dist_to_shape(&self, s: &Shape) -> f64;
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

pub fn poly(pts: &[Pt]) -> Poly {
    Poly::new(pts)
}

pub const fn rt(l: f64, b: f64, r: f64, t: f64) -> Rt {
    Rt::new(l, b, r, t)
}

pub const fn seg(st: Pt, en: Pt) -> Segment {
    Segment::new(st, en)
}

pub fn tri(a: Pt, b: Pt, c: Pt) -> Tri {
    Tri::new([a, b, c])
}
