use eframe::egui::epaint::{Mesh, TessellationOptions, Tessellator};
use eframe::egui::{epaint, Color32, Context, PointerButton, Response, Sense, Ui, Widget};
use lazy_static::lazy_static;
use memeroute::model::geom::{Pt, Rt};
use memeroute::model::pcb::{Component, Keepout, Padstack, Pcb, Shape, ShapeType, Side};
use memeroute::model::transform::Tf;

use crate::pcb::primitives::{fill_circle, fill_polygon, fill_rect, stroke_path, stroke_polygon};
use crate::pcb::{to_pos2, to_pt, to_rt};

// Index 0 is front, index 1 is back.
lazy_static! {
    static ref KEEPOUT: Color32 = Color32::from_rgba_unmultiplied(155, 27, 0, 255);

    static ref OUTLINE: [Color32; 2] = [
        Color32::from_rgba_unmultiplied(89, 113, 193, 255),
        Color32::from_rgba_unmultiplied(168, 0, 186, 255)
    ];

    static ref BOUNDARY: Color32 = Color32::from_rgba_unmultiplied(255, 199, 46, 255);

    static ref PRIMARY: [Color32; 5] = [
        Color32::from_rgba_unmultiplied(20, 55, 173, 255),
        Color32::from_rgba_unmultiplied(89, 113, 193, 255),
        Color32::from_rgba_unmultiplied(56, 84, 178, 255),
        Color32::from_rgba_unmultiplied(14, 42, 133, 255),
        Color32::from_rgba_unmultiplied(9, 31, 105, 255),
    ];

    static ref SECONDARY: [Color32; 5] = [
        Color32::from_rgba_unmultiplied(255, 44, 0, 255),
        Color32::from_rgba_unmultiplied(255, 126, 99, 255),
        Color32::from_rgba_unmultiplied(255, 91, 57, 255),
        Color32::from_rgba_unmultiplied(197, 34, 0, 255),
        Color32::from_rgba_unmultiplied(155, 27, 0, 255),
    ];
}

#[derive(Debug, Clone, PartialEq)]
pub struct PcbView {
    pcb: Pcb,
    screen_area: Rt,
    local_area: Rt,
    offset: Pt,
    zoom: f64,
    dirty: bool,
    mesh: Mesh,
}

impl Widget for &mut PcbView {
    fn ui(self, ui: &mut Ui) -> Response {
        let (response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap_finite(), Sense::click_and_drag());

        if response.dragged_by(PointerButton::Middle) {
            let p = response.drag_delta();
            self.offset += Pt::new(p.x as f64, p.y as f64);
        }

        if ui.rect_contains_pointer(response.rect) {
            // TODO: Zoom on current loc
            let delta = ui.ctx().input().scroll_delta.y as f64;
            self.zoom *= 1.0 + 4.0 * delta / response.rect.height() as f64;
        }

        self.set_screen_area(to_rt(response.rect));
        let mesh = self.render(ui.ctx());
        painter.rect_filled(response.rect, 0.0, Color32::WHITE);
        painter.add(epaint::Shape::Mesh(mesh));
        response
    }
}

impl PcbView {
    pub fn new(pcb: Pcb, local_area: Rt) -> Self {
        Self {
            pcb,
            local_area,
            dirty: true,
            offset: Pt::zero(),
            zoom: 1.0,
            screen_area: Default::default(),
            mesh: Mesh::default(),
        }
    }

    fn set_screen_area(&mut self, screen_area: Rt) {
        self.screen_area = screen_area;
        self.dirty = true;
    }

    fn draw_shape(&self, tf: &Tf, v: &Shape, col: Color32) -> Vec<epaint::Shape> {
        let mut shapes = Vec::new();
        match &v.shape {
            ShapeType::Rect(s) => shapes.push(fill_rect(tf, *s, col)),
            ShapeType::Circle(s) => shapes.push(fill_circle(tf, s.p, s.r, col)),
            ShapeType::Polygon(s) => {
                shapes.push(fill_polygon(tf, &s.pts, col));
                shapes.extend(stroke_polygon(tf, &s.pts, s.width, col));
            }
            ShapeType::Path(s) => {
                // Treat paths with width 0 as having a width of 0.2 mm;
                let w = if s.width == 0.0 { 0.2 } else { s.width };
                shapes.extend(stroke_path(tf, &s.pts, w, col))
            }
            ShapeType::Arc(_) => todo!(),
        }
        shapes
    }

    fn draw_keepout(&self, tf: &Tf, v: &Keepout, col: Color32) -> Vec<epaint::Shape> {
        self.draw_shape(tf, &v.shape, col)
    }

    fn draw_padstack(&self, _tf: &Tf, _v: &Padstack) {}

    fn draw_component(&self, tf: &Tf, v: &Component) -> Vec<epaint::Shape> {
        let mut shapes = Vec::new();
        let side_idx = match v.side {
            Side::Front => 0,
            Side::Back => 1,
        };
        // TODO(check): is it correct to flip both axes?
        let side_tf =
            if v.side == Side::Back { Tf::scale(Pt::new(-1.0, -1.0)) } else { Tf::identity() };
        let tf = tf * Tf::translate(v.p) * Tf::rotate(v.rotation) * side_tf;

        for outline in v.outlines.iter() {
            shapes.extend(self.draw_shape(&tf, outline, OUTLINE[side_idx]));
        }
        for keepout in v.keepouts.iter() {
            shapes.extend(self.draw_keepout(&tf, keepout, *KEEPOUT));
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
            let tf = Tf::new();
            let mut tess = Tessellator::from_options(TessellationOptions {
                pixels_per_point: ctx.pixels_per_point(),
                aa_size: 1.0 / ctx.pixels_per_point(),
                anti_alias: false,
                ..Default::default()
            });
            for boundary in self.pcb.boundaries() {
                let shapes = self.draw_shape(&tf, boundary, *BOUNDARY);
                Self::tessellate(ctx, &mut tess, &mut mesh, shapes);
            }
            for keepout in self.pcb.keepouts() {
                let shapes = self.draw_keepout(&tf, keepout, *KEEPOUT);
                Self::tessellate(ctx, &mut tess, &mut mesh, shapes);
            }
            for component in self.pcb.components() {
                let shapes = self.draw_component(&tf, component);
                Self::tessellate(ctx, &mut tess, &mut mesh, shapes);
            }
            self.mesh = mesh;
        }
        let mut mesh = self.mesh.clone();
        if self.dirty {
            let tf = Tf::translate(self.offset)
                * Tf::scale(Pt::new(self.zoom, self.zoom))
                * Tf::affine(self.local_area, self.screen_area);
            for vert in mesh.vertices.iter_mut() {
                vert.pos = to_pos2(tf.pt(to_pt(vert.pos)));
            }
            self.dirty = false;
        }
        mesh
    }
}
