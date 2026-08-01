#!/usr/bin/env python3
"""Generate silver-corpus.toml from the pinned upstream C++ producers.

Literal matches() calls are discovered mechanically. The six layout-scroll
names assembled through helper arguments are deliberately listed below, and
the two producerless files are deliberately classified as provenance-unknown.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path

UPSTREAM_REF = "d788e8ec6e8b598526607d6a1e8818e8b637b60c"
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

PROVENANCE_UNKNOWN = ("interpolator", "multitouch_debug")
FORCED_BLOCKERS = {
    "db_health_tracker": "runtime-frame-loop-nontermination",
    "echo_show_demo": "renderer-paint-allocation",
    "bidirectional_binding_source": "external-bindable-artboard-registration",
    "data_bind_artboard_input": "external-bindable-artboard-registration",
    "data_viz_demo": "runtime-frame-loop-nontermination-after-nested-view-model-mutation",
    "global_variables_test": "runtime-frame-loop-nontermination-with-global-view-models",
}
CLASSIFIED_RUNTIME_BLOCKERS = {
    "multi_listeners": (
        "runtime-script-asset-vm-instantiation-and-occurrence-attachment"
    ),
    "bindable_artboard_nesty": "external-bindable-artboard-with-bound-view-model-injection",
    "data_binding_artboards_default_test": "external-bindable-artboard-registration",
    "data_binding_artboards_test": "external-bindable-artboard-registration",
    "databind_external_artboard_main": "external-bindable-artboard-and-view-model-graph-registration",
    "databind_viewmodel": "runtime-owned-view-model-reference-replacement-by-instance-handle",
    "image_binding_with_listener": "live-decoded-image-view-model-payload-injection",
    "list_to_path": "runtime-owned-heterogeneous-list-path-item-graph-construction",
    "multi_listeners-rebind": "runtime-owned-nested-view-model-reference-replacement-by-handle",
    "replace_vm_instance": "runtime-owned-shared-view-model-graph-reparenting",
    "replace_vm_instance-double-nest": "runtime-owned-nested-list-view-model-reference-replacement",
    "replace_vm_instance-list": "runtime-owned-list-view-model-reference-replacement",
    "stateful_artboard_swap": "live-bindable-artboard-value-and-bound-view-model-swap",
    "stateful_list_props": "stateful-component-list-bridge-observation-for-dynamic-items",
    "stateful_list_props_lifecycle": "stateful-component-list-bridge-detach-and-rebind-lifecycle",
    "stateful_source_switch": "live-bindable-artboard-source-swap-with-stateful-child-borrowing",
    "transition_self_comparator_test": "runtime-owned-composite-list-and-view-model-comparator-mutation",
    "rebind_with_nested_viewmodel": "runtime-owned-nested-view-model-reference-replacement-by-handle",
    "gamepad_test": "serialized-gamepad-buffer-ingestion-and-device-state-tracking",
    "layout_hug_artboard": "top-level-computed-layout-width-height-exposure",
    "layout_scroll_snap_carousel": "scroll-constraint-physics-running-state-exposure",
}
EXACT = (
    "hittest_ab_text_parent",
    "lock_icon_demo",
    "text_listener_simpler",
    "collapse_data_binds-test_3",
    "databind_solo_to_enum",
    "listener_view_model",
    "viewmodel_image_reset",
    "zero_width_space_line_break",
    "advance_blend_mode-inputs",
    "advance_blend_mode-vms",
    "animated_clipping-layout",
    "artboard_list_map_rules",
    "artboard_width_test",
    "bidirectional_precedence-source_first",
    "component_based_conditions",
    "component_based_conditions-Artboard2",
    "component_list_follow_path_distance",
    "component_list_follow_path",
    "data_bind_font_test",
    "component_stateful",
    "custom_property_trigger_bind",
    "computed_root_transform-nested_artboard",
    "custom_property_enum",
    "data_converter_to_number",
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
    "group_effect-main-missing-targets",
    "hittest_ab1",
    "hittest_ab1_grand_parent",
    "hittest_ab1_parent",
    "hittest_collapsed_layouts",
    "hittest_nested",
    "image_fit_alignment_3",
    "image_fit_alignment_updated_test",
    "list_items",
    "list_to_length_test",
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
    "sorted_listeners",
    "spotify_kids_app_icon",
    "stacked_path_effects",
    "state_transition_fire_trigger",
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
)


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
        for _ in range(int(0.25 / 0.016)):
            actions += [
                frame,
                pointer("pointer-move", "artboard-width/2", y, seconds=0.016),
                advance(0.016),
                draw,
            ]
            y -= 40.0
        actions += [
            frame,
            pointer("pointer-move", "artboard-width/2", y, seconds=0.016),
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
        actions = [bind, advance(0.1), draw]
        for _ in range(7):
            actions += [frame, action("focus-next"), advance(0.1), draw]
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
        for processed, count, focus in ((False, 1.0, True), (False, 2.0, False), (False, 3.0, False)):
            actions += [
                action("set-view-model-boolean", property="stageProcessed", value=processed),
                action("set-view-model-number", property="stageCount", value=count),
                advance(0.016), draw, frame,
            ]
            if focus:
                actions += [action("focus-next"), advance(0.016), draw, frame]
        actions += [action("focus-next"), advance(0.016), draw, frame, action("focus-next"), advance(0.016), draw]
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
        return (bind, advance(0.1), draw, frame, action("set-view-model-number", property="n1", value=500.0), advance(0.1), draw, frame, advance(0.016), draw)

    if silver_id == "data_viz_demo":
        actions = [bind, advance(0.1), draw, action("set-view-model-number", property="item1/value", value=20.0)]
        actions += repeated_frames(30, 0.064)
        return tuple(actions)

    if silver_id == "data_bind_artboard_input":
        return (
            action("bind-fresh-view-model"), draw, advance(0.1), draw,
            frame, advance(0.1), draw, frame, advance(0.1), draw,
            frame, advance(0.1), draw,
            action("set-view-model-artboard", property="artboardProperty", value=1),
            frame, advance(0.1), draw,
            action("set-view-model-artboard", property="artboardProperty", value=10),
            frame, advance(0.1), draw,
        )

    if silver_id == "bidirectional_binding_source":
        actions = [action("create-default-view-model"), action("set-view-model-boolean", property="costume_db_bool", value=True), action("bind-prepared-view-model"), advance(0.0), draw]
        actions += repeated_frames(9, 0.016)
        return tuple(actions)

    if silver_id == "global_variables_test":
        actions = [bind, advance(0.1), draw]
        actions += repeated_frames(int(1.0 / 0.016), 0.016)
        return tuple(actions)

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
        actions = [bind, advance(0.1), draw, frame, advance(0.1), draw]
        for artboard_name in (
            "recursive-grand-child-1",
            "recursive-parent",
            "recursive-grand-parent",
            "recursive-grand-child-2",
        ):
            actions += [
                frame,
                action(
                    "set-view-model-artboard-by-name",
                    property="ab",
                    artboard=artboard_name,
                ),
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
        for _ in range(int(3.0 / 0.016)):
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
        return (
            bind,
            advance(0.1),
            draw,
            frame,
            advance(0.016),
            draw,
            action(
                "set-view-model-asset-by-name",
                property="imageProperty",
                asset="image2",
            ),
            advance(0.0),
            frame,
            advance(0.016),
            draw,
            action(
                "set-view-model-asset-by-name",
                property="imageProperty",
                asset="image3",
            ),
            advance(0.0),
            frame,
            advance(0.016),
            draw,
        )

    return None


DIVERGENCES = dict(
    line.split("|", 1)
    for line in """
animated_clipping-nodes|frame 10, op 328 (drawPath): expected drawPath, got makeRenderPath
component_list_hit_order|frame 1, op 106 (color): expected color, got save
component_list_grouped|frame 13, op 746 (color): expected color, got save
component_list_virtualized_scroll_manual|frame 2, op 384 (color): expected color, got makeRenderPaint
data_binding_artboards_test_recursive|frame 1, op 118 (makeRenderPaint): expected makeRenderPaint, got frame
deterministic_mode|frame 0, op 25 (transform), field xy: expected -0.0 (0x80000000), got 0
draw_index_list|frame 0, op 35 (color): expected color, got makeRenderPaint
focus_collapsing|frame 3, op 192 (color), field paint_id: expected 6, got 11
formula_random-always|frame 1, op 44 (transform), field tx: expected 10, got 521.8384
formula_random-once|frame 1, op 44 (transform), field tx: expected 10, got 510.0007
formula_random-source_change|frame 1, op 44 (transform), field tx: expected 10, got 521.8384
global_viewmodels_test-set_instance|frame 1, op 163 (frame): expected frame, got color
hittest_ab_2_non_virtualized|frame 0, op 198 (color): expected color, got save
hittest_ab_2_virtualized|frame 0, op 132 (color): expected color, got save
image_fit_alignment|frame 2, op 115 (transform), field tx: expected 462.03198, got -197.96802
interactive_scrolling|frame 0, op 42 (transform), field xy: expected -0.0 (0x80000000), got 0
interpolate_to_end|frame 1, op 63 (addRawPath): expected 954 fields, got 975
keyboard_listener|frame 0, op 85 (color): expected color, got save
keyboard_listener-KeyboardInput|frame 1, op 214 (color): expected color, got save
juice|frame 0, op 40 (blendMode): expected blendMode, got makeRenderPaint
list_focus_order|frame 0, op 78 (addRawPath), field point: expected (-0.0 (0x80000000), 137.20052), got (-0.0 (0x80000000), 137.20053)
layout_scroll_visibility|frame 0, op 130 (transform), field xy: expected -0.0 (0x80000000), got 0
multitouch_enter-MainList|frame 1, op 179 (color): expected color, got save
relative_data_bind_path-fire-trigger|frame 1, op 48 (color): expected color, got save
relative_data_bind_path-listener|frame 1, op 72 (makeRenderPath): expected makeRenderPath, got drawPath
relative_data_bind_path-scripted-input|frame 0, op 39 (transform), field tx: expected 115.56351, got 250
scroll_intent|frame 0, op 69 (transform), field xy: expected -0.0 (0x80000000), got 0
scroll_test|frame 0, op 56 (transform), field xy: expected -0.0 (0x80000000), got 0
text_feather_falloff|frame 0, op 29 (feather): expected feather, got save
ai_assitant|frame 0, op 82 (makeLinearGradient): expected makeLinearGradient, got feather
artboard_list_overrides_horizontal|frame 1, op 303 (rewind): expected rewind, got drawPath
artboard_list_overrides_vertical|frame 1, op 303 (rewind): expected rewind, got drawPath
bankcard|frame 0, op 22 (blendMode): expected blendMode, got makeRenderPaint
bidirectional_precedence-target_first|frame 0, op 24 (transform), field tx: expected 252.5, got 100
car_widgets_v01|frame 0, op 222 (blendMode): expected blendMode, got makeRenderPaint
clear_viewmodel_list|frame 0, op 10 (makeRenderPaint): expected makeRenderPaint, got save
clipping_and_draw_order|frame 2, op 161 (transform), field tx: expected 0, got 1121
collapsable_data_binding|frame 0, op 14 (save): expected save, got color
collapse_data_binds-test_1|frame 10, op 760 (rewind): expected rewind, got drawPath
collapse_data_binds-test_2|frame 15, op 315 (addRawPath): expected 151 fields, got 256
collapsing_elements|frame 2, op 943 (rewind): expected rewind, got drawPath
component_list_child_origin|frame 0, op 315 (transform), field xy: expected -0.0 (0x80000000), got 0
component_stateful_vm_instance|frame 2, op 109 (addRawPath), field point: expected (0, -100), got (0, -50)
component_stateful_vm_instance_2|frame 2, op 96 (transform), field xx: expected 0.97985506, got 0.994951
computed_root_transform-list|frame 1, op 255 (rewind): expected rewind, got drawPath
computed_values_test|frame 0, op 54 (addRawPath), field point: expected (256.2, -0.0 (0x80000000)), got (245, -0.0 (0x80000000))
data_bind_solo-solos-to-values|frame 0, op 81 (addRawPath): expected 752 fields, got 669
data_bind_keyframes_test|frame 4, op 159 (save): expected save, got restore
data_converter_interpolator_reset|frame 1, op 30 (save): expected save, got color
drag_event|frame 23, op 602 (save): expected save, got color
focus_traversal|frame 0, op 95 (color): expected color, got save
fit_font_size_test|frame 2, op 199 (makeRenderPath): expected makeRenderPath, got rewind
global_viewmodels_test-auto_instance|frame 0, op 27 (color): expected color, got save
group_effect|frame 0, op 46 (addRawPath): expected 163 fields, got 3
hide_test|frame 0, op 50 (color), field paint_id: expected 14, got 10
hittest_ab1|frame 1, op 153 (color): expected color, got save
hittest_ab1_grand_parent|frame 2, op 304 (color): expected color, got save
hittest_ab1_parent|frame 1, op 192 (color): expected color, got save
hittest_ab_shape_parent|frame 3, op 353 (save): expected save, got color
hittest_nested|frame 1, op 155 (save): expected save, got color
hunter_x_demo|frame 0, op 488 (blendMode): expected blendMode, got makeRenderPaint
image_fit_alignment_2|frame 1, op 95 (setVertexBufferData): expected setVertexBufferData, got save
interpolation_zero_duration|frame 1, op 38 (transform), field tx: expected 0, got 200
layout_anim_bound|frame 2, op 146 (addRawPath), field point: expected (450, 0), got (250, 0)
layout_anim_component_list|frame 1, op 89 (addRawPath), field point: expected (500, 0), got (495.2, 0)
layout_anim_nested|frame 1, op 86 (addRawPath), field point: expected (500, 0), got (495.2, 0)
layout_aspect_ratio|frame 0, op 42 (addRawPath), field point: expected (142, 71), got (142, 133)
layout_display|frame 3, op 188 (makeRenderPath): expected makeRenderPath, got rewind
layout_fixed_fill|frame 1, op 57 (addRawPath), field point: expected (300, 0), got (150, 0)
layout_paint|frame 0, op 77 (drawPath): expected drawPath, got makeRenderPath
multi_listeners|frame 2, op 253 (makeRenderPath): expected makeRenderPath, got rewind
multitouch_enter-MultiScroll|frame 0, op 95 (transform), field xy: expected -0.0 (0x80000000), got 0
nested_artboard_quantize_and_speed|frame 0, op 75 (transform), field xx: expected 0.95105654, got 1
nested_events|frame 1, op 166 (makeRenderPath): expected makeRenderPath, got rewind
number_to_list_nested_children|frame 0, op 141 (color): expected color, got save
path_effect_with_feathers|frame 0, op 21 (feather), field paint_id: expected 8, got 5
pointer_exit|frame 31, op 1173 (save): expected save, got color
reset_phase_multi_main|frame 0, op 25 (color): expected color, got makeRenderPaint
rewards_demo|frame 0, op 22 (blendMode): expected blendMode, got makeRenderPaint
scroll_threshold-all-scroll|frame 0, op 82 (transform), field xy: expected -0.0 (0x80000000), got 0
scroll_threshold-horizontal-scroll|frame 0, op 79 (transform), field xy: expected -0.0 (0x80000000), got 0
scroll_threshold-vertical-scroll|frame 0, op 69 (transform), field xy: expected -0.0 (0x80000000), got 0
spotify_kids_demo|frame 0, op 200 (blendMode): expected blendMode, got makeRenderPaint
stateful_keyed_trigger|frame 1, op 30 (color): expected color, got save
stateful_multi_property|frame 1, op 134 (rewind): expected rewind, got drawPath
stateful_nested|frame 0, op 39 (color), field paint_id: expected 15, got 10
superbowl|frame 0, op 2825 (color), field paint_id: expected 220, got 208
text_input|frame 0, op 25 (transform), field xy: expected -0.0 (0x80000000), got 0
text_vertical_trim_test|frame 3, op 219 (transform), field ty: expected 177.93579, got 182.76001
time_based_interpolation|frame 1, op 65 (transform), field tx: expected 250.07309, got 250.29443
transition_artboard_condition_test|frame 0, op 16 (frameSize), field width: expected 983, got 984
unbound_stateful_component|frame 0, op 9 (color), field value: expected 4278255360, got 4278190080
virtualize_blendmode|frame 0, op 33 (color): expected color, got save
virtualized_artboard_databound_children|frame 5, op 365 (makeRenderPaint): expected makeRenderPaint, got save
word_joiner_test|frame 2, op 262 (transform), field ty: expected -39.996094, got -15.796875
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
    source = re.sub(r"/\*.*?\*/", "", source, flags=re.DOTALL)
    return re.sub(r"//[^\n]*", "", source)


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
            else int(
                float(match.group("numerator")) / float(match.group("denominator"))
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
        source = path.read_text(encoding="utf-8", errors="replace")
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
                lane = "scripted" if "/scripting/" in f"/{relative}" else "runtime"
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
                actions: str | tuple[dict[str, object], ...] = "cpp-test-body"
                status = "pending-scripted" if lane == "scripted" else "pending"
                blocker = None
                if lane == "runtime":
                    actions, blocker = executable_actions(chunk, state_machine, animation)
                    if (ported_actions := fl_d4_actions(silver_id)) is not None:
                        actions, blocker = ported_actions, None
                    if (ported_actions := p1q_view_model_actions(silver_id)) is not None:
                        actions, blocker = ported_actions, None
                    if (ported_actions := p1q_pointer_actions(silver_id)) is not None:
                        actions, blocker = ported_actions, None
                    if (ported_actions := p1q_round2_actions(silver_id)) is not None:
                        actions, blocker = ported_actions, None
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
                    blocker = FORCED_BLOCKERS.get(
                        silver_id, CLASSIFIED_RUNTIME_BLOCKERS.get(silver_id, blocker)
                    )
                    if blocker in FORCED_BLOCKERS.values():
                        actions = ()
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
                if lane == "scripted":
                    note = (
                        "Scripted producer provenance is catalogued; scripted action/output "
                        "replay is explicitly deferred to the next adoption step."
                    )
                elif blocker is not None:
                    note = (
                        f"Unsupported feature: missing runtime surface {blocker}; the pinned "
                        "C++ action stream requires that unported runtime surface."
                    )
                elif silver_id in EXACT:
                    status = "exact"
                    note = (
                        "Rust renderer stream is operation-exact with the pinned C++ silver "
                        "baseline after replaying the TEST_CASE actions."
                    )
                else:
                    difference = DIVERGENCES.get(silver_id)
                    if difference is None:
                        raise ValueError(
                            f"{silver_id} has executable actions but no Rust result classification"
                        )
                    status = "diverges"
                    note = (
                        "Genuine Rust-vs-C++ divergence after replaying the pinned TEST_CASE "
                        f"actions; first difference: {difference}."
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
    return [
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
            actions=(),
            status="unsupported-feature",
            producer_class="layout-scroll-dynamic",
            provenance_file="tests/unit_tests/runtime/layout_scroll_test.cpp",
            provenance_test=test_name,
            producer_line=line,
            note=(
                "Unsupported feature: scroll-constraint-physics-running-state-exposure; "
                "the pinned helper terminates from ScrollConstraint::physics()->isRunning(), "
                "which has no public Rust runtime surface."
            ),
        )
        for silver_id, source, artboard, test_name, line in DYNAMIC_LAYOUT_SCROLL
    ]


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
            note="No producer/reference exists in the pinned upstream runtime tests.",
        )
        for silver_id in PROVENANCE_UNKNOWN
    ]


def discover(runtime_dir: Path) -> list[Producer]:
    producers = literal_producers(runtime_dir) + dynamic_producers() + unknown_producers()
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
    runtime = sum(producer.lane == "runtime" for producer in producers)
    scripted = sum(producer.lane == "scripted" for producer in producers)
    unknown = sum(producer.status == "provenance-unknown" for producer in producers)
    if (len(producers), runtime, scripted, unknown) != (238, 195, 41, 2):
        raise ValueError(
            "ratchet mismatch: "
            f"entries={len(producers)} runtime={runtime} scripted={scripted} unknown={unknown}"
        )

    lines = [
        "# Generated by tools/silver-corpus/generate_manifest.py.",
        "# Runtime entries carry executable action streams or named feature blockers.",
        "",
        "[corpus]",
        "version = 1",
        f"upstream_ref = {quoted(UPSTREAM_REF)}",
        "expected_entries = 238",
        "expected_runtime = 195",
        "expected_scripted = 41",
        "max_provenance_unknown = 2",
        f"min_cpp_rust_exact = {len(EXACT)}",
        "cpp_rust_exact_ids = ["
        + ", ".join(quoted(silver_id) for silver_id in sorted(EXACT))
        + "]",
        "",
    ]
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
        default=Path("/Users/levi/dev/oss/rive-runtime"),
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
