use std::ops::Mul;

use num::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::model::geom::{Pt, Rt};

#[derive(Debug, Default, PartialEq, Copy, Clone)]
pub struct Tf {
    m: [Decimal; 9], // Row major.
}

impl Tf {
    pub const fn new() -> Self {
        Self::identity()
    }

    pub const fn identity() -> Self {
        Self {
            m: [dec!(1), dec!(0), dec!(0), dec!(0), dec!(1), dec!(0), dec!(0), dec!(0), dec!(1)],
        }
    }

    pub const fn scale(p: Pt) -> Self {
        Self { m: [p.x, dec!(0), dec!(0), dec!(0), p.y, dec!(0), dec!(0), dec!(0), dec!(1)] }
    }

    pub const fn translate(p: Pt) -> Self {
        Self { m: [dec!(1), dec!(0), p.x, dec!(0), dec!(1), p.y, dec!(0), dec!(0), dec!(1)] }
    }

    pub fn rotate(r: Decimal) -> Self {
        let cos = Decimal::from_f64(r.to_f64().unwrap().cos()).unwrap();
        let sin = Decimal::from_f64(r.to_f64().unwrap().sin()).unwrap();
        Self { m: [cos, -sin, dec!(0), sin, cos, dec!(0), dec!(0), dec!(0), dec!(1)] }
    }

    pub fn affine(from: Rt, to: Rt) -> Self {
        let xscale = to.w / from.w;
        let yscale = to.h / from.h;
        let offset = to.tl() - from.tl();
        Self::translate(offset) * Self::scale(Pt::new(xscale, yscale))
    }

    pub fn concat(&self, tf: &Tf) -> Tf {
        let mut m =
            [dec!(0), dec!(0), dec!(0), dec!(0), dec!(0), dec!(0), dec!(0), dec!(0), dec!(0)];
        for r in 0..3 {
            for k in 0..3 {
                for c in 0..3 {
                    m[r * 3 + c] += self.m[r * 3 + k] * tf.m[k * 3 + c];
                }
            }
        }
        Tf { m }
    }

    pub fn inv(&self) -> Tf {
        let m = &self.m;
        let d = m[0] * (m[4] * m[8] - m[7] * m[5]) - m[1] * (m[3] * m[8] - m[5] * m[6])
            + m[2] * (m[3] * m[7] - m[4] * m[6]);

        let inv = [
            (m[4] * m[8] - m[7] * m[5]) / d,
            (m[2] * m[7] - m[1] * m[8]) / d,
            (m[1] * m[5] - m[2] * m[4]) / d,
            (m[5] * m[6] - m[3] * m[8]) / d,
            (m[0] * m[8] - m[2] * m[6]) / d,
            (m[3] * m[2] - m[0] * m[5]) / d,
            (m[3] * m[7] - m[6] * m[4]) / d,
            (m[6] * m[1] - m[0] * m[7]) / d,
            (m[0] * m[4] - m[3] * m[1]) / d,
        ];
        Tf { m: inv }
    }

    pub fn pt(&self, p: Pt) -> Pt {
        Pt::new(
            p.x * self.m[0] + p.y * self.m[1] + self.m[2],
            p.x * self.m[3] + p.y * self.m[4] + self.m[5],
        )
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
        self.concat(&rhs)
    }
}

impl Mul<Tf> for &Tf {
    type Output = Tf;

    fn mul(self, rhs: Tf) -> Self::Output {
        self.concat(&rhs)
    }
}

impl Mul<&Tf> for Tf {
    type Output = Tf;

    fn mul(self, rhs: &Tf) -> Self::Output {
        self.concat(rhs)
    }
}
