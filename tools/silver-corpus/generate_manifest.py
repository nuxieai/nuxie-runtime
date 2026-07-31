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
}
EXACT = (
    "animated_clipping-layout",
    "artboard_list_map_rules",
    "component_list_follow_path_distance",
    "component_stateful",
    "custom_property_enum",
    "data_bind_solo-values-to-solos",
    "databind_artboard",
    "event_trigger_event",
    "fill_trim_path",
    "focus_test",
    "follow_path_animate_shape",
    "follow_path_constraint",
    "format_number_with_commas",
    "group_effect-main-missing-targets",
    "hittest_collapsed_layouts",
    "image_fit_alignment_3",
    "image_fit_alignment_updated_test",
    "multitouch",
    "multitouch_enter",
    "n_slice_triangle",
    "nested_artboard_origin_override_test",
    "nested_hug",
    "nested_needs_advance",
    "pause_nested_artboard",
    "recursive_data_bind",
    "relative_data_binding",
    "stacked_path_effects",
    "target_event",
    "text_follow_path_shape_length",
    "transition_index_condition",
    "vertical_align_ellipsis",
    "viewmodel_list_trigger",
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


DIVERGENCES = dict(
    line.split("|", 1)
    for line in """
advance_blend_mode-inputs|frame 0, op 19 (color), field value: expected 4286928727, got 4282614325
advance_blend_mode-vms|frame 0, op 19 (color), field value: expected 4286928727, got 4283534636
animated_clipping-nodes|frame 10, op 328 (drawPath): expected drawPath, got rewind
artboard_list_overrides_horizontal|frame 1, op 303 (rewind): expected rewind, got drawPath
artboard_list_overrides_vertical|frame 1, op 303 (rewind): expected rewind, got drawPath
artboard_width_test|frame 0, op 13 (color), field value: expected 4291035136, got 4280270848
bankcard|frame 0, op 22 (blendMode): expected blendMode, got makeRenderPaint
clear_viewmodel_list|frame 0, op 10 (makeRenderPaint): expected makeRenderPaint, got save
clipping_and_draw_order|frame 2, op 161 (transform), field tx: expected 0, got 1121
collapse_data_binds-test_1|frame 10, op 721 (drawPath): expected drawPath, got rewind
collapse_data_binds-test_2|frame 1, op 76 (makeRenderPath): expected makeRenderPath, got rewind
collapsing_elements|frame 2, op 943 (rewind): expected rewind, got drawPath
component_list_child_origin|frame 0, op 315 (transform), field xy: expected -0.0 (0x80000000), got 0
computed_root_transform-list|frame 1, op 206 (drawPath): expected drawPath, got rewind
computed_root_transform-nested_artboard|frame 1, op 144 (drawPath): expected drawPath, got rewind
computed_values_test|frame 0, op 54 (addRawPath), field point: expected (256.2, -0.0 (0x80000000)), got (245, -0.0 (0x80000000))
data_bind_solo-solos-to-values|frame 0, op 81 (addRawPath): expected 752 fields, got 669
data_converter_interpolator_reset|frame 1, op 30 (save): expected save, got color
data_converter_to_number|frame 1, op 110 (makeRenderPath): expected makeRenderPath, got rewind
focus_traversal|frame 0, op 95 (color): expected color, got save
follow_path_animate_solo|frame 125, op 2406 (clipPath): expected clipPath, got rewind
follow_path_animate_target|frame 1, op 150 (clipPath): expected clipPath, got rewind
global_viewmodels_test-auto_instance|frame 0, op 27 (color): expected color, got save
hide_test|frame 0, op 45 (color), field paint_id: expected 4, got 13
hittest_ab1|frame 1, op 153 (color): expected color, got save
hittest_ab1_grand_parent|frame 2, op 304 (color): expected color, got save
hittest_ab1_parent|frame 1, op 192 (color): expected color, got save
hittest_nested|frame 1, op 155 (save): expected save, got color
hunter_x_demo|frame 0, op 488 (blendMode): expected blendMode, got makeRenderPaint
image_fit_alignment_2|frame 1, op 95 (setVertexBufferData): expected setVertexBufferData, got save
interpolation_zero_duration|frame 1, op 38 (transform), field tx: expected 0, got 200
layout_anim_bound|frame 2, op 146 (addRawPath), field point: expected (450, 0), got (250, 0)
layout_anim_component_list|frame 1, op 89 (addRawPath), field point: expected (500, 0), got (495.2, 0)
layout_anim_nested|frame 1, op 86 (addRawPath), field point: expected (500, 0), got (495.2, 0)
layout_aspect_ratio|frame 0, op 42 (addRawPath), field point: expected (142, 71), got (142, 133)
layout_display|frame 3, op 173 (drawPath): expected drawPath, got rewind
layout_paint|frame 0, op 77 (drawPath): expected drawPath, got makeRenderPath
list_items|frame 1, op 105 (drawPath): expected drawPath, got rewind
list_to_length_test|frame 1, op 139 (drawPath): expected drawPath, got rewind
multi_listeners|frame 2, op 253 (makeRenderPath): expected makeRenderPath, got rewind
nested_events|frame 1, op 166 (makeRenderPath): expected makeRenderPath, got rewind
number_to_list_nested_children|frame 0, op 141 (color): expected color, got save
path_effect_with_feathers|frame 0, op 21 (feather), field paint_id: expected 8, got 5
reset_phase_multi_main|frame 0, op 25 (color): expected color, got makeRenderPaint
saturation|frame 1, op 96 (makeRenderPath): expected makeRenderPath, got rewind
sorted_listeners|frame 0, op 32 (restore): expected restore, got save
spotify_kids_app_icon|frame 1, op 285 (clipPath): expected clipPath, got rewind
spotify_kids_demo|frame 0, op 200 (blendMode): expected blendMode, got makeRenderPaint
state_transition_fire_trigger|frame 1, op 127 (makeRenderPath): expected makeRenderPath, got rewind
stateful_keyed_trigger|frame 1, op 30 (color): expected color, got save
superbowl|frame 0, op 2825 (color), field paint_id: expected 220, got 610
text_input|frame 0, op 25 (transform), field xy: expected -0.0 (0x80000000), got 0
text_stroke_test|frame 1, op 55 (makeRenderPath): expected makeRenderPath, got rewind
text_vertical_trim_test|frame 3, op 219 (transform), field ty: expected 177.93579, got 182.76001
time_based_interpolation|frame 1, op 65 (transform), field tx: expected 250.07309, got 250.29443
transition_actions|frame 2, op 72 (makeRenderPath): expected makeRenderPath, got rewind
transition_artboard_condition_test|frame 0, op 16 (frameSize), field width: expected 983, got 984
transition_duration_bind_list|frame 0, op 13 (makeRenderPaint): expected makeRenderPaint, got frame
transition_duration_bind_nested|frame 0, op 57 (color): expected color, got frame
trigger_based_listeners|frame 1, op 85 (makeRenderPath): expected makeRenderPath, got rewind
trigger_fires_single_change|frame 1, op 67 (makeRenderPath): expected makeRenderPath, got rewind
unbound_stateful_component|frame 0, op 9 (color), field value: expected 4278255360, got 4278190080
viewmodel_based_condition|frame 0, op 24 (color), field paint_id: expected 6, got 5
virtualize_blendmode|frame 0, op 33 (color): expected color, got save
""".strip().splitlines()
)

CPP_NUMBER = r"[-+]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)(?:f)?"
POINTER_CALL_PATTERN = (
    r"(?P<pointer>\b\w+->(?P<pointer_method>pointerDown|pointerMove|pointerUp|pointerExit)"
    r"\s*\(\s*(?:rive::)?Vec2D\s*\(\s*(?P<pointer_x>"
    + CPP_NUMBER
    + r")\s*,\s*(?P<pointer_y>"
    + CPP_NUMBER
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
            x = float(match.group("pointer_x").removesuffix("f"))
            y = float(match.group("pointer_y").removesuffix("f"))
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
                    blocker = FORCED_BLOCKERS.get(silver_id, blocker)
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
                        f"Unsupported feature: {blocker}; the upstream C++ body cannot yet be "
                        "replayed faithfully by the Rust action interpreter."
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
                "Unsupported feature: layout-scroll-physics; helper-local coordinates and "
                "physics-settlement control flow are not yet executable by the interpreter."
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
