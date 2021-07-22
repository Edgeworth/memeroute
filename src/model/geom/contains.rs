use crate::model::geom::math::{ge, is_left_of, is_right_of, lt, orientation};
use crate::model::primitive::capsule::Capsule;
use crate::model::primitive::path_shape::Path;
use crate::model::primitive::point::Pt;
use crate::model::primitive::polygon::{edges, Poly};
use crate::model::primitive::rect::Rt;
use crate::model::primitive::segment::Segment;
use crate::model::primitive::triangle::Tri;
use crate::model::primitive::{line, seg};

pub fn poly_contains_cap(a: &Poly, b: &Capsule) -> bool {
    todo!()
}

pub fn poly_contains_path(a: &Poly, b: &Path) -> bool {
    todo!()
}

pub fn poly_contains_pt(a: &Poly, b: &Pt) -> bool {
    // Winding number test. Look at horizontal line at b.y and count crossings
    // of edges from |a|. Treats points on the boundary of the polygon as
    // contained.
    let mut winding = 0;
    for [&p0, &p1] in a.edges() {
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

pub fn poly_contains_rt(a: &Poly, b: &Rt) -> bool {
    // Check point containment of |b| in |a|.
    let pts = b.pts();
    for p in pts {
        if !poly_contains_pt(a, &p) {
            return false;
        }
    }
    // Check segment containment of |b| in |a| if |a| is non-convex.
    if !a.is_convex() {
        for [&p0, &p1] in edges(&pts) {
            if !poly_contains_seg(a, &seg(p0, p1)) {
                return false;
            }
        }
    }
    true
}

pub fn poly_contains_seg(a: &Poly, b: &Segment) -> bool {
    // Check that both endpoints of |b| are in a.
    if !poly_contains_pt(a, &b.st()) || !poly_contains_pt(a, &b.en()) {
        return false;
    }

    // If |a| is convex only need to check endpoint containment.
    if a.is_convex() {
        return true;
    }

    // Check that |b| does not cross any edge of |a|.
    for [&p0, &p1] in a.edges() {
        let p_st = orientation(&b.line(), p0);
        let p_en = orientation(&b.line(), p1);
        let b_st = orientation(&line(p0, p1), b.st());
        let b_en = orientation(&line(p0, p1), b.en());
        // Segments are crossing and no collinear points.
        if p_st != p_en && b_st != b_en {
            return false;
        }
    }
    true
}

pub fn rt_contains_poly(a: &Rt, b: &Poly) -> bool {
    // Just check all points in |b| are in |a|.
    for p in b.pts() {
        if !a.contains(*p) {
            return false;
        }
    }
    true
}

pub fn rt_contains_tri(a: &Rt, b: &Tri) -> bool {
    // Just check all points in |b| are in |a|.
    for p in b.pts() {
        if !a.contains(*p) {
            return false;
        }
    }
    true
}

pub fn tri_contains_pt(a: &Tri, b: &Pt) -> bool {
    let orientation0 = orientation(&line(a[0], a[1]), *b);
    let orientation1 = orientation(&line(a[1], a[2]), *b);
    let orientation2 = orientation(&line(a[2], a[0]), *b);
    orientation0 == orientation1 && orientation1 == orientation2
}
