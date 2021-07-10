use std::collections::HashMap;

use eyre::{eyre, Result};

use crate::dsn::types::{
    DsnComponent, DsnDimensionUnit, DsnId, DsnImage, DsnKeepout, DsnKeepoutType, DsnNet,
    DsnPadstack, DsnPcb, DsnPin, DsnRect, DsnShape, DsnSide,
};
use crate::model::circle::Circle;
use crate::model::path::Path;
use crate::model::pcb::{
    Component, Keepout, KeepoutType, Layer, Net, Padstack, Pcb, Pin, PinRef, Shape, ShapeType, Side,
};
use crate::model::polygon::Polygon;
use crate::model::pt::Pt;
use crate::model::rt::Rt;

#[derive(Debug, Clone, PartialEq)]
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
        let h = self.coord(v.rect.h);
        Rt {
            x: self.coord(v.rect.x),
            y: -self.coord(v.rect.y) - h, // Convert to positive y is up axes.
            w: self.coord(v.rect.w),
            h,
        }
    }

    fn pt(&self, v: Pt) -> Pt {
        // Convert to positive y is up axes.
        Pt { x: self.coord(v.x), y: -self.coord(v.y) }
    }

    fn rot(&self, r: f64) -> f64 {
        // Since coords are flipped need to invert rotations.
        -r
    }

    fn shape(&self, v: &DsnShape) -> Shape {
        match v {
            DsnShape::Rect(v) => {
                Shape { layer: v.layer_id.clone(), shape: ShapeType::Rect(self.rect(v)) }
            }
            DsnShape::Circle(v) => Shape {
                layer: v.layer_id.clone(),
                shape: ShapeType::Circle(Circle {
                    r: self.coord(v.diameter / 2.0),
                    p: self.pt(v.p),
                }),
            },
            DsnShape::Polygon(v) => Shape {
                layer: v.layer_id.clone(),
                shape: ShapeType::Polygon(Polygon {
                    width: self.coord(v.aperture_width),
                    pts: v.pts.iter().map(|&v| self.pt(v)).collect(),
                }),
            },
            DsnShape::Path(v) => Shape {
                layer: v.layer_id.clone(),
                shape: ShapeType::Path(Path {
                    width: self.coord(v.aperture_width),
                    pts: v.pts.iter().map(|&v| self.pt(v)).collect(),
                }),
            },
            DsnShape::QArc(v) => todo!(),
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
            self.pcb.add_boundary(self.shape(v));
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
