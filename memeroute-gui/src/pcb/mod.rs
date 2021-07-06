use eframe::egui::{Pos2, Rect};
use memeroute::model::geom::{Pt, Rt};

pub mod pcb_view;
pub mod primitives;

pub fn to_pos2(p: Pt) -> Pos2 {
    Pos2::new(p.x as f32, p.y as f32)
}

pub fn to_rect(r: Rt) -> Rect {
    Rect::from_two_pos(to_pos2(r.tl()), to_pos2(r.br()))
}

pub fn to_rt(r: Rect) -> Rt {
    Rt::new(r.left() as f64, r.top() as f64, r.width() as f64, r.height() as f64)
}
