use crate::model::shape::circle::Circle;
use crate::model::shape::path::Path;
use crate::model::shape::polygon::Polygon;
use crate::model::shape::rt::Rt;

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

pub fn polygon_contains_rect(a: &Polygon, b: &Rt) -> bool {
    let a = a.as_parry();
    a.contains_local_point(&b.bl().into())
        && a.contains_local_point(&b.br().into())
        && a.contains_local_point(&b.tl().into())
        && a.contains_local_point(&b.tr().into())
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
