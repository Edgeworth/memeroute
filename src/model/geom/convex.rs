use crate::model::geom::math::{is_collinear, is_left_of, is_strictly_left_of};
use crate::model::primitive::line;
use crate::model::primitive::point::Pt;

#[must_use]
pub fn remove_collinear(pts: &[Pt]) -> Vec<Pt> {
    if pts.len() <= 2 {
        return pts.to_vec();
    }
    let mut out = vec![pts[0], pts[1]];
    for &p in pts.iter().skip(2) {
        let l = out.len();
        if is_collinear(out[l - 2], out[l - 1], p) {
            out.pop();
        }
        out.push(p);
    }
    out
}

pub fn ensure_ccw(pts: &mut [Pt]) {
    if pts.len() > 2 && !is_left_of(&line(pts[0], pts[1]), pts[2]) {
        pts.reverse();
    }
}

// Tests if a CCW polygon |pts| is convex.
#[must_use]
pub fn is_convex_ccw(pts: &[Pt]) -> bool {
    for i in 0..pts.len() {
        let a = pts[i];
        let b = pts[(i + 1) % pts.len()];
        let c = pts[(i + 2) % pts.len()];
        if !is_strictly_left_of(&line(a, b), c) {
            return false;
        }
    }
    true
}
