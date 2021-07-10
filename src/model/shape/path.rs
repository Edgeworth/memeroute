use parry2d_f64::shape::Compound;

use crate::model::pt::Pt;
use crate::model::shape::rt::Rt;

#[derive(Clone)]
pub struct Path {
    pts: Vec<Pt>,
    width: f64,
    parry: Compound,
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?} {:?}", self.pts, self.width))
    }
}

impl Path {
    pub fn new(pts: Vec<Pt>, width: f64) -> Self {
        let parry = Compound::new(vec![]);
        Self { pts, width, parry }
    }

    pub fn bounds(&self) -> Rt {
        self.parry.local_aabb().into()
    }

    pub fn pts(&self) -> &[Pt] {
        &self.pts
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    fn as_parry(&self) -> &Compound {
        &self.parry
    }
}

impl_parry2d!(Path);
