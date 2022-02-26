use crate::model::geom::distance::line_pt_dist;
use crate::model::geom::intersects::{line_intersects_line, line_intersects_seg};
use crate::model::primitive::point::Pt;
use crate::model::primitive::rect::Rt;
use crate::model::primitive::shape::Shape;
use crate::model::primitive::ShapeOps;

#[derive(Debug, Copy, Clone)]
pub struct Line {
    st: Pt,
    en: Pt,
}

impl Line {
    #[must_use]
    pub const fn new(st: Pt, en: Pt) -> Self {
        Self { st, en }
    }

    #[must_use]
    pub const fn st(&self) -> Pt {
        self.st
    }

    #[must_use]
    pub const fn en(&self) -> Pt {
        self.en
    }

    #[must_use]
    pub fn dir(&self) -> Pt {
        self.en - self.st
    }

    // Projects |p| onto this line.
    #[must_use]
    pub fn project(&self, p: Pt) -> Pt {
        let dir = self.dir();
        let k = dir.dot(p - self.st) / dir.mag2();
        self.st + k * dir
    }
}

impl ShapeOps for Line {
    fn bounds(&self) -> Rt {
        // Bounds kind of undefined for a Line.
        Rt::empty()
    }

    fn shape(self) -> Shape {
        Shape::Line(self)
    }

    fn intersects_shape(&self, s: &Shape) -> bool {
        match s {
            Shape::Capsule(_) => todo!(),
            Shape::Circle(_) => todo!(),
            Shape::Compound(_) => todo!(),
            Shape::Line(s) => line_intersects_line(self, s),
            Shape::Path(_) => todo!(),
            Shape::Point(_) => todo!(),
            Shape::Polygon(_) => todo!(),
            Shape::Rect(_) => todo!(),
            Shape::Segment(s) => line_intersects_seg(self, s),
            Shape::Tri(_) => todo!(),
        }
    }

    fn contains_shape(&self, s: &Shape) -> bool {
        match s {
            Shape::Capsule(_) => todo!(),
            Shape::Circle(_) => todo!(),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(_) => todo!(),
            Shape::Point(_) => todo!(),
            Shape::Polygon(_) => todo!(),
            Shape::Rect(_) => todo!(),
            Shape::Segment(_) => todo!(),
            Shape::Tri(_) => todo!(),
        }
    }

    fn dist_to_shape(&self, s: &Shape) -> f64 {
        match s {
            Shape::Capsule(_) => todo!(),
            Shape::Circle(_) => todo!(),
            Shape::Compound(_) => todo!(),
            Shape::Line(_) => todo!(),
            Shape::Path(_) => todo!(),
            Shape::Point(s) => line_pt_dist(self, s),
            Shape::Polygon(_) => todo!(),
            Shape::Rect(_) => todo!(),
            Shape::Segment(_) => todo!(),
            Shape::Tri(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use crate::model::primitive::{line, pt};

    #[test]
    fn test_project() {
        assert_relative_eq!(line(pt(1.0, 1.0), pt(3.0, 5.0)).project(pt(3.0, 3.0)), pt(2.2, 3.4));
    }
}
