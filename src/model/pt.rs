use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use derive_more::Display;
use nalgebra::{vector, Vector2};
use parry2d_f64::math::{Point, Real};
use serde::{Deserialize, Serialize};

use crate::model::sz::Sz;

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

    pub fn as_sz(&self) -> Sz {
        Sz::new(self.x, self.y)
    }

    pub fn offset(&self, dx: f64, dy: f64) -> Pt {
        Pt::new(self.x + dx, self.y + dy)
    }

    pub fn cross(&self, p: Pt) -> f64 {
        self.x * p.y - self.y * p.x
    }

    pub fn perp(&self) -> Pt {
        Pt::new(-self.y, self.x).norm()
    }

    pub fn dist(&self, b: Pt) -> f64 {
        (b - *self).mag()
    }

    pub fn mag(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn norm(&self) -> Pt {
        let mag = self.mag();
        Pt::new(self.x / mag, self.y / mag)
    }
}

impl From<Pt> for Vector2<f64> {
    fn from(p: Pt) -> Self {
        vector![p.x, p.y]
    }
}

impl From<Pt> for Point<Real> {
    fn from(p: Pt) -> Self {
        Point::new(p.x, p.y)
    }
}

impl From<Point<Real>> for Pt {
    fn from(p: Point<Real>) -> Self {
        Pt::new(p.x, p.y)
    }
}

impl_op_ex!(-|a: &Pt| -> Pt { Pt::new(-a.x, -a.y) });

impl_op_ex!(+ |a: &Pt, b: &Pt| -> Pt { Pt::new(a.x + b.x, a.y + b.y) });
impl_op_ex!(+= |a: &mut Pt, b: &Pt| { a.x += b.x; a.y += b.y; });
impl_op_ex!(-|a: &Pt, b: &Pt| -> Pt { Pt::new(a.x - b.x, a.y - b.y) });
impl_op_ex!(-= |a: &mut Pt, b: &Pt| { a.x -= b.x; a.y -= b.y; });

impl_op_ex_commutative!(*|a: &Pt, b: &f64| -> Pt { Pt::new(a.x * b, a.y * b) });
impl_op_ex_commutative!(/|a: &Pt, b: &f64| -> Pt { Pt::new(a.x / b, a.y / b) });

impl_op_ex_commutative!(+ |a: &Pt, b: &Sz| -> Pt { Pt::new(a.x + b.w, a.y + b.h) });
impl_op_ex!(+= |a: &mut Pt, b: &Sz| { a.x += b.w; a.y += b.h; });
impl_op_ex_commutative!(-|a: &Pt, b: &Sz| -> Pt { Pt::new(a.x - b.w, a.y - b.h) });
impl_op_ex!(-= |a: &mut Pt, b: &Sz| { a.x -= b.w; a.y -= b.h; });

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

    pub fn zero() -> Self {
        Self::new(0, 0)
    }

    pub fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

impl_op_ex!(-|a: &PtI| -> PtI { PtI::new(-a.x, -a.y) });

impl_op_ex!(+ |a: &PtI, b: &PtI| -> PtI { PtI::new(a.x + b.x, a.y + b.y) });
impl_op_ex!(+= |a: &mut PtI, b: &PtI| { a.x += b.x; a.y += b.y; });
impl_op_ex!(-|a: &PtI, b: &PtI| -> PtI { PtI::new(a.x - b.x, a.y - b.y) });
impl_op_ex!(-= |a: &mut PtI, b: &PtI| { a.x -= b.x; a.y -= b.y; });

impl_op_ex_commutative!(*|a: &PtI, b: &i64| -> PtI { PtI::new(a.x * b, a.y * b) });
impl_op_ex_commutative!(/|a: &PtI, b: &i64| -> PtI { PtI::new(a.x / b, a.y / b) });
