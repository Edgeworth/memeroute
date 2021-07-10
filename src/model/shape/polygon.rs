use parry2d_f64::shape::{ConvexPolygon, RoundShape};

use crate::model::pt::Pt;
use crate::model::shape::rt::Rt;

#[derive(Debug, Clone)]
pub struct Polygon {
    pts: Vec<Pt>,
    width: f64,
    parry: RoundShape<ConvexPolygon>,
}

impl Polygon {
    pub fn new(pts: Vec<Pt>, width: f64) -> Self {
        let v: Vec<Point<Real>> = pts.iter().map(|p| p.into()).collect();
        let parry = RoundShape::<ConvexPolygon> {
            base_shape: ConvexPolygon::from_convex_polyline(v)
                .unwrap_or_else(|| panic!("bad polygon {:?}", pts)),
            border_radius: width / 2.0,
        };
        Self { pts, width, parry }
    }

    pub fn bounds(&self) -> Rt {
        self.parry.compute_local_aabb().into()
    }

    pub fn pts(&self) -> &[Pt] {
        &self.pts
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn as_parry(&self) -> &RoundShape<ConvexPolygon> {
        &self.parry
    }
}

impl_parry2d!(Polygon);
