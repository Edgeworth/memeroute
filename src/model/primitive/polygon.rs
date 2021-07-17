use earcutr::earcut;

use crate::model::geom::bounds::point_cloud_bounds;
use crate::model::geom::convex::{ensure_ccw, remove_collinear};
use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::triangle::Tri;
use crate::model::primitive::{tri, ShapeOps};

// Represents a simple non-convex polygon.
#[derive(Debug, Clone)]
pub struct Polygon {
    pts: Vec<Pt>,
    tri: Vec<Tri>,
    tri_idx: Vec<u32>,
}

impl Polygon {
    pub fn new(pts: &[Pt]) -> Self {
        let mut pts = remove_collinear(pts);
        ensure_ccw(&mut pts);
        let verts: Vec<f64> = pts.iter().map(|v| [v.x, v.y]).flatten().collect();
        let tri_idx: Vec<_> = earcut(&verts, &vec![], 2).iter().map(|&v| v as u32).collect();
        let tri = tri_idx
            .array_chunks::<3>()
            .map(|v| tri(pts[v[0] as usize], pts[v[1] as usize], pts[v[2] as usize]))
            .collect();
        Self { pts, tri, tri_idx }
    }




    pub fn pts(&self) -> &[Pt] {
        &self.pts
    }

    pub fn tri(&self) -> &[Tri] {
        &self.tri
    }

    pub fn tri_idx(&self) -> &[u32] {
        &self.tri_idx
    }
}

impl ShapeOps for Polygon {
    fn bounds(&self) -> Rt {
        point_cloud_bounds(&self.pts)
    }

    fn shape(self) -> Shape {
        Shape::Polygon(self)
    }
}
