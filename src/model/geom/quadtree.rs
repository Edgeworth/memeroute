use std::mem::swap;

use crate::model::geom::bounds::rt_cloud_bounds;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::ShapeOps;

type NodeIdx = usize;
pub type ShapeIdx = usize;

// How many tests to do before splitting a node.
const TEST_THRESHOLD: usize = 4;
const NO_NODE: NodeIdx = 0;

#[derive(Debug, Copy, Clone)]
struct IntersectData {
    shape_idx: ShapeIdx,
    tests: usize, // How many times we had to test against shapes directly.
}

#[derive(Debug, Clone)]
struct Node {
    intersect: Vec<IntersectData>, // Which shapes intersect this node.
    contain: Vec<ShapeIdx>,        // Which shapes contain this node.
    bl: NodeIdx,
    br: NodeIdx,
    tr: NodeIdx,
    tl: NodeIdx,
    test_count: usize,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            intersect: Vec::new(),
            contain: Vec::new(),
            bl: NO_NODE,
            br: NO_NODE,
            tr: NO_NODE,
            tl: NO_NODE,
            test_count: 0,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct QuadTree {
    shapes: Vec<Shape>,
    free_shapes: Vec<ShapeIdx>, // List of indices of shapes that have been deleted.
    nodes: Vec<Node>,
    bounds: Rt,
    // TODO: Add cache?
}

impl QuadTree {
    pub fn new(shapes: Vec<Shape>) -> Self {
        let bounds = rt_cloud_bounds(shapes.iter().map(|s| s.bounds()));
        let nodes = vec![
            Node::default(),
            Node {
                intersect: (0..shapes.len())
                    .map(|shape_idx| IntersectData { shape_idx, tests: 0 })
                    .collect(),
                ..Default::default()
            },
        ];
        Self { shapes, free_shapes: Vec::new(), nodes, bounds }
    }

    pub fn with_bounds(r: &Rt) -> Self {
        Self {
            shapes: Vec::new(),
            free_shapes: Vec::new(),
            nodes: vec![Node::default(), Node::default()],
            bounds: *r,
        }
    }

    pub fn empty() -> Self {
        Self {
            shapes: Vec::new(),
            free_shapes: Vec::new(),
            nodes: vec![Node::default(), Node::default()],
            bounds: Rt::empty(),
        }
    }

    pub fn add_shape(&mut self, s: Shape) -> ShapeIdx {
        let bounds = self.bounds().united(&s.bounds());
        // If this shape expands the bounds, rebuild the tree.
        // TODO: Don't rebuild the tree?
        if bounds != self.bounds() {
            let shape_idx = self.shapes.len();
            let mut shapes = Vec::new();
            swap(&mut shapes, &mut self.shapes);
            shapes.push(s);
            *self = Self::new(shapes);
            shape_idx
        } else {
            let shape_idx = if let Some(shape_idx) = self.free_shapes.pop() {
                self.shapes[shape_idx] = s;
                shape_idx
            } else {
                self.shapes.push(s);
                self.shapes.len() - 1
            };
            self.nodes[1].intersect.push(IntersectData { shape_idx, tests: 0 });
            shape_idx
        }
    }

    pub fn remove_shape(&mut self, s: ShapeIdx) {
        // Remove everything referencing this shape.
        for node in self.nodes.iter_mut() {
            node.intersect.retain(|v| v.shape_idx != s);
            node.contain.retain(|&v| v != s);
        }
        self.free_shapes.push(s);
    }

    pub fn bounds(&self) -> Rt {
        self.bounds
    }

    pub fn intersects(&mut self, s: &Shape) -> bool {
        self.inter(s, 1, self.bounds())
    }

    pub fn contains(&mut self, s: &Shape) -> bool {
        self.contain(s, 1, self.bounds())
    }

    pub fn dist(&mut self, _s: &Shape) -> f64 {
        todo!()
    }

    fn inter(&mut self, s: &Shape, idx: NodeIdx, r: Rt) -> bool {
        // No intersection in this node if we don't intersect the bounds.
        if !s.intersects_shape(&r.shape()) {
            return false;
        }

        // If there are any shapes containing this node they must intersect with
        // |s| since it intersects |bounds|.
        if !self.nodes[idx].contain.is_empty() {
            return true;
        }

        // TODO: Could check if |s| contains the bounds here and return true if
        // intersect is non-empty.

        // Check children, if they exist. Do this first as we expect traversing
        // the tree to be faster. Only actually do intersection tests if we have
        // to.
        if self.nodes[idx].bl != NO_NODE && self.inter(s, self.nodes[idx].bl, r.bl_quadrant()) {
            return true;
        }
        if self.nodes[idx].br != NO_NODE && self.inter(s, self.nodes[idx].br, r.br_quadrant()) {
            return true;
        }
        if self.nodes[idx].tr != NO_NODE && self.inter(s, self.nodes[idx].tr, r.tr_quadrant()) {
            return true;
        }
        if self.nodes[idx].tl != NO_NODE && self.inter(s, self.nodes[idx].tl, r.tl_quadrant()) {
            return true;
        }

        // Check shapes that intersect this node:
        let mut had_intersection = false;
        for inter in self.nodes[idx].intersect.iter_mut() {
            inter.tests += 1;
            if self.shapes[inter.shape_idx].intersects_shape(s) {
                had_intersection = true;
                break;
            }
        }
        self.maybe_push_down(idx, r);

        had_intersection
    }

    fn contain(&mut self, s: &Shape, idx: NodeIdx, r: Rt) -> bool {
        // No containment of |s| if the bounds don't intersect |s|.
        if !r.intersects_shape(s) {
            return false;
        }

        // If bounds contains |s| and there is something that contains the
        // bounds, then that contains |s|.
        if !self.nodes[idx].contain.is_empty() && r.contains_shape(s) {
            return true;
        }

        // Check children, if they exist. Do this first as we expect traversing
        // the tree to be faster. Only actually do intersection tests if we have
        // to.
        if self.nodes[idx].bl != NO_NODE && self.contain(s, self.nodes[idx].bl, r.bl_quadrant()) {
            return true;
        }
        if self.nodes[idx].br != NO_NODE && self.contain(s, self.nodes[idx].br, r.br_quadrant()) {
            return true;
        }
        if self.nodes[idx].tr != NO_NODE && self.contain(s, self.nodes[idx].tr, r.tr_quadrant()) {
            return true;
        }
        if self.nodes[idx].tl != NO_NODE && self.contain(s, self.nodes[idx].tl, r.tl_quadrant()) {
            return true;
        }

        // Check shapes that intersect this node:
        let mut had_containment = false;
        for inter in self.nodes[idx].intersect.iter_mut() {
            inter.tests += 1;
            if self.shapes[inter.shape_idx].contains_shape(s) {
                had_containment = true;
                break;
            }
        }
        self.maybe_push_down(idx, r);

        had_containment
    }

    // Move any shapes to child nodes, if necessary.
    fn maybe_push_down(&mut self, idx: NodeIdx, r: Rt) {
        let push_down: Vec<_> =
            self.nodes[idx].intersect.drain_filter(|v| v.tests >= TEST_THRESHOLD).collect();
        if !push_down.is_empty() {
            self.ensure_children(idx);

            for inter in push_down {
                let Node { bl, br, tr, tl, .. } = self.nodes[idx];
                let shape = &self.shapes[inter.shape_idx];

                // Put it into all children it intersects.
                for (quad, quad_idx) in [
                    (r.bl_quadrant().shape(), bl),
                    (r.br_quadrant().shape(), br),
                    (r.tr_quadrant().shape(), tr),
                    (r.tl_quadrant().shape(), tl),
                ] {
                    if shape.intersects_shape(&quad) {
                        self.nodes[quad_idx]
                            .intersect
                            .push(IntersectData { shape_idx: inter.shape_idx, tests: 0 });

                        if quad.contains_shape(shape) {
                            self.nodes[quad_idx].contain.push(inter.shape_idx);
                        }
                    }
                }
            }
        }
    }

    fn ensure_children(&mut self, idx: NodeIdx) {
        if self.nodes[idx].bl == NO_NODE {
            self.nodes[idx].bl = self.nodes.len();
            self.nodes.push(Node::default());
            self.nodes[idx].br = self.nodes.len();
            self.nodes.push(Node::default());
            self.nodes[idx].tr = self.nodes.len();
            self.nodes.push(Node::default());
            self.nodes[idx].tl = self.nodes.len();
            self.nodes.push(Node::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::primitive::{poly, pt, rt, tri};

    #[test]
    fn test_quadtree_tri() {
        let mut qt = QuadTree::new(vec![tri(pt(1.0, 2.0), pt(5.0, 2.0), pt(4.0, 5.0)).shape()]);
        for _ in 0..TEST_THRESHOLD {
            assert!(qt.intersects(&pt(3.0, 3.0).shape()));
        }

        assert!(qt.intersects(&pt(3.0, 3.0).shape()));
        assert!(qt.intersects(&rt(3.0, 3.0, 4.0, 4.0).shape()));
    }

    #[test]
    fn test_quadtree_poly() {
        let mut qt = QuadTree::new(vec![poly(&[pt(1.0, 2.0), pt(5.0, 2.0), pt(4.0, 5.0)]).shape()]);
        for _ in 0..TEST_THRESHOLD {
            assert!(qt.intersects(&pt(3.0, 3.0).shape()));
        }

        assert!(qt.intersects(&pt(3.0, 3.0).shape()));
        assert!(qt.intersects(&rt(3.0, 3.0, 4.0, 4.0).shape()));
    }
}
