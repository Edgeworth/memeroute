use std::f64::consts::PI;
use std::ops::Mul;

use approx::{assert_relative_eq, relative_eq};
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
        let scale = Self::scale(Pt::new(xscale, yscale));
        let offset = to.bl() - scale.pt(from.bl());
        Self::translate(offset) * scale
    }

    pub fn inv(&self) -> Tf {
        Tf { m: self.m.try_inverse().unwrap() }
    }

    pub fn pt(&self, p: Pt) -> Pt {
        let v = self.m * vector![p.x, p.y, 1.0];
        Pt::new(v.x, v.y)
    }

    // If there's a rotation, output will be a polygon not a Rt.
    pub fn rt(&self, r: &Rt) -> ShapeType {
        if relative_eq!(self.m[(1, 0)], 0.0) && relative_eq!(self.m[(0, 1)], 0.0) {
            let a = self.pt(r.bl());
            let b = self.pt(r.tr());
            ShapeType::Rect(Rt::enclosing(a, b))
        } else {
            let poly = Polygon::new(&[r.tl(), r.bl(), r.br(), r.tr()], 0.0);
            ShapeType::Polygon(self.polygon(&poly))
        }
    }

    // TODO: Assumes similarity transformation.
    fn check_similarity(&self) {
        assert_relative_eq!(self.m[(2, 0)], 0.0);
        assert_relative_eq!(self.m[(2, 1)], 0.0);
        assert_relative_eq!(self.m[(2, 2)], 1.0);
        assert_relative_eq!(self.m[(0, 0)], self.m[(1, 1)]);
        assert_relative_eq!(self.m[(0, 1)], -self.m[(1, 0)]);
    }

    pub fn length(&self, l: f64) -> f64 {
        self.check_similarity();
        l * Pt::new(self.m[(0, 0)], self.m[(1, 0)]).mag()
    }

    pub fn circle(&self, c: &Circle) -> Circle {
        Circle::new(self.pt(c.p()), self.length(c.r()))
    }

    pub fn polygon(&self, p: &Polygon) -> Polygon {
        let pts = p.pts().iter().map(|&v| self.pt(v)).collect::<Vec<_>>();
        Polygon::new(&pts, self.length(p.width()))
    }

    pub fn path(&self, p: &Path) -> Path {
        let pts = p.pts().iter().map(|&v| self.pt(v)).collect::<Vec<_>>();
        Path::new(&pts, self.length(p.width()))
    }

    pub fn shape(&self, s: &ShapeType) -> ShapeType {
        match s {
            ShapeType::Rect(s) => self.rt(s),
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
