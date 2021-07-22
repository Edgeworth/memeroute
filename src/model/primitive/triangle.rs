use std::ops::Index;

use derive_more::Display;

use crate::model::geom::bounds::pt_cloud_bounds;
use crate::model::geom::contains::tri_contains_pt;
use crate::model::geom::convex::ensure_ccw;
use crate::model::geom::intersects::{cap_intersects_tri, rt_intersects_tri};
use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::segment::Segment;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::{seg, ShapeOps};

// Is in CCW order.
#[derive(Debug, Display, Copy, Clone)]
#[display(fmt = "Tri[{}, {}, {}]", "self.pts[0]", "self.pts[1]", "self.pts[2]")]
pub struct Tri {
    pts: [Pt; 3],
}

impl Tri {
    pub fn new(mut pts: [Pt; 3]) -> Self {
        ensure_ccw(&mut pts);
        Self { pts }
    }

    pub fn pts(&self) -> &[Pt; 3] {
        &self.pts
    }

    pub fn segs(&self) -> [Segment; 3] {
        [
            seg(self.pts[0], self.pts[1]),
            seg(self.pts[1], self.pts[2]),
            seg(self.pts[2], self.pts[0]),
        ]
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
            Shape::Capsule(s) => cap_intersects_tri(s, self),
            Shape::Circle(_) => todo!(),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(_) => todo!(),
            Shape::Point(s) => tri_contains_pt(self, s),
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

impl Index<usize> for Tri {
    type Output = Pt;

    fn index(&self, index: usize) -> &Self::Output {
        &self.pts[index]
    }
}
