use std::f32::consts::PI;

use eframe::egui::{epaint, Color32};
use memeroute::model::geom::{Pt, Rt};
use memeroute::model::transform::Tf;
use num_traits::FromPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::pcb::{to_pos2, to_rect};

const NUM_POINTS: usize = 16;
const EP: Decimal = dec!(1.0e-5);

pub fn fill_rect(tf: &Tf, rt: Rt, col: Color32) -> epaint::Shape {
    epaint::Shape::Rect {
        rect: to_rect(tf.rt(rt)),
        corner_radius: 0.0,
        fill: col,
        stroke: Default::default(),
    }
}

pub fn fill_circle(tf: &Tf, pt: Pt, r: Decimal, col: Color32) -> epaint::Shape {
    let mut vert = Vec::new();
    for i in 0..NUM_POINTS {
        let rad = 2.0 * PI * i as f32 / NUM_POINTS as f32;
        let rad_next = 2.0 * PI * (i + 1) as f32 / NUM_POINTS as f32;
        vert.push(to_pos2(tf.pt(pt)));
        vert.push(to_pos2(tf.pt(Pt::new(
            pt.x + Decimal::from_f32(rad.cos()).unwrap() * r,
            pt.y + Decimal::from_f32(rad.sin()).unwrap() * r,
        ))));
        vert.push(to_pos2(tf.pt(Pt::new(
            pt.x + Decimal::from_f32(rad_next.cos()).unwrap() * r,
            pt.y + Decimal::from_f32(rad_next.sin()).unwrap() * r,
        ))));
    }
    epaint::Shape::Path { points: vert, closed: true, fill: col, stroke: Default::default() }
}

pub fn fill_polygon(tf: &Tf, pts: &[Pt], col: Color32) -> epaint::Shape {
    let mut vert = Vec::new();
    for &pt in pts {
        vert.push(to_pos2(tf.pt(pt)));
    }
    epaint::Shape::Path { points: vert, closed: true, fill: col, stroke: Default::default() }
}

pub fn stroke_polygon(tf: &Tf, pts: &[Pt], width: Decimal, col: Color32) -> Vec<epaint::Shape> {
    let mut vert = pts.to_owned();
    if let Some(first) = vert.first().copied() {
        vert.push(first);
    }
    stroke_path(tf, &vert, width, col)
}

pub fn stroke_path(tf: &Tf, pts: &[Pt], width: Decimal, col: Color32) -> Vec<epaint::Shape> {
    let mut shapes = Vec::new();
    let width = width / dec!(2);
    for &[p0, p1] in pts.array_windows::<2>() {
        shapes.push(fill_circle(tf, p0, width / dec!(2), col));

        if p0.dist(p1) > EP {
            let perp = (p1 - p0).perp();
            let vert = [p0 - width * perp, p0 + width * perp, p1 + width * perp, p1 - width * perp];
            shapes.push(fill_polygon(tf, &vert, col));
        }
    }
    if let Some(last) = pts.last() {
        shapes.push(fill_circle(tf, *last, width, col));
    }
    shapes
}
