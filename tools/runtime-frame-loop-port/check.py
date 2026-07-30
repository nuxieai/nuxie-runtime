#!/usr/bin/env python3
"""Fail-closed checker for the C++-corresponding runtime frame-loop port."""

from __future__ import annotations

import argparse
import collections
import fnmatch
import hashlib
import json
import pathlib
import re
import subprocess
import sys
import tomllib
from typing import Any, Iterable


TOOL_DIR = pathlib.Path(__file__).resolve().parent
if str(TOOL_DIR) not in sys.path:
    sys.path.insert(0, str(TOOL_DIR))

from source_fingerprint import (
    SourceFingerprintError,
    candidate_source_fingerprint,
    rust_runner_provenance,
)


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


class CheckFailure(Exception):
    """Raised when the frame-loop proof is incomplete or inconsistent."""


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
        raw = re.match(r"(?:br|r)(?P<hashes>#{0,255})\"", source[index:])
        if raw is not None:
            hashes = raw.group("hashes")
            start_length = raw.end()
            terminator = '"' + hashes
            end = source.find(terminator, index + start_length)
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
            char_literal = re.match(r"'(?:\\.|[^'\\\n])+'", source[index:])
            if char_literal is not None:
                end = index + char_literal.end()
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


def validate_trace_rust_ref(
    repo_root: pathlib.Path, rust_ref: object, errors: list[str]
) -> None:
    if not isinstance(rust_ref, str) or re.fullmatch(r"[0-9a-f]{40}", rust_ref) is None:
        errors.append("trace evidence Rust ref is missing or is not a full commit SHA")
        return
    commit = subprocess.run(
        ["git", "-C", str(repo_root), "cat-file", "-e", f"{rust_ref}^{{commit}}"],
        text=True,
        capture_output=True,
        check=False,
    )
    if commit.returncode != 0:
        errors.append(f"trace evidence Rust ref does not exist: {rust_ref}")
        return
    head = git_head(repo_root)
    if rust_ref == head:
        return
    ancestor = subprocess.run(
        ["git", "-C", str(repo_root), "merge-base", "--is-ancestor", rust_ref, head],
        text=True,
        capture_output=True,
        check=False,
    )
    if ancestor.returncode != 0:
        errors.append(
            "trace evidence Rust ref is neither HEAD nor an ancestor production candidate"
        )
        return
    changed = subprocess.run(
        ["git", "-C", str(repo_root), "diff", "--name-only", f"{rust_ref}..{head}"],
        text=True,
        capture_output=True,
        check=False,
    )
    if changed.returncode != 0:
        errors.append(
            "cannot verify publication-only changes after trace evidence Rust ref"
        )
        return
    unauthorized = sorted(
        path
        for path in changed.stdout.splitlines()
        if not path.startswith("docs/runtime-frame-loop-")
    )
    if unauthorized:
        errors.append(
            "trace evidence Rust ref is stale across non-publication changes: "
            + ", ".join(unauthorized)
        )


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
    elif decision:
        errors.append(f"{subject} is {status} but unexpectedly cites {decision}")
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
        expected_wave = source_set_waves.get(source_set)
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
        mapped = {
            value.strip()
            for value in str(manifest.get("rust_module", "")).split(";")
            if value.strip()
        }
        if mapped and mapped != set(rust_modules):
            errors.append(
                f"file {path} maps to {sorted(rust_modules)}, "
                f"but file correspondence maps it to {sorted(mapped)}"
            )

        status = check_status(
            subject=f"file {path}",
            row=row,
            porting_rules=porting_rules,
            decision_ids=decision_ids,
            require_closed=require_closed,
            errors=errors,
        )
        status_counts[status] += 1

        verification = str(manifest.get("verification", ""))
        manifest_status = str(manifest.get("status", ""))
        if status in CLOSED_STATUSES:
            if verification != "orchestrator-verified":
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
    return by_path, status_counts


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

    waves = list(ledger.get("wave", []))
    wave_order = topological_order(waves, errors)
    wave_ids = {str(row.get("id", "")) for row in waves}

    manifest_rows = list(file_manifest.get("file", []))
    manifest_files = {str(row.get("upstream", "")): row for row in manifest_rows}
    if len(manifest_files) != len(manifest_rows):
        errors.append("file correspondence contains duplicate upstream paths")
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
    file_rows, file_status_counts = validate_file_rows(
        rows=list(ledger.get("file", [])),
        assignments=assignments,
        source_set_waves=source_set_waves,
        manifest_files=manifest_files,
        repo_root=repo_root,
        porting_rules=porting_rules,
        decision_ids=decision_ids,
        require_closed=require_closed,
        errors=errors,
    )

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
    validate_trace_rust_ref(repo_root, trace.get("rust_ref"), errors)
    validate_trace_artifacts(trace, ledger, errors)
    recorded_rust_candidate_source = trace.get("rust_candidate_source")
    if not isinstance(recorded_rust_candidate_source, dict):
        errors.append("trace evidence has no Rust candidate source fingerprint")
    else:
        try:
            actual_rust_candidate_source = candidate_source_fingerprint(
                repo_root, evidence_path=trace_path
            )
        except SourceFingerprintError as error:
            errors.append(f"cannot verify Rust candidate source: {error}")
        else:
            if recorded_rust_candidate_source != actual_rust_candidate_source:
                errors.append(
                    "trace evidence Rust candidate source fingerprint is stale; "
                    f"recorded={recorded_rust_candidate_source!r}, "
                    f"actual={actual_rust_candidate_source!r}"
                )
            expected_rust_runner_provenance = rust_runner_provenance(
                actual_rust_candidate_source
            )
            if (
                trace.get("rust_runner_provenance")
                != expected_rust_runner_provenance
            ):
                errors.append(
                    "trace evidence Rust runner provenance is missing or stale"
                )
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
    for row in gap_rows:
        gap_id = str(row.get("id", ""))
        status = str(row.get("status", ""))
        if not gap_id:
            errors.append("gap has an empty id")
        if status not in {"open", "closed"}:
            errors.append(f"gap {gap_id} has invalid status {status!r}")
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
    semantic_resolver_seam_exists = any(
        re.search(
            r"\btrait\s+SemanticNodeResolver\b",
            path.read_text(encoding="utf-8", errors="replace"),
        )
        is not None
        for path in repo_root.glob("crates/nuxie-runtime/src/state_machine/**/*.rs")
        if path.is_file()
    )
    for row in ratchet_rows:
        ratchet_id = str(row.get("id", ""))
        pattern_text = str(row.get("pattern", ""))
        globs = [str(value) for value in row.get("globs", [])]
        content_begin = str(row.get("content_begin", ""))
        content_end = str(row.get("content_end", ""))
        content_sha256 = str(row.get("content_sha256", ""))
        maximum = row.get("max_occurrences")
        minimum = row.get("min_occurrences", 0)
        if (
            not ratchet_id
            or not pattern_text
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
                else:
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
