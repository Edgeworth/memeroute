use std::sync::LazyLock;

use eframe::egui::epaint::{Mesh, TessellationOptions, Tessellator};
use eframe::egui::{epaint, Color32, Context, PointerButton, Response, Sense, Ui, Widget};
use memegeom::primitive::point::Pt;
use memegeom::primitive::rect::Rt;
use memegeom::primitive::shape::Shape;
use memegeom::primitive::{path, pt, ShapeOps};
use memegeom::tf::Tf;
use memeroute::model::pcb::{
    Component, Keepout, LayerId, LayerSet, LayerShape, Padstack, Pcb, Pin,
};

use crate::pcb::primitives::{fill_circle, fill_polygon, fill_rt, stroke_path};
use crate::pcb::{to_pos2, to_pt, to_rt};

// Index 0 is front, index 1 is back.
// TODO!! This

static KEEPOUT: LazyLock<Color32> =
    LazyLock::new(|| Color32::from_rgba_unmultiplied(155, 27, 0, 180));

static OUTLINE: LazyLock<[Color32; 2]> = LazyLock::new(|| {
    [
        Color32::from_rgba_unmultiplied(89, 113, 193, 180),
        Color32::from_rgba_unmultiplied(168, 0, 186, 180),
    ]
});

static BOUNDARY: LazyLock<Color32> =
    LazyLock::new(|| Color32::from_rgba_unmultiplied(255, 199, 46, 180));

static PIN: LazyLock<[Color32; 2]> = LazyLock::new(|| {
    [
        Color32::from_rgba_unmultiplied(0, 27, 161, 180),
        Color32::from_rgba_unmultiplied(0, 27, 161, 180),
    ]
});

static WIRE: LazyLock<[Color32; 2]> = LazyLock::new(|| {
    [
        Color32::from_rgba_unmultiplied(252, 3, 182, 180),
        Color32::from_rgba_unmultiplied(0, 166, 52, 180),
    ]
});

static VIA: LazyLock<Color32> =
    LazyLock::new(|| Color32::from_rgba_unmultiplied(100, 100, 100, 180));

static DEBUG: LazyLock<Color32> =
    LazyLock::new(|| Color32::from_rgba_unmultiplied(123, 0, 255, 180));

#[must_use]
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
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click_and_drag());

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
            screen_area: Rt::default(),
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

    fn layer_id_to_color_idx(id: LayerId) -> usize {
        id
    }

    fn draw_shape(tf: &Tf, v: &LayerShape, col: Color32) -> Vec<epaint::Shape> {
        let mut shapes = Vec::new();
        match &v.shape {
            Shape::Rect(s) => shapes.push(fill_rt(tf, s, col)),
            Shape::Circle(s) => shapes.push(fill_circle(tf, s.p(), s.r(), col)),
            Shape::Polygon(s) => shapes.push(fill_polygon(tf, s.pts(), s.tri_idx(), col)),
            Shape::Path(s) => {
                // Treat paths with a radius of 0 as having a radius of 0.1 mm (arbitrary).
                let r = if s.r() == 0.0 { 0.1 } else { s.r() };
                shapes.extend(stroke_path(tf, s.pts(), r, col));
            }
            _ => todo!(),
        }
        shapes
    }

    fn draw_keepout(tf: &Tf, v: &Keepout, col: Color32) -> Vec<epaint::Shape> {
        Self::draw_shape(tf, &v.shape, col)
    }

    fn draw_padstack(tf: &Tf, v: &Padstack, col: Color32) -> Vec<epaint::Shape> {
        let mut shapes = Vec::new();
        for shape in &v.shapes {
            shapes.extend(Self::draw_shape(tf, shape, col));
        }
        shapes
    }

    fn draw_pin(tf: &Tf, v: &Pin, col: Color32) -> Vec<epaint::Shape> {
        Self::draw_padstack(&(tf * v.tf()), &v.padstack, col)
    }

    fn draw_component(tf: &Tf, v: &Component) -> Vec<epaint::Shape> {
        let mut shapes = Vec::new();
        let tf = tf * v.tf();
        // TODO: Push this colour handling down, just do per layer colours.
        for outline in &v.outlines {
            let idx = outline.layers.first().unwrap();
            shapes.extend(Self::draw_shape(&tf, outline, OUTLINE[idx]));
        }
        for keepout in &v.keepouts {
            shapes.extend(Self::draw_keepout(&tf, keepout, *KEEPOUT));
        }
        for pin in v.pins() {
            let idx = pin.padstack.layers().first().unwrap();
            shapes.extend(Self::draw_pin(&tf, pin, PIN[idx]));
        }
        shapes
    }

    fn tessellate(tess: &mut Tessellator, mesh: &mut Mesh, shapes: Vec<epaint::Shape>) {
        for s in shapes {
            tess.tessellate_shape(s, mesh);
        }
    }

    fn render(&mut self, ctx: &Context) -> Mesh {
        if self.mesh.is_empty() {
            let mut mesh = Mesh::default();
            let tf = Tf::new();
            let mut tess = Tessellator::new(
                ctx.pixels_per_point(),
                TessellationOptions { feathering: false, ..Default::default() },
                ctx.fonts().font_image_size(),
                vec![],
            );
            for boundary in self.pcb.boundaries() {
                let shapes = Self::draw_shape(&tf, boundary, *BOUNDARY);
                Self::tessellate(&mut tess, &mut mesh, shapes);
            }
            for keepout in self.pcb.keepouts() {
                let shapes = Self::draw_keepout(&tf, keepout, *KEEPOUT);
                Self::tessellate(&mut tess, &mut mesh, shapes);
            }
            for component in self.pcb.components() {
                let shapes = Self::draw_component(&tf, component);
                Self::tessellate(&mut tess, &mut mesh, shapes);
            }
            for wire in self.pcb.wires() {
                // TODO!!: Fix up layerset to color mapping.
                let col = WIRE[Self::layer_id_to_color_idx(wire.shape.layers.id().unwrap())];
                let shapes = Self::draw_shape(&tf, &wire.shape, col);
                Self::tessellate(&mut tess, &mut mesh, shapes);
            }
            for via in self.pcb.vias() {
                let shapes = Self::draw_padstack(&via.tf(), &via.padstack, *VIA);
                Self::tessellate(&mut tess, &mut mesh, shapes);
            }
            for rt in self.pcb.debug_rts() {
                let mut pts = rt.pts().to_vec();
                pts.push(rt.pts()[0]);
                let shape = path(&pts, 0.05).shape();
                let shapes =
                    Self::draw_shape(&tf, &LayerShape { shape, layers: LayerSet::empty() }, *DEBUG);
                Self::tessellate(&mut tess, &mut mesh, shapes);
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
            for vert in &mut mesh.vertices {
                vert.pos = to_pos2(tf.pt(to_pt(vert.pos)));
            }
            self.dirty = false;
        }
        mesh
    }
}
