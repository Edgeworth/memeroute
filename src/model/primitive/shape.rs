use crate::model::geom::contains::{
    circle_contains_circle, circle_contains_path, circle_contains_polygon, circle_contains_rt,
    path_contains_circle, path_contains_path, path_contains_polygon, path_contains_rt,
    polygon_contains_circle, polygon_contains_path, polygon_contains_polygon, polygon_contains_rt,
    rt_contains_circle, rt_contains_path, rt_contains_polygon, rt_contains_rt,
};
use crate::model::geom::intersects::{
    circle_intersects_circle, circle_intersects_path, circle_intersects_polygon,
    path_intersects_path, polygon_intersects_path, polygon_intersects_polygon,
    rt_intersects_circle, rt_intersects_path, rt_intersects_polygon, rt_intersects_rt,
};
use crate::model::primitive::arc::Arc;
use crate::model::primitive::circle::Circle;
use crate::model::primitive::path::Path;
use crate::model::primitive::polygon::Polygon;
use crate::model::primitive::rt::Rt;

#[derive(Debug, Clone)]
pub enum Shape {
    Rect(Rt),
    Circle(Circle),
    Polygon(Polygon),
    Path(Path),
    Arc(Arc),
}

impl Shape {
    pub fn bounds(&self) -> Rt {
        match self {
            Shape::Rect(s) => *s,
            Shape::Circle(s) => s.bounds(),
            Shape::Polygon(s) => s.bounds(),
            Shape::Path(s) => s.bounds(),
            Shape::Arc(_) => todo!(),
        }
    }

    pub fn intersects(&self, s: &Shape) -> bool {
        match (self, s) {
            (Shape::Rect(a), Shape::Rect(b)) => rt_intersects_rt(a, b),
            (Shape::Rect(a), Shape::Circle(b)) => rt_intersects_circle(a, b),
            (Shape::Rect(a), Shape::Polygon(b)) => rt_intersects_polygon(a, b),
            (Shape::Rect(a), Shape::Path(b)) => rt_intersects_path(a, b),
            (Shape::Circle(a), Shape::Rect(b)) => rt_intersects_circle(b, a),
            (Shape::Circle(a), Shape::Circle(b)) => circle_intersects_circle(a, b),
            (Shape::Circle(a), Shape::Polygon(b)) => circle_intersects_polygon(a, b),
            (Shape::Circle(a), Shape::Path(b)) => circle_intersects_path(a, b),
            (Shape::Polygon(a), Shape::Rect(b)) => rt_intersects_polygon(b, a),
            (Shape::Polygon(a), Shape::Circle(b)) => circle_intersects_polygon(b, a),
            (Shape::Polygon(a), Shape::Polygon(b)) => polygon_intersects_polygon(a, b),
            (Shape::Polygon(a), Shape::Path(b)) => polygon_intersects_path(a, b),
            (Shape::Path(a), Shape::Rect(b)) => rt_intersects_path(b, a),
            (Shape::Path(a), Shape::Circle(b)) => circle_intersects_path(b, a),
            (Shape::Path(a), Shape::Polygon(b)) => polygon_intersects_path(b, a),
            (Shape::Path(a), Shape::Path(b)) => path_intersects_path(a, b),
            (Shape::Arc(_), _) => todo!(),
            (_, Shape::Arc(_)) => todo!(),
        }
    }

    pub fn contains(&self, s: &Shape) -> bool {
        match (self, s) {
            (Shape::Rect(a), Shape::Rect(b)) => rt_contains_rt(a, b),
            (Shape::Rect(a), Shape::Circle(b)) => rt_contains_circle(a, b),
            (Shape::Rect(a), Shape::Polygon(b)) => rt_contains_polygon(a, b),
            (Shape::Rect(a), Shape::Path(b)) => rt_contains_path(a, b),
            (Shape::Circle(a), Shape::Rect(b)) => circle_contains_rt(a, b),
            (Shape::Circle(a), Shape::Circle(b)) => circle_contains_circle(a, b),
            (Shape::Circle(a), Shape::Polygon(b)) => circle_contains_polygon(a, b),
            (Shape::Circle(a), Shape::Path(b)) => circle_contains_path(a, b),
            (Shape::Polygon(a), Shape::Rect(b)) => polygon_contains_rt(a, b),
            (Shape::Polygon(a), Shape::Circle(b)) => polygon_contains_circle(a, b),
            (Shape::Polygon(a), Shape::Polygon(b)) => polygon_contains_polygon(a, b),
            (Shape::Polygon(a), Shape::Path(b)) => polygon_contains_path(a, b),
            (Shape::Path(a), Shape::Rect(b)) => path_contains_rt(a, b),
            (Shape::Path(a), Shape::Circle(b)) => path_contains_circle(a, b),
            (Shape::Path(a), Shape::Polygon(b)) => path_contains_polygon(a, b),
            (Shape::Path(a), Shape::Path(b)) => path_contains_path(a, b),
            (Shape::Arc(_), _) => todo!(),
            (_, Shape::Arc(_)) => todo!(),
        }
    }

    pub fn filled(self) -> Shape {
        match self {
            Shape::Path(s) => Shape::Polygon(Polygon::new(s.pts(), s.width())),
            Shape::Arc(_) => todo!(),
            s => s,
        }
    }
}
