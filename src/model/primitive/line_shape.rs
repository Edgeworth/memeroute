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
    pub const fn new(st: Pt, en: Pt) -> Self {
        Self { st, en }
    }

    pub const fn st(&self) -> Pt {
        self.st
    }

    pub const fn en(&self) -> Pt {
        self.en
    }

    pub fn dir(&self) -> Pt {
        self.en - self.st
    }

    // Projects |p| onto this line.
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
