use crate::model::pt::Pt;
use crate::model::rt::Rt;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Circle {
    pub r: f64, // Radius
    pub p: Pt,  // Center
}

impl Circle {
    pub fn bounds(&self) -> Rt {
        let v = Pt::new(self.r, self.r);
        Rt::enclosing(self.p - v, self.p + v)
    }

    pub fn contains(&self, p: Pt) -> bool {
        false
    }
}
