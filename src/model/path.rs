use crate::model::pt::Pt;
use crate::model::rt::Rt;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Path {
    pub width: f64,
    pub pts: Vec<Pt>,
}

impl Path {
    pub fn bounds(&self) -> Rt {
        let mut b = Rt::empty();
        let v = Pt::new(self.width / 2.0, self.width / 2.0);
        for p in self.pts.iter() {
            let r = Rt::enclosing(*p - v, *p + v);
            b = b.united(r);
        }
        b
    }
}
