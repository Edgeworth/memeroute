use eframe::egui::{Pos2, Rect};
use memeroute::model::primitive::point::Pt;
use memeroute::model::primitive::rect::Rt;
use memeroute::model::primitive::{pt, rt};

pub mod pcb_view;
pub mod primitives;

#[must_use]
pub fn to_pos2(p: Pt) -> Pos2 {
    Pos2::new(p.x as f32, p.y as f32)
}

#[must_use]
pub fn to_pt(p: Pos2) -> Pt {
    pt(p.x as f64, p.y as f64)
}

#[must_use]
pub fn to_rect(r: &Rt) -> Rect {
    Rect::from_two_pos(to_pos2(r.bl()), to_pos2(r.tr()))
}

#[must_use]
pub fn to_rt(r: Rect) -> Rt {
    // Using r.top() is correct here because our Rt's are flipped compared to Rects.
    rt(r.left() as f64, r.top() as f64, r.right() as f64, r.bottom() as f64)
}
