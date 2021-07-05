use eframe::egui::{Color32, Painter, Response, Sense, Ui, Widget};
use lazy_static::lazy_static;
use memeroute::model::geom::{Pt, Rt};
use memeroute::model::pcb::{Component, Keepout, Padstack, Pcb, Shape, ShapeType};
use memeroute::model::transform::Tf;

use crate::pcb::primitives::{fill_circle, fill_polygon, stroke_path, stroke_polygon};
use crate::pcb::{to_rect, to_rt};

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
    screen_area: Rt,
    local_area: Rt,
    tf: Tf,
}

impl<'a> Widget for PcbView<'a> {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap_finite(), Sense::click_and_drag());
        self.set_screen_area(to_rt(response.rect));
        self.render(&painter);
        response
    }
}

impl<'a> PcbView<'a> {
    pub fn new(pcb: &'a Pcb, local_area: Rt) -> Self {
        Self { pcb, screen_area: Default::default(), local_area, tf: Tf::identity() }
    }

    fn set_screen_area(&mut self, screen_area: Rt) {
        self.screen_area = screen_area;
        self.tf = Tf::affine(self.local_area, self.screen_area);
    }

    fn draw_shape(&self, p: &Painter, tf: &Tf, v: &Shape) {
        match &v.shape {
            ShapeType::Rect(s) => p.rect_filled(to_rect(tf.rt(*s)), 0.0, PRIMARY[0]),
            ShapeType::Circle(s) => fill_circle(p, tf, s.p, s.r, PRIMARY[0]),
            ShapeType::Polygon(s) => {
                fill_polygon(p, tf, &s.pts, SECONDARY[1]);
                stroke_polygon(p, tf, &s.pts, s.width, SECONDARY[0]);
            }
            ShapeType::Path(s) => stroke_path(p, tf, &s.pts, s.width, SECONDARY[0]),
            ShapeType::Arc(s) => todo!(),
        };
    }

    fn draw_keepout(&self, p: &Painter, tf: &Tf, v: &Keepout) {
        self.draw_shape(p, tf, &v.shape);
    }


    fn draw_padstack(&self, p: &Painter, tf: &Tf, v: &Padstack) {}

    fn draw_component(&self, p: &Painter, tf: &Tf, v: &Component) {
        let tf = tf * Tf::translate(v.p);
        for outline in v.outlines.iter() {
            self.draw_shape(p, &tf, outline);
        }
        for keepout in v.keepouts.iter() {
            self.draw_keepout(p, &tf, keepout);
        }
    }

    fn render(&self, p: &Painter) {
        p.rect_filled(to_rect(self.screen_area), 0.0, Color32::WHITE);
        for boundary in self.pcb.boundaries() {
            self.draw_shape(p, &self.tf, boundary);
        }
        for keepout in self.pcb.keepouts() {
            self.draw_keepout(p, &self.tf, keepout);
        }
        for component in self.pcb.components() {
            self.draw_component(p, &self.tf, component);
        }
    }
}
