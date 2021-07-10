use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use approx::relative_eq;
use derive_more::Display;
use serde::{Deserialize, Serialize};

use crate::model::pt::Pt;
use crate::model::sz::Sz;

#[derive(Debug, Default, PartialEq, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {}, {}, {})", x, y, w, h)]
pub struct Rt {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rt {
    pub const fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    pub const fn ptsz(p: Pt, sz: Sz) -> Self {
        Self { x: p.x, y: p.y, w: sz.w, h: sz.h }
    }

    pub fn empty() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub fn from_sz(sz: Sz) -> Self {
        Self::ptsz(Pt::zero(), sz)
    }

    pub fn b(&self) -> f64 {
        self.y + self.h
    }

    pub fn r(&self) -> f64 {
        self.x + self.w
    }

    pub fn bl(&self) -> Pt {
        Pt::new(self.x, self.b())
    }

    pub fn br(&self) -> Pt {
        Pt::new(self.r(), self.b())
    }

    pub fn tl(&self) -> Pt {
        Pt::new(self.x, self.y)
    }

    pub fn tr(&self) -> Pt {
        Pt::new(self.r(), self.y)
    }

    pub fn sz(&self) -> Sz {
        Sz::new(self.w, self.h)
    }

    pub fn center(&self) -> Pt {
        Pt::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn area(&self) -> f64 {
        self.w * self.h
    }

    pub fn with_sz(&self, sz: Sz) -> Rt {
        Rt::ptsz(self.tl(), sz)
    }

    pub fn inset(&self, d: Sz) -> Rt {
        self.inset_xy(d.w, d.h)
    }

    pub fn inset_xy(&self, dx: f64, dy: f64) -> Rt {
        let wsub = if 2.0 * dx < self.w { 2.0 * dx } else { self.w };
        let hsub = if 2.0 * dy < self.h { 2.0 * dy } else { self.h };
        Rt::new(self.x + wsub / 2.0, self.y + hsub / 2.0, self.w - wsub, self.h - hsub)
    }

    pub fn contains(&self, p: Pt) -> bool {
        p.x >= self.x && p.y >= self.y && p.x <= self.r() && p.y <= self.b()
    }

    pub fn is_empty(&self) -> bool {
        self.w == 0.0 && self.h == 0.0
    }

    pub fn united(&self, rt: Rt) -> Rt {
        if rt.is_empty() {
            *self
        } else if self.is_empty() {
            rt
        } else {
            let x = self.x.min(rt.x);
            let y = self.y.min(rt.y);
            let r = self.r().min(rt.r());
            let b = self.b().min(rt.b());
            Rt::new(x, y, r - x, b - y)
        }
    }

    pub fn enclosing(pa: Pt, pb: Pt) -> Rt {
        let x = pa.x.min(pb.x);
        let y = pa.y.min(pb.y);
        let r = pa.x.max(pb.x);
        let b = pa.y.max(pb.y);
        Rt::new(x, y, r - x, b - y)
    }

    // Returns a rectangle with the same area that matches the aspect ratio of |r|.
    pub fn match_aspect(&self, r: &Rt) -> Rt {
        if relative_eq!(r.w, 0.0) {
            Rt::new(self.x, self.y, 0.0, self.h)
        } else if relative_eq!(r.h, 0.0) {
            Rt::new(self.x, self.y, self.w, 0.0)
        } else {
            let aspect = (r.w / r.h).sqrt();
            let len = self.area().sqrt();
            Rt::new(self.x, self.y, len * aspect, len / aspect)
        }
    }
}

impl From<Sz> for Rt {
    fn from(sz: Sz) -> Self {
        Rt::ptsz(Pt::zero(), sz)
    }
}

impl Add<Rt> for Rt {
    type Output = Rt;
    fn add(self, o: Rt) -> Self::Output {
        Rt::new(self.x + o.x, self.y + o.y, self.w + o.w, self.h + o.h)
    }
}

impl AddAssign<Rt> for Rt {
    fn add_assign(&mut self, o: Rt) {
        self.x = self.x + o.x;
        self.y = self.y + o.y;
        self.w = self.w + o.w;
        self.h = self.h + o.h;
    }
}

impl Sub<Rt> for Rt {
    type Output = Rt;
    fn sub(self, o: Rt) -> Self::Output {
        Rt::new(self.x - o.x, self.y - o.y, self.w - o.w, self.h - o.h)
    }
}

impl SubAssign<Rt> for Rt {
    fn sub_assign(&mut self, o: Rt) {
        self.x = self.x - o.x;
        self.y = self.y - o.y;
        self.w = self.w - o.w;
        self.h = self.h - o.h;
    }
}

impl Add<Pt> for Rt {
    type Output = Rt;
    fn add(self, o: Pt) -> Self::Output {
        Rt::new(self.x + o.x, self.y + o.y, self.w, self.h)
    }
}

impl AddAssign<Pt> for Rt {
    fn add_assign(&mut self, o: Pt) {
        self.x = self.x + o.x;
        self.y = self.y + o.y;
    }
}

impl Sub<Pt> for Rt {
    type Output = Rt;
    fn sub(self, o: Pt) -> Self::Output {
        Rt::new(self.x - o.x, self.y - o.y, self.w, self.h)
    }
}

impl SubAssign<Pt> for Rt {
    fn sub_assign(&mut self, o: Pt) {
        self.x = self.x - o.x;
        self.y = self.y - o.y;
    }
}

impl Mul<f64> for Rt {
    type Output = Rt;
    fn mul(self, o: f64) -> Self::Output {
        Rt::new(self.x * o, self.y * o, self.w * o, self.h * o)
    }
}

impl Mul<Rt> for f64 {
    type Output = Rt;
    fn mul(self, o: Rt) -> Self::Output {
        Rt::new(self * o.x, self * o.y, self * o.w, self * o.h)
    }
}

impl Div<f64> for Rt {
    type Output = Rt;
    fn div(self, o: f64) -> Self::Output {
        Rt::new(self.x / o, self.y / o, self.w / o, self.h / o)
    }
}

impl Div<Rt> for f64 {
    type Output = Rt;
    fn div(self, o: Rt) -> Self::Output {
        Rt::new(self / o.x, self / o.y, self / o.w, self / o.h)
    }
}

impl Mul<i64> for Rt {
    type Output = Rt;
    fn mul(self, o: i64) -> Self::Output {
        let o = o as f64;
        Rt::new(self.x * o, self.y * o, self.w * o, self.h * o)
    }
}

impl Mul<Rt> for i64 {
    type Output = Rt;
    fn mul(self, o: Rt) -> Self::Output {
        let v = self as f64;
        Rt::new(v * o.x, v * o.y, v * o.w, v * o.h)
    }
}

impl Div<i64> for Rt {
    type Output = Rt;
    fn div(self, o: i64) -> Self::Output {
        let o = o as f64;
        Rt::new(self.x / o, self.y / o, self.w / o, self.h / o)
    }
}

impl Div<Rt> for i64 {
    type Output = Rt;
    fn div(self, o: Rt) -> Self::Output {
        let v = self as f64;
        Rt::new(v / o.x, v / o.y, v / o.w, v / o.h)
    }
}

impl Mul<u64> for Rt {
    type Output = Rt;
    fn mul(self, o: u64) -> Self::Output {
        let o = o as f64;
        Rt::new(self.x * o, self.y * o, self.w * o, self.h * o)
    }
}

impl Mul<Rt> for u64 {
    type Output = Rt;
    fn mul(self, o: Rt) -> Self::Output {
        let v = self as f64;
        Rt::new(v * o.x, v * o.y, v * o.w, v * o.h)
    }
}

impl Div<u64> for Rt {
    type Output = Rt;
    fn div(self, o: u64) -> Self::Output {
        let o = o as f64;
        Rt::new(self.x / o, self.y / o, self.w / o, self.h / o)
    }
}

impl Div<Rt> for u64 {
    type Output = Rt;
    fn div(self, o: Rt) -> Self::Output {
        let v = self as f64;
        Rt::new(v / o.x, v / o.y, v / o.w, v / o.h)
    }
}
