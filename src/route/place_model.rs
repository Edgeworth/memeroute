use std::collections::HashMap;

use eyre::Result;
use memegeom::geom::math::le;
use memegeom::geom::qt::quadtree::ShapeIdx;
use memegeom::geom::qt::query::{Kinds, KindsQuery, Query, ShapeInfo, Tag, TagQuery, NO_TAG};
use memegeom::primitive::compound::Compound;
use memegeom::primitive::point::Pt;
use memegeom::primitive::rect::Rt;
use memegeom::primitive::{path, ShapeOps};
use memegeom::tf::Tf;

use crate::model::pcb::{
    Clearance, LayerId, LayerSet, LayerShape, Net, ObjectKind, Padstack, Pcb, Pin, PinRef, Via,
    Wire,
};
use crate::name::Id;

pub type PlaceId = (LayerId, ShapeIdx);

// Need to handle:
// but also keeping them for hole drils
#[must_use]
#[derive(Debug, Default, Clone)]
pub struct PlaceModel {
    pcb: Pcb,
    // TODO: Can move layerids to quadtree?
    boundary: HashMap<LayerId, Compound>,
    blocked: HashMap<LayerId, Compound>,
    pins: HashMap<PinRef, Vec<PlaceId>>, // Record which pins correspond to which place ids in |blocked|.
    bounds: Rt,
}

impl PlaceModel {
    pub fn new(pcb: Pcb) -> Self {
        let mut m = Self {
            pcb: Pcb::default(), // Initially set as empty since we will initialise.
            boundary: HashMap::new(),
            blocked: HashMap::new(),
            pins: HashMap::new(),
            bounds: Rt::empty(),
        };
        m.init(pcb);
        m
    }

    pub fn debug_rts(&self) -> Vec<Rt> {
        // 0 = F.Cu, 1 = B.Cu
        self.blocked.get(&1).unwrap().quadtree().rts()
    }

    pub fn pcb(&self) -> &Pcb {
        &self.pcb
    }

    // Creates a wire for a given net, but doesn't add it.
    pub fn create_wire(&self, net_id: Id, layer: LayerId, pts: &[Pt]) -> Wire {
        let rs = self.pcb.net_ruleset(net_id);
        let shape =
            LayerShape { layers: LayerSet::one(layer), shape: path(pts, rs.radius()).shape() };
        Wire { shape, net_id }
    }

    pub fn add_wire(&mut self, wire: &Wire) -> Vec<PlaceId> {
        Self::add_shape(
            self.bounds,
            &mut self.blocked,
            &Tf::identity(),
            &wire.shape,
            Tag(wire.net_id),
            ObjectKind::Wire.query(),
        )
    }

    // Creates a via for a given net, but doesn't add it.
    pub fn create_via(&self, net_id: Id, p: Pt) -> Via {
        // TODO: consult ruleset to choose via.
        Via { padstack: self.pcb.via_padstacks()[0].clone(), p, net_id }
    }

    pub fn add_via(&mut self, via: &Via) -> Vec<PlaceId> {
        self.add_padstack(&via.tf(), &via.padstack, Tag(via.net_id), ObjectKind::Via.query())
    }

    // Adds all pins in the given net.
    pub fn add_net(&mut self, pcb: &Pcb, net: &Net) -> Result<()> {
        for p in &net.pins {
            let (component, pin) = pcb.pin_ref(p)?;
            self.add_pin(&component.tf(), p.clone(), pin, Tag(net.id));
        }
        Ok(())
    }

    // Removes all pins in the given net.
    pub fn remove_net(&mut self, net: &Net) {
        for p in &net.pins {
            self.remove_pin(p);
        }
    }

    pub fn is_wire_blocked(&self, wire: &Wire) -> bool {
        self.is_shape_blocked(
            &Tf::identity(),
            &wire.shape,
            TagQuery::Except(Tag(wire.net_id)),
            ObjectKind::Wire,
            self.pcb.net_ruleset(wire.net_id).clearances(),
        )
    }

    pub fn is_via_blocked(&self, via: &Via) -> bool {
        self.is_padstack_blocked(
            &via.tf(),
            &via.padstack,
            TagQuery::All,
            ObjectKind::Via,
            self.pcb.net_ruleset(via.net_id).clearances(),
        )
    }

    pub fn is_shape_blocked(
        &self,
        tf: &Tf,
        ls: &LayerShape,
        q: TagQuery,
        kind: ObjectKind,
        clearances: &[Clearance],
    ) -> bool {
        let s = tf.shape(&ls.shape);

        for layer in ls.layers.iter() {
            if let Some(boundary) = self.boundary.get(&layer) {
                // TODO: Convert boundary to path and compute distance to it for clearance.
                if !boundary.contains(&s, Query(q, KindsQuery::All)) {
                    return true;
                }
            }
        }

        // Check for intersection first, it's generally cheaper than checking distance.
        for layer in ls.layers.iter() {
            if let Some(blocked) = self.blocked.get(&layer) {
                if blocked.intersects(&s, Query(q, KindsQuery::All)) {
                    return true;
                }
            }
        }

        // Check for clearance.
        for layer in ls.layers.iter() {
            if let Some(blocked) = self.blocked.get(&layer) {
                for c in clearances {
                    let d = blocked.dist(&s, Query(q, KindsQuery::HasCommon(c.subset_for(kind))));
                    if le(d, c.amount()) {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn init(&mut self, pcb: Pcb) {
        let tf = Tf::identity();

        self.bounds = self.bounds.united(&pcb.bounds());
        for boundary in pcb.boundaries() {
            Self::add_shape(
                self.bounds,
                &mut self.boundary,
                &tf,
                boundary,
                NO_TAG,
                ObjectKind::Area.query(),
            );
        }

        for wire in pcb.wires() {
            self.add_wire(wire);
        }
        for via in pcb.vias() {
            self.add_via(via);
        }
        for keepout in pcb.keepouts() {
            Self::add_shape(
                self.bounds,
                &mut self.blocked,
                &tf,
                &keepout.shape,
                NO_TAG,
                ObjectKind::Area.query(),
            );
        }

        for c in pcb.components() {
            let tf = tf * c.tf();
            for pin in c.pins() {
                let r = PinRef::new(c, pin);
                let tag = if let Some(tag) = pcb.pin_ref_net(&r) { Tag(tag) } else { NO_TAG };
                self.add_pin(&tf, r, pin, tag);
            }
            for keepout in &c.keepouts {
                Self::add_shape(
                    self.bounds,
                    &mut self.blocked,
                    &tf,
                    &keepout.shape,
                    NO_TAG,
                    ObjectKind::Area.query(),
                );
            }
        }
        self.pcb = pcb;
    }

    fn add_shape(
        bounds: Rt,
        map: &mut HashMap<LayerId, Compound>,
        tf: &Tf,
        ls: &LayerShape,
        tag: Tag,
        kinds: Kinds,
    ) -> Vec<PlaceId> {
        let s = tf.shape(&ls.shape);
        let mut idxs = Vec::new();

        for layer in ls.layers.iter() {
            idxs.extend(
                map.entry(layer)
                    .or_insert_with(|| Compound::with_bounds(&bounds))
                    .add_shape(ShapeInfo::new(s.clone(), tag, kinds))
                    .iter()
                    .map(|&v| (layer, v)),
            );
        }

        idxs
    }

    fn add_padstack(
        &mut self,
        tf: &Tf,
        padstack: &Padstack,
        tag: Tag,
        kinds: Kinds,
    ) -> Vec<PlaceId> {
        padstack
            .shapes
            .iter()
            .flat_map(|shape| {
                Self::add_shape(self.bounds, &mut self.blocked, tf, shape, tag, kinds)
            })
            .collect()
    }

    fn add_pin(&mut self, tf: &Tf, pinref: PinRef, pin: &Pin, tag: Tag) -> Vec<PlaceId> {
        let ids = self.add_padstack(&(tf * pin.tf()), &pin.padstack, tag, ObjectKind::Pin.query());
        let e = self.pins.entry(pinref).or_insert_with(Vec::new);
        for &id in &ids {
            e.push(id);
        }
        ids
    }

    fn remove_pin(&mut self, p: &PinRef) {
        if let Some(ids) = self.pins.remove(p) {
            for id in ids {
                self.remove_shape(id);
            }
        }
    }

    fn remove_shape(&mut self, id: PlaceId) {
        self.blocked.get_mut(&id.0).unwrap().remove_shape(id.1);
    }

    fn is_padstack_blocked(
        &self,
        tf: &Tf,
        padstack: &Padstack,
        q: TagQuery,
        kind: ObjectKind,
        clearances: &[Clearance],
    ) -> bool {
        padstack.shapes.iter().any(|shape| self.is_shape_blocked(tf, shape, q, kind, clearances))
    }
}
