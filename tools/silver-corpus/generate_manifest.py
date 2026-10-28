#!/usr/bin/env python3
"""Generate silver-corpus.toml from the pinned upstream C++ producers.

Active literal matches() and checkTextLayoutSilver() calls are discovered mechanically. The layout-scroll
and grid-stack names assembled through helper arguments are deliberately
listed below, and producerless files are deliberately classified as
provenance-unknown.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import struct
import subprocess
from dataclasses import dataclass
from pathlib import Path

UPSTREAM_REF = "fdb3d5f6a5c3d6a5870db0629dab139931fb0acf"
LITERAL_MATCH = re.compile(
    r'(?:silver\.matches|serializer\(\)->matches)\(\s*"([^"]+)"', re.MULTILINE
)
TEST_CASE = re.compile(r"\bTEST_CASE\s*\(")
RIV_STRING = re.compile(r'"([^"\n]*\.riv)"')
ARTBOARD_NAME = re.compile(r'artboardNamed\(\s*"([^"]+)"')
ANIMATION_NAME = re.compile(r'animationNamed\(\s*"([^"]+)"')
STATE_MACHINE_NAME = re.compile(r'stateMachineNamed\(\s*"([^"]+)"')
SAMPLE_TIME = re.compile(
    r"(?:advanceAndApply|advance)\(\s*([0-9]+(?:\.[0-9]+)?)(?:f)?\s*\)"
)


@dataclass(frozen=True)
class Producer:
    id: str
    source: str
    dependencies: tuple[str, ...]
    artboard: str
    animation: str
    state_machine: str
    lane: str
    deterministic: str
    random: str
    view_model: str
    sample_times: tuple[float, ...]
    actions: str | tuple[dict[str, object], ...]
    status: str
    producer_class: str
    provenance_file: str
    provenance_test: str
    producer_line: int
    note: str
    clone_artboard_instance: bool = False


DYNAMIC_LAYOUT_SCROLL = (
    (
        "layout_scroll_snap_padding_layouts",
        "layout/layout_scroll_snap_padding.riv",
        "ScrollLayouts",
        "Scroll snap respects viewport padding (layouts)",
        550,
    ),
    (
        "layout_scroll_snap_padding_list",
        "layout/layout_scroll_snap_padding.riv",
        "ScrollList",
        "Scroll snap respects viewport padding (list)",
        556,
    ),
    (
        "layout_scroll_snap_padding_virtualized",
        "layout/layout_scroll_snap_padding.riv",
        "ScrollListVirtualized",
        "Scroll snap respects viewport padding (virtualized list)",
        562,
    ),
    (
        "layout_scroll_drag_multiplier_layouts",
        "layout/layout_scroll_drag_multiplier.riv",
        "ScrollLayouts",
        "Scroll drag multiplier (layouts)",
        569,
    ),
    (
        "layout_scroll_drag_multiplier_list",
        "layout/layout_scroll_drag_multiplier.riv",
        "ScrollList",
        "Scroll drag multiplier (list)",
        575,
    ),
    (
        "layout_scroll_drag_multiplier_virtualized",
        "layout/layout_scroll_drag_multiplier.riv",
        "ScrollListVirtualized",
        "Scroll drag multiplier (virtualized list)",
        581,
    ),
)

DYNAMIC_GRID_STACK = (
    (
        "layout_grid_stack_grid_with_layouts",
        "GridWithLayouts",
        "grid with layouts silver",
        44,
    ),
    (
        "layout_grid_stack_stack_with_layouts",
        "StackWithLayouts",
        "stack with layouts silver",
        49,
    ),
    (
        "layout_grid_stack_grid_with_layouts_size_changing",
        "GridWithLayoutsSizeChanging",
        "grid with layouts size changing silver",
        54,
    ),
    (
        "layout_grid_stack_grid_with_layouts_span",
        "GridWithLayoutsSpan",
        "grid with layouts span silver",
        60,
    ),
    (
        "layout_grid_stack_grid_with_layouts_size_span_changing",
        "GridWithLayoutsSizeSpanChanging",
        "grid with layouts size span changing silver",
        66,
    ),
    (
        "layout_grid_stack_grid_with_layout_participants",
        "GridWithLayoutParticipants",
        "grid with layout participants silver",
        72,
    ),
)

SERIALIZED_RENDERING_FFP_CONTRACT_DIVERGENCES = {
    "car_widgets_v01",
    # Keyframe interpolation contracts in the production lane (1ae307dc03);
    # the strict-fp lane reproduces the stored silver exactly.
    "data_converter_to_number",
    "hunter_x_demo",
    "rewards_demo",
}

# Layout is a deliberate product divergence from upstream Rive (2026-10-14):
# Nuxie lays out with stock Taffy and CSS semantics, not Rive's Yoga. Each
# case whose first difference is caused by that solver change is listed with
# the reason. Historical exact replacements pin the full Rust stream too, so
# changes after the recorded first difference also fail validation.
LAYOUT_DIVERGENCES: dict[str, str] = {
    "game_menu_ad_police_files": (
        "After correcting the finite fit-content container offer, frame 14 op 4868 "
        "is the description container (object 170, path 60): CSS retains text179's "
        "754.0928 inline size plus 40 padding, giving 794.0928; C++ uses its "
        "676.91016 widest wrapped line plus 40, giving 716.91016. Single-owner "
        "controls associate the path with this container. This explains the "
        "first remaining difference, not every later replay operation"
    ),
    "layoutstest_8-planets-grid": (
        "CSS Fill uses 0% bases in the 4377-point root; the right pane's automatic "
        "minimum from eight 284-point cells holds it at 2272, leaving 2105 on the "
        "left. C++ content bases 284 and 2272 share 1821 free points, giving "
        "1194.5 and 3182.5. This explains the first difference, not the entire replay"
    ),
    "layout_grid_stack_grid_with_layout_participants": (
        "CSS automatic minima retain Shape22's natural height 221.80002 in cell19 "
        "and natural width 151.38115 in participant29. The first row grows from "
        "190 to 221.80002; later the participant cannot shrink with its cell. "
        "Diagnostic zeroing of only those two minima makes all 121 frames match C++"
    ),
    "inventory_demo_test_v2": (
        "CSS Fill uses a 0% basis: the three columns share 1640 equally instead of "
        "adding equal free space to content bases 515, 320 and 430. The middle "
        "item starts at 800 instead of 842.5. This explains the first difference, "
        "not every later replay operation"
    ),
    "collapse_data_binds-test_1": (
        "Fill is CSS flex: 1 1 0%, so the four Fill cells of the 500-point row share it "
        "equally at 125 points each whatever their text, where C++ sizes each Fill from "
        "its content first (118.98, 118.98, 118.98 and 143.05). The last cell's text moves "
        "to x 419.16; C++ places it at 411.32, which already differed by 1.18 points "
        "because a collapsed sibling shifts the item less than Yoga does"
    ),
    "layout_aspect_ratio": (
        "CSS sizing resolves the aspect-ratio box to 142x133 where Yoga keeps "
        "the authored 2:1 ratio at 142x71 (the Taffy fork already produced 142x133)"
    ),
    "layout_text_match": (
        "CSS fit-content keeps a hugging text box at its 300 max width; Yoga "
        "shrinks it to the widest wrapped line (262.43)"
    ),
    "layout_text_match_7_3": (
        "CSS fit-content keeps a hugging text box at its 300 max width; Yoga "
        "shrinks it to the widest wrapped line (262.43)"
    ),
}


# Previous stream (UNIV-4844 final) db_health_tracker: f7c6d803e6b9873b07ab7dd4ab8f0acd31b01d5516b4500d07192925687fbb68.
# Previous difference (main) collapse_data_binds-test_1: frame 0, op 100 (transform), field tx: expected 411.31592, got 410.13672
# UNIV-4844: historical EXACT IDs and the numeric floor stay unchanged.
# Each reviewed replacement pins the first difference and every Rust stream byte.
# Previous review artboard_list_overrides_vertical: frame 0, op 73 (addRawPath), field point: expected (500, 0), got (700, 0)
# Previous review bankcard: frame 1, op 711 (addRawPath), field point: expected (810, 0), got (826.94385, 0)
# Previous review db_health_tracker: frame 0, op 4764 (addRawPath), field point: expected (968, 0), got (10134, 0)
# Previous review global_view_models_scripting_test: frame 0, op 49 (addRawPath), field point: expected (945, 0), got (980, 0)
# Previous review hittest_ab1_grand_parent: frame 0, op 149 (transform), field ty: expected 100, got 0
# Previous review hittest_ab1_parent: frame 0, op 129 (addRawPath), field point: expected (572, 0), got (738.65234, 0)
# Previous review number_to_list_nested_children: frame 0, op 190 (addRawPath), field point: expected (250, 59.791016), got (250, 61)
# Previous review reset_phase_multi_main: frame 0, op 57 (addRawPath), field point: expected (180, 0), got (186, 0)
# Previous review scroll_threshold-all-scroll: frame 0, op 62 (addRawPath), field point: expected (500, 0), got (1500, 0)
# Previous review scroll_threshold-horizontal-scroll: frame 0, op 59 (addRawPath), field point: expected (500, 0), got (1700, 0)
# Previous review text_input: frame 0, op 20 (addRawPath), field point: expected (200, 0), got (492.99805, 0)
# Previous review (d8217e4476) collapsing_elements: frame 0, op 612 (addRawPath), field point: expected (500, -0.0 (0x80000000)), got (499.95004, -0.0 (0x80000000)); now exact with C++
# Previous review (d8217e4476) draw_index_list: frame 0, op 54 (color): expected color, got makeRenderPaint; now exact with C++
# Previous review (d8217e4476) layout_scroll_snap_carousel: frame 0, op 58 (addRawPath), field point: expected (105, -0.0 (0x80000000)), got (94.75, -0.0 (0x80000000)); now exact with C++
# Previous review (d8217e4476) superbowl: frame 0, op 2964 (transform), field xx: expected 0.6741602, got 0.42949218; now exact with C++
# Previous review (d8217e4476) text_layout_7_3: frame 0, op 137 (transform), field tx: expected 35.7041, got 29.107422; now exact with C++
# Previous review (d8217e4476) text_layout_pre_7_3: frame 0, op 137 (transform), field tx: expected 35.7041, got 29.107422; now exact with C++
# Previous review (d8217e4476) artboard_list_overrides_vertical: frame 0, op 78 (addRawPath), field point: expected (700, 0), got (500, 0)
# Previous review (d8217e4476) component_list_child_origin: frame 0, op 105 (transform), field ty: expected 29, got 12.5
# Previous review (d8217e4476) db_health_tracker: frame 0, op 7788 (transform), field tx: expected 864.3635, got 857.43286
# Previous stream (bca2243599) db_health_tracker: 95122ffa7516a8c686466e3ec58f964b6736e3c3e92640380b71a4d05f95a29d. The
# update pass now measures a panel's grouped content at its laid-out transforms rather than
# the identity matrices of the first layout, so one y scale matches C++ (0.16, was 0.170283).
# Previous review (d8217e4476) global_view_models_scripting_test: frame 0, op 54 (transform), field tx: expected 500, got 482.14285
# Previous review (d8217e4476) hittest_ab1_grand_parent: frame 0, op 149 (transform), field ty: expected 100, got 31.04248
# Previous review (d8217e4476) hittest_ab1_parent: frame 0, op 141 (transform), field tx: expected 238.65234, got 184.80838
# Previous review (d8217e4476) reset_phase_multi_main: frame 0, op 69 (addRawPath), field point: expected (62, -0.0 (0x80000000)), got (60, -0.0 (0x80000000))
# Previous review (8d8025fd4a) component_list_child_origin: frame 1, op 546 (addRawPath), field point: expected (309.3333, 250), got (309.33334, 312); now exact with C++, since a translated ScrollConstraint's Fill content box has a zero automatic minimum on its scroll axis
REVIEWED_LAYOUT_DIVERGENCES: dict[str, str] = {'animated_clipping-layout': 'frame 0, op 34 (addRawPath), field point: expected (409.5, 0), got '
                             '(273, 0)',
 'hittest_ab_text_parent': 'frame 0, op 189 (transform), field ty: expected 358.17838, got 333.33334',
 'list_focus_order': 'frame 0, op 67 (transform), field ty: expected 362.79947, got 333.33334',
 'list_items': 'frame 0, op 48 (transform), field tx: expected 317.5, got 250',
 'superbowl': 'frame 0, op 3208 (transform), field ty: expected 184.33334, got 225',
 'artboard_list_overrides_vertical': 'frame 0, op 73 (addRawPath), field point: expected (500, 0), '
                                     'got (700, 0)',
 'bankcard': 'frame 1, op 711 (addRawPath), field point: expected (810, 0), got (826.94385, 0)',
 'data_viz_demo': 'frame 0, op 1289 (makeLinearGradient), field end: expected (276.66666, '
                  '16.325203), got (268.33334, 16.325203)',
 'db_health_tracker': 'frame 0, op 4764 (addRawPath), field point: expected (968, 0), got (10134, '
                      '0)',
 'global_view_models_scripting_test': 'frame 0, op 49 (addRawPath), field point: expected (945, '
                                      '0), got (980, 0)',
 'hittest_ab1_grand_parent': 'frame 0, op 149 (transform), field ty: expected 100, got 0',
 'hittest_ab1_parent': 'frame 0, op 129 (addRawPath), field point: expected (572, 0), got '
                       '(738.65234, 0)',
 'number_to_list_nested_children': 'frame 0, op 190 (addRawPath), field point: expected (250, '
                                   '59.791016), got (250, 61)',
 'reset_phase_multi_main': 'frame 0, op 57 (addRawPath), field point: expected (180, 0), got (186, '
                           '0)',
 'scroll_threshold-all-scroll': 'frame 0, op 90 (transform), field tx: expected 500, got 375'}

# 2026-10-27: refresh four pre-6771fdad5 streams after a full operation audit.
# The 84d5cde02e + upstream5551696a pool capture reproduces each old seal exactly.
# data_viz_demo f82bc7c5 and global_view_models_scripting_test 301407ac lose only
# 11 and 1 redundant path-reset/update pairs respectively, identical to upstream.
# list_focus_order 5258c290 and number_to_list_nested_children eb7c6220 reorder
# paint initialization at exactly the same 10 and 123 positions as upstream;
# all other operation bytes and all draw-time colors remain unchanged.
# Existing CSS first differences, historical exact IDs and floor stay unchanged.
REVIEWED_LAYOUT_STREAM_SHA256: dict[str, str] = {'animated_clipping-layout': 'e00881c43f3f84f8d7d9c32ae2a6aad794a7d62e5681b6ce8023962ee1d0cb7d',
 'hittest_ab_text_parent': '8be660a737ee025a5030acf941de273a610986bfb33c7ee90cdf504983b0dcd7',
 'list_focus_order': '712562fd43a862ec29558776d7b38f363b6c1547dea8ddca0d4756276e7e20f7',
 'list_items': '92388e8df708294c6d99b0a043d9a41f909648a9d70ac8d8a8b0fe69d453686d',
 'superbowl': '180ea9a1137368605e133d43ba8a23c744ebaeb3a6707cea7a88b24fa8986f52',
 'artboard_list_overrides_vertical': '3dd7323dcd6b9c4cdb6aedd64ba15f00465bd9e76310ddac90a42df3542b1631',
 'data_viz_demo': '191787e4b37e2eb0c93787aa8fe41517e984c7d9debfd5cd1b5dd0802aa7cf3d',
 'db_health_tracker': 'f384916c59fd4cad91674f5b8d3e212bbc2a99ae976c291bbbd4a36f4e6d78af',
 'global_view_models_scripting_test': '45c846f6453bb01f673d085d1d0ece851b2a935b6d8ef82ba564c641cecffb2d',
 'hittest_ab1_grand_parent': 'aab2993936f2b1df51a30cc49504bd92f23fef02cd0010badf6e32fe5dc95cfb',
 'hittest_ab1_parent': '3a03f5dbf20a07e0244612a4827eec609abf1af5993a91a7dbe608f0c68ad61f',
 'number_to_list_nested_children': '7b9acf7c63a244a80db1b1295b5d82d9078a27dfc9359ebc9b61d20993d79ed3',
 'reset_phase_multi_main': '2f9c57fd492cf31d67bee2e0225a59cbbd25800a72cbd91377f2ff030d61e3db',
 'scroll_threshold-all-scroll': 'b0a71d6e33d0c2cc4b3612190c4fee833f2b11505dfe37b4e833b5fc1453b05f'}

# Original bankcard hash 86f2d351 reproduces on macos-aarch64 at 704761ef16.
# Linux differs only in platform float bytes. Both current streams also include
# upstream 6c65f385's faithfully translated opacity initialization ordering.
# Select exactly the running target's full stream; never accept either hash.
REVIEWED_LAYOUT_STREAM_SHA256_BY_TARGET: dict[str, dict[str, str]] = {
    "bankcard": {
        "linux-x86_64": "b131c276bb5a6a7959ad2eb03dd45d6b5aeda42e9e587fdc846d27a153b77575",
        "macos-aarch64": "e385b79ca0968a6be70e024a1eb2e5bdd7580169cad248be223b8f5f88501c02",
    },
}

LAYOUT_DIVERGENCES.update({'hittest_ab_text_parent': 'Fill is CSS flex: 1 1 0%. In the nested ab-text artboard the three Fill rows of the 500-point column are 166.67 points each whatever their content, where C++ sizes them from their content first, so the third row starts at y 333.33 instead of 358.18.',
 'list_focus_order': 'Fill is CSS flex: 1 1 0%. The three Fill rows of the 500-point column are 166.67 points each whatever their content, where C++ sizes them from their content first (137.20, 225.60 and 137.20), so the last row starts at y 333.33 instead of 362.80.',
 'list_items': 'Fill is CSS flex: 1 1 0%. Once the actions append a list item, the two Fill columns that hold the lists still share the 500-point row equally at 250 points, where C++ sizes each from its list first, so the second column starts at x 250 instead of 317.5.',
 'superbowl': 'Fill is CSS flex: 1 1 0%. The three Fill rows of the 216-point panel take 61, 61 and 94 points: equal shares of 72, except the last row, whose 94-point content is its automatic minimum. C++ sizes them from their content first (40.67, 40.67 and 134.67), so content in the last row moves from y 184.33 to 225.',
 'animated_clipping-layout': 'CSS flex-shrink 1 lets the Fixed 409.5-wide nested box fit its '
                             '273-wide host instead of extending beyond it.',
 'artboard_list_overrides_vertical': "The list's seven child artboards keep their definite "
                                     '100-point widths as automatic minima, so the list content is '
                                     "700 points wide instead of C++'s 500 and its path runs to x "
                                     '700.',
 'bankcard': 'In the card artboard the NoWrap card number has a 740.94-point min-content line. Its '
             'box and the card root keep CSS automatic minima, so the root is 866.94 points wide '
             'instead of the 850-point artboard and its clip path runs to x 826.94 instead of 810. '
             'C++ keeps the number row at the 724-point content width.',
 'data_viz_demo': 'The two 50-percent chart cells in the 772-point row have 34-point padding edges '
                  'and a 20-point gap. Their 352-point bases shrink to (772 - 68 - 20)/2 = 342. '
                  'The LineBars gradient endpoint is five sixths of the cell width after 20 '
                  'padding, changing from 276.66666 to 268.33334.',
 'db_health_tracker': "The panel's CSS automatic minimum includes its definite 10000-point child "
                      'artboard and the 174 points around it, so the panel is 10174 points wide '
                      'instead of the 1008-point offer that C++ and main keep. Its rounded path '
                      'runs to x 10134 instead of 968. A bar box that its background shape held '
                      'at 51 points is 20 points high again, as in C++, since a shape that layout '
                      'resizes is the box\'s paint and gives it no automatic minimum.',
 'global_view_models_scripting_test': 'The mounted Main and GlobalName artboards keep their '
                                      'definite 500 and 480-point widths as automatic minima, so '
                                      'their parent is 980 points wide instead of the 945-point '
                                      'offer. C++ keeps 945 and lets them overflow.',
 'hittest_ab1_grand_parent': 'In the 208.60547-point grand-parent clipper the nested artboard '
                             'keeps its definite 572-point height as its automatic minimum, so CSS '
                             'flex-shrink 1 takes the overflow from the empty Fixed 100-point '
                             'item, which shrinks to zero. The following origin moves from 100 to '
                             '0.',
 'hittest_ab1_parent': 'Parent-clipper keeps the automatic minimum of its definite 238.65234 and '
                       '500-point children, so it is 738.65234 points wide instead of the '
                       '572-point offer that C++ keeps.',
 'number_to_list_nested_children': "The list box's CSS automatic minimum includes its definite "
                                   '61-point child artboards, so it stays 61 points high instead '
                                   'of the 59.79 points left under the header. C++ lets the '
                                   '61-point items overflow.',
 'reset_phase_multi_main': "The parent's CSS automatic minimum includes its three child artboards' "
                           'definite 62-point widths, so it is 186 points wide instead of the '
                           '180-point offer. The children keep 62 as in C++.',
 'scroll_threshold-all-scroll': 'Overflow mapping keeps the viewport at 500 and the content at '
                                '1500. Inside the content row, Fixed bases 500 and 1500 share 1500 '
                                'by CSS shrink weights, becoming 375 and 1125. Scrolling keeps its '
                                '1000-point range; every threshold-drag frame remains compared.'})


def reviewed_divergence(silver_id: str) -> str | None:
    return REVIEWED_LAYOUT_DIVERGENCES.get(silver_id, DIVERGENCES.get(silver_id))


def layout_divergence_note(silver_id: str, difference: str) -> str | None:
    reason = LAYOUT_DIVERGENCES.get(silver_id)
    if reason is None:
        return None
    return (
        "Layout divergence (stock Taffy with CSS semantics replaces Rive's Yoga): "
        f"{reason.rstrip('.')}; first difference: {difference}."
    )

PROVENANCE_UNKNOWN = {
    "interpolator": (
        "Public history contains no producer reference. Added by e6b7fd58066576edfaf414ebf5eaa865063a0aed "
        "(private source pin a1966ba517b9c886bd56cba961892df348787100); unchanged Git blob "
        "a2c83bae8d85520f399f48485986f8bf9cf5599e, 67 bytes, SHA-256 "
        "0d1dfa8e41fd60b1ebb625f644823910c432a53ef9e2937574e72bd542a1b180. "
        "That commit adds the separately named viewmodel_from_instance producer, not this baseline. "
        "No fixture or replay may be inferred from the filename or serialized operations."
    ),
    "multitouch_debug": (
        "Public history contains no producer reference. Added by a3cfedf2cceef41b6cd6851563f4f5f16ea499dd; "
        "unchanged Git blob 3b6203cfec5817cc97e738237ab351abd2922876, 723 bytes, SHA-256 "
        "cf6b08f9f374df5a02a001a78e4d5d887bd63c90e1f7cf4814125d174a34e5e0. "
        "That commit adds the separately named remove_from_list producer, not this baseline. "
        "No fixture or replay may be inferred from the filename or serialized operations."
    ),
}
CLASSIFIED_RUNTIME_BLOCKERS = {}
EXACT = (
    "stateful_artboard_swap",
    "stateful_source_switch",
    "stateful_list_props_lifecycle",
    "transition_self_comparator_test",
    "multi_listeners-rebind",
    "rebind_with_nested_viewmodel",
    "replace_vm_instance",
    "replace_vm_instance-list",
    "replace_vm_instance-double-nest",
    "artboard_opacity_and_transform_test",
    "image_computed_transform_bind",
    "solo_index_test",
    "data_bind_blob_test",
    "image_binding_with_listener",
    "stateful_component_image_test",
    "data_binding_artboards_test",
    "bindable_artboard_nesty",
    "bidirectional_binding_source",
    "additive_blendmode_test",
    "data_viz_demo",
    "db_health_tracker",
    "echo_show_demo",
    "focus_traversal_click_to_focus",
    "focus_traversal_data_bound",
    "text_layout_pre_7_3",
    "text_layout_7_3",
    "gamepad_inputs_test",
    "gamepad_inputs_test-collapsing",
    "layout_order_pointer_test",
    "layout_animation_transition_test",
    "text_background_feather_test",
    "joystick_databound_keyframe_test",
    "text_fit_test",
    "ik_anim_test",
    "ai_assitant",
    "artboard_list_overrides_horizontal",
    "artboard_list_overrides_vertical",
    "bankcard",
    "clear_viewmodel_list",
    "color_passthrough_test",
    "collapsable_data_binding",
    "collapse_data_binds-test_2",
    "collapsing_elements",
    "component_list_child_origin",
    "component_list_grouped",
    "component_list_hit_order",
    "component_list_virtualized_scroll_manual",
    "computed_values_test",
    "databind_viewmodel",
    "drag_event",
    "draw_index_list",
    "interactive_scrolling",
    "interpolate_to_end",
    "formula_random-always",
    "formula_random-once",
    "formula_random-source_change",
    "layout_anim_bound",
    "layout_anim_component_list",
    "layout_anim_nested",
    "layout_display",
    "layout_fixed_fill",
    "layout_paint",
    "layout_scroll_drag_multiplier_layouts",
    "layout_scroll_drag_multiplier_list",
    "layout_scroll_drag_multiplier_virtualized",
    "layout_scroll_snap_padding_layouts",
    "layout_scroll_snap_padding_list",
    "layout_scroll_snap_padding_virtualized",
    "layout_scroll_snap_carousel",
    "layout_scroll_visibility",
    "nested_artboard_quantize_and_speed",
    "number_to_list_nested_children",
    "reset_phase_multi_main",
    "scroll_intent",
    "scroll_test",
    "scroll_threshold-all-scroll",
    "scroll_threshold-horizontal-scroll",
    "scroll_threshold-vertical-scroll",
    "spotify_kids_demo",
    "stateful_multi_property",
    "stateful_nested",
    "superbowl",
    "text_feather_falloff",
    "text_input",
    "text_vertical_trim_test",
    "transition_artboard_condition_test",
    "virtualized_artboard_databound_children",
    "virtualize_blendmode",
    "word_joiner_test",
    "hittest_ab_text_parent",
    "lock_icon_demo",
    "text_listener_simpler",
    "collapse_data_binds-test_3",
    "databind_solo_to_enum",
    "layout_solos",
    "layout_solos_fit_to_layout_parent",
    "collapsed_databinds_test",
    "solo_nested_artboard_leaf_no_solo",
    "solo_nested_artboard_leaf_fits_parent_layout",
    "solo_nested_artboard_leaf_solo",
    "listener_view_model",
    "viewmodel_image_reset",
    "zero_width_space_line_break",
    "fit_font_size_test",
    "fit_font_size_hug_test",
    "advance_blend_mode-inputs",
    "advance_blend_mode-vms",
    "animated_clipping-layout",
    "animated_clipping-nodes",
    "artboard_list_map_rules",
    "artboard_width_test",
    "bidirectional_precedence-source_first",
    "bidirectional_precedence-target_first",
    "bidirectional_stateful_property",
    "clipping_and_draw_order",
    "component_based_conditions",
    "component_based_conditions-Artboard2",
    "component_list_follow_path_distance",
    "component_list_follow_path",
    "component_stateful_vm_instance",
    "component_stateful_vm_instance_2",
    "data_bind_font_test",
    "data_bind_artboard_input",
    "data_binding_artboards_default_test",
    "data_binding_artboards_test_recursive",
    "databind_external_artboard_main",
    "component_stateful",
    "custom_property_trigger_bind",
    "computed_root_transform-nested_artboard",
    "custom_property_enum",
    "data_bind_solo-values-to-solos",
    "databind_artboard",
    "event_trigger_event",
    "fill_trim_path",
    "focus_test",
    "focus_traversal",
    "focusable_element",
    "follow_path_animate_shape",
    "follow_path_animate_solo",
    "follow_path_animate_target",
    "follow_path_constraint",
    "format_number_with_commas",
    "global_viewmodels_test-auto_instance",
    "group_effect",
    "group_effect-main-missing-targets",
    "hittest_ab1",
    "hittest_ab1_grand_parent",
    "hittest_ab1_parent",
    "hide_test",
    "juice",
    "hittest_collapsed_layouts",
    "hittest_nested",
    "image_fit_alignment_2",
    "layout_grid_stack_grid_with_layouts",
    "layout_grid_stack_stack_with_layouts",
    "layout_grid_stack_grid_with_layouts_size_changing",
    "layout_hug_artboard",
    "image_fit_alignment_3",
    "image_fit_alignment_updated_test",
    "list_items",
    "list_to_length_test",
    "list_to_path",
    "multi_listeners",
    "multitouch",
    "multitouch_enter",
    "n_slice_triangle",
    "nested_artboard_origin_override_test",
    "nested_events",
    "nested_hug",
    "nested_needs_advance",
    "pause_nested_artboard",
    "recursive_data_bind",
    "relative_data_bind_path",
    "relative_data_binding",
    "saturation",
    "scripted_animated_oval",
    "scripted_oval",
    "sorted_listeners",
    "spotify_kids_app_icon",
    "stacked_path_effects",
    "state_transition_fire_trigger",
    "stateful_keyed_trigger",
    "target_event",
    "text_follow_path_shape_length",
    "text_stroke_test",
    "transition_actions",
    "transition_duration_bind_list",
    "transition_duration_bind_nested",
    "transition_index_condition",
    "trigger_based_listeners",
    "trigger_fires_single_change",
    "vertical_align_ellipsis",
    "viewmodel_list_trigger",
    "viewmodel_based_condition",
    "computed_root_transform-list",
    "data_bind_keyframes_test",
    "data_bind_solo-solos-to-values",
    "data_converter_interpolator_reset",
    "deterministic_mode",
    "focus_collapsing",
    "gamepad_test",
    "global_variables_test",
    "global_view_models_scripting_test",
    "global_viewmodels_test-set_instance",
    "hittest_ab_2_non_virtualized",
    "hittest_ab_2_virtualized",
    "hittest_ab_shape_parent",
    "image_fit_alignment",
    "interpolation_zero_duration",
    "keyboard_listener",
    "keyboard_listener-KeyboardInput",
    "layout_grid_stack_grid_with_layouts_size_span_changing",
    "layout_grid_stack_grid_with_layouts_span",
    "list_focus_order",
    "multitouch_enter-MainList",
    "multitouch_enter-MultiScroll",
    "pointer_exit",
    "paused_nested_artboard_opacity",
    "path_effect_with_feathers",
    "relative_data_bind_path-fire-trigger",
    "relative_data_bind_path-listener",
    "relative_data_bind_path-scripted-input",
    "time_based_interpolation",
    "unbound_stateful_component",
)

SCRIPTED_EXACT_NOTES = {
    "scripted_text_wrapped": (
        "The literal native Luau text layout/draw case in "
        "crates/nuxie-scripting/tests/upstream_scripting_text.rs passes the "
        "pinned SRIV comparison and renderer balance assertion."
    ),
    "scripted_text_bidi": (
        "The literal native Luau mixed-direction text case in "
        "crates/nuxie-scripting/tests/upstream_scripting_text.rs passes the "
        "pinned SRIV comparison, visual-order assertions and renderer balance."
    ),
    "scripted_text_glyphs": (
        "The literal native Luau placed-glyph outline case in "
        "crates/nuxie-scripting/tests/upstream_scripting_text.rs passes the "
        "pinned SRIV comparison and renderer balance assertion."
    ),
    "scripted_path_effect_clip": (
        "The literal upstream initial draw plus 60 frame advances is ported in "
        "crates/nuxie-runtime/tests/upstream_scripting_path_effect.rs "
        "(a_clip_follows_the_path_effect_on_its_sources_fill) and passes the "
        "pinned SRIV comparison, including cached fill-effect clip updates."
    ),
    "script_create_text_runs": (
        "The pinned C++ action sequence is active and operation-exact across authored "
        "initialization plus push, swap, remove, insert, pop, and shift list mutations. "
        "The paired-owner correction restores ScriptedViewModel.new construction and "
        "DataBindContextValueList::apply -> Text::updateList shape invalidation."
    ),
    "scripted_animated_oval": (
        "The literal 1000-frame script loop, per-frame balance and Lua stack checks, "
        "full GC, and renderer stream are operation-exact with the pinned C++ silver "
        "baseline."
    ),
    "scripted_oval": (
        "The literal scripted oval draw and renderer stream are operation-exact with "
        "the pinned C++ silver baseline."
    ),
}

SCRIPTED_REPLAY_IDS = ('image_scripting_property_value', 'list_index_script_access', 'listener_action_inputs', 'remove_from_list', 'replace_view_model', 'reset_shared_viewmodel_instance_test', 'reuse_path_in_effect', 'script_affects_has_changed', 'script_artboards', 'script_artboards_opacity', 'script_artboards_origin', 'script_color_data_test', 'script_converter_with_dependency', 'script_converter_with_dependency_2', 'script_converter_with_dependency_with_library', 'script_converter_with_dependency_with_library_with_update', 'script_create_text_runs', 'script_create_viewmodel_instance', 'script_input_color_trigger', 'script_layout_grid', 'script_namespace_test', 'script_path_effects', 'script_path_opacity', 'script_paths', 'script_string_converter', 'scripted_animated_oval', 'scripted_artboard_render', 'scripted_as_path', 'scripted_data_context', 'scripted_data_converter_bound_input', 'scripted_listener_action', 'scripted_oval', 'scripted_path_effect_clip', 'scripted_property_image', 'scripted_text_bidi', 'scripted_text_glyphs', 'scripted_text_wrapped', 'scripted_transition_condition', 'scripted_viewmodel_cache', 'scripting_linear_animation', 'scripting_root_viewmodel', 'viewmodel_access', 'viewmodel_from_context', 'viewmodel_from_instance', 'viewmodel_instance_to_artboard')


EXACT = tuple(dict.fromkeys((*EXACT, *SCRIPTED_REPLAY_IDS)))

REVIEWED_LAYOUT_DIVERGENCES["list_index_script_access"] = "frame 0, op 70 (transform), field ty: expected 369.66666, got 333.33334"
REVIEWED_LAYOUT_STREAM_SHA256["list_index_script_access"] = "e52f737b071059b5fd95e8f63679fdad4ab81026aa35d4d93152f9865226ff45"

# Existing direct scripted-test SRIV seals; do not substitute golden stream hashes.
for _id, _operation, _hash in (
    ("script_create_text_runs", 109, "2147653268787fc95d1cce0ef3e758a2956e722e383b4cd0c463069ee8437644"),
    ("script_create_viewmodel_instance", 126, "52b2e4340ef34ed2042e28176ce1f01304376280fba8a508467eef66c50434f6"),
):
    REVIEWED_LAYOUT_DIVERGENCES[_id] = f"frame 0, op {_operation} (transform), field tx: expected 609.83514, got 601.51874"
    REVIEWED_LAYOUT_STREAM_SHA256[_id] = _hash
    LAYOUT_DIVERGENCES[_id] = "CSS Fill uses 0% bases: five buttons in the 749.90-point row share equal 140.38 widths regardless of label; C++ distributes free space after content widths"

def data_binding_viewmodel_actions(
    silver_id: str,
) -> tuple[dict[str, object], ...] | None:
    if silver_id != "databind_viewmodel":
        return None
    return (
        {"kind": "create-default-view-model"},
        {"kind": "bind-prepared-view-model"},
        {"kind": "advance", "target": "state-machine", "seconds": 0.016},
        {"kind": "draw"},
        {"kind": "frame"},
        {"kind": "create-named-view-model", "view_model": "StatefulChild"},
        {
            "kind": "set-named-view-model-number",
            "view_model": "StatefulChild",
            "property": "num",
            "value": 44.0,
        },
        {
            "kind": "replace-view-model",
            "property": "statefulChild",
            "view_model": "StatefulChild",
        },
        {"kind": "advance", "target": "state-machine", "seconds": 0.016},
        {"kind": "draw"},
        {"kind": "frame"},
        {
            "kind": "set-named-view-model-number",
            "view_model": "StatefulChild",
            "property": "num",
            "value": 44.0,
        },
        {"kind": "advance", "target": "state-machine", "seconds": 0.016},
        {"kind": "draw"},
        {"kind": "frame"},
        {"kind": "pointer-down", "x": 25.0, "y": 25.0, "pointer_id": 0},
        {"kind": "pointer-up", "x": 25.0, "y": 25.0, "pointer_id": 0},
        {"kind": "advance", "target": "state-machine", "seconds": 0.016},
        {"kind": "draw"},
    )


def fl_e8_list_path_actions(silver_id: str) -> tuple[dict[str, object], ...] | None:
    """The eight named phases from data_binding_test.cpp's ListPath producer."""

    if silver_id != "list_to_path":
        return None

    def advance_draw(frame: bool = True) -> list[dict[str, object]]:
        actions: list[dict[str, object]] = [
            {"kind": "advance", "target": "state-machine", "seconds": 0.0},
            {"kind": "advance", "target": "state-machine", "seconds": 0.016},
            {"kind": "draw"},
        ]
        if frame:
            actions.append({"kind": "frame"})
        return actions

    def append(
        view_model: str,
        numbers: dict[str, float],
        index: int | None = None,
    ) -> dict[str, object]:
        action: dict[str, object] = {
            "kind": "append-view-model-list-item",
            "list": "lis",
            "view_model": view_model,
            "number_properties": numbers,
        }
        if index is not None:
            action["index"] = index
        return action

    actions: list[dict[str, object]] = [
        {"kind": "create-default-view-model"},
        {"kind": "bind-prepared-view-model"},
    ]
    actions += advance_draw()

    # D-LP-XY
    actions += [
        append("vertex-x-y", {}),
        append("vertex-x-y", {"x": 100.0}),
        append("vertex-x-y", {"x": 100.0, "y": 100.0}),
        append("vertex-x-y", {"y": 100.0}),
    ]
    actions += advance_draw()

    # D-LP-RD
    actions.append(
        append(
            "vertex-rotation-distance",
            {"x": 200.0, "rotation": 1.5, "distance": 20.0},
            2,
        )
    )
    actions += advance_draw()

    # D-LP-DETACHED
    actions.append(
        append(
            "vertex-detached",
            {
                "x": 200.0,
                "y": 100.0,
                "inRotation": 1.0,
                "outRotation": 2.0,
                "inDistance": 10.0,
                "outDistance": 30.0,
            },
            3,
        )
    )
    actions += advance_draw()

    # D-LP-POINT
    actions.append(
        append(
            "vertex-in-out",
            {
                "x": 100.0,
                "y": 200.0,
                "inX": 40.0,
                "inY": 20.0,
                "outX": 10.0,
                "outY": 30.0,
            },
            4,
        )
    )
    actions += advance_draw()

    # D-LP-INVALID
    actions.append(append("non-vertex", {}, 5))
    actions += advance_draw()

    # D-LP-PARTIAL
    actions.append(
        append(
            "vertex-incomplete",
            {
                "x": 100.0,
                "y": 300.0,
                "inDistance": 60.0,
                "inRotation": -1.0,
                "outX": 30.0,
                "inX": -30.0,
            },
            4,
        )
    )
    actions += advance_draw()

    def set_item(index: int, property_name: str, value: float) -> dict[str, object]:
        return {
            "kind": "set-view-model-list-item-number",
            "list": "lis",
            "index": index,
            "property": property_name,
            "value": value,
        }

    # D-LP-LIVE: the equality no-op plus five overlapping live sources.
    actions += [
        set_item(4, "inX", -30.0),
        set_item(0, "x", 50.0),
        set_item(2, "rotation", 1.0),
        set_item(3, "inDistance", 30.0),
        set_item(5, "outY", 40.0),
    ]
    actions += advance_draw(frame=False)
    for frame in range(60):
        actions += [
            {"kind": "frame"},
            set_item(4, "inRotation", float(frame * 6)),
            set_item(2, "rotation", float(frame * 6)),
            {"kind": "advance", "target": "state-machine", "seconds": 0.01},
            {"kind": "advance", "target": "state-machine", "seconds": 0.0},
            {"kind": "draw"},
        ]
    return tuple(actions)


def fl_d4_actions(silver_id: str) -> tuple[dict[str, object], ...] | None:
    """Literal action ports for the mutable DataBind tests owned by FL-D4."""

    create = {"kind": "create-default-view-model"}
    bind = {"kind": "bind-prepared-view-model"}

    if silver_id == "viewmodel_list_trigger":
        actions = [
            create,
            bind,
            {"kind": "advance", "target": "state-machine", "seconds": 0.1},
            {"kind": "draw"},
            {
                "kind": "fire-view-model-list-item-trigger",
                "list": "lis",
                "index": 0,
                "trigger": "tri",
            },
        ]
        actions += [
            action
            for _ in range(4)
            for action in (
                {"kind": "frame"},
                {
                    "kind": "fire-view-model-list-item-trigger",
                    "list": "lis",
                    "index": 0,
                    "trigger": "tri",
                },
                {"kind": "advance", "target": "state-machine", "seconds": 0.064},
                {"kind": "draw"},
            )
        ]
        return tuple(actions)

    if silver_id == "list_items":
        return (
            create,
            {
                "kind": "append-view-model-list-item",
                "list": "lis1",
                "view_model": "child",
                "string_property": "label",
                "string_value": "test",
            },
            bind,
            {"kind": "advance", "target": "state-machine", "seconds": 0.1},
            {"kind": "draw"},
            {"kind": "frame"},
            {"kind": "remove-view-model-list-item", "list": "lis1", "index": 0},
            {"kind": "advance", "target": "state-machine", "seconds": 0.1},
            {"kind": "draw"},
        )

    if silver_id == "list_to_length_test":
        actions = [
            create,
            bind,
            {"kind": "advance", "target": "state-machine", "seconds": 0.1},
            {"kind": "draw"},
        ]
        actions += [
            action
            for _ in range(4)
            for action in (
                {"kind": "frame"},
                {
                    "kind": "append-view-model-list-item",
                    "list": "lis",
                    "view_model": "child",
                },
                {"kind": "advance", "target": "state-machine", "seconds": 0.1},
                {"kind": "advance", "target": "state-machine", "seconds": 0.1},
                {"kind": "draw"},
            )
        ]
        return tuple(actions)

    if silver_id == "data_converter_interpolator_reset":
        actions = []
        for final_color, final_number in ((0xFF00FF00, 500.0), (0xFF0000FF, 0.0)):
            if actions:
                actions.append({"kind": "frame"})
            actions += [
                create,
                {"kind": "set-view-model-number", "property": "xPos", "value": 250.0},
                {"kind": "set-view-model-color", "property": "col", "value": 0xFFFF0000},
                bind,
                {"kind": "advance", "target": "state-machine", "seconds": 0.1},
                {"kind": "draw"},
                {
                    "kind": "set-view-model-color",
                    "property": "col",
                    "value": final_color,
                },
                {
                    "kind": "set-view-model-number",
                    "property": "xPos",
                    "value": final_number,
                },
            ]
            actions += [
                action
                for _ in range(62)
                for action in (
                    {"kind": "frame"},
                    {"kind": "advance", "target": "state-machine", "seconds": 0.016},
                    {"kind": "draw"},
                )
            ]
        return tuple(actions)

    if silver_id == "interpolation_zero_duration":
        actions = [
            create,
            bind,
            {"kind": "advance", "target": "state-machine", "seconds": 0.1},
            {"kind": "draw"},
            {"kind": "set-view-model-number", "property": "objectX", "value": 200.0},
        ]
        actions += [
            action
            for _ in range(15)
            for action in (
                {"kind": "frame"},
                {"kind": "advance", "target": "state-machine", "seconds": 0.1},
                {"kind": "draw"},
            )
        ]
        for duration, target in ((0.0, 400.0), (1.0, 200.0)):
            actions += [
                {
                    "kind": "set-view-model-number",
                    "property": "interpValue",
                    "value": duration,
                },
                {"kind": "advance", "target": "state-machine", "seconds": 0.016},
                {
                    "kind": "set-view-model-number",
                    "property": "objectX",
                    "value": target,
                },
                {"kind": "advance", "target": "state-machine", "seconds": 0.016},
            ]
            actions += [
                action
                for _ in range(15)
                for action in (
                    {"kind": "frame"},
                    {"kind": "advance", "target": "state-machine", "seconds": 0.1},
                    {"kind": "draw"},
                )
            ]
        return tuple(actions)

    return None


def repeated_frames(count: int, seconds: float) -> list[dict[str, object]]:
    return [
        item
        for _ in range(count)
        for item in (
            action("frame"),
            action("advance", target="state-machine", seconds=seconds),
            action("draw"),
        )
    ]


def focus_flags_actions(silver_id: str) -> tuple[dict[str, object], ...] | None:
    """Literal 9b9cd7b1 focus producers, including focus-mutating assertion walks."""
    if silver_id not in ("focus_traversal_click_to_focus", "focus_traversal_data_bound"):
        return None
    advance = action("advance", target="state-machine", seconds=0.016)
    draw = action("draw")
    frame = action("frame")
    clear = action("clear-focus")
    nested = [0, 0]
    parents = [[1, 0, row, 0] for row in range(3)]
    children = [nested + [0], nested + [1]] + [p + [i] for p in parents for i in range(2)]
    actions = [action("bind-default-view-model")]

    def boolean(path: str, value: bool, row: int | None = None) -> None:
        if row is None:
            actions.append(action("set-view-model-boolean", property=path, value=value))
        else:
            actions.append(action("set-view-model-list-item-boolean", list="layoutChildren", index=row, property=path, value=value))

    def leaf(child: str, focusable: bool, traversable: bool, row: int | None = None) -> None:
        boolean(child + "/focusable", focusable, row)
        boolean(child + "/traversable", traversable, row)

    def settle() -> None:
        actions.extend([advance, advance, draw, frame])

    def expect(path: list[int] | None) -> None:
        actions.append(action("assert-focus-node", **({"path": path} if path is not None else {})))

    def select(path: list[int]) -> None:
        actions.append(action("set-focus-node", path=path))

    def click(path: list[int], expected: list[int]) -> None:
        actions.append(action("click-focus-node", path=path))
        settle()
        expect(expected)

    def order(paths: list[list[int]], reverse: bool = False) -> None:
        actions.append(action("assert-focus-order", paths=list(reversed(paths)) if reverse else paths, reverse=reverse))

    if silver_id == "focus_traversal_click_to_focus":
        for child in ("node/child1", "node/child2"):
            leaf(child, True, True)
        for row in range(3):
            for child in ("child1", "child2"):
                leaf(child, True, True, row)
        actions.extend([advance, advance])
        for child in children:
            actions.append(clear)
            click(child, child)
        leaf("node/child2", False, True)
        actions.extend([advance, clear])
        select(children[1])
        expect(children[1])
        click(children[0], children[1])
        leaf("node/child2", True, True)
        actions.append(advance)
        click(children[0], children[0])
        leaf("child1", False, True, 1)
        actions.extend([advance, clear])
        select(children[7])
        click(children[5], children[7])
        click(children[3], children[3])
        leaf("child1", True, True, 1)
        actions.append(advance)
        click(children[5], children[5])
        leaf("node/child1", True, False)
        actions.extend([advance, clear])
        click(children[1], children[1])
        leaf("node/child1", False, False)
        actions.extend([advance, clear])
        select(children[0])
        click(children[1], children[0])
    else:
        actions.append(advance)
        baseline = [nested, *parents]
        settle()
        order(baseline)
        order(baseline, True)
        boolean("node/child1/focusable", True)
        settle()
        order(baseline)
        select(children[1])
        expect(children[1])
        boolean("node/child1/focusable", False)
        boolean("node/child1/traversable", True)
        settle()
        order(baseline)
        actions.append(clear)
        select(children[1])
        expect(None)
        boolean("node/child1/focusable", True)
        settle()
        order([nested, children[1], *parents])
        leaf("node/child2", True, True)
        settle()
        order([nested, *children[:2], *parents])
        order([nested, *children[:2], *parents], True)
        for child in ("child2", "child1"):
            leaf(child, True, True, 1)
        settle()
        order([nested, *children[:2], parents[0], parents[1], *children[4:6], parents[2]])
        for row in range(3):
            for child in ("child1", "child2"):
                leaf(child, True, True, row)
        settle()
        all_nodes = [nested, *children[:2]] + [n for row, p in enumerate(parents) for n in (p, *children[2 + row * 2:4 + row * 2])]
        order(all_nodes)
        order(all_nodes, True)
        for child in ("child1", "child2"):
            leaf(child, False, False, 0)
        settle()
        order([n for n in all_nodes if n not in children[2:4]])
    actions.extend([advance, draw])
    return tuple(actions)


def p1q_view_model_actions(
    silver_id: str,
) -> tuple[dict[str, object], ...] | None:
    """Literal action ports for P1-q view-model mutation producers."""

    bind = action("bind-default-view-model")
    advance = lambda seconds: action(
        "advance", target="state-machine", seconds=seconds
    )
    draw = action("draw")

    if silver_id == "component_stateful_vm_instance":
        actions = [bind, advance(0.1), draw]
        actions.append(action("set-view-model-number", property="h", value=200.0))
        actions += repeated_frames(30, 0.016)
        actions.append(action("set-view-model-number", property="h", value=50.0))
        actions += repeated_frames(30, 0.016)
        return tuple(actions)

    if silver_id == "component_stateful_vm_instance_2":
        actions = [bind, advance(0.1), draw]
        actions += repeated_frames(30, 0.016)
        actions.append(
            action("set-view-model-string", property="label", value="Override")
        )
        actions += repeated_frames(30, 0.016)
        return tuple(actions)

    if silver_id == "stateful_multi_property":
        actions = [bind, advance(0.1), draw]
        mutations = (
            ("set-view-model-number", "btn1Count", 180.0),
            ("set-view-model-color", "btn1Tint", 0xFFFF3344),
            ("set-view-model-string", "btn1Label", "One"),
            ("set-view-model-boolean", "btn1Clip", True),
            ("set-view-model-enum", "btn1Display", 1),
            ("set-view-model-number", "btn2Count", 60.0),
            ("set-view-model-color", "btn2Tint", 0xFF33AAFF),
            ("set-view-model-string", "btn2Label", "Two"),
            ("set-view-model-boolean", "btn2Clip", True),
            ("set-view-model-enum", "btn2Display", 1),
        )
        for kind, property_name, value in mutations:
            actions.append(action(kind, property=property_name, value=value))
            actions += repeated_frames(5, 0.016)
        return tuple(actions)

    if silver_id == "stateful_nested":
        actions = [bind, advance(0.1), draw]
        for kind, property_name, value in (
            ("set-view-model-string", "btn1Label", "One"),
            ("set-view-model-color", "btn1Tint", 0xFFFF3344),
            ("set-view-model-string", "btn2Label", "Two"),
            ("set-view-model-color", "btn2Tint", 0xFF33AAFF),
        ):
            actions.append(action(kind, property=property_name, value=value))
            actions += repeated_frames(5, 0.016)
        return tuple(actions)

    if silver_id in {
        "component_based_conditions",
        "component_based_conditions-Artboard2",
    }:
        actions = [action("bind-fresh-view-model"), advance(0.1), draw, action("frame")]
        if silver_id == "component_based_conditions":
            actions.append(
                action(
                    "set-view-model-number",
                    property="numberProperty",
                    value=1.5,
                )
            )
        else:
            actions.append(
                action("set-view-model-boolean", property="vmBool", value=True)
            )
        actions += [advance(0.016), draw]
        actions += repeated_frames(25, 0.1)
        return tuple(actions)

    if silver_id == "collapsable_data_binding":
        return (
            bind,
            advance(0.1),
            draw,
            action("frame"),
            action("set-view-model-color", property="col", value=0xFFFF0000),
            advance(0.1),
            draw,
            action("frame"),
            action("set-view-model-number", property="soloIndex", value=1.0),
            advance(0.1),
            draw,
            action("frame"),
            action("set-view-model-color", property="col", value=0xFF00FF00),
            action("set-view-model-number", property="soloIndex", value=0.0),
            advance(0.1),
            draw,
            action("frame"),
            action("set-view-model-number", property="soloIndex", value=1.0),
            advance(0.1),
            draw,
        )

    if silver_id == "joystick_databound_keyframe_test":
        actions = [bind, advance(0.016), draw]
        for index in range(5):
            actions += [
                action("frame"),
                action("pointer-move", x=f"artboard-width*{index}/5", y=250.0, seconds=0.0),
                advance(0.2),
                draw,
            ]
        return tuple(actions)

    if silver_id == "data_bind_keyframes_test":
        actions = [bind, advance(0.016), draw]
        actions += repeated_frames(5, 0.2)
        actions += [
            action(
                "set-view-model-string",
                property="keyfTextStart",
                value="updated--text",
            ),
            action("set-view-model-color", property="colorStart", value=0xFFFFFF00),
            action("set-view-model-number", property="startX", value=100.0),
        ]
        actions += repeated_frames(5, 0.2)
        return tuple(actions)

    if silver_id in {
        "bidirectional_precedence-source_first",
        "bidirectional_precedence-target_first",
    }:
        return (
            action("create-default-view-model"),
            action("set-view-model-number", property="x", value=100.0),
            action("set-view-model-number", property="y", value=100.0),
            action("bind-prepared-view-model"),
            advance(0.0),
            advance(0.016),
            draw,
        )

    if silver_id == "component_list_follow_path":
        actions = [bind, advance(0.1), draw]
        actions += repeated_frames(30, 0.016)
        actions.append(
            action("set-view-model-number", property="ItemCount", value=5.0)
        )
        actions += repeated_frames(30, 0.016)
        return tuple(actions)

    if silver_id == "ai_assitant":
        actions = [bind, advance(0.1), draw]
        for index in range(3):
            actions += [
                action("frame"),
                action("set-view-model-number", property="left", value=index * 10.0),
                action("set-view-model-number", property="bottom", value=index * 5.0),
                action("set-view-model-number", property="top", value=index * 3.0),
                action("set-view-model-number", property="right", value=index * 2.0),
                advance(0.1),
                draw,
            ]
        return tuple(actions)

    if silver_id == "collapse_data_binds-test_3":
        return (
            advance(0.0),
            bind,
            advance(0.016),
            draw,
            action("frame"),
            action("set-view-model-enum", property="display_2", value=1),
            advance(0.016),
            draw,
            action("frame"),
            action("set-view-model-enum", property="display_1", value=1),
            advance(0.016),
            draw,
            action("frame"),
            action("set-view-model-enum", property="display_2", value=0),
            advance(0.016),
            draw,
            action("frame"),
            action("set-view-model-enum", property="display_1", value=0),
            advance(0.016),
            draw,
        )

    if silver_id == "collapsed_databinds_test":
        # solo_test.cpp at 074bfb13: exactly three draws. The eight .25s
        # advances are not individually drawn or announced as frames.
        return (
            action("frame-size"),
            action("select-state-machine"),
            bind,
            advance(0.0),
            draw,
            action("frame"),
            advance(0.016),
            draw,
            *(advance(0.25) for _ in range(8)),
            action("frame"),
            draw,
        )

    if silver_id in ("layout_solos", "layout_solos_fit_to_layout_parent"):
        return (
            action("frame-size"),
            *((action("opt-in-nested-leaves-to-layout-parent"),) if silver_id.endswith("fit_to_layout_parent") else ()),
            action("select-state-machine"),
            action("bind-selected-artboard-authored-view-model", if_present=False),
            action("render-view-model-enum-states", property="states", seconds=0.016),
        )

    if silver_id == "databind_solo_to_enum":
        return (
            bind,
            advance(0.0),
            draw,
            action("frame"),
            action("pointer-down", x=425.0, y=70.0, pointer_id=0),
            action("pointer-up", x=425.0, y=70.0, pointer_id=0),
            advance(0.016),
            draw,
        )

    if silver_id == "fit_font_size_hug_test":
        return (
            action("frame-size"),
            action("select-state-machine"),
            action("bind-selected-artboard-fresh-view-model"),
            advance(0.0),
            draw,
            action("frame"),
            advance(0.016),
            draw,
        )

    if silver_id == "fit_font_size_test":
        actions = [bind, advance(0.1), draw]
        for _ in range(6):
            actions += [
                action("frame"),
                action("fire-view-model-trigger", property="trigger"),
                advance(0.1),
                draw,
            ]
        return tuple(actions)

    if silver_id == "layout_fixed_fill":
        actions = [bind, advance(0.0), draw]
        actions.append(
            action("set-view-model-boolean", property="booleanProperty", value=True)
        )
        actions += repeated_frames(15, 0.016)
        return tuple(actions)

    if silver_id == "listener_view_model":
        return (
            bind,
            advance(0.0),
            draw,
            action("frame"),
            action("set-view-model-color", property="col", value=0x64000A0F),
            advance(0.016),
            draw,
            action("frame"),
            action("fire-view-model-trigger", property="tri"),
            advance(0.016),
            draw,
            action("frame"),
            action("set-view-model-number", property="num1", value=55.0),
            advance(0.016),
            draw,
        )

    if silver_id == "nested_artboard_quantize_and_speed":
        actions = [bind, advance(0.1), draw]
        actions += repeated_frames(62, 0.016)
        actions += [
            action("set-view-model-number", property="speed", value=4.0),
            action("set-view-model-number", property="quant", value=7.0),
            advance(0.016),
        ]
        actions += repeated_frames(62, 0.016)
        return tuple(actions)

    if silver_id == "viewmodel_image_reset":
        return (
            bind,
            advance(0.1),
            draw,
            action("set-view-model-asset", property="img", value=-1),
            action("frame"),
            advance(0.1),
            advance(0.1),
            draw,
        )

    if silver_id == "data_bind_font_test":
        return (
            bind,
            advance(0.0),
            draw,
            action("frame"),
            advance(0.016),
            draw,
            action("frame"),
            action(
                "set-view-model-font-bytes",
                property="fontProperty",
                source="kablammo.ttf",
            ),
            advance(0.016),
            draw,
            action("frame"),
            action("pointer-down", x=490.0, y=490.0, pointer_id=0),
            action("pointer-up", x=490.0, y=490.0, pointer_id=0),
            advance(0.016),
            draw,
            action("frame"),
            action("pointer-down", x=490.0, y=20.0, pointer_id=0),
            action("pointer-up", x=490.0, y=20.0, pointer_id=0),
            advance(0.016),
            draw,
        )

    if silver_id == "car_widgets_v01":
        actions = [
            bind,
            advance(0.1),
            draw,
            action(
                "set-view-model-number",
                property="COMPASS/Rotation",
                value=20.0,
            ),
            action(
                "set-view-model-number",
                property="TIRE PSI/FL Tyre",
                value=10.0,
            ),
        ]
        actions += repeated_frames(62, 0.016)
        return tuple(actions)

    if silver_id == "rewards_demo":
        actions = [
            bind,
            advance(0.1),
            draw,
            action("fire-view-model-trigger", property="Button/Pressed"),
        ]
        actions += repeated_frames(20, 0.1)
        return tuple(actions)

    if silver_id == "group_effect":
        actions = [
            bind,
            advance(0.0),
            draw,
            action("frame"),
            action("set-view-model-number", property="dashValue", value=10.0),
            action("advance", target="artboard", seconds=0.0),
            advance(0.0),
            draw,
        ]
        actions += repeated_frames(15, 0.064)
        return tuple(actions)

    if silver_id == "word_joiner_test":
        def insert_joiners(text: str, positions: range, copies: int = 1) -> str:
            for position in positions:
                text = text[:position] + "\u2060" * copies + text[position:]
            return text

        def set_all_text(text: str) -> list[dict[str, object]]:
            return [
                action("set-view-model-string", property=property_name, value=text)
                for property_name in ("txt1", "txt2", "txt3", "txt4")
            ]

        text = "123456789012345678901234567890"
        values = [text]
        text = insert_joiners(text, range(29, 20, -1))
        values.append(text)
        text = insert_joiners(text, range(20, 10, -1))
        values.append(text)

        long_lines = "1234567890" * 6 + "|\n" + "1234567890" * 6 + "\n"
        values.append(long_lines)
        values.append(insert_joiners(long_lines, range(50, 20, -1)))
        values.append(long_lines)
        values.append(insert_joiners(long_lines, range(50, 20, -1), copies=3))

        spaced_lines = (
            "1234567890 " * 5
            + "1234567890|\n"
            + "1234567890 " * 5
            + "1234567890\n"
        )
        values.append(spaced_lines)
        values.append(insert_joiners(spaced_lines, range(50, 20, -1), copies=3))

        actions = [bind, advance(0.1), draw]
        for value in values:
            actions.append(action("frame"))
            actions += set_all_text(value)
            actions += [advance(0.1), draw]
        return tuple(actions)

    if silver_id == "zero_width_space_line_break":
        return (
            bind,
            advance(0.1),
            draw,
            action("frame"),
            action(
                "set-view-model-string",
                property="txt",
                value="12345678901234567890",
            ),
            advance(0.1),
            draw,
            action("frame"),
            action(
                "set-view-model-string",
                property="txt",
                # C++ narrows 8203 to a single char before insertion.
                value="1234567890\u000b1234567890",
            ),
            advance(0.1),
            draw,
        )

    return None


def p1q_pointer_actions(silver_id: str) -> tuple[dict[str, object], ...] | None:
    """Expand pointer variables and loops from pinned C++ producers."""

    bind = action("bind-default-view-model")
    draw = action("draw")
    frame = action("frame")
    advance = lambda seconds: action(
        "advance", target="state-machine", seconds=seconds
    )
    pointer = lambda kind, x, y, pointer_id=0: action(
        kind,
        x=x,
        y=y,
        **(
            {"seconds": 0.0, "pointer_id": pointer_id}
            if kind == "pointer-move"
            else {"pointer_id": pointer_id}
        ),
    )

    if silver_id == "drag_event":
        actions = [
            bind,
            advance(0.1),
            draw,
            frame,
            pointer("pointer-down", 250.0, 250.0),
            pointer("pointer-up", 250.0, 250.0),
            advance(0.1),
            draw,
            frame,
            pointer("pointer-down", 250.0, 250.0),
            advance(0.1),
            draw,
        ]
        for coordinate in range(250, 50, -10):
            actions += [
                frame,
                pointer("pointer-move", float(coordinate), float(coordinate)),
                advance(0.1),
                draw,
            ]
        actions += [
            pointer("pointer-up", 50.0, 50.0),
            frame,
            pointer("pointer-down", 50.0, 50.0),
            advance(0.1),
            pointer("pointer-up", 50.0, 50.0),
            advance(0.1),
            draw,
        ]
        return tuple(actions)

    if silver_id == "pointer_exit":
        actions = [bind, advance(0.1), draw]
        for x, y in (
            [(float(value), 250.0) for value in range(100, 401, 30)]
            + [(float(value), 250.0) for value in range(500, 100, -30)]
            + [(240.0, float(value)) for value in range(500, 100, -30)]
        ):
            actions += [frame, pointer("pointer-move", x, y), advance(0.016), draw]
        return tuple(actions)

    if silver_id in {"hittest_ab_text_parent", "hittest_ab_shape_parent"}:
        actions = [bind, advance(0.1), draw]
        positions = (
            (
                [(float(value), 320.0) for value in range(400, 550, 10)]
                + [(500.0, float(value)) for value in range(200, 450, 10)]
            )
            if silver_id == "hittest_ab_text_parent"
            else (
                [(310.0, float(value)) for value in range(0, 550, 20)]
                + [(float(value), 420.0) for value in range(220, 530, 20)]
            )
        )
        for x, y in positions:
            actions += [frame, pointer("pointer-move", x, y), advance(0.016), draw]
        return tuple(actions)

    if silver_id == "virtualized_artboard_databound_children":
        actions = [
            bind,
            advance(0.1),
            draw,
            pointer("pointer-move", 60.0, 200.0),
            advance(0.016),
            draw,
            frame,
            pointer("pointer-down", 60.0, 200.0),
            advance(0.016),
            draw,
        ]
        for y in range(200, -500, -20):
            actions += [
                frame,
                pointer("pointer-move", 60.0, float(y)),
                advance(0.016),
                draw,
            ]
        actions += [
            frame,
            pointer("pointer-up", 60.0, -500.0),
            advance(0.016),
            draw,
        ]
        return tuple(actions)

    if silver_id == "multitouch_enter-MultiScroll":
        actions = [
            advance(0.1),
            draw,
            frame,
            advance(0.016),
            draw,
            pointer("pointer-down", 50.0, 400.0, 7),
            pointer("pointer-down", 350.0, 400.0, 8),
        ]
        for y in range(380, -1, -20):
            actions += [
                frame,
                pointer("pointer-move", 50.0, float(y), 7),
                pointer("pointer-move", 350.0, float(y), 8),
                advance(0.016),
                draw,
            ]
        actions += [
            pointer("pointer-up", 50.0, 0.0, 7),
            pointer("pointer-up", 350.0, 0.0, 8),
        ]
        return tuple(actions)

    if silver_id in {
        "scroll_threshold-vertical-scroll",
        "scroll_threshold-horizontal-scroll",
        "scroll_threshold-all-scroll",
    }:
        def coordinates(position: float) -> tuple[float | str, float | str]:
            if silver_id == "scroll_threshold-vertical-scroll":
                return "artboard-width/2", position
            if silver_id == "scroll_threshold-horizontal-scroll":
                return position, "artboard-height/2"
            return position, position

        actions = [bind, advance(0.1), draw, frame]
        thresholds = (
            (40.0, 10.0)
            if silver_id != "scroll_threshold-all-scroll"
            else (50.0, 32.0)
        )
        for threshold in thresholds:
            x, y = coordinates(70.0)
            actions += [pointer("pointer-down", x, y), advance(0.1), draw]
            position = 70.0
            while position > threshold:
                x, y = coordinates(position)
                actions += [frame, pointer("pointer-move", x, y), advance(0.1), draw]
                position -= 8.0
            x, y = coordinates(position)
            actions += [frame, pointer("pointer-up", x, y), advance(0.1), draw]
        return tuple(actions)

    return None


def stateful_component_actions(silver_id: str) -> tuple[dict[str, object], ...] | None:
    # These literal producers include object-identity/lifecycle assertions and
    # invalid list operations that must remain no-ops. Their native replay owner
    # preserves the complete C++ setup, mutation, frame and assertion sequences.
    scenarios = {
        "stateful_artboard_swap": "artboard-swap",
        "stateful_source_switch": "source-switch",
        "stateful_list_props_lifecycle": "list-lifecycle",
        "transition_self_comparator_test": "self-comparator",
    }
    scenario = scenarios.get(silver_id)
    return (action("replay-stateful-component", scenario=scenario),) if scenario else None


def view_model_rebinding_actions(silver_id: str) -> tuple[dict[str, object], ...] | None:
    """Preserve the upstream retained references; replacing a reference is not cloning."""
    def advance(seconds: float) -> dict[str, object]:
        return action("advance", target="state-machine", seconds=seconds)

    draw, frame = action("draw"), action("frame")
    if silver_id == "replace_vm_instance":
        # This literal multi-artboard producer owns its full setup: all three
        # artboards, then machines, then independent authored VMI trees.
        return (action("replay-view-model-dependents"),)
    if silver_id == "multi_listeners-rebind":
        return (
            action("create-default-view-model"),
            action("retain-view-model", name="old-child", property="child"),
            action("bind-prepared-view-model"), advance(0.016), draw, frame,
            action("set-named-view-model-number", view_model="old-child", property="num", value=2.0),
            advance(0.016), draw, frame,
            action("set-named-view-model-number", view_model="old-child", property="num2", value=2.0),
            advance(0.016), draw, frame,
            action("create-retained-view-model", name="new-child", view_model="RebindChild", fresh=False),
            action("replace-retained-view-model", property="child", replacement="new-child"),
            advance(0.016), draw, frame,
            action("set-named-view-model-number", view_model="new-child", property="num", value=2.0),
            advance(0.016), draw, frame,
            action("set-named-view-model-number", view_model="new-child", property="num2", value=2.0),
            advance(0.016), draw, frame,
            action("set-named-view-model-number", view_model="old-child", property="num", value=3.0),
            advance(0.016), draw,
        )
    if silver_id == "rebind_with_nested_viewmodel":
        return (
            action("bind-default-view-model"), advance(0.0), advance(0.016), draw, frame,
            action("retain-view-model", name="old-second", property="sec"),
            action("retain-view-model", name="old-third", **{"from": "old-second"}, property="thi"),
            action("set-retained-view-model-string", name="old-third", property="label", value="updated"),
            action("release-retained-view-model", name="old-third"),
            action("release-retained-view-model", name="old-second"),
            advance(0.016), draw, frame,
            action("create-retained-view-model", name="third", view_model="Third", fresh=True),
            action("set-retained-view-model-string", name="third", property="label", value="new updated"),
            action("create-retained-view-model", name="second", view_model="Second", fresh=True),
            action("replace-retained-view-model", owner="second", property="thi", replacement="third"),
            action("replace-retained-view-model", property="sec", replacement="second"),
            advance(0.016), draw,
        )
    if silver_id in ("replace_vm_instance-list", "replace_vm_instance-double-nest"):
        steps = [action("bind-authored-view-model-instance", instance_index=0),
                 advance(0.1), draw, frame,
                 action("create-retained-view-model", name="grandchild", view_model="main-list-grandchild", fresh=False),
                 action("set-retained-view-model-string", name="grandchild", property="label", value="modified")]
        if silver_id.endswith("-double-nest"):
            steps.append(action("retain-view-model", name="list-owner", list="lis", index=0))
        else:
            steps.append(action("retain-view-model", name="list-owner"))
        steps.extend((
            action("retain-view-model", name="existing-item", **{"from": "list-owner"}, list="lis", index=1),
            action("replace-retained-view-model", owner="existing-item", property="grandchild", replacement="grandchild"),
            advance(0.1), draw,
            action("release-retained-view-model", name="existing-item"), frame,
            action("create-retained-view-model", name="child", view_model="main-list-child", fresh=False),
            action("retain-view-model", name="child-grandchild", **{"from": "child"}, property="grandchild"),
            action("set-retained-view-model-string", name="child-grandchild", property="label", value="new label"),
            action("release-retained-view-model", name="child-grandchild"),
            action("append-retained-view-model-list-item", owner="list-owner", list="lis", item="child"),
            advance(0.1), draw, frame,
            action("create-retained-view-model", name="grandchild2", view_model="main-list-grandchild", fresh=False),
            action("set-retained-view-model-string", name="grandchild2", property="label", value="modified 2"),
            action("replace-retained-view-model", owner="child", property="grandchild", replacement="grandchild2"),
            advance(0.1), draw,
            action("release-retained-view-model", name="grandchild2"),
            action("release-retained-view-model", name="child"),
        ))
        return tuple(steps)
    return None


def bound_value_replay_actions(silver_id: str) -> tuple[dict[str, object], ...] | None:
    """Literal upstream actions for bound coordinates, computed values and assets."""
    def advance(seconds: float) -> dict[str, object]:
        return action("advance", target="state-machine", seconds=seconds)

    bind_default = action("bind-default-view-model")
    bind_authored = action("bind-authored-view-model-instance", instance_index=0)
    draw = action("draw")
    frame = action("frame")
    if silver_id == "artboard_opacity_and_transform_test":
        return (bind_default, advance(0.1), draw) + tuple(
            step for _ in range(11) for step in (
                frame, advance(0.1),
                action("pointer-down", x={"view_model_number": "xPos"}, y={"view_model_number": "yPos"}, pointer_id=0),
                action("pointer-up", x={"view_model_number": "xPos"}, y={"view_model_number": "yPos"}, pointer_id=0),
                draw,
            )
        )
    if silver_id == "solo_index_test":
        return (bind_default, advance(0.1), draw) + tuple(
            step for index in (1.0, 2.0, 3.0) for step in (
                frame, action("set-view-model-number", property="index", value=index),
                advance(0.1), draw,
            )
        )
    if silver_id == "image_computed_transform_bind":
        return (bind_default, advance(0.0), advance(0.016), draw) + tuple(
            action("assert-view-model-number", property=name, value=150.0, margin=5.0)
            for name in ("img1Width", "img1Height", "img2Width", "img2Height")
        ) + tuple(repeated_frames(62, 0.032)) + tuple(
            action("assert-view-model-number", property=name, value=value, margin=0.01)
            for name, value in (("img1Width", 200.0), ("img1Height", 200.0),
                                ("img2Width", 250.0), ("img2Height", 250.0))
        )
    if silver_id == "data_bind_blob_test":
        return (bind_authored, advance(0.1), draw, frame, advance(0.1), draw) + tuple(
            repeated_frames(4, 0.5)
        ) + (
            action("set-view-model-blob-bytes", property="xml", source="data_enum_roundtrip.rml", name="data_enum_roundtrip.rml"),
            frame, advance(0.5), draw,
        )
    if silver_id == "stateful_component_image_test":
        return (bind_authored, advance(0.1), draw, frame, advance(0.1), draw,
                action("set-view-model-image-bytes", property="img", source="open_source.jpg", expected_size=8880),
                frame, advance(0.1), draw)
    if silver_id == "image_binding_with_listener":
        click = (action("pointer-down", x=650.0, y=650.0, pointer_id=0),
                 action("pointer-up", x=650.0, y=650.0, pointer_id=0))
        return (bind_authored, advance(0.1), draw, frame, *click, advance(0.1), draw,
                action("set-view-model-image-bytes", property="image1", source="open_source.jpg", expected_size=8880),
                frame, advance(0.1), draw, frame, *click, advance(0.1), draw,
                action("clear-view-model-image", property="image1"),
                frame, advance(0.1), draw, frame, *click, advance(0.1), draw)
    return None


def p1q_round2_actions(silver_id: str) -> tuple[dict[str, object], ...] | None:
    """Faithful action ports for the second P1-q unsupported sweep."""

    bind = action("bind-default-view-model")
    frame = action("frame")
    draw = action("draw")
    advance = lambda seconds: action(
        "advance", target="state-machine", seconds=seconds
    )
    pointer = lambda kind, x, y, pointer_id=0, seconds=0.0: action(
        kind,
        x=x,
        y=y,
        **(
            {"seconds": seconds, "pointer_id": pointer_id}
            if kind == "pointer-move"
            else {"pointer_id": pointer_id}
        ),
    )

    if silver_id == "layout_scroll_snap_carousel":
        actions = [
            bind,
            advance(0.0),
            draw,
            pointer(
                "pointer-move",
                "artboard-width/2",
                "artboard-height/2",
                seconds=1.0,
            ),
            pointer("pointer-down", "artboard-width/2", "artboard-height/2"),
        ]
        for step in range(1, 5):
            actions += [
                frame,
                pointer(
                    "pointer-move",
                    f"artboard-width/2+{1500 * step / 4:g}",
                    "artboard-height/2",
                    seconds=float(step + 1),
                ),
                advance(0.016),
                draw,
            ]
        actions += [
            pointer(
                "pointer-up",
                "artboard-width/2+1500",
                "artboard-height/2",
            ),
            action(
                "advance-draw-until-scroll-physics-stops",
                max_frames=300,
                seconds=0.016,
            ),
        ]
        return tuple(actions)

    if silver_id in {"hittest_ab_2_non_virtualized", "hittest_ab_2_virtualized"}:
        actions = [
            bind,
            advance(0.1),
            draw,
            frame,
            action("set-view-model-number", property="scroll-offset", value=-100.0),
            advance(0.1),
            draw,
        ]
        coordinate = 200.0
        while coordinate > 100.0:
            actions += [
                frame,
                pointer("pointer-move", 50.0, coordinate),
                advance(0.016),
                draw,
            ]
            coordinate -= 10.0
        coordinate = 75.0
        actions += [
            frame,
            pointer("pointer-down", 50.0, coordinate),
            advance(0.1),
            draw,
        ]
        while coordinate > -500.0:
            actions += [
                frame,
                pointer("pointer-move", 50.0, coordinate),
                advance(0.016),
                draw,
            ]
            coordinate -= 20.0
        actions += [
            frame,
            pointer("pointer-up", 50.0, coordinate),
            advance(0.016),
            draw,
        ]
        coordinate = 110.0
        while coordinate > -5.0:
            actions += [
                frame,
                pointer("pointer-move", 50.0, coordinate),
                advance(0.016),
                draw,
            ]
            coordinate -= 4.0
        return tuple(actions)

    if silver_id == "deterministic_mode":
        actions = [
            bind,
            advance(0.016),
            draw,
            pointer("pointer-down", "artboard-width/2", 400.0),
            advance(0.016),
            draw,
        ]
        y = 400.0
        time = 0.0
        for _ in range(int(0.25 / 0.016)):
            time = cpp_f32(time + cpp_f32(0.016))
            actions += [
                frame,
                pointer("pointer-move", "artboard-width/2", y, seconds=time),
                advance(0.016),
                draw,
            ]
            y -= 40.0
        time = cpp_f32(time + cpp_f32(0.016))
        actions += [
            frame,
            pointer("pointer-move", "artboard-width/2", y, seconds=time),
            pointer("pointer-up", "artboard-width/2", y),
            advance(0.016),
            draw,
        ]
        actions += repeated_frames(int(1.0 / 0.016), 0.016)
        return tuple(actions)

    if silver_id == "draw_index_list":
        y = 90.0
        actions = [bind, advance(0.1), pointer("pointer-down", 30.0, y), draw]
        for _ in range(41):
            actions += [
                frame,
                advance(0.016),
                draw,
                pointer("pointer-move", 30.0, y),
            ]
            y -= 10.0
        actions += [frame, advance(0.016), draw, pointer("pointer-up", 30.0, y)]
        for x, y in ((100.0, 45.0), (100.0, 51.0), (100.0, 91.0)):
            actions += [
                frame,
                pointer("pointer-down", x, y),
                pointer("pointer-up", x, y),
                advance(0.016),
                draw,
            ]
        for index, click_y in enumerate((45.0, 85.0, 45.0)):
            if index in (0, 2):
                actions += [
                    frame,
                    pointer("pointer-down", 30.0, 90.0),
                    pointer("pointer-move", 30.0, 10.0),
                    pointer("pointer-up", 30.0, 10.0),
                    advance(0.016),
                    draw,
                ]
            actions += [
                frame,
                pointer("pointer-down", 100.0, click_y),
                pointer("pointer-up", 100.0, click_y),
                advance(0.016),
                draw,
            ]
        return tuple(actions)

    if silver_id == "multitouch_enter-MainList":
        actions = [bind, advance(0.1), draw, frame]
        actions += [pointer("pointer-down", 122.5845, 443.8406, 9), advance(0.016), draw, frame]
        actions += [
            pointer("pointer-down", 459.5410, 188.4058, 8),
            pointer("pointer-down", 333.3333, 248.1884, 7),
            advance(0.016),
            draw,
            frame,
        ]
        for x, y, pointer_id in (
            (459.5410, 188.4058, 8),
            (123.7923, 444.4445, 9),
            (333.3333, 248.1884, 7),
        ):
            actions += [pointer("pointer-up", x, y, pointer_id), pointer("pointer-exit", x, y, pointer_id)]
        actions += [advance(0.016), draw, frame]
        actions += [
            pointer("pointer-down", 118.9613, 439.6135, 7),
            pointer("pointer-down", 346.6183, 269.9276, 9),
            pointer("pointer-down", 459.5410, 194.4444, 8),
            advance(0.016),
            draw,
            frame,
        ]
        for x, y, pointer_id in (
            (346.6183, 269.9276, 9),
            (122.5845, 440.8212, 7),
            (459.5410, 194.4444, 8),
        ):
            actions += [pointer("pointer-up", x, y, pointer_id), pointer("pointer-exit", x, y, pointer_id)]
        actions += [advance(0.016), draw, frame]
        actions += [
            pointer("pointer-move", 50.0, 300.0, 7),
            pointer("pointer-move", 250.0, 200.0, 8),
            advance(0.016),
            draw,
        ]
        for offset in range(20, 301, 20):
            actions += [
                frame,
                pointer("pointer-move", 50.0 + offset, 300.0, 7),
                pointer("pointer-move", 250.0 + offset, 200.0, 8),
                advance(0.016),
                draw,
            ]
        return tuple(actions)

    if silver_id == "component_list_hit_order":
        actions = [bind, advance(0.1), draw]
        for x in (175.0, 325.0, 100.0):
            actions += [
                frame,
                pointer("pointer-move", x, 50.0),
                pointer("pointer-down", x, 50.0),
                pointer("pointer-up", x, 50.0),
                advance(0.1),
                draw,
            ]
        return tuple(actions)

    if silver_id == "component_list_virtualized_scroll_manual":
        return (
            bind,
            advance(0.1),
            draw,
            frame,
            pointer("pointer-move", 250.0, 50.0),
            pointer("pointer-down", 250.0, 50.0),
            advance(0.1),
            draw,
            frame,
            pointer("pointer-move", 50.0, 50.0),
            advance(0.1),
            draw,
            pointer("pointer-up", 50.0, 50.0),
        )

    if silver_id == "scroll_test":
        actions = [advance(0.1), draw, frame]
        actions += [
            pointer("pointer-down", "artboard-width/2", "artboard-height/2"),
            advance(0.1),
            advance(1.0),
            draw,
            frame,
            pointer("pointer-down", 260.0, 500.0),
            advance(0.1),
            advance(1.0),
            draw,
        ]
        frames = int(1.0 / 0.016)
        for index in range(frames):
            actions += [
                frame,
                pointer(
                    "pointer-move",
                    260.0 - index * 100.0 / frames,
                    500.0 - index * 400.0 / frames,
                ),
                advance(0.1),
                advance(0.016),
                draw,
            ]
        actions += [
            frame,
            pointer("pointer-up", 160.0, 100.0),
            advance(0.1),
            advance(0.016),
            draw,
            frame,
            pointer("pointer-down", 50.0, 500.0),
            advance(0.1),
            advance(1.0),
            draw,
        ]
        for index in range(frames):
            actions += [
                frame,
                pointer(
                    "pointer-move",
                    50.0 + index * 100.0 / frames,
                    500.0 - index * 400.0 / frames,
                ),
                advance(0.1),
                advance(0.016),
                draw,
            ]
        actions += [
            frame,
            pointer("pointer-up", 150.0, 100.0),
            advance(0.1),
            advance(0.016),
            draw,
        ]
        return tuple(actions)

    if silver_id == "interactive_scrolling":
        drag = action(
            "vertical-pointer-drag",
            x="artboard-width/2",
            start_y="artboard-height-20",
            end_y_exclusive=120.0,
            step=20.0,
            advance_seconds=0.1,
            pointer_id=0,
        )
        return (
            bind,
            advance(0.1),
            draw,
            drag,
            action("set-view-model-boolean", property="isInteractive", value=True),
            advance(0.1),
            drag,
        )

    if silver_id == "focusable_element":
        actions = [action("bind-fresh-view-model"), advance(0.1), draw]
        for path in ([0], [1], [1, 0], [1, 1], [1, 2], [2], None):
            actions += [frame, action("focus-next"), advance(0.1)]
            actions += [action("assert-focus-node", **({"path": path} if path is not None else {})), draw]
        return tuple(actions)

    if silver_id == "keyboard_listener":
        actions = [bind, advance(0.016), draw, frame]
        actions += [action("focus-previous"), advance(0.016), draw, frame]
        actions += [action("key-input", key=32, modifiers=0, pressed=False, repeat=False), advance(0.016), draw, frame]
        actions += [action("focus-previous") for _ in range(3)]
        actions += [advance(0.016), draw, frame, action("key-input", key=32, modifiers=0, pressed=False, repeat=False), advance(0.016), draw, frame]
        actions += [action("focus-previous") for _ in range(2)]
        actions += [advance(0.016), draw, frame, action("key-input", key=32, modifiers=0, pressed=False, repeat=False), advance(0.016), draw, frame]
        actions += [action("focus-previous"), advance(0.016), draw, frame, action("key-input", key=32, modifiers=0, pressed=False, repeat=False), advance(0.016), draw]
        return tuple(actions)

    if silver_id == "keyboard_listener-KeyboardInput":
        k = lambda key, modifiers, pressed, repeat: action(
            "key-input", key=key, modifiers=modifiers, pressed=pressed, repeat=repeat
        )
        return (
            bind, advance(0.016), draw, frame, action("focus-next"), advance(0.016), draw, frame,
            k(65, 0, True, False), advance(0.016), draw, frame,
            k(65, 0, True, True), advance(0.016), k(65, 0, False, False), advance(0.016),
            k(65, 1, True, False), advance(0.016),
            k(69, 0, False, False), k(69, 0, True, True), k(69, 0, True, False), advance(0.016),
            k(66, 0, True, False), advance(0.016), k(66, 0, False, False), advance(0.016),
            k(66, 0, True, True), advance(0.016), k(68, 0, True, False), advance(0.016),
            k(68, 9, True, False), advance(0.016), k(67, 9, True, False), advance(0.016),
            k(67, 1, True, False), advance(0.016), k(88, 1, True, False), advance(0.016), draw,
        )

    if silver_id == "list_focus_order":
        actions = [bind, advance(0.016), draw, frame, action("focus-next"), advance(0.016), draw, frame]
        actions += [action("focus-next") for _ in range(3)] + [advance(0.016), draw, frame]
        for processed, count, focus in ((False, 1.0, True), (False, 2.0, True), (False, 3.0, False)):
            actions += [
                action("set-view-model-boolean", property="stageProcessed", value=processed),
                action("set-view-model-number", property="stageCount", value=count),
                advance(0.016), draw, frame,
            ]
            if focus:
                actions += [action("focus-next"), advance(0.016), draw, frame]
        actions += [action("focus-next"), advance(0.016), draw]
        return tuple(actions)

    if silver_id == "focus_collapsing":
        actions = [bind, advance(0.016), draw, frame, action("focus-next"), advance(0.016), draw, frame]
        actions += [action("focus-next"), advance(0.016), draw, frame]
        actions += [action("set-view-model-number", property="opacity", value=0.0), advance(0.016), advance(0.016), draw, frame]
        actions += [action("set-view-model-number", property="opacity", value=1.0), advance(0.016), draw, frame]
        actions += [action("focus-next"), action("focus-next"), advance(0.016), draw, frame]
        actions += [action("set-view-model-boolean", property="isMainLayout2Visible", value=False), advance(0.016), action("focus-next"), advance(0.016), draw, frame]
        actions += [action("focus-next"), advance(0.016), draw, frame]
        actions += [action("focus-next"), action("focus-next"), advance(0.016), draw, frame]
        actions += [action("set-view-model-boolean", property="isMainLayout2Visible", value=True), advance(0.016), action("focus-next"), draw, frame]
        actions += [advance(0.016), action("focus-next"), advance(0.016), draw, frame]
        actions += [action("focus-next"), advance(0.016), action("focus-next"), draw, frame]
        actions += [action("focus-next"), advance(0.016), draw, frame]
        actions += [action("focus-next"), advance(0.016), draw]
        return tuple(actions)

    named = {
        "relative_data_bind_path": "ViewModel1",
        "relative_data_bind_path-listener": "SML_VM2",
        "relative_data_bind_path-fire-trigger": "SMFT-VM2",
        "relative_data_bind_path-scripted-input": "SI-VM2",
    }
    if silver_id in named:
        actions = [bind, advance(0.1), draw, frame]
        if silver_id == "relative_data_bind_path-listener":
            actions += [action("set-view-model-number", property="num", value=100.0), advance(0.1), draw, frame]
        elif silver_id == "relative_data_bind_path-fire-trigger":
            actions += [advance(0.1), draw, frame, action("fire-view-model-trigger", property="reset"), advance(0.1), draw, frame]
        elif silver_id == "relative_data_bind_path-scripted-input":
            actions += [action("set-view-model-boolean", property="child/paused", value=False), advance(1.0), draw, frame, action("set-view-model-boolean", property="child/paused", value=True), action("set-view-model-boolean", property="child/boo", value=False), advance(1.0), draw, frame]
        actions += [action("bind-named-default-view-model", view_model=named[silver_id]), advance(0.1), draw]
        if silver_id == "relative_data_bind_path":
            actions += [frame, action("bind-named-default-view-model", view_model="ViewModel2"), advance(0.1), draw]
        elif silver_id == "relative_data_bind_path-listener":
            actions += [frame, action("set-view-model-number", property="num", value=100.0), advance(0.1), draw]
        elif silver_id == "relative_data_bind_path-fire-trigger":
            actions += [frame, advance(0.1), draw]
        elif silver_id == "relative_data_bind_path-scripted-input":
            actions += [frame, action("set-view-model-boolean", property="child/paused", value=False), advance(1.0), draw, frame, action("set-view-model-boolean", property="child/paused", value=True), action("set-view-model-boolean", property="child/boo", value=False), advance(1.0), draw]
        return tuple(actions)

    if silver_id in {"formula_random-source_change", "formula_random-once", "formula_random-always"}:
        actions = [
            action("clear-randoms"),
            action("assert-random-calls", count=0),
            action("add-random-value", value=0.0),
            bind,
            advance(0.1),
            draw,
            action("assert-random-calls", count=1),
            frame,
        ]
        if silver_id != "formula_random-always":
            actions.append(action("add-random-value", value=1.0))
        actions += [
            action("set-view-model-number", property="n1", value=500.0),
            advance(0.1),
            draw,
            action("advance-draw-frames", frames=62, seconds=0.016),
            action(
                "assert-random-calls",
                count={
                    "formula_random-source_change": 2,
                    "formula_random-once": 1,
                    "formula_random-always": 64,
                }[silver_id],
            ),
            action("clear-randoms"),
        ]
        return tuple(actions)

    if silver_id in {"data_viz_demo", "db_health_tracker", "echo_show_demo"}:
        # serialized_rendering_test.cpp binds authored instance 0 directly
        # to machine 0, using a fresh instance only for an unset viewModelId.
        actions = [
            action("bind-selected-artboard-authored-view-model", if_present=False),
            advance(0.1),
            draw,
        ]
        if silver_id == "data_viz_demo":
            actions.append(action("set-view-model-number", property="item1/value", value=20.0))
            # int(0.8f / 0.064f), not the old dormant 30-frame recipe.
            actions += repeated_frames(12, 0.064)
        else:
            # Both producers use int(1.0f / 0.016f) frames; echo advances
            # 0.1 seconds each frame even though its count uses 0.016.
            actions += repeated_frames(62, 0.1 if silver_id == "echo_show_demo" else 0.016)
        return tuple(actions)

    if silver_id == "data_bind_artboard_input":
        return (
            action("bind-fresh-view-model"), draw, advance(0.1), draw,
            action("set-view-model-bindable-artboard-by-name", property="artboardProperty", artboard="child2"),
            frame, advance(0.1), draw,
            action("set-view-model-bindable-artboard-by-name", property="artboardProperty", artboard="child1"),
            frame, advance(0.1), draw,
            action("set-view-model-artboard-from-file", property="artboardProperty", file="data_binding_artboards_source_test.riv", artboard="default"),
            frame, advance(0.1), draw,
            action("set-view-model-artboard", property="artboardProperty", value=1),
            frame, advance(0.1), draw,
            action("set-view-model-artboard", property="artboardProperty", value=10),
            frame, advance(0.1), draw,
        )

    if silver_id == "bidirectional_binding_source":
        actions = [
            action("create-default-view-model"),
            action("bind-prepared-view-model"),
            advance(0.0),
            action("prepare-bindable-artboard-from-file", name="costume1", file="bidirectional_binding_target_1.riv", artboard="costume_artboard"),
            action("prepare-bindable-artboard-from-file", name="costume2", file="bidirectional_binding_target_2.riv", artboard="costume_artboard"),
            action("set-view-model-boolean", property="costume_db_bool", value=True),
            draw,
            frame,
            advance(0.016),
            draw,
        ]
        # C++ retains these two bindable occurrences and assigns each twice.
        for name in ("costume1", "costume2", "costume1", "costume2"):
            actions += [
                frame,
                action(
                    "set-view-model-prepared-bindable-artboard",
                    property="costume_db_artboard",
                    name=name,
                ),
                advance(0.016),
                draw,
                frame,
                advance(0.016),
                draw,
            ]
        return tuple(actions)

    if silver_id == "data_binding_artboards_default_test":
        return (
            action("bind-authored-view-model"), advance(0.1), draw,
            frame, advance(0.1), draw,
            action("set-view-model-bindable-artboard-by-name", property="ab", artboard="ch1"),
            frame, advance(0.1), draw,
            action("set-view-model-bindable-artboard-by-name", property="ab", artboard="ch2"),
            frame, advance(0.1), draw,
            action("set-view-model-artboard-from-file", property="ab", file="data_binding_artboards_source_test.riv", artboard="default"),
            frame, advance(0.1), draw,
        )

    if silver_id == "databind_external_artboard_main":
        return (
            advance(0.0), draw, frame,
            action("bind-fresh-view-model"),
            action("set-view-model-artboard-from-file", property="ab", file="databind_external_artboard_child.riv", artboard="ExternalChild"),
            action("replace-view-model-from-file", property="child", file="databind_external_artboard_child.riv", view_model="Child", instance="Instance"),
            action("set-view-model-string", property="child/label", value="updated label"),
            advance(0.016), draw,
        )

    if silver_id == "bindable_artboard_nesty":
        actions = [
            action("create-fresh-view-model"),
            action("prepare-bindable-artboard-from-file", name="inserted", file="bindable_artboard_child.riv", artboard="Artboard"),
            action("prepare-fresh-view-model-from-file", name="renamed", file="bindable_artboard_child.riv", view_model="ViewModelRenamed"),
            action("set-view-model-prepared-bindable-artboard", property="someArtboard", name="inserted"),
            action("set-artboard-bound-prepared-view-model", property="someArtboard", name="renamed"),
            action("bind-prepared-view-model"),
            advance(0.016), draw,
            frame, pointer("pointer-down", 250, 250), pointer("pointer-up", 250, 250), advance(0.016), draw,
        ]
        actions += repeated_frames(5, 0.1)
        actions += [frame, pointer("pointer-down", 250, 250), pointer("pointer-up", 250, 250), advance(0.016), draw]
        return tuple(actions)

    if silver_id == "data_binding_artboards_test":
        actions = [
            action("bind-authored-view-model"), advance(0.1), draw,
            frame, advance(0.1), draw,
        ]
        for source_file, artboard in (
            ("data_binding_artboards_test.riv", "ch1"),
            ("data_binding_artboards_test.riv", "ch2"),
            ("data_binding_artboards_source_test.riv", "source_1"),
        ):
            actions += [
                action("prepare-bindable-artboard-from-file", name=artboard, file=source_file, artboard=artboard),
                action("set-view-model-prepared-bindable-artboard", property="ab", name=artboard),
                frame, advance(0.1), draw,
            ]
        for _ in range(2):
            actions += [action("fire-view-model-trigger", property="ch/tr"), frame, advance(0.1), draw]
        actions.append(action("prepare-bindable-artboard-from-file", name="source_2", file="data_binding_artboards_source_test.riv", artboard="source_2"))
        for name in ("source_2", "source_1", "ch2"):
            actions += [
                action("set-view-model-prepared-bindable-artboard", property="ab", name=name),
                frame, advance(0.1), draw,
            ]
        actions += [
            action("clear-view-model-artboard", property="ab"), frame, advance(0.1), draw,
            action("set-view-model-prepared-bindable-artboard", property="ab", name="ch2"), frame, advance(0.1), draw,
        ]
        return tuple(actions)

    if silver_id == "global_variables_test":
        return (
            action("create-default-main-view-model"),
            action("set-state-machine-main-view-model"),
            action("set-state-machine-default-global-view-models"),
            action("bind-state-machine-view-models"),
            advance(0.1),
            draw,
            action("advance-draw-frames", frames=int(1.0 / 0.016), seconds=0.016),
        )

    if silver_id == "global_viewmodels_test-set_instance":
        return (
            action("create-default-view-model"),
            action(
                "set-global-view-model-color",
                **{"global": "GlobalColors"},
                property="c1",
                value=0xFFFFFF00,
            ),
            action("bind-prepared-view-model"),
            advance(0.0),
            draw,
            frame,
            advance(0.016),
            draw,
            action("create-default-view-model"),
            action("set-view-model-string", property="label", value="label updated"),
            action(
                "set-global-view-model-color",
                **{"global": "GlobalColors"},
                property="c1",
                value=0xFF00FFFF,
            ),
            action("bind-prepared-view-model"),
            frame,
            advance(0.016),
            draw,
        )

    if silver_id == "layout_scroll_visibility":
        actions = [bind, advance(0.0), draw]
        for index in range(300):
            if index == 30:
                actions.append(action("set-view-model-enum", property="vis2", value=1))
            elif index == 90:
                actions.append(action("set-view-model-enum", property="vis3", value=1))
            elif index == 150:
                actions.append(action("set-view-model-enum", property="vis2", value=0))
            elif index == 210:
                actions.append(action("set-view-model-enum", property="vis4", value=1))
            elif index == 270:
                actions += [
                    action("set-view-model-enum", property="vis2", value=0),
                    action("set-view-model-enum", property="vis3", value=0),
                    action("set-view-model-enum", property="vis4", value=0),
                ]
            actions += [frame, advance(1.0 / 60.0), draw]
        return tuple(actions)

    if silver_id == "scroll_intent":
        actions = [bind, advance(0.0), draw]
        for index in range(35):
            if index == 5:
                actions.append(action("set-view-model-number", property="scrollIndex", value=2.0))
            elif index == 10:
                actions.append(action("set-view-model-enum", property="display", value=1))
            elif index == 15:
                actions.append(action("set-view-model-number", property="scrollIndex", value=4.0))
            elif index == 20:
                actions.append(action("set-view-model-enum", property="display", value=0))
            elif index == 25:
                actions.append(action("set-view-model-number", property="scrollIndex", value=100.0))
            elif index == 30:
                actions.append(action("set-view-model-number", property="scrollIndex", value=0.0))
            actions += [frame, advance(1.0 / 60.0), draw]
        return tuple(actions)

    if silver_id == "data_binding_artboards_test_recursive":
        actions = [action("bind-authored-view-model"), advance(0.1), draw, frame, advance(0.1), draw]
        for artboard_name in (
            "recursive-grand-child-1",
            "recursive-parent",
            "recursive-grand-parent",
            "recursive-grand-child-2",
        ):
            actions += [
                action(
                    "set-view-model-bindable-artboard-by-name",
                    property="ab",
                    artboard=artboard_name,
                ),
                frame,
                advance(0.1),
                draw,
            ]
        return tuple(actions)

    if silver_id == "component_list_grouped":
        actions = [bind, advance(0.1), draw]
        actions += repeated_frames(int(1.0 / 0.16), 0.16)
        for index, property_name, value in (
            (0, "x", -90.0),
            (1, "x", 25.0),
            (2, "x", 150.0),
            (0, "y", -50.0),
            (1, "y", 100.0),
            (2, "y", -200.0),
        ):
            actions += [
                action(
                    "set-view-model-list-item-number",
                    list="List property",
                    index=index,
                    property=property_name,
                    value=value,
                ),
                advance(0.1),
                draw,
                frame,
            ]
        for x, y in ((210.0, 250.0), (325.0, 400.0), (450.0, 100.0)):
            for _ in range(int(1.0 / 0.16)):
                actions += [pointer("pointer-move", x, y), advance(0.16), draw, frame]
        return tuple(actions)

    if silver_id == "custom_property_trigger_bind":
        actions = [bind, advance(0.0), draw]
        actions += repeated_frames(int(1.0 / 0.16), 0.16)
        return tuple(actions)

    if silver_id == "text_feather_falloff":
        actions = [action("advance", target="animation", seconds=0.0), draw]
        for _ in range(60):
            actions += [
                frame,
                action("advance", target="animation", seconds=1.0 / 60.0),
                draw,
            ]
        return tuple(actions)

    if silver_id == "juice":
        actions = [action("advance", target="animation", seconds=0.0), draw]
        # Pinned source derives this from the selected animation's one-second
        # duration: int(durationSeconds() / 0.016f) == 62.
        for _ in range(62):
            actions += [
                frame,
                action("advance", target="animation", seconds=0.016),
                draw,
            ]
        return tuple(actions)

    if silver_id == "interpolate_to_end":
        actions = [
            bind,
            advance(0.1),
            draw,
            action("set-view-model-number", property="num", value=1000.0),
            advance(0.001),
        ]
        actions += repeated_frames(5, 0.25)
        return tuple(actions)

    if silver_id == "image_fit_alignment":
        actions = [bind, advance(0.1), draw]
        actions += repeated_frames(20, 0.016)
        actions += [
            action(
                "set-view-model-asset-by-name",
                property="imageProperty",
                asset="image2",
            ),
            advance(0.0),
        ]
        actions += repeated_frames(20, 0.016)
        actions += [
            action(
                "set-view-model-asset-by-name",
                property="imageProperty",
                asset="image3",
            ),
            advance(0.0),
        ]
        actions += repeated_frames(20, 0.016)
        return tuple(actions)

    return None


def p2e_gamepad_actions(
    silver_id: str,
) -> tuple[dict[str, object], ...] | None:
    """Replay pinned gamepad_test.cpp's complete serialized-input sequence."""

    if silver_id != "gamepad_test":
        return None

    def connected(device_id: int, mapping: str = "standard") -> dict[str, object]:
        return {"kind": "connected", "device_id": device_id, "mapping": mapping}

    def update(
        device_id: int, input_kind: str, index: int, value: float
    ) -> dict[str, object]:
        return {
            "kind": "update",
            "device_id": device_id,
            "input": input_kind,
            "index": index,
            "value": value,
        }

    def batch(*records: dict[str, object]) -> dict[str, object]:
        return {"kind": "gamepad-batch", "records": list(records)}

    def advance(seconds: float) -> dict[str, object]:
        return action("advance", target="state-machine", seconds=seconds)

    frame = action("frame")
    draw = action("draw")
    actions = [
        action("bind-default-view-model"),
        advance(0.0),
        action("frame-size"),
        advance(0.1),
        draw,
        frame,
        batch(connected(3)),
        advance(0.016),
        draw,
        frame,
        batch(update(3, "button", 0, 1.0)),
        advance(0.016),
        draw,
        frame,
        batch(connected(5), connected(1, "unknown")),
        advance(0.016),
        draw,
        frame,
        batch(update(5, "button", 2, 1.0), update(1, "button", 2, 1.0)),
        advance(0.016),
        draw,
    ]
    for index in range(1, 10):
        actions += [
            frame,
            batch(
                update(3, "axis", 0, index * 0.1),
                update(5, "axis", 1, index * -0.1),
            ),
            advance(0.016),
            draw,
        ]
    actions += [
        frame,
        batch({"kind": "disconnected", "device_id": 1}),
        advance(0.016),
        draw,
        action("focus-next"),
        frame,
        batch(update(3, "button", 0, 0.0)),
        advance(0.016),
        draw,
        frame,
        batch(update(3, "button", 0, 1.0)),
        advance(0.016),
        draw,
        frame,
        batch(update(3, "button", 0, 0.0)),
        advance(0.016),
        draw,
        frame,
        batch(update(3, "button", 1, 1.0)),
        advance(0.016),
        draw,
        frame,
        batch(update(3, "axis", 0, 0.5), update(3, "axis", 1, 0.5)),
        advance(0.016),
        draw,
    ]
    return tuple(actions)


DIVERGENCES = dict(
    line.split("|", 1)
    for line in """
layoutstest_8-planets-grid|frame 0, op 34900 (addRawPath), field point: expected (1194.5, 0), got (2105, 0)
inventory_demo_test_v2|frame 0, op 3862 (transform), field tx: expected 842.5, got 800
game_menu_ad_police_files|frame 14, op 4868 (addRawPath), field point: expected (716.91016, 0), got (794.0928, 0)
layout_grid_stack_grid_with_layout_participants|frame 0, op 169 (addRawPath), field point: expected (360, 310), got (360, 341.80002)
car_widgets_v01|frame 0, op 10306 (addRawPath): expected 60 fields, got 56
collapse_data_binds-test_1|frame 0, op 100 (transform), field tx: expected 411.31592, got 419.16016
data_converter_to_number|frame 41, op 2120 (addRawPath): expected 1850 fields, got 1443
focus_traversal|frame 0, op 95 (color): expected color, got save
hittest_ab1|frame 1, op 153 (color): expected color, got save
hittest_ab1_grand_parent|frame 2, op 304 (color): expected color, got save
hittest_ab1_parent|frame 1, op 192 (color): expected color, got save
hittest_nested|frame 1, op 155 (save): expected save, got color
hunter_x_demo|frame 0, op 5055 (addRawPath): expected 20 fields, got 22
layout_aspect_ratio|frame 0, op 42 (addRawPath), field point: expected (142, 71), got (142, 133)
layout_text_match|frame 0, op 441 (addRawPath), field point: expected (262.43164, 0), got (300, 0)
layout_text_match_7_3|frame 0, op 441 (addRawPath), field point: expected (262.43164, 0), got (300, 0)
nested_events|frame 1, op 166 (makeRenderPath): expected makeRenderPath, got rewind
rewards_demo|frame 0, op 1461 (addRawPath): expected 44 fields, got 46
""".strip().splitlines()
)

CPP_NUMBER = r"[-+]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:f)?"
POINTER_COORDINATE = (
    r"(?:"
    + CPP_NUMBER
    + r"|artboard->(?:width|height)\s*\(\s*\)\s*(?:/\s*2(?:\.0)?f?|\*\s*0\.8|\-\s*20))"
)
POINTER_CALL_PATTERN = (
    r"(?P<pointer>\b\w+->(?P<pointer_method>pointerDown|pointerMove|pointerUp|pointerExit)"
    r"\s*\(\s*(?:rive::)?Vec2D\s*\(\s*(?P<pointer_x>"
    + POINTER_COORDINATE
    + r")\s*,\s*(?P<pointer_y>"
    + POINTER_COORDINATE
    + r")\s*\)\s*(?:,\s*(?P<pointer_arg1>"
    + CPP_NUMBER
    + r")\s*)?(?:,\s*(?P<pointer_arg2>"
    + CPP_NUMBER
    + r")\s*)?\))"
)
POINTER_CALL = re.compile(POINTER_CALL_PATTERN)
ANY_POINTER_CALL = re.compile(
    r"\b\w+->(?:pointerDown|pointerMove|pointerUp|pointerExit)\s*\("
)
ACTION_CALL = re.compile(
    r"(?P<frame>\bsilver\.addFrame\s*\(\s*\))"
    r"|(?P<bind>\b\w+->bindViewModelInstance\s*\([^;]*\))"
    r"|(?P<advance>\b(?P<advance_owner>\w+)->(?P<advance_method>advanceAndApply|advance)"
    r"\(\s*(?P<seconds>[0-9]+(?:\.[0-9]+)?)(?:f)?\s*\))"
    r"|"
    + POINTER_CALL_PATTERN
    + r"|(?P<draw>\b\w+->draw\s*\([^;]*\))"
)
LOOP = re.compile(
    r"\bfor\s*\([^;]*;[^;]*<\s*(?P<count>\w+|[0-9]+)[^;]*;[^)]*\)\s*\{"
)
FRAME_COUNT = re.compile(
    r"\b(?:int|size_t)\s+(?P<name>\w+)\s*=\s*"
    r"(?:(?P<literal>[0-9]+)|"
    r"(?:\(int\)\s*)?\(\s*(?P<numerator>[0-9]+(?:\.[0-9]+)?)(?:f)?\s*/\s*"
    r"(?P<denominator>[0-9]+(?:\.[0-9]+)?)(?:f)?\s*\))"
)


def cpp_f32(value: float | str) -> float:
    """Round a value through the pinned C++ `float` representation."""
    return struct.unpack("=f", struct.pack("=f", float(value)))[0]


def cpp_float_division_to_int(numerator: str, denominator: str) -> int:
    """Match `(int)(Nf / Df)`: f32 operands, f32 division, truncation."""
    quotient = cpp_f32(cpp_f32(numerator) / cpp_f32(denominator))
    return int(quotient)


def action(kind: str, **values: object) -> dict[str, object]:
    return {"kind": kind, **values}


def pointer_coordinate(value: str) -> float | str:
    compact = re.sub(r"\s+", "", value).removesuffix("f")
    try:
        return float(compact)
    except ValueError:
        return {
            "artboard->width()/2.0": "artboard-width/2",
            "artboard->width()/2": "artboard-width/2",
            "artboard->height()/2.0": "artboard-height/2",
            "artboard->height()/2": "artboard-height/2",
            "artboard->width()*0.8": "artboard-width*0.8",
            "artboard->height()-20": "artboard-height-20",
        }[compact]


def strip_cpp_comments(source: str) -> str:
    """Blank C++ comments without changing offsets or quoted literals."""
    result = list(source)
    mode = "code"
    index = 0
    while index < len(source):
        char = source[index]
        following = source[index + 1] if index + 1 < len(source) else ""
        if mode == "code":
            if char == '"':
                mode = "string"
            elif char == "'":
                mode = "character"
            elif char == "/" and following == "/":
                result[index] = result[index + 1] = " "
                mode = "line-comment"
                index += 1
            elif char == "/" and following == "*":
                result[index] = result[index + 1] = " "
                mode = "block-comment"
                index += 1
        elif mode in ("string", "character"):
            if char == "\\":
                index += 1
            elif (mode == "string" and char == '"') or (
                mode == "character" and char == "'"
            ):
                mode = "code"
        elif mode == "line-comment":
            if char == "\n":
                mode = "code"
            else:
                result[index] = " "
        elif mode == "block-comment":
            if char == "*" and following == "/":
                result[index] = result[index + 1] = " "
                mode = "code"
                index += 1
            elif char != "\n":
                result[index] = " "
        index += 1
    return "".join(result)


def matching_brace(source: str, opening: int) -> int:
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return index
    raise ValueError("unterminated C++ action block")


def flat_actions(source: str, state_machine: str, animation: str) -> list[dict[str, object]]:
    actions: list[dict[str, object]] = []
    for match in ACTION_CALL.finditer(source):
        if match.group("frame"):
            actions.append(action("frame"))
        elif match.group("bind"):
            actions.append(action("bind-default-view-model"))
        elif match.group("draw"):
            actions.append(action("draw"))
        elif match.group("pointer"):
            method = match.group("pointer_method")
            x = pointer_coordinate(match.group("pointer_x"))
            y = pointer_coordinate(match.group("pointer_y"))
            arg1 = match.group("pointer_arg1")
            arg2 = match.group("pointer_arg2")
            if method == "pointerMove":
                actions.append(
                    action(
                        "pointer-move",
                        x=x,
                        y=y,
                        seconds=(
                            float(arg1.removesuffix("f"))
                            if arg1 is not None
                            else 0.0
                        ),
                        pointer_id=(
                            int(float(arg2.removesuffix("f")))
                            if arg2 is not None
                            else 0
                        ),
                    )
                )
            else:
                actions.append(
                    action(
                        {
                            "pointerDown": "pointer-down",
                            "pointerUp": "pointer-up",
                            "pointerExit": "pointer-exit",
                        }[method],
                        x=x,
                        y=y,
                        pointer_id=(
                            int(float(arg1.removesuffix("f")))
                            if arg1 is not None
                            else 0
                        ),
                    )
                )
        else:
            owner = match.group("advance_owner").lower()
            target = (
                "artboard"
                if "artboard" in owner
                else "animation"
                if animation != "none" and "state" not in owner and "machine" not in owner
                else "state-machine"
            )
            actions.append(
                action(
                    "advance",
                    target=target,
                    seconds=float(match.group("seconds")),
                )
            )
    return actions


def expand_action_loops(
    source: str,
    state_machine: str,
    animation: str,
    counts: dict[str, int],
) -> list[dict[str, object]]:
    result: list[dict[str, object]] = []
    position = 0
    while loop := LOOP.search(source, position):
        result.extend(flat_actions(source[position : loop.start()], state_machine, animation))
        count_text = loop.group("count")
        count = int(count_text) if count_text.isdigit() else counts.get(count_text)
        if count is None:
            raise ValueError(f"runtime-derived loop count {count_text}")
        opening = source.find("{", loop.start(), loop.end())
        closing = matching_brace(source, opening)
        body = expand_action_loops(
            source[opening + 1 : closing], state_machine, animation, counts
        )
        result.extend(body * count)
        position = closing + 1
    result.extend(flat_actions(source[position:], state_machine, animation))
    return result


def blocking_subsystem(chunk: str) -> str | None:
    checks = (
        (
            r"\b(?:submitGamepadsFromBuffer|connected|disconnected|updateOne)\s*\(",
            "gamepad-input-sequence",
        ),
        (
            r"\b(?:focusNext|focusPrevious|keyInput)\s*\(",
            "focus-keyboard-dispatch",
        ),
        (
            r"\b(?:setViewModelInstance|setGlobalViewModelInstance|globalViewModelNames)"
            r"\s*\(|\bstateMachine->bind\s*\(",
            "global-view-model-setup",
        ),
        (
            r"\bcreateDefaultViewModelInstance\s*\(\s*vm\s*\)"
            r"|\bfile->viewModel\s*\(",
            "named-view-model-instance",
        ),
        (r"\bpropertyValue\s*\(", "view-model-mutation"),
        (r"\b(?:addItem|removeItem|clear|swap)\s*\(", "component-list-mutation"),
        (r"\b(?:inputNamed|getNumber|getBool|getTrigger)\s*\(", "state-machine-input-mutation"),
        (r"\b(?:textValueRun|TextValueRun|shapeId|fontSize|sizing)\b", "text-layout-mutation"),
        (r"\b(?:layoutWidth|layoutHeight|setArtboardSize)\b", "layout-mutation"),
        (r"\b(?:find|findObject)\s*(?:<[^>]+>)?\s*\(", "runtime-object-mutation"),
        (r"\b(?:random|Random)\b", "random-sequence-encoding"),
    )
    for pattern, subsystem in checks:
        if re.search(pattern, chunk):
            return subsystem
    return None


def executable_actions(
    chunk: str, state_machine: str, animation: str
) -> tuple[tuple[dict[str, object], ...], str | None]:
    clean = strip_cpp_comments(chunk)
    if len(ANY_POINTER_CALL.findall(clean)) != len(POINTER_CALL.findall(clean)):
        return (), "pointer-expression-encoding"
    blocker = blocking_subsystem(clean)
    if blocker is not None:
        return (), blocker
    counts = {
        match.group("name"): (
            int(match.group("literal"))
            if match.group("literal") is not None
            else cpp_float_division_to_int(
                match.group("numerator"), match.group("denominator")
            )
        )
        for match in FRAME_COUNT.finditer(clean)
    }
    try:
        actions = expand_action_loops(clean, state_machine, animation, counts)
    except ValueError:
        return (), "runtime-derived-loop"
    if not actions or not any(item["kind"] == "draw" for item in actions):
        return (), "cpp-action-encoding"
    return tuple(actions), None


def quoted(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def unique(values: list[str]) -> tuple[str, ...]:
    return tuple(dict.fromkeys(values))


def strip_asset_prefix(value: str) -> str:
    return value.removeprefix("assets/")


def test_chunks(source: str) -> list[tuple[str, int, str]]:
    starts = list(TEST_CASE.finditer(source))
    chunks: list[tuple[str, int, str]] = []
    for index, start in enumerate(starts):
        end = starts[index + 1].start() if index + 1 < len(starts) else len(source)
        chunk = source[start.start() : end]
        name = re.search(r'"([^"]+)"', chunk)
        if name is None:
            continue
        line = source.count("\n", 0, start.start()) + 1
        chunks.append((name.group(1), line, chunk))
    return chunks


def infer_selector(pattern: re.Pattern[str], chunk: str, default_marker: str) -> str:
    names = unique(pattern.findall(chunk))
    if names:
        return " | ".join(names)
    return default_marker


def literal_producers(runtime_dir: Path) -> list[Producer]:
    runtime_tests = runtime_dir / "tests/unit_tests/runtime"
    files = sorted(runtime_tests.glob("*.cpp")) + sorted(
        (runtime_tests / "scripting").glob("*.cpp")
    )
    producers: list[Producer] = []
    for path in files:
        relative = path.relative_to(runtime_dir).as_posix()
        source = strip_cpp_comments(path.read_text(encoding="utf-8", errors="replace"))
        # These helpers sit between TEST_CASEs. Keep their bodies out of preceding
        # producers while preserving upstream provenance line numbers.
        source = re.sub(
            r"static (?:void (?:checkTextLayoutSilver|renderSoloLeafArtboard)|rive::rcp<rive::File> importTextWithMinorVersion|std::pair<float, float> solveTitleLayout)\([^)]*\)\s*\{.*?^\}",
            lambda match: "\n" * match.group(0).count("\n"),
            source,
            flags=re.MULTILINE | re.DOTALL,
        )
        for test_name, test_line, chunk in test_chunks(source):
            for match in LITERAL_MATCH.finditer(chunk):
                silver_id = match.group(1)
                riv_sources = unique(
                    [strip_asset_prefix(value) for value in RIV_STRING.findall(chunk)]
                )
                if silver_id == "gamepad_test":
                    # Its fixture is opened by openReadyStateMachine above the test.
                    riv_sources = ("gamepad_test.riv",)
                primary = riv_sources[0] if riv_sources else "inline-script"
                dependencies = riv_sources[1:]
                serialized_script = path.name == "serialized_scripts_test.cpp"
                lane = (
                    "scripted"
                    if "/scripting/" in f"/{relative}" or serialized_script
                    else "runtime"
                )
                serialized = path.name == "serialized_rendering_test.cpp"
                producer_class = (
                    "serialized-rendering"
                    if serialized
                    else "scripted-literal"
                    if lane == "scripted"
                    else "runtime-literal"
                )
                deterministic = (
                    "enabled"
                    if "deterministicMode = true" in chunk
                    else "cpp-test-defined"
                )
                sample_times = tuple(float(value) for value in SAMPLE_TIME.findall(chunk))
                artboard = infer_selector(
                    ARTBOARD_NAME,
                    chunk,
                    "default" if "artboard" in chunk else "cpp-test-defined",
                )
                if silver_id == "layout_scroll_snap_carousel":
                    artboard = "main"
                animation = infer_selector(
                    ANIMATION_NAME,
                    chunk,
                    "default" if "defaultAnimation" in chunk else "none",
                )
                if silver_id == "text_feather_falloff":
                    animation = "default"
                state_machine = infer_selector(
                    STATE_MACHINE_NAME,
                    chunk,
                    "default"
                    if "defaultStateMachine" in chunk or "stateMachineAt" in chunk
                    else "none",
                )
                if silver_id == "gamepad_test":
                    state_machine = "default"
                view_model = (
                    "cpp-test-defined"
                    if "ViewModel" in chunk
                    or "viewModel" in chunk
                    or "bindViewModelInstance" in chunk
                    else "none"
                )
                if silver_id == "gamepad_test":
                    view_model = "cpp-test-defined"
                actions: str | tuple[dict[str, object], ...] = "cpp-test-body"
                status = "pending-scripted" if lane == "scripted" else "pending"
                blocker = None
                if serialized_script:
                    pointer_log = re.search(r'replayPointerLog\(\s*"([^"]+)"', chunk)
                    if pointer_log is None:
                        raise ValueError(f"{silver_id} has no pointer log")
                    pointer_source = strip_asset_prefix(pointer_log.group(1))
                    dependencies = (*dependencies, pointer_source)
                    actions = (
                        action("bind-authored-view-model-instance", instance_index=0),
                        action("advance", target="state-machine", seconds=0.1),
                        action("draw"),
                        action("replay-pointer-log", source=pointer_source, fps=30.0),
                    )
                if lane == "runtime":
                    actions, blocker = executable_actions(chunk, state_machine, animation)
                    if (ported_actions := focus_flags_actions(silver_id)) is not None:
                        actions, blocker = ported_actions, None
                    if (ported_actions := fl_d4_actions(silver_id)) is not None:
                        actions, blocker = ported_actions, None
                    if (ported_actions := p1q_view_model_actions(silver_id)) is not None:
                        actions, blocker = ported_actions, None
                    if (ported_actions := p1q_pointer_actions(silver_id)) is not None:
                        actions, blocker = ported_actions, None
                    if (ported_actions := fl_e8_list_path_actions(silver_id)) is not None:
                        actions, blocker = ported_actions, None
                    if (ported_actions := p1q_round2_actions(silver_id)) is not None:
                        actions, blocker = ported_actions, None
                    if (ported_actions := p2e_gamepad_actions(silver_id)) is not None:
                        actions, blocker = ported_actions, None
                    if (
                        ported_actions := data_binding_viewmodel_actions(silver_id)
                    ) is not None:
                        actions, blocker = ported_actions, None
                    if silver_id == "layout_hug_artboard":
                        actions = (
                            {"kind": "advance", "target": "artboard", "seconds": 0.0},
                            {"kind": "layout-frame-size"},
                            {"kind": "draw"},
                            {"kind": "frame"},
                            {"kind": "advance", "target": "state-machine", "seconds": 0.016},
                            {"kind": "draw"},
                        )
                        blocker = None
                    if silver_id == "sorted_listeners":
                        # The C++ producer calls
                        # `file->createViewModelInstance(artboard.get())`
                        # rather than selecting the authored default instance
                        # (`state_machine_test.cpp:546`).
                        actions = tuple(
                            {"kind": "bind-fresh-view-model"}
                            if action.get("kind") == "bind-default-view-model"
                            else action
                            for action in actions
                        )
                    if silver_id == "ik_anim_test":
                        # ik_constraint_test.cpp binds createViewModelInstance(
                        # artboard->viewModelId(), 0), then draws at 0, .1,
                        # and four further .5-second advances. Not default VMI.
                        actions = (
                            {"kind": "bind-authored-view-model-instance", "instance_index": 0},
                            {"kind": "advance", "target": "state-machine", "seconds": 0.0},
                            {"kind": "draw"},
                            {"kind": "frame"},
                            {"kind": "advance", "target": "state-machine", "seconds": 0.1},
                            {"kind": "draw"},
                        ) + tuple(
                            action
                            for _ in range(4)
                            for action in (
                                {"kind": "frame"},
                                {"kind": "advance", "target": "state-machine", "seconds": 0.5},
                                {"kind": "draw"},
                            )
                        )
                        blocker = None
                    if silver_id == "gamepad_inputs_test":
                        # focus_test.cpp at 9b9cd7b1: four draws, announcing
                        # device 0 before right-shoulder press/release batches.
                        actions = (
                            action("bind-fresh-view-model"),
                            action("advance", target="state-machine", seconds=0.0),
                            action("assert-focus-child-count", path=[], count=1),
                            action("assert-focus-child-count", path=[0], count=1),
                            action("assert-focus-child-count", path=[0, 0], count=3),
                            action("draw"),
                            action("frame"),
                            action("advance", target="state-machine", seconds=0.016),
                            action("draw"),
                            action("gamepad-batch", records=[{
                                "kind": "connected", "device_id": 0,
                                "mapping": "standard", "button_count": 17, "axis_count": 4,
                            }]),
                        ) + tuple(
                            step
                            for value in (1.0, 0.0)
                            for step in (
                                action("frame"),
                                action("gamepad-batch", records=[{
                                    "kind": "update", "device_id": 0,
                                    "input": "button", "index": 5, "value": value,
                                }]),
                                action("advance", target="state-machine", seconds=0.016),
                                action("draw"),
                            )
                        )
                        blocker = None
                    if silver_id == "gamepad_inputs_test-collapsing":
                        actions = (
                            action("bind-default-view-model"),
                            action("advance", target="state-machine", seconds=0.0),
                            action("draw"),
                        ) + tuple(
                            step
                            for _ in range(2)
                            for step in (
                                action("frame"),
                                action("pointer-down", x=475.0, y=475.0, pointer_id=0),
                                action("pointer-up", x=475.0, y=475.0, pointer_id=0),
                                action("advance", target="state-machine", seconds=0.016),
                                action("draw"),
                            )
                        )
                        blocker = None
                    if silver_id == "additive_blendmode_test":
                        # blend_mode_test.cpp: default artboard, fresh VMI bound
                        # directly to machine 0; zero draw, then .016 and
                        # int(1.0f/.016f) additional frames.
                        actions = (
                            action("frame-size"),
                            action("select-state-machine"),
                            action("bind-selected-artboard-fresh-view-model"),
                            action("advance", target="state-machine", seconds=0.0),
                            action("draw"),
                        ) + tuple(repeated_frames(1 + cpp_float_division_to_int("1.0", "0.016"), 0.016))
                        blocker = None
                    if silver_id == "layout_animation_transition_test":
                        # layout_test.cpp: authored VM 0 (or artboard-created
                        # default), direct machine binding; draw at zero, then
                        # .016 and int(2.0f/.016f) additional frames.
                        actions = (
                            action("frame-size"),
                            action("select-state-machine"),
                            action("bind-selected-artboard-authored-view-model", if_present=False),
                            action("advance", target="state-machine", seconds=0.0),
                            action("draw"),
                        ) + tuple(repeated_frames(1 + cpp_float_division_to_int("2.0", "0.016"), 0.016))
                        blocker = None
                    if silver_id == "layout_order_pointer_test":
                        # layout_test.cpp: fresh VMI, initial draw, then five
                        # pointer clicks and .1-second advances before drawing.
                        actions = (
                            action("bind-fresh-view-model"),
                            action("advance", target="state-machine", seconds=0.0),
                            action("draw"),
                        ) + tuple(
                            step
                            for point in (25.0, 60.0, 150.0, 300.0, 450.0)
                            for step in (
                                action("frame"),
                                action("pointer-down", x=point, y=point, pointer_id=0),
                                action("pointer-up", x=point, y=point, pointer_id=0),
                                action("advance", target="state-machine", seconds=0.1),
                                action("draw"),
                            )
                        )
                        blocker = None
                    if silver_id == "text_background_feather_test":
                        actions = (
                            action("bind-authored-view-model-instance", instance_index=0),
                            action("advance", target="state-machine", seconds=0.032),
                            action("draw"),
                        )
                        blocker = None
                    if silver_id == "text_fit_test":
                        # text_test.cpp: authored VMI 0, draw after .032,
                        # then int(3.0f / .032f) further frames.
                        actions = (
                            action("bind-authored-view-model-instance", instance_index=0),
                            action("advance", target="state-machine", seconds=0.032),
                            action("draw"),
                        ) + tuple(repeated_frames(93, 0.032))
                        blocker = None
                    if silver_id == "color_passthrough_test":
                        # color_channels_test.cpp binds the authored default
                        # view-model instance to state machine 0, draws after
                        # .1 seconds, then adds twelve .25-second frames.
                        actions = (
                            action("bind-default-view-model"),
                            action("advance", target="state-machine", seconds=0.1),
                            action("draw"),
                        ) + tuple(repeated_frames(12, 0.25))
                        blocker = None
                    if silver_id == "global_view_models_scripting_test":
                        # global_viewmodels_test.cpp passes the nullable authored
                        # instance at index 0 directly to bindViewModelInstance.
                        # Do not replace that with default-instance resolution.
                        actions = (
                            action(
                                "bind-authored-view-model-instance",
                                instance_index=0,
                            ),
                            action("advance", target="state-machine", seconds=0.0),
                            action("draw"),
                            action("frame"),
                            action("advance", target="state-machine", seconds=0.1),
                            action("draw"),
                        )
                        blocker = None
                    if (ported_actions := bound_value_replay_actions(silver_id)) is not None:
                        actions, blocker = ported_actions, None
                        if silver_id == "data_bind_blob_test":
                            dependencies = (*dependencies, "data_enum_roundtrip.rml")
                        elif silver_id in ("image_binding_with_listener", "stateful_component_image_test"):
                            dependencies = (*dependencies, "open_source.jpg")
                    if (ported_actions := view_model_rebinding_actions(silver_id)) is not None:
                        actions, blocker = ported_actions, None
                    if (ported_actions := stateful_component_actions(silver_id)) is not None:
                        actions, blocker = ported_actions, None
                    blocker = CLASSIFIED_RUNTIME_BLOCKERS.get(silver_id, blocker)
                    if blocker is not None:
                        status = "unsupported-feature"
                line = test_line + chunk.count("\n", 0, match.start())
                note = (
                    "Serialized-rendering producer is catalogued from the full upstream test "
                    "body; shared action-DSL translation and Rust replay remain pending."
                    if serialized
                    else "Literal upstream producer is catalogued; shared action-DSL "
                    "translation and Rust replay remain pending."
                )
                if lane == "scripted" and silver_id in SCRIPTED_REPLAY_IDS:
                    status = "exact"
                    actions = (action("replay-scripted"),)
                    note = "Literal pinned Luau producer is executable in the shared Silver runner; unchanged full-stream comparison is required."
                    if silver_id == "list_index_script_access":
                        status = "diverges"
                        note = "Layout divergence (stock Taffy with CSS semantics replaces Rive's Yoga): CSS Fill uses 0% basis; preserving the existing scripted-property whole-stream exception; first difference: frame 0, op 70 (transform), field ty: expected 369.66666, got 333.33334."
                    if silver_id in {"script_create_text_runs", "script_create_viewmodel_instance"}:
                        status = "diverges"
                        note = layout_divergence_note(silver_id, REVIEWED_LAYOUT_DIVERGENCES[silver_id])
                elif lane == "scripted" and silver_id in SCRIPTED_EXACT_NOTES:
                    status = "exact"
                    note = SCRIPTED_EXACT_NOTES[silver_id]
                elif serialized_script:
                    status = "diverges"
                    note = (
                        "Literal authored instance-0 binding, 0.1-second advance, initial "
                        "draw, and recorded pointer log replay at 30 fps are executable. "
                        "The recorded first difference is an intentional CSS difference, tracked "
                        "in UNIV-3015; later differences are not claimed individually adjudicated."
                    )
                    difference = reviewed_divergence(silver_id)
                    if difference is not None:
                        layout_note = layout_divergence_note(silver_id, difference)
                        if layout_note is not None:
                            note += " " + layout_note
                elif lane == "scripted":
                    note = (
                        "Scripted producer provenance is catalogued; scripted action/output "
                        "replay is explicitly deferred to the next adoption step."
                    )
                elif blocker is not None:
                    note = (
                        f"Unsupported feature: missing runtime surface {blocker}; the pinned "
                        "C++ action stream requires that unported runtime surface."
                    )
                elif silver_id in EXACT and silver_id not in REVIEWED_LAYOUT_DIVERGENCES:
                    status = "exact"
                    note = (
                        "Rust renderer stream is operation-exact with the pinned C++ silver "
                        "baseline after replaying the TEST_CASE actions."
                    )
                    if silver_id in ("focus_traversal_click_to_focus", "focus_traversal_data_bound", "focusable_element"):
                        note = (
                            "Exact comparison contract for the 9b9cd7b1 focus producer, including explicit focus walks and bound list-item mutations. "
                            "Enrollment alone is not a validation result."
                        )
                    if silver_id in ("gamepad_inputs_test", "gamepad_inputs_test-collapsing"):
                        note = (
                            "Exact comparison contract for the literal gamepad/focus "
                            "producer at 9b9cd7b1. Enrollment alone is not a validation result."
                        )
                    if silver_id == "additive_blendmode_test":
                        note = (
                            "Exact comparison contract for the literal 64-draw additive "
                            "producer at 3d0d3f56: default artboard, machine 0, fresh VMI "
                            "bound directly, zero advance/draw then 63 .016-second frames. "
                            "Enrollment alone is not a validation result."
                        )
                    if silver_id == "layout_animation_transition_test":
                        note = (
                            "Exact comparison contract for the interrupted layout animation "
                            "producer at 621f2a2e: direct authored VM binding, initial zero "
                            "advance/draw, then 125 .016-second advance/draw frames. "
                            "Enrollment alone is not a validation result."
                        )
                    if silver_id == "layout_order_pointer_test":
                        note = (
                            "Exact comparison contract for the six-draw pointer-order "
                            "producer at 61f00897: fresh VMI, initial draw at zero, then "
                            "five clicks with .1-second advances. Enrollment alone is not "
                            "a validation result."
                        )
                    if silver_id in ("layout_solos", "layout_solos_fit_to_layout_parent"):
                        note = (
                            "Exact comparison contract for solo_test.cpp through 3b2c51e2: "
                            "the opt-in sibling asserts false and enables every leaf flag first; bind the authored "
                            "view-model instance (or create the artboard default when unset), "
                            "walk every DataEnum state with a successful setter, then "
                            "advance/apply 0.016, draw and addFrame for each. Enrollment "
                            "alone is not a validation result."
                        )
                    if silver_id == "ik_anim_test":
                        note = (
                            "Exact comparison contract for the literal six-draw IK producer: "
                            "pass the nullable authored view-model instance 0 lookup unchanged "
                            "to binding, advance 0 then 0.1, then "
                            "four 0.5-second frames. Enrollment alone is not a validation result."
                        )
                    if silver_id == "text_background_feather_test":
                        note = (
                            "Exact comparison contract for the one-draw background feather "
                            "producer at d6107a91: authored VMI 0, advance .032, draw. "
                            "Enrollment alone is not a validation result."
                        )
                    if silver_id == "text_fit_test":
                        note = (
                            "Exact comparison contract for the literal 94-draw varying-size "
                            "text producer at 73f94edc. Enrollment alone is not a validation result."
                        )
                    if silver_id == "color_passthrough_test":
                        note = (
                            "Exact comparison contract for the literal thirteen-draw color "
                            "passthrough producer: bind the authored default view-model instance "
                            "to state machine 0, advance 0.1, then twelve 0.25-second frames. "
                            "Enrollment alone is not a validation result."
                        )
                else:
                    difference = reviewed_divergence(silver_id)
                    if difference is None:
                        raise ValueError(
                            f"{silver_id} has executable actions but no Rust result classification"
                        )
                    status = "diverges"
                    layout_note = layout_divergence_note(silver_id, difference)
                    if layout_note is not None:
                        note = layout_note
                    elif silver_id in SERIALIZED_RENDERING_FFP_CONTRACT_DIVERGENCES:
                        note = (
                            "Narrow producer-build-mode divergence: the pinned upstream unit-test "
                            "silvers are built with --no_ffp_contract, while upstream production "
                            "defaults to -ffp-contract=on; first difference: "
                            f"{difference}."
                        )
                    else:
                        note = (
                            "Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE "
                            f"actions; first difference: {difference}."
                        )
                if lane == "scripted" and silver_id in {"scripted_oval", "scripted_animated_oval", "scripted_text_wrapped", "scripted_text_bidi", "scripted_text_glyphs"}:
                    # Exact scripted producers are owned by the literal inline
                    # C++ test body. Any .riv opened by that body is an input
                    # dependency, not the producer source.
                    primary = "inline-script"
                    dependencies = riv_sources
                if silver_id in {
                    "scripted_text_wrapped", "scripted_text_bidi", "scripted_text_glyphs",
                }:
                    # TextTest installs both fonts before executing the literal
                    # script, including scenes that draw with only one font.
                    primary = "inline-script"
                    dependencies = (
                        "fonts/Inter_18pt-Regular.ttf", "IBMPlexSansArabic-Regular.ttf",
                    )
                producers.append(
                    Producer(
                        id=silver_id,
                        source=primary,
                        dependencies=dependencies,
                        artboard=artboard,
                        animation=animation,
                        state_machine=state_machine,
                        lane=lane,
                        deterministic=deterministic,
                        random="deterministic"
                        if deterministic == "enabled"
                        else "cpp-test-defined",
                        view_model=view_model,
                        sample_times=sample_times,
                        actions=actions,
                        status=status,
                        producer_class=producer_class,
                        provenance_file=relative,
                        provenance_test=test_name,
                        producer_line=line,
                        note=note,
                    )
                )
    return producers


def dynamic_producers() -> list[Producer]:
    def actions_for(silver_id: str) -> tuple[dict[str, object], ...]:
        drag_multiplier = "drag_multiplier" in silver_id
        distance = 600.0 if drag_multiplier else 1500.0
        steps = 8 if drag_multiplier else 4
        start_y = "artboard-height/2"
        actions = [
            action("bind-default-view-model"),
            action("advance", target="state-machine", seconds=0.0),
            action("draw"),
            action(
                "pointer-move",
                x="artboard-width/2",
                y=start_y,
                seconds=1.0,
                pointer_id=0,
            ),
            action(
                "pointer-down",
                x="artboard-width/2",
                y=start_y,
                pointer_id=0,
            ),
        ]
        for step in range(1, steps + 1):
            offset = distance * step / steps
            y = f"artboard-height/2-{offset:g}"
            actions += [
                action("frame"),
                action(
                    "pointer-move",
                    x="artboard-width/2",
                    y=y,
                    seconds=float(step + 1),
                    pointer_id=0,
                ),
                action("advance", target="state-machine", seconds=0.016),
                action("draw"),
            ]
        actions += [
            action(
                "pointer-up",
                x="artboard-width/2",
                y=f"artboard-height/2-{distance:g}",
                pointer_id=0,
            ),
            action(
                "advance-draw-until-scroll-physics-stops",
                max_frames=56,
                seconds=0.016,
            ),
        ]
        return tuple(actions)

    layout_scroll = []
    for silver_id, source, artboard, test_name, line in DYNAMIC_LAYOUT_SCROLL:
        if silver_id in EXACT and silver_id not in REVIEWED_LAYOUT_DIVERGENCES:
            status = "exact"
            note = (
                "Rust renderer stream is operation-exact with the pinned C++ silver "
                "baseline after replaying the TEST_CASE actions."
            )
        else:
            difference = reviewed_divergence(silver_id)
            if difference is None:
                raise ValueError(f"{silver_id} has no Rust result classification")
            status = "diverges"
            note = layout_divergence_note(silver_id, difference) or (
                "Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE "
                f"actions; first difference: {difference}."
            )
        layout_scroll.append(
            Producer(
                id=silver_id,
                source=source,
                dependencies=(),
                artboard=artboard,
                animation="none",
                state_machine="default",
                lane="runtime",
                deterministic="enabled",
                random="deterministic",
                view_model="bind-default-if-present",
                sample_times=(0.0, 0.016),
                actions=actions_for(silver_id),
                status=status,
                producer_class="layout-scroll-dynamic",
                provenance_file="tests/unit_tests/runtime/layout_scroll_test.cpp",
                provenance_test=test_name,
                producer_line=line,
                note=note,
                # The pinned helpers call artboardNamed() and then instance().
                clone_artboard_instance=True,
            )
        )

    grid_actions = (
        action("advance", target="state-machine", seconds=0.0),
        action("draw"),
        *(
            item
            for _ in range(120)
            for item in (
                action("frame"),
                action("advance", target="state-machine", seconds=0.016),
                action("draw"),
            )
        ),
    )
    grid_stack = []
    for silver_id, artboard, test_name, line in DYNAMIC_GRID_STACK:
        blocker = CLASSIFIED_RUNTIME_BLOCKERS.get(silver_id)
        if blocker is not None:
            status = "unsupported-feature"
            actions = ()
            note = (
                f"Unsupported feature: missing runtime surface {blocker}; the pinned "
                "C++ gridStackSilver action stream requires that unported runtime surface."
            )
        elif silver_id in EXACT and silver_id not in REVIEWED_LAYOUT_DIVERGENCES:
            status = "exact"
            actions = grid_actions
            note = (
                "Rust renderer stream is operation-exact with the pinned C++ silver "
                "baseline after replaying the gridStackSilver helper actions."
            )
        else:
            difference = reviewed_divergence(silver_id)
            if difference is None:
                raise ValueError(f"{silver_id} has no Rust result classification")
            status = "diverges"
            actions = grid_actions
            note = layout_divergence_note(silver_id, difference) or (
                "Genuine Rust-vs-C++ divergence after replaying the pinned "
                f"gridStackSilver helper actions; first difference: {difference}."
            )
        grid_stack.append(
            Producer(
                id=silver_id,
                source="layout_grid_stack.riv",
                dependencies=(),
                artboard=artboard,
                animation="none",
                state_machine="default",
                lane="runtime",
                deterministic="cpp-test-defined",
                random="cpp-test-defined",
                view_model="none",
                sample_times=(0.0, 0.016),
                actions=actions,
                status=status,
                producer_class="grid-stack-dynamic",
                provenance_file=(
                    "tests/unit_tests/runtime/layout_grid_stack_silver_test.cpp"
                ),
                provenance_test=test_name,
                producer_line=line,
                note=note,
            )
        )
    return layout_scroll + grid_stack


def unknown_producers() -> list[Producer]:
    return [
        Producer(
            id=silver_id,
            source="provenance-unknown",
            dependencies=(),
            artboard="provenance-unknown",
            animation="provenance-unknown",
            state_machine="provenance-unknown",
            lane="unknown",
            deterministic="provenance-unknown",
            random="provenance-unknown",
            view_model="provenance-unknown",
            sample_times=(),
            actions="provenance-unknown",
            status="provenance-unknown",
            producer_class="provenance-unknown",
            provenance_file="",
            provenance_test="",
            producer_line=0,
            note=PROVENANCE_UNKNOWN[silver_id],
        )
        for silver_id in PROVENANCE_UNKNOWN
    ]


def retired_producers() -> list[Producer]:
    return [Producer(
        id="stateful_list_props", source="stateful_list_props.riv", dependencies=(),
        artboard="Main", animation="none", state_machine="default", lane="retired",
        deterministic="retired-upstream", random="retired-upstream", view_model="retired-upstream",
        sample_times=(), actions=(), status="retired-upstream", producer_class="retired-upstream",
        provenance_file="tests/unit_tests/runtime/component_test.cpp",
        provenance_test="Stateful Component list with input/output bridge binds", producer_line=284,
        note=(
            "Known producer deliberately disabled upstream, not an unsupported runtime feature. "
            "Introduced by ea7277bbe1d70583e7f7ebd594ddb0dc6fbc010d; retired by "
            "43a7dfcd7f3458d19fe915b002f0b3e187d8c048; associated implementation removed by "
            "a207d78846610d333e34688daa47acde5f600cad. The producer remains block-commented "
            "at the pinned component_test.cpp:284: per-list-item stateful VM instances were not created. "
            "Baseline Git blob 32b177bc6a5c503af381defae02d8f9e912c6762, 29261 bytes, SHA-256 "
            "de4518d633626451ddb090b434d7a56d7329bb2b5a3bf959f9982bfb0dcb8ab9. "
            "The fixture changed at retirement (4a2fc297 to 291fb45d); replaying the current fixture "
            "against this historical baseline would not reproduce its producer. Retained as an archived "
            "baseline only; never executable or exact coverage. The separate active "
            "stateful_list_props_lifecycle producer remains enrolled."
        ),
    )]


def text_layout_producers(runtime_dir: Path) -> list[Producer]:
    relative = "tests/unit_tests/runtime/text_test.cpp"
    source = strip_cpp_comments((runtime_dir / relative).read_text(encoding="utf-8"))
    calls = re.compile(
        r'checkTextLayoutSilver\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*\)'
    )
    # checkTextLayoutSilver: optional default VMI, advance zero and draw, then
    # four addFrame/advance(.016)/draw iterations. The selected default machine
    # is optional; without it the helper advances the artboard instead.
    actions = (
        action("bind-default-view-model-if-present"),
        action("advance", target="state-machine-or-artboard", seconds=0.0),
        action("draw"),
        *(
            item
            for _ in range(4)
            for item in (
                action("frame"),
                action("advance", target="state-machine-or-artboard", seconds=0.016),
                action("draw"),
            )
        ),
    )
    producers = []
    for test_name, test_line, chunk in test_chunks(source):
        for match in calls.finditer(chunk):
            asset, silver_id = match.groups()
            layout_note = (
                layout_divergence_note(silver_id, reviewed_divergence(silver_id))
                if silver_id in LAYOUT_DIVERGENCES
                else None
            )
            producers.append(Producer(
                id=silver_id,
                source=strip_asset_prefix(asset),
                dependencies=(),
                artboard="default",
                animation="none",
                state_machine="default-if-present",
                lane="runtime",
                deterministic="cpp-test-defined",
                random="cpp-test-defined",
                view_model="bind-default-if-present",
                sample_times=(0.0, 0.016),
                actions=actions,
                status="diverges" if layout_note else "exact",
                producer_class="text-layout-dynamic",
                provenance_file=relative,
                provenance_test=test_name,
                producer_line=test_line + chunk.count("\n", 0, match.start()),
                note=layout_note or (
                    "Exact comparison contract for the five-frame checkTextLayoutSilver "
                    "helper at 4ca3b88a. Enrollment alone is not a validation result."
                ),
            ))
    return producers


def solo_leaf_producers(runtime_dir: Path) -> list[Producer]:
    relative = "tests/unit_tests/runtime/solo_test.cpp"
    source = strip_cpp_comments((runtime_dir / relative).read_text(encoding="utf-8"))
    calls = re.compile(r'renderSoloLeafArtboard\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*(true|false)\s*,\s*(true|false)\s*\)')
    producers = []
    for test_name, test_line, chunk in test_chunks(source):
        for match in calls.finditer(chunk):
            artboard, silver_id, solo, fit = match.groups()
            producers.append(Producer(
                id=silver_id, source="solo_nested_artboard_leaf.riv", dependencies=(),
                artboard=artboard, animation="none", state_machine="default",
                lane="runtime", deterministic="cpp-test-defined", random="cpp-test-defined",
                view_model="bind-authored-if-present", sample_times=(0.016,),
                actions=(action("frame-size"), action("assert-solo-leaf", expect_solo=solo == "true", expect_fit_to_layout_parent=fit == "true"),
                         action("select-state-machine"),
                         action("bind-selected-artboard-authored-view-model", if_present=True),
                         action("advance", target="state-machine", seconds=0.016),
                         action("draw"), action("frame")),
                status="exact", producer_class="solo-leaf-dynamic",
                provenance_file=relative, provenance_test=test_name,
                producer_line=test_line + chunk.count("\n", 0, match.start()),
                note="Literal renderSoloLeafArtboard helper at 3b2c51e2, including authored parent/flag/fit assertions. Enrollment alone is not a validation result.",
            ))
    return producers


def discover(runtime_dir: Path) -> list[Producer]:
    producers = (
        literal_producers(runtime_dir) + dynamic_producers()
        + text_layout_producers(runtime_dir) + solo_leaf_producers(runtime_dir) + unknown_producers() + retired_producers()
    )
    ids = [producer.id for producer in producers]
    duplicates = sorted({silver_id for silver_id in ids if ids.count(silver_id) > 1})
    if duplicates:
        raise ValueError(f"duplicate producer ids: {duplicates}")

    silver_ids = sorted(
        path.stem
        for path in (runtime_dir / "tests/unit_tests/silvers").glob("*.sriv")
    )
    if sorted(ids) != silver_ids:
        unrepresented = sorted(set(silver_ids) - set(ids))
        without_file = sorted(set(ids) - set(silver_ids))
        raise ValueError(
            f"producer/silver mismatch: unrepresented={unrepresented}, "
            f"without_file={without_file}"
        )
    return sorted(producers, key=lambda producer: producer.id)


def verify_upstream_ref(runtime_dir: Path) -> None:
    try:
        actual = subprocess.run(
            ["git", "-C", str(runtime_dir), "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
    except (OSError, subprocess.CalledProcessError) as error:
        raise ValueError(f"cannot resolve upstream ref at {runtime_dir}: {error}") from error
    if actual != UPSTREAM_REF:
        raise ValueError(f"upstream ref is {actual}; expected {UPSTREAM_REF}")


def render_action_value(value: object) -> str:
    if isinstance(value, str):
        return quoted(value)
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, float):
        if value.is_integer():
            return f"{value:.1f}"
        return format(value, ".9g")
    if isinstance(value, dict):
        fields = ", ".join(
            f"{quoted(str(key))} = {render_action_value(item)}"
            for key, item in value.items()
        )
        return "{ " + fields + " }"
    if isinstance(value, (list, tuple)):
        return "[" + ", ".join(render_action_value(item) for item in value) + "]"
    raise TypeError(f"unsupported action value {value!r}")


def render_actions(actions: str | tuple[dict[str, object], ...]) -> str:
    if isinstance(actions, str):
        return quoted(actions)
    rendered = []
    for item in actions:
        fields = ", ".join(
            f"{key} = {render_action_value(value)}" for key, value in item.items()
        )
        rendered.append(f"{{ {fields} }}")
    return "[" + ", ".join(rendered) + "]"


def render(producers: list[Producer]) -> str:
    single_ids = REVIEWED_LAYOUT_STREAM_SHA256.keys()
    target_ids = REVIEWED_LAYOUT_STREAM_SHA256_BY_TARGET.keys()
    if single_ids & target_ids:
        raise ValueError("reviewed stream must use exactly one hash representation")
    if REVIEWED_LAYOUT_DIVERGENCES.keys() != single_ids | target_ids:
        raise ValueError("reviewed layout difference and full stream hash IDs must match")
    for silver_id, stream_hash in REVIEWED_LAYOUT_STREAM_SHA256.items():
        if not re.fullmatch(r"[0-9a-f]{64}", stream_hash):
            raise ValueError(f"{silver_id} needs a full Rust stream SHA-256")
    for silver_id, hashes in REVIEWED_LAYOUT_STREAM_SHA256_BY_TARGET.items():
        if not hashes or hashes.keys() - {"linux-x86_64", "macos-aarch64"}:
            raise ValueError(f"{silver_id} needs supported target stream hashes")
        if any(not re.fullmatch(r"[0-9a-f]{64}", value) for value in hashes.values()):
            raise ValueError(f"{silver_id} needs full target Rust stream SHA-256 hashes")
    runtime = sum(producer.lane == "runtime" for producer in producers)
    scripted = sum(producer.lane == "scripted" for producer in producers)
    unknown = sum(producer.status == "provenance-unknown" for producer in producers)
    retired = sum(producer.lane == "retired" for producer in producers)
    if (len(producers), runtime, scripted, unknown, retired) != (282, 231, 48, 2, 1):
        raise ValueError(
            "ratchet mismatch: "
            f"entries={len(producers)} runtime={runtime} scripted={scripted} unknown={unknown} retired={retired}"
        )

    lines = [
        "# Generated by tools/silver-corpus/generate_manifest.py.",
        "# Runtime entries carry executable action streams or named feature blockers.",
        "",
        "[corpus]",
        "version = 1",
        f"upstream_ref = {quoted(UPSTREAM_REF)}",
        "expected_entries = 282",
        "expected_runtime = 231",
        "expected_scripted = 48",
        "max_provenance_unknown = 2",
        f"min_cpp_rust_exact = {len(EXACT)}",
        "cpp_rust_exact_ids = ["
        + ", ".join(quoted(silver_id) for silver_id in sorted(EXACT))
        + "]",
        "",
    ]
    # Keep every historical exact ID and the numeric floor. Reviewed CSS
    # cases satisfy that obligation only through an explicit, exact replay pin.
    lines.extend(["reviewed_layout_divergences = ["])
    for silver_id, difference in sorted(REVIEWED_LAYOUT_DIVERGENCES.items()):
        if silver_id not in EXACT:
            raise ValueError(f"{silver_id} is not a historical exact obligation")
        if silver_id in REVIEWED_LAYOUT_STREAM_SHA256_BY_TARGET:
            hashes = REVIEWED_LAYOUT_STREAM_SHA256_BY_TARGET[silver_id]
            stream_pin = ", rust_stream_sha256_by_target = { " + ", ".join(
                quoted(target) + " = " + quoted(value)
                for target, value in sorted(hashes.items())
            ) + " }"
        else:
            stream_pin = ", rust_stream_sha256 = " + quoted(REVIEWED_LAYOUT_STREAM_SHA256[silver_id])
        lines.append("  { id = " + quoted(silver_id) + ", first_difference = " + quoted(difference)
                     + stream_pin + " },")
    lines.extend(["]", ""])
    for producer in producers:
        lines.extend(
            [
                "[[case]]",
                f"id = {quoted(producer.id)}",
                "expected = "
                + quoted(f"tests/unit_tests/silvers/{producer.id}.sriv"),
                f"source = {quoted(producer.source)}",
                "dependencies = ["
                + ", ".join(quoted(value) for value in producer.dependencies)
                + "]",
                f"artboard = {quoted(producer.artboard)}",
                *(
                    ["clone_artboard_instance = true"]
                    if producer.clone_artboard_instance
                    else []
                ),
                f"animation = {quoted(producer.animation)}",
                f"state_machine = {quoted(producer.state_machine)}",
                f"lane = {quoted(producer.lane)}",
                f"deterministic = {quoted(producer.deterministic)}",
                f"random = {quoted(producer.random)}",
                f"view_model = {quoted(producer.view_model)}",
                "sample_times = ["
                + ", ".join(format(value, ".9g") for value in producer.sample_times)
                + "]",
                "actions = " + render_actions(producer.actions),
                'verification = "sriv-v1-epsilon"',
                f"status = {quoted(producer.status)}",
                f"producer_class = {quoted(producer.producer_class)}",
                f"provenance_file = {quoted(producer.provenance_file)}",
                f"provenance_test = {quoted(producer.provenance_test)}",
                f"producer_line = {producer.producer_line}",
                f"note = {quoted(producer.note)}",
                "",
            ]
        )
    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--rive-runtime-dir",
        type=Path,
        default=Path(
            os.environ.get("RIVE_RUNTIME_DIR", "/Users/levi/dev/oss/rive-runtime")
        ),
    )
    parser.add_argument("--output", type=Path, default=Path("silver-corpus.toml"))
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--skip-ref-check", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if not args.skip_ref_check:
        verify_upstream_ref(args.rive_runtime_dir)
    generated = render(discover(args.rive_runtime_dir))
    if args.check:
        existing = args.output.read_text(encoding="utf-8")
        if existing != generated:
            raise SystemExit(
                f"{args.output} is stale; run tools/silver-corpus/generate_manifest.py"
            )
        print(f"{args.output}: generated manifest is current")
        return 0
    args.output.write_text(generated, encoding="utf-8")
    print(f"wrote {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
