use eframe::egui::epaint::{Mesh, TessellationOptions, Tessellator};
use eframe::egui::{epaint, Color32, Context, Response, Sense, Ui, Widget};
use lazy_static::lazy_static;
use memeroute::model::geom::Rt;
use memeroute::model::pcb::{Component, Keepout, Padstack, Pcb, Shape, ShapeType};
use memeroute::model::transform::Tf;

use crate::pcb::primitives::{fill_circle, fill_polygon, fill_rect, stroke_path, stroke_polygon};
use crate::pcb::to_rt;

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
pub struct PcbView {
    pcb: Pcb,
    screen_area: Rt,
    local_area: Rt,
    tf: Tf,
    mesh: Mesh,
}

impl Widget for &mut PcbView {
    fn ui(self, ui: &mut Ui) -> Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap_finite(), Sense::click_and_drag());
        self.set_screen_area(to_rt(response.rect));
        let mesh = self.render(ui.ctx());
        painter.add(epaint::Shape::Mesh(mesh));
        response
    }
}

impl PcbView {
    pub fn new(pcb: Pcb, local_area: Rt) -> Self {
        Self {
            pcb,
            screen_area: Default::default(),
            local_area,
            tf: Tf::identity(),
            mesh: Mesh::default(),
        }
    }

    fn set_screen_area(&mut self, screen_area: Rt) {
        self.screen_area = screen_area;
        self.tf = Tf::affine(self.local_area, self.screen_area);
    }

    fn draw_shape(&self, tf: &Tf, v: &Shape) -> Vec<epaint::Shape> {
        let mut shapes = Vec::new();
        match &v.shape {
            ShapeType::Rect(s) => shapes.push(fill_rect(tf, *s, PRIMARY[0])),
            ShapeType::Circle(s) => shapes.push(fill_circle(tf, s.p, s.r, PRIMARY[0])),
            ShapeType::Polygon(s) => {
                shapes.push(fill_polygon(tf, &s.pts, SECONDARY[1]));
                shapes.extend(stroke_polygon(tf, &s.pts, s.width, SECONDARY[0]));
            }
            ShapeType::Path(s) => shapes.extend(stroke_path(tf, &s.pts, s.width, SECONDARY[0])),
            ShapeType::Arc(_) => todo!(),
        }
        shapes
    }

    fn draw_keepout(&self, tf: &Tf, v: &Keepout) -> Vec<epaint::Shape> {
        self.draw_shape(tf, &v.shape)
    }

    fn draw_padstack(&self, _tf: &Tf, _v: &Padstack) {}

    fn draw_component(&self, tf: &Tf, v: &Component) -> Vec<epaint::Shape> {
        let mut shapes = Vec::new();
        let tf = tf * Tf::translate(v.p);
        for outline in v.outlines.iter() {
            shapes.extend(self.draw_shape(&tf, outline));
        }
        for keepout in v.keepouts.iter() {
            shapes.extend(self.draw_keepout(&tf, keepout));
        }
        shapes
    }

    fn tessellate(
        ctx: &Context,
        tess: &mut Tessellator,
        mesh: &mut Mesh,
        shapes: Vec<epaint::Shape>,
    ) {
        for s in shapes.into_iter() {
            tess.tessellate_shape(ctx.fonts().texture().size(), s, mesh);
        }
    }

    fn render(&mut self, ctx: &Context) -> Mesh {
        if self.mesh.is_empty() {
            let mut mesh = Mesh::default();
            let mut tess = Tessellator::from_options(TessellationOptions {
                pixels_per_point: ctx.pixels_per_point(),
                aa_size: 1.0 / ctx.pixels_per_point(),
                anti_alias: false,
                ..Default::default()
            });
            Self::tessellate(
                ctx,
                &mut tess,
                &mut mesh,
                vec![fill_rect(&Tf::new(), self.screen_area, Color32::WHITE)],
            );
            for boundary in self.pcb.boundaries() {
                Self::tessellate(ctx, &mut tess, &mut mesh, self.draw_shape(&self.tf, boundary));
            }
            for keepout in self.pcb.keepouts() {
                Self::tessellate(ctx, &mut tess, &mut mesh, self.draw_keepout(&self.tf, keepout));
            }
            for component in self.pcb.components() {
                Self::tessellate(
                    ctx,
                    &mut tess,
                    &mut mesh,
                    self.draw_component(&self.tf, component),
                );
            }
            self.mesh = mesh;
        }
        self.mesh.clone()
    }
}
