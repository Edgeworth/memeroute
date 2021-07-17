use crate::model::primitive::polygon::Polygon;
use crate::model::primitive::rect::Rt;

pub fn poly_contains_rt(_a: &Polygon, _b: &Rt) -> bool {
    // TODO: can't just test point containment for non-convex polygon.
    todo!()
}
