use rust_decimal::Decimal;
use strum::{Display as EnumDisplay, EnumString};

use crate::model::geom::{PtF, RtF};

// Types defined in DSN specification.

// <number> = [<sign>] (<positive_integer> | <real> | <fraction>)
// <dimension> = <number>
// <vertex> = PtF

// <unit_descriptor> = (unit <dimension_unit>)
// <dimension_unit> = [inch | mil | cm | mm | um]
#[derive(Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum DimensionUnit {
    Inch,
    Mil,
    Cm,
    Mm,
    Um,
}

impl Default for DimensionUnit {
    fn default() -> Self {
        Self::Inch
    }
}

// <id>, <component_id>, <pin_id>, <padstack_id>, <via_id>, <image_id>,
// <net_id>, <pcb_id>, <class_id> = std::string
pub type Id = String;

// <pin_reference> = <component_id>-<pin_id>
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PinRef {
    pub component_id: Id,
    pub pin_id: Id,
}

// <layer_id> = <id> | pcb | signal | power
pub type LayerId = Id;

// <rectangle_descriptor> = (rect <layer_id> <vertex> <vertex>)
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Rect {
    pub layer_id: LayerId,
    pub rect: RtF,
}

// <circle_descriptor> = (circle <layer_id> <diameter> [<vertex>])
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Circle {
    pub layer_id: LayerId,
    pub diameter: Decimal,
    pub p: PtF, // Defaults to PCB origin.
}

// <polygon_descriptor> = (polygon <layer_id> <aperture_width> {<vertex>}
//    [(aperture_type [round | square])])
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Polygon {
    pub layer_id: LayerId,
    pub aperture_width: Decimal,
    pub pts: Vec<PtF>,
}

// <path_descriptor> = (path <layer_id> <aperture_width> {<vertex>}
//    [(aperture_type [round | square])])
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Path {
    pub layer_id: LayerId,
    pub aperture_width: Decimal,
    pub pts: Vec<PtF>,
}

// <qarc_descriptor> = (qarc <layer_id> <aperture_width>
//    <vertex> <vertex> <vertex>)
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct QArc {
    pub layer_id: LayerId,
    pub aperture_width: Decimal,
    pub start: PtF,
    pub end: PtF,
    pub center: PtF,
}

// <shape_descriptor> = = [<rectangle_descriptor> | <circle_descriptor> |
//    <polygon_descriptor> | <path_descriptor> | <qarc_descriptor>]
#[derive(Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum Shape {
    Rect(Rect),
    Circle(Circle),
    Polygon(Polygon),
    Path(Path),
    QArc(QArc),
}

impl Default for Shape {
    fn default() -> Self {
        Self::Rect(Rect::default())
    }
}

// <window_descriptor> = (window <shape_descriptor>) - can only be rect or
// polygon.
#[derive(Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum Window {
    Rect(Rect),
    Polygon(Polygon),
}

impl Default for Window {
    fn default() -> Self {
        Self::Rect(Rect::default())
    }
}

// <object_type> = [pin | smd | via | wire | area | testpoint]
#[derive(Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum ObjectType {
    Pin,
    Smd,
    Via,
    Wire,
    Area,
    #[strum(serialize = "testpoint")]
    TestPoint,
}

// <clearance_type> = [<object_type>_<object_type> | smd_via_same_net |
//    via_via_same_net | buried_via_gap [(layer_depth <positive_integer>)] |
//    antipad_gap | pad_to_turn_gap | smd_to_turn_gap]

// <clearance_descriptor> = (clearance <positive_dimension> [(type {<clearance_type>})]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Clearance {}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PadstackShape {
    pub shape: Shape,
    pub windows: Vec<Window>, // Subtracts from the LocalShape
}

// <attach_descriptor> = (attach [off | on [(use_via <via_id>)]])
// <reduced_shape_descriptor> = (reduced <shape_descriptor>)
// <padstack_descriptor> = (padstack <padstack_id> [<unit_descriptor>]
//    {(shape <shape_descriptor> [<reduced_shape_descriptor>]
//        [(connect [on | off])] [{<window_descriptor>}])}
//    [<attach_descriptor>] [{<pad_via_site_descriptor>}] [(rotate [on | off])]
//    [(absolute [on | off])] [(rule <clearance_descriptor>)])
// A padstack describes an exposed area for connecting components to. Pins
// connect onto padstacks. There are multiple PadstackShapes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Padstack {
    pub padstack_id: Id,
    pub shapes: Vec<PadstackShape>,
    pub attach: bool, // Default is to allow vias under SMD pads.
}

impl Default for Padstack {
    fn default() -> Self {
        Self { padstack_id: Default::default(), shapes: Default::default(), attach: true }
    }
}

// Describes a side of the PCB.
#[derive(Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum Side {
    Front,
    Back,
    Both,
}

impl Default for Side {
    fn default() -> Self {
        Self::Front
    }
}

// {(pin <padstack_id> [(rotate <rotation>)]
//    [<reference_descriptor> | <pin_array_descriptor>]
//    [<user_property_descriptor>])}
// Describes a pin within an image (type of component). Pins connect onto
// padstacks.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Pin {
    pub padstack_id: Id,   // Padstack describes the shape of the pin
    pub rotation: Decimal, // Rotation in degrees. Default to 0
    pub pin_id: Id,        // Describes TODO e.g. 1@1
    pub p: PtF,            // Location of the pin relative to the parent component (placement).
}

// Keepout: No routing whatsoever.
// ViaKeepout: No vias.
// WireKeepout: No wires.
#[derive(Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum KeepoutType {
    Keepout,
    ViaKeepout,
    WireKeepout,
}

impl Default for KeepoutType {
    fn default() -> Self {
        Self::Keepout
    }
}

// <keepout_descriptor> = (
//    [keepout | place_keepout | via_keepout | wire_keepout | bend_keepout | elongate_keepout]
//    [<id>] [(sequence_number <keepout_sequence_number>)]
//    <shape_descriptor>
//    [(rule <clearance_descriptor>)]
//    [(place_rule <spacing_descriptor>)]
//    [{<window_descriptor>}])
// Describes an area where no routing can occur.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Keepout {
    pub keepout_type: KeepoutType,
    pub shape: Shape,
}

// <image_descriptor> = (image <image_id>
//    [(side [front | back | both])]
//    [<unit_descriptor>]
//    [<outline_descriptor>]
//    {(pin <padstack_id> [(rotate <rotation>)]
//        [<reference_descriptor> | <pin_array_descriptor>] [<user_property_descriptor>])}
//    [{<conductor_shape_descriptor>}]
//    [{<conductor_via_descriptor>}]
//    [<rule_descriptor>]
//    [<place_rule_descriptor>]
//    [{<keepout_descriptor>}]
//    [<image_property_descriptor>])
// Describes a component type.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Image {
    pub image_id: Id,
    pub outlines: Vec<Shape>,
    pub pins: Vec<Pin>,
    pub keepouts: Vec<Keepout>,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum LockType {
    None,
    Position,
    Gate,
}

impl Default for LockType {
    fn default() -> Self {
        Self::None
    }
}

// <placement_reference> = (place <component_id>
//    [<vertex> <side> <rotation>]
//    [<mirror_descriptor>]
//    [<component_status_descriptor>]
//    [(logical_part <logical_part_id>]
//    [<place_rule_descriptor>]
//    [<component_property_descriptor>]
//    [(lock_type {[position | gate | subgate | pin]})]
//    [<rule_descriptor>> | <region_descriptor> | null]
//    [(PN <part_number>)])
// Describes the location of a component.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PlacementRef {
    pub component_id: Id,
    pub p: PtF,
    pub side: Side,
    pub rotation: Decimal,
    pub lock_type: LockType,
    pub part_number: Id,
}

// <component_instance> = (component <image_id> {<placement_reference>})
// Describes instances of a component. The component type is determined
// by the image referred to by |image_id|. There is a component at each
// location specified by each placement reference.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Component {
    pub image_id: Id,
    pub refs: Vec<PlacementRef>,
}

// <net_descriptor> = (net <net_id>
//    [(unassigned)]
//    [(net_number <integer>)]
//    [(pins {<pin_reference>}) | (order {<pin_reference>})]
//    [<component_order_descriptor>]
//    [(type [fix | normal])]
//    [<user_property_descriptor>]
//    [<circuit_descriptor>]
//    [<rule_descriptor>]
//    [{<layer_rule_descriptor>}]
//    [<fromto_descriptor>]
//    [(expose {<pin_reference>})]
//    [(noexpose {<pin_reference>})]
//    [(source {<pin_reference>})]
//    [(load {<pin_reference>})]
//    [(terminator {<pin_reference>})]
//    [(supply [power | ground])])
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Net {
    pub net_id: Id,
    pub pins: Vec<PinRef>, // Of the form: ComponentId-PinId
}

// <circuit_descriptors> = [<delay_descriptor> |
//    <total_delay_descriptor> |
//    <length_descriptor> |
//    <total_length_descriptor> |
//    <match_fromto_length_descriptor> |
//    <match_fromto_delay_descriptor> |
//    <match_group_length_descriptor> |
//    <match_group_delay_descriptor> |
//    <match_net_length_descriptor> |
//    <match_net_delay_descriptor> |
//    <relative_delay_descriptor> |
//    <relative_group_delay_descriptor> |
//    <relative_group_length_descriptor> |
//    <relative_length_descriptor> |
//    <sample_window_descriptor> |
//    <switch_window_descriptor> |
//    <shield_descriptor> |
//    <max_restricted_layer_length_descriptor> |
//    (priority <positive_integer>) |
//    (use_layer {<layer_name>}) |
//    (use_via {[<padstack_id> |
//        (use_array <via_array_template_id> [<row> <column>])]})]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Circuit {}

// <rule_descriptor> = (rule {<rule_descriptors>})
// <rule_descriptors> =
//    [<clearance_descriptor> |
//    <effective_via_length_descriptor> |
//    <interlayer_clearance_descriptor> |
//    <junction_type_descriptor> |
//    <length_amplitude_descriptor> |
//    <length_factor_descriptor> |
//    <length_gap_descriptor> |
//    <limit_bends_descriptor> |
//    <limit_crossing_descriptor> |
//    <limit_vias_descriptor> |
//    <limit_way_descriptor> |
//    <max_noise_descriptor> |
//    <max_stagger_descriptor> |
//    <max_stub_descriptor> |
//    <max_total_vias_descriptor> |
//    {<parallel_noise_descriptor>} |
//    {<parallel_segment_descriptor>} |
//    <pin_width_taper_descriptor> |
//    <power_fanout_descriptor> |
//    <redundant_wiring_descriptor> |
//    <reorder_descriptor> |
//    <restricted_layer_length_factor_descriptor> |
//    <saturation_length_descriptor> |
//    <shield_gap_descriptor> |
//    <shield_loop_descriptor> |
//    <shield_tie_down_interval_descriptor> |
//    <shield_width_descriptor> |
//    {<stack_via_descriptor>} |
//    {<stack_via_depth_descriptor>} |
//    {<tandem_noise_descriptor>} |
//    {<tandem_segment_descriptor>} |
//    <tandem_shield_overhang_descriptor> |
//    <testpoint_rule_descriptor> |
//    <time_length_factor_descriptor> |
//    <tjunction_descriptor> |
//    <track_id_descriptor> |
//    <via_at_smd_descriptor> |
//    <via_pattern_descriptor> |
//    <width_descriptor>]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Rule {}

// <class_descriptor> = (class <class_id>
//    {[{<net_id>} | {<composite_name_list>}]} [<circuit_descriptor>]
//    [<rule_descriptor>] [{<layer_rule_descriptor>}] [<topology_descriptor>])
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Class {
    pub class_id: Id,
    pub net_ids: Vec<Id>,
    pub circuits: Vec<Circuit>,
    pub rules: Vec<Rule>,
}

// <network_descriptor> = (network
//    {<net_descriptor>}
//    [{<class_descriptor>}]
//    [{<class_class_descriptor>}]
//    [{<group_descriptor>}]
//    [{<group_set_descriptor>}]
//    [{<pair_descriptor>}]
//    [{<bundle_descriptor>}])
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Network {
    pub nets: Vec<Net>,
    pub classes: Vec<Class>,
}

// <library_descriptor> = (library
//    [<unit_descriptor>]
//    {<image_descriptor>}
//    [{<jumper_descriptor>}]
//    {<padstack_descriptor>}
//    {<via_array_template_descriptor>}
//    [<directory_descriptor>]
//    [<extra_image_directory_descriptor>]
//    [{<family_family_descriptor>}]
//    [{<image_image_descriptor>}])
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Library {
    pub images: Vec<Image>,
    pub padstacks: Vec<Padstack>,
}

// <layer_type> = [signal | power | mixed | jumper]
#[derive(Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum LayerType {
    Signal,
    Power,
    Mixed,
    Jumper,
}

impl Default for LayerType {
    fn default() -> Self {
        Self::Signal
    }
}

// <layer_descriptor> = (layer <layer_name>
//    (type <layer_type>)
//    [<user_property_descriptor>]
//    [(direction <direction_type>)]
//    [<rule_descriptor>]
//    [(cost <cost_descriptor> [(type [length | way])])]
//    [(use_net {<net_id>})])
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Layer {
    pub layer_name: Id,
    pub layer_type: LayerType,
}

// <plane_descriptor> = (plane <net_id>
//    <shape_descriptor>
//    [{<window_descriptor>}])
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Plane {
    pub net_id: Id,
    pub shape: Shape,
    pub windows: Vec<Window>,
}

// <boundary_descriptor> = (boundary
//    [{<path_descriptor>} | <rectangle_descriptor>] [<rule_descriptor>])

// <structure_descriptor> = (structure
//    [<unit_descriptor> | <resolution_descriptor> | null]
//    {<layer_descriptor>}
//    [<layer_noise_weight_descriptor>]
//    {<boundary_descriptor>}
//    [<place_boundary_descriptor>]
//    [{<plane_descriptor>}]
//    [{<region_descriptor>}]
//    [{<keepout_descriptor>}]
//    <via_descriptor>
//    [<control_descriptor>]
//    <rule_descriptor>
//    [<structure_place_rule_descriptor>]
//    {<grid_descriptor>})
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Structure {
    pub boundaries: Vec<Shape>,
    pub keepouts: Vec<Keepout>,
    pub layers: Vec<Layer>,
    pub planes: Vec<Plane>,
    pub rules: Vec<Rule>,
    pub vias: Vec<Id>,
}

// <placement_descriptor> = (placement
//    [<unit_descriptor> | <resolution_descriptor> | null]
//    [<place_control_descriptor>]
//    {<component_instance>})
// Describes the location of components on the pcb.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Placement {
    pub components: Vec<Component>,
}

// <resolution_descriptor> = (resolution <dimension_unit> <positive_integer>)
// Everything is in the units specified by |dimension|, but the resolution
// of numbers specified is limited by 1 / |amount|. That is, there are
// |amount| divisions in |dimension|.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub amount: i32,              // Default value is 2540000.
    pub dimension: DimensionUnit, // Default value is INCH.
}

impl Default for Resolution {
    fn default() -> Self {
        Self { amount: 2540000, dimension: DimensionUnit::Inch }
    }
}

// <wire_shape_descriptor> = (wire
//    <shape_descriptor>
//    [(net <net_id>)]
//    [(turret <turret#>)]
//    [(type [fix | route | normal | protect])]
//    [(attr [test | fanout | bus | jumper])]
//    [(shield <net_id>)]
//    [{<window_descriptor>}]
//    [(connect
//        (terminal <object_type> [<pin_reference>])
//        (terminal <object_type> [<pin_reference>]))]
//    [(supply)])
// Describes a trace. Traces may have any shape.
#[derive(Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum WireType {
    Fix,
    Route,
    Normal,
    Protect,
}
#[derive(Debug, Clone, PartialEq, Eq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum WireAttr {
    Test,
    Fanout,
    Bus,
    Jumper,
}
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Wire {}

// <wire_via_descriptor> = (via
//    <padstack_id> {<vertex>}
//    [(net <net_id>)]
//    [(via_number <via#>)]
//    [(type [fix | route | normal | protect])]
//    [(attr [test | fanout | jumper |
//    virtual_pin <virtual_pin_name>])]
//    [(contact {<layer_id>})]
//    [(supply)])
// (virtual_pin
//    <virtual_pin_name> <vertex> (net <net_id>))
// Describes a via.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Via {}

// <wiring_descriptor> = (wiring
//    [<unit_descriptor> | <resolution_descriptor> | null]
//    {<wire_descriptor>}
//    [<test_points_descriptor>]
//    {[<supply_pin_descriptor>]})
// Describes pre-existing traces and vias on the PCB.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Wiring {
    pub wires: Wire,
    pub vias: Via,
}

// <design_descriptor> = (pcb <pcb_id>
//    [<parser_descriptor>]
//    [<capacitance_resolution_descriptor>]
//    [<conductance_resolution_descriptor>]
//    [<current_resolution_descriptor>]
//    [<inductance_resolution_descriptor>]
//    [<resistance_resolution_descriptor>]
//    [<resolution_descriptor>]
//    [<time_resolution_descriptor>]
//    [<voltage_resolution_descriptor>]
//    [<unit_descriptor>]
//    [<structure_descriptor> | <file_descriptor>]
//    [<placement_descriptor> | <file_descriptor>]
//    [<library_descriptor> | <file_descriptor>]
//    [<floor_plan_descriptor> | <file_descriptor>]
//    [<part_library_descriptor> | <file_descriptor>]
//    [<network_descriptor> | <file_descriptor>]
//    [<wiring_descriptor>]
//    [<color_descriptor>])
// Describes an overall PCB.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Pcb {
    pub pcb_id: Id,
    pub library: Library,
    pub network: Network,
    pub placement: Placement,
    pub resolution: Resolution,
    pub structure: Structure,
    pub unit: Resolution, // Overrides |resolution| - amount is always 1.
    pub wiring: Wiring,
}
