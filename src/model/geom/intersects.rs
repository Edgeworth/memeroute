use crate::model::geom::distance::circ_rt_dist;
use crate::model::geom::math::{lt, pts_same_side};
use crate::model::primitive::circle::Circle;
use crate::model::primitive::line;
use crate::model::primitive::path_shape::Path;
use crate::model::primitive::polygon::Polygon;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::segment::Segment;
use crate::model::primitive::triangle::Tri;

pub fn circ_intersect_rt(a: &Circle, b: &Rt) -> bool {
    // Check if the circle centre is contained in the rect or
    // the distance from the boundary of the rect to the circle is less than 0.
    b.contains(a.p()) || lt(circ_rt_dist(a, b), 0.0)
}

pub fn path_intersects_rt(a: &Path, b: &Rt) -> bool {
    false
}

pub fn poly_intersects_rt(a: &Polygon, b: &Rt) -> bool {
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
    if pts_same_side(&line(tri[0], tri[1]), rt) {
        return false;
    }
    if pts_same_side(&line(tri[1], tri[2]), rt) {
        return false;
    }
    if pts_same_side(&line(tri[2], tri[0]), rt) {
        return false;
    }
    // Test rect axes:
    if pts_same_side(&line(rt[0], rt[1]), tri) {
        return false;
    }
    if pts_same_side(&line(rt[1], rt[2]), tri) {
        return false;
    }
    if pts_same_side(&line(rt[2], rt[3]), tri) {
        return false;
    }
    if pts_same_side(&line(rt[3], rt[0]), tri) {
        return false;
    }
    true
}

pub fn seg_intersects_rt(a: &Segment, b: &Rt) -> bool {
    todo!()
}
