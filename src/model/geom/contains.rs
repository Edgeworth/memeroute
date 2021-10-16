use crate::model::geom::distance::{polyline_pt_dist, pt_seg_dist};
use crate::model::geom::math::{ge, is_left_of, is_right_of, le, lt, orientation};
use crate::model::primitive::capsule::Capsule;
use crate::model::primitive::circle::Circle;
use crate::model::primitive::path_shape::Path;
use crate::model::primitive::point::Pt;
use crate::model::primitive::polygon::Poly;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::segment::Segment;
use crate::model::primitive::triangle::Tri;
use crate::model::primitive::{line, ShapeOps};

pub fn cap_contains_pt(a: &Capsule, b: &Pt) -> bool {
    // Bounding box check.
    if !a.bounds().contains(*b) {
        return false;
    }          

    le(pt_seg_dist(b, &a.seg()), a.r())
}

pub fn cap_contains_rt(a: &Capsule, b: &Rt) -> bool {
    // Bounding box check.
    if !a.bounds().contains_rt(b) {
        return false;
    }

    for p in b.pts() {
        if !cap_contains_pt(a, &p) {
            return false;
        }
    }
    true
}

pub fn circ_contains_rt(a: &Circle, b: &Rt) -> bool {
    // Sufficient to check all rectangle points are within the circle.
    circ_contains_pt(a, &b.bl())
        && circ_contains_pt(a, &b.br())
        && circ_contains_pt(a, &b.tr())
        && circ_contains_pt(a, &b.tl())
}

pub fn circ_contains_pt(a: &Circle, b: &Pt) -> bool {
    le(a.p().dist(*b), a.r())
}

pub fn path_contains_rt(a: &Path, b: &Rt) -> bool {
    // Bounding box check.
    if !a.bounds().contains_rt(b) {
        return false;
    }

    // This function is too complicated to have an exact solution.
    // An approach is to split |a| into quads and circles, then compute the
    // intersection of the quads and |b|. Then, do voronoi with the circles
    // and ensure the non-intersected parts of |b| are covered.
    // This function is only used in the quadtree and it doesn't have to
    // be exact so instead just check each capsule. It will miss cases
    // where the rectangle goes over multiple capsules.
    for cap in a.caps() {
        if cap_contains_rt(&cap, b) {
            return true;
        }
    }
    false
}

pub fn path_contains_seg(_a: &Path, _b: &Segment) -> bool {
    todo!()
}

pub fn poly_contains_cap(a: &Poly, b: &Capsule) -> bool {
    // Bounding box check.
    if !a.bounds().contains_rt(&b.bounds()) {
        return false;
    }

    // First check both end caps are in the polygon.
    if !poly_contains_circ(a, &b.st_cap()) {
        return false;
    }
    if !poly_contains_circ(a, &b.en_cap()) {
        return false;
    }
    // Check left and right walls of the segment are in the polygon.
    if !poly_contains_seg(a, &b.left_seg()) {
        return false;
    }
    if !poly_contains_seg(a, &b.right_seg()) {
        return false;
    }
    true
}

pub fn poly_contains_circ(a: &Poly, b: &Circle) -> bool {
    // Test that the centre of the circle is in the polygon.
    if !poly_contains_pt(a, &b.p()) {
        return false;
    }
    ge(polyline_pt_dist(a.pts(), &b.p()), b.r())
}

pub fn poly_contains_path(a: &Poly, b: &Path) -> bool {
    // Bounding box check.
    if !a.bounds().contains_rt(&b.bounds()) {
        return false;
    }

    for cap in b.caps() {
        if !poly_contains_cap(a, &cap) {
            return false;
        }
    }
    true
}

pub fn poly_contains_pt(a: &Poly, b: &Pt) -> bool {
    // Bounding box check.
    if !a.bounds().contains(*b) {
        return false;
    }

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
    // Bounding box check.
    if !a.bounds().contains_rt(b) {
        return false;
    }

    // Check point containment of |b| in |a|.
    for p in b.pts() {
        if !poly_contains_pt(a, &p) {
            return false;
        }
    }
    // Check segment containment of |b| in |a| if |a| is non-convex.
    if !a.is_convex() {
        for seg in b.segs() {
            if !poly_contains_seg(a, &seg) {
                return false;
            }
        }
    }
    true
}

pub fn poly_contains_seg(a: &Poly, b: &Segment) -> bool {
    // Bounding box check.
    if !a.bounds().contains_rt(&b.bounds()) {
        return false;
    }

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

pub fn rt_contains_cap(a: &Rt, b: &Capsule) -> bool {
    // Bounding box check.
    if !a.contains_rt(&b.bounds()) {
        return false;
    }

    // First check both end caps are in the rect.
    if !rt_contains_circ(a, &b.st_cap()) {
        return false;
    }
    if !rt_contains_circ(a, &b.en_cap()) {
        return false;
    }
    // Check left and right walls of the segment are in the rect.
    if !rt_contains_seg(a, &b.left_seg()) {
        return false;
    }
    if !rt_contains_seg(a, &b.right_seg()) {
        return false;
    }
    true
}

pub fn rt_contains_circ(a: &Rt, b: &Circle) -> bool {
    // Check the centre is in the rectangle:
    if !a.contains(b.p()) {
        return false;
    }
    // Check the shortest distance to the wall is less than or equal to the
    // radius.
    let x_dist = (b.p().x - a.l()).min(a.r() - b.p().x);
    if lt(x_dist, b.r()) {
        return false;
    }
    let y_dist = (b.p().y - a.b()).min(a.t() - b.p().y);
    if lt(y_dist, b.r()) {
        return false;
    }
    true
}

pub fn rt_contains_path(a: &Rt, b: &Path) -> bool {
    // Bounding box check.
    if !a.contains_rt(&b.bounds()) {
        return false;
    }

    // Just check all points in |b| are in |a|.
    for cap in b.caps() {
        if !rt_contains_cap(a, &cap) {
            return false;
        }
    }
    true
}

pub fn rt_contains_poly(a: &Rt, b: &Poly) -> bool {
    // Bounding box check.
    if !a.contains_rt(&b.bounds()) {
        return false;
    }

    // Just check all points in |b| are in |a|.
    for p in b.pts() {
        if !a.contains(*p) {
            return false;
        }
    }
    true
}

pub fn rt_contains_seg(a: &Rt, b: &Segment) -> bool {
    // Just need to check containment of both endpoints.
    a.contains(b.st()) && a.contains(b.en())
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
