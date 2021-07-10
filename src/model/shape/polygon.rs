use crate::model::pt::Pt;
use crate::model::shape::rt::Rt;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Polygon {
    pub pts: Vec<Pt>,
    pub width: f64,
}

impl Polygon {
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
