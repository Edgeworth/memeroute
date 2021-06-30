# Copyright 2021 Eliot Courtney.
import re
import sys

keywords = [
    "absolute", "added", "add_group", "add_pins", "allow_antenna",
    "allow_redundant_wiring", "amp", "ancestor", "antipad", "aperture_type",
    "array", "attach", "attr", "average_pair_length", "back", "base_design",
    "bbv_ctr2ctr", "bend_keepout", "bond", "both", "bottom", "bottom_layer_sel",
    "boundary", "brickpat", "bundle", "bus", "bypass", "capacitance_resolution",
    "capacitor", "case_sensitive", "cct1", "cct1a", "center_center",
    "checking_trim_by_pin", "circ", "circle", "circuit", "class",
    "class_class", "classes", "clear", "clearance", "cluster", "cm", "color",
    "colors", "comment", "comp", "comp_edge_center", "comp_order", "component",
    "composite", "conductance_resolution", "conductor", "conflict", "connect",
    "constant", "contact", "control", "corner", "corners", "cost",
    "created_time", "cross", "crosstalk_model", "current_resolution",
    "deleted", "deleted_keepout", "delta", "diagonal", "direction",
    "directory", "discrete", "effective_via_length", "elongate_keepout",
    "exclude", "expose", "extra_image_directory", "family", "family_family",
    "family_family_spacing", "fanout", "farad", "file", "fit", "fix",
    "flip_style", "floor_plan", "footprint", "forbidden",
    "force_to_terminal_point", "forgotten", "free", "fromto", "front",
    "front_only", "gap", "gate", "gates", "generated_by_freeroute", "global",
    "grid", "group", "group_set", "guide", "hard", "height", "high", "history",
    "horizontal", "host_cad", "host_version", "image", "image_conductor",
    "image_image", "image_image_spacing", "image_outline_clearance",
    "image_set", "image_type", "inch", "include", "include_pins_in_crosstalk",
    "inductance_resolution", "insert", "instcnfg", "inter_layer_clearance",
    "jumper", "junction_type", "keepout", "kg", "kohm", "large",
    "large_large", "layer", "layer_depth", "layer_noise_weight",
    "layer_pair", "layer_rule", "length", "length_amplitude",
    "length_factor", "length_gap", "library", "library_out", "limit",
    "limit_bends", "limit_crossing", "limit_vias", "limit_way", "linear",
    "linear_interpolation", "load", "lock_type", "logical_part",
    "logical_part_mapping", "low", "match_fromto_delay", "match_fromto_length",
    "match_group_delay", "match_group_length", "match_net_delay",
    "match_net_length", "max_delay", "max_len", "max_length", "max_noise",
    "max_restricted_layer_length", "max_stagger", "max_stub", "max_total_delay",
    "max_total_length", "max_total_vias", "medium", "mhenry", "mho", "microvia",
    "mid_driven", "mil", "min_gap", "mirror", "mirror_first", "mixed", "mm",
    "negative_diagonal", "net", "net_number", "net_out", "net_pin_changes",
    "nets", "network", "network_out", "no", "noexpose", "noise_accumulation",
    "noise_calculation", "normal", "object_type", "off", "off_grid", "offset",
    "on", "open", "opposite_side", "order", "orthogonal", "outline", "overlap",
    "pad", "pad_pad", "padstack", "pair", "parallel", "parallel_noise",
    "parallel_segment", "parser", "part_library", "path", "pcb",
    "permit_orient", "permit_side", "physical", "physical_part_mapping",
    "piggyback", "pin", "pin_allow", "pin_cap_via", "pin_via_cap",
    "pin_width_taper", "pins", "pintype", "place", "place_boundary",
    "place_control", "place_keepout", "place_rule", "placement", "plan",
    "plane", "pn", "point", "polyline_path", "polygon", "position",
    "positive_diagonal", "power", "power_dissipation", "power_fanout",
    "prefix", "primary", "priority", "property", "protect", "qarc", "quarter",
    "radius", "ratio", "ratio_tolerance", "rect", "reduced", "region",
    "region_class", "region_class_class", "region_net", "relative_delay",
    "relative_group_delay", "relative_group_length", "relative_length",
    "reorder", "reroute_order_viols", "resistance_resolution", "resistor",
    "resolution", "restricted_layer_length_factor", "room", "rotate",
    "rotate_first", "round", "roundoff_rotation", "route",
    "route_to_fanout_only", "routes", "routes_include", "rule",
    "same_net_checking", "sample_window", "saturation_length", "sec",
    "secondary", "self", "sequence_number", "session", "set_color",
    "set_pattern", "shape", "shield", "shield_gap", "shield_loop",
    "shield_tie_down_interval", "shield_width", "side", "signal", "site",
    "small", "smd", "snap", "snap_angle", "soft", "source",
    "space_in_quoted_tokens", "spacing", "spare", "spiral_via", "square",
    "stack_via", "stack_via_depth", "standard", "starburst", "status",
    "structure", "structure_out", "subgate", "subgates", "substituted", "such",
    "suffix", "super_placement", "supply", "supply_pin", "swapping",
    "switch_window", "system", "tandem_noise", "tandem_segment",
    "tandem_shield_overhang", "terminal", "terminator", "term_only", "test",
    "test_points", "testpoint", "threshold", "time_length_factor",
    "time_resolution", "tjunction", "tolerance", "top", "topology", "total",
    "track_id", "turret", "type", "um", "unassigned", "unconnects", "unit",
    "up", "use_array", "use_layer", "use_net", "use_via", "value", "vertical",
    "via", "via_array_template", "via_at_smd", "via_keepout", "via_number",
    "via_rotate_first", "via_site", "via_size", "virtual_pin", "volt",
    "voltage_resolution", "was_is", "way", "weight", "width", "window", "wire",
    "wire_keepout", "wires", "wires_include", "wiring", "write_resolution",
    "pin_pin", "smd_pin", "via_pin", "wire_pin", "area_pin", "testpoint_pin",
    "pin_smd", "smd_smd", "via_smd", "wire_smd", "area_smd", "testpoint_smd",
    "pin_via", "smd_via", "via_via", "wire_via", "area_via", "testpoint_via",
    "pin_wire", "smd_wire", "via_wire", "wire_wire", "area_wire",
    "testpoint_wire", "pin_area", "smd_area", "via_area", "wire_area",
    "area_area", "testpoint_area", "pin_testpoint", "smd_testpoint",
    "via_testpoint", "wire_testpoint", "area_testpoint", "testpoint_testpoint",
    "smd_via_same_net", "via_via_same_net", "buried_via_gap", "antipad_gap",
    "pad_to_turn_gap", "smd_to_turn_gap", ]

path = []
counts = {}


def add_path():
    global counts
    ct = '.'.join(path)
    counts.setdefault(ct, 0)
    counts[ct] += 1


def valid_keyword(t):
    return t in keywords


def print_counts():
    global counts
    vals = [(v, k) for k, v in counts.items()]
    vals.sort(key=lambda x: (x[1], x[0]))
    for v, k in vals:
        print('%s: %d' % (k, v))


def process(data):
    idx = 0

    def next_token():
        nonlocal idx
        idx += 1
        return tokens[idx - 1]

    quote = '"'
    if '(string_quote ")' in data:
        quote = '"'
    if '(string_quote \')' in data:
        quote = '\''
    if '(string_quote $)' in data:
        quote = '$'
    data = data.replace('(string_quote ")', '')
    data = data.replace('(string_quote \')', '')
    data = data.replace('(string_quote $)', '')
    data = data.replace(r'\"', '')  # get rid of any escaped quotes
    data = data.lower()  # just force everything to lowercase
    new_data = ''
    i = 0
    while i < len(data):
        new_data += data[i]
        if data[i] == quote:
            i += 1
            while data[i] != quote:
                i += 1
            new_data += quote
        i += 1
    tokens = re.split(r'([()\s])', new_data)
    tokens = [i.strip() for i in tokens]
    tokens = [i for i in tokens if i]

    while idx < len(tokens):
        t = next_token()
        if t == '(':
            nt = next_token()
            assert nt != ')'
            path.append(nt)
            add_path()
        elif t == ')':
            path.pop()
        else:
            # Need to be in some paren.
            assert len(path) > 0
            if valid_keyword(t):
                path.append(t)
                add_path()
                path.pop()
    assert len(path) == 0


for v in sys.argv[1:]:
    try:
        data = open(v).read()
        process(data)
    except:
        print('Failed to process', v)
print_counts()
