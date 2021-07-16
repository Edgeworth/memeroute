use std::f64::consts::TAU;

use eframe::egui::epaint::{Mesh, Vertex};
use eframe::egui::{epaint, Color32};
use memeroute::model::pt::Pt;
use memeroute::model::primitive::rt::Rt;
use memeroute::model::tf::Tf;

use crate::pcb::to_pos2;

const NUM_POINTS: usize = 16;
const EP: f64 = 1.0e-5;

pub fn fill_rect(tf: &Tf, rt: &Rt, col: Color32) -> epaint::Shape {
    fill_polygon(tf, &[rt.tl(), rt.bl(), rt.br(), rt.tr()], &[0, 1, 2, 0, 2, 3], col)
}

pub fn fill_circle(tf: &Tf, pt: Pt, r: f64, col: Color32) -> epaint::Shape {
    let mut vert = Vec::new();
    for i in 0..NUM_POINTS {
        let rad = TAU * i as f64 / NUM_POINTS as f64;
        let rad_next = TAU * (i + 1) as f64 / NUM_POINTS as f64;
        vert.push(to_pos2(tf.pt(Pt::new(pt.x + rad.cos() * r, pt.y + rad.sin() * r))));
        vert.push(to_pos2(tf.pt(Pt::new(pt.x + rad_next.cos() * r, pt.y + rad_next.sin() * r))));
    }
    epaint::Shape::Path { points: vert, closed: true, fill: col, stroke: Default::default() }
}

pub fn fill_polygon(tf: &Tf, pts: &[Pt], tris: &[u32], col: Color32) -> epaint::Shape {
    let vert = pts
        .iter()
        .map(|&v| Vertex { pos: to_pos2(tf.pt(v)), uv: Default::default(), color: col })
        .collect();
    epaint::Shape::Mesh(Mesh {
        indices: tris.to_owned(),
        vertices: vert,
        texture_id: Default::default(),
    })
}

pub fn stroke_polygon(tf: &Tf, pts: &[Pt], width: f64, col: Color32) -> Vec<epaint::Shape> {
    let mut vert = pts.to_owned();
    if let Some(first) = vert.first().copied() {
        vert.push(first);
    }
    stroke_path(tf, &vert, width, col)
}

pub fn stroke_path(tf: &Tf, pts: &[Pt], width: f64, col: Color32) -> Vec<epaint::Shape> {
    let mut shapes = Vec::new();
    let width = width / 2.0;
    for &[p0, p1] in pts.array_windows::<2>() {
        shapes.push(fill_circle(tf, p0, width, col));

        if p0.dist(p1) > EP {
            let perp = (p1 - p0).perp();
            let vert = [p0 - width * perp, p0 + width * perp, p1 + width * perp, p1 - width * perp];
            shapes.push(fill_polygon(tf, &vert, &[0, 1, 2, 0, 2, 3], col));
        }
    }
    if let Some(last) = pts.last() {
        shapes.push(fill_circle(tf, *last, width, col));
    }
    shapes
}
