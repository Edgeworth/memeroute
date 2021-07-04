use std::ops::Neg;

use derive_more::Display;
use num::Num;
use num_traits::Zero;
use paste::paste;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub trait Number = Clone + Copy + Num + Default + PartialOrd + Ord + PartialEq + Eq;

macro_rules! binop_vec2_vec2 {
    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident; $rhs_type:ty, $rhs_f1:ident, $rhs_f2:ident;) => {};

    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident; $rhs_type:ty, $rhs_f1:ident, $rhs_f2:ident;
     $op_trait:ident, $op_fn:ident, $op:tt;
     $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        impl<T: Number> std::ops::$op_trait<$rhs_type> for $lhs_type {
            type Output = $lhs_type;

            fn $op_fn(self, o: $rhs_type) -> Self::Output {
                <$lhs_type>::new(self.$lhs_f1 $op o.$rhs_f1, self.$lhs_f2 $op o.$rhs_f2)
            }
        }

        paste!{
        impl<T: Number>
                std::ops:: [<$op_trait Assign>] <$rhs_type> for $lhs_type {
            fn [<$op_fn _assign>](&mut self, o: $rhs_type) {
                self.$lhs_f1 = self.$lhs_f1 $op o.$rhs_f1;
                self.$lhs_f2 = self.$lhs_f2 $op o.$rhs_f2;
            }
        }
        }

        binop_vec2_vec2!($lhs_type, $lhs_f1, $lhs_f2; $rhs_type, $rhs_f1, $rhs_f2;
            $($op_trait_next, $op_fn_next, $op_next;)*);
    };
}

macro_rules! binop_vec4_vec4 {
    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident, $lhs_f3:ident, $lhs_f4:ident;
     $rhs_type:ty, $rhs_f1:ident, $rhs_f2:ident, $rhs_f3:ident, $rhs_f4:ident;) => {};

    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident, $lhs_f3:ident, $lhs_f4:ident;
     $rhs_type:ty, $rhs_f1:ident, $rhs_f2:ident, $rhs_f3:ident, $rhs_f4:ident;
     $op_trait:ident, $op_fn:ident, $op:tt;
     $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        impl<T: Number> std::ops::$op_trait<$rhs_type> for $lhs_type {
            type Output = $lhs_type;

            fn $op_fn(self, o: $rhs_type) -> Self::Output {
                <$lhs_type>::new(self.$lhs_f1 $op o.$rhs_f1, self.$lhs_f2 $op o.$rhs_f2,
                    self.$lhs_f3 $op o.$rhs_f3, self.$lhs_f4 $op o.$rhs_f4)
            }
        }

        paste!{
        impl<T: Number> std::ops:: [<$op_trait Assign>] <$rhs_type> for $lhs_type {
            fn [<$op_fn _assign>](&mut self, o: $rhs_type) {
                self.$lhs_f1 = self.$lhs_f1 $op o.$rhs_f1;
                self.$lhs_f2 = self.$lhs_f2 $op o.$rhs_f2;
                self.$lhs_f3 = self.$lhs_f3 $op o.$rhs_f3;
                self.$lhs_f4 = self.$lhs_f4 $op o.$rhs_f4;
            }
        }
        }

        binop_vec4_vec4!($lhs_type, $lhs_f1, $lhs_f2, $lhs_f3, $lhs_f4;
            $rhs_type, $rhs_f1, $rhs_f2, $rhs_f3, $rhs_f4;
            $($op_trait_next, $op_fn_next, $op_next;)*);
    };
}

macro_rules! binop_vec4_scalar {
    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident,
     $lhs_f3:ident, $lhs_f4:ident; $rhs_type:ty $(,)?;) => {};

    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident, $lhs_f3:ident, $lhs_f4:ident;
     $rhs_type:ty $(,)?; $op_trait:ident, $op_fn:ident, $op:tt;
     $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        const _: () = {
            type T = $rhs_type;
            impl std::ops::$op_trait<$rhs_type> for $lhs_type
            {
                type Output = $lhs_type;

                fn $op_fn(self, o: $rhs_type) -> Self::Output {
                    <$lhs_type>::new(self.$lhs_f1 $op o, self.$lhs_f2 $op o,
                        self.$lhs_f3 $op o, self.$lhs_f4 $op o)
                }
            }

           impl std::ops::$op_trait<$lhs_type> for $rhs_type
            {
                type Output = $lhs_type;

                fn $op_fn(self, o: $lhs_type) -> Self::Output {
                    <$lhs_type>::new(self $op o.$lhs_f1, self $op o.$lhs_f2,
                        self $op o.$lhs_f3, self $op o.$lhs_f4)
                }
            }
        };
        binop_vec4_scalar!($lhs_type, $lhs_f1, $lhs_f2, $lhs_f3, $lhs_f4; $rhs_type;
                $($op_trait_next, $op_fn_next, $op_next;)*);
       };

    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident, $lhs_f3:ident, $lhs_f4:ident; $rhs_type:ty
     $(,$rhs_next:ty)+ $(,)?; $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        binop_vec4_scalar!($lhs_type, $lhs_f1, $lhs_f2, $lhs_f3, $lhs_f4; $rhs_type;
            $($op_trait_next, $op_fn_next, $op_next;)*);
        binop_vec4_scalar!($lhs_type, $lhs_f1, $lhs_f2, $lhs_f3, $lhs_f4; $($rhs_next,)+;
            $($op_trait_next, $op_fn_next, $op_next;)*);
    };
}

macro_rules! binop_vec4_vec2_left {
    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident, $lhs_f3:ident, $lhs_f4:ident;
     $rhs_type:ty, $rhs_f1:ident, $rhs_f2:ident;) => {};

    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident, $lhs_f3:ident, $lhs_f4:ident;
     $rhs_type:ty, $rhs_f1:ident, $rhs_f2:ident; $op_trait:ident, $op_fn:ident, $op:tt;
     $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        impl<T: Number> std::ops::$op_trait<$rhs_type> for $lhs_type {
            type Output = $lhs_type;

            fn $op_fn(self, o: $rhs_type) -> Self::Output {
                <$lhs_type>::new(self.$lhs_f1 $op o.$rhs_f1, self.$lhs_f2 $op o.$rhs_f2,
                    self.$lhs_f3, self.$lhs_f4)
            }
        }

        paste!{
        impl<T: Number> std::ops:: [<$op_trait Assign>] <$rhs_type> for $lhs_type {
            fn [<$op_fn _assign>](&mut self, o: $rhs_type) {
                self.$lhs_f1 = self.$lhs_f1 $op o.$rhs_f1;
                self.$lhs_f2 = self.$lhs_f2 $op o.$rhs_f2;
            }
        }
        }

        binop_vec4_vec2_left!($lhs_type, $lhs_f1, $lhs_f2, $lhs_f3, $lhs_f4;
            $rhs_type, $rhs_f1, $rhs_f2; $($op_trait_next, $op_fn_next, $op_next;)*);
    };
}

macro_rules! binop_vec2_scalar {
    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident; $rhs_type:ty $(,)?;) => {};

    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident; $rhs_type:ty $(,)?;
     $op_trait:ident, $op_fn:ident, $op:tt;
     $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        const _: () = {
            type T = $rhs_type;
            impl std::ops::$op_trait<$rhs_type> for $lhs_type
            {
                type Output = $lhs_type;

                fn $op_fn(self, o: $rhs_type) -> Self::Output {
                    <$lhs_type>::new(self.$lhs_f1 $op o, self.$lhs_f2 $op o)
                }
            }

           impl std::ops::$op_trait<$lhs_type> for $rhs_type
            {
                type Output = $lhs_type;

                fn $op_fn(self, o: $lhs_type) -> Self::Output {
                    <$lhs_type>::new(self $op o.$lhs_f1, self $op o.$lhs_f2)
                }
            }
        };
        binop_vec2_scalar!($lhs_type, $lhs_f1, $lhs_f2; $rhs_type;
                $($op_trait_next, $op_fn_next, $op_next;)*);
       };

    ($lhs_type:ty, $lhs_f1:ident, $lhs_f2:ident; $rhs_type:ty $(,$rhs_next:ty)+ $(,)?;
     $($op_trait_next:ident, $op_fn_next:ident, $op_next:tt;)*) => {
        binop_vec2_scalar!($lhs_type, $lhs_f1, $lhs_f2; $rhs_type;
            $($op_trait_next, $op_fn_next, $op_next;)*);
        binop_vec2_scalar!($lhs_type, $lhs_f1, $lhs_f2; $($rhs_next,)+;
            $($op_trait_next, $op_fn_next, $op_next;)*);
    };
}

#[repr(C)]
#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {}, {}, {})", x, y, w, h)]
pub struct Rt2D<T: Number> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T: Number> Zero for Rt2D<T> {
    fn zero() -> Self {
        Self::new(T::zero(), T::zero(), T::zero(), T::zero())
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

impl<T: Number> Rt2D<T> {
    pub const fn new(x: T, y: T, w: T, h: T) -> Self {
        Self { x, y, w, h }
    }

    pub const fn ptsz(p: Pt2D<T>, sz: Sz2D<T>) -> Self {
        Self { x: p.x, y: p.y, w: sz.w, h: sz.h }
    }

    pub fn from_sz(sz: Sz2D<T>) -> Self {
        Self::ptsz(Pt2D::zero(), sz)
    }

    pub fn b(&self) -> T {
        self.y + self.h
    }

    pub fn r(&self) -> T {
        self.x + self.w
    }

    pub fn bl(&self) -> Pt2D<T> {
        Pt2D::new(self.x, self.b())
    }

    pub fn br(&self) -> Pt2D<T> {
        Pt2D::new(self.r(), self.b())
    }

    pub fn tl(&self) -> Pt2D<T> {
        Pt2D::new(self.x, self.y)
    }

    pub fn tr(&self) -> Pt2D<T> {
        Pt2D::new(self.r(), self.y)
    }

    pub fn sz(&self) -> Sz2D<T> {
        Sz2D::new(self.w, self.h)
    }

    pub fn center(&self) -> Pt2D<T> {
        let v2 = T::one() + T::one();
        Pt2D::new(self.x + self.w / v2, self.y + self.h / v2)
    }

    pub fn with_sz(&self, sz: Sz2D<T>) -> Rt2D<T> {
        Rt2D::ptsz(self.tl(), sz)
    }

    pub fn inset(&self, d: Sz2D<T>) -> Rt2D<T> {
        self.inset_xy(d.w, d.h)
    }

    pub fn inset_xy(&self, dx: T, dy: T) -> Rt2D<T> {
        let v2 = T::one() + T::one();
        let wsub = if v2 * dx < self.w { v2 * dx } else { self.w };
        let hsub = if v2 * dy < self.h { v2 * dy } else { self.h };
        Rt2D::new(self.x + wsub / v2, self.y + hsub / v2, self.w - wsub, self.h - hsub)
    }

    pub fn contains(&self, p: Pt2D<T>) -> bool {
        p.x >= self.x && p.y >= self.y && p.x <= self.r() && p.y <= self.b()
    }

    pub fn is_empty(&self) -> bool {
        self.w == T::zero() && self.h == T::zero()
    }

    pub fn united(&self, rt: &Rt2D<T>) -> Rt2D<T> {
        if rt.is_empty() {
            *self
        } else if self.is_empty() {
            *rt
        } else {
            let x = self.x.min(rt.x);
            let y = self.y.min(rt.y);
            let r = self.r().min(rt.r());
            let b = self.b().min(rt.b());
            Rt2D::new(x, y, r - x, b - y)
        }
    }

    pub fn enclosing(pa: &Pt2D<T>, pb: &Pt2D<T>) -> Rt2D<T> {
        // Make one larger for right and bottom so |b| is inside.
        let x = pa.x.min(pb.x);
        let y = pa.y.min(pb.y);
        let r = pa.x.max(pb.x);
        let b = pa.y.max(pb.y);
        Rt2D::new(x, y, r - x, b - y)
    }
}

impl<T: Number> From<Sz2D<T>> for Rt2D<T> {
    fn from(sz: Sz2D<T>) -> Self {
        Rt2D::ptsz(Pt2D::zero(), sz)
    }
}

binop_vec4_scalar!(Rt2D<T>, x, y, w, h; Decimal, i64, u64, i32, u32; Mul, mul, *; Div, div, /;);
binop_vec4_vec2_left!(Rt2D<T>, x, y, w, h; Pt2D<T>, x, y; Add, add, +; Sub, sub, -;);
binop_vec4_vec4!(Rt2D<T>, x, y, w, h; Rt2D<T>, x, y, w, h; Add, add, +; Sub, sub, -;);

#[repr(C)]
#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {})", x, y)]
pub struct Pt2D<T: Number> {
    pub x: T,
    pub y: T,
}

impl<T: Number + Neg<Output = T>> Neg for Pt2D<T> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self { x: -self.x, y: -self.y }
    }
}

impl<T: Number> Zero for Pt2D<T> {
    fn zero() -> Self {
        Self::new(T::zero(), T::zero())
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

impl<T: Number> Pt2D<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn as_array(&self) -> [T; 2] {
        [self.x, self.y]
    }

    pub fn as_sz(&self) -> Sz2D<T> {
        Sz2D::new(self.x, self.y)
    }

    pub fn offset(&self, dx: T, dy: T) -> Pt2D<T> {
        Pt2D::new(self.x + dx, self.y + dy)
    }

    pub fn cross(&self, p: &Pt2D<T>) -> T {
        self.x * p.y - self.y * p.x
    }
}

impl<T: Number> From<[T; 2]> for Pt2D<T> {
    fn from([x, y]: [T; 2]) -> Self {
        Pt2D::new(x, y)
    }
}

impl<T: Number> From<(T, T)> for Pt2D<T> {
    fn from((x, y): (T, T)) -> Self {
        Pt2D::new(x, y)
    }
}

impl<T: Number> From<&(T, T)> for Pt2D<T> {
    fn from((ref x, ref y): &(T, T)) -> Self {
        Pt2D::new(*x, *y)
    }
}

binop_vec2_vec2!(Pt2D<T>, x, y; Pt2D<T>, x, y; Add, add, +; Sub, sub, -;);
binop_vec2_vec2!(Pt2D<T>, x, y; Sz2D<T>, w, h; Add, add, +; Sub, sub, -;);

#[repr(C)]
#[derive(Debug, Default, Eq, PartialEq, Hash, Copy, Clone, Display, Serialize, Deserialize)]
#[display(fmt = "({}, {})", w, h)]
pub struct Sz2D<T: Number> {
    pub w: T,
    pub h: T,
}

impl<T: Number> Zero for Sz2D<T> {
    fn zero() -> Self {
        Self::new(T::zero(), T::zero())
    }

    fn is_zero(&self) -> bool {
        *self == Self::zero()
    }
}

impl<T: Number> Sz2D<T> {
    pub const fn new(w: T, h: T) -> Self {
        Self { w, h }
    }

    pub fn area(&self) -> T {
        self.w * self.h
    }

    pub fn min(self, o: Sz2D<T>) -> Self {
        Self::new(if self.w < o.w { self.w } else { o.w }, if self.h < o.h { self.h } else { o.h })
    }

    pub fn max(self, o: Sz2D<T>) -> Self {
        Self::new(if self.w > o.w { self.w } else { o.w }, if self.h > o.h { self.h } else { o.h })
    }
}

impl<T: Number> From<[T; 2]> for Sz2D<T> {
    fn from([w, h]: [T; 2]) -> Self {
        Sz2D::new(w, h)
    }
}

impl<T: Number> From<(T, T)> for Sz2D<T> {
    fn from((w, h): (T, T)) -> Self {
        Sz2D::new(w, h)
    }
}

impl<T: Number> From<Sz2D<T>> for (T, T) {
    fn from(sz: Sz2D<T>) -> Self {
        (sz.w, sz.h)
    }
}

binop_vec2_vec2!(Sz2D<T>, w, h; Sz2D<T>, w, h; Add, add, +; Sub, sub, -; Div, div, /;);
binop_vec2_scalar!(Sz2D<T>, w, h; Decimal, i64, u64, i32, u32; Mul, mul, *; Div, div, /;);

pub type RtF = Rt2D<Decimal>;
pub type PtF = Pt2D<Decimal>;
pub type SzF = Sz2D<Decimal>;

pub type Rt = Rt2D<i64>;
pub type Pt = Pt2D<i64>;
pub type Sz = Sz2D<i64>;
