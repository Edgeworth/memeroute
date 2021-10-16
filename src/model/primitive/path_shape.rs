use std::ops::Index;

use crate::model::geom::bounds::pt_cloud_bounds;
use crate::model::geom::contains::{path_contains_rt, path_contains_seg};
use crate::model::geom::convex::remove_collinear;
use crate::model::geom::distance::{cap_path_dist, circ_path_dist, path_poly_dist, rt_path_dist};
use crate::model::geom::intersects::{
    cap_intersects_path, circ_intersects_path, path_intersects_path, path_intersects_poly,
    path_intersects_rt,
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
    bounds: Rt,
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?} {:?}", self.pts, self.r))
    }
}

impl Path {
    pub fn new(pts: &[Pt], r: f64) -> Self {
        let pts = remove_collinear(pts);
        let bounds = pt_cloud_bounds(&pts).inset(-r / 2.0, -r / 2.0);
        Self { pts, r, bounds }
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
        self.bounds
    }

    fn shape(self) -> Shape {
        Shape::Path(self)
    }

    fn intersects_shape(&self, s: &Shape) -> bool {
        match s {
            Shape::Capsule(s) => cap_intersects_path(s, self),
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
            Shape::Capsule(s) => cap_path_dist(s, self),
            Shape::Circle(s) => circ_path_dist(s, self),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(_) => todo!(),
            Shape::Point(_) => todo!(),
            Shape::Polygon(s) => path_poly_dist(self, s),
            Shape::Rect(s) => rt_path_dist(s, self),
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
