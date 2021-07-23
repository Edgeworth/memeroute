use eframe::egui::epaint::{Mesh, TessellationOptions, Tessellator};
use eframe::egui::{epaint, Color32, Context, PointerButton, Response, Sense, Ui, Widget};
use lazy_static::lazy_static;
use memeroute::model::pcb::{Component, Keepout, LayerShape, Padstack, Pcb, Pin, Side};
use memeroute::model::primitive::point::Pt;
use memeroute::model::primitive::rect::Rt;
use memeroute::model::primitive::shape::Shape;
use memeroute::model::primitive::{path, pt, ShapeOps};
use memeroute::model::tf::Tf;

use crate::pcb::primitives::{fill_circle, fill_polygon, fill_rt, stroke_path};
use crate::pcb::{to_pos2, to_pt, to_rt};

// Index 0 is front, index 1 is back.
lazy_static! {
    static ref KEEPOUT: Color32 = Color32::from_rgba_unmultiplied(155, 27, 0, 180);

    static ref OUTLINE: [Color32; 2] = [
        Color32::from_rgba_unmultiplied(89, 113, 193, 180),
        Color32::from_rgba_unmultiplied(168, 0, 186, 180)
    ];

    static ref BOUNDARY: Color32 = Color32::from_rgba_unmultiplied(255, 199, 46, 180);

    static ref PIN: [Color32; 2] = [
        Color32::from_rgba_unmultiplied(0, 27, 161, 180),
        Color32::from_rgba_unmultiplied(0, 27, 161, 180),
    ];

    static ref WIRE: [Color32; 2] = [
        Color32::from_rgba_unmultiplied(252, 3, 182, 180),
        Color32::from_rgba_unmultiplied(0, 166, 52, 180)
    ];

    static ref VIA: Color32 = Color32::from_rgba_unmultiplied(100, 100, 100, 180);

    static ref DEBUG: Color32 = Color32::from_rgba_unmultiplied(123, 0, 255, 180);
}

#[derive(Debug, Clone)]
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
            self.offset += pt(p.x as f64, p.y as f64);
        }

        if ui.rect_contains_pointer(response.rect) {
            let pos = to_pt(ui.ctx().input().pointer.interact_pos().unwrap());
            let delta = ui.ctx().input().scroll_delta.y as f64;
            let fac = 10.0 * delta / response.rect.height() as f64;
            self.offset = self.offset + (self.offset - pos) * fac;
            self.zoom *= 1.0 + fac;
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

    pub fn set_pcb(&mut self, pcb: Pcb) {
        self.pcb = pcb;
        self.dirty = true;
        self.mesh.clear(); // Regenerate mesh.
    }

    fn set_screen_area(&mut self, screen_area: Rt) {
        self.screen_area = screen_area;
        self.local_area = self.local_area.match_aspect(&self.screen_area);
        self.dirty = true;
    }

    fn layer_id_to_color_idx(&self, id: &str) -> usize {
        // TODO: Generalise this?
        match id {
            "F.Cu" => 0,
            "B.Cu" => 1,
            _ => 0,
        }
    }

    fn draw_shape(&self, tf: &Tf, v: &LayerShape, col: Color32) -> Vec<epaint::Shape> {
        let mut shapes = Vec::new();
        match &v.shape {
            Shape::Rect(s) => shapes.push(fill_rt(tf, s, col)),
            Shape::Circle(s) => shapes.push(fill_circle(tf, s.p(), s.r(), col)),
            Shape::Polygon(s) => shapes.push(fill_polygon(tf, s.pts(), s.tri_idx(), col)),
            Shape::Path(s) => {
                // Treat paths with a radius of 0 as having a radius of 0.1 mm (arbitrary).
                let r = if s.r() == 0.0 { 0.1 } else { s.r() };
                shapes.extend(stroke_path(tf, s.pts(), r, col))
            }
            _ => todo!(),
        }
        shapes
    }

    fn draw_keepout(&self, tf: &Tf, v: &Keepout, col: Color32) -> Vec<epaint::Shape> {
        self.draw_shape(tf, &v.shape, col)
    }

    fn draw_padstack(&self, tf: &Tf, v: &Padstack, col: Color32) -> Vec<epaint::Shape> {
        let mut shapes = Vec::new();
        for shape in v.shapes.iter() {
            shapes.extend(self.draw_shape(tf, shape, col));
        }
        shapes
    }

    fn draw_pin(&self, tf: &Tf, v: &Pin, col: Color32) -> Vec<epaint::Shape> {
        self.draw_padstack(&(tf * v.tf()), &v.padstack, col)
    }

    fn draw_component(&self, tf: &Tf, v: &Component) -> Vec<epaint::Shape> {
        let mut shapes = Vec::new();
        let side_idx = match v.side {
            Side::Front => 0,
            Side::Back => 1,
        };
        let tf = tf * v.tf();
        for outline in v.outlines.iter() {
            shapes.extend(self.draw_shape(&tf, outline, OUTLINE[side_idx]));
        }
        for keepout in v.keepouts.iter() {
            shapes.extend(self.draw_keepout(&tf, keepout, *KEEPOUT));
        }
        for pin in v.pins() {
            shapes.extend(self.draw_pin(&tf, pin, PIN[side_idx]))
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
            for wire in self.pcb.wires() {
                let col = WIRE[self.layer_id_to_color_idx(&wire.shape.layer)];
                let shapes = self.draw_shape(&tf, &wire.shape, col);
                Self::tessellate(ctx, &mut tess, &mut mesh, shapes);
            }
            for via in self.pcb.vias() {
                let shapes = self.draw_padstack(&via.tf(), &via.padstack, *VIA);
                Self::tessellate(ctx, &mut tess, &mut mesh, shapes);
            }
            for rt in self.pcb.debug_rts() {
                let mut pts = rt.pts().to_vec();
                pts.push(rt.pts()[0]);
                let shape = path(&pts, 0.1).shape();
                let shapes =
                    self.draw_shape(&tf, &LayerShape { shape, layer: "pcb".to_owned() }, *DEBUG);
                Self::tessellate(ctx, &mut tess, &mut mesh, shapes);
            }
            self.mesh = mesh;
        }
        let mut mesh = self.mesh.clone();
        if self.dirty {
            let inv = Tf::scale(pt(1.0, -1.0)); // Invert y axis
            let local_area = inv.rt(&self.local_area).bounds();
            let tf = Tf::translate(self.offset)
                * Tf::scale(pt(self.zoom, self.zoom))
                * Tf::affine(&local_area, &self.screen_area)
                * inv;
            for vert in mesh.vertices.iter_mut() {
                vert.pos = to_pos2(tf.pt(to_pt(vert.pos)));
            }
            self.dirty = false;
        }
        mesh
    }
}
