use approx::assert_relative_eq;
use earcutr::earcut;
use parry2d_f64::shape::{ConvexPolygon, RoundShape, TriMesh};

use crate::model::geom::convex::{ensure_ccw, is_convex_ccw, remove_collinear};
use crate::model::pt::Pt;
use crate::model::shape::rt::Rt;

#[derive(Clone)]
enum ParryShape {
    Convex(RoundShape<ConvexPolygon>),
    Concave(TriMesh),
}

impl std::fmt::Debug for ParryShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("polygon")
    }
}

#[derive(Debug, Clone)]
pub struct Polygon {
    pts: Vec<Pt>,
    tris: Vec<u32>,
    width: f64,
    parry: ParryShape,
}

impl Polygon {
    pub fn new(pts: &[Pt], width: f64) -> Self {
        let mut pts = remove_collinear(&pts);
        ensure_ccw(&mut pts);
        let verts: Vec<f64> = pts.iter().map(|v| [v.x, v.y]).flatten().collect();
        let tris: Vec<_> = earcut(&verts, &vec![], 2).iter().map(|&v| v as u32).collect();

        let points: Vec<Point<Real>> = pts.iter().map(|p| p.into()).collect();
        let parry = if is_convex_ccw(&pts) {
            ParryShape::Convex(RoundShape::<ConvexPolygon> {
                base_shape: ConvexPolygon::from_convex_polyline(points)
                    .unwrap_or_else(|| panic!("bad polygon {:?}", pts)),
                border_radius: width / 2.0,
            })
        } else {
            // Currently don't support non-convex polygons with rounded corners.
            assert_relative_eq!(width, 0.0);
            ParryShape::Concave(TriMesh::new(points, tris.array_chunks::<3>().copied().collect()))
        };
        Self { pts, tris, width, parry }
    }

    pub fn bounds(&self) -> Rt {
        self.as_parry().compute_local_aabb().into()
    }

    pub fn pts(&self) -> &[Pt] {
        &self.pts
    }

    pub fn tris(&self) -> &[u32] {
        &self.tris
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn as_parry(&self) -> &dyn Shape {
        match &self.parry {
            ParryShape::Convex(v) => v,
            ParryShape::Concave(v) => v,
        }
    }
}

impl_parry2d!(Polygon);
