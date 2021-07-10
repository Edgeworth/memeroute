use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

use derive_more::Display;
use nalgebra::{vector, Vector2};
use serde::{Deserialize, Serialize};

use crate::model::sz::Sz;

#[derive(Debug, Default, PartialEq, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {})", x, y)]
pub struct Pt {
    pub x: f64,
    pub y: f64,
}

impl Neg for Pt {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y }
    }
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

impl From<[f64; 2]> for Pt {
    fn from([x, y]: [f64; 2]) -> Self {
        Pt::new(x, y)
    }
}

impl From<(f64, f64)> for Pt {
    fn from((x, y): (f64, f64)) -> Self {
        Pt::new(x, y)
    }
}

impl From<&(f64, f64)> for Pt {
    fn from((ref x, ref y): &(f64, f64)) -> Self {
        Pt::new(*x, *y)
    }
}

impl From<Pt> for Vector2<f64> {
    fn from(p: Pt) -> Self {
        vector![p.x, p.y]
    }
}

impl Add<Pt> for Pt {
    type Output = Pt;
    fn add(self, o: Pt) -> Self::Output {
        Pt::new(self.x + o.x, self.y + o.y)
    }
}

impl AddAssign<Pt> for Pt {
    fn add_assign(&mut self, o: Pt) {
        self.x = self.x + o.x;
        self.y = self.y + o.y;
    }
}

impl Sub<Pt> for Pt {
    type Output = Pt;
    fn sub(self, o: Pt) -> Self::Output {
        Pt::new(self.x - o.x, self.y - o.y)
    }
}

impl SubAssign<Pt> for Pt {
    fn sub_assign(&mut self, o: Pt) {
        self.x = self.x - o.x;
        self.y = self.y - o.y;
    }
}

impl Add<Sz> for Pt {
    type Output = Pt;
    fn add(self, o: Sz) -> Self::Output {
        Pt::new(self.x + o.w, self.y + o.h)
    }
}

impl AddAssign<Sz> for Pt {
    fn add_assign(&mut self, o: Sz) {
        self.x = self.x + o.w;
        self.y = self.y + o.h;
    }
}

impl Sub<Sz> for Pt {
    type Output = Pt;
    fn sub(self, o: Sz) -> Self::Output {
        Pt::new(self.x - o.w, self.y - o.h)
    }
}

impl SubAssign<Sz> for Pt {
    fn sub_assign(&mut self, o: Sz) {
        self.x = self.x - o.w;
        self.y = self.y - o.h;
    }
}

impl Mul<f64> for Pt {
    type Output = Pt;
    fn mul(self, o: f64) -> Self::Output {
        Pt::new(self.x * o, self.y * o)
    }
}

impl Div<f64> for Pt {
    type Output = Pt;
    fn div(self, o: f64) -> Self::Output {
        Pt::new(self.x / o, self.y / o)
    }
}

impl Mul<Pt> for f64 {
    type Output = Pt;
    fn mul(self, o: Pt) -> Self::Output {
        Pt::new(self * o.x, self * o.y)
    }
}

impl Div<Pt> for f64 {
    type Output = Pt;
    fn div(self, o: Pt) -> Self::Output {
        Pt::new(self / o.x, self / o.y)
    }
}
