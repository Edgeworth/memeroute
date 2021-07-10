use crate::model::pt::Pt;
use crate::model::shape::rt::Rt;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Circle {
    r: f64, // Radius
    p: Pt,  // Center
}

impl Circle {
    pub fn new(r: f64, p: Pt) -> Self {
        Self { r, p }
    }

    pub fn bounds(&self) -> Rt {
        let v = Pt::new(self.r, self.r);
        Rt::enclosing(self.p - v, self.p + v)
    }

    pub fn r(&self) -> f64 {
        self.r
    }

    pub fn p(&self) -> Pt {
        self.p
    }
}
