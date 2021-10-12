use crate::model::geom::math::eq;
use crate::model::primitive::capsule::Capsule;
use crate::model::primitive::circle::Circle;
use crate::model::primitive::compound::Compound;
use crate::model::primitive::line_shape::Line;
use crate::model::primitive::path_shape::Path;
use crate::model::primitive::point::Pt;
use crate::model::primitive::polygon::Poly;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::segment::Segment;
use crate::model::primitive::triangle::Tri;
use crate::model::primitive::{poly, ShapeOps};
use crate::model::tf::Tf;

#[derive(Debug, Clone)]
pub enum Shape {
    Capsule(Capsule),
    Circle(Circle),
    Compound(Compound),
    Line(Line),
    Path(Path),
    Point(Pt),
    Polygon(Poly),
    Rect(Rt),
    Segment(Segment),
    Tri(Tri),
}

impl Shape {
    pub fn filled(self) -> Shape {
        match self {
            Shape::Path(s) => {
                assert!(eq(s.r(), 0.0), "path width not supported for polygons");
                poly(s.pts()).shape()
            }
            s => s,
        }
    }

    pub fn apply(&mut self, tf: &Tf) {
        *self = tf.shape(self);
    }
}

impl ShapeOps for Shape {
    fn bounds(&self) -> Rt {
        match self {
            Shape::Capsule(s) => s.bounds(),
            Shape::Circle(s) => s.bounds(),
            Shape::Compound(s) => s.bounds(),
            Shape::Line(s) => s.bounds(),
            Shape::Path(s) => s.bounds(),
            Shape::Point(s) => s.bounds(),
            Shape::Polygon(s) => s.bounds(),
            Shape::Rect(s) => s.bounds(),
            Shape::Segment(s) => s.bounds(),
            Shape::Tri(s) => s.bounds(),
        }
    }

    fn shape(self) -> Shape {
        self
    }

    fn intersects_shape(&self, s: &Shape) -> bool {
        match self {
            Shape::Capsule(us) => us.intersects_shape(s),
            Shape::Circle(us) => us.intersects_shape(s),
            Shape::Compound(us) => us.intersects_shape(s),
            Shape::Line(us) => us.intersects_shape(s),
            Shape::Path(us) => us.intersects_shape(s),
            Shape::Point(us) => us.intersects_shape(s),
            Shape::Polygon(us) => us.intersects_shape(s),
            Shape::Rect(us) => us.intersects_shape(s),
            Shape::Segment(us) => us.intersects_shape(s),
            Shape::Tri(us) => us.intersects_shape(s),
        }
    }

    fn contains_shape(&self, s: &Shape) -> bool {
        match self {
            Shape::Capsule(us) => us.contains_shape(s),
            Shape::Circle(us) => us.contains_shape(s),
            Shape::Compound(us) => us.contains_shape(s),
            Shape::Line(us) => us.contains_shape(s),
            Shape::Path(us) => us.contains_shape(s),
            Shape::Point(us) => us.contains_shape(s),
            Shape::Polygon(us) => us.contains_shape(s),
            Shape::Rect(us) => us.contains_shape(s),
            Shape::Segment(us) => us.contains_shape(s),
            Shape::Tri(us) => us.contains_shape(s),
        }
    }

    fn dist_to_shape(&self, s: &Shape) -> f64 {
        match self {
            Shape::Capsule(us) => us.dist_to_shape(s),
            Shape::Circle(us) => us.dist_to_shape(s),
            Shape::Compound(us) => us.dist_to_shape(s),
            Shape::Line(us) => us.dist_to_shape(s),
            Shape::Path(us) => us.dist_to_shape(s),
            Shape::Point(us) => us.dist_to_shape(s),
            Shape::Polygon(us) => us.dist_to_shape(s),
            Shape::Rect(us) => us.dist_to_shape(s),
            Shape::Segment(us) => us.dist_to_shape(s),
            Shape::Tri(us) => us.dist_to_shape(s),
        }
    }
}
