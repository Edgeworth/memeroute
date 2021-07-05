use std::ops::{Add, AddAssign, Div, DivAssign, Mul, Neg, Sub, SubAssign};

use derive_more::Display;
use num_traits::Zero;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {}, {}, {})", x, y, w, h)]
pub struct Rt {
    pub x: Decimal,
    pub y: Decimal,
    pub w: Decimal,
    pub h: Decimal,
}

impl Zero for Rt {
    fn zero() -> Self {
        Self::new(dec!(0), dec!(0), dec!(0), dec!(0))
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

impl Rt {
    pub const fn new(x: Decimal, y: Decimal, w: Decimal, h: Decimal) -> Self {
        Self { x, y, w, h }
    }

    pub const fn ptsz(p: Pt, sz: Sz) -> Self {
        Self { x: p.x, y: p.y, w: sz.w, h: sz.h }
    }

    pub fn from_sz(sz: Sz) -> Self {
        Self::ptsz(Pt::zero(), sz)
    }

    pub fn b(&self) -> Decimal {
        self.y + self.h
    }

    pub fn r(&self) -> Decimal {
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
        Pt::new(self.x + self.w / dec!(2), self.y + self.h / dec!(2))
    }

    pub fn with_sz(&self, sz: Sz) -> Rt {
        Rt::ptsz(self.tl(), sz)
    }

    pub fn inset(&self, d: Sz) -> Rt {
        self.inset_xy(d.w, d.h)
    }

    pub fn inset_xy(&self, dx: Decimal, dy: Decimal) -> Rt {
        let wsub = if dec!(2) * dx < self.w { dec!(2) * dx } else { self.w };
        let hsub = if dec!(2) * dy < self.h { dec!(2) * dy } else { self.h };
        Rt::new(self.x + wsub / dec!(2), self.y + hsub / dec!(2), self.w - wsub, self.h - hsub)
    }

    pub fn contains(&self, p: Pt) -> bool {
        p.x >= self.x && p.y >= self.y && p.x <= self.r() && p.y <= self.b()
    }

    pub fn is_empty(&self) -> bool {
        self.w.is_zero() && self.h.is_zero()
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

impl Mul<Decimal> for Rt {
    type Output = Rt;
    fn mul(self, o: Decimal) -> Self::Output {
        Rt::new(self.x * o, self.y * o, self.w * o, self.h * o)
    }
}

impl Mul<Rt> for Decimal {
    type Output = Rt;
    fn mul(self, o: Rt) -> Self::Output {
        Rt::new(self * o.x, self * o.y, self * o.w, self * o.h)
    }
}

impl Div<Decimal> for Rt {
    type Output = Rt;
    fn div(self, o: Decimal) -> Self::Output {
        Rt::new(self.x / o, self.y / o, self.w / o, self.h / o)
    }
}

impl Div<Rt> for Decimal {
    type Output = Rt;
    fn div(self, o: Rt) -> Self::Output {
        Rt::new(self / o.x, self / o.y, self / o.w, self / o.h)
    }
}

impl Mul<i64> for Rt {
    type Output = Rt;
    fn mul(self, o: i64) -> Self::Output {
        let o = Decimal::new(o, 0);
        Rt::new(self.x * o, self.y * o, self.w * o, self.h * o)
    }
}

impl Mul<Rt> for i64 {
    type Output = Rt;
    fn mul(self, o: Rt) -> Self::Output {
        let v = Decimal::new(self, 0);
        Rt::new(v * o.x, v * o.y, v * o.w, v * o.h)
    }
}

impl Div<i64> for Rt {
    type Output = Rt;
    fn div(self, o: i64) -> Self::Output {
        let o = Decimal::new(o, 0);
        Rt::new(self.x / o, self.y / o, self.w / o, self.h / o)
    }
}

impl Div<Rt> for i64 {
    type Output = Rt;
    fn div(self, o: Rt) -> Self::Output {
        let v = Decimal::new(self, 0);
        Rt::new(v / o.x, v / o.y, v / o.w, v / o.h)
    }
}

impl Mul<u64> for Rt {
    type Output = Rt;
    fn mul(self, o: u64) -> Self::Output {
        let o = Decimal::new(o as i64, 0);
        Rt::new(self.x * o, self.y * o, self.w * o, self.h * o)
    }
}

impl Mul<Rt> for u64 {
    type Output = Rt;
    fn mul(self, o: Rt) -> Self::Output {
        let v = Decimal::new(self as i64, 0);
        Rt::new(v * o.x, v * o.y, v * o.w, v * o.h)
    }
}

impl Div<u64> for Rt {
    type Output = Rt;
    fn div(self, o: u64) -> Self::Output {
        let o = Decimal::new(o as i64, 0);
        Rt::new(self.x / o, self.y / o, self.w / o, self.h / o)
    }
}

impl Div<Rt> for u64 {
    type Output = Rt;
    fn div(self, o: Rt) -> Self::Output {
        let v = Decimal::new(self as i64, 0);
        Rt::new(v / o.x, v / o.y, v / o.w, v / o.h)
    }
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {})", x, y)]
pub struct Pt {
    pub x: Decimal,
    pub y: Decimal,
}

impl Neg for Pt {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y }
    }
}

impl Zero for Pt {
    fn zero() -> Self {
        Self::new(Decimal::zero(), Decimal::zero())
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

impl Pt {
    pub const fn new(x: Decimal, y: Decimal) -> Self {
        Self { x, y }
    }

    pub fn as_array(&self) -> [Decimal; 2] {
        [self.x, self.y]
    }

    pub fn as_sz(&self) -> Sz {
        Sz::new(self.x, self.y)
    }

    pub fn offset(&self, dx: Decimal, dy: Decimal) -> Pt {
        Pt::new(self.x + dx, self.y + dy)
    }

    pub fn cross(&self, p: Pt) -> Decimal {
        self.x * p.y - self.y * p.x
    }

    pub fn perp(&self) -> Pt {
        Pt::new(-self.y, self.x)
    }

    pub fn dist(&self, b: Pt) -> Decimal {
        (b - *self).mag()
    }

    pub fn mag(&self) -> Decimal {
        (self.x * self.x + self.y * self.y).sqrt().unwrap()
    }
}

impl From<[Decimal; 2]> for Pt {
    fn from([x, y]: [Decimal; 2]) -> Self {
        Pt::new(x, y)
    }
}

impl From<(Decimal, Decimal)> for Pt {
    fn from((x, y): (Decimal, Decimal)) -> Self {
        Pt::new(x, y)
    }
}

impl From<&(Decimal, Decimal)> for Pt {
    fn from((ref x, ref y): &(Decimal, Decimal)) -> Self {
        Pt::new(*x, *y)
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

impl Mul<Decimal> for Pt {
    type Output = Pt;
    fn mul(self, o: Decimal) -> Self::Output {
        Pt::new(self.x * o, self.y * o)
    }
}

impl Div<Decimal> for Pt {
    type Output = Pt;
    fn div(self, o: Decimal) -> Self::Output {
        Pt::new(self.x / o, self.y / o)
    }
}

impl Mul<Pt> for Decimal {
    type Output = Pt;
    fn mul(self, o: Pt) -> Self::Output {
        Pt::new(self * o.x, self * o.y)
    }
}

impl Div<Pt> for Decimal {
    type Output = Pt;
    fn div(self, o: Pt) -> Self::Output {
        Pt::new(self / o.x, self / o.y)
    }
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {})", w, h)]
pub struct Sz {
    pub w: Decimal,
    pub h: Decimal,
}

impl Zero for Sz {
    fn zero() -> Self {
        Self::new(Decimal::zero(), Decimal::zero())
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

impl Sz {
    pub const fn new(w: Decimal, h: Decimal) -> Self {
        Self { w, h }
    }

    pub fn area(&self) -> Decimal {
        self.w * self.h
    }

    pub fn min(self, o: Sz) -> Self {
        Self::new(if self.w < o.w { self.w } else { o.w }, if self.h < o.h { self.h } else { o.h })
    }

    pub fn max(self, o: Sz) -> Self {
        Self::new(if self.w > o.w { self.w } else { o.w }, if self.h > o.h { self.h } else { o.h })
    }
}

impl From<[Decimal; 2]> for Sz {
    fn from([w, h]: [Decimal; 2]) -> Self {
        Sz::new(w, h)
    }
}

impl From<(Decimal, Decimal)> for Sz {
    fn from((w, h): (Decimal, Decimal)) -> Self {
        Sz::new(w, h)
    }
}

impl From<Sz> for (Decimal, Decimal) {
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

impl Mul<Decimal> for Sz {
    type Output = Sz;
    fn mul(self, o: Decimal) -> Self::Output {
        Sz::new(self.w * o, self.h * o)
    }
}

impl Div<Decimal> for Sz {
    type Output = Sz;
    fn div(self, o: Decimal) -> Self::Output {
        Sz::new(self.w / o, self.h / o)
    }
}

impl Mul<Sz> for Decimal {
    type Output = Sz;
    fn mul(self, o: Sz) -> Self::Output {
        Sz::new(self * o.w, self * o.h)
    }
}

impl Div<Sz> for Decimal {
    type Output = Sz;
    fn div(self, o: Sz) -> Self::Output {
        Sz::new(self / o.w, self / o.h)
    }
}
