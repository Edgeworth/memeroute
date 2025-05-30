use std::f64::consts::TAU;

use eframe::egui::epaint::{Mesh, PathShape, Vertex};
use eframe::egui::{Color32, epaint};
use eframe::emath::Pos2;
use eframe::epaint::{PathStroke, TextureId};
use memegeom::primitive::point::Pt;
use memegeom::primitive::pt;
use memegeom::primitive::rect::Rt;
use memegeom::tf::Tf;

use crate::pcb::to_pos2;

const NUM_POINTS: usize = 16;
const EP: f64 = 1.0e-5;

pub fn fill_rt(tf: &Tf, rt: &Rt, col: Color32) -> epaint::Shape {
    fill_polygon(tf, &rt.pts(), &[0, 1, 2, 0, 2, 3], col)
}

pub fn fill_circle(tf: &Tf, p: Pt, r: f64, col: Color32) -> epaint::Shape {
    let mut vert = Vec::new();
    for i in 0..NUM_POINTS {
        let rad = TAU * i as f64 / NUM_POINTS as f64;
        let rad_next = TAU * (i + 1) as f64 / NUM_POINTS as f64;
        vert.push(to_pos2(tf.pt(pt(p.x + rad.cos() * r, p.y + rad.sin() * r))));
        vert.push(to_pos2(tf.pt(pt(p.x + rad_next.cos() * r, p.y + rad_next.sin() * r))));
    }
    epaint::Shape::Path(PathShape {
        points: vert,
        closed: true,
        fill: col,
        stroke: PathStroke::default(),
    })
}

pub fn fill_polygon(tf: &Tf, pts: &[Pt], tris: &[u32], col: Color32) -> epaint::Shape {
    let vert = pts
        .iter()
        .map(|&v| Vertex { pos: to_pos2(tf.pt(v)), uv: Pos2::default(), color: col })
        .collect();
    epaint::Shape::Mesh(
        Mesh { indices: tris.to_owned(), vertices: vert, texture_id: TextureId::default() }.into(),
    )
}

#[must_use]
pub fn stroke_polygon(tf: &Tf, pts: &[Pt], width: f64, col: Color32) -> Vec<epaint::Shape> {
    let mut vert = pts.to_owned();
    if let Some(first) = vert.first().copied() {
        vert.push(first);
    }
    stroke_path(tf, &vert, width, col)
}

#[must_use]
pub fn stroke_path(tf: &Tf, pts: &[Pt], r: f64, col: Color32) -> Vec<epaint::Shape> {
    let mut shapes = Vec::new();
    for &[p0, p1] in pts.array_windows::<2>() {
        shapes.push(fill_circle(tf, p0, r, col));

        if p0.dist(p1) > EP {
            let perp = (p1 - p0).perp();
            let vert = [p0 - r * perp, p0 + r * perp, p1 + r * perp, p1 - r * perp];
            shapes.push(fill_polygon(tf, &vert, &[0, 1, 2, 0, 2, 3], col));
        }
    }
    if let Some(last) = pts.last() {
        shapes.push(fill_circle(tf, *last, r, col));
    }
    shapes
}
