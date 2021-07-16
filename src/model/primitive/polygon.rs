use earcutr::earcut;

use crate::model::geom::bounds::point_cloud_bounds;
use crate::model::geom::convex::{ensure_ccw, remove_collinear};
use crate::model::primitive::rt::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::pt::Pt;

#[derive(Debug, Clone)]
pub struct Polygon {
    pts: Vec<Pt>,
    tris: Vec<u32>,
    width: f64,
}

impl Polygon {
    pub fn new(pts: &[Pt], width: f64) -> Self {
        let mut pts = remove_collinear(pts);
        ensure_ccw(&mut pts);
        let verts: Vec<f64> = pts.iter().map(|v| [v.x, v.y]).flatten().collect();
        let tris: Vec<_> = earcut(&verts, &vec![], 2).iter().map(|&v| v as u32).collect();
        Self { pts, tris, width }
    }

    pub fn shape(self) -> Shape {
        Shape::Polygon(self)
    }

    pub fn bounds(&self) -> Rt {
        point_cloud_bounds(&self.pts).inset_xy(-self.width / 2.0, -self.width / 2.0)
    }

    pub fn pts(&self) -> &[Pt] {
        &self.pts
    }

    pub fn tris(&self) -> &[u32] {
        &self.tris
    }

    pub fn width(&self) -> f64 {
        self.width
    }
}
