use std::str::FromStr;

use eyre::{eyre, Result};
use rust_decimal::Decimal;

use crate::dsn::token::{Tok, Token};
use crate::dsn::types::{
    DsnCircle, DsnCircuit, DsnClass, DsnClearance, DsnClearanceType, DsnComponent,
    DsnDimensionUnit, DsnImage, DsnKeepout, DsnKeepoutType, DsnLayer, DsnLayerType, DsnLibrary,
    DsnLockType, DsnNet, DsnNetwork, DsnPadstack, DsnPadstackShape, DsnPath, DsnPcb, DsnPin,
    DsnPinRef, DsnPlacement, DsnPlacementRef, DsnPlane, DsnPolygon, DsnQArc, DsnRect,
    DsnResolution, DsnRule, DsnShape, DsnSide, DsnStructure, DsnVia, DsnWindow, DsnWire, DsnWiring,
};
use crate::model::geom::{Pt, Rt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parser {
    toks: Vec<Token>,
    idx: usize,
    pcb: DsnPcb,
}

impl Parser {
    pub fn new(toks: &[Token]) -> Self {
        Self { toks: toks.to_vec(), idx: 0, pcb: Default::default() }
    }

    pub fn parse(mut self) -> Result<DsnPcb> {
        self.pcb()?;
        Ok(self.pcb)
    }

    fn peek(&mut self, ahead: usize) -> Result<Token> {
        if self.idx + ahead < self.toks.len() {
            Ok(self.toks[self.idx + ahead].clone())
        } else {
            Err(eyre!("unexpected EOF"))
        }
    }

    fn next(&mut self) -> Result<Token> {
        if self.idx < self.toks.len() {
            self.idx += 1;
            Ok(self.toks[self.idx - 1].clone())
        } else {
            Err(eyre!("unexpected EOF"))
        }
    }

    fn expect(&mut self, t: Tok) -> Result<Token> {
        match self.next()? {
            x if x.tok == t => Ok(x),
            x => Err(eyre!("unexpected token {}", x)),
        }
    }

    fn literal(&mut self) -> Result<String> {
        Ok(self.next()?.s)
    }

    fn ignore(&mut self) -> Result<()> {
        let inside_expr = self.peek(0)?.tok != Tok::Lparen;
        loop {
            let t = self.next()?;
            if t.tok == Tok::Rparen {
                break;
            }
            if t.tok == Tok::Lparen {
                self.ignore()?;
                // Handle the case of being called at the start of an expression.
                if !inside_expr {
                    break;
                }
            }
        }
        Ok(())
    }

    fn pcb(&mut self) -> Result<()> {
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Pcb)?;
        self.pcb.pcb_id = self.literal()?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::Library => self.pcb.library = self.library()?,
                Tok::Network => self.pcb.network = self.network()?,
                Tok::Parser => self.ignore()?, // Handled during lexing.
                Tok::Placement => self.pcb.placement = self.placement()?,
                Tok::Resolution => self.pcb.resolution = self.resolution()?,
                Tok::Structure => self.pcb.structure = self.structure()?,
                Tok::Unit => self.ignore()?, // Ignore for now.
                Tok::Wiring => self.pcb.wiring = self.wiring()?,
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(())
    }

    fn library(&mut self) -> Result<DsnLibrary> {
        let mut v = DsnLibrary::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Library)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::Image => v.images.push(self.image()?),
                Tok::Padstack => v.padstacks.push(self.padstack()?),
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn network(&mut self) -> Result<DsnNetwork> {
        let mut v = DsnNetwork::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Network)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::Class => v.classes.push(self.class()?),
                Tok::Net => v.nets.push(self.net()?),
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn placement(&mut self) -> Result<DsnPlacement> {
        let mut v = DsnPlacement::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Placement)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::Component => v.components.push(self.component()?),
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn resolution(&mut self) -> Result<DsnResolution> {
        let mut v = DsnResolution::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Resolution)?;
        v.dimension = match self.next()?.tok {
            Tok::Inch => DsnDimensionUnit::Inch,
            Tok::Mil => DsnDimensionUnit::Mil,
            Tok::Cm => DsnDimensionUnit::Cm,
            Tok::Mm => DsnDimensionUnit::Mm,
            Tok::Um => DsnDimensionUnit::Um,
            _ => return Err(eyre!("unknown dimension unit")),
        };
        v.amount = self.integer()?;
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn structure(&mut self) -> Result<DsnStructure> {
        let mut v = DsnStructure::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Structure)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::Boundary => {
                    self.expect(Tok::Lparen)?;
                    self.expect(Tok::Boundary)?;
                    v.boundaries.push(self.shape()?);
                    self.expect(Tok::Rparen)?;
                }
                Tok::Keepout | Tok::ViaKeepout | Tok::WireKeepout => {
                    v.keepouts.push(self.keepout()?)
                }
                Tok::Layer => v.layers.push(self.layer()?),
                Tok::Plane => v.planes.push(self.plane()?),
                Tok::Rule => v.rules.extend(self.rule()?),
                Tok::Via => {
                    self.expect(Tok::Lparen)?;
                    self.expect(Tok::Via)?;
                    while self.peek(0)?.tok != Tok::Rparen {
                        v.vias.push(self.literal()?);
                    }
                    self.expect(Tok::Rparen)?;
                }
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn wiring(&mut self) -> Result<DsnWiring> {
        let mut v = DsnWiring::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Wiring)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn via(&mut self) -> Result<DsnVia> {
        let mut v = DsnVia::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Via)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn wire(&mut self) -> Result<DsnWire> {
        let mut v = DsnWire::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Wire)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn layer(&mut self) -> Result<DsnLayer> {
        let mut v = DsnLayer::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Layer)?;
        v.layer_name = self.literal()?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::Type => {
                    self.expect(Tok::Lparen)?;
                    self.expect(Tok::Type)?;
                    match self.next()?.tok {
                        Tok::Jumper => v.layer_type = DsnLayerType::Jumper,
                        Tok::Mixed => v.layer_type = DsnLayerType::Mixed,
                        Tok::Power => v.layer_type = DsnLayerType::Power,
                        Tok::Signal => v.layer_type = DsnLayerType::Signal,
                        _ => return Err(eyre!("unrecognised layer type")),
                    }
                    self.expect(Tok::Rparen)?;
                }
                Tok::Property => self.ignore()?, // Ignore user properties.
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn plane(&mut self) -> Result<DsnPlane> {
        let mut v = DsnPlane::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Plane)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn component(&mut self) -> Result<DsnComponent> {
        let mut v = DsnComponent::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Component)?;
        v.image_id = self.literal()?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::Place => v.refs.push(self.placement_ref()?),
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn placement_ref(&mut self) -> Result<DsnPlacementRef> {
        let mut v = DsnPlacementRef::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Place)?;
        v.component_id = self.literal()?;
        v.p = self.vertex()?; // Assume we have vertex information.
        v.side = self.side()?;
        v.rotation = self.number()?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::LockType => {
                    self.expect(Tok::Lparen)?;
                    self.expect(Tok::LockType)?;
                    match self.next()?.tok {
                        Tok::Gate => v.lock_type = DsnLockType::Gate,
                        Tok::Position => v.lock_type = DsnLockType::Position,
                        _ => return Err(eyre!("unrecognised layer type")),
                    }
                    self.expect(Tok::Rparen)?;
                }
                Tok::Pn => {
                    self.expect(Tok::Lparen)?;
                    self.expect(Tok::Pn)?;
                    v.part_number = self.literal()?;
                    self.expect(Tok::Rparen)?;
                }
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn image(&mut self) -> Result<DsnImage> {
        let mut v = DsnImage::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Image)?;
        v.image_id = self.literal()?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::Outline => {
                    self.expect(Tok::Lparen)?;
                    self.expect(Tok::Outline)?;
                    v.outlines.push(self.shape()?);
                    self.expect(Tok::Rparen)?;
                }
                Tok::Pin => v.pins.push(self.pin()?),
                Tok::Keepout | Tok::ViaKeepout | Tok::WireKeepout => {
                    v.keepouts.push(self.keepout()?)
                }
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn keepout(&mut self) -> Result<DsnKeepout> {
        let mut v = DsnKeepout::default();
        self.expect(Tok::Lparen)?;
        v.keepout_type = match self.next()?.tok {
            Tok::Keepout => DsnKeepoutType::Keepout,
            Tok::ViaKeepout => DsnKeepoutType::ViaKeepout,
            Tok::WireKeepout => DsnKeepoutType::WireKeepout,
            _ => return Err(eyre!("unrecognised keepout type")),
        };
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::Rect | Tok::Circle | Tok::Polygon | Tok::Path | Tok::Qarc => {
                    v.shape = self.shape()?
                }
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn pin(&mut self) -> Result<DsnPin> {
        let mut v = DsnPin::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Pin)?;
        v.padstack_id = self.literal()?;
        if self.peek(0)?.tok == Tok::Lparen {
            // Rotation.
            self.expect(Tok::Lparen)?;
            self.expect(Tok::Rotate)?;
            v.rotation = self.number()?;
            self.expect(Tok::Rparen)?;
        }
        v.pin_id = self.literal()?;
        v.p = self.vertex()?;
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn padstack(&mut self) -> Result<DsnPadstack> {
        let mut v = DsnPadstack::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Padstack)?;
        v.padstack_id = self.literal()?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::Attach => {
                    self.expect(Tok::Lparen)?;
                    self.expect(Tok::Attach)?;
                    v.attach = self.onoff()?;
                    self.expect(Tok::Rparen)?;
                }
                Tok::Shape => v.shapes.push(self.padstack_shape()?),
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn padstack_shape(&mut self) -> Result<DsnPadstackShape> {
        let mut v = DsnPadstackShape::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Shape)?;
        v.shape = self.shape()?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::Window => v.windows.push(self.window()?),
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn clearance(&mut self) -> Result<DsnClearance> {
        let mut v = DsnClearance::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Clearance)?;
        v.amount = self.number()?;

        while self.peek(0)?.tok != Tok::Rparen {
            self.expect(Tok::Lparen)?;
            self.expect(Tok::Type)?;
            v.types.push(match self.next()?.tok {
                Tok::DefaultSmd => DsnClearanceType::DefaultSmd,
                Tok::SmdSmd => DsnClearanceType::SmdSmd,
                _ => return Err(eyre!("unrecognised clearance type")),
            });
            self.expect(Tok::Rparen)?;
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn window(&mut self) -> Result<DsnWindow> {
        let mut v = DsnWindow::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Window)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn shape(&mut self) -> Result<DsnShape> {
        match self.peek(1)?.tok {
            Tok::Circle => Ok(DsnShape::Circle(self.circle()?)),
            Tok::Path => Ok(DsnShape::Path(self.path()?)),
            Tok::Polygon => Ok(DsnShape::Polygon(self.polygon()?)),
            Tok::Qarc => Ok(DsnShape::QArc(self.qarc()?)),
            Tok::Rect => Ok(DsnShape::Rect(self.rect()?)),
            _ => Err(eyre!("unrecognised shape type")),
        }
    }

    fn rect(&mut self) -> Result<DsnRect> {
        let mut v = DsnRect::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Rect)?;
        v.layer_id = self.literal()?;
        let a = self.vertex()?;
        let b = self.vertex()?;
        v.rect = Rt::enclosing(&a, &b); // Opposite points but can be in either order.
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn circle(&mut self) -> Result<DsnCircle> {
        let mut v = DsnCircle::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Circle)?;
        v.layer_id = self.literal()?;
        v.diameter = self.number()?;
        if self.peek(0)?.tok != Tok::Rparen {
            v.p = self.vertex()?;
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn polygon(&mut self) -> Result<DsnPolygon> {
        let mut v = DsnPolygon::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Polygon)?;
        v.layer_id = self.literal()?;
        v.aperture_width = self.number()?;
        while self.peek(0)?.tok != Tok::Rparen {
            v.pts.push(self.vertex()?);
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn path(&mut self) -> Result<DsnPath> {
        let mut v = DsnPath::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Path)?;
        v.layer_id = self.literal()?;
        v.aperture_width = self.number()?;
        while self.peek(0)?.tok != Tok::Rparen {
            v.pts.push(self.vertex()?);
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn qarc(&mut self) -> Result<DsnQArc> {
        let mut v = DsnQArc::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Qarc)?;
        v.layer_id = self.literal()?;
        v.aperture_width = self.number()?;
        v.start = self.vertex()?;
        v.end = self.vertex()?;
        v.center = self.vertex()?;
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn class(&mut self) -> Result<DsnClass> {
        let mut v = DsnClass::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Class)?;
        v.class_id = self.literal()?;
        while self.peek(0)?.tok != Tok::Rparen {
            let pt = self.peek(0)?;
            if pt.tok == Tok::Lparen {
                let t = self.peek(1)?;
                match t.tok {
                    Tok::Circuit => v.circuits.push(self.circuit()?),
                    Tok::Rule => v.rules.extend(self.rule()?),
                    _ => return Err(eyre!("unrecognised token '{}'", t)),
                }
            } else {
                v.net_ids.push(self.literal()?);
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn circuit(&mut self) -> Result<DsnCircuit> {
        let mut v = DsnCircuit::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Circuit)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::UseVia => {
                    self.expect(Tok::Lparen)?;
                    self.expect(Tok::UseVia)?;
                    v.use_via = self.literal()?;
                    self.expect(Tok::Rparen)?;
                }
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn net(&mut self) -> Result<DsnNet> {
        let mut v = DsnNet::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Net)?;
        v.net_id = self.literal()?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::Pins => {
                    self.expect(Tok::Lparen)?;
                    self.expect(Tok::Pins)?;
                    while self.peek(0)?.tok != Tok::Rparen {
                        v.pins.push(self.pin_ref()?);
                    }
                    self.expect(Tok::Rparen)?;
                }
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn rule(&mut self) -> Result<Vec<DsnRule>> {
        let mut v = Vec::new();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Rule)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::Width => {
                    self.expect(Tok::Lparen)?;
                    self.expect(Tok::Width)?;
                    let width = self.number()?;
                    v.push(DsnRule::Width(width));
                    self.expect(Tok::Rparen)?;
                }
                Tok::Clearance => v.push(DsnRule::Clearance(self.clearance()?)),
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn vertex(&mut self) -> Result<Pt> {
        Ok(Pt::new(self.number()?, self.number()?))
    }

    fn unit(&mut self) -> Result<DsnDimensionUnit> {
        let mut v = DsnDimensionUnit::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Unit)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn pin_ref(&mut self) -> Result<DsnPinRef> {
        let p = self.literal()?;
        let (a, b) = p.split_once('-').ok_or_else(|| eyre!("invalid pin reference {}", p))?;
        Ok(DsnPinRef { component_id: a.to_owned(), pin_id: b.to_owned() })
    }

    fn onoff(&mut self) -> Result<bool> {
        match self.next()?.tok {
            Tok::Off => Ok(false),
            Tok::On => Ok(true),
            _ => Err(eyre!("expected off or not")),
        }
    }

    fn side(&mut self) -> Result<DsnSide> {
        match self.next()?.tok {
            Tok::Back => Ok(DsnSide::Back),
            Tok::Both => Ok(DsnSide::Both),
            Tok::Front => Ok(DsnSide::Front),
            _ => Err(eyre!("unrecognised side type")),
        }
    }

    fn number(&mut self) -> Result<Decimal> {
        // TODO: Handle fractions.
        Ok(Decimal::from_str(&self.literal()?)?)
    }

    fn integer(&mut self) -> Result<i32> {
        // TODO: Handle fractions.
        Ok(i32::from_str(&self.literal()?)?)
    }
}
