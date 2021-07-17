use approx::{relative_eq, relative_ne};

use crate::model::primitive::line_shape::Line;
use crate::model::primitive::point::Pt;

pub const EP: f64 = 1e-6;

pub fn pt_eq(a: Pt, b: Pt) -> bool {
    relative_eq!(a, b, epsilon = EP)
}

pub fn eq(a: f64, b: f64) -> bool {
    relative_eq!(a, b, epsilon = EP)
}

pub fn ne(a: f64, b: f64) -> bool {
    relative_ne!(a, b, epsilon = EP)
}

pub fn gt(a: f64, b: f64) -> bool {
    ne(a, b) && a > b
}

pub fn ge(a: f64, b: f64) -> bool {
    eq(a, b) || a > b
}

pub fn le(a: f64, b: f64) -> bool {
    eq(a, b) || a < b
}

pub fn lt(a: f64, b: f64) -> bool {
    ne(a, b) && a < b
}

// Return cross-product of OA and OB.
pub fn cross_at(o: Pt, a: Pt, b: Pt) -> f64 {
    (o - a).cross(o - b)
}

// Returns true iff p is strictly left of line.
pub fn is_strictly_left_of(l: &Line, p: Pt) -> bool {
    gt(cross_at(l.st(), l.en(), p), 0.0)
}

pub fn is_left_of(l: &Line, p: Pt) -> bool {
    ge(cross_at(l.st(), l.en(), p), 0.0)
}

pub fn is_collinear(a: Pt, b: Pt, c: Pt) -> bool {
    eq(cross_at(a, b, c), 0.0)
}

// Returns true iff all points |p| are on the same side of |l|.
pub fn pts_same_side(l: &Line, pts: &[Pt]) -> bool {
    if pts.is_empty() {
        return true;
    }
    let is_left = is_left_of(l, pts[0]);
    for p in pts.iter() {
        if is_left != is_left_of(l, *p) {
            return false;
        }
    }
    true
}
