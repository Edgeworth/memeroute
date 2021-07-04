use eframe::egui::emath::RectTransform;
use eframe::egui::{epaint, Color32, Painter, Pos2, Rect, Response, Sense, Ui, Widget};
use lazy_static::lazy_static;
use memeroute::model::geom::{Pt, Rt};
use memeroute::model::pcb::{Component, Keepout, Padstack, Pcb, Shape, ShapeType};
use rust_decimal_macros::dec;

use crate::pcb::{to_pos2, to_rect};

lazy_static! {
    static ref PRIMARY: [Color32; 5] = [
        Color32::from_rgba_unmultiplied(20, 55, 173, 150),
        Color32::from_rgba_unmultiplied(89, 113, 193, 150),
        Color32::from_rgba_unmultiplied(56, 84, 178, 150),
        Color32::from_rgba_unmultiplied(14, 42, 133, 150),
        Color32::from_rgba_unmultiplied(9, 31, 105, 150),
    ];

    static ref SECONDARY: [Color32; 5] = [
        Color32::from_rgba_unmultiplied(255, 44, 0, 150),
        Color32::from_rgba_unmultiplied(255, 126, 99, 150),
        Color32::from_rgba_unmultiplied(255, 91, 57, 150),
        Color32::from_rgba_unmultiplied(197, 34, 0, 150),
        Color32::from_rgba_unmultiplied(155, 27, 0, 150),
    ];
}

#[derive(Debug, Clone, PartialEq)]
pub struct PcbView<'a> {
    pcb: &'a Pcb,
    screen_area: Rect,
    local_area: Rt,
    tf: RectTransform,
}

impl<'a> Widget for PcbView<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        ui.heading("pcb view");

        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap_finite(), Sense::click_and_drag());
        self.set_screen_area(&response.rect);
        self.render(&painter);
        response
    }
}

impl<'a> PcbView<'a> {
    pub fn new(pcb: &'a Pcb, local_area: Rt) -> Self {
        Self {
            pcb,
            screen_area: Rect::NOTHING,
            local_area,
            tf: RectTransform::identity(Rect::NOTHING),
        }
    }

    fn set_screen_area(&mut self, screen_area: &Rect) {
        self.screen_area = *screen_area;
        self.tf = RectTransform::from_to(to_rect(&self.local_area), self.screen_area);
    }

    fn pt(&self, v: &Pt) -> Pos2 {
        self.tf.transform_pos(to_pos2(v))
    }

    fn rect(&self, v: &Rt) -> Rect {
        self.tf.transform_rect(to_rect(v))
    }

    fn draw_shape(&self, p: &Painter, v: &Shape) {
        match &v.shape {
            ShapeType::Rect(s) => p.rect_filled(self.rect(s), 0.0, PRIMARY[0]),
            ShapeType::Circle(_) => {}
            ShapeType::Polygon(s) => {
                p.add(epaint::Shape::Path {
                    points: s.pts.iter().map(|pt| self.pt(pt)).collect(),
                    closed: true,
                    fill: SECONDARY[1],
                    stroke: epaint::Stroke::new(2.0, SECONDARY[0]),
                });
            }
            ShapeType::Path(_) => {}
            ShapeType::Arc(_) => {}
        };
    }

    fn draw_keepout(&self, p: &Painter, v: &Keepout) {
        self.draw_shape(p, &v.shape);
    }


    fn draw_padstack(&self, p: &Painter, v: &Padstack) {}

    fn draw_component(&self, p: &Painter, v: &Component) {}

    fn render(&self, p: &Painter) {
        p.rect_filled(self.screen_area, 0.0, Color32::WHITE);
        for keepout in self.pcb.keepouts() {
            self.draw_keepout(p, keepout);
        }
    }
}
