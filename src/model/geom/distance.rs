use crate::model::primitive::circle::Circle;
use crate::model::primitive::rect::Rt;

// Returns the distance from the circle to the boundary of the
// rectangle.
pub fn circ_rt_dist(a: &Circle, b: &Rt) -> f64 {
    // Project circle centre onto the rectangle:
    let p = a.p().clamp(b);
    p.dist(a.p()) - a.r()
}
