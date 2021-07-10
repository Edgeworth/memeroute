use crate::model::shape::arc::Arc;
use crate::model::shape::circle::Circle;
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
