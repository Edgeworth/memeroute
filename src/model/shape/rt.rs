use approx::relative_eq;
use auto_ops::{impl_op_ex, impl_op_ex_commutative};
use derive_more::Display;
use parry2d_f64::shape::ConvexPolygon;

use crate::model::pt::{Pt, PtI};
use crate::model::sz::Sz;

#[derive(Debug, Default, Clone, Display)]
#[display(fmt = "({}, {}, {}, {})", l, t, w, h)]
pub struct Rt {
    l: f64,
    t: f64,
    w: f64,
    h: f64,
    parry: Option<ConvexPolygon>,
}

impl Rt {
    pub fn new(l: f64, t: f64, w: f64, h: f64) -> Self {
        let parry = ConvexPolygon::from_convex_polyline(vec![
            Point::new(l, t),
            Point::new(l, t + h),
            Point::new(l + w, t + h),
            Point::new(l + w, t),
        ])
        .unwrap();
        Self { l, t, w, h, parry: Some(parry) }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn l(&self) -> f64 {
        self.l
    }

    pub fn t(&self) -> f64 {
        self.t
    }

    pub fn w(&self) -> f64 {
        self.w
    }

    pub fn h(&self) -> f64 {
        self.h
    }

    pub fn b(&self) -> f64 {
        self.t + self.h
    }

    pub fn r(&self) -> f64 {
        self.l + self.w
    }

    pub fn bl(&self) -> Pt {
        Pt::new(self.l, self.b())
    }

    pub fn br(&self) -> Pt {
        Pt::new(self.r(), self.b())
    }

    pub fn tl(&self) -> Pt {
        Pt::new(self.l, self.t)
    }

    pub fn tr(&self) -> Pt {
        Pt::new(self.r(), self.t)
    }

    pub fn sz(&self) -> Sz {
        Sz::new(self.w, self.h)
    }

    pub fn center(&self) -> Pt {
        Pt::new(self.l + self.w / 2.0, self.t + self.h / 2.0)
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
        Rt::new(self.l + wsub / 2.0, self.t + hsub / 2.0, self.w - wsub, self.h - hsub)
    }

    pub fn contains(&self, p: Pt) -> bool {
        p.x >= self.l && p.y >= self.t && p.x <= self.r() && p.y <= self.b()
    }

    pub fn is_empty(&self) -> bool {
        self.w == 0.0 && self.h == 0.0
    }

    pub fn united(&self, rt: &Rt) -> Rt {
        if rt.is_empty() {
            self.clone()
        } else if self.is_empty() {
            rt.clone()
        } else {
            let x = self.l.min(rt.l);
            let y = self.t.min(rt.t);
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
            Rt::new(self.l, self.t, 0.0, self.h)
        } else if relative_eq!(r.h, 0.0) {
            Rt::new(self.l, self.t, self.w, 0.0)
        } else {
            let aspect = (r.w / r.h).sqrt();
            let len = self.area().sqrt();
            Rt::new(self.l, self.t, len * aspect, len / aspect)
        }
    }

    pub fn as_parry(&self) -> &ConvexPolygon {
        self.parry.as_ref().unwrap()
    }
}

impl From<AABB> for Rt {
    fn from(r: AABB) -> Self {
        (&r).into()
    }
}

impl From<&AABB> for Rt {
    fn from(r: &AABB) -> Self {
        Rt::enclosing(r.mins.into(), r.maxs.into())
    }
}

impl PartialEq for Rt {
    fn eq(&self, o: &Self) -> bool {
        self.tl() == o.tl() && self.br() == o.br()
    }
}

impl_op_ex!(+ |a: &Rt, b: &Rt| -> Rt { Rt::new(a.l + b.l, a.t + b.t, a.w + b.w, a.h + b.h) });
impl_op_ex!(+= |a: &mut Rt, b: &Rt| { a.l += b.l; a.t += b.t; a.w += b.w; a.h += b.h; });
impl_op_ex!(-|a: &Rt, b: &Rt| -> Rt { Rt::new(a.l - b.l, a.t - b.t, a.w - b.w, a.h - b.h) });
impl_op_ex!(-= |a: &mut Rt, b: &Rt| { a.l -= b.l; a.t -= b.t; a.w -= b.w; a.h -= b.h; });

impl_op_ex_commutative!(+|a: &Rt, b: &Pt| -> Rt { Rt::new(a.l + b.x, a.t + b.y, a.w, a.h) });
impl_op_ex_commutative!(-|a: &Rt, b: &Pt| -> Rt { Rt::new(a.l - b.x, a.t - b.y, a.w, a.h) });

impl_op_ex_commutative!(*|a: &Rt, b: &f64| -> Rt { Rt::new(a.l * b, a.t * b, a.w * b, a.h * b) });
impl_op_ex_commutative!(/|a: &Rt, b: &f64| -> Rt { Rt::new(a.l / b, a.t / b, a.w / b, a.h / b) });
impl_op_ex_commutative!(*|a: &Rt, b: &i64| -> Rt {
    let b = *b as f64;
    Rt::new(a.l * b, a.t * b, a.w * b, a.h * b)
});
impl_op_ex_commutative!(/|a: &Rt, b: &i64| -> Rt {
    let b = *b as f64;
    Rt::new(a.l / b, a.t / b, a.w / b, a.h / b)
});

impl_parry2d!(Rt);

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Display)]
#[display(fmt = "({}, {}, {}, {})", x, y, w, h)]
pub struct RtI {
    pub x: i64,
    pub y: i64,
    pub w: i64,
    pub h: i64,
}

impl RtI {
    pub const fn new(x: i64, y: i64, w: i64, h: i64) -> Self {
        Self { x, y, w, h }
    }

    pub fn empty() -> Self {
        Self::new(0, 0, 0, 0)
    }

    pub fn b(&self) -> i64 {
        self.y + self.h
    }

    pub fn r(&self) -> i64 {
        self.x + self.w
    }

    pub fn bl(&self) -> PtI {
        PtI::new(self.x, self.b())
    }

    pub fn br(&self) -> PtI {
        PtI::new(self.r(), self.b())
    }

    pub fn tl(&self) -> PtI {
        PtI::new(self.x, self.y)
    }

    pub fn tr(&self) -> PtI {
        PtI::new(self.r(), self.y)
    }

    pub fn center(&self) -> PtI {
        PtI::new(self.x + self.w / 2, self.y + self.h / 2)
    }

    pub fn area(&self) -> i64 {
        self.w * self.h
    }

    pub fn contains(&self, p: PtI) -> bool {
        p.x >= self.x && p.y >= self.y && p.x <= self.r() && p.y <= self.b()
    }

    pub fn is_empty(&self) -> bool {
        self.w == 0 && self.h == 0
    }

    pub fn enclosing(pa: PtI, pb: PtI) -> RtI {
        let x = pa.x.min(pb.x);
        let y = pa.y.min(pb.y);
        let r = pa.x.max(pb.x);
        let b = pa.y.max(pb.y);
        RtI::new(x, y, r - x, b - y)
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
