use std::cell::{Ref, RefCell};

use crate::model::geom::quadtree::{QuadTree, ShapeIdx, Tag};
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

    pub fn empty() -> Self {
        Self { qt: RefCell::new(QuadTree::empty()) }
    }

    pub fn with_bounds(r: &Rt) -> Self {
        Self { qt: RefCell::new(QuadTree::with_bounds(r)) }
    }

    pub fn add_shape(&self, s: Shape, tag: &Tag) -> Vec<ShapeIdx> {
        self.qt.borrow_mut().add_shape(s, tag)
    }

    pub fn remove_shape(&mut self, s: ShapeIdx) {
        self.qt.borrow_mut().remove_shape(s)
    }

    pub fn intersects(&self, s: &Shape, q: TagQuery) -> bool {
        self.qt.borrow_mut().intersects(s, q)
    }

    // N.B. this will check if any one shape in the compound contains |s|.
    // If |s| is covered using multiple shapes then that won't be detected.
    pub fn contains(&self, s: &Shape, q: TagQuery) -> bool {
        self.qt.borrow_mut().contains(s, q)
    }

    pub fn dist(&self, s: &Shape, q: TagQuery) -> f64 {
        self.qt.borrow_mut().dist(s, q)
    }

    pub fn quadtree(&self) -> Ref<'_, QuadTree> {
        self.qt.borrow()
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
        self.qt.borrow_mut().intersects(s, TagQuery::All)
    }

    // N.B. this will check if any one shape in the compound contains |s|.
    // If |s| is covered using multiple shapes then that won't be detected.
    fn contains_shape(&self, s: &Shape) -> bool {
        self.qt.borrow_mut().contains(s, TagQuery::All)
    }

    fn dist_to_shape(&self, s: &Shape) -> f64 {
        self.qt.borrow_mut().dist(s, TagQuery::All)
    }
}
