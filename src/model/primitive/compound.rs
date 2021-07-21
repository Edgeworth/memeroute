use std::cell::RefCell;

use crate::model::geom::quadtree::QuadTree;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::ShapeOps;

// Represents a collection of shapes.
// Backed by a quadtree-like spatial data structure.
#[derive(Debug, Default, Clone)]
pub struct Compound {
    qt: RefCell<QuadTree>,
}

impl Compound {
    pub fn new(shapes: Vec<Shape>) -> Self {
        Self { qt: RefCell::new(QuadTree::new(shapes)) }
    }
}

impl ShapeOps for Compound {
    fn bounds(&self) -> Rt {
        self.qt.borrow().bounds()
    }

    fn shape(self) -> Shape {
        Shape::Compound(self)
    }

    fn intersects_shape(&self, s: &Shape) -> bool {
        self.qt.borrow_mut().intersects(s)
    }

    // N.B. this will check if any one shape in the compound contains |s|.
    // If |s| is covered using multiple shapes then that won't be detected.
    fn contains_shape(&self, s: &Shape) -> bool {
        self.qt.borrow_mut().contains(s)
    }

    fn dist_to_shape(&self, s: &Shape) -> f64 {
        self.qt.borrow_mut().dist(s)
    }
}
