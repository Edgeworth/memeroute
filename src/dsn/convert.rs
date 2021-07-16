use std::collections::HashMap;

use eyre::{eyre, Result};

use crate::dsn::types::{
    DsnComponent, DsnDimensionUnit, DsnId, DsnImage, DsnKeepout, DsnKeepoutType, DsnNet,
    DsnPadstack, DsnPcb, DsnPin, DsnRect, DsnShape, DsnSide,
};
use crate::model::geom::math::pt_eq;
use crate::model::pcb::{
    Component, Keepout, KeepoutType, Layer, LayerShape, Net, Padstack, Pcb, Pin, PinRef, Side,
};
use crate::model::pt::Pt;
use crate::model::primitive::circle::Circle;
use crate::model::primitive::path::Path;
use crate::model::primitive::polygon::Polygon;
use crate::model::primitive::rt::Rt;

#[derive(Debug, Clone)]
pub struct Converter {
    dsn: DsnPcb,
    pcb: Pcb,
    padstacks: HashMap<DsnId, Padstack>,
    images: HashMap<DsnId, Component>,
}

impl Converter {
    pub fn new(dsn: DsnPcb) -> Self {
        Self { dsn, pcb: Default::default(), padstacks: HashMap::new(), images: HashMap::new() }
    }

    fn mm(&self) -> f64 {
        match self.dsn.resolution.dimension {
            DsnDimensionUnit::Inch => 25.4,
            DsnDimensionUnit::Mil => 0.0254,
            DsnDimensionUnit::Cm => 10.0,
            DsnDimensionUnit::Mm => 1.0,
            DsnDimensionUnit::Um => 0.001,
        }
    }

    fn coord(&self, v: f64) -> f64 {
        self.mm() * v
    }

    fn rect(&self, v: &DsnRect) -> Rt {
        Rt::new(
            self.coord(v.rect.l()),
            self.coord(v.rect.b()),
            self.coord(v.rect.w()),
            self.coord(v.rect.h()),
        )
    }

    fn pt(&self, v: Pt) -> Pt {
        Pt { x: self.coord(v.x), y: self.coord(v.y) }
    }

    fn rot(&self, r: f64) -> f64 {
        r
    }

    fn shape(&self, v: &DsnShape) -> LayerShape {
        match v {
            DsnShape::Rect(v) => {
                LayerShape { layer: v.layer_id.clone(), shape: self.rect(v).shape() }
            }
            DsnShape::Circle(v) => LayerShape {
                layer: v.layer_id.clone(),
                shape: Circle::new(self.pt(v.p), self.coord(v.diameter / 2.0)).shape(),
            },
            DsnShape::Polygon(v) => {
                let mut pts: Vec<Pt> = v.pts.iter().map(|&v| self.pt(v)).collect();
                // Polygons seem to have the first vertex repeated.
                if pts.len() >= 2 && pt_eq(*pts.first().unwrap(), *pts.last().unwrap()) {
                    pts.pop();
                }
                LayerShape {
                    layer: v.layer_id.clone(),
                    shape: Polygon::new(&pts, self.coord(v.aperture_width)).shape(),
                }
            }
            DsnShape::Path(v) => LayerShape {
                layer: v.layer_id.clone(),
                shape: Path::new(
                    &v.pts.iter().map(|&v| self.pt(v)).collect::<Vec<_>>(),
                    self.coord(v.aperture_width),
                )
                .shape(),
            },
            DsnShape::QArc(_v) => todo!(),
        }
    }

    fn keepout(&self, v: &DsnKeepout) -> Keepout {
        Keepout {
            kind: match v.keepout_type {
                DsnKeepoutType::Keepout => KeepoutType::Keepout,
                DsnKeepoutType::ViaKeepout => KeepoutType::ViaKeepout,
                DsnKeepoutType::WireKeepout => KeepoutType::WireKeepout,
            },
            shape: self.shape(&v.shape),
        }
    }

    fn padstack(&self, v: &DsnPadstack) -> Padstack {
        Padstack {
            id: v.padstack_id.clone(),
            shapes: v.shapes.iter().map(|s| self.shape(&s.shape)).collect(),
            attach: v.attach,
        }
    }

    fn pin(&self, v: &DsnPin) -> Result<Pin> {
        Ok(Pin {
            id: v.pin_id.clone(),
            padstack: self
                .padstacks
                .get(&v.padstack_id)
                .ok_or_else(|| eyre!("missing padstack with id {}", v.padstack_id))?
                .clone(),
            rotation: self.rot(v.rotation),
            p: self.pt(v.p),
        })
    }


    fn image(&self, v: &DsnImage) -> Result<Component> {
        let mut c = Component::default();
        c.outlines = v.outlines.iter().map(|p| self.shape(p)).collect();
        c.keepouts = v.keepouts.iter().map(|p| self.keepout(p)).collect();
        for pin in v.pins.iter() {
            c.add_pin(self.pin(pin)?);
        }
        Ok(c)
    }

    fn components(&self, v: &DsnComponent) -> Result<Vec<Component>> {
        let mut components = Vec::new();
        for pl in v.refs.iter() {
            let mut c = self
                .images
                .get(&v.image_id)
                .ok_or_else(|| eyre!("missing image with id {}", v.image_id))?
                .clone();
            c.id = pl.component_id.clone();
            c.p = self.pt(pl.p);
            c.side = match pl.side {
                DsnSide::Front => Side::Front,
                DsnSide::Back => Side::Back,
                DsnSide::Both => return Err(eyre!("invalid side specification")),
            };
            c.rotation = self.rot(pl.rotation);
            components.push(c);
        }
        Ok(components)
    }

    fn net(&self, v: &DsnNet) -> Result<Net> {
        Ok(Net {
            id: v.net_id.clone(),
            pins: v
                .pins
                .iter()
                .map(|p| PinRef { component: p.component_id.clone(), pin: p.pin_id.clone() })
                .collect(),
        })
    }

    fn convert_padstacks(&mut self) -> Result<()> {
        for v in self.dsn.library.padstacks.iter() {
            if self.padstacks.insert(v.padstack_id.clone(), self.padstack(v)).is_some() {
                return Err(eyre!("duplicate padstack with id {}", v.padstack_id));
            }
        }
        Ok(())
    }

    fn convert_images(&mut self) -> Result<()> {
        for v in self.dsn.library.images.iter() {
            if self.images.insert(v.image_id.clone(), self.image(v)?).is_some() {
                return Err(eyre!("duplicate image with id {}", v.image_id));
            }
        }
        Ok(())
    }

    pub fn convert(mut self) -> Result<Pcb> {
        self.pcb.set_id(&self.dsn.pcb_id);
        if self.dsn.unit.dimension != self.dsn.resolution.dimension {
            return Err(eyre!(
                "unit override unimplemented: {} {}",
                self.dsn.unit.dimension,
                self.dsn.resolution.dimension
            ));
        }
        self.convert_padstacks()?; // Padstacks are used in images.
        self.convert_images()?;

        // Physical structure:
        for v in self.dsn.structure.layers.iter() {
            self.pcb.add_layer(Layer::new(&v.layer_name));
        }
        for v in self.dsn.structure.boundaries.iter() {
            // Convert boundaries to closed shapes.
            let LayerShape { layer, shape } = self.shape(v);
            self.pcb.add_boundary(LayerShape { layer, shape: shape.filled() });
        }
        for v in self.dsn.structure.keepouts.iter() {
            self.pcb.add_keepout(self.keepout(v));
        }
        for v in self.dsn.structure.vias.iter() {
            self.pcb.add_via_padstack(
                self.padstacks.get(v).ok_or_else(|| eyre!("unknown padstack id {}", v))?.clone(),
            );
        }
        for v in self.dsn.placement.components.iter() {
            for component in self.components(v)?.into_iter() {
                self.pcb.add_component(component);
            }
        }

        // Routing:
        for v in self.dsn.network.nets.iter() {
            self.pcb.add_net(self.net(v)?);
        }

        // TODO: Add wires
        // TODO: Add vias
        // TODO: Support classes for nets.
        Ok(self.pcb)
    }
}
