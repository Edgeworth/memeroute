use crate::model::primitive::circle::Circle;
use crate::model::primitive::path::Path;
use crate::model::primitive::polygon::Polygon;
use crate::model::primitive::rt::Rt;

pub fn rt_intersects_rt(_a: &Rt, _b: &Rt) -> bool {
    todo!()
}

pub fn rt_intersects_circle(_a: &Rt, _b: &Circle) -> bool {
    todo!()
}

pub fn rt_intersects_polygon(_a: &Rt, _b: &Polygon) -> bool {
    todo!()
}

pub fn rt_intersects_path(_a: &Rt, _b: &Path) -> bool {
    todo!()
}

pub fn circle_intersects_circle(_a: &Circle, _b: &Circle) -> bool {
    todo!()
}

pub fn circle_intersects_polygon(_a: &Circle, _b: &Polygon) -> bool {
    todo!()
}

pub fn circle_intersects_path(_a: &Circle, _b: &Path) -> bool {
    todo!()
}

pub fn polygon_intersects_polygon(_a: &Polygon, _b: &Polygon) -> bool {
    todo!()
}

pub fn polygon_intersects_path(_a: &Polygon, _b: &Path) -> bool {
    todo!()
}

pub fn path_intersects_path(_a: &Path, _b: &Path) -> bool {
    todo!()
}
