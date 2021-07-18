use approx::{AbsDiffEq, RelativeEq};
use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use derive_more::Display;
use nalgebra::{vector, Vector2};
use serde::{Deserialize, Serialize};

use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::{pt, pti, rt, ShapeOps};

#[derive(Debug, Default, PartialEq, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {})", x, y)]
pub struct Pt {
    pub x: f64,
    pub y: f64,
}

impl Pt {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn is_zero(&self) -> bool {
        *self == Self::zero()
    }

    pub fn as_array(&self) -> [f64; 2] {
        [self.x, self.y]
    }

    pub fn offset(&self, dx: f64, dy: f64) -> Pt {
        pt(self.x + dx, self.y + dy)
    }

    pub fn cross(&self, p: Pt) -> f64 {
        self.x * p.y - self.y * p.x
    }

    pub fn perp(&self) -> Pt {
        pt(-self.y, self.x).norm()
    }

    pub fn dist(&self, b: Pt) -> f64 {
        (b - *self).mag()
    }

    pub fn mag(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn norm(&self) -> Pt {
        let mag = self.mag();
        pt(self.x / mag, self.y / mag)
    }

    // Clamps the point to be in the range defined by |r|.
    pub fn clamp(&self, r: &Rt) -> Pt {
        pt(self.x.clamp(r.l(), r.r()), self.y.clamp(r.b(), r.t()))
    }
}

impl AbsDiffEq for Pt {
    type Epsilon = f64;

    fn default_epsilon() -> f64 {
        f64::default_epsilon()
    }

    fn abs_diff_eq(&self, o: &Self, epsilon: f64) -> bool {
        f64::abs_diff_eq(&self.x, &o.x, epsilon) && f64::abs_diff_eq(&self.y, &o.y, epsilon)
    }
}

impl RelativeEq for Pt {
    fn default_max_relative() -> f64 {
        f64::default_max_relative()
    }

    fn relative_eq(&self, o: &Self, epsilon: f64, max_relative: f64) -> bool {
        f64::relative_eq(&self.x, &o.x, epsilon, max_relative)
            && f64::relative_eq(&self.y, &o.y, epsilon, max_relative)
    }
}

impl From<Pt> for Vector2<f64> {
    fn from(p: Pt) -> Self {
        vector![p.x, p.y]
    }
}

impl ShapeOps for Pt {
    fn bounds(&self) -> Rt {
        rt(self.x, self.y, 0.0, 0.0)
    }

    fn shape(self) -> Shape {
        Shape::Point(self)
    }
}

impl_op_ex!(-|a: &Pt| -> Pt { pt(-a.x, -a.y) });

impl_op_ex!(+ |a: &Pt, b: &Pt| -> Pt { pt(a.x + b.x, a.y + b.y) });
impl_op_ex!(+= |a: &mut Pt, b: &Pt| { a.x += b.x; a.y += b.y; });
impl_op_ex!(-|a: &Pt, b: &Pt| -> Pt { pt(a.x - b.x, a.y - b.y) });
impl_op_ex!(-= |a: &mut Pt, b: &Pt| { a.x -= b.x; a.y -= b.y; });

impl_op_ex_commutative!(*|a: &Pt, b: &f64| -> Pt { pt(a.x * b, a.y * b) });
impl_op_ex_commutative!(/|a: &Pt, b: &f64| -> Pt { pt(a.x / b, a.y / b) });

#[derive(Debug, Default, PartialEq, Eq, Hash, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {})", x, y)]
pub struct PtI {
    pub x: i64,
    pub y: i64,
}

impl PtI {
    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self::new(0, 0)
    }

    pub fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

impl_op_ex!(-|a: &PtI| -> PtI { pti(-a.x, -a.y) });

impl_op_ex!(+ |a: &PtI, b: &PtI| -> PtI { pti(a.x + b.x, a.y + b.y) });
impl_op_ex!(+= |a: &mut PtI, b: &PtI| { a.x += b.x; a.y += b.y; });
impl_op_ex!(-|a: &PtI, b: &PtI| -> PtI { pti(a.x - b.x, a.y - b.y) });
impl_op_ex!(-= |a: &mut PtI, b: &PtI| { a.x -= b.x; a.y -= b.y; });

impl_op_ex_commutative!(*|a: &PtI, b: &i64| -> PtI { pti(a.x * b, a.y * b) });
impl_op_ex_commutative!(/|a: &PtI, b: &i64| -> PtI { pti(a.x / b, a.y / b) });