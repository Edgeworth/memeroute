use std::collections::HashMap;

use eyre::{eyre, Result};
use memegeom::primitive::circle::Circle;
use memegeom::primitive::path_shape::Path;
use memegeom::primitive::point::Pt;
use memegeom::primitive::polygon::Poly;
use memegeom::primitive::rect::Rt;
use memegeom::primitive::shape::Shape;
use strum::IntoEnumIterator;

use crate::model::pcb::{
    Component, LayerKind, LayerSet, LayerShape, Net, Padstack, Pcb, Via, Wire,
};
use crate::name::Id;

const MAX_COL: usize = 120;
const INDENT: usize = 2;
const NEWLINE_MAX_INDENT: usize = 8;
const MM_RESOLUTION: usize = 100000;

#[must_use]
#[derive(Debug, Clone)]
pub struct PcbToSession {
    pcb: Pcb,
    s: String,
    indent: usize, // Current indent.
    col: usize,    // Current column number.
}

impl PcbToSession {
    pub fn new(pcb: Pcb) -> Self {
        Self { pcb, s: String::new(), indent: 0, col: 0 }
    }

    fn newline(&mut self) {
        self.s += "\n";
        self.s += &" ".repeat((self.indent - 1) * INDENT);
    }

    fn append(&mut self, s: &str) {
        // Newline if we would go over the column limit.
        if s.len() < MAX_COL && self.col + s.len() > MAX_COL {
            self.newline();
        }
        self.s += s;
    }

    fn token(&mut self, tok: &str) {
        self.append(" ");
        self.append(tok);
    }

    fn id(&mut self, id: Id) {
        self.name(&self.pcb.to_name(id));
    }

    fn name(&mut self, name: &str) {
        // TODO: Assumes double quotes.
        self.token(&("\"".to_owned() + name + "\""));
    }

    fn coord(&mut self, v: f64) {
        let v = (v * MM_RESOLUTION as f64).round() as i64;
        self.token(&v.to_string());
    }

    fn rot(&mut self, v: f64) {
        // Can have up to two decimal places according to spec.
        self.token(&format!("{v:.2}"));
    }

    fn side(&mut self, back: bool) {
        let side = if back { "back" } else { "front" };
        self.token(side);
    }

    fn begin(&mut self, name: &str) {
        self.indent += 1;
        if self.indent != 1 && self.indent < NEWLINE_MAX_INDENT {
            // Put stuff on a newline if we aren't too indented.
            self.newline();
        }
        self.append("(");
        self.append(name);
    }

    fn end(&mut self) {
        self.append(")");
        self.indent -= 1;
    }

    fn resolution(&mut self) {
        self.begin("resolution");
        self.token("mm");
        self.token(&MM_RESOLUTION.to_string());
        self.end();
    }

    fn component(&mut self, name: &str, cs: Vec<Component>) {
        self.begin("component");
        self.name(name);
        for c in cs {
            self.begin("place");
            self.id(c.id);
            self.pt(c.p);
            self.side(c.flipped());
            self.rot(c.rotation);
            self.end();
        }
        self.end();
    }

    fn pt(&mut self, p: Pt) {
        self.coord(p.x);
        self.coord(p.y);
    }

    fn circle(&mut self, layer: &str, s: &Circle) {
        self.begin("circle");
        self.name(layer);
        self.coord(s.r() * 2.0);
        self.pt(s.p());
        self.end();
    }

    fn path(&mut self, layer: &str, s: &Path) {
        self.begin("path");
        self.name(layer);
        self.coord(s.r() * 2.0);
        for pt in s.pts() {
            self.pt(*pt);
        }
        self.end();
    }

    fn polygon(&mut self, layer: &str, s: &Poly) {
        self.begin("polygon");
        self.name(layer);
        self.coord(0.0);
        for pt in s.pts() {
            self.pt(*pt);
        }
        self.end();
    }

    fn rect(&mut self, layer: &str, s: &Rt) {
        self.begin("rect");
        self.name(layer);
        self.pt(s.bl());
        self.pt(s.tr());
        self.end();
    }

    fn layer_id(&self, l: LayerSet) -> Option<String> {
        // Try to find a layer ID or specctra layer ID name that fits this
        // layer set.
        if let Some(lid) = l.id() {
            // If only one layer is set, just use that layer's name.
            Some(self.pcb.to_name(self.pcb.layer_by_id(lid).name_id))
        } else {
            // Otherwise, search for a LayerKind that gives this set.
            for kind in LayerKind::iter() {
                if l == self.pcb.layers_by_kind(kind) {
                    return Some(
                        match kind {
                            LayerKind::All => "all",
                            LayerKind::Signal => "signal",
                            LayerKind::Power => "power",
                            LayerKind::Mixed => "mixed",
                            LayerKind::Jumper => "jumper",
                        }
                        .to_string(),
                    );
                }
            }
            None
        }
    }

    fn shape(&mut self, shape: &LayerShape) {
        let l = self.layer_id(shape.layers).unwrap();
        match &shape.shape {
            Shape::Circle(s) => self.circle(&l, s),
            Shape::Path(s) => self.path(&l, s),
            Shape::Polygon(s) => self.polygon(&l, s),
            Shape::Rect(s) => self.rect(&l, s),
            _ => unimplemented!(), // TODO: Transform these shapes.
        }
    }

    fn padstack(&mut self, ps: &Padstack) {
        self.begin("padstack");
        self.id(ps.id);

        for shape in &ps.shapes {
            self.begin("shape");
            self.shape(shape);
            self.end();
        }

        if ps.attach {
            self.begin("attach");
            self.token("on");
            self.end();
        }

        self.end();
    }

    fn wire(&mut self, w: &Wire) {
        self.begin("wire");
        self.shape(&w.shape);
        self.end();
    }

    fn via(&mut self, v: &Via) {
        self.begin("via");
        self.id(v.padstack.id);
        self.pt(v.p);
        self.begin("net");
        self.id(v.net_id);
        self.end();
        self.end();
    }

    fn net(&mut self, net: &Net, wires: &[Wire], vias: &[Via]) {
        self.begin("net");
        self.id(net.id);
        for wire in wires {
            self.wire(wire);
        }
        for via in vias {
            self.via(via);
        }
        self.end();
    }

    pub fn convert(mut self) -> Result<String> {
        let pcb = self.pcb.clone();

        self.begin("session");
        self.id(pcb.pcb_id());

        self.begin("base_design");
        self.id(pcb.pcb_id());
        self.end();

        self.begin("placement");

        self.resolution();

        let mut footprints: HashMap<String, Vec<Component>> = HashMap::new();
        for c in pcb.components() {
            footprints.entry(pcb.to_name(c.footprint_id)).or_insert_with(Vec::new).push(c.clone());
        }
        for (name, cs) in footprints {
            self.component(&name, cs);
        }

        self.end();

        self.begin("routes");
        self.resolution();

        self.begin("library_out");
        // Output vias used
        for ps in pcb.via_padstacks() {
            self.padstack(ps);
        }
        self.end();

        self.begin("network_out");
        let mut nets: HashMap<Id, (Net, Vec<Wire>, Vec<Via>)> = HashMap::new();
        for net in pcb.nets() {
            nets.insert(net.id, (net.clone(), Vec::new(), Vec::new()));
        }
        for wire in pcb.wires() {
            nets.get_mut(&wire.net_id)
                .ok_or_else(|| eyre!("missing net with name {}", pcb.to_name(wire.net_id)))?
                .1
                .push(wire.clone());
        }
        for via in pcb.vias() {
            nets.get_mut(&via.net_id)
                .ok_or_else(|| eyre!("missing net with name {}", pcb.to_name(via.net_id)))?
                .2
                .push(via.clone());
        }

        for (net, wires, vias) in nets.values() {
            self.net(net, wires, vias);
        }
        self.end();

        self.end();

        self.end();
        Ok(self.s)
    }
}
