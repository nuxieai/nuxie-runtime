#!/usr/bin/env python3
"""Fail-closed checker for the C++-corresponding runtime frame-loop port."""

from __future__ import annotations

import argparse
import collections
import fnmatch
import functools
import hashlib
import json
import pathlib
import re
import subprocess
import sys
import tempfile
import tomllib
from typing import Any, Iterable


TOOL_DIR = pathlib.Path(__file__).resolve().parent
if str(TOOL_DIR) not in sys.path:
    sys.path.insert(0, str(TOOL_DIR))

STATUSES = {
    "faithful",
    "adapted",
    "divergent-by-decision",
    "pending",
    "compensation",
}
CLOSED_STATUSES = {"faithful", "adapted", "divergent-by-decision"}
LIFECYCLE_PHASES = (
    "construct",
    "retain",
    "dirty",
    "update",
    "advance",
    "draw",
    "clone",
    "drop",
)
CITATION_RE = re.compile(r"^(cpp|rust):(.+):(\d+)(?:-(\d+))?$")
UNBOUND_SCRIPTED_CONSTRUCTOR_RATCHET = (
    "scripted_object_unbound_constructor_enters_live_context"
)
SEMANTIC_ORDINAL_PROJECTION_RATCHET = (
    "state_machine_semantic_ordinal_projection"
)
NESTED_EVENT_OWNER_BOUNDARY_RATCHETS = {
    "state_machine_nested_event_collection_outside_instance_policy": "collection",
    "state_machine_nested_animation_selection_outside_instance_policy": "selection",
    "state_machine_nested_event_dispatch_outside_instance_policy": "dispatch",
    "state_machine_nested_audio_unwind_outside_instance_policy": "audio",
}
NESTED_EVENT_OWNER_BOUNDARY_DETECTOR = "rust_nested_event_owner_boundary"
NESTED_EVENT_OWNER_MODULE = pathlib.PurePosixPath(
    "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs"
)
NESTED_EVENT_OWNER_DETECTOR_MANIFEST = (
    TOOL_DIR / "rust-owner-detector" / "Cargo.toml"
)
FL_B_FROZEN_SCOPE_REF = "d788e8ec6e8b598526607d6a1e8818e8b637b60c"
FL_B_FROZEN_SCOPE_FILES = frozenset(
    {
        "src/animation/animation_reset.cpp",
        "src/animation/animation_reset_factory.cpp",
        "src/animation/animation_state.cpp",
        "src/animation/animation_state_instance.cpp",
        "src/animation/blend_animation.cpp",
        "src/animation/blend_animation_1d.cpp",
        "src/animation/blend_animation_direct.cpp",
        "src/animation/blend_state.cpp",
        "src/animation/blend_state_1d.cpp",
        "src/animation/blend_state_1d_input.cpp",
        "src/animation/blend_state_1d_instance.cpp",
        "src/animation/blend_state_1d_viewmodel.cpp",
        "src/animation/blend_state_direct.cpp",
        "src/animation/blend_state_direct_instance.cpp",
        "src/animation/blend_state_transition.cpp",
        "src/animation/cubic_ease_interpolator.cpp",
        "src/animation/cubic_interpolator.cpp",
        "src/animation/cubic_interpolator_component.cpp",
        "src/animation/cubic_interpolator_solver.cpp",
        "src/animation/cubic_value_interpolator.cpp",
        "src/animation/elastic_ease.cpp",
        "src/animation/elastic_interpolator.cpp",
        "src/animation/interpolating_keyframe.cpp",
        "src/animation/keyed_object.cpp",
        "src/animation/keyed_property.cpp",
        "src/animation/keyframe.cpp",
        "src/animation/keyframe_bool.cpp",
        "src/animation/keyframe_callback.cpp",
        "src/animation/keyframe_color.cpp",
        "src/animation/keyframe_double.cpp",
        "src/animation/keyframe_id.cpp",
        "src/animation/keyframe_interpolator.cpp",
        "src/animation/keyframe_string.cpp",
        "src/animation/keyframe_uint.cpp",
        "src/animation/linear_animation.cpp",
        "src/animation/linear_animation_instance.cpp",
        "src/animation/nested_animation.cpp",
        "src/animation/nested_bool.cpp",
        "src/animation/nested_linear_animation.cpp",
        "src/animation/nested_number.cpp",
        "src/animation/nested_remap_animation.cpp",
        "src/animation/nested_simple_animation.cpp",
        "src/animation/nested_trigger.cpp",
        "src/animation/property_recorder.cpp",
        "src/importers/keyed_property_importer.cpp",
    }
)
FL_E8_FILES = frozenset(
    {
        "src/shapes/list_path.cpp",
        "src/text/raw_text.cpp",
        "src/text/text_modifier.cpp",
        "src/text/text_style.cpp",
        "src/text/text_style_feature.cpp",
        "src/text/text_target_modifier.cpp",
        "src/text/text_variation_modifier.cpp",
    }
)
FL_E8_WP1_FILES = frozenset(
    {
        "src/text/text_modifier.cpp",
        "src/text/text_style.cpp",
        "src/text/text_style_feature.cpp",
        "src/text/text_target_modifier.cpp",
        "src/text/text_variation_modifier.cpp",
    }
)
FL_E8_WP2_FILES = FL_E8_WP1_FILES | {"src/shapes/list_path.cpp"}
FL_E8_WP3_FILES = FL_E8_WP1_FILES | frozenset({"src/text/raw_text.cpp"})
FL_E8_WAVE_FILES = FL_E8_WP2_FILES | FL_E8_WP3_FILES
FL_E8_FORBIDDEN_DECISIONS = frozenset({"D13", "D14", "D15"})
FL_E8_FORBIDDEN_CEILINGS = (
    "dynamic-list-path",
    "standalone-raw-text",
    "static-text-extensions",
)
FL_E_W120_PENDING_VERIFICATION_FILES = frozenset(
    {
        "src/layout.cpp",
        "src/layout/artboard_component_list_override.cpp",
        "src/layout_component.cpp",
        "src/math/random.cpp",
        "src/solo.cpp",
        "src/text/font_hb.cpp",
        "src/text/fully_shaped_text.cpp",
        "src/text/glyph_lookup.cpp",
        "src/text/line_breaker.cpp",
        "src/text/text.cpp",
        "src/text/text_engine.cpp",
        "src/text/text_follow_path_modifier.cpp",
        "src/text/text_modifier_group.cpp",
        "src/text/text_modifier_range.cpp",
        "src/text/text_style_axis.cpp",
        "src/text/text_variation_helper.cpp",
        "src/text/utf.cpp",
    }
)


class CheckFailure(Exception):
    """Raised when the frame-loop proof is incomplete or inconsistent."""


RUST_RAW_STRING_START = re.compile(r"(?:br|r)(?P<hashes>#{0,255})\"")
RUST_CHAR_LITERAL = re.compile(
    r"'(?:"
    r"\\(?:[nrt0\\'\"]|x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f_]{1,6}\})"
    r"|[^'\\\n]"
    r")'"
)


@functools.lru_cache(maxsize=512)
def strip_rust_comments_and_strings(source: str) -> str:
    """Replace Rust comments/string contents while preserving offsets/newlines."""

    result = list(source)
    index = 0
    block_depth = 0
    while index < len(source):
        if block_depth:
            if source.startswith("/*", index):
                result[index : index + 2] = "  "
                block_depth += 1
                index += 2
            elif source.startswith("*/", index):
                result[index : index + 2] = "  "
                block_depth -= 1
                index += 2
            else:
                if source[index] != "\n":
                    result[index] = " "
                index += 1
            continue
        if source.startswith("//", index):
            end = source.find("\n", index)
            end = len(source) if end == -1 else end
            result[index:end] = " " * (end - index)
            index = end
            continue
        if source.startswith("/*", index):
            result[index : index + 2] = "  "
            block_depth = 1
            index += 2
            continue
        raw = RUST_RAW_STRING_START.match(source, index)
        if raw is not None:
            hashes = raw.group("hashes")
            content_start = raw.end()
            terminator = '"' + hashes
            end = source.find(terminator, content_start)
            end = len(source) if end == -1 else end + len(terminator)
            for offset in range(index, end):
                if source[offset] != "\n":
                    result[offset] = " "
            index = end
            continue
        if source[index] == '"':
            cursor = index + 1
            while cursor < len(source):
                if source[cursor] == "\\":
                    cursor += 2
                    continue
                cursor += 1
                if source[cursor - 1] == '"':
                    break
            for offset in range(index, min(cursor, len(source))):
                if source[offset] != "\n":
                    result[offset] = " "
            index = cursor
            continue
        if source[index] == "'":
            char_literal = RUST_CHAR_LITERAL.match(source, index)
            if char_literal is not None:
                end = char_literal.end()
                result[index:end] = " " * (end - index)
                index = end
                continue
        index += 1
    return "".join(result)


def unbound_scripted_constructor_hits(source: str) -> list[int]:
    """Find constructor paths that enter live binding without a top-level guard."""

    function_pattern = re.compile(
        r"(?m)^[ \t]*(?:(?:pub(?:\([^\n)]*\))?|const|async|unsafe|"
        r"extern(?:[ \t]+\"[^\n\"]*\")?)[ \t]+)*fn[ \t]+"
        r"instantiate_script_listener_actions_with_optional_factory\b"
    )
    functions = list(function_pattern.finditer(source))
    if len(functions) != 1:
        return [functions[0].start()] if functions else [0]
    function = functions[0]
    function_indent = len(function.group(0)) - len(function.group(0).lstrip(" \t"))
    suffix = strip_rust_comments_and_strings(source[function.start() :])
    open_brace = suffix.find("{")
    if open_brace == -1:
        return [function.start()]
    brace_depth = 0
    paren_depth = 0
    bracket_depth = 0
    close_brace = None
    brace_depths: list[int] = [0] * len(suffix)
    paren_depths: list[int] = [0] * len(suffix)
    bracket_depths: list[int] = [0] * len(suffix)
    for index, character in enumerate(suffix):
        brace_depths[index] = brace_depth
        paren_depths[index] = paren_depth
        bracket_depths[index] = bracket_depth
        if character == "{":
            brace_depth += 1
        elif character == "}":
            brace_depth -= 1
            if index > open_brace and brace_depth == 0:
                close_brace = index
                break
        elif character == "(":
            paren_depth += 1
        elif character == ")":
            paren_depth -= 1
        elif character == "[":
            bracket_depth += 1
        elif character == "]":
            bracket_depth -= 1
    if close_brace is None:
        return [function.start()]

    body = suffix[open_brace + 1 : close_brace]
    body_brace_depths = brace_depths[open_brace + 1 : close_brace]
    body_paren_depths = paren_depths[open_brace + 1 : close_brace]
    body_bracket_depths = bracket_depths[open_brace + 1 : close_brace]

    def has_plain_context(offset: int, expected_brace_depth: int) -> bool:
        return (
            body_brace_depths[offset] == expected_brace_depth
            and body_paren_depths[offset] == 0
            and body_bracket_depths[offset] == 0
        )

    def line_indent(offset: int) -> int | None:
        line_start = body.rfind("\n", 0, offset) + 1
        prefix = body[line_start:offset]
        if prefix.strip():
            return None
        return len(prefix)

    def exact_token(token: str) -> int | None:
        offsets = [match.start() for match in re.finditer(re.escape(token), body)]
        if len(offsets) != 1:
            return None
        return offsets[0]

    retry = exact_token("retry_cold_scripted_objects_during_constructor")
    converter = exact_token("instantiate_state_machine_data_converters")
    if retry is None or converter is None:
        return [function.start()]
    retry_is_top_level = (
        has_plain_context(retry, 1)
        and line_indent(retry) == function_indent + 4
    )
    retry_is_direct_else = (
        has_plain_context(retry, 2)
        and line_indent(retry) == function_indent + 8
    )
    if retry_is_direct_else:
        stack: list[int] = []
        for index, character in enumerate(body[:retry]):
            if character == "{":
                stack.append(index)
            elif character == "}" and stack:
                stack.pop()
        retry_is_direct_else = bool(stack) and re.search(
            r"\}[ \t\r\n]*else[ \t\r\n]*\{[ \t\r\n]*$",
            body[: stack[-1] + 1],
        ) is not None
    if not (retry_is_top_level or retry_is_direct_else):
        return [function.start()]
    if not (
        has_plain_context(converter, 1)
        and line_indent(converter) == function_indent + 4
    ):
        return [function.start()]

    guard_pattern = re.compile(
        r"(?m)^(?P<indent>[ \t]*)if[ \t\r\n]+"
        r"!machine\.has_scripted_listener_data_context\(\)"
        r"[ \t\r\n]*\{[ \t\r\n]*return[ \t\r\n]+Ok\(\(\)\);"
        r"[ \t\r\n]*\}"
    )
    guards = [
        guard
        for guard in guard_pattern.finditer(body)
        if has_plain_context(guard.start(), 1)
        and len(guard.group("indent")) == function_indent + 4
    ]
    if len(guards) != 1:
        return [function.start()]
    guard = guards[0]
    if not (retry < guard.start() < converter):
        return [function.start()]
    if re.search(r"#[ \t\r\n]*\[", body[retry : guard.start()]):
        return [function.start()]
    return []


def semantic_ordinal_projection_hits(
    source: str,
    *,
    resolver_seam_exists: bool,
) -> list[int]:
    """Find renamed SemanticData ordinal scans when the repo owns the seam."""

    if not resolver_seam_exists:
        return []
    stripped = strip_rust_comments_and_strings(source)
    function_pattern = re.compile(
        r"(?m)^[ \t]*(?:(?:pub(?:\([^\n)]*\))?|const|async|unsafe|"
        r"extern(?:[ \t]+\"[^\n\"]*\")?)[ \t]+)*fn[ \t]+[A-Za-z_]\w*"
    )
    hits: list[int] = []
    for function in function_pattern.finditer(stripped):
        open_brace = stripped.find("{", function.end())
        if open_brace == -1:
            continue
        semicolon = stripped.find(";", function.end(), open_brace)
        if semicolon != -1:
            continue
        depth = 0
        close_brace = None
        for index in range(open_brace, len(stripped)):
            character = stripped[index]
            if character == "{":
                depth += 1
            elif character == "}":
                depth -= 1
                if depth == 0:
                    close_brace = index
                    break
        if close_brace is None:
            continue
        body = source[open_brace + 1 : close_brace]
        if '"SemanticData"' not in body:
            continue
        filtered_nth = re.search(
            r"\.(?:iter|into_iter)\s*\(\)[\s\S]{0,800}"
            r"(?:filter|filter_map)\s*\([\s\S]{0,500}\"SemanticData\""
            r"[\s\S]{0,500}\.nth\s*\(",
            body,
        )
        ordinal_loop = re.search(
            r"\bfor\b[\s\S]{0,500}\.(?:iter|enumerate)\s*\(\)"
            r"[\s\S]{0,800}\"SemanticData\"",
            body,
        )
        increment = re.search(
            r"\b([A-Za-z_]\w*)\s*(?:\+=\s*1|=\s*\1\.saturating_add\s*\(\s*1\s*\))",
            body,
        )
        compared = (
            increment is not None
            and re.search(
                rf"(?:\b{re.escape(increment.group(1))}\b\s*=="
                rf"|==\s*\b{re.escape(increment.group(1))}\b)",
                body,
            )
            is not None
        )
        if filtered_nth is not None or (
            ordinal_loop is not None and increment is not None and compared
        ):
            hits.append(function.start())
    return hits


@functools.lru_cache(maxsize=1)
def rust_nested_event_owner_detector_binary() -> pathlib.Path:
    """Build the syn-based ownership resolver into a content-addressed cache."""

    if not NESTED_EVENT_OWNER_DETECTOR_MANIFEST.is_file():
        raise CheckFailure(
            "missing syn-based nested-event ownership detector manifest"
        )
    helper_dir = NESTED_EVENT_OWNER_DETECTOR_MANIFEST.parent
    digest = hashlib.sha256()
    workspace_root = TOOL_DIR.parents[1]
    for path in [workspace_root / "Cargo.toml", workspace_root / "Cargo.lock"]:
        digest.update(path.name.encode())
        digest.update(path.read_bytes())
    for path in sorted(helper_dir.rglob("*")):
        if path.is_file():
            digest.update(path.relative_to(helper_dir).as_posix().encode())
            digest.update(path.read_bytes())
    target_dir = (
        pathlib.Path(tempfile.gettempdir())
        / "nuxie-runtime-frame-loop-owner-detector"
        / digest.hexdigest()
    )
    executable = target_dir / "debug/runtime-frame-loop-owner-detector"
    if executable.is_file():
        return executable
    result = subprocess.run(
        [
            "cargo",
            "build",
            "--quiet",
            "--locked",
            "--manifest-path",
            str(NESTED_EVENT_OWNER_DETECTOR_MANIFEST),
            "--target-dir",
            str(target_dir),
        ],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0 or not executable.is_file():
        raise CheckFailure(
            "cannot build syn-based nested-event ownership detector: "
            + result.stderr.strip()
        )
    return executable


@functools.lru_cache(maxsize=1024)
def rust_nested_event_owner_analysis(
    source: str,
    guarded_aliases: tuple[tuple[str, str], ...] = (),
) -> tuple[
    dict[str, tuple[int, ...]],
    dict[str, tuple[int, ...]],
    frozenset[tuple[str, str]],
    dict[str, tuple[tuple[int, int, str, str, str], ...]],
]:
    """Resolve guarded ownership paths with syn and return fail-closed hits."""

    result = subprocess.run(
        [
            str(rust_nested_event_owner_detector_binary()),
            *(f"{kind}:{alias}" for kind, alias in guarded_aliases),
        ],
        input=source,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise CheckFailure(
            "syn-based nested-event ownership detector failed: "
            + result.stderr.strip()
        )
    hits: dict[str, list[int]] = {
        "collection": [],
        "selection": [],
        "dispatch": [],
        "audio": [],
    }
    sites: dict[str, list[int]] = {
        "collection": [],
        "selection": [],
        "dispatch": [],
        "audio": [],
    }
    exports: set[tuple[str, str]] = set()
    matches: dict[str, list[tuple[int, int, str, str, str]]] = {
        "collection": [],
        "selection": [],
        "dispatch": [],
        "audio": [],
    }
    for line in result.stdout.splitlines():
        fields = line.split()
        if (
            len(fields) == 3
            and fields[0] == "export"
            and fields[1] in hits
        ):
            exports.add((fields[1], fields[2]))
        elif (
            len(fields) == 3
            and fields[0] in {"hit", "site"}
            and fields[1] in hits
            and fields[2].isdigit()
        ):
            target = hits if fields[0] == "hit" else sites
            target[fields[1]].append(int(fields[2]))
        elif (
            len(fields) == 7
            and fields[0] == "match"
            and fields[1] in matches
            and fields[2].isdigit()
            and fields[3].isdigit()
            and re.fullmatch(r"[0-9a-f]{64}", fields[6]) is not None
        ):
            matches[fields[1]].append(
                (
                    int(fields[2]),
                    int(fields[3]),
                    fields[4],
                    fields[5],
                    fields[6],
                )
            )
        else:
            raise CheckFailure(
                f"invalid syn-based nested-event ownership detector output: {line!r}"
            )
    return (
        {kind: tuple(sorted(set(offsets))) for kind, offsets in hits.items()},
        {kind: tuple(sorted(set(offsets))) for kind, offsets in sites.items()},
        frozenset(exports),
        {
            kind: tuple(sorted(set(records)))
            for kind, records in matches.items()
        },
    )


def nested_event_owner_exports(source: str) -> set[tuple[str, str]]:
    """Collect guarded names exported from the dedicated owner module."""

    return set(rust_nested_event_owner_analysis(source)[2])


def nested_event_owner_boundary_hits(
    source: str,
    kind: str,
    *,
    guarded_aliases: Iterable[tuple[str, str]] = (),
    count_sites: bool = False,
) -> list[int]:
    """Find resolved or fail-closed event mechanics outside the owner module."""

    aliases = tuple(sorted(set(guarded_aliases)))
    hits, sites, _, _ = rust_nested_event_owner_analysis(source, aliases)
    selected = sites if count_sites else hits
    if kind not in selected:
        raise ValueError(f"unknown nested-event boundary detector kind: {kind}")
    return list(selected[kind])


def nested_event_owner_boundary_matches(
    source: str,
    kind: str,
    *,
    guarded_aliases: Iterable[tuple[str, str]] = (),
) -> list[tuple[int, int, str, str, str]]:
    """Return exact enclosing-item/name matches for registry validation."""

    aliases = tuple(sorted(set(guarded_aliases)))
    _, _, _, matches = rust_nested_event_owner_analysis(source, aliases)
    if kind not in matches:
        raise ValueError(f"unknown nested-event boundary detector kind: {kind}")
    return list(matches[kind])


def read_toml(path: pathlib.Path) -> dict[str, Any]:
    try:
        with path.open("rb") as source:
            return tomllib.load(source)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise CheckFailure(f"cannot read {path}: {error}") from error


def git_head(path: pathlib.Path) -> str:
    result = subprocess.run(
        ["git", "-C", str(path), "rev-parse", "HEAD"],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise CheckFailure(
            f"cannot resolve upstream HEAD at {path}: {result.stderr.strip()}"
        )
    return result.stdout.strip()


def validate_trace_artifacts(
    trace: dict[str, Any], ledger: dict[str, Any], errors: list[str]
) -> None:
    required_keys = {
        "cpp_binary_sha256",
        "cpp_coverage_sha256",
        "cpp_mechanism_coverage_sha256",
        "cpp_steady_coverage_sha256",
        "rust_binary_sha256",
        "rust_coverage_sha256",
        "rust_mechanism_coverage_sha256",
        "rust_steady_coverage_sha256",
    }
    expected = ledger.get("expected_trace_artifacts")
    artifacts = trace.get("artifacts")
    if not isinstance(expected, dict) or set(expected) != required_keys:
        errors.append(
            "ownership ledger expected trace artifact hashes do not match the v2 schema"
        )
        return
    if not isinstance(artifacts, dict) or set(artifacts) != required_keys:
        errors.append("trace evidence artifact hashes do not match the v2 schema")
        return
    invalid = sorted(
        key
        for key in required_keys
        if not isinstance(artifacts[key], str)
        or re.fullmatch(r"[0-9a-f]{64}", artifacts[key]) is None
        or not isinstance(expected[key], str)
        or re.fullmatch(r"[0-9a-f]{64}", expected[key]) is None
    )
    if invalid:
        errors.append(
            "trace evidence artifact hashes do not match the v2 schema: "
            + ", ".join(invalid)
        )
        return
    if artifacts != expected:
        errors.append(
            "trace evidence artifact hashes do not match the ownership packet manifest"
        )


def duplicate_values(values: Iterable[str]) -> list[str]:
    counts = collections.Counter(values)
    return sorted(value for value, count in counts.items() if count > 1)


def manifest_rust_modules(row: dict[str, Any]) -> list[str]:
    return [
        value.strip()
        for value in str(row.get("rust_module", "")).split(";")
        if value.strip()
    ]


def validate_scatter_ratchet(
    *,
    manifest: dict[str, Any],
    rows: list[dict[str, Any]],
    errors: list[str],
) -> tuple[int, int | None]:
    multi_module_rows = [
        row for row in rows if len(manifest_rust_modules(row)) >= 2
    ]
    for row in multi_module_rows:
        note = str(row.get("note", ""))
        if re.search(r"\b(?:MR|exception)\b", note, re.IGNORECASE) is not None:
            continue
        upstream = str(row.get("upstream", "")) or "<missing upstream>"
        row_id = str(row.get("b6_row_id", ""))
        row_name = f"{row_id} ({upstream})" if row_id else upstream
        errors.append(
            f"multi-module row {row_name} must have an MR or exception marker in note"
        )

    count = len(multi_module_rows)
    ratchet = manifest.get("scatter_ratchet")
    if not isinstance(ratchet, dict):
        errors.append("file correspondence scatter_ratchet table is missing")
        return count, None
    maximum = ratchet.get("max_multi_module_rows")
    if not isinstance(maximum, int) or isinstance(maximum, bool) or maximum < 0:
        errors.append(
            "scatter_ratchet.max_multi_module_rows must be a non-negative integer"
        )
        return count, None
    if count > maximum:
        errors.append(f"scatter ratchet increased to {count} > {maximum}")
    return count, maximum


def validate_citation(
    citation: str,
    repo_root: pathlib.Path,
    rive_runtime_dir: pathlib.Path,
    errors: list[str],
) -> None:
    match = CITATION_RE.fullmatch(citation)
    if match is None:
        errors.append(f"invalid citation (expected cpp|rust:path:line): {citation}")
        return
    root_kind, relative, start_text, end_text = match.groups()
    root = rive_runtime_dir if root_kind == "cpp" else repo_root
    source = root / relative
    if not source.is_file():
        errors.append(f"citation file does not exist: {citation}")
        return
    line_count = sum(1 for _ in source.open(encoding="utf-8", errors="replace"))
    start = int(start_text)
    end = int(end_text or start_text)
    if start < 1 or end < start or end > line_count:
        errors.append(
            f"citation line is outside {relative} (1..{line_count}): {citation}"
        )


def topological_order(waves: list[dict[str, Any]], errors: list[str]) -> list[str]:
    ids = [str(wave.get("id", "")) for wave in waves]
    duplicates = duplicate_values(ids)
    if duplicates:
        errors.append(f"duplicate wave ids: {', '.join(duplicates)}")
    known = set(ids)
    incoming: dict[str, set[str]] = {}
    sequence: dict[str, int] = {}
    for wave in waves:
        wave_id = str(wave.get("id", ""))
        if not wave_id:
            errors.append("wave has an empty id")
            continue
        deps = {str(value) for value in wave.get("depends_on", [])}
        missing = sorted(deps - known)
        if missing:
            errors.append(
                f"wave {wave_id} has unknown dependencies: {', '.join(missing)}"
            )
        incoming[wave_id] = deps & known
        value = wave.get("sequence")
        if not isinstance(value, int):
            errors.append(f"wave {wave_id} has no integer sequence")
        else:
            sequence[wave_id] = value
    duplicates = duplicate_values(str(value) for value in sequence.values())
    if duplicates:
        errors.append(f"duplicate wave sequence values: {', '.join(duplicates)}")
    for wave_id, deps in incoming.items():
        for dependency in deps:
            if sequence.get(dependency, 0) >= sequence.get(wave_id, 0):
                errors.append(
                    f"wave {wave_id} must follow dependency {dependency} in sequence"
                )
    work = {key: set(value) for key, value in incoming.items()}
    order: list[str] = []
    while len(order) < len(work):
        ready = sorted(
            (key for key, value in work.items() if not value and key not in order),
            key=lambda key: sequence.get(key, 0),
        )
        if not ready:
            errors.append(
                "wave dependency cycle: "
                + ", ".join(sorted(key for key in work if key not in order))
            )
            break
        for current in ready:
            order.append(current)
            for dependencies in work.values():
                dependencies.discard(current)
    return order


def check_status(
    *,
    subject: str,
    row: dict[str, Any],
    porting_rules: str,
    decision_ids: set[str],
    decision_ceilings: dict[str, str],
    require_closed: bool,
    errors: list[str],
) -> str:
    status = str(row.get("status", ""))
    if status not in STATUSES:
        errors.append(f"{subject} has invalid status {status!r}")
        return status
    if require_closed and status not in CLOSED_STATUSES:
        errors.append(f"closed frame loop required but {subject} is {status}")
    rule = str(row.get("rule", ""))
    decision = str(row.get("decision", ""))
    ceiling = str(row.get("ceiling", ""))
    if status == "adapted":
        if not re.fullmatch(r"(?:AF|RF|FLR)-\d+", rule):
            errors.append(f"{subject} is adapted without an AF/RF/FLR rule")
        elif f"**{rule} " not in porting_rules:
            errors.append(f"{subject} cites missing PORTING.md rule {rule}")
    elif rule:
        errors.append(f"{subject} is {status} but unexpectedly cites rule {rule}")
    if status == "divergent-by-decision":
        if not decision:
            errors.append(f"{subject} is divergent-by-decision without a D-row")
        elif decision not in decision_ids:
            errors.append(f"{subject} cites unknown decision {decision}")
        if not ceiling:
            errors.append(
                f"{subject} is divergent-by-decision without a named ceiling"
            )
        elif decision_ceilings.get(decision) != ceiling:
            errors.append(
                f"{subject} cites ceiling {ceiling!r}, but decision {decision} "
                f"binds {decision_ceilings.get(decision)!r}"
            )
        elif f"**{ceiling}**" not in porting_rules:
            errors.append(
                f"{subject} cites named ceiling {ceiling!r} missing from PORTING.md"
            )
    elif decision:
        errors.append(f"{subject} is {status} but unexpectedly cites {decision}")
    elif ceiling:
        errors.append(f"{subject} is {status} but unexpectedly cites ceiling {ceiling}")
    return status


def expand_source_sets(
    *,
    source_sets: list[dict[str, Any]],
    manifest_files: dict[str, dict[str, Any]],
    repo_root: pathlib.Path,
    rive_runtime_dir: pathlib.Path,
    wave_ids: set[str],
    errors: list[str],
) -> tuple[dict[str, str], dict[str, str]]:
    assignments: dict[str, str] = {}
    source_set_waves: dict[str, str] = {}
    all_cpp = sorted(
        path.relative_to(rive_runtime_dir).as_posix()
        for path in (rive_runtime_dir / "src").rglob("*.cpp")
        if "/generated/" not in path.as_posix()
    )
    for row in source_sets:
        set_id = str(row.get("id", ""))
        wave = str(row.get("wave", ""))
        include = [str(value) for value in row.get("include", [])]
        exclude = [str(value) for value in row.get("exclude", [])]
        if not set_id:
            errors.append("source_set has an empty id")
            continue
        if wave not in wave_ids:
            errors.append(f"source_set {set_id} has unknown wave {wave!r}")
        source_set_waves[set_id] = wave
        if not include:
            errors.append(f"source_set {set_id} has no include patterns")
            continue
        if not str(row.get("static_closure", "")).strip():
            errors.append(f"source_set {set_id} has no static_closure rationale")
        matches = [
            path
            for path in all_cpp
            if any(fnmatch.fnmatchcase(path, pattern) for pattern in include)
            and not any(fnmatch.fnmatchcase(path, pattern) for pattern in exclude)
        ]
        if not matches:
            errors.append(f"source_set {set_id} matches no pinned C++ files")
        rust_modules = [str(value) for value in row.get("rust_modules", [])]
        if not rust_modules:
            errors.append(f"source_set {set_id} has no Rust modules")
        for rust_module in rust_modules:
            if not (repo_root / rust_module).is_file():
                errors.append(
                    f"source_set {set_id} Rust module does not exist: {rust_module}"
                )
        for path in matches:
            if path in assignments:
                errors.append(
                    f"C++ file {path} is assigned by both {assignments[path]} and {set_id}"
                )
                continue
            assignments[path] = set_id
            manifest = manifest_files.get(path)
            if manifest is None:
                errors.append(f"C++ file {path} is absent from file correspondence")
    return assignments, source_set_waves


def validate_frozen_wave_scopes(
    *,
    rows: list[dict[str, Any]],
    assignments: dict[str, str],
    source_set_waves: dict[str, str],
    wave_ids: set[str],
    upstream_ref: str,
    errors: list[str],
) -> None:
    seen_waves: set[str] = set()
    for row in rows:
        wave = str(row.get("wave", ""))
        if wave in seen_waves:
            errors.append(f"duplicate frozen wave scope for {wave!r}")
            continue
        seen_waves.add(wave)
        if wave not in wave_ids:
            errors.append(f"frozen wave scope has unknown wave {wave!r}")

        files = [str(value) for value in row.get("files", [])]
        expected_file_count = row.get("expected_file_count")
        if not isinstance(expected_file_count, int) or expected_file_count < 1:
            errors.append(
                f"frozen wave {wave} has invalid expected_file_count "
                f"{expected_file_count!r}"
            )
        elif len(files) != expected_file_count:
            errors.append(
                f"frozen wave {wave} declares {len(files)} files; "
                f"expected {expected_file_count}"
            )
        duplicates = duplicate_values(files)
        if duplicates:
            errors.append(
                f"frozen wave {wave} has duplicate files: {', '.join(duplicates)}"
            )

        expanded_files = {
            path
            for path, source_set in assignments.items()
            if source_set_waves.get(source_set) == wave
        }
        frozen_files = set(files)
        if wave == "FL-B" and upstream_ref == FL_B_FROZEN_SCOPE_REF:
            if expected_file_count != len(FL_B_FROZEN_SCOPE_FILES):
                errors.append(
                    "pinned frozen wave FL-B expected_file_count must remain "
                    f"{len(FL_B_FROZEN_SCOPE_FILES)}"
                )
            if frozen_files != FL_B_FROZEN_SCOPE_FILES:
                missing = sorted(FL_B_FROZEN_SCOPE_FILES - frozen_files)
                unexpected = sorted(frozen_files - FL_B_FROZEN_SCOPE_FILES)
                errors.append(
                    "pinned frozen wave FL-B literal membership differs from "
                    f"{FL_B_FROZEN_SCOPE_REF}: missing={missing!r}, "
                    f"unexpected={unexpected!r}"
                )
        if expanded_files != frozen_files:
            missing = sorted(frozen_files - expanded_files)
            unexpected = sorted(expanded_files - frozen_files)
            errors.append(
                f"frozen wave {wave} membership differs from expanded source scope: "
                f"missing={missing!r}, unexpected={unexpected!r}"
            )
    if upstream_ref == FL_B_FROZEN_SCOPE_REF and "FL-B" not in seen_waves:
        errors.append(
            "missing pinned frozen wave scope for FL-B at "
            f"{FL_B_FROZEN_SCOPE_REF}"
        )


def validate_file_rows(
    *,
    rows: list[dict[str, Any]],
    assignments: dict[str, str],
    source_set_waves: dict[str, str],
    manifest_files: dict[str, dict[str, Any]],
    repo_root: pathlib.Path,
    porting_rules: str,
    decision_ids: set[str],
    decision_ceilings: dict[str, str],
    pending_verification_paths: set[str],
    require_closed: bool,
    errors: list[str],
) -> tuple[dict[str, dict[str, Any]], collections.Counter[str]]:
    paths = [str(row.get("upstream", "")) for row in rows]
    duplicates = duplicate_values(paths)
    if duplicates:
        errors.append(f"duplicate frame-loop file rows: {', '.join(duplicates)}")
    by_path = {str(row.get("upstream", "")): row for row in rows}
    missing = sorted(set(assignments) - set(by_path))
    outside = sorted(set(by_path) - set(assignments))
    if missing:
        errors.append(
            "expanded frame-loop files missing classification rows: "
            + ", ".join(missing[:12])
        )
    if outside:
        errors.append(
            "file classification rows outside expanded frame-loop scope: "
            + ", ".join(outside[:12])
        )
    unknown_pending_verification_paths = sorted(
        pending_verification_paths - set(by_path)
    )
    if unknown_pending_verification_paths:
        errors.append(
            "candidate pending-verification paths outside the file ledger: "
            + ", ".join(unknown_pending_verification_paths[:12])
        )

    status_counts: collections.Counter[str] = collections.Counter()
    for path in sorted(set(assignments) & set(by_path)):
        row = by_path[path]
        source_set = str(row.get("source_set", ""))
        wave = str(row.get("wave", ""))
        if source_set != assignments[path]:
            errors.append(
                f"file {path} names source_set {source_set!r}, "
                f"expected {assignments[path]!r}"
            )
        expected_wave = (
            "FL-E8" if path in FL_E8_FILES else source_set_waves.get(source_set)
        )
        if wave != expected_wave:
            errors.append(
                f"file {path} names wave {wave!r}, expected {expected_wave!r}"
            )
        dynamically_reached = row.get("dynamically_reached")
        if not isinstance(dynamically_reached, bool):
            errors.append(f"file {path} has no boolean dynamically_reached value")

        rust_modules = [str(value) for value in row.get("rust_modules", [])]
        if not rust_modules:
            errors.append(f"file {path} has no target Rust modules")
        for rust_module in rust_modules:
            if not (repo_root / rust_module).is_file():
                errors.append(f"file {path} Rust module does not exist: {rust_module}")

        manifest = manifest_files.get(path, {})
        mapped = set(manifest_rust_modules(manifest))
        correspondence_scope = str(
            row.get("correspondence_scope", "whole-file")
        )
        if correspondence_scope not in {"whole-file", "verified-fragment"}:
            errors.append(
                f"file {path} has invalid correspondence_scope "
                f"{correspondence_scope!r}"
            )
        verified_fragment = correspondence_scope == "verified-fragment"
        if verified_fragment and not set(rust_modules) < mapped:
            errors.append(
                f"file {path} verified fragment must map to a strict subset of "
                "the whole-file correspondence modules"
            )
        elif not verified_fragment and mapped and mapped != set(rust_modules):
            errors.append(
                f"file {path} maps to {sorted(rust_modules)}, "
                f"but file correspondence maps it to {sorted(mapped)}"
            )

        status = check_status(
            subject=f"file {path}",
            row=row,
            porting_rules=porting_rules,
            decision_ids=decision_ids,
            decision_ceilings=decision_ceilings,
            require_closed=require_closed,
            errors=errors,
        )
        status_counts[status] += 1

        verification = str(manifest.get("verification", ""))
        manifest_status = str(manifest.get("status", ""))
        if verified_fragment:
            if status != "faithful":
                errors.append(
                    f"file {path} verified fragment must be faithful, got "
                    f"{status!r}"
                )
            if (
                manifest_status != "pending"
                or verification != "pending-verification"
            ):
                errors.append(
                    f"file {path} verified fragment requires pending whole-file "
                    "correspondence"
                )
        elif status in CLOSED_STATUSES:
            verification_is_accepted = verification == "orchestrator-verified" or (
                path in pending_verification_paths
                and verification == "pending-verification"
            )
            if not verification_is_accepted:
                errors.append(
                    f"file {path} is {status} before file correspondence is "
                    "orchestrator-verified"
                )
            expected_manifest_status = (
                "divergent-by-decision"
                if status == "divergent-by-decision"
                else "faithful"
            )
            if manifest_status != expected_manifest_status:
                errors.append(
                    f"file {path} is {status}, but file correspondence is "
                    f"{manifest_status!r}"
                )
        if path in pending_verification_paths and (
            status not in CLOSED_STATUSES or verification != "pending-verification"
        ):
            errors.append(
                f"candidate path {path} must be closed with file correspondence "
                "pending-verification"
            )
    return by_path, status_counts


def validate_fl_e8_policy(
    *,
    phase: str,
    waves: list[dict[str, Any]],
    file_rows: list[dict[str, Any]],
    expected_counts: dict[str, Any],
    decisions: list[dict[str, Any]],
    porting_rules: str,
    parity_register: str,
    candidate_paths: set[str],
    errors: list[str],
) -> None:
    """Ratchet the user-approved FL-E8 ledger and sole FLR-20 ceiling."""
    if phase not in {
        "fl-e8-implementation",
        "fl-e8-wp1-candidate",
        "fl-e8-wave-candidate",
    }:
        return

    decision_ids = {str(row.get("id", "")) for row in decisions}
    for decision_id in sorted(FL_E8_FORBIDDEN_DECISIONS & decision_ids):
        errors.append(f"rejected FL-E8 decision {decision_id} must not reappear")
    for decision in decisions:
        if str(decision.get("rule", "")) == "FLR-20" and (
            str(decision.get("id", "")) != "D3"
            or str(decision.get("ceiling", "")) != "layout-engine"
        ):
            errors.append("FLR-20 may bind only D3/layout-engine")

    for number in (13, 14, 15):
        if re.search(rf"(?m)^\s*{number}\.\s", parity_register):
            errors.append(f"parity gap register must not contain D{number}")

    flr20 = porting_rules.partition("- **FLR-20 ")[2].partition("\n- **")[0]
    if not flr20:
        errors.append("PORTING.md is missing FLR-20")
    else:
        if "**layout-engine**" not in flr20 or "D3" not in flr20:
            errors.append("FLR-20 must name D3/layout-engine")
        if "user-approved D-row" not in flr20:
            errors.append("FLR-20 must require an explicit user-approved D-row")
        for ceiling in FL_E8_FORBIDDEN_CEILINGS:
            if ceiling in flr20:
                errors.append(f"FLR-20 must not name rejected ceiling {ceiling!r}")

    fl_e8_waves = [row for row in waves if str(row.get("id", "")) == "FL-E8"]
    if len(fl_e8_waves) != 1:
        errors.append("FL-E8 must have exactly one wave row")
    else:
        wave = fl_e8_waves[0]
        if wave.get("sequence") != 6:
            errors.append("FL-E8 wave sequence must be 6")
        if wave.get("depends_on") != ["FL-E"]:
            errors.append("FL-E8 must depend exactly on FL-E")

    wave_sequence = {
        str(wave.get("id", "")): int(wave.get("sequence", 0))
        for wave in waves
        if isinstance(wave.get("sequence"), int)
    }
    post_fl_e8_faithful = sum(
        1
        for row in file_rows
        if wave_sequence.get(str(row.get("wave", "")), 0) > 6
        and row.get("status") == "faithful"
    )
    post_fl_e8_replacements = sum(
        1
        for old_path, new_path in {
            ("src/nested_artboard_origin.cpp", "src/component_origin.cpp"),
        }
        if not any(row.get("upstream") == old_path for row in file_rows)
        and any(
            row.get("upstream") == new_path
            and wave_sequence.get(str(row.get("wave", "")), 0) > 6
            for row in file_rows
        )
    )

    def phase_faithful_count(baseline: int) -> int:
        # FL-E8's frozen phase total remains the baseline. Later waves are
        # additive rather than permission to replace that historical ratchet
        # with the current manifest's self-declared expected total.
        return baseline + post_fl_e8_faithful - post_fl_e8_replacements

    rows = {
        str(row.get("upstream", "")): row
        for row in file_rows
        if str(row.get("wave", "")) == "FL-E8"
    }
    actual_paths = set(rows)
    if actual_paths != FL_E8_FILES:
        errors.append(
            "FL-E8 wave must contain exactly seven directive-owned rows: "
            f"missing={sorted(FL_E8_FILES - actual_paths)!r}, "
            f"unexpected={sorted(actual_paths - FL_E8_FILES)!r}"
        )

    if phase == "fl-e8-implementation":
        wrong = sorted(
            path for path, row in rows.items() if row.get("status") != "pending"
        )
        if wrong:
            errors.append(f"FL-E8 WP0 rows must all be pending: {wrong!r}")
        required_counts = {
            "faithful": phase_faithful_count(334),
            "divergent-by-decision": 1,
            "pending": 7,
        }
        if candidate_paths:
            errors.append("FL-E8 WP0 must not have a candidate allowlist")
    elif phase == "fl-e8-wp1-candidate":
        wrong_faithful = sorted(
            path
            for path in FL_E8_WP1_FILES
            if rows.get(path, {}).get("status") != "faithful"
        )
        wrong_pending = sorted(
            path
            for path in FL_E8_FILES - FL_E8_WP1_FILES
            if rows.get(path, {}).get("status") != "pending"
        )
        if wrong_faithful or wrong_pending:
            errors.append(
                "FL-E8 WP1 statuses are incoherent: "
                f"faithful={wrong_faithful!r}, pending={wrong_pending!r}"
            )
        required_counts = {
            "faithful": phase_faithful_count(339),
            "divergent-by-decision": 1,
            "pending": 2,
        }
        if candidate_paths != FL_E8_WP1_FILES:
            errors.append("FL-E8 WP1 candidate allowlist must be exactly the five promoted rows")
    else:
        wrong_faithful = sorted(
            path
            for path in FL_E8_WAVE_FILES
            if rows.get(path, {}).get("status") != "faithful"
        )
        wrong_pending = sorted(
            path
            for path in FL_E8_FILES - FL_E8_WAVE_FILES
            if rows.get(path, {}).get("status") != "pending"
        )
        if wrong_faithful or wrong_pending:
            errors.append(
                "FL-E8 wave statuses are incoherent: "
                f"faithful={wrong_faithful!r}, pending={wrong_pending!r}"
            )
        required_counts = {
            "faithful": phase_faithful_count(341),
            "divergent-by-decision": 1,
            "pending": 0,
        }
        if candidate_paths != FL_E8_WAVE_FILES:
            errors.append(
                "FL-E8 wave candidate allowlist must be exactly the seven promoted rows"
            )

    for status, expected in required_counts.items():
        if expected_counts.get(status) != expected:
            errors.append(
                f"FL-E8 {phase} requires {status}={expected}, "
                f"got {expected_counts.get(status)!r}"
            )


def validate_fl_e8_wp1_artifacts(repo_root: pathlib.Path, errors: list[str]) -> None:
    """R-ST-OWNER: pin WP1 source boundaries, fixtures, corpus, and probes."""
    owner_predicates = {
        "crates/nuxie-runtime/src/text/text_modifier.rs": "static_text_modifier_is_unsupported",
        "crates/nuxie-runtime/src/text/text_style_feature.rs": "static_text_style_feature_is_unsupported",
        "crates/nuxie-runtime/src/text/text_target_modifier.rs": "static_text_target_modifier_is_unsupported",
        "crates/nuxie-runtime/src/text/text_variation_modifier.rs": "static_text_variation_modifier_is_unsupported",
    }
    for relative, predicate in owner_predicates.items():
        path = repo_root / relative
        source = path.read_text(encoding="utf-8") if path.is_file() else ""
        if predicate in source:
            errors.append(f"FL-E8 WP1 old static rejection predicate remains: {predicate}")
        if not source:
            errors.append(f"FL-E8 WP1 direct owner is missing: {relative}")

    for relative in (
        "fixtures/fl-e8/text_style_feature.riv",
        "fixtures/fl-e8/text_variation_modifier.riv",
    ):
        path = repo_root / relative
        if not path.is_file() or path.stat().st_size == 0:
            errors.append(f"FL-E8 WP1 fixture is missing or empty: {relative}")

    corpus_path = repo_root / "corpus.toml"
    try:
        corpus = tomllib.loads(corpus_path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError):
        corpus = {}
    rows = list(corpus.get("file", []))
    expected_corpus = {
        "text_style_feature_fl_e8": "fixtures/fl-e8/text_style_feature.riv",
        "text_variation_modifier_fl_e8": "fixtures/fl-e8/text_variation_modifier.riv",
    }
    for corpus_id, expected_path in expected_corpus.items():
        matches = [row for row in rows if row.get("id") == corpus_id]
        if len(matches) != 1 or matches[0].get("path") != expected_path or matches[0].get("status") != "exact":
            errors.append(
                f"FL-E8 WP1 corpus entry {corpus_id} must occur once as exact path {expected_path}"
            )

    probe_path = repo_root / "crates/nuxie-runtime/tests/cpp_probe.rs"
    probe = probe_path.read_text(encoding="utf-8") if probe_path.is_file() else ""
    for artifact in ("D-ST-STRUCT", "D-ST-FONT", "D-ST-FEATURE", "D-ST-VARIATION", "D-ST-TARGET"):
        if artifact not in probe:
            errors.append(f"FL-E8 WP1 live differential marker is missing: {artifact}")

    cpp_probe_path = repo_root / "tools/cpp-probe/main.cpp"
    cpp_probe = cpp_probe_path.read_text(encoding="utf-8") if cpp_probe_path.is_file() else ""
    for mode in (
        "--runtime-fl-e8-static-text",
        "--runtime-fl-e8-font-swap",
        "--runtime-fl-e8-feature-cycle",
        "--runtime-fl-e8-variation-cycle",
    ):
        if mode not in cpp_probe:
            errors.append(f"FL-E8 WP1 C++ probe mode is missing: {mode}")

    codegen_path = repo_root / "tools/nuxie-codegen/src/main.rs"
    codegen = codegen_path.read_text(encoding="utf-8") if codegen_path.is_file() else ""
    for fixture_name in ("text-style-feature", "text-variation-modifier"):
        if fixture_name not in codegen:
            errors.append(f"FL-E8 WP1 fixture emitter is missing: {fixture_name}")
    fixture_test = repo_root / "tools/nuxie-codegen/tests/fl_e8_fixtures.rs"
    if not fixture_test.is_file():
        errors.append("FL-E8 WP1 deterministic fixture-emission integration test is missing")


def validate_fl_e8_wp2_artifacts(repo_root: pathlib.Path, errors: list[str]) -> None:
    """R-LP-OWNER: pin ListPath ownership, differentials, and exact corpora."""
    owner_path = repo_root / "crates/nuxie-runtime/src/shapes/list_path.rs"
    owner = owner_path.read_text(encoding="utf-8") if owner_path.is_file() else ""
    for token in (
        "RuntimeListPathState",
        "RuntimeListPathVertexListener",
        "RuntimeListPathSubscription",
        "RuntimeCubicDetachedVertex",
        "clear_invalid",
    ):
        if token not in owner:
            errors.append(f"FL-E8 WP2 ListPath owner token is missing: {token}")

    bind_path = repo_root / "crates/nuxie-runtime/src/data_bind/data_bind_context.rs"
    bind_source = bind_path.read_text(encoding="utf-8") if bind_path.is_file() else ""
    if "RuntimeArtboardListTarget::ListPath" not in bind_source:
        errors.append("FL-E8 WP2 ListPath list-consumer dispatch is missing")
    if 'target.type_name != "ArtboardComponentList"' in bind_source:
        errors.append("FL-E8 WP2 old explicit ListPath rejection branch remains")

    rust_probe_path = repo_root / "crates/nuxie-runtime/tests/cpp_probe.rs"
    silver_probe_path = repo_root / "tools/silver-corpus/tests/fl_e8_list_path.rs"
    rust_probes = "\n".join(
        path.read_text(encoding="utf-8") if path.is_file() else ""
        for path in (rust_probe_path, silver_probe_path)
    )
    for artifact in (
        "D-LP-INIT",
        "D-LP-XY",
        "D-LP-RD",
        "D-LP-DETACHED",
        "D-LP-POINT",
        "D-LP-INVALID",
        "D-LP-PARTIAL",
        "D-LP-LIVE",
        "D-LP-EDGE",
    ):
        if artifact not in rust_probes:
            errors.append(f"FL-E8 WP2 differential marker is missing: {artifact}")

    cpp_probe_path = repo_root / "tools/cpp-probe/main.cpp"
    cpp_probe = cpp_probe_path.read_text(encoding="utf-8") if cpp_probe_path.is_file() else ""
    if "--runtime-fl-e8-list-path" not in cpp_probe:
        errors.append("FL-E8 WP2 C++ probe mode is missing: --runtime-fl-e8-list-path")

    for manifest_name, table_name in (("corpus.toml", "file"), ("silver-corpus.toml", "case")):
        manifest_path = repo_root / manifest_name
        try:
            manifest = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
        except (OSError, tomllib.TOMLDecodeError):
            manifest = {}
        matches = [
            row for row in manifest.get(table_name, []) if row.get("id") == "list_to_path"
        ]
        if len(matches) != 1 or matches[0].get("status") != "exact":
            errors.append(
                f"FL-E8 WP2 {manifest_name} list_to_path must occur once with status=exact"
            )

    generator_path = repo_root / "tools/silver-corpus/generate_manifest.py"
    generator = generator_path.read_text(encoding="utf-8") if generator_path.is_file() else ""
    if "fl_e8_list_path_actions" not in generator or "range(60)" not in generator:
        errors.append("FL-E8 WP2 generated eight-phase/60-frame action stream is missing")


def validate_fl_e8_wp3_artifacts(repo_root: pathlib.Path, errors: list[str]) -> None:
    """R-RT-OWNER: pin the facade, shared engine, integrated bitmap branch, and live probes."""
    sources = {
        relative: (repo_root / relative).read_text(encoding="utf-8")
        if (repo_root / relative).is_file()
        else ""
        for relative in (
            "crates/nuxie/src/lib.rs",
            "crates/nuxie/src/raw_text.rs",
            "crates/nuxie-runtime/src/text/raw_text.rs",
            "crates/nuxie-runtime/src/text/text_engine.rs",
            "crates/nuxie-runtime/src/text.rs",
            "crates/nuxie-runtime/src/draw.rs",
            "crates/nuxie/tests/raw_text_differential.rs",
            "tools/cpp-probe/main.cpp",
        )
    }
    required = {
        "crates/nuxie/src/lib.rs": ("mod raw_text", "RawText"),
        "crates/nuxie/src/raw_text.rs": ("pub struct RawText", "RawTextFont"),
        "crates/nuxie-runtime/src/text/raw_text.rs": (
            "pub struct RawText",
            "runtime_classify_color_glyph",
        ),
        "crates/nuxie-runtime/src/text/text_engine.rs": (
            "runtime_classify_color_glyph",
            "runtime_extract_color_glyph_layers",
        ),
        "crates/nuxie-runtime/src/text.rs": ("RuntimeIntegratedColorGlyphCommand",),
        "crates/nuxie-runtime/src/draw.rs": ("emoji_images", "draw_image"),
        "crates/nuxie/tests/raw_text_differential.rs": (
            "D-RT-API",
            "D-RT-COLOR-188",
            "D-RT-COLOR-474",
        ),
        "tools/cpp-probe/main.cpp": ("--raw-text-probe",),
    }
    for relative, markers in required.items():
        for marker in markers:
            if marker not in sources[relative]:
                errors.append(f"FL-E8 WP3 artifact {relative} is missing {marker}")
    combined = "\n".join(sources.values())
    if "raw_text_is_unsupported" in combined or "standalone_raw_text_is_unsupported" in combined:
        errors.append("FL-E8 WP3 standalone RawText rejection predicate remains")


def check(
    *,
    repo_root: pathlib.Path,
    rive_runtime_dir: pathlib.Path,
    ledger_path: pathlib.Path,
    gaps_path: pathlib.Path,
    file_manifest_path: pathlib.Path,
    require_closed: bool,
) -> str:
    ledger = read_toml(ledger_path)
    gaps = read_toml(gaps_path)
    file_manifest = read_toml(file_manifest_path)
    errors: list[str] = []

    if ledger.get("version") != 1:
        errors.append("ownership ledger version must be 1")
    if gaps.get("version") != 1:
        errors.append("gap inventory version must be 1")
    upstream_ref = str(ledger.get("upstream_ref", ""))
    if not re.fullmatch(r"[0-9a-f]{40}", upstream_ref):
        errors.append("ownership ledger upstream_ref must be a full 40-hex SHA")
    else:
        actual = git_head(rive_runtime_dir)
        if actual != upstream_ref:
            errors.append(
                f"upstream checkout is {actual}; frame-loop ledger pins {upstream_ref}"
            )
    if gaps.get("upstream_ref") != upstream_ref:
        errors.append("gap inventory and ownership ledger pin different upstream refs")
    if file_manifest.get("upstream_ref") != upstream_ref:
        errors.append("file correspondence and frame-loop ledger pin different refs")

    porting_path = repo_root / str(
        ledger.get("porting_rules_file", "docs/PORTING.md")
    )
    try:
        porting_rules = porting_path.read_text(encoding="utf-8")
    except OSError as error:
        raise CheckFailure(f"cannot read porting rules {porting_path}: {error}") from error

    active_family = ledger.get("active_owner_family", {})
    if not isinstance(active_family, dict):
        errors.append("active_owner_family must be a TOML table")
    else:
        family_id = str(active_family.get("id", ""))
        checklist_relative = str(active_family.get("checklist", ""))
        checklist_path = repo_root / checklist_relative
        if not family_id:
            errors.append("active_owner_family has no id")
        if not checklist_relative:
            errors.append(f"active owner family {family_id!r} has no checklist")
            checklist = ""
        else:
            try:
                checklist = checklist_path.read_text(encoding="utf-8")
            except OSError as error:
                errors.append(
                    f"cannot read active owner-family checklist "
                    f"{checklist_relative}: {error}"
                )
                checklist = ""
        family_cpp_files = [
            str(value) for value in active_family.get("cpp_files", [])
        ]
        if not family_cpp_files:
            errors.append(f"active owner family {family_id!r} has no C++ files")
        for cpp_file in family_cpp_files:
            if not (rive_runtime_dir / cpp_file).is_file():
                errors.append(
                    f"active owner family {family_id!r} cites missing C++ "
                    f"file {cpp_file}"
                )
            if checklist and f"`{cpp_file}`" not in checklist:
                errors.append(
                    f"active owner-family checklist omits C++ file {cpp_file}"
                )
        adversarial = [
            str(value) for value in active_family.get("required_adversarial", [])
        ]
        checklist_state = str(active_family.get("checklist_state", "candidate"))
        if checklist_state not in {"planning", "candidate"}:
            errors.append(
                f"active owner family {family_id!r} has invalid checklist_state "
                f"{checklist_state!r}"
            )
        if not adversarial:
            errors.append(
                f"active owner family {family_id!r} has no adversarial checklist"
            )
        for item in adversarial:
            completed = f"- [x] {item}:" in checklist
            planned = f"- [ ] {item}:" in checklist
            if checklist_state == "candidate" and not completed:
                errors.append(
                    f"active owner-family checklist omits completed "
                    f"adversarial row {item!r}"
                )
            elif checklist_state == "planning" and not (planned or completed):
                errors.append(
                    f"active owner-family checklist omits required "
                    f"adversarial row {item!r}"
                )

    decisions = list(gaps.get("decision", []))
    decision_ids = {str(row.get("id", "")) for row in decisions}
    duplicates = duplicate_values(str(row.get("id", "")) for row in decisions)
    if duplicates:
        errors.append(f"duplicate decision ids: {', '.join(duplicates)}")
    decision_ceilings: dict[str, str] = {}
    for row in decisions:
        decision_id = str(row.get("id", ""))
        decision_rule = str(row.get("rule", ""))
        decision_ceiling = str(row.get("ceiling", ""))
        if bool(decision_rule) != bool(decision_ceiling):
            errors.append(
                f"decision {decision_id} must cite both a rule and named ceiling"
            )
        if decision_rule:
            if not re.fullmatch(r"(?:AF|RF|FLR)-\d+", decision_rule):
                errors.append(
                    f"decision {decision_id} has invalid rule {decision_rule!r}"
                )
            elif f"**{decision_rule} " not in porting_rules:
                errors.append(
                    f"decision {decision_id} cites missing PORTING.md rule "
                    f"{decision_rule}"
                )
        if decision_ceiling:
            decision_ceilings[decision_id] = decision_ceiling

    waves = list(ledger.get("wave", []))
    wave_order = topological_order(waves, errors)
    wave_ids = {str(row.get("id", "")) for row in waves}

    manifest_rows = list(file_manifest.get("file", []))
    manifest_files = {str(row.get("upstream", "")): row for row in manifest_rows}
    if len(manifest_files) != len(manifest_rows):
        errors.append("file correspondence contains duplicate upstream paths")
    scatter_count, scatter_maximum = validate_scatter_ratchet(
        manifest=file_manifest,
        rows=manifest_rows,
        errors=errors,
    )
    assignments, source_set_waves = expand_source_sets(
        source_sets=list(ledger.get("source_set", [])),
        manifest_files=manifest_files,
        repo_root=repo_root,
        rive_runtime_dir=rive_runtime_dir,
        wave_ids=wave_ids,
        errors=errors,
    )
    validate_frozen_wave_scopes(
        rows=list(ledger.get("frozen_wave_scope", [])),
        assignments=assignments,
        source_set_waves=source_set_waves,
        wave_ids=wave_ids,
        upstream_ref=upstream_ref,
        errors=errors,
    )
    phase = str(ledger.get("phase", ""))
    candidate_paths_value = ledger.get("candidate_pending_verification_files", [])
    if not isinstance(candidate_paths_value, list):
        errors.append("candidate_pending_verification_files must be an array")
        candidate_paths: list[str] = []
    else:
        candidate_paths = [str(value) for value in candidate_paths_value]
    candidate_path_duplicates = duplicate_values(candidate_paths)
    if candidate_path_duplicates:
        errors.append(
            "duplicate candidate pending-verification paths: "
            + ", ".join(candidate_path_duplicates)
        )
    if phase in {
        "fl-e-wave-acceptance-candidate",
        "fl-e8-wp1-candidate",
        "fl-e8-wave-candidate",
    }:
        if not candidate_paths:
            errors.append(
                f"{phase} requires an explicit "
                "candidate_pending_verification_files allowlist"
            )
        pending_verification_paths = set(candidate_paths)
        if phase in {"fl-e8-wp1-candidate", "fl-e8-wave-candidate"}:
            pending_verification_paths.update(FL_E_W120_PENDING_VERIFICATION_FILES)
    elif phase == "fl-e8-implementation":
        if candidate_paths:
            errors.append(
                "candidate_pending_verification_files is not used during FL-E8 WP0"
            )
        pending_verification_paths = set(FL_E_W120_PENDING_VERIFICATION_FILES)
    else:
        if candidate_paths:
            errors.append(
                "candidate_pending_verification_files is only valid in phase "
                "fl-e-wave-acceptance-candidate, fl-e8-wp1-candidate, or "
                "fl-e8-wave-candidate"
            )
        pending_verification_paths = set()

    file_rows, file_status_counts = validate_file_rows(
        rows=list(ledger.get("file", [])),
        assignments=assignments,
        source_set_waves=source_set_waves,
        manifest_files=manifest_files,
        repo_root=repo_root,
        porting_rules=porting_rules,
        decision_ids=decision_ids,
        decision_ceilings=decision_ceilings,
        pending_verification_paths=pending_verification_paths,
        require_closed=require_closed,
        errors=errors,
    )

    parity_register_path = repo_root / "docs/parity-gap-register.md"
    parity_register = (
        parity_register_path.read_text(encoding="utf-8")
        if parity_register_path.is_file()
        else ""
    )
    validate_fl_e8_policy(
        phase=phase,
        waves=waves,
        file_rows=list(ledger.get("file", [])),
        expected_counts=dict(ledger.get("expected_file_status_counts", {})),
        decisions=decisions,
        porting_rules=porting_rules,
        parity_register=parity_register,
        candidate_paths=set(candidate_paths),
        errors=errors,
    )
    if phase in {"fl-e8-wp1-candidate", "fl-e8-wave-candidate"}:
        validate_fl_e8_wp1_artifacts(repo_root, errors)
    if phase == "fl-e8-wave-candidate":
        validate_fl_e8_wp2_artifacts(repo_root, errors)
        validate_fl_e8_wp3_artifacts(repo_root, errors)

    trace_path = repo_root / str(
        ledger.get("trace_evidence_file", "docs/runtime-frame-loop-trace.json")
    )
    try:
        trace = json.loads(trace_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise CheckFailure(f"cannot read trace evidence {trace_path}: {error}") from error
    if trace.get("schema") != "nuxie-runtime-frame-loop-trace/v2":
        errors.append("trace evidence schema is not v2")
    if trace.get("upstream_ref") != upstream_ref:
        errors.append("trace evidence pins a different upstream ref")
    validate_trace_artifacts(trace, ledger, errors)
    trace_scope = trace.get("scope", {})
    if trace_scope.get("static_cpp_files") != len(assignments):
        errors.append(
            "trace evidence static_cpp_files does not match expanded source scope"
        )
    expected_corpus = {
        "advance_blend_mode",
        "ai_assitant",
        "align_target",
        "animated_clipping",
        "animation_reset_cases",
        "spotify_kids_demo",
    }
    if set(trace.get("corpus", [])) != expected_corpus:
        errors.append("trace evidence does not cover the canonical six-entry corpus")
    fixture_rows = list(ledger.get("trace_mechanism_fixture", []))
    expected_mechanism_corpus = {
        str(row.get("id", "")) for row in fixture_rows
    }
    expected_steady_corpus = {
        str(row.get("id", ""))
        for row in fixture_rows
        if row.get("steady", True)
    }
    if set(trace.get("mechanism_corpus", [])) != expected_mechanism_corpus:
        errors.append("trace evidence does not cover the mechanism corpus")
    if set(trace.get("steady_corpus", [])) != expected_steady_corpus:
        errors.append("trace evidence does not cover the steady mechanism corpus")
    trace_fixture_hashes = trace.get("mechanism_fixture_sha256", {})
    trace_input_hashes = trace.get("mechanism_input_sha256", {})
    expected_input_ids = {
        str(row.get("id", ""))
        for row in fixture_rows
        if str(row.get("input_script", ""))
    }
    if set(trace_input_hashes) != expected_input_ids:
        errors.append(
            "trace evidence mechanism input hashes do not match the "
            "interactive fixture set"
        )
    for row in fixture_rows:
        fixture_id = str(row.get("id", ""))
        relative_path = str(row.get("path", ""))
        expected_hash = str(row.get("sha256", ""))
        fixture_path = rive_runtime_dir / relative_path
        if not fixture_id or not relative_path or len(expected_hash) != 64:
            errors.append(f"trace mechanism fixture {fixture_id!r} is incomplete")
            continue
        try:
            actual_hash = hashlib.sha256(fixture_path.read_bytes()).hexdigest()
        except OSError as error:
            errors.append(
                f"cannot read trace mechanism fixture {fixture_path}: {error}"
            )
            continue
        if actual_hash != expected_hash:
            errors.append(
                f"trace mechanism fixture {fixture_id} hash is {actual_hash}, "
                f"expected {expected_hash}"
            )
        if trace_fixture_hashes.get(fixture_id) != expected_hash:
            errors.append(
                f"trace evidence fixture hash for {fixture_id} is stale"
            )
        relative_input = str(row.get("input_script", ""))
        expected_input_hash = str(row.get("input_sha256", ""))
        if relative_input:
            input_path = repo_root / relative_input
            if len(expected_input_hash) != 64:
                errors.append(
                    f"trace mechanism input {fixture_id!r} is incomplete"
                )
                continue
            try:
                actual_input_hash = hashlib.sha256(
                    input_path.read_bytes()
                ).hexdigest()
            except OSError as error:
                errors.append(
                    f"cannot read trace mechanism input {input_path}: {error}"
                )
                continue
            if actual_input_hash != expected_input_hash:
                errors.append(
                    f"trace mechanism input {fixture_id} hash is "
                    f"{actual_input_hash}, expected {expected_input_hash}"
                )
            if trace_input_hashes.get(fixture_id) != expected_input_hash:
                errors.append(
                    f"trace evidence input hash for {fixture_id} is stale"
                )
        elif expected_input_hash:
            errors.append(
                f"trace mechanism fixture {fixture_id} has an input hash "
                "without an input script"
            )
    operations = trace.get("golden_stream_operations", {})
    if operations.get("cpp") != operations.get("rust"):
        errors.append("trace evidence golden-stream work counts differ")
    mechanism_operations = trace.get("mechanism_golden_stream_operations", {})
    if mechanism_operations.get("cpp") != mechanism_operations.get("rust"):
        errors.append("trace evidence mechanism golden-stream work counts differ")
    expected_landmarks = ledger.get("expected_trace_landmarks", {})
    trace_sections = {
        "frame": "landmarks",
        "construction": "construction_landmarks",
        "mechanism_frame": "mechanism_landmarks",
        "mechanism_construction": "mechanism_construction_landmarks",
        "steady": "steady_landmarks",
    }
    for expected_name, trace_name in trace_sections.items():
        expected_names = expected_landmarks.get(expected_name, [])
        if not isinstance(expected_names, list):
            errors.append(f"expected_trace_landmarks.{expected_name} is missing")
            continue
        actual_names = set(trace.get(trace_name, {}))
        if actual_names != set(str(value) for value in expected_names):
            missing = sorted(set(expected_names) - actual_names)
            stale = sorted(actual_names - set(expected_names))
            errors.append(
                f"trace {trace_name} set differs; missing={missing}, stale={stale}"
            )
    steady_landmarks = trace.get("steady_landmarks", {})
    required_steady_zero = {
        "component_dirt_consumptions",
        "constraint_applications",
        "follow_path_measure_rebuilds",
        "skin_buffer_rebuilds",
        "draw_order_sort",
        "clipping_redundancy_clear",
        "layout_compute",
        "internal_owner_rediscovery",
        "drawable_owner_lookup",
    }
    for name in sorted(
        required_steady_zero
        & set(expected_landmarks.get("steady", []))
    ):
        counts = steady_landmarks.get(name, {})
        for side in ("cpp", "rust"):
            if counts.get(side) != 0:
                errors.append(
                    f"steady trace {name}.{side} must be zero, "
                    f"got {counts.get(side)!r}"
                )
    for side in ("cpp", "rust"):
        if not trace.get("functions", {}).get(side):
            errors.append(f"trace evidence has no reached {side} functions")
    reached_cpp_files = set(trace.get("functions", {}).get("cpp", {}))
    for path, row in file_rows.items():
        if path not in assignments:
            continue
        recorded = row.get("dynamically_reached")
        actual = path in reached_cpp_files
        if recorded != actual:
            errors.append(
                f"file {path} dynamically_reached={recorded!r}, "
                f"but trace evidence says {actual}"
            )

    expected_files = ledger.get("expected_file_status_counts", {})
    for status in sorted(STATUSES):
        expected = expected_files.get(status)
        if not isinstance(expected, int):
            errors.append(f"expected_file_status_counts.{status} is missing")
        elif file_status_counts[status] != expected:
            errors.append(
                f"file status count {status}={file_status_counts[status]}, "
                f"expected {expected}"
            )

    imported_member_count = 0
    member_status_counts: collections.Counter[str] = collections.Counter()
    for row in ledger.get("import_ledger", []):
        import_id = str(row.get("id", ""))
        import_path = repo_root / str(row.get("path", ""))
        imported = read_toml(import_path)
        if imported.get("upstream_ref") != upstream_ref:
            errors.append(f"import_ledger {import_id} pins a different upstream ref")
        if imported.get("phase") != "closed":
            errors.append(f"import_ledger {import_id} is not closed")
        owners = list(imported.get("owner", []))
        expected_count = row.get("expected_owner_count")
        if not isinstance(expected_count, int) or len(owners) != expected_count:
            errors.append(
                f"import_ledger {import_id} has {len(owners)} owners, "
                f"expected {expected_count}"
            )
        for owner in owners:
            imported_status = str(owner.get("status", ""))
            if imported_status == "exact":
                member_status_counts["faithful"] += 1
            elif imported_status == "adapted":
                member_status_counts["adapted"] += 1
            else:
                errors.append(
                    f"import_ledger {import_id} owner {owner.get('id')} "
                    f"is not closed: {imported_status}"
                )
        imported_member_count += len(owners)

    members = list(ledger.get("member", []))
    member_ids = [str(row.get("id", "")) for row in members]
    duplicates = duplicate_values(member_ids)
    if duplicates:
        errors.append(f"duplicate member ids: {', '.join(duplicates)}")
    for row in members:
        member_id = str(row.get("id", ""))
        if not member_id:
            errors.append("member has an empty id")
            continue
        wave = str(row.get("wave", ""))
        if wave not in wave_ids:
            errors.append(f"member {member_id} has unknown wave {wave!r}")
        cpp_files = [str(value) for value in row.get("cpp_files", [])]
        if not cpp_files:
            errors.append(f"member {member_id} has no cpp_files")
        for cpp_file in cpp_files:
            if cpp_file not in assignments:
                errors.append(
                    f"member {member_id} cites C++ file outside frame-loop scope: "
                    f"{cpp_file}"
                )
        rust_file = repo_root / str(row.get("rust_file", ""))
        anchor = str(row.get("rust_anchor", ""))
        if not rust_file.is_file():
            errors.append(f"member {member_id} Rust file does not exist: {rust_file}")
        elif not anchor:
            errors.append(f"member {member_id} has an empty rust_anchor")
        elif anchor not in rust_file.read_text(encoding="utf-8", errors="replace"):
            errors.append(
                f"member {member_id} anchor {anchor!r} is absent from "
                f"{rust_file.relative_to(repo_root)}"
            )
        status = check_status(
            subject=f"member {member_id}",
            row=row,
            porting_rules=porting_rules,
            decision_ids=decision_ids,
            decision_ceilings=decision_ceilings,
            require_closed=require_closed,
            errors=errors,
        )
        member_status_counts[status] += 1
        lifecycle = row.get("lifecycle", {})
        if not isinstance(lifecycle, dict):
            lifecycle = {}
        if status in CLOSED_STATUSES:
            for phase in LIFECYCLE_PHASES:
                citations = lifecycle.get(phase, [])
                if not isinstance(citations, list) or not citations:
                    errors.append(f"member {member_id} lifecycle {phase} is empty")
                    continue
                for citation in citations:
                    validate_citation(
                        str(citation), repo_root, rive_runtime_dir, errors
                    )

    expected_members = ledger.get("expected_member_status_counts", {})
    for status in sorted(STATUSES):
        expected = expected_members.get(status)
        if not isinstance(expected, int):
            errors.append(f"expected_member_status_counts.{status} is missing")
        elif member_status_counts[status] != expected:
            errors.append(
                f"member status count {status}={member_status_counts[status]}, "
                f"expected {expected}"
            )

    ratchet_results: list[tuple[str, int, int, int]] = []
    gap_rows = list(gaps.get("gap", []))
    gap_ids = [str(row.get("id", "")) for row in gap_rows]
    duplicates = duplicate_values(gap_ids)
    if duplicates:
        errors.append(f"duplicate gap ids: {', '.join(duplicates)}")
    gap_status_counts: collections.Counter[str] = collections.Counter()
    for row in gap_rows:
        gap_id = str(row.get("id", ""))
        status = str(row.get("status", ""))
        if not gap_id:
            errors.append("gap has an empty id")
        if status not in {"open", "closed"}:
            errors.append(f"gap {gap_id} has invalid status {status!r}")
        else:
            gap_status_counts[status] += 1
        if require_closed and status != "closed":
            errors.append(f"closed frame loop required but gap {gap_id} is open")
        citations = row.get("citations", [])
        if not isinstance(citations, list) or not citations:
            errors.append(f"gap {gap_id} has no citations")
        else:
            for citation in citations:
                validate_citation(
                    str(citation), repo_root, rive_runtime_dir, errors
                )
        if not str(row.get("mechanism", "")).strip():
            errors.append(f"gap {gap_id} has no mechanism")
        if not str(row.get("closure", "")).strip():
            errors.append(f"gap {gap_id} has no closure")

    expected_gap_status_counts = gaps.get("expected_gap_status_counts", {})
    for status in ("open", "closed"):
        expected = expected_gap_status_counts.get(status)
        if not isinstance(expected, int):
            errors.append(f"expected_gap_status_counts.{status} is missing")
        elif gap_status_counts[status] != expected:
            errors.append(
                f"gap status count {status}={gap_status_counts[status]}, "
                f"expected {expected}"
            )

    mismatch_counters = {
        name
        for trace_name in trace_sections.values()
        for name, counts in trace.get(trace_name, {}).items()
        if counts.get("cpp") != counts.get("rust")
    }
    gap_counters = {
        str(row.get("counter", ""))
        for row in gap_rows
        if str(row.get("counter", ""))
    }
    gap_counters.update(
        str(counter)
        for row in gap_rows
        for counter in row.get("counters", [])
    )
    untracked_mismatches = sorted(mismatch_counters - gap_counters)
    if untracked_mismatches:
        errors.append(
            "trace landmark mismatches have no gap rows: "
            + ", ".join(untracked_mismatches)
        )

    ratchet_rows = list(gaps.get("ratchet", []))
    ratchet_ids = [str(row.get("id", "")) for row in ratchet_rows]
    duplicates = duplicate_values(ratchet_ids)
    if duplicates:
        errors.append(f"duplicate ratchet ids: {', '.join(duplicates)}")
    owner_boundary_registry: set[tuple[str, str, str, str, str]] = set()
    for row in gaps.get("owner_boundary_allow", []):
        file = str(row.get("file", ""))
        kind = str(row.get("kind", ""))
        anchor = str(row.get("anchor", ""))
        guarded_name = str(row.get("guarded_name", ""))
        site_hash = str(row.get("site_hash", ""))
        key = (file, kind, anchor, guarded_name, site_hash)
        if (
            not file
            or pathlib.PurePosixPath(file).is_absolute()
            or ".." in pathlib.PurePosixPath(file).parts
            or kind not in set(NESTED_EVENT_OWNER_BOUNDARY_RATCHETS.values())
            or re.fullmatch(
                r"[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*",
                anchor,
            )
            is None
            or re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", guarded_name) is None
            or re.fullmatch(r"[0-9a-f]{64}", site_hash) is None
        ):
            errors.append(
                "owner boundary registry row must have a relative file, "
                "known kind, full enclosing-item anchor, guarded name, and "
                "site_hash"
            )
            continue
        if key in owner_boundary_registry:
            errors.append(
                "duplicate owner boundary registry row: "
                f"{file} {kind} {anchor} {guarded_name} {site_hash}"
            )
            continue
        owner_boundary_registry.add(key)
    matched_owner_boundary_registry: set[
        tuple[str, str, str, str, str]
    ] = set()
    semantic_resolver_seam_exists = any(
        re.search(
            r"\btrait\s+SemanticNodeResolver\b",
            path.read_text(encoding="utf-8", errors="replace"),
        )
        is not None
        for path in repo_root.glob("crates/nuxie-runtime/src/state_machine/**/*.rs")
        if path.is_file()
    )
    owner_export_origins: dict[tuple[str, str], str] = {}
    owner_path = repo_root / NESTED_EVENT_OWNER_MODULE
    if owner_path.is_file():
        owner_relative = NESTED_EVENT_OWNER_MODULE.as_posix()
        for exported in nested_event_owner_exports(
            owner_path.read_text(encoding="utf-8", errors="replace")
        ):
            owner_export_origins[exported] = owner_relative
    nested_event_guarded_aliases = set(owner_export_origins)
    for row in ratchet_rows:
        ratchet_id = str(row.get("id", ""))
        pattern_text = str(row.get("pattern", ""))
        detector = str(row.get("detector", ""))
        globs = [str(value) for value in row.get("globs", [])]
        content_begin = str(row.get("content_begin", ""))
        content_end = str(row.get("content_end", ""))
        content_sha256 = str(row.get("content_sha256", ""))
        maximum = row.get("max_occurrences")
        minimum = row.get("min_occurrences", 0)
        if (
            not ratchet_id
            or (
                not pattern_text
                and not (
                    detector == NESTED_EVENT_OWNER_BOUNDARY_DETECTOR
                    and ratchet_id in NESTED_EVENT_OWNER_BOUNDARY_RATCHETS
                )
            )
            or not globs
            or not isinstance(maximum, int)
            or not isinstance(minimum, int)
            or minimum < 0
            or minimum > maximum
        ):
            errors.append(f"ratchet {ratchet_id!r} is incomplete")
            continue
        if content_sha256 and (
            not content_begin
            or not content_end
            or re.fullmatch(r"[0-9a-f]{64}", content_sha256) is None
        ):
            errors.append(
                f"ratchet {ratchet_id} has incomplete content digest metadata"
            )
            continue
        pattern = None
        if pattern_text:
            try:
                pattern = re.compile(pattern_text)
            except re.error as error:
                errors.append(f"ratchet {ratchet_id} has invalid regex: {error}")
                continue
        count = 0
        hits: list[str] = []
        content_regions: list[bytes] = []
        for glob in globs:
            for path in sorted(repo_root.glob(glob)):
                if not path.is_file():
                    continue
                source = path.read_text(encoding="utf-8", errors="replace")
                if ratchet_id == UNBOUND_SCRIPTED_CONSTRUCTOR_RATCHET:
                    found_offsets = unbound_scripted_constructor_hits(source)
                elif ratchet_id == SEMANTIC_ORDINAL_PROJECTION_RATCHET:
                    found_offsets = semantic_ordinal_projection_hits(
                        source,
                        resolver_seam_exists=semantic_resolver_seam_exists,
                    )
                elif detector == NESTED_EVENT_OWNER_BOUNDARY_DETECTOR:
                    relative_path = pathlib.PurePosixPath(path.relative_to(repo_root))
                    if (
                        relative_path.stem == "tests"
                        or relative_path.stem.endswith("_tests")
                        or "tests" in relative_path.parts
                    ):
                        continue
                    if (
                        relative_path == NESTED_EVENT_OWNER_MODULE
                    ):
                        continue
                    detector_kind = NESTED_EVENT_OWNER_BOUNDARY_RATCHETS[
                        ratchet_id
                    ]
                    found_offsets = nested_event_owner_boundary_hits(
                        source,
                        detector_kind,
                        guarded_aliases=nested_event_guarded_aliases,
                    )
                    relative_file = path.relative_to(repo_root).as_posix()
                    registered = {
                        (anchor, guarded_name, site_hash)
                        for file, kind, anchor, guarded_name, site_hash in owner_boundary_registry
                        if file == relative_file and kind == detector_kind
                    }
                    match_records = nested_event_owner_boundary_matches(
                        source,
                        detector_kind,
                        guarded_aliases=nested_event_guarded_aliases,
                    )
                    if registered:
                        unregistered_offsets = []
                        consumed: set[tuple[str, str, str]] = set()
                        for (
                            anchor_offset,
                            site_offset,
                            anchor,
                            guarded_name,
                            site_hash,
                        ) in match_records:
                            match_key = (anchor, guarded_name, site_hash)
                            registry_key = (
                                relative_file,
                                detector_kind,
                                anchor,
                                guarded_name,
                                site_hash,
                            )
                            if (
                                match_key in registered
                                and match_key not in consumed
                            ):
                                consumed.add(match_key)
                                matched_owner_boundary_registry.add(registry_key)
                                continue
                            unregistered_offsets.append(site_offset)
                            errors.append(
                                "unregistered owner boundary hit "
                                f"{relative_file} {detector_kind} {anchor} "
                                f"{guarded_name} {site_hash}"
                                + (
                                    " propagated from owner export "
                                    f"{owner_export_origins[(detector_kind, guarded_name)]}"
                                    f"::{guarded_name}"
                                    if (detector_kind, guarded_name)
                                    in owner_export_origins
                                    else ""
                                )
                            )
                        recorded_anchor_offsets = {
                            anchor_offset
                            for (
                                anchor_offset,
                                _,
                                _,
                                _,
                                _,
                            ) in match_records
                        }
                        unmatched_hit_offsets = (
                            set(found_offsets) - recorded_anchor_offsets
                        )
                        found_offsets = sorted(
                            set(unregistered_offsets) | unmatched_hit_offsets
                        )
                    else:
                        for (
                            _,
                            _,
                            anchor,
                            guarded_name,
                            _,
                        ) in match_records:
                            origin = owner_export_origins.get(
                                (detector_kind, guarded_name)
                            )
                            if origin is not None:
                                errors.append(
                                    "propagated owner boundary hit "
                                    f"{relative_file} {detector_kind} {anchor} "
                                    f"{guarded_name} from owner export "
                                    f"{origin}::{guarded_name}"
                                )
                else:
                    assert pattern is not None
                    found_offsets = [
                        match.start() for match in pattern.finditer(source)
                    ]
                count += len(found_offsets)
                for offset in found_offsets:
                    line_number = source.count("\n", 0, offset) + 1
                    hits.append(f"{path.relative_to(repo_root)}:{line_number}")
                if content_sha256:
                    begin_offsets = [
                        match.start()
                        for match in re.finditer(
                            re.escape(content_begin), source
                        )
                    ]
                    end_offsets = [
                        match.start()
                        for match in re.finditer(re.escape(content_end), source)
                    ]
                    if len(begin_offsets) != 1 or len(end_offsets) != 1:
                        errors.append(
                            f"ratchet {ratchet_id} content delimiters must "
                            f"occur exactly once in {path.relative_to(repo_root)}"
                        )
                        continue
                    begin_offset = begin_offsets[0]
                    end_offset = end_offsets[0] + len(content_end)
                    if end_offset <= begin_offset:
                        errors.append(
                            f"ratchet {ratchet_id} content delimiters are "
                            f"out of order in {path.relative_to(repo_root)}"
                        )
                        continue
                    content_regions.append(
                        source[begin_offset:end_offset].encode("utf-8")
                    )
        ratchet_results.append((ratchet_id, count, minimum, maximum))
        if content_sha256:
            if len(content_regions) != 1:
                errors.append(
                    f"ratchet {ratchet_id} content digest requires exactly "
                    f"one delimited source region, found {len(content_regions)}"
                )
            else:
                actual_content_sha256 = hashlib.sha256(
                    content_regions[0]
                ).hexdigest()
                if actual_content_sha256 != content_sha256:
                    errors.append(
                        f"ratchet {ratchet_id} content digest changed: "
                        f"expected {content_sha256}, got "
                        f"{actual_content_sha256}"
                    )
        if count < minimum:
            errors.append(
                f"ratchet {ratchet_id} decreased to {count} < {minimum}; "
                "required structural proof is missing"
            )
        if count > maximum:
            errors.append(
                f"ratchet {ratchet_id} increased to {count} > {maximum}; "
                f"first hits: {', '.join(hits[:8])}"
            )
        if require_closed and count != 0:
            errors.append(
                f"closed frame loop required but ratchet {ratchet_id} has {count} hits"
            )

    for file, kind, anchor, guarded_name, site_hash in sorted(
        owner_boundary_registry
    ):
        if (
            file,
            kind,
            anchor,
            guarded_name,
            site_hash,
        ) not in matched_owner_boundary_registry:
            errors.append(
                "registered owner boundary anchor is missing "
                f"{file} {kind} {anchor} {guarded_name} {site_hash}"
            )

    if errors:
        raise CheckFailure("\n".join(f"- {error}" for error in errors))

    files = ", ".join(
        f"{status}={file_status_counts[status]}" for status in sorted(STATUSES)
    )
    member_summary = ", ".join(
        f"{status}={member_status_counts[status]}" for status in sorted(STATUSES)
    )
    ratchets = ", ".join(
        f"{ratchet_id}={count}/{minimum}..{maximum}"
        for ratchet_id, count, minimum, maximum in ratchet_results
    )
    return (
        f"runtime-frame-loop-port: files={len(assignments)} ({files}); "
        f"members={len(members) + imported_member_count} ({member_summary}); "
        f"gaps={len(gap_rows)}; waves={' -> '.join(wave_order)}; "
        f"scatter={scatter_count}/{scatter_maximum}; "
        f"ratchets[{ratchets}]"
    )


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser()
    result.add_argument("--repo-root", type=pathlib.Path, required=True)
    result.add_argument("--rive-runtime-dir", type=pathlib.Path, required=True)
    result.add_argument("--ledger", type=pathlib.Path, required=True)
    result.add_argument("--gaps", type=pathlib.Path, required=True)
    result.add_argument("--file-manifest", type=pathlib.Path, required=True)
    result.add_argument("--require-closed", action="store_true")
    return result


def main() -> int:
    args = parser().parse_args()
    try:
        summary = check(
            repo_root=args.repo_root.resolve(),
            rive_runtime_dir=args.rive_runtime_dir.resolve(),
            ledger_path=args.ledger.resolve(),
            gaps_path=args.gaps.resolve(),
            file_manifest_path=args.file_manifest.resolve(),
            require_closed=args.require_closed,
        )
    except CheckFailure as error:
        print(f"runtime-frame-loop-port check failed:\n{error}", file=sys.stderr)
        return 1
    print(summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
