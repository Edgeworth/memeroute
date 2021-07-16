use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use derive_more::Display;

use crate::model::geom::math::f64_eq;
use crate::model::primitive::shape::Shape;
use crate::model::pt::{Pt, PtI};
use crate::model::sz::Sz;

#[derive(Debug, Default, Copy, Clone, Display)]
#[display(fmt = "({}, {}, {}, {})", x, y, w, h)]
pub struct Rt {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rt {
    // x and y denotes the bottom left point of the rectangle.
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    pub fn shape(self) -> Shape {
        Shape::Rect(self)
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn w(&self) -> f64 {
        self.w
    }

    pub fn h(&self) -> f64 {
        self.h
    }

    pub fn l(&self) -> f64 {
        self.x
    }

    pub fn t(&self) -> f64 {
        self.y + self.h
    }

    pub fn r(&self) -> f64 {
        self.x + self.w
    }

    pub fn b(&self) -> f64 {
        self.y
    }

    pub fn bl(&self) -> Pt {
        Pt::new(self.l(), self.b())
    }

    pub fn br(&self) -> Pt {
        Pt::new(self.r(), self.b())
    }

    pub fn tl(&self) -> Pt {
        Pt::new(self.l(), self.t())
    }

    pub fn tr(&self) -> Pt {
        Pt::new(self.r(), self.t())
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

    pub fn inset(&self, d: Sz) -> Rt {
        self.inset_xy(d.w, d.h)
    }

    pub fn inset_xy(&self, dx: f64, dy: f64) -> Rt {
        let wsub = if 2.0 * dx < self.w { 2.0 * dx } else { self.w };
        let hsub = if 2.0 * dy < self.h { 2.0 * dy } else { self.h };
        Rt::new(self.x + wsub / 2.0, self.y + hsub / 2.0, self.w - wsub, self.h - hsub)
    }

    pub fn contains(&self, p: Pt) -> bool {
        p.x >= self.l() && p.y >= self.b() && p.x <= self.r() && p.y <= self.t()
    }

    pub fn is_empty(&self) -> bool {
        self.w == 0.0 && self.h == 0.0
    }

    pub fn united(&self, rt: &Rt) -> Rt {
        if rt.is_empty() {
            *self
        } else if self.is_empty() {
            *rt
        } else {
            let x = self.x.min(rt.x);
            let y = self.y.min(rt.y);
            let r = self.r().max(rt.r());
            let t = self.t().max(rt.t());
            Rt::new(x, y, r - x, t - y)
        }
    }

    pub fn enclosing(pa: Pt, pb: Pt) -> Rt {
        let x = pa.x.min(pb.x);
        let y = pa.y.min(pb.y);
        let r = pa.x.max(pb.x);
        let t = pa.y.max(pb.y);
        Rt::new(x, y, r - x, t - y)
    }

    // Returns a rectangle with the same area that matches the aspect ratio of |r|.
    pub fn match_aspect(&self, r: &Rt) -> Rt {
        if f64_eq(r.w, 0.0) {
            Rt::new(self.x, self.y, 0.0, self.h)
        } else if f64_eq(r.h, 0.0) {
            Rt::new(self.x, self.y, self.w, 0.0)
        } else {
            let aspect = (r.w / r.h).sqrt();
            let len = self.area().sqrt();
            Rt::new(self.x, self.y, len * aspect, len / aspect)
        }
    }
}

impl PartialEq for Rt {
    fn eq(&self, o: &Self) -> bool {
        self.tl() == o.tl() && self.br() == o.br()
    }
}

impl_op_ex!(+ |a: &Rt, b: &Rt| -> Rt { Rt::new(a.x + b.x, a.y + b.y, a.w + b.w, a.h + b.h) });
impl_op_ex!(+= |a: &mut Rt, b: &Rt| { a.x += b.x; a.y += b.y; a.w += b.w; a.h += b.h; });
impl_op_ex!(-|a: &Rt, b: &Rt| -> Rt { Rt::new(a.x - b.x, a.y - b.y, a.w - b.w, a.h - b.h) });
impl_op_ex!(-= |a: &mut Rt, b: &Rt| { a.x -= b.x; a.y -= b.y; a.w -= b.w; a.h -= b.h; });

impl_op_ex_commutative!(+|a: &Rt, b: &Pt| -> Rt { Rt::new(a.x + b.x, a.y + b.y, a.w, a.h) });
impl_op_ex_commutative!(-|a: &Rt, b: &Pt| -> Rt { Rt::new(a.x - b.x, a.y - b.y, a.w, a.h) });

impl_op_ex_commutative!(*|a: &Rt, b: &f64| -> Rt { Rt::new(a.x * b, a.y * b, a.w * b, a.h * b) });
impl_op_ex_commutative!(/|a: &Rt, b: &f64| -> Rt { Rt::new(a.x / b, a.y / b, a.w / b, a.h / b) });
impl_op_ex_commutative!(*|a: &Rt, b: &i64| -> Rt {
    let b = *b as f64;
    Rt::new(a.x * b, a.y * b, a.w * b, a.h * b)
});
impl_op_ex_commutative!(/|a: &Rt, b: &i64| -> Rt {
    let b = *b as f64;
    Rt::new(a.x / b, a.y / b, a.w / b, a.h / b)
});

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Display)]
#[display(fmt = "({}, {}, {}, {})", x, y, w, h)]
pub struct RtI {
    x: i64,
    y: i64,
    w: i64,
    h: i64,
}

impl RtI {
    pub const fn new(x: i64, y: i64, w: i64, h: i64) -> Self {
        Self { x, y, w, h }
    }


    pub const fn w(&self) -> i64 {
        self.w
    }

    pub const fn h(&self) -> i64 {
        self.h
    }

    pub const fn l(&self) -> i64 {
        self.x
    }

    pub const fn t(&self) -> i64 {
        self.y + self.h
    }

    pub const fn r(&self) -> i64 {
        self.x + self.w
    }

    pub const fn b(&self) -> i64 {
        self.y
    }

    pub const fn bl(&self) -> PtI {
        PtI::new(self.l(), self.b())
    }

    pub const fn br(&self) -> PtI {
        PtI::new(self.r(), self.b())
    }

    pub const fn tl(&self) -> PtI {
        PtI::new(self.l(), self.t())
    }

    pub const fn tr(&self) -> PtI {
        PtI::new(self.r(), self.t())
    }

    pub fn enclosing(pa: PtI, pb: PtI) -> RtI {
        let x = pa.x.min(pb.x);
        let y = pa.y.min(pb.y);
        let r = pa.x.max(pb.x);
        let t = pa.y.max(pb.y);
        RtI::new(x, y, r - x, t - y)
    }
}

impl_op_ex!(+ |a: &RtI, b: &RtI| -> RtI { RtI::new(a.x + b.x, a.y + b.y, a.w + b.w, a.h + b.h) });
impl_op_ex!(+= |a: &mut RtI, b: &RtI| { a.x += b.x; a.y += b.y; a.w += b.w; a.h += b.h; });
impl_op_ex!(-|a: &RtI, b: &RtI| -> RtI { RtI::new(a.x - b.x, a.y - b.y, a.w - b.w, a.h - b.h) });
impl_op_ex!(-= |a: &mut RtI, b: &RtI| { a.x -= b.x; a.y -= b.y; a.w -= b.w; a.h -= b.h; });

impl_op_ex_commutative!(+|a: &RtI, b: &PtI| -> RtI { RtI::new(a.x + b.x, a.y + b.y, a.w, a.h) });
impl_op_ex_commutative!(-|a: &RtI, b: &PtI| -> RtI { RtI::new(a.x - b.x, a.y - b.y, a.w, a.h) });

impl_op_ex_commutative!(*|a: &RtI, b: &i64| -> RtI {
    RtI::new(a.x * b, a.y * b, a.w * b, a.h * b)
});
