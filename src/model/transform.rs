use std::f64::consts::PI;
use std::ops::Mul;

use nalgebra::{vector, Matrix3};

use crate::model::geom::{Pt, Rt};

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Tf {
    m: Matrix3<f64>,
}

impl Tf {
    pub fn new() -> Self {
        Self::identity()
    }

    pub fn identity() -> Self {
        Self { m: Matrix3::identity() }
    }

    pub fn scale(p: Pt) -> Self {
        Self { m: Matrix3::new_nonuniform_scaling(&p.into()) }
    }

    pub fn translate(p: Pt) -> Self {
        Self { m: Matrix3::new_translation(&p.into()) }
    }

    pub fn rotate(deg: f64) -> Self {
        Self { m: Matrix3::new_rotation(deg / 180.0 * PI) }
    }

    pub fn affine(from: Rt, to: Rt) -> Self {
        let xscale = to.w / from.w;
        let yscale = to.h / from.h;
        let offset = to.tl() - from.tl();
        Self::translate(offset) * Self::scale(Pt::new(xscale, yscale))
    }

    pub fn inv(&self) -> Tf {
        Tf { m: self.m.try_inverse().unwrap() }
    }

    pub fn pt(&self, p: Pt) -> Pt {
        let v = self.m * vector![p.x, p.y, 1.0];
        Pt::new(v.x, v.y)
    }

    pub fn rt(&self, r: Rt) -> Rt {
        let a = self.pt(r.tl());
        let b = self.pt(r.br());
        Rt::enclosing(a, b)
    }

    pub fn pts(&self, p: &[Pt]) -> Vec<Pt> {
        p.iter().map(|&v| self.pt(v)).collect()
    }
}

impl Mul<Tf> for Tf {
    type Output = Tf;

    fn mul(self, rhs: Tf) -> Self::Output {
        Tf { m: self.m * rhs.m }
    }
}

impl Mul<Tf> for &Tf {
    type Output = Tf;

    fn mul(self, rhs: Tf) -> Self::Output {
        Tf { m: self.m * rhs.m }
    }
}

impl Mul<&Tf> for Tf {
    type Output = Tf;

    fn mul(self, rhs: &Tf) -> Self::Output {
        Tf { m: self.m * rhs.m }
    }
}

impl Mul<&Tf> for &Tf {
    type Output = Tf;

    fn mul(self, rhs: &Tf) -> Self::Output {
        Tf { m: self.m * rhs.m }
    }
}
