use crate::model::geom::bounds::point_cloud_bounds;
use crate::model::primitive::rt::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::pt::Pt;

#[derive(Debug, Copy, Clone)]
pub struct Tri {
    pts: [Pt; 3],
}

impl Tri {
    pub fn new(pts: [Pt; 3]) -> Self {
        Self { pts }
    }

    pub fn shape(self) -> Shape {
        Shape::Tri(self)
    }

    pub fn bounds(&self) -> Rt {
        point_cloud_bounds(&self.pts)
    }

    pub fn pts(&self) -> &[Pt; 3] {
        &self.pts
    }
}
