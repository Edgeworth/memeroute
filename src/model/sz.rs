use auto_ops::{impl_op_ex, impl_op_ex_commutative};
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

impl_op_ex!(+ |a: &Sz, b: &Sz| -> Sz { Sz::new(a.w + b.w, a.h + b.h) });
impl_op_ex!(+= |a: &mut Sz, b: &Sz| { a.w += b.w; a.h += b.h; });
impl_op_ex!(-|a: &Sz, b: &Sz| -> Sz { Sz::new(a.w - b.w, a.h - b.h) });
impl_op_ex!(-= |a: &mut Sz, b: &Sz| { a.w -= b.w; a.h -= b.h; });

impl_op_ex_commutative!(*|a: &Sz, b: &f64| -> Sz { Sz::new(a.w * b, a.h * b) });
impl_op_ex_commutative!(/|a: &Sz, b: &f64| -> Sz { Sz::new(a.w / b, a.h / b) });
