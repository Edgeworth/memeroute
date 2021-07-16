use parry2d_f64::shape::{Capsule, Compound, Segment, SharedShape};

use crate::model::geom::convex::remove_collinear;
use crate::model::pt::Pt;
use crate::model::primitive::identity;
use crate::model::primitive::rt::Rt;
use crate::model::primitive::shape::Shape;

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
    pub fn new(pts: &[Pt], width: f64) -> Self {
        let pts = remove_collinear(pts);
        let mut v = Vec::new();
        for [a, b] in pts.array_windows::<2>() {
            v.push((
                identity(),
                SharedShape::new(Capsule {
                    segment: Segment { a: a.into(), b: b.into() },
                    radius: width / 2.0,
                }),
            ));
        }
        let parry = Compound::new(v);
        Self { pts, width, parry }
    }

    pub fn shape(self) -> Shape {
        Shape::Path(self)
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

    pub fn as_parry(&self) -> &Compound {
        &self.parry
    }
}

impl_parry2d!(Path);