use crate::model::primitive::circle::Circle;
use crate::model::primitive::path::Path;
use crate::model::primitive::polygon::Polygon;
use crate::model::primitive::rt::Rt;

pub fn rect_contains_rect(_a: &Rt, _b: &Rt) -> bool {
    todo!()
}

pub fn rect_contains_circle(_a: &Rt, _b: &Circle) -> bool {
    todo!()
}

pub fn rect_contains_polygon(_a: &Rt, _b: &Polygon) -> bool {
    todo!()
}

pub fn rect_contains_path(_a: &Rt, _b: &Path) -> bool {
    todo!()
}

pub fn circle_contains_rect(_a: &Circle, _b: &Rt) -> bool {
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

pub fn polygon_contains_rect(_a: &Polygon, _b: &Rt) -> bool {
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

pub fn path_contains_rect(_a: &Path, _b: &Rt) -> bool {
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
