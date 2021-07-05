use std::f32::consts::PI;

use eframe::egui::{epaint, pos2, Color32, Painter};
use memeroute::model::geom::Pt;
use memeroute::model::transform::Tf;
use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

use crate::pcb::to_pos2;

const NUM_POINTS: usize = 16;
const EP: Decimal = dec!(1.0e-5);

pub fn fill_circle(p: &Painter, tf: &Tf, pt: Pt, r: Decimal, col: Color32) {
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
    p.add(epaint::Shape::Path {
        points: vert,
        closed: true,
        fill: col,
        stroke: Default::default(),
    });
}

pub fn fill_polygon(p: &Painter, tf: &Tf, pts: &[Pt], col: Color32) {
    let mut vert = Vec::new();
    for &pt in pts {
        vert.push(to_pos2(tf.pt(pt)));
    }
    p.add(epaint::Shape::Path {
        points: vert,
        closed: true,
        fill: col,
        stroke: Default::default(),
    });
}

pub fn stroke_polygon(p: &Painter, tf: &Tf, pts: &[Pt], width: Decimal, col: Color32) {
    let mut vert = pts.to_owned();
    if let Some(first) = vert.first().copied() {
        vert.push(first);
    }
    stroke_path(p, tf, &vert, width, col);
}

pub fn stroke_path(p: &Painter, tf: &Tf, pts: &[Pt], width: Decimal, col: Color32) {
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
