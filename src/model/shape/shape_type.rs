use eyre::Result;
use parry2d_f64::query::intersection_test;

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
            (ShapeType::Rect(a), ShapeType::Rect(b)) => todo!(),
            (ShapeType::Rect(a), ShapeType::Circle(b)) => todo!(),
            (ShapeType::Rect(a), ShapeType::Polygon(b)) => todo!(),
            (ShapeType::Rect(a), ShapeType::Path(b)) => todo!(),
            (ShapeType::Rect(a), ShapeType::Arc(b)) => todo!(),
            (ShapeType::Circle(a), ShapeType::Rect(b)) => todo!(),
            (ShapeType::Circle(a), ShapeType::Circle(b)) => todo!(),
            (ShapeType::Circle(a), ShapeType::Polygon(b)) => todo!(),
            (ShapeType::Circle(a), ShapeType::Path(b)) => todo!(),
            (ShapeType::Circle(a), ShapeType::Arc(b)) => todo!(),
            (ShapeType::Polygon(a), ShapeType::Rect(b)) => todo!(),
            (ShapeType::Polygon(a), ShapeType::Circle(b)) => todo!(),
            (ShapeType::Polygon(a), ShapeType::Polygon(b)) => todo!(),
            (ShapeType::Polygon(a), ShapeType::Path(b)) => todo!(),
            (ShapeType::Polygon(a), ShapeType::Arc(b)) => todo!(),
            (ShapeType::Path(a), ShapeType::Rect(b)) => todo!(),
            (ShapeType::Path(a), ShapeType::Circle(b)) => todo!(),
            (ShapeType::Path(a), ShapeType::Polygon(b)) => todo!(),
            (ShapeType::Path(a), ShapeType::Path(b)) => todo!(),
            (ShapeType::Path(a), ShapeType::Arc(b)) => todo!(),
            (ShapeType::Arc(_), _) => todo!(),
        }
    }

    pub fn filled(self) -> ShapeType {
        match self {
            ShapeType::Path(s) => ShapeType::Polygon(Polygon::new(s.pts().to_owned(), s.width())),
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
