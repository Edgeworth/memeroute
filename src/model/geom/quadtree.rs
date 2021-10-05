use std::collections::HashMap;
use std::mem::swap;

use crate::model::geom::bounds::rt_cloud_bounds;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::ShapeOps;

type NodeIdx = usize;
pub type ShapeIdx = usize;
pub type Tag = usize;

pub const NO_TAG: usize = usize::MAX;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Query {
    All,
    One(Tag),
    Except(Tag),
}

// How many tests to do before splitting a node.
const TEST_THRESHOLD: usize = 4;
const MAX_DEPTH: usize = 7;
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
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct QuadTree {
    shapes: Vec<(Shape, Tag)>,
    free_shapes: Vec<ShapeIdx>, // List of indices of shapes that have been deleted.
    nodes: Vec<Node>,
    bounds: Rt,
    intersect_cache: HashMap<ShapeIdx, bool>, // Caches intersection tests.
    contain_cache: HashMap<ShapeIdx, bool>,   // Caches containment tests.
}

impl QuadTree {
    pub fn new(shapes: Vec<(Shape, Tag)>) -> Self {
        let bounds = rt_cloud_bounds(shapes.iter().map(|s| s.0.bounds()));
        let nodes = vec![
            Node::default(),
            Node {
                intersect: (0..shapes.len())
                    .map(|shape_idx| IntersectData { shape_idx, tests: 0 })
                    .collect(),
                ..Default::default()
            },
        ];
        Self { shapes, nodes, bounds, ..Default::default() }
    }

    pub fn with_bounds(r: &Rt) -> Self {
        Self { nodes: vec![Node::default(), Node::default()], bounds: *r, ..Default::default() }
    }

    pub fn empty() -> Self {
        Self {
            nodes: vec![Node::default(), Node::default()],
            bounds: Rt::empty(),
            ..Default::default()
        }
    }

    // Gets the current rectangles of the quad tree.
    pub fn rts(&self) -> Vec<Rt> {
        let mut rts = Vec::new();
        self.rts_internal(1, self.bounds(), &mut rts);
        rts
    }

    pub fn shapes(&self) -> &[(Shape, Tag)] {
        &self.shapes
    }

    fn rts_internal(&self, idx: NodeIdx, r: Rt, rts: &mut Vec<Rt>) {
        if idx == NO_NODE {
            return;
        }
        rts.push(r);
        self.rts_internal(self.nodes[idx].bl, r.bl_quadrant(), rts);
        self.rts_internal(self.nodes[idx].br, r.br_quadrant(), rts);
        self.rts_internal(self.nodes[idx].tr, r.tr_quadrant(), rts);
        self.rts_internal(self.nodes[idx].tl, r.tl_quadrant(), rts);
    }

    // Split paths up so they are spread out more.
    // Split compound shapes up.
    fn decompose_shape(s: Shape) -> Vec<Shape> {
        if let Shape::Compound(s) = s {
            s.quadtree().shapes().iter().map(|v| v.0.clone()).collect()
        } else if let Shape::Path(s) = s {
            s.caps().map(|v| v.shape()).collect()
        } else {
            vec![s]
        }
    }

    pub fn add_shape(&mut self, s: Shape, tag: Tag) -> Vec<ShapeIdx> {
        let bounds = self.bounds().united(&s.bounds());
        // If this shape expands the bounds, rebuild the tree.
        // TODO: Don't rebuild the tree?
        let s = Self::decompose_shape(s);
        let mut shape_idxs = Vec::new();
        if bounds != self.bounds() {
            let mut shapes = Vec::new();
            swap(&mut shapes, &mut self.shapes);
            for shape in s {
                shape_idxs.push(shapes.len());
                shapes.push((shape, tag));
            }
            *self = Self::new(shapes);
        } else {
            for shape in s {
                let shape_idx = if let Some(shape_idx) = self.free_shapes.pop() {
                    self.shapes[shape_idx] = (shape, tag);
                    shape_idx
                } else {
                    self.shapes.push((shape, tag));
                    self.shapes.len() - 1
                };
                shape_idxs.push(shape_idx);
                self.nodes[1].intersect.push(IntersectData { shape_idx, tests: 0 });
            }
        }
        shape_idxs
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

    fn reset_cache(&mut self) {
        self.intersect_cache.clear();
        self.contain_cache.clear();
    }

    pub fn intersects(&mut self, s: &Shape, q: Query) -> bool {
        self.reset_cache();
        self.inter(s, &q, 1, self.bounds(), 0)
    }

    pub fn contains(&mut self, s: &Shape, q: Query) -> bool {
        self.reset_cache();
        self.contain(s, &q, 1, self.bounds(), 0)
    }

    pub fn dist(&mut self, _s: &Shape, _q: Query) -> f64 {
        todo!()
    }

    fn cached_intersects(
        shapes: &[(Shape, Tag)],
        cache: &mut HashMap<ShapeIdx, bool>,
        idx: ShapeIdx,
        s: &Shape,
        q: &Query,
    ) -> bool {
        match q {
            Query::One(tag) if tag != &shapes[idx].1 => return false,
            Query::Except(tag) if tag == &shapes[idx].1 => return false,
            _ => {}
        }
        if let Some(res) = cache.get(&idx) {
            *res
        } else {
            let res = shapes[idx].0.intersects_shape(s);
            cache.insert(idx, res);
            res
        }
    }

    fn cached_contains(
        shapes: &[(Shape, Tag)],
        cache: &mut HashMap<ShapeIdx, bool>,
        idx: ShapeIdx,
        s: &Shape,
        q: &Query,
    ) -> bool {
        match q {
            Query::One(tag) if tag != &shapes[idx].1 => return false,
            Query::Except(tag) if tag == &shapes[idx].1 => return false,
            _ => {}
        }
        if let Some(res) = cache.get(&idx) {
            *res
        } else {
            let res = shapes[idx].0.contains_shape(s);
            cache.insert(idx, res);
            res
        }
    }

    fn inter(&mut self, s: &Shape, q: &Query, idx: NodeIdx, r: Rt, depth: usize) -> bool {
        // No intersection in this node if we don't intersect the bounds.
        if !s.intersects_shape(&r.shape()) {
            return false;
        }

        // If there are any shapes containing this node they must intersect with
        // |s| since it intersects |bounds|.
        if *q == Query::All && !self.nodes[idx].contain.is_empty() {
            return true;
        }

        // TODO: Could check if |s| contains the bounds here and return true if
        // intersect is non-empty.

        // Check children, if they exist. Do this first as we expect traversing
        // the tree to be faster. Only actually do intersection tests if we have
        // to.
        if self.nodes[idx].bl != NO_NODE
            && self.inter(s, q, self.nodes[idx].bl, r.bl_quadrant(), depth + 1)
        {
            return true;
        }
        if self.nodes[idx].br != NO_NODE
            && self.inter(s, q, self.nodes[idx].br, r.br_quadrant(), depth + 1)
        {
            return true;
        }
        if self.nodes[idx].tr != NO_NODE
            && self.inter(s, q, self.nodes[idx].tr, r.tr_quadrant(), depth + 1)
        {
            return true;
        }
        if self.nodes[idx].tl != NO_NODE
            && self.inter(s, q, self.nodes[idx].tl, r.tl_quadrant(), depth + 1)
        {
            return true;
        }

        // Check shapes that intersect this node:
        let mut had_intersection = false;
        for inter in self.nodes[idx].intersect.iter_mut() {
            inter.tests += 1;
            if Self::cached_intersects(
                &self.shapes,
                &mut self.intersect_cache,
                inter.shape_idx,
                s,
                q,
            ) {
                had_intersection = true;
                break;
            }
        }
        self.maybe_push_down(idx, r, depth);

        had_intersection
    }

    fn contain(&mut self, s: &Shape, q: &Query, idx: NodeIdx, r: Rt, depth: usize) -> bool {
        // No containment of |s| if the bounds don't intersect |s|.
        if !r.intersects_shape(s) {
            return false;
        }

        // If bounds contains |s| and there is something that contains the
        // bounds, then that contains |s|.
        if *q == Query::All && !self.nodes[idx].contain.is_empty() && r.contains_shape(s) {
            return true;
        }

        // Check children, if they exist. Do this first as we expect traversing
        // the tree to be faster. Only actually do intersection tests if we have
        // to.
        if self.nodes[idx].bl != NO_NODE
            && self.contain(s, q, self.nodes[idx].bl, r.bl_quadrant(), depth + 1)
        {
            return true;
        }
        if self.nodes[idx].br != NO_NODE
            && self.contain(s, q, self.nodes[idx].br, r.br_quadrant(), depth + 1)
        {
            return true;
        }
        if self.nodes[idx].tr != NO_NODE
            && self.contain(s, q, self.nodes[idx].tr, r.tr_quadrant(), depth + 1)
        {
            return true;
        }
        if self.nodes[idx].tl != NO_NODE
            && self.contain(s, q, self.nodes[idx].tl, r.tl_quadrant(), depth + 1)
        {
            return true;
        }

        // Check shapes that intersect this node:
        let mut had_containment = false;
        for inter in self.nodes[idx].intersect.iter_mut() {
            inter.tests += 1;
            if Self::cached_contains(&self.shapes, &mut self.contain_cache, inter.shape_idx, s, q) {
                had_containment = true;
                break;
            }
        }
        self.maybe_push_down(idx, r, depth);

        had_containment
    }

    // Move any shapes to child nodes, if necessary.
    fn maybe_push_down(&mut self, idx: NodeIdx, r: Rt, depth: usize) {
        if depth > MAX_DEPTH {
            return;
        }
        let push_down: Vec<_> =
            self.nodes[idx].intersect.drain_filter(|v| v.tests >= TEST_THRESHOLD).collect();
        if !push_down.is_empty() {
            self.ensure_children(idx);

            for inter in push_down {
                let Node { bl, br, tr, tl, .. } = self.nodes[idx];
                let shape = &self.shapes[inter.shape_idx].0;

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

                        if shape.contains_shape(&quad.shape()) {
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
    use rand::prelude::SmallRng;
    use rand::{Rng, SeedableRng};

    use super::*;
    use crate::model::primitive::{circ, poly, pt, rt, tri};

    #[test]
    fn test_quadtree_tri() {
        let mut qt =
            QuadTree::new(vec![(tri(pt(1.0, 2.0), pt(5.0, 2.0), pt(4.0, 5.0)).shape(), NO_TAG)]);
        for _ in 0..TEST_THRESHOLD {
            assert!(qt.intersects(&pt(3.0, 3.0).shape(), Query::All));
        }

        assert!(qt.intersects(&pt(3.0, 3.0).shape(), Query::All));
        assert!(qt.intersects(&rt(3.0, 3.0, 4.0, 4.0).shape(), Query::All));
    }

    #[test]
    fn test_quadtree_poly() {
        let mut qt = QuadTree::new(vec![(
            poly(&[pt(1.0, 2.0), pt(5.0, 2.0), pt(4.0, 5.0)]).shape(),
            NO_TAG,
        )]);
        for _ in 0..TEST_THRESHOLD {
            assert!(qt.intersects(&pt(3.0, 3.0).shape(), Query::All));
        }

        assert!(qt.intersects(&pt(3.0, 3.0).shape(), Query::All));
        assert!(qt.intersects(&rt(3.0, 3.0, 4.0, 4.0).shape(), Query::All));
    }

    #[test]
    fn test_quadtree_poly2() {
        let poly = poly(&[
            pt(136.606, -131.891),
            pt(139.152, -134.437),
            pt(141.344, -132.245),
            pt(138.798, -129.699),
        ])
        .shape();
        let mut qt = QuadTree::new(vec![(poly.clone(), NO_TAG)]);

        let mut r = SmallRng::seed_from_u64(0);
        for _ in 0..100 {
            let p0 = pt(r.gen_range(-50.0..150.0), r.gen_range(-150.0..-100.0));
            let p1 = pt(r.gen_range(-50.0..150.0), r.gen_range(-150.0..-100.0));
            assert_eq!(poly.contains_shape(&p0.shape()), qt.contains(&p0.shape(), Query::All));
            let rt = Rt::enclosing(p0, p1);
            assert_eq!(poly.contains_shape(&rt.shape()), qt.contains(&rt.shape(), Query::All));
            let c = circ(p0, r.gen_range(0.01..100.0));
            assert_eq!(poly.contains_shape(&c.shape()), qt.contains(&c.shape(), Query::All));
        }
    }
}
