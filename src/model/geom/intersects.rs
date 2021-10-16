use crate::model::geom::contains::{cap_contains_pt, tri_contains_pt};
use crate::model::geom::distance::{rt_seg_dist, seg_seg_dist};
use crate::model::geom::math::{le, lt, ne, orientation, pts_strictly_right_of};
use crate::model::primitive::capsule::Capsule;
use crate::model::primitive::circle::Circle;
use crate::model::primitive::line_shape::Line;
use crate::model::primitive::path_shape::Path;
use crate::model::primitive::polygon::Poly;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::segment::Segment;
use crate::model::primitive::triangle::Tri;
use crate::model::primitive::{cap, ShapeOps};

pub fn cap_intersects_cap(a: &Capsule, b: &Capsule) -> bool {
    // Check bounding boxes.
    if !a.bounds().intersects(&b.bounds()) {
        return false;
    }

    le(seg_seg_dist(&a.seg(), &b.seg()), a.r() + b.r())
}

pub fn cap_intersects_circ(a: &Capsule, b: &Circle) -> bool {
    // Compute minkowski sum of |a| and |b| and check containment.
    let sum = cap(a.st(), a.en(), a.r() + b.r());
    cap_contains_pt(&sum, &b.p())
}

pub fn cap_intersects_path(a: &Capsule, b: &Path) -> bool {
    // Check if any cap in the path intersects this cap.
    for cap in b.caps() {
        if cap_intersects_cap(a, &cap) {
            return true;
        }
    }
    false
}

pub fn cap_intersects_poly(a: &Capsule, b: &Poly) -> bool {
    for tri in b.tri() {
        if cap_intersects_tri(a, tri) {
            return true;
        }
    }
    false
}

pub fn cap_intersects_rt(a: &Capsule, b: &Rt) -> bool {
    // Check bounding boxes.
    if !a.bounds().intersects(b) {
        return false;
    }

    if b.contains(a.st()) || b.contains(a.en()) {
        return true;
    }

    le(rt_seg_dist(b, &a.seg()), a.r())
}

pub fn cap_intersects_tri(a: &Capsule, b: &Tri) -> bool {
    // Check if the capsule is contained within the triangle:
    if tri_contains_pt(b, &a.st()) || tri_contains_pt(b, &a.en()) {
        return true;
    }
    // Otherwise to intersect, the triangle boundary needs to be intersecting
    // the capsule.
    for seg in b.segs() {
        if le(seg_seg_dist(&a.seg(), &seg), a.r()) {
            return true;
        }
    }
    false
}

pub fn circ_intersects_circ(a: &Circle, b: &Circle) -> bool {
    le(a.p().dist(b.p()), a.r() + b.r())
}

pub fn circ_intersects_path(a: &Circle, b: &Path) -> bool {
    // Test all capsules in path against circle.
    for cap in b.caps() {
        if cap_intersects_circ(&cap, a) {
            return true;
        }
    }
    false
}

pub fn circ_intersects_poly(a: &Circle, b: &Poly) -> bool {
    // Check bounding boxes.
    if !a.bounds().intersects(&b.bounds()) {
        return false;
    }

    for tri in b.tri() {
        if circ_intersects_tri(a, tri) {
            return true;
        }
    }
    false
}

pub fn circ_intersects_rt(a: &Circle, b: &Rt) -> bool {
    // Check bounding boxes.
    if !a.bounds().intersects(b) {
        return false;
    }

    // Check if the circle centre is contained in the rect or
    // the distance from the boundary of the rect to the circle is less than 0.
    // Project circle centre onto the rectangle:
    let p = a.p().clamp(b);
    let d = p.dist(a.p()) - a.r();
    b.contains(a.p()) || lt(d, 0.0)
}

pub fn circ_intersects_tri(a: &Circle, b: &Tri) -> bool {
    // Check bounding boxes.
    if !a.bounds().intersects(&b.bounds()) {
        return false;
    }

    // Take the minkowski sum of the circle and triangle. Just need to test
    // if the triangle contains the centre of the circle or any of its
    // capsules contain the point.
    if tri_contains_pt(b, &a.p()) {
        return true;
    }
    for seg in b.segs() {
        if cap_contains_pt(&cap(seg.st(), seg.en(), a.r()), &a.p()) {
            return true;
        }
    }
    false
}

pub fn line_intersects_line(a: &Line, b: &Line) -> bool {
    // Intersects if not parallel.
    ne(a.dir().cross(b.dir()), 0.0)
}

pub fn line_intersects_seg(_a: &Line, _b: &Segment) -> bool {
    todo!()
}

pub fn path_intersects_path(a: &Path, b: &Path) -> bool {
    // Try pairwise intersection of capsules.
    for i in 0..a.len() - 1 {
        for j in i..b.len() - 1 {
            let cap0 = cap(a[i], a[i + 1], a.r());
            let cap1 = cap(b[j], b[j + 1], b.r());
            if cap_intersects_cap(&cap0, &cap1) {
                return true;
            }
        }
    }
    false
}

pub fn path_intersects_rt(a: &Path, b: &Rt) -> bool {
    // Check whether each capsule in the path intersects the rectangle.
    for cap in a.caps() {
        if cap_intersects_rt(&cap, b) {
            return true;
        }
    }
    false
}

pub fn path_intersects_poly(a: &Path, b: &Poly) -> bool {
    // Check path capsules.
    for cap in a.caps() {
        if cap_intersects_poly(&cap, b) {
            return true;
        }
    }
    false
}

pub fn poly_intersects_rt(a: &Poly, b: &Rt) -> bool {
    for tri in a.tri() {
        if rt_intersects_tri(b, tri) {
            return true;
        }
    }
    false
}

pub fn rt_intersects_rt(a: &Rt, b: &Rt) -> bool {
    a.intersects(b)
}

pub fn rt_intersects_tri(a: &Rt, b: &Tri) -> bool {
    let rt = &a.pts();
    let tri = b.pts();
    // Test tri axes:
    for seg in b.segs() {
        if pts_strictly_right_of(&seg.line(), rt) {
            return false;
        }
    }
    // Test rect axes:
    for seg in a.segs() {
        if pts_strictly_right_of(&seg.line(), tri) {
            return false;
        }
    }
    true
}

pub fn rt_intersects_seg(a: &Rt, b: &Segment) -> bool {
    if a.contains(b.st()) || a.contains(b.en()) {
        return true;
    }
    // Test seg axis:
    if pts_strictly_right_of(&b.line(), &a.pts()) {
        return false;
    }
    // Test rect axes:
    for seg in a.segs() {
        if pts_strictly_right_of(&seg.line(), &[b.st(), b.en()]) {
            return false;
        }
    }
    true
}

pub fn seg_intersects_seg(a: &Segment, b: &Segment) -> bool {
    // Check if the segment endpoints are on opposite sides of the other segment.
    let a_st = orientation(&b.line(), a.st());
    let a_en = orientation(&b.line(), a.en());
    let b_st = orientation(&a.line(), b.st());
    let b_en = orientation(&a.line(), b.en());
    // No collinear points. Everything on different sides.
    if a_st != a_en && b_st != b_en {
        return true;
    }
    // Check collinear cases. Need to check both x and y coordinates to handle
    // vertical and horizontal segments.
    let a_rt = Rt::enclosing(a.st(), a.en());
    let b_rt = Rt::enclosing(b.st(), b.en());
    if a_st == 0 && b_rt.contains(a.st()) {
        return true;
    }
    if a_en == 0 && b_rt.contains(a.en()) {
        return true;
    }
    if b_st == 0 && a_rt.contains(b.st()) {
        return true;
    }
    if b_en == 0 && a_rt.contains(b.en()) {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use crate::model::primitive::{pt, rt, seg, tri};
    use crate::model::tf::Tf;

    fn test_seg_seg_permutations(a: &Segment, b: &Segment, res: bool) {
        // Try each permutation of orderings
        assert_eq!(seg_intersects_seg(a, b), res, "{} {} intersects? {}", a, b, res);
        assert_eq!(seg_intersects_seg(b, a), res, "{} {} intersects? {}", a, b, res);
        let a = seg(a.en(), a.st());
        let b = seg(b.en(), b.st());
        assert_eq!(seg_intersects_seg(&a, &b), res, "{} {} intersects? {}", a, b, res);
        assert_eq!(seg_intersects_seg(&b, &a), res, "{} {} intersects? {}", a, b, res);
    }

    #[test]
    fn test_seg_seg() {
        let tests = &[
            // Crossing
            (seg(pt(1.0, 1.0), pt(3.0, 4.0)), seg(pt(2.0, 4.0), pt(3.0, 1.0)), true),
            // Shared endpoints, not parallel
            (seg(pt(1.0, 1.0), pt(2.0, 3.0)), seg(pt(2.0, 3.0), pt(4.0, 1.0)), true),
            // Shared endpoints, parallel, one point of intersection
            (seg(pt(1.0, 1.0), pt(3.0, 2.0)), seg(pt(3.0, 2.0), pt(5.0, 3.0)), true),
            // Endpoint abutting segment, perpendicular
            (seg(pt(1.0, 1.0), pt(3.0, 3.0)), seg(pt(2.0, 4.0), pt(4.0, 2.0)), true),
            // Same segments
            (seg(pt(1.0, 1.0), pt(1.0, 1.0)), seg(pt(1.0, 1.0), pt(1.0, 1.0)), true),
            // Parallel and overlapping
            (seg(pt(1.0, 1.0), pt(3.0, 1.0)), seg(pt(2.0, 1.0), pt(4.0, 1.0)), true),
            // Parallel and contained
            (seg(pt(1.0, 1.0), pt(4.0, 1.0)), seg(pt(2.0, 1.0), pt(3.0, 1.0)), true),
            // Parallel segments with one shared endpoint overlapping
            (seg(pt(1.0, 1.0), pt(3.0, 1.0)), seg(pt(1.0, 1.0), pt(4.0, 1.0)), true),
            // Degenerate: One segment is a point, on the other segment.
            (seg(pt(1.0, 1.0), pt(3.0, 1.0)), seg(pt(2.0, 1.0), pt(2.0, 1.0)), true),
            // Degenerate: One segment is a point, on the other segment's endpoint
            (seg(pt(1.0, 1.0), pt(3.0, 1.0)), seg(pt(3.0, 1.0), pt(3.0, 1.0)), true),
            // Degenerate: Same segments and they are points
            (seg(pt(1.0, 1.0), pt(1.0, 1.0)), seg(pt(1.0, 1.0), pt(1.0, 1.0)), true),
            // Parallel, not intersecting
            (seg(pt(1.0, 3.0), pt(3.0, 1.0)), seg(pt(2.0, 4.0), pt(4.0, 2.0)), false),
            // Perpendicular, not intersecting, projection of endpoint onto other is
            // an endpoint
            (seg(pt(1.0, 1.0), pt(3.0, 3.0)), seg(pt(4.0, 2.0), pt(5.0, 1.0)), false),
            // Perpendicular, not intersecting
            (seg(pt(1.0, 1.0), pt(3.0, 3.0)), seg(pt(3.0, 1.0), pt(4.0, 0.0)), false),
            // Degenerate: Both are points, not intersecting
            (seg(pt(1.0, 1.0), pt(1.0, 1.0)), seg(pt(2.0, 1.0), pt(2.0, 1.0)), false),
            // Degenerate: One is a point, collinear with the other segment, not intersecting.
            (seg(pt(1.0, 1.0), pt(3.0, 3.0)), seg(pt(4.0, 4.0), pt(4.0, 4.0)), false),
            // Degenerate: One is a point, not intersecting.
            (seg(pt(1.0, 1.0), pt(3.0, 3.0)), seg(pt(1.0, 2.0), pt(1.0, 2.0)), false),
        ];

        for (a, b, res) in tests {
            test_seg_seg_permutations(a, b, *res);
            // Negating pts should not change result.
            let a = &seg(-a.st(), -a.en());
            let b = &seg(-b.st(), -b.en());
            test_seg_seg_permutations(a, b, *res);
            // Rotating should not change result.
            let tf = Tf::rotate(42.0);
            let a = &tf.seg(a);
            let b = &tf.seg(b);
            test_seg_seg_permutations(a, b, *res);
            // Translating should not change result.
            let tf = Tf::translate(pt(-3.0, 4.0));
            let a = &tf.seg(a);
            let b = &tf.seg(b);
            test_seg_seg_permutations(a, b, *res);
            // Scaling should not change result.
            let tf = Tf::scale(pt(-0.4, 0.7));
            let a = &tf.seg(a);
            let b = &tf.seg(b);
            test_seg_seg_permutations(a, b, *res);
        }
    }

    fn permute_tri(t: &Tri) -> Vec<Tri> {
        t.pts().iter().permutations(3).map(|v| tri(*v[0], *v[1], *v[2])).collect()
    }

    #[test]
    fn test_rt_tri() {
        let tests = &[
            // Regular intersection
            (rt(1.0, 2.0, 3.0, 3.0), tri(pt(2.0, 2.5), pt(2.0, 1.0), pt(3.0, 1.0)), true),
            // Just touching the rect.
            (rt(1.0, 2.0, 3.0, 3.0), tri(pt(3.0, 3.0), pt(4.0, 3.0), pt(4.0, 5.0)), true),
            (rt(1.0, 2.0, 3.0, 3.0), tri(pt(1.0, 4.0), pt(3.0, 4.0), pt(2.0, 5.0)), false),
            (
                rt(14.4, -148.8, 15.20, -148.0),
                tri(pt(52.5, -19.75), pt(34.0, -19.75), pt(15.0, -50.75)),
                false,
            ),
        ];

        for (a, t, res) in tests {
            for b in permute_tri(t) {
                assert_eq!(rt_intersects_tri(a, &b), *res, "{} {} intersect? {}", a, b, res);
            }
        }
    }

    #[test]
    fn test_cap_rt() {
        let tests = &[
            (cap(pt(1.0, 1.0), pt(7.0, 1.0), 1.0), rt(1.0, 1.0, 2.0, 2.0), true),
            (cap(pt(1.0, 1.0), pt(7.0, 1.0), 1.0), rt(3.0, 1.0, 3.0, 2.0), true),
            (cap(pt(122.8, -44.4), pt(109.2, -44.4), 0.32), rt(113.6, -44.8, 114.4, -44.0), true),
            (cap(pt(1.0, 1.0), pt(7.0, 1.0), 1.0), rt(3.0, 0.0, 3.0, 1.0), true),
            (cap(pt(1.0, 1.0), pt(7.0, 1.0), 1.0), rt(2.0, 3.0, 3.0, 4.0), false),
        ];

        for (a, b, res) in tests {
            assert_eq!(cap_intersects_rt(a, b), *res, "{} {} intersect? {}", a, b, res);
        }
    }
}
