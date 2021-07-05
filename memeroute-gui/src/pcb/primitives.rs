use std::f32::consts::PI;

use eframe::egui::emath::RectTransform;
use eframe::egui::{epaint, pos2, Color32, Painter};
use memeroute::model::geom::Pt;
use num_traits::ToPrimitive;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::pcb::to_pos2;

const NUM_POINTS: usize = 16;
const EP: Decimal = dec!(1.0e-5);

pub fn fill_circle(p: &Painter, tf: RectTransform, pt: Pt, r: Decimal, col: Color32) {
    let mut vert = Vec::new();
    let pt = to_pos2(pt);
    let r = r.to_f32().unwrap();
    for i in 0..NUM_POINTS {
        let rad = 2.0 * PI * i as f32 / NUM_POINTS as f32;
        let rad_next = 2.0 * PI * (i + 1) as f32 / NUM_POINTS as f32;
        vert.push(tf.transform_pos(pos2(pt.x, pt.y)));
        vert.push(tf.transform_pos(pos2(pt.x + rad.cos() * r, pt.y + rad.sin() * r)));
        vert.push(tf.transform_pos(pos2(pt.x + rad_next.cos() * r, pt.y + rad_next.sin() * r)));
    }
    p.add(epaint::Shape::Path {
        points: vert,
        closed: true,
        fill: col,
        stroke: Default::default(),
    });
}

pub fn fill_polygon(p: &Painter, tf: RectTransform, pts: &[Pt], col: Color32) {
    let mut vert = Vec::new();
    for &pt in pts {
        vert.push(tf.transform_pos(to_pos2(pt)));
    }
    p.add(epaint::Shape::Path {
        points: vert,
        closed: true,
        fill: col,
        stroke: Default::default(),
    });
}

pub fn stroke_polygon(p: &Painter, tf: RectTransform, pts: &[Pt], width: Decimal, col: Color32) {
    let mut vert = pts.to_owned();
    if let Some(first) = vert.first().copied() {
        vert.push(first);
    }
    stroke_path(p, tf, &vert, width, col);
}

pub fn stroke_path(p: &Painter, tf: RectTransform, pts: &[Pt], width: Decimal, col: Color32) {
    let width = width / dec!(2);
    for &[p0, p1] in pts.array_windows::<2>() {
        fill_circle(p, tf, p0, width / dec!(2), col);

        if p0.dist(p1) > EP {
            let perp = (p1 - p0).perp();
            let vert = [p0 - width * perp, p0 + width * perp, p1 + width * perp, p1 - width * perp];
            fill_polygon(p, tf, &vert, col);
        }
    }
    if let Some(last) = pts.last() {
        fill_circle(p, tf, *last, width, col);
    }
}
