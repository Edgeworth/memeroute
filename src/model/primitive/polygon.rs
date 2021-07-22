use std::cmp::Ordering;
use std::ops::Index;

use earcutr::earcut;

use crate::model::geom::bounds::pt_cloud_bounds;
use crate::model::geom::contains::{
    poly_contains_cap, poly_contains_circ, poly_contains_path, poly_contains_pt, poly_contains_rt,
    poly_contains_seg,
};
use crate::model::geom::convex::{ensure_ccw, is_convex_ccw, remove_collinear};
use crate::model::geom::intersects::poly_intersects_rt;
use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::triangle::Tri;
use crate::model::primitive::{tri, ShapeOps};

// Represents a simple non-convex polygon.
// Stored in CCW order.
// TODO: make polygons use quadtree?
#[derive(Debug, Clone)]
pub struct Poly {
    pts: Vec<Pt>,
    tri: Vec<Tri>,
    tri_idx: Vec<u32>,
    is_convex: bool,
}

impl Poly {
    pub fn new(pts: &[Pt]) -> Self {
        let mut pts = remove_collinear(pts);
        ensure_ccw(&mut pts);
        let verts: Vec<f64> = pts.iter().map(|v| [v.x, v.y]).flatten().collect();
        let tri_idx: Vec<_> = earcut(&verts, &vec![], 2).iter().map(|&v| v as u32).collect();
        let tri = tri_idx
            .array_chunks::<3>()
            .map(|v| tri(pts[v[0] as usize], pts[v[1] as usize], pts[v[2] as usize]))
            .collect();
        let is_convex = is_convex_ccw(&pts);
        Self { pts, tri, tri_idx, is_convex }
    }

    pub fn pts(&self) -> &[Pt] {
        &self.pts
    }

    pub fn edges(&self) -> EdgeIterator<'_> {
        edges(&self.pts)
    }

    pub fn tri(&self) -> &[Tri] {
        &self.tri
    }

    pub fn tri_idx(&self) -> &[u32] {
        &self.tri_idx
    }

    pub fn is_convex(&self) -> bool {
        self.is_convex
    }
}

impl ShapeOps for Poly {
    fn bounds(&self) -> Rt {
        pt_cloud_bounds(&self.pts)
    }

    fn shape(self) -> Shape {
        Shape::Polygon(self)
    }

    fn intersects_shape(&self, s: &Shape) -> bool {
        match s {
            Shape::Capsule(_) => todo!(),
            Shape::Circle(_) => todo!(),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(_) => todo!(),
            Shape::Point(s) => poly_contains_pt(self, s),
            Shape::Polygon(_) => todo!(),
            Shape::Rect(s) => poly_intersects_rt(self, s),
            Shape::Segment(_) => todo!(),
            Shape::Tri(_) => todo!(),
        }
    }

    fn contains_shape(&self, s: &Shape) -> bool {
        match s {
            Shape::Capsule(s) => poly_contains_cap(self, s),
            Shape::Circle(s) => poly_contains_circ(self, s),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(s) => poly_contains_path(self, s),
            Shape::Point(s) => poly_contains_pt(self, s),
            Shape::Polygon(_) => todo!(),
            Shape::Rect(s) => poly_contains_rt(self, s),
            Shape::Segment(s) => poly_contains_seg(self, s),
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

impl Index<usize> for Poly {
    type Output = Pt;

    fn index(&self, index: usize) -> &Self::Output {
        &self.pts[index]
    }
}

pub struct EdgeIterator<'a> {
    pts: &'a [Pt],
    idx: usize,
}

impl<'a> EdgeIterator<'a> {
    pub fn new(pts: &'a [Pt]) -> Self {
        Self { pts, idx: 0 }
    }
}

impl<'a> Iterator for EdgeIterator<'a> {
    type Item = [&'a Pt; 2];

    fn next(&mut self) -> Option<Self::Item> {
        let edge = match self.pts.len().cmp(&(self.idx + 1)) {
            Ordering::Less => None,
            Ordering::Equal => Some([&self.pts[self.idx], &self.pts[0]]),
            Ordering::Greater => Some([&self.pts[self.idx], &self.pts[self.idx + 1]]),
        };
        self.idx += 1;
        edge
    }
}

pub fn edges(pts: &[Pt]) -> EdgeIterator<'_> {
    EdgeIterator::new(pts)
}
