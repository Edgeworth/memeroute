use crate::model::pt::Pt;
use crate::model::shape::circle::Circle;
use crate::model::shape::path::Path;
use crate::model::shape::polygon::Polygon;
use crate::model::shape::rt::Rt;

pub fn rect_contains_rect(a: &Rt, b: &Rt) -> bool {
    todo!()
}

pub fn rect_contains_circle(a: &Rt, b: &Circle) -> bool {
    todo!()
}

pub fn rect_contains_polygon(a: &Rt, b: &Polygon) -> bool {
    todo!()
}

pub fn rect_contains_path(a: &Rt, b: &Path) -> bool {
    todo!()
}

pub fn circle_contains_rect(a: &Circle, b: &Rt) -> bool {
    todo!()
}

pub fn circle_contains_circle(a: &Circle, b: &Circle) -> bool {
    todo!()
}

pub fn circle_contains_polygon(a: &Circle, b: &Polygon) -> bool {
    todo!()
}

pub fn circle_contains_path(a: &Circle, b: &Path) -> bool {
    todo!()
}

pub fn polygon_contains_rect(a: &Polygon, b: &Rt) -> bool {
    let a = a.as_parry();
    a.contains_local_point(&b.bl().into())
        && a.contains_local_point(&b.br().into())
        && a.contains_local_point(&b.tl().into())
        && a.contains_local_point(&b.tr().into())
}

pub fn polygon_contains_circle(a: &Polygon, b: &Circle) -> bool {
    todo!()
}

pub fn polygon_contains_polygon(a: &Polygon, b: &Polygon) -> bool {
    todo!()
}

pub fn polygon_contains_path(a: &Polygon, b: &Path) -> bool {
    todo!()
}

pub fn path_contains_rect(a: &Path, b: &Rt) -> bool {
    todo!()
}

pub fn path_contains_circle(a: &Path, b: &Circle) -> bool {
    todo!()
}

pub fn path_contains_polygon(a: &Path, b: &Polygon) -> bool {
    todo!()
}

pub fn path_contains_path(a: &Path, b: &Path) -> bool {
    todo!()
}
