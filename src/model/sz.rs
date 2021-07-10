use std::ops::{Add, AddAssign, Div, DivAssign, Mul, Sub, SubAssign};

use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {})", w, h)]
pub struct Sz {
    pub w: f64,
    pub h: f64,
}

impl Sz {
    pub const fn new(w: f64, h: f64) -> Self {
        Self { w, h }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn is_zero(&self) -> bool {
        *self == Self::zero()
    }

    pub fn area(&self) -> f64 {
        self.w * self.h
    }

    pub fn min(self, o: Sz) -> Self {
        Self::new(if self.w < o.w { self.w } else { o.w }, if self.h < o.h { self.h } else { o.h })
    }

    pub fn max(self, o: Sz) -> Self {
        Self::new(if self.w > o.w { self.w } else { o.w }, if self.h > o.h { self.h } else { o.h })
    }
}

impl From<[f64; 2]> for Sz {
    fn from([w, h]: [f64; 2]) -> Self {
        Sz::new(w, h)
    }
}

impl From<(f64, f64)> for Sz {
    fn from((w, h): (f64, f64)) -> Self {
        Sz::new(w, h)
    }
}

impl From<Sz> for (f64, f64) {
    fn from(sz: Sz) -> Self {
        (sz.w, sz.h)
    }
}

impl Add<Sz> for Sz {
    type Output = Sz;
    fn add(self, o: Sz) -> Self::Output {
        Sz::new(self.w + o.w, self.h + o.h)
    }
}

impl AddAssign<Sz> for Sz {
    fn add_assign(&mut self, o: Sz) {
        self.w = self.w + o.w;
        self.h = self.h + o.h;
    }
}

impl Sub<Sz> for Sz {
    type Output = Sz;
    fn sub(self, o: Sz) -> Self::Output {
        Sz::new(self.w - o.w, self.h - o.h)
    }
}

impl SubAssign<Sz> for Sz {
    fn sub_assign(&mut self, o: Sz) {
        self.w = self.w - o.w;
        self.h = self.h - o.h;
    }
}

impl Div<Sz> for Sz {
    type Output = Sz;
    fn div(self, o: Sz) -> Self::Output {
        Sz::new(self.w / o.w, self.h / o.h)
    }
}

impl DivAssign<Sz> for Sz {
    fn div_assign(&mut self, o: Sz) {
        self.w = self.w / o.w;
        self.h = self.h / o.h;
    }
}

impl Mul<f64> for Sz {
    type Output = Sz;
    fn mul(self, o: f64) -> Self::Output {
        Sz::new(self.w * o, self.h * o)
    }
}

impl Div<f64> for Sz {
    type Output = Sz;
    fn div(self, o: f64) -> Self::Output {
        Sz::new(self.w / o, self.h / o)
    }
}

impl Mul<Sz> for f64 {
    type Output = Sz;
    fn mul(self, o: Sz) -> Self::Output {
        Sz::new(self * o.w, self * o.h)
    }
}

impl Div<Sz> for f64 {
    type Output = Sz;
    fn div(self, o: Sz) -> Self::Output {
        Sz::new(self / o.w, self / o.h)
    }
}
