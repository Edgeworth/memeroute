use std::cmp::Ordering;

use approx::{relative_eq, relative_ne};

use crate::model::primitive::line_shape::Line;
use crate::model::primitive::point::Pt;

pub const EP: f64 = 1e-6;

pub fn f64_cmp(a: &f64, b: &f64) -> Ordering {
    a.partial_cmp(b).unwrap()
}

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

// -1 for CW (right of), 0 for collinear, 1 for CCW (left of)
pub fn orientation(l: &Line, p: Pt) -> i32 {
    let v = cross_at(l.st(), l.en(), p);
    if eq(v, 0.0) {
        0
    } else if v > 0.0 {
        1
    } else {
        -1
    }
}

// Returns true iff p is strictly left of line.
pub fn is_strictly_left_of(l: &Line, p: Pt) -> bool {
    gt(cross_at(l.st(), l.en(), p), 0.0)
}

pub fn is_left_of(l: &Line, p: Pt) -> bool {
    ge(cross_at(l.st(), l.en(), p), 0.0)
}

pub fn is_strictly_right_of(l: &Line, p: Pt) -> bool {
    lt(cross_at(l.st(), l.en(), p), 0.0)
}

pub fn is_right_of(l: &Line, p: Pt) -> bool {
    le(cross_at(l.st(), l.en(), p), 0.0)
}

pub fn is_collinear(a: Pt, b: Pt, c: Pt) -> bool {
    eq(cross_at(a, b, c), 0.0)
}

pub fn pts_strictly_right_of(l: &Line, pts: &[Pt]) -> bool {
    for p in pts {
        if !is_strictly_right_of(l, *p) {
            return false;
        }
    }
    true
}

// Returns true iff all points |p| are on the same side of |l|.
pub fn pts_same_side(l: &Line, pts: &[Pt]) -> bool {
    let mut had_one = false;
    let mut had_neg_one = false;
    for p in pts {
        match orientation(l, *p) {
            1 => had_one = true,
            -1 => had_neg_one = true,
            _ => {}
        }
    }
    !(had_one && had_neg_one)
}

// Returns true iff all points |p| are on the same side of |l| and not collinear.
pub fn pts_strictly_same_side(l: &Line, pts: &[Pt]) -> bool {
    if pts.is_empty() {
        return true;
    }
    let o = orientation(l, pts[0]);
    if o == 0 {
        return false;
    }
    for p in pts {
        if o != orientation(l, *p) {
            return false;
        }
    }
    true
}
