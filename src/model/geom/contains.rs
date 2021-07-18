use crate::model::geom::math::{ge, is_left_of, is_right_of, lt};
use crate::model::primitive::line;
use crate::model::primitive::point::Pt;
use crate::model::primitive::polygon::Polygon;
use crate::model::primitive::rect::Rt;

pub fn poly_contains_rt(a: &Polygon, b: &Rt) -> bool {
    // TODO: can't just test point containment for non-convex polygon.
    // Check point and segment containment of |b| in |a|.
    for p in b.pts() {
        if !poly_contains_pt(a, &p) {
            return false;
        }
    }
    true
}

pub fn poly_contains_pt(a: &Polygon, b: &Pt) -> bool {
    // Winding number test. Look at horizontal line at b.y and count crossings
    // of edges from |a|. Treats points on the boundary of the polygon as
    // contained.
    let mut winding = 0;
    for &[p0, p1] in a.pts().array_windows::<2>() {
        // Treat points at b.y as slightly above it.
        if ge(p0.y, b.y) {
            // Downward crossing edge with |b| to the right of it decreases
            // winding number.
            if lt(p1.y, b.y) && is_right_of(&line(p0, p1), *b) {
                winding -= 1;
            }
        } else if ge(p1.y, b.y) && is_left_of(&line(p0, p1), *b) {
            // Upward crossing edge with |b| to the left of it increases
            // winding number.
            winding += 1;
        }
    }
    winding != 0
}
