use crate::model::geom::intersects::seg_intersects_seg;
use crate::model::geom::math::f64_cmp;
use crate::model::primitive::circle::Circle;
use crate::model::primitive::line_shape::Line;
use crate::model::primitive::point::Pt;
use crate::model::primitive::polygon::Poly;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::seg;
use crate::model::primitive::segment::Segment;

// Distance functions should return 0 if there is intersection or containment.
// This property is used by quadtree which returns 0 if it detects an intersection
// by e.g. regular intersection tests.

// TODO:Audit these functions.

// Returns the distance from the circle to the boundary of the
// rectangle. Doesn't work if the point is inside the rectangle.
pub fn circ_rt_dist(a: &Circle, b: &Rt) -> f64 {
    // Project circle centre onto the rectangle:
    let p = a.p().clamp(b);
    p.dist(a.p()) - a.r()
}

pub fn line_pt_dist(a: &Line, b: &Pt) -> f64 {
    b.dist(a.project(*b))
}

// Works inside of the polygon too. Distance to the boundary.
pub fn pt_poly_dist(a: &Pt, b: &Poly) -> f64 {
    b.edges().map(|[&p0, &p1]| pt_seg_dist(a, &seg(p0, p1))).min_by(f64_cmp).unwrap()
}

pub fn pt_seg_dist(a: &Pt, b: &Segment) -> f64 {
    let st_dist = a.dist(b.st());
    let en_dist = a.dist(b.en());
    let project = b.line().project(*a);
    let dist = st_dist.min(en_dist);
    if b.contains(project) { dist.min(a.dist(project)) } else { dist }
}

pub fn rt_rt_dist(_a: &Rt, _b: &Rt) -> f64 {
    todo!()
}

pub fn rt_seg_dist(a: &Rt, b: &Segment) -> f64 {
    // Check for closest distance from the segment to the edges of the rectangle.
    a.segs().iter().map(|seg| seg_seg_dist(seg, b)).min_by(f64_cmp).unwrap()
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
