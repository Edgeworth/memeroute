use std::collections::HashMap;

use crate::model::geom::qt::quadtree::ShapeIdx;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::ShapeOps;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QueryId(pub usize);
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QueryKind(pub usize);

pub const NO_ID: QueryId = QueryId(usize::MAX);
pub const NO_KIND: QueryKind = QueryKind(usize::MAX);

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Query {
    All,
    Id(QueryId),
    ExceptId(QueryId),
    Kind(QueryKind),
}

pub fn matches_query(s: &ShapeInfo, q: Query) -> bool {
    match q {
        Query::All => true,
        Query::Id(tag) => tag == s.id,
        Query::ExceptId(tag) => tag != s.id,
        Query::Kind(tag) => tag == s.kind,
    }
}

#[derive(Debug, Clone)]
pub struct ShapeInfo {
    shape: Shape,
    id: QueryId,
    kind: QueryKind,
}

impl ShapeInfo {
    pub fn new(shape: Shape, id: QueryId, kind: QueryKind) -> Self {
        Self { shape, id, kind }
    }

    pub fn anon(shape: Shape) -> Self {
        Self { shape, id: NO_ID, kind: NO_KIND }
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn id(&self) -> QueryId {
        self.id
    }

    pub fn kind(&self) -> QueryKind {
        self.kind
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
    let kind = s.kind;
    shapes.into_iter().map(|shape| ShapeInfo { shape, id, kind }).collect()
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
