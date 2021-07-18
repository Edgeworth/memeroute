use crate::model::geom::math::{ge, is_left_of, is_right_of, lt};
use crate::model::primitive::point::Pt;
use crate::model::primitive::polygon::Polygon;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::segment::Segment;
use crate::model::primitive::{line, seg};

pub fn poly_contains_rt(a: &Polygon, b: &Rt) -> bool {
    // Check point containment of |b| in |a|.
    let pts = b.pts();
    for p in pts {
        if !poly_contains_pt(a, &p) {
            return false;
        }
    }
    // Check segment containment of |b| in |a| if |a| is non-convex.
    if !a.is_convex() {
        for i in 0..pts.len() {
            let p0 = pts[i];
            let p1 = pts[(i + 1) % pts.len()];
            if !poly_contains_seg(a, &seg(p0, p1)) {
                return false;
            }
        }
    }
    true
}

pub fn poly_contains_pt(a: &Polygon, b: &Pt) -> bool {
    // Winding number test. Look at horizontal line at b.y and count crossings
    // of edges from |a|. Treats points on the boundary of the polygon as
    // contained.
    let mut winding = 0;
    let pts = a.pts();
    for i in 0..pts.len() {
        let p0 = pts[i];
        let p1 = pts[(i + 1) % pts.len()];
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

pub fn poly_contains_seg(_a: &Polygon, _b: &Segment) -> bool {
    todo!()
}
