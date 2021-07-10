
use crate::model::pt::Pt;
use crate::model::shape::rt::Rt;

#[derive(Debug, Clone)]
pub struct Circle {
    parry: Capsule,
}

impl Circle {
    pub fn new(r: f64, p: Pt) -> Self {
        Self { parry: Capsule { segment: Segment { a: p.into(), b: p.into() }, radius: r } }
    }

    pub fn bounds(&self) -> Rt {
        self.parry.local_aabb().into()
    }

    pub fn r(&self) -> f64 {
        self.parry.radius
    }

    pub fn p(&self) -> Pt {
        self.parry.segment.a.into()
    }

    fn as_parry(&self) -> &Capsule {
        &self.parry
    }
}

impl_parry2d!(Circle);