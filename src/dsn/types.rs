use memegeom::primitive::point::Pt;
use memegeom::primitive::rect::Rt;
use strum::{Display as EnumDisplay, EnumString};

// Types defined in DSN specification.

// <number> = [<sign>] (<positive_integer> | <real> | <fraction>)
// <dimension> = <number>
// <vertex> = PtF

// <unit_descriptor> = (unit <dimension_unit>)
// <dimension_unit> = [inch | mil | cm | mm | um]
#[derive(Debug, Clone, PartialEq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum DsnDimensionUnit {
    Inch,
    Mil,
    Cm,
    Mm,
    Um,
}

impl Default for DsnDimensionUnit {
    fn default() -> Self {
        Self::Inch
    }
}

// <id>, <component_id>, <pin_id>, <padstack_id>, <via_id>, <image_id>,
// <net_id>, <pcb_id>, <class_id> = std::string
pub type DsnId = String;

// <layer_id> = <id> | pcb | signal | power
pub type DsnLayerId = DsnId;

// <rectangle_descriptor> = (rect <layer_id> <vertex> <vertex>)
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnRect {
    pub layer_id: DsnLayerId,
    pub rect: Rt,
}

// <circle_descriptor> = (circle <layer_id> <diameter> [<vertex>])
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnCircle {
    pub layer_id: DsnLayerId,
    pub diameter: f64,
    pub p: Pt, // Defaults to PCB origin.
}

// <polygon_descriptor> = (polygon <layer_id> <aperture_width> {<vertex>}
//    [(aperture_type [round | square])])
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnPolygon {
    pub layer_id: DsnLayerId,
    pub aperture_width: f64,
    pub pts: Vec<Pt>,
}

// <path_descriptor> = (path <layer_id> <aperture_width> {<vertex>}
//    [(aperture_type [round | square])])
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnPath {
    pub layer_id: DsnLayerId,
    pub aperture_width: f64,
    pub pts: Vec<Pt>,
}

// <qarc_descriptor> = (qarc <layer_id> <aperture_width>
//    <vertex> <vertex> <vertex>)
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnQArc {
    pub layer_id: DsnLayerId,
    pub aperture_width: f64,
    pub start: Pt,
    pub end: Pt,
    pub center: Pt,
}

// <shape_descriptor> = = [<rectangle_descriptor> | <circle_descriptor> |
//    <polygon_descriptor> | <path_descriptor> | <qarc_descriptor>]
#[derive(Debug, Clone, PartialEq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum DsnShape {
    Rect(DsnRect),
    Circle(DsnCircle),
    Polygon(DsnPolygon),
    Path(DsnPath),
    QArc(DsnQArc),
}

impl Default for DsnShape {
    fn default() -> Self {
        Self::Rect(DsnRect::default())
    }
}

// <window_descriptor> = (window <shape_descriptor>) - can only be rect or
// polygon.
#[derive(Debug, Clone, PartialEq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum DsnWindow {
    Rect(DsnRect),
    Polygon(DsnPolygon),
}

impl Default for DsnWindow {
    fn default() -> Self {
        Self::Rect(DsnRect::default())
    }
}

// <object_type> = [pin | smd | via | wire | area | testpoint]
#[derive(Debug, Clone, PartialEq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum DsnObjectType {
    Pin,
    Smd,
    Via,
    Wire,
    Area,
    #[strum(serialize = "testpoint")]
    TestPoint,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnPadstackShape {
    pub shape: DsnShape,
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
#[derive(Debug, Clone, PartialEq)]
pub struct DsnPadstack {
    pub padstack_id: DsnId,
    pub shapes: Vec<DsnPadstackShape>,
    pub attach: bool, // Default is to allow vias under SMD pads.
}

impl Default for DsnPadstack {
    fn default() -> Self {
        Self { padstack_id: String::new(), shapes: Vec::new(), attach: true }
    }
}

// Describes a side of the PCB.
#[derive(Debug, Clone, PartialEq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum DsnSide {
    Front,
    Back,
    Both,
}

impl Default for DsnSide {
    fn default() -> Self {
        Self::Front
    }
}

// {(pin <padstack_id> [(rotate <rotation>)]
//    [<reference_descriptor> | <pin_array_descriptor>]
//    [<user_property_descriptor>])}
// Describes a pin within an image (type of component). Pins connect onto
// padstacks.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnPin {
    pub padstack_id: DsnId, // Padstack describes the shape of the pin
    pub rotation: f64,      // Rotation in degrees. Default to 0
    pub pin_id: DsnId,      // Describes TODO e.g. 1@1
    pub p: Pt,              // Location of the pin relative to the parent component (placement).
}

// Keepout: No routing whatsoever.
// ViaKeepout: No vias.
// WireKeepout: No wires.
#[derive(Debug, Clone, PartialEq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum DsnKeepoutType {
    Keepout,
    ViaKeepout,
    WireKeepout,
}

impl Default for DsnKeepoutType {
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
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnKeepout {
    pub keepout_type: DsnKeepoutType,
    pub shape: DsnShape,
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
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnImage {
    pub image_id: DsnId,
    pub outlines: Vec<DsnShape>,
    pub pins: Vec<DsnPin>,
    pub keepouts: Vec<DsnKeepout>,
}

#[derive(Debug, Clone, PartialEq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum DsnLockType {
    None,
    Position,
    Gate,
}

impl Default for DsnLockType {
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
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnPlacementRef {
    pub component_id: DsnId,
    pub p: Pt,
    pub side: DsnSide,
    pub rotation: f64,
    pub lock_type: DsnLockType,
    pub part_number: DsnId,
}

// <component_instance> = (component <image_id> {<placement_reference>})
// Describes instances of a component. The component type is determined
// by the image referred to by |image_id|. There is a component at each
// location specified by each placement reference.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnComponent {
    pub image_id: DsnId,
    pub refs: Vec<DsnPlacementRef>,
}

// <pin_reference> = <component_id>-<pin_id>
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnPinRef {
    pub component_id: DsnId,
    pub pin_id: DsnId,
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
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnNet {
    pub net_id: DsnId,
    pub pins: Vec<DsnPinRef>, // Of the form: ComponentId-PinId
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
// Describes some rules about routing. Included within a class.
#[derive(Debug, Clone, PartialEq)]
pub enum DsnCircuit {
    UseVia(DsnId), // Padstack id of via to use
}

// <clearance_type> = [<object_type>_<object_type> | smd_via_same_net |
//    via_via_same_net | buried_via_gap [(layer_depth <positive_integer>)] |
//    antipad_gap | pad_to_turn_gap | smd_to_turn_gap]
#[derive(Debug, Clone, PartialEq)]
pub enum DsnClearanceType {
    All, // If unspecified, choose all.
    // This is not part of the official spec but default seems to be used to
    // mean wildcard for any type (and overriden by specific designations)
    DefaultSmd,
    SmdSmd,
}
// <clearance_descriptor> = (clearance <positive_dimension> [(type {<clearance_type>})]
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnClearance {
    pub amount: f64,
    pub types: Vec<DsnClearanceType>,
}

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
#[derive(Debug, Clone, PartialEq)]
pub enum DsnRule {
    Width(f64),
    Clearance(DsnClearance),
}

// <class_descriptor> = (class <class_id>
//    {[{<net_id>} | {<composite_name_list>}]} [<circuit_descriptor>]
//    [<rule_descriptor>] [{<layer_rule_descriptor>}] [<topology_descriptor>])
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnClass {
    pub class_id: DsnId,
    pub net_ids: Vec<DsnId>,
    pub circuits: Vec<DsnCircuit>,
    pub rules: Vec<DsnRule>,
}

// <network_descriptor> = (network
//    {<net_descriptor>}
//    [{<class_descriptor>}]
//    [{<class_class_descriptor>}]
//    [{<group_descriptor>}]
//    [{<group_set_descriptor>}]
//    [{<pair_descriptor>}]
//    [{<bundle_descriptor>}])
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnNetwork {
    pub nets: Vec<DsnNet>,
    pub classes: Vec<DsnClass>,
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
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnLibrary {
    pub images: Vec<DsnImage>,
    pub padstacks: Vec<DsnPadstack>,
}

// <layer_type> = [signal | power | mixed | jumper]
#[derive(Debug, Clone, PartialEq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum DsnLayerType {
    Signal,
    Power,
    Mixed,
    Jumper,
}

impl Default for DsnLayerType {
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
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnLayer {
    pub layer_name: DsnId,
    pub layer_type: DsnLayerType,
}

// <plane_descriptor> = (plane <net_id>
//    <shape_descriptor>
//    [{<window_descriptor>}])
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnPlane {
    pub net_id: DsnId,
    pub shape: DsnShape,
    pub windows: Vec<DsnWindow>,
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
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnStructure {
    pub boundaries: Vec<DsnShape>,
    pub keepouts: Vec<DsnKeepout>,
    pub layers: Vec<DsnLayer>,
    pub planes: Vec<DsnPlane>,
    pub rules: Vec<DsnRule>,
    pub vias: Vec<DsnId>,
}

// <placement_descriptor> = (placement
//    [<unit_descriptor> | <resolution_descriptor> | null]
//    [<place_control_descriptor>]
//    {<component_instance>})
// Describes the location of components on the pcb.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnPlacement {
    pub components: Vec<DsnComponent>,
}

// <resolution_descriptor> = (resolution <dimension_unit> <positive_integer>)
// Everything is in the units specified by |dimension|, but the resolution
// of numbers specified is limited by 1 / |amount|. That is, there are
// |amount| divisions in |dimension|.
#[derive(Debug, Clone, PartialEq)]
pub struct DsnResolution {
    pub amount: i32,                 // Default value is 2540000.
    pub dimension: DsnDimensionUnit, // Default value is INCH.
}

impl Default for DsnResolution {
    fn default() -> Self {
        Self { amount: 2540000, dimension: DsnDimensionUnit::Inch }
    }
}

#[derive(Debug, Clone, PartialEq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum DsnWireType {
    Fix,
    Route,
    Normal,
    Protect,
}

#[derive(Debug, Clone, PartialEq, EnumString, EnumDisplay)]
#[strum(serialize_all = "snake_case")]
pub enum DsnWireAttr {
    Test,
    Fanout,
    Bus,
    Jumper,
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
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnWire {}

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
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnVia {}

// <wiring_descriptor> = (wiring
//    [<unit_descriptor> | <resolution_descriptor> | null]
//    {<wire_descriptor>}
//    [<test_points_descriptor>]
//    {[<supply_pin_descriptor>]})
// Describes pre-existing traces and vias on the PCB.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnWiring {
    pub wires: Vec<DsnWire>,
    pub vias: Vec<DsnVia>,
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
#[derive(Debug, Default, Clone, PartialEq)]
pub struct DsnPcb {
    pub pcb_id: DsnId,
    pub library: DsnLibrary,
    pub network: DsnNetwork,
    pub placement: DsnPlacement,
    pub resolution: DsnResolution,
    pub structure: DsnStructure,
    pub unit: DsnResolution, // Overrides |resolution| - amount is always 1.
    pub wiring: DsnWiring,
}
