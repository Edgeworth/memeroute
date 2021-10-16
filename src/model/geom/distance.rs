use crate::model::geom::contains::poly_contains_pt;
use crate::model::geom::intersects::{
    cap_intersects_poly, circ_intersects_poly, circ_intersects_rt, poly_intersects_rt,
    rt_intersects_rt, rt_intersects_seg, seg_intersects_seg,
};
use crate::model::geom::math::eq;
use crate::model::primitive::capsule::Capsule;
use crate::model::primitive::circle::Circle;
use crate::model::primitive::line_shape::Line;
use crate::model::primitive::path_shape::Path;
use crate::model::primitive::point::Pt;
use crate::model::primitive::polygon::{Poly, edges};
use crate::model::primitive::rect::Rt;
use crate::model::primitive::seg;
use crate::model::primitive::segment::Segment;

// Distance functions should return 0 if there is intersection or containment.
// This property is used by quadtree which returns 0 if it detects an intersection
// by e.g. regular intersection tests.

fn min_dist(iter: impl Iterator<Item = f64>) -> f64 {
    let mut best = f64::MAX;
    for i in iter {
        best = best.min(i);
        if eq(best, 0.0) {
            return best;
        }
    }
    best
}

pub fn cap_cap_dist(a: &Capsule, b: &Capsule) -> f64 {
    let d = seg_seg_dist(&a.seg(), &b.seg()) - a.r() - b.r();
    d.max(0.0)
}

pub fn cap_circ_dist(a: &Capsule, b: &Circle) -> f64 {
    let d = pt_seg_dist(&b.p(), &a.seg()) - a.r() - b.r();
    d.max(0.0)
}

pub fn cap_path_dist(a: &Capsule, b: &Path) -> f64 {
    min_dist(b.caps().map(|cap| cap_cap_dist(a, &cap)))
}

pub fn cap_poly_dist(a: &Capsule, b: &Poly) -> f64 {
    if cap_intersects_poly(a, b) {
        0.0
    } else {
        min_dist(b.edges().map(|[&p0, &p1]| cap_seg_dist(a, &seg(p0, p1))))
    }
}

pub fn cap_rt_dist(a: &Capsule, b: &Rt) -> f64 {
    let d = rt_seg_dist(b, &a.seg()) - a.r();
    d.max(0.0)
}

pub fn cap_seg_dist(a: &Capsule, b: &Segment) -> f64 {
    let d = seg_seg_dist(&a.seg(), b) - a.r();
    d.max(0.0)
}

pub fn circ_circ_dist(a: &Circle, b: &Circle) -> f64 {
    let d = pt_pt_dist(&a.p(), &b.p()) - a.r() - b.r();
    d.max(0.0)
}

pub fn circ_path_dist(a: &Circle, b: &Path) -> f64 {
    min_dist(b.caps().map(|cap| cap_circ_dist(&cap, a)))
}

pub fn circ_poly_dist(a: &Circle, b: &Poly) -> f64 {
    if circ_intersects_poly(a, b) {
        0.0
    } else {
        let d = poly_pt_dist(b, &a.p()) - a.r();
        d.max(0.0)
    }
}

pub fn circ_rt_dist(a: &Circle, b: &Rt) -> f64 {
    if circ_intersects_rt(a, b) {
        0.0
    } else {
        // Project circle centre onto the rectangle:
        let p = a.p().clamp(b);
        p.dist(a.p()) - a.r()
    }
}

pub fn line_pt_dist(a: &Line, b: &Pt) -> f64 {
    b.dist(a.project(*b))
}

pub fn path_poly_dist(a: &Path, b: &Poly) -> f64 {
    min_dist(a.caps().map(|cap| cap_poly_dist(&cap, b)))
}

// Distance to a polygon outline.
pub fn polyline_pt_dist(a: &[Pt], b: &Pt) -> f64 {
min_dist(edges(a).map(|[&p0, &p1]| pt_seg_dist(b, &seg(p0, p1))))
}

pub fn poly_pt_dist(a: &Poly, b: &Pt) -> f64 {
    if poly_contains_pt(a, b) {
        0.0
    } else {
        polyline_pt_dist(a.pts(), b)
    }
}

pub fn poly_rt_dist(a: &Poly, b: &Rt) -> f64 {
    if poly_intersects_rt(a, b) {
        0.0
    } else {
        min_dist(a.edges().map(|[&p0, &p1]| rt_seg_dist(b, &seg(p0, p1))))
    }
}

pub fn pt_pt_dist(a: &Pt, b: &Pt) -> f64 {
    a.dist(*b)
}

pub fn pt_rt_dist(a: &Pt, b: &Rt) -> f64 {
    if b.contains(*a) {
        0.0
    } else {
        // Project centre onto the rectangle:
        let p = a.clamp(b);
        p.dist(*a)
    }
}

pub fn pt_seg_dist(a: &Pt, b: &Segment) -> f64 {
    let st_dist = a.dist(b.st());
    let en_dist = a.dist(b.en());
    let project = b.line().project(*a);
    let dist = st_dist.min(en_dist);
    if b.contains(project) { dist.min(a.dist(project)) } else { dist }
}

pub fn rt_path_dist(a: &Rt, b: &Path) -> f64 {
    min_dist(b.caps().map(|cap| cap_rt_dist(&cap, a)))
}

pub fn rt_rt_dist(a: &Rt, b: &Rt) -> f64 {
    if rt_intersects_rt(a, b) {
        0.0
    } else {
        // check points on a to b
        min_dist(a.pts().iter().map(|p| pt_rt_dist(p, b)))
    }
}

pub fn rt_seg_dist(a: &Rt, b: &Segment) -> f64 {
    if rt_intersects_seg(a, b) {
        0.0
    } else {
        // Check for closest distance from the segment to the edges of the rectangle.
        min_dist(a.segs().iter().map(|seg| seg_seg_dist(seg, b)))
    }
}

pub fn seg_seg_dist(a: &Segment, b: &Segment) -> f64 {
    // Closest distance must be between an endpoint and a segment, unless
    // the segments cross, in which case it is zero.
    if seg_intersects_seg(a, b) {
        return 0.0;
    }
    let mut best = pt_seg_dist(&a.st(), b);
    best = best.min(pt_seg_dist(&a.en(), b));
    best = best.min(pt_seg_dist(&b.st(), a));
    best = best.min(pt_seg_dist(&b.en(), a));
    best
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::model::primitive::{circ, pt};

    #[test]
    fn test_circ_circ() {
        assert_relative_eq!(
            0.0,
            circ_circ_dist(&circ(pt(0.0, 0.0), 0.4), &circ(pt(0.0, 0.0), 0.4))
        );
        assert_relative_eq!(
            130.94659781997535,
            circ_circ_dist(&circ(pt(111.6414, -70.632), 0.762), &circ(pt(0.0, 0.0), 0.4))
        );
    }
}
