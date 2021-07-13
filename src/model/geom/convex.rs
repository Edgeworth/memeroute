use crate::model::geom::math::{is_left_of, is_strictly_left_of};
use crate::model::pt::Pt;

pub fn ensure_ccw(pts: &mut [Pt]) {
    // TODO: Remove collinear.
    if pts.len() > 2 && !is_left_of(pts[2], pts[0], pts[1]) {
        pts.reverse()
    }
}

pub fn is_convex_ccw(pts: &[Pt]) -> bool {
    for i in 0..pts.len() {
        let a = pts[i];
        let b = pts[(i + 1) % pts.len()];
        let c = pts[(i + 2) % pts.len()];
        if !is_strictly_left_of(c, a, b) {
            return false;
        }
    }
    true
}
