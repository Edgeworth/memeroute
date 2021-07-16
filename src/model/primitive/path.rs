use crate::model::geom::bounds::point_cloud_bounds;
use crate::model::geom::convex::remove_collinear;
use crate::model::primitive::rt::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::pt::Pt;

#[derive(Clone)]
pub struct Path {
    pts: Vec<Pt>,
    width: f64,
}

impl std::fmt::Debug for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?} {:?}", self.pts, self.width))
    }
}

impl Path {
    pub fn new(pts: &[Pt], width: f64) -> Self {
        Self { pts: remove_collinear(pts), width }
    }

    pub fn shape(self) -> Shape {
        Shape::Path(self)
    }

    pub fn bounds(&self) -> Rt {
        point_cloud_bounds(&self.pts).inset_xy(-self.width / 2.0, -self.width / 2.0)
    }

    pub fn pts(&self) -> &[Pt] {
        &self.pts
    }

    pub fn width(&self) -> f64 {
        self.width
    }
}
