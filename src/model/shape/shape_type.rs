use parry2d_f64::query::intersection_test;

use crate::model::geom::contains::{
    circle_contains_circle, circle_contains_path, circle_contains_polygon, circle_contains_rect,
    path_contains_circle, path_contains_path, path_contains_polygon, path_contains_rect,
    polygon_contains_circle, polygon_contains_path, polygon_contains_polygon,
    polygon_contains_rect, rect_contains_circle, rect_contains_path, rect_contains_polygon,
    rect_contains_rect,
};
use crate::model::shape::arc::Arc;
use crate::model::shape::circle::Circle;
use crate::model::shape::identity;
use crate::model::shape::path::Path;
use crate::model::shape::polygon::Polygon;
use crate::model::shape::rt::Rt;

#[derive(Debug, Clone)]
pub enum ShapeType {
    Rect(Rt),
    Circle(Circle),
    Polygon(Polygon),
    Path(Path),
    Arc(Arc),
}

impl ShapeType {
    pub fn bounds(&self) -> Rt {
        match self {
            ShapeType::Rect(s) => s.clone(),
            ShapeType::Circle(s) => s.bounds(),
            ShapeType::Polygon(s) => s.bounds(),
            ShapeType::Path(s) => s.bounds(),
            ShapeType::Arc(_) => todo!(),
        }
    }

    pub fn intersects(&self, s: &ShapeType) -> bool {
        intersection_test(&identity(), self, &identity(), s).unwrap()
    }

    pub fn contains(&self, s: &ShapeType) -> bool {
        match (self, s) {
            (ShapeType::Rect(a), ShapeType::Rect(b)) => rect_contains_rect(a, b),
            (ShapeType::Rect(a), ShapeType::Circle(b)) => rect_contains_circle(a, b),
            (ShapeType::Rect(a), ShapeType::Polygon(b)) => rect_contains_polygon(a, b),
            (ShapeType::Rect(a), ShapeType::Path(b)) => rect_contains_path(a, b),
            (ShapeType::Rect(_), ShapeType::Arc(_)) => todo!(),
            (ShapeType::Circle(a), ShapeType::Rect(b)) => circle_contains_rect(a, b),
            (ShapeType::Circle(a), ShapeType::Circle(b)) => circle_contains_circle(a, b),
            (ShapeType::Circle(a), ShapeType::Polygon(b)) => circle_contains_polygon(a, b),
            (ShapeType::Circle(a), ShapeType::Path(b)) => circle_contains_path(a, b),
            (ShapeType::Circle(_), ShapeType::Arc(_)) => todo!(),
            (ShapeType::Polygon(a), ShapeType::Rect(b)) => polygon_contains_rect(a, b),
            (ShapeType::Polygon(a), ShapeType::Circle(b)) => polygon_contains_circle(a, b),
            (ShapeType::Polygon(a), ShapeType::Polygon(b)) => polygon_contains_polygon(a, b),
            (ShapeType::Polygon(a), ShapeType::Path(b)) => polygon_contains_path(a, b),
            (ShapeType::Polygon(_), ShapeType::Arc(_)) => todo!(),
            (ShapeType::Path(a), ShapeType::Rect(b)) => path_contains_rect(a, b),
            (ShapeType::Path(a), ShapeType::Circle(b)) => path_contains_circle(a, b),
            (ShapeType::Path(a), ShapeType::Polygon(b)) => path_contains_polygon(a, b),
            (ShapeType::Path(a), ShapeType::Path(b)) => path_contains_path(a, b),
            (ShapeType::Path(_), ShapeType::Arc(_)) => todo!(),
            (ShapeType::Arc(_), _) => todo!(),
        }
    }

    pub fn filled(self) -> ShapeType {
        match self {
            ShapeType::Path(s) => ShapeType::Polygon(Polygon::new(s.pts(), s.width())),
            ShapeType::Arc(_) => todo!(),
            s => s,
        }
    }

    pub fn as_parry(&self) -> &dyn parry2d_f64::shape::Shape {
        match self {
            ShapeType::Rect(s) => s.as_parry(),
            ShapeType::Circle(s) => s.as_parry(),
            ShapeType::Polygon(s) => s.as_parry(),
            ShapeType::Path(s) => s.as_parry(),
            ShapeType::Arc(_) => todo!(),
        }
    }
}

impl_parry2d!(ShapeType);
