use crate::model::geom::bounds::pt_cloud_bounds;
use crate::model::geom::intersects::rt_intersects_tri;
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
        pt_cloud_bounds(&self.pts)
    }

    fn shape(self) -> Shape {
        Shape::Tri(self)
    }

    fn intersects_shape(&self, s: &Shape) -> bool {
        match s {
            Shape::Capsule(_) => todo!(),
            Shape::Circle(_) => todo!(),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(_) => todo!(),
            Shape::Point(_) => todo!(),
            Shape::Polygon(_) => todo!(),
            Shape::Rect(s) => rt_intersects_tri(s, self),
            Shape::Segment(_) => todo!(),
            Shape::Tri(_) => todo!(),
        }
    }

    fn contains_shape(&self, s: &Shape) -> bool {
        match s {
            Shape::Capsule(_) => todo!(),
            Shape::Circle(_) => todo!(),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(_) => todo!(),
            Shape::Point(_) => todo!(),
            Shape::Polygon(_) => todo!(),
            Shape::Rect(_) => todo!(),
            Shape::Segment(_) => todo!(),
            Shape::Tri(_) => todo!(),
        }
    }

    fn dist_to_shape(&self, s: &Shape) -> f64 {
        match s {
            Shape::Capsule(_) => todo!(),
            Shape::Circle(_) => todo!(),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(_) => todo!(),
            Shape::Point(_) => todo!(),
            Shape::Polygon(_) => todo!(),
            Shape::Rect(_) => todo!(),
            Shape::Segment(_) => todo!(),
            Shape::Tri(_) => todo!(),
        }
    }
}
