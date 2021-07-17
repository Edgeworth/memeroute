use crate::model::geom::bounds::point_cloud_bounds;
use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::ShapeOps;

#[derive(Debug, Copy, Clone)]
pub struct Tri {
    pts: [Pt; 3],
}

impl Tri {
    pub const fn new(pts: [Pt; 3]) -> Self {
        Self { pts }
    }

    pub fn pts(&self) -> &[Pt; 3] {
        &self.pts
    }
}

impl ShapeOps for Tri {
    fn bounds(&self) -> Rt {
        point_cloud_bounds(&self.pts)
    }

    fn shape(self) -> Shape {
        Shape::Tri(self)
    }
}
