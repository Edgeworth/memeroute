use std::ops::Index;

use crate::model::geom::bounds::pt_cloud_bounds;
use crate::model::geom::contains::{path_contains_rt, path_contains_seg};
use crate::model::geom::convex::remove_collinear;
use crate::model::geom::intersects::{
    circ_intersects_path, path_intersects_path, path_intersects_poly, path_intersects_rt,
};
use crate::model::primitive::capsule::Capsule;
use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::{cap, ShapeOps};

#[derive(Clone)]
pub struct Path {
    pts: Vec<Pt>,
    r: f64,
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?} {:?}", self.pts, self.r))
    }
}

impl Path {
    pub fn new(pts: &[Pt], r: f64) -> Self {
        Self { pts: remove_collinear(pts), r }
    }

    pub fn len(&self) -> usize {
        self.pts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn pts(&self) -> &[Pt] {
        &self.pts
    }

    pub fn caps(&self) -> impl '_ + Iterator<Item = Capsule> {
        self.pts.array_windows::<2>().map(move |v| cap(v[0], v[1], self.r))
    }

    pub const fn r(&self) -> f64 {
        self.r
    }
}

impl ShapeOps for Path {
    fn bounds(&self) -> Rt {
        pt_cloud_bounds(&self.pts).inset(-self.r / 2.0, -self.r / 2.0)
    }

    fn shape(self) -> Shape {
        Shape::Path(self)
    }

    fn intersects_shape(&self, s: &Shape) -> bool {
        match s {
            Shape::Capsule(_) => todo!(),
            Shape::Circle(s) => circ_intersects_path(s, self),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(s) => path_intersects_path(self, s),
            Shape::Point(_) => todo!(),
            Shape::Polygon(s) => path_intersects_poly(self, s),
            Shape::Rect(s) => path_intersects_rt(self, s),
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
            Shape::Rect(s) => path_contains_rt(self, s),
            Shape::Segment(s) => path_contains_seg(self, s),
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

impl Index<usize> for Path {
    type Output = Pt;

    fn index(&self, index: usize) -> &Self::Output {
        &self.pts[index]
    }
}
