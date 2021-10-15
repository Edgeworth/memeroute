use std::collections::HashMap;

use rust_dense_bitset::DenseBitSet;

use crate::model::geom::qt::quadtree::ShapeIdx;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::ShapeOps;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct QueryId(pub usize);
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct QueryKinds(pub DenseBitSet);

pub const NO_ID: QueryId = QueryId(usize::MAX);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Query {
    All,
    Id(QueryId),
    ExceptId(QueryId),
    CommonKind(QueryKinds), // Query all shapes who have a common kind with the query value.
}

pub fn matches_query(s: &ShapeInfo, q: Query) -> bool {
    match q {
        Query::All => true,
        Query::Id(id) => id == s.id,
        Query::ExceptId(id) => id != s.id,
        Query::CommonKind(kinds) => (kinds.0 & s.kinds.0).any(),
    }
}

#[derive(Debug, Clone)]
pub struct ShapeInfo {
    shape: Shape,
    id: QueryId,
    kinds: QueryKinds, // A bitmask.
}

impl ShapeInfo {
    pub fn new(shape: Shape, id: QueryId, kinds: QueryKinds) -> Self {
        Self { shape, id, kinds }
    }

    pub fn anon(shape: Shape) -> Self {
        Self { shape, id: NO_ID, kinds: QueryKinds(DenseBitSet::new()) }
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn id(&self) -> QueryId {
        self.id
    }

    pub fn kinds(&self) -> QueryKinds {
        self.kinds
    }
}

// Split paths up so they are spread out more.
// Split compound shapes up.
pub fn decompose_shape(s: ShapeInfo) -> Vec<ShapeInfo> {
    let shapes = match s.shape {
        Shape::Compound(s) => s.quadtree().shapes().iter().map(|v| v.shape.clone()).collect(),
        Shape::Path(s) => s.caps().map(|v| v.shape()).collect(),
        s => vec![s],
    };
    let id = s.id;
    let kinds = s.kinds;
    shapes.into_iter().map(|shape| ShapeInfo { shape, id, kinds }).collect()
}

pub fn cached_intersects(
    shapes: &[ShapeInfo],
    cache: &mut HashMap<ShapeIdx, bool>,
    idx: ShapeIdx,
    s: &Shape,
    q: Query,
) -> bool {
    if !matches_query(&shapes[idx], q) {
        false
    } else if let Some(res) = cache.get(&idx) {
        *res
    } else {
        let res = shapes[idx].shape().intersects_shape(s);
        cache.insert(idx, res);
        res
    }
}

pub fn cached_contains(
    shapes: &[ShapeInfo],
    cache: &mut HashMap<ShapeIdx, bool>,
    idx: ShapeIdx,
    s: &Shape,
    q: Query,
) -> bool {
    if !matches_query(&shapes[idx], q) {
        false
    } else if let Some(res) = cache.get(&idx) {
        *res
    } else {
        let res = shapes[idx].shape().contains_shape(s);
        cache.insert(idx, res);
        res
    }
}

pub fn cached_dist(
    shapes: &[ShapeInfo],
    cache: &mut HashMap<ShapeIdx, f64>,
    idx: ShapeIdx,
    s: &Shape,
    q: Query,
) -> f64 {
    if !matches_query(&shapes[idx], q) {
        f64::MAX // Let's say distance to the empty set is infinity-ish.
    } else if let Some(res) = cache.get(&idx) {
        *res
    } else {
        let res = shapes[idx].shape().dist_to_shape(s);
        cache.insert(idx, res);
        res
    }
}
