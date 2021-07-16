use crate::model::primitive::circle::Circle;
use crate::model::primitive::path::Path;
use crate::model::primitive::polygon::Polygon;
use crate::model::primitive::rt::Rt;

pub fn rt_contains_rt(_a: &Rt, _b: &Rt) -> bool {
    todo!()
}

pub fn rt_contains_circle(_a: &Rt, _b: &Circle) -> bool {
    todo!()
}

pub fn rt_contains_polygon(_a: &Rt, _b: &Polygon) -> bool {
    todo!()
}

pub fn rt_contains_path(_a: &Rt, _b: &Path) -> bool {
    todo!()
}

pub fn circle_contains_rt(_a: &Circle, _b: &Rt) -> bool {
    todo!()
}

pub fn circle_contains_circle(_a: &Circle, _b: &Circle) -> bool {
    todo!()
}

pub fn circle_contains_polygon(_a: &Circle, _b: &Polygon) -> bool {
    todo!()
}

pub fn circle_contains_path(_a: &Circle, _b: &Path) -> bool {
    todo!()
}

pub fn polygon_contains_rt(_a: &Polygon, _b: &Rt) -> bool {
    // TODO: can't just test point containment for non-convex polygon.
    todo!()
}

pub fn polygon_contains_circle(_a: &Polygon, _b: &Circle) -> bool {
    todo!()
}

pub fn polygon_contains_polygon(_a: &Polygon, _b: &Polygon) -> bool {
    todo!()
}

pub fn polygon_contains_path(_a: &Polygon, _b: &Path) -> bool {
    todo!()
}

pub fn path_contains_rt(_a: &Path, _b: &Rt) -> bool {
    todo!()
}

pub fn path_contains_circle(_a: &Path, _b: &Circle) -> bool {
    todo!()
}

pub fn path_contains_polygon(_a: &Path, _b: &Polygon) -> bool {
    todo!()
}

pub fn path_contains_path(_a: &Path, _b: &Path) -> bool {
    todo!()
}
