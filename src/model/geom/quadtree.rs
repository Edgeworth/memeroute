use crate::model::geom::bounds::rt_cloud_bounds;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::ShapeOps;

type NodeIdx = usize;
type ShapeIdx = usize;

// How many tests to do before splitting a node.
const TEST_THRESHOLD: usize = 4;
const NO_NODE: NodeIdx = 0;

#[derive(Debug, Copy, Clone)]
struct IntersectData {
    shape: ShapeIdx,
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
                    .map(|shape| IntersectData { shape, tests: 0 })
                    .collect(),
                ..Default::default()
            },
        ];
        Self { shapes, nodes, bounds }
    }

    pub fn bounds(&self) -> Rt {
        self.bounds
    }

    pub fn intersects(&mut self, s: &Shape) -> bool {
        self.inter(s, 1, self.bounds())
    }

    pub fn contains(&mut self, s: &Shape) -> bool {
        todo!()
    }

    pub fn dist(&mut self, s: &Shape) -> f64 {
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

        // TODO: Could check if |s| intersects the bounds here and return true if
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
        for inter in self.nodes[idx].intersect.iter_mut() {
            let shape = &self.shapes[inter.shape];
            if shape.intersects_shape(s) {
                return true;
            }
        }
        self.maybe_push_down(idx, r);

        false
    }

    // Move any shapes to child nodes, if necessary.
    fn maybe_push_down(&mut self, idx: NodeIdx, r: Rt) {
        let push_down: Vec<_> =
            self.nodes[idx].intersect.drain_filter(|v| v.tests >= TEST_THRESHOLD).collect();
        if !push_down.is_empty() {
            self.ensure_children(idx);

            for inter in push_down {
                println!("push down");
                let Node { bl, br, tr, tl, .. } = self.nodes[idx];
                let shape = &self.shapes[inter.shape];

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
                            .push(IntersectData { shape: inter.shape, tests: 0 });

                        if quad.contains_shape(shape) {
                            self.nodes[quad_idx].contain.push(inter.shape);
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
