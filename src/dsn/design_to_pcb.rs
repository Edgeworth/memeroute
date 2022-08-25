use std::collections::HashMap;

use eyre::{eyre, Result};
use itertools::Itertools;
use memedsn::types::{
    DsnCircuit, DsnClass, DsnClearance, DsnClearanceType, DsnComponent, DsnDimensionUnit, DsnImage,
    DsnKeepout, DsnKeepoutType, DsnLayerType, DsnNet, DsnPadstack, DsnPcb, DsnPin, DsnRect,
    DsnRule, DsnShape, DsnSide,
};
use memegeom::geom::math::{eq, pt_eq};
use memegeom::primitive::point::Pt;
use memegeom::primitive::rect::Rt;
use memegeom::primitive::{circ, path, poly, rt, ShapeOps};
use strum::IntoEnumIterator;

use crate::model::pcb::{
    Clearance, Component, Keepout, KeepoutType, Layer, LayerId, LayerKind, LayerSet, LayerShape,
    Net, ObjectKind, Padstack, Pcb, Pin, PinRef, Rule, RuleSet,
};
use crate::name::Id;

#[must_use]
#[derive(Debug, Clone)]
pub struct DesignToPcb {
    dsn: DsnPcb,
    pcb: Pcb,
    padstacks: HashMap<Id, Padstack>,
    images: HashMap<Id, Component>,
    layers: HashMap<Id, LayerId>,
}

impl DesignToPcb {
    pub fn new(dsn: DsnPcb) -> Self {
        Self {
            dsn,
            pcb: Pcb::default(),
            padstacks: HashMap::new(),
            images: HashMap::new(),
            layers: HashMap::new(),
        }
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
        rt(
            self.coord(v.rect.l()),
            self.coord(v.rect.b()),
            self.coord(v.rect.r()),
            self.coord(v.rect.t()),
        )
    }

    fn pt(&self, v: Pt) -> Pt {
        Pt { x: self.coord(v.x), y: self.coord(v.y) }
    }

    fn rot(r: f64) -> f64 {
        r
    }

    fn layers(&self, name: &str) -> Result<LayerSet> {
        Ok(match name {
            "signal" => self.pcb.layers_by_kind(LayerKind::Signal),
            "jumper" => self.pcb.layers_by_kind(LayerKind::Jumper),
            "mixed" => self.pcb.layers_by_kind(LayerKind::Mixed),
            "power" => self.pcb.layers_by_kind(LayerKind::Power),
            "pcb" => self.pcb.layers_by_kind(LayerKind::All), // Pcb used for boundary. Put on all layers.
            _ => LayerSet::one(
                *self
                    .layers
                    .get(&self.pcb.to_id(name))
                    .ok_or_else(|| eyre!("unknown layer {}", name))?,
            ),
        })
    }

    fn shape(&self, v: &DsnShape) -> Result<LayerShape> {
        Ok(match v {
            DsnShape::Rect(v) => {
                LayerShape { layers: self.layers(&v.layer_id)?, shape: self.rect(v).shape() }
            }
            DsnShape::Circle(v) => LayerShape {
                layers: self.layers(&v.layer_id)?,
                shape: circ(self.pt(v.p), self.coord(v.diameter / 2.0)).shape(),
            },
            DsnShape::Polygon(v) => {
                let mut pts: Vec<Pt> = v.pts.iter().map(|&v| self.pt(v)).collect();
                // Polygons seem to have the first vertex repeated.
                if pts.len() >= 2 && pt_eq(*pts.first().unwrap(), *pts.last().unwrap()) {
                    pts.pop();
                }
                assert!(eq(v.aperture_width, 0.0), "aperture width for polygons is unsupported");
                LayerShape { layers: self.layers(&v.layer_id)?, shape: poly(&pts).shape() }
            }
            DsnShape::Path(v) => LayerShape {
                layers: self.layers(&v.layer_id)?,
                shape: path(
                    &v.pts.iter().map(|&v| self.pt(v)).collect::<Vec<_>>(),
                    self.coord(v.aperture_width) / 2.0,
                )
                .shape(),
            },
            DsnShape::QArc(_v) => todo!(),
        })
    }

    fn keepout(&self, v: &DsnKeepout) -> Result<Keepout> {
        Ok(Keepout {
            kind: match v.keepout_type {
                DsnKeepoutType::Keepout => KeepoutType::Keepout,
                DsnKeepoutType::ViaKeepout => KeepoutType::ViaKeepout,
                DsnKeepoutType::WireKeepout => KeepoutType::WireKeepout,
            },
            shape: self.shape(&v.shape)?,
        })
    }

    fn padstack(&self, v: &DsnPadstack) -> Result<Padstack> {
        Ok(Padstack {
            id: self.pcb.to_id(&v.padstack_id),
            shapes: v.shapes.iter().map(|s| self.shape(&s.shape)).collect::<Result<_>>()?,
            attach: v.attach,
        })
    }

    fn pin(&self, v: &DsnPin) -> Result<Pin> {
        Ok(Pin {
            id: self.pcb.to_id(&v.pin_id),
            padstack: self
                .padstacks
                .get(&self.pcb.to_id(&v.padstack_id))
                .ok_or_else(|| eyre!("missing padstack with id {}", v.padstack_id))?
                .clone(),
            rotation: Self::rot(v.rotation),
            p: self.pt(v.p),
        })
    }

    fn image(&self, v: &DsnImage) -> Result<Component> {
        let mut c = Component::default();
        c.footprint_id = self.pcb.to_id(&v.image_id);
        c.outlines = v.outlines.iter().map(|p| self.shape(p)).collect::<Result<_>>()?;
        c.keepouts = v.keepouts.iter().map(|p| self.keepout(p)).collect::<Result<_>>()?;
        for pin in &v.pins {
            c.add_pin(self.pin(pin)?);
        }
        Ok(c)
    }

    fn components(&self, v: &DsnComponent) -> Result<Vec<Component>> {
        let mut components = Vec::new();
        for pl in &v.refs {
            let mut c = self
                .images
                .get(&self.pcb.to_id(&v.image_id))
                .ok_or_else(|| eyre!("missing image with id {}", v.image_id))?
                .clone();
            c.id = self.pcb.to_id(&pl.component_id);
            c.p = self.pt(pl.p);
            c.rotation = Self::rot(pl.rotation);
            match pl.side {
                DsnSide::Front => {}
                DsnSide::Back => c.flip(self.pcb.layers().len()),
                DsnSide::Both => return Err(eyre!("invalid side specification")),
            };
            components.push(c);
        }
        Ok(components)
    }

    fn net(&self, v: &DsnNet) -> Net {
        Net {
            id: self.pcb.to_id(&v.net_id),
            pins: v
                .pins
                .iter()
                .map(|p| PinRef {
                    component: self.pcb.to_id(&p.component_id),
                    pin: self.pcb.to_id(&p.pin_id),
                })
                .collect(),
        }
    }

    fn clearance_type(v: &DsnClearanceType) -> Vec<(ObjectKind, ObjectKind)> {
        match v {
            DsnClearanceType::All | DsnClearanceType::DefaultSmd => {
                ObjectKind::iter().cartesian_product(ObjectKind::iter()).collect()
            }
            DsnClearanceType::SmdSmd => vec![(ObjectKind::Smd, ObjectKind::Smd)],
        }
    }

    fn clearance(&self, v: &DsnClearance) -> Clearance {
        let pairs = v.types.iter().fold(vec![], |mut a, b| {
            a.extend(Self::clearance_type(b));
            a
        });
        Clearance::new(self.coord(v.amount), &pairs)
    }

    fn rule(&self, v: &DsnRule) -> Rule {
        match v {
            DsnRule::Width(w) => Rule::Radius(self.coord(*w) / 2.0),
            DsnRule::Clearance(c) => Rule::Clearance(self.clearance(c)),
        }
    }

    fn circuit(&self, v: &DsnCircuit) -> Rule {
        match v {
            DsnCircuit::UseVia(name) => Rule::UseVia(self.pcb.to_id(name)),
        }
    }

    fn ruleset(&self, v: &DsnClass) -> Result<RuleSet> {
        let id = self.pcb.to_id(&v.class_id);
        let mut rules: Vec<Rule> = v.rules.iter().map(|r| self.rule(r)).collect();
        rules.extend(v.circuits.iter().map(|c| self.circuit(c)));
        RuleSet::new(id, rules)
    }

    fn convert_padstacks(&mut self) -> Result<()> {
        for v in &self.dsn.library.padstacks {
            if self.padstacks.insert(self.pcb.to_id(&v.padstack_id), self.padstack(v)?).is_some() {
                return Err(eyre!("duplicate padstack with id {}", v.padstack_id));
            }
        }
        Ok(())
    }

    fn convert_images(&mut self) -> Result<()> {
        for v in &self.dsn.library.images {
            if self.images.insert(self.pcb.to_id(&v.image_id), self.image(v)?).is_some() {
                return Err(eyre!("duplicate image with id {}", v.image_id));
            }
        }
        Ok(())
    }

    pub fn convert(mut self) -> Result<Pcb> {
        self.pcb.set_pcb_name(&self.dsn.pcb_id);
        if self.dsn.unit.dimension != self.dsn.resolution.dimension {
            return Err(eyre!(
                "unit override unimplemented: {} {}",
                self.dsn.unit.dimension,
                self.dsn.resolution.dimension
            ));
        }

        // Layers needed for padstacks and images.
        for (id, v) in self.dsn.structure.layers.iter().enumerate() {
            let id = id as LayerId;
            if self.layers.insert(self.pcb.to_id(&v.layer_name), id).is_some() {
                return Err(eyre!("duplicate layer with id {}", v.layer_name));
            }
            let kind = match v.layer_type {
                DsnLayerType::Signal => LayerKind::Signal,
                DsnLayerType::Power => LayerKind::Power,
                DsnLayerType::Mixed => LayerKind::Mixed,
                DsnLayerType::Jumper => LayerKind::Jumper,
            };
            self.pcb.add_layer(Layer {
                name_id: self.pcb.to_id(&v.layer_name),
                layer_id: id,
                kind,
            });
        }

        self.convert_padstacks()?; // Padstacks are used in images.
        self.convert_images()?;

        // Physical structure:
        for v in &self.dsn.structure.boundaries {
            // Convert boundaries to closed shapes.
            let LayerShape { layers, shape } = self.shape(v)?;
            self.pcb.add_boundary(LayerShape { layers, shape: shape.filled() });
        }
        for v in &self.dsn.structure.keepouts {
            self.pcb.add_keepout(self.keepout(v)?);
        }
        for v in &self.dsn.structure.vias {
            self.pcb.add_via_padstack(
                self.padstacks
                    .get(&self.pcb.to_id(v))
                    .ok_or_else(|| eyre!("unknown padstack id {}", v))?
                    .clone(),
            );
        }
        for v in &self.dsn.placement.components {
            for component in self.components(v)? {
                self.pcb.add_component(component);
            }
        }

        // Routing:
        for v in &self.dsn.network.nets {
            self.pcb.add_net(self.net(v));
        }
        for v in &self.dsn.network.classes {
            let ruleset = self.ruleset(v)?;
            self.pcb.add_ruleset(ruleset.clone());
            // Check for default ruleset:
            if v.net_ids.is_empty() {
                self.pcb.set_default_net_ruleset(ruleset.id);
            } else {
                for net in &v.net_ids {
                    self.pcb.set_net_ruleset(self.pcb.to_id(net), ruleset.id);
                }
            }
        }

        // TODO: Add wires
        // TODO: Add vias
        // TODO: Support classes for nets.
        // TODO: Support rules from structure.
        Ok(self.pcb)
    }
}
