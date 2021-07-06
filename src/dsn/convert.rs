use std::collections::HashMap;

use eyre::{eyre, Result};

use crate::dsn::types::{
    DsnComponent, DsnDimensionUnit, DsnId, DsnImage, DsnKeepout, DsnKeepoutType, DsnNet,
    DsnPadstack, DsnPcb, DsnPin, DsnRect, DsnShape, DsnSide,
};
use crate::model::geom::{Pt, Rt};
use crate::model::pcb::{
    Arc, Circle, Component, Keepout, KeepoutType, Layer, Net, Padstack, Path, Pcb, Pin, PinRef,
    Polygon, Shape, ShapeType, Side,
};

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
        Rt {
            x: self.coord(v.rect.x),
            y: -self.coord(v.rect.y), // Convert to positive y is up axes.
            w: self.coord(v.rect.w),
            h: self.coord(v.rect.h),
        }
    }

    fn pt(&self, v: Pt) -> Pt {
        // Convert to positive y is up axes.
        Pt { x: self.coord(v.x), y: -self.coord(v.y) }
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
            DsnShape::QArc(v) => Shape {
                layer: v.layer_id.clone(),
                shape: ShapeType::Arc(Arc {
                    width: self.coord(v.aperture_width),
                    start: self.pt(v.start),
                    end: self.pt(v.end),
                    center: self.pt(v.center),
                }),
            },
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
            rotation: v.rotation,
            p: self.pt(v.p),
        })
    }


    fn image(&self, v: &DsnImage) -> Result<Component> {
        Ok(Component {
            outlines: v.outlines.iter().map(|p| self.shape(p)).collect(),
            keepouts: v.keepouts.iter().map(|p| self.keepout(p)).collect(),
            pins: v.pins.iter().map(|p| self.pin(p)).collect::<Result<_>>()?,
            ..Default::default()
        })
    }

    fn components(&self, v: &DsnComponent) -> Result<Vec<Component>> {
        let mut components = Vec::new();
        for pl in v.refs.iter() {
            let component = self
                .images
                .get(&v.image_id)
                .ok_or_else(|| eyre!("missing image with id {}", v.image_id))?
                .clone();
            let component = Component {
                id: pl.component_id.clone(),
                p: self.pt(pl.p),
                side: match pl.side {
                    DsnSide::Front => Side::Front,
                    DsnSide::Back => Side::Back,
                    DsnSide::Both => return Err(eyre!("invalid side specification")),
                },
                rotation: pl.rotation,
                ..component
            };
            components.push(component);
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
