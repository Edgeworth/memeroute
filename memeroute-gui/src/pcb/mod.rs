use eframe::egui::{Pos2, Rect};
use memeroute::model::geom::{Pt, Rt};
use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;

pub mod pcb_view;
pub mod primitives;

pub fn to_pos2(p: Pt) -> Pos2 {
    Pos2::new(p.x.to_f32().unwrap(), p.y.to_f32().unwrap())
}

pub fn to_rect(r: Rt) -> Rect {
    Rect::from_two_pos(to_pos2(r.tl()), to_pos2(r.br()))
}

pub fn to_rt(r: Rect) -> Rt {
    Rt::new(
        Decimal::from_f32(r.left()).unwrap(),
        Decimal::from_f32(r.top()).unwrap(),
        Decimal::from_f32(r.width()).unwrap(),
        Decimal::from_f32(r.height()).unwrap(),
    )
}
