use std::str::FromStr;

use eyre::{eyre, Result};
use rust_decimal::Decimal;

use crate::dsn::token::{Tok, Token};
use crate::dsn::types::{
    Circle, Circuit, Class, Clearance, Component, DimensionUnit, Image, Keepout, KeepoutType,
    Layer, LayerType, Library, LockType, Net, Network, Padstack, PadstackShape, Path, Pcb, Pin,
    PinRef, Placement, PlacementRef, Plane, Polygon, QArc, Rect, Resolution, Rule, Shape, Side,
    Structure, Via, Window, Wire, Wiring,
};
use crate::model::geom::{PtF, RtF};

pub struct Parser {
    toks: Vec<Token>,
    idx: usize,
    pcb: Pcb,
}

impl Parser {
    pub fn new(toks: &[Token]) -> Self {
        Self { toks: toks.to_vec(), idx: 0, pcb: Default::default() }
    }

    pub fn parse(mut self) -> Result<Pcb> {
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

    fn library(&mut self) -> Result<Library> {
        let mut v = Library::default();
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

    fn network(&mut self) -> Result<Network> {
        let mut v = Network::default();
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

    fn placement(&mut self) -> Result<Placement> {
        let mut v = Placement::default();
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

    fn resolution(&mut self) -> Result<Resolution> {
        let mut v = Resolution::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Resolution)?;
        v.dimension = match self.next()?.tok {
            Tok::Inch => DimensionUnit::Inch,
            Tok::Mil => DimensionUnit::Mil,
            Tok::Cm => DimensionUnit::Cm,
            Tok::Mm => DimensionUnit::Mm,
            Tok::Um => DimensionUnit::Um,
            _ => return Err(eyre!("unknown dimension unit")),
        };
        v.amount = self.integer()?;
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn structure(&mut self) -> Result<Structure> {
        let mut v = Structure::default();
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
                Tok::Rule => v.rules.push(self.rule()?),
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

    fn wiring(&mut self) -> Result<Wiring> {
        let mut v = Wiring::default();
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

    fn via(&mut self) -> Result<Via> {
        let mut v = Via::default();
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

    fn wire(&mut self) -> Result<Wire> {
        let mut v = Wire::default();
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

    fn layer(&mut self) -> Result<Layer> {
        let mut v = Layer::default();
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
                        Tok::Jumper => v.layer_type = LayerType::Jumper,
                        Tok::Mixed => v.layer_type = LayerType::Mixed,
                        Tok::Power => v.layer_type = LayerType::Power,
                        Tok::Signal => v.layer_type = LayerType::Signal,
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

    fn plane(&mut self) -> Result<Plane> {
        let mut v = Plane::default();
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

    fn component(&mut self) -> Result<Component> {
        let mut v = Component::default();
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

    fn placement_ref(&mut self) -> Result<PlacementRef> {
        let mut v = PlacementRef::default();
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
                        Tok::Gate => v.lock_type = LockType::Gate,
                        Tok::Position => v.lock_type = LockType::Position,
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

    fn image(&mut self) -> Result<Image> {
        let mut v = Image::default();
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

    fn keepout(&mut self) -> Result<Keepout> {
        let mut v = Keepout::default();
        self.expect(Tok::Lparen)?;
        v.keepout_type = match self.next()?.tok {
            Tok::Keepout => KeepoutType::Keepout,
            Tok::ViaKeepout => KeepoutType::ViaKeepout,
            Tok::WireKeepout => KeepoutType::WireKeepout,
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

    fn pin(&mut self) -> Result<Pin> {
        let mut v = Pin::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Keepout)?;
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

    fn padstack(&mut self) -> Result<Padstack> {
        let mut v = Padstack::default();
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

    fn padstack_shape(&mut self) -> Result<PadstackShape> {
        let mut v = PadstackShape::default();
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

    fn clearance(&mut self) -> Result<Clearance> {
        let mut v = Clearance::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Clearance)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn window(&mut self) -> Result<Window> {
        let mut v = Window::default();
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

    fn shape(&mut self) -> Result<Shape> {
        match self.peek(1)?.tok {
            Tok::Circle => Ok(Shape::Circle(self.circle()?)),
            Tok::Path => Ok(Shape::Path(self.path()?)),
            Tok::Polygon => Ok(Shape::Polygon(self.polygon()?)),
            Tok::Qarc => Ok(Shape::QArc(self.qarc()?)),
            Tok::Rect => Ok(Shape::Rect(self.rect()?)),
            _ => Err(eyre!("unrecognised shape type")),
        }
    }

    fn rect(&mut self) -> Result<Rect> {
        let mut v = Rect::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Rect)?;
        v.layer_id = self.literal()?;
        let a = self.vertex()?;
        let b = self.vertex()?;
        v.rect = RtF::enclosing(&a, &b); // Opposite points but can be in either order.
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn circle(&mut self) -> Result<Circle> {
        let mut v = Circle::default();
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

    fn polygon(&mut self) -> Result<Polygon> {
        let mut v = Polygon::default();
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

    fn path(&mut self) -> Result<Path> {
        let mut v = Path::default();
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

    fn qarc(&mut self) -> Result<QArc> {
        let mut v = QArc::default();
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

    fn class(&mut self) -> Result<Class> {
        let mut v = Class::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Class)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn circuit(&mut self) -> Result<Circuit> {
        let mut v = Circuit::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Circuit)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn net(&mut self) -> Result<Net> {
        let mut v = Net::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Net)?;
        v.net_id = self.literal()?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                Tok::Pins => {
                    self.expect(Tok::Lparen)?;
                    self.expect(Tok::Net)?;
                    while self.peek(0)?.tok != Tok::Rparen {
                        v.pins.push(self.pin_ref()?);
                    }
                    self.expect(Tok::Rparen)?;
                    break;
                }
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn rule(&mut self) -> Result<Rule> {
        let mut v = Rule::default();
        self.expect(Tok::Lparen)?;
        self.expect(Tok::Rule)?;
        while self.peek(0)?.tok != Tok::Rparen {
            let t = self.peek(1)?;
            match t.tok {
                _ => return Err(eyre!("unrecognised token '{}'", t)),
            }
        }
        self.expect(Tok::Rparen)?;
        Ok(v)
    }

    fn vertex(&mut self) -> Result<PtF> {
        Ok(PtF::new(self.number()?, self.number()?))
    }

    fn unit(&mut self) -> Result<DimensionUnit> {
        let mut v = DimensionUnit::default();
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

    fn pin_ref(&mut self) -> Result<PinRef> {
        let p = self.literal()?;
        let (a, b) = p.split_once('-').ok_or_else(|| eyre!("invalid pin reference {}", p))?;
        Ok(PinRef { component_id: a.to_owned(), pin_id: b.to_owned() })
    }

    fn onoff(&mut self) -> Result<bool> {
        match self.next()?.tok {
            Tok::Off => Ok(false),
            Tok::On => Ok(true),
            _ => Err(eyre!("expected off or not")),
        }
    }

    fn side(&mut self) -> Result<Side> {
        match self.next()?.tok {
            Tok::Back => Ok(Side::Back),
            Tok::Both => Ok(Side::Both),
            Tok::Front => Ok(Side::Front),
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
