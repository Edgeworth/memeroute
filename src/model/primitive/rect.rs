use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use derive_more::Display;

use crate::model::geom::math::{eq, ge, gt, le, lt};
use crate::model::primitive::point::{Pt, PtI};
use crate::model::primitive::shape::Shape;
use crate::model::primitive::{pt, pti, rt, ShapeOps};

#[derive(Debug, Copy, Clone, Display)]
#[display(fmt = "({}, {}, {}, {})", l, b, r, t)]
pub struct Rt {
    l: f64,
    b: f64,
    r: f64,
    t: f64,
}

impl Default for Rt {
    fn default() -> Self {
        Self::empty()
    }
}

// Rt covers the range [l, r] . [b, t]. Empty rectangles have r < l or t < b.
// Note that a rectangle with l == r and b == t may not be empty although have
// a width and height of zero.
impl Rt {
    pub const fn new(l: f64, b: f64, r: f64, t: f64) -> Self {
        Self { l, b, r, t }
    }

    pub const fn empty() -> Self {
        rt(0.0, 0.0, -1.0, -1.0)
    }

    pub fn is_empty(&self) -> bool {
        lt(self.r, self.l) || lt(self.t, self.b)
    }

    pub fn w(&self) -> f64 {
        self.r - self.l
    }

    pub fn h(&self) -> f64 {
        self.t - self.b
    }

    pub const fn l(&self) -> f64 {
        self.l
    }

    pub const fn t(&self) -> f64 {
        self.t
    }

    pub const fn r(&self) -> f64 {
        self.r
    }

    pub const fn b(&self) -> f64 {
        self.b
    }

    pub const fn bl(&self) -> Pt {
        pt(self.l(), self.b())
    }

    pub const fn br(&self) -> Pt {
        pt(self.r(), self.b())
    }

    pub const fn tl(&self) -> Pt {
        pt(self.l(), self.t())
    }

    pub const fn tr(&self) -> Pt {
        pt(self.r(), self.t())
    }

    pub const fn pts(&self) -> [Pt; 4] {
        [self.bl(), self.br(), self.tr(), self.tl()]
    }

    pub fn center(&self) -> Pt {
        pt((self.l + self.r) / 2.0, (self.b + self.t) / 2.0)
    }

    pub fn area(&self) -> f64 {
        self.w() * self.h()
    }

    // Insetting a rectangle more than its size will produce a rectangle
    // containing the single center point .
    pub fn inset(&self, dx: f64, dy: f64) -> Rt {
        let wsub = self.w().min(2.0 * dx) / 2.0;
        let hsub = self.h().min(2.0 * dy) / 2.0;
        rt(self.l + wsub, self.b + hsub, self.r - wsub, self.t - hsub)
    }

    pub fn contains(&self, p: Pt) -> bool {
        ge(p.x, self.l()) && ge(p.y, self.b()) && le(p.x, self.r()) && le(p.y, self.t())
    }

    pub fn intersects(&self, r: &Rt) -> bool {
        le(self.l(), r.r()) && ge(self.r(), r.l()) && gt(self.t(), r.b()) && le(self.b(), r.t())
    }

    pub fn united(&self, rect: &Rt) -> Rt {
        if rect.is_empty() {
            *self
        } else if self.is_empty() {
            *rect
        } else {
            let l = self.l.min(rect.l);
            let b = self.b.min(rect.b);
            let r = self.r().max(rect.r());
            let t = self.t().max(rect.t());
            rt(l, b, r, t)
        }
    }

    pub fn enclosing(pa: Pt, pb: Pt) -> Rt {
        let l = pa.x.min(pb.x);
        let b = pa.y.min(pb.y);
        let r = pa.x.max(pb.x);
        let t = pa.y.max(pb.y);
        rt(l, b, r, t)
    }

    // Returns a rectangle with the same area that matches the aspect ratio of |r|.
    pub fn match_aspect(&self, r: &Rt) -> Rt {
        if eq(r.w(), 0.0) {
            rt(self.l, self.b, self.l, self.t)
        } else if eq(r.h(), 0.0) {
            rt(self.l, self.b, self.r, self.b)
        } else {
            let aspect = (r.w() / r.h()).sqrt();
            let len = self.area().sqrt();
            rt(self.l, self.b, self.l + len * aspect, self.b + len / aspect)
        }
    }
}

impl PartialEq for Rt {
    fn eq(&self, o: &Self) -> bool {
        self.tl() == o.tl() && self.br() == o.br()
    }
}

impl ShapeOps for Rt {
    fn bounds(&self) -> Rt {
        *self
    }

    fn shape(self) -> Shape {
        Shape::Rect(self)
    }
}

// impl_op_ex!(+ |a: &Rt, b: &Rt| -> Rt { rt(a.l + b.l, a.b + b.b, a.w + b.w, a.h + b.h) });
// impl_op_ex!(+= |a: &mut Rt, b: &Rt| { a.l += b.l; a.b += b.b; a.w += b.w; a.h += b.h; });
// impl_op_ex!(-|a: &Rt, b: &Rt| -> Rt { rt(a.l - b.l, a.b - b.b, a.w - b.w, a.h - b.h) });
// impl_op_ex!(-= |a: &mut Rt, b: &Rt| { a.l -= b.l; a.b -= b.b; a.w -= b.w; a.h -= b.h; });

// impl_op_ex_commutative!(+|a: &Rt, b: &Pt| -> Rt { rt(a.l + b.x, a.b + b.y, a.w, a.h) });
// impl_op_ex_commutative!(-|a: &Rt, b: &Pt| -> Rt { rt(a.l - b.x, a.b - b.y, a.w, a.h) });

impl_op_ex_commutative!(*|a: &Rt, b: &f64| -> Rt { rt(a.l * b, a.b * b, a.r * b, a.t * b) });
impl_op_ex_commutative!(/|a: &Rt, b: &f64| -> Rt { rt(a.l / b, a.b / b, a.r / b, a.t / b) });
impl_op_ex_commutative!(*|a: &Rt, b: &i64| -> Rt {
    let b = *b as f64;
    rt(a.l * b, a.b * b, a.r * b, a.t * b)
});
impl_op_ex_commutative!(/|a: &Rt, b: &i64| -> Rt {
    let b = *b as f64;
    rt(a.l / b, a.b / b, a.r / b, a.t / b)
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
        pti(self.l(), self.b())
    }

    pub const fn br(&self) -> PtI {
        pti(self.r(), self.b())
    }

    pub const fn tl(&self) -> PtI {
        pti(self.l(), self.t())
    }

    pub const fn tr(&self) -> PtI {
        pti(self.r(), self.t())
    }

    pub fn inset(&self, dx: i64, dy: i64) -> RtI {
        let wsub = if 2 * dx < self.w { 2 * dx } else { self.w };
        let hsub = if 2 * dy < self.h { 2 * dy } else { self.h };
        RtI::new(self.x + wsub / 2, self.y + hsub / 2, self.w - wsub, self.h - hsub)
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
