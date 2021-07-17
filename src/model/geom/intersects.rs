use crate::model::geom::math::pts_same_side;
use crate::model::primitive::line;
use crate::model::primitive::polygon::Polygon;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::triangle::Tri;

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
    let rt = &[a.bl(), a.br(), a.tr(), a.tl()];
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
