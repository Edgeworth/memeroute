use std::f64::consts::PI;
use std::ops::Mul;

use approx::assert_relative_eq;
use nalgebra::{vector, Matrix3};

use crate::model::pt::Pt;
use crate::model::shape::circle::Circle;
use crate::model::shape::path::Path;
use crate::model::shape::polygon::Polygon;
use crate::model::shape::rt::Rt;
use crate::model::shape::shape_type::ShapeType;

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

    pub fn affine(from: &Rt, to: &Rt) -> Self {
        let xscale = to.w() / from.w();
        let yscale = to.h() / from.h();
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

    pub fn rt(&self, r: &Rt) -> Rt {
        let a = self.pt(r.tl());
        let b = self.pt(r.br());
        Rt::enclosing(a, b)
    }

    pub fn circle(&self, c: &Circle) -> Circle {
        let radii = self.pt(Pt::new(c.r(), c.r()));
        // TODO: Assumes similarity transformation.
        assert_relative_eq!(radii.x, radii.y);
        Circle::new(radii.x, self.pt(c.p()))
    }

    pub fn polygon(&self, p: &Polygon) -> Polygon {
        let w = self.pt(Pt::new(p.width(), p.width()));
        // TODO: Assumes similarity transformation.
        assert_relative_eq!(w.x, w.y);
        Polygon::new(p.pts().iter().map(|&v| self.pt(v)).collect(), w.x)
    }

    pub fn path(&self, p: &Path) -> Path {
        let w = self.pt(Pt::new(p.width(), p.width()));
        // TODO: Assumes similarity transformation.
        assert_relative_eq!(w.x, w.y);
        Path::new(p.pts().iter().map(|&v| self.pt(v)).collect(), w.x)
    }

    pub fn shape(&self, s: &ShapeType) -> ShapeType {
        match s {
            ShapeType::Rect(s) => ShapeType::Rect(self.rt(s)),
            ShapeType::Circle(s) => ShapeType::Circle(self.circle(s)),
            ShapeType::Polygon(s) => ShapeType::Polygon(self.polygon(s)),
            ShapeType::Path(s) => ShapeType::Path(self.path(s)),
            ShapeType::Arc(_) => todo!(),
        }
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
