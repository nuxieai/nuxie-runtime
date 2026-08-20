#!/usr/bin/env python3
"""Fail-closed validator for the native Metal mechanical-port campaign."""

from __future__ import annotations

import argparse
import collections
import csv
import hashlib
import pathlib
import re
import subprocess
import sys
import tomllib
from typing import Any, Iterable


SOURCE_STATUSES = {"pending", "in-progress", "ported", "verified"}
OWNER_STATUSES = {"pending", "in-progress", "ported", "verified"}
VERIFIED_STATUSES = {"ported", "verified"}
TRANSLATION_STATUSES = {
    "pending",
    "ready",
    "in-progress",
    "translated",
    "reviewed",
    "fixed",
    "compiled",
    "verified",
}
TRANSLATION_PHASES = {"trial", "bulk"}
TRANSLATION_WORKER_ROLES = {"luna-extra-high", "sol-high"}
TRANSLATION_REVIEWER_ROLES = {"sol-high"}
TRANSLATION_FIXER_ROLES = {"sol-high"}
LIFETIME_STATUSES = {"review-needed", "prepared", "verified"}
LIFETIME_COLUMNS = (
    "schema_version",
    "upstream_ref",
    "unit",
    "upstream_path",
    "field",
    "cpp_ownership",
    "rust_shape",
    "threading",
    "concrete_native_downcast_seam",
    "release_invariant",
    "failure_invariant",
    "status",
    "evidence",
)
RENDER_CONTEXT_FILE_MAP_COLUMNS = (
    "version",
    "upstream_sha",
    "upstream_file",
    "lines",
    "symbol",
    "status",
    "rust_owner",
    "remaining",
)
RENDER_CONTEXT_FILE_MAP_STATUSES = {"ported", "partial", "missing"}
RENDER_CONTEXT_FILE_MAP_SOURCES = {
    "renderer/include/rive/renderer/metal/render_context_metal_impl.h",
    "renderer/src/metal/render_context_metal_impl.mm",
}
RENDER_CONTEXT_FIELD_MAP_COLUMNS = (
    "version",
    "upstream_sha",
    "upstream_file",
    "cpp_type",
    "cpp_field",
    "declaration_line",
    "configuration",
    "rust_owner",
    "rust_field",
    "construction_and_publication",
    "mutation_thread",
    "submission_and_completion",
    "destruction_order",
    "null_and_failure",
    "safe_rust_adaptation",
    "status",
    "evidence",
)
RENDER_CONTEXT_FIELD_MAP_STATUSES = {"review-needed", "prepared", "verified"}
RENDER_CONTEXT_CONFIGURATION_MAP_COLUMNS = (
    "version",
    "upstream_sha",
    "upstream_file",
    "block",
    "lines",
    "branch_lines",
    "configurations",
    "source_behavior",
    "rust_owner",
    "rust_configuration",
    "status",
    "remaining",
    "evidence",
)
RENDER_CONTEXT_CONFIGURATION_MAP_STATUSES = {
    "review-needed",
    "prepared",
    "verified",
}
RENDER_CONTEXT_CONFIGURATION_MAP_SOURCES = {
    "renderer/include/rive/renderer/metal/render_context_metal_impl.h",
    "renderer/src/metal/render_context_metal_impl.mm",
    "renderer/src/metal/background_shader_compiler.h",
    "renderer/src/metal/background_shader_compiler.mm",
}
TRANSLATION_CONVENTION_COLUMNS = (
    "version",
    "convention",
    "cpp_shape",
    "rust_rule",
    "invariant",
    "forbidden",
    "status",
    "evidence",
)
TRANSLATION_CONVENTION_STATUSES = {"review-needed", "frozen", "verified"}
TRANSLATION_CONVENTION_IDS = {
    "objc-retained-nullable",
    "intrusive-reference-counting",
    "byte-ranges-and-alignment",
    "enums-flags-slots-formats",
    "assertions-and-errors",
    "callbacks-workers-completion",
    "preprocessor-configurations",
    "destruction-and-drop-order",
}
RENDER_CONTEXT_FIELD_DECLARATION_SPANS = (
    (
        "renderer/include/rive/renderer/metal/render_context_metal_impl.h",
        "RenderTargetMetal",
        75,
        86,
        re.compile(r"\b(m_[A-Za-z0-9_]+)\b"),
    ),
    (
        "renderer/include/rive/renderer/metal/render_context_metal_impl.h",
        "RenderContextMetalImpl::ContextOptions",
        98,
        108,
        re.compile(
            r"\b(shaderCompilationMode|disableFramebufferReads|synthesizedFailureType)\b"
        ),
    ),
    (
        "renderer/src/metal/render_context_metal_impl.mm",
        "RenderContextMetalImpl::DrawPipeline",
        399,
        400,
        re.compile(r"\b(m_[A-Za-z0-9_]+)\b"),
    ),
    (
        "renderer/include/rive/renderer/metal/render_context_metal_impl.h",
        "RenderContextMetalImpl",
        235,
        280,
        re.compile(r"\b(m_[A-Za-z0-9_]+)\b"),
    ),
)
FOUNDATION_TRIAL_UNITS = {
    "ore-types": {"renderer/include/rive/renderer/ore/ore_types.hpp"},
    "ore-rstb-container": {
        "renderer/include/rive/renderer/ore/ore_rstb_entry_container.hpp"
    },
    "ore-binding-map": {
        "renderer/include/rive/renderer/ore/ore_binding_map.hpp",
        "renderer/src/ore/ore_binding_map.cpp",
    },
}
FOUNDATION_TRIAL_TARGETS = {
    "ore-types": {"crates/nuxie-ore-metal/src/types.rs"},
    "ore-rstb-container": {
        "crates/nuxie-ore-metal/src/rstb_entry_container.rs"
    },
    "ore-binding-map": {"crates/nuxie-ore-metal/src/binding_map.rs"},
}
CITATION_RE = re.compile(r"^(cpp|rust):(.+):(\d+)(?:-(\d+))?$")


class CheckFailure(Exception):
    """Raised when the port campaign documents are incomplete or inconsistent."""


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


def git_tracked_file(repo_root: pathlib.Path, relative: str) -> bool:
    if not (repo_root / relative).is_file():
        return False
    result = subprocess.run(
        ["git", "-C", str(repo_root), "ls-files", "--error-unmatch", "--", relative],
        text=True,
        capture_output=True,
        check=False,
    )
    return result.returncode == 0


def duplicate_values(values: Iterable[str]) -> list[str]:
    counts = collections.Counter(values)
    return sorted(value for value, count in counts.items() if count > 1)


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def parse_provenance(path: pathlib.Path, errors: list[str]) -> dict[str, str]:
    fields: dict[str, str] = {}
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as error:
        errors.append(f"cannot read reference provenance {path}: {error}")
        return fields
    for line_number, raw_line in enumerate(lines, 1):
        line = raw_line.strip()
        if not line:
            continue
        if "=" not in line:
            errors.append(f"{path} line {line_number} is not key=value provenance")
            continue
        key, value = (part.strip() for part in line.split("=", 1))
        if not key or not value:
            errors.append(f"{path} line {line_number} has an empty key or value")
        elif key in fields:
            errors.append(f"{path} repeats provenance field `{key}`")
        else:
            fields[key] = value
    return fields


def validate_reference_provenance(
    manifest: dict[str, Any], repo_root: pathlib.Path, errors: list[str]
) -> None:
    rows = list(manifest.get("reference_provenance", []))
    duplicates = duplicate_values(str(row.get("id", "")) for row in rows)
    if duplicates:
        errors.append(f"duplicate reference provenance rows: {', '.join(duplicates)}")
    upstream_ref = str(manifest.get("upstream_ref", ""))
    for row in rows:
        record_id = str(row.get("id", ""))
        relative_paths = {
            key: str(row.get(key, "")) for key in ("path", "stream", "reference")
        }
        resolved: dict[str, pathlib.Path] = {}
        for key, relative in relative_paths.items():
            path = repo_root / relative
            resolved[key] = path
            if not relative or not path.is_file():
                errors.append(
                    f"reference provenance {record_id} names missing {key} path {relative}"
                )
            elif not git_tracked_file(repo_root, relative):
                errors.append(
                    f"reference provenance {record_id} names untracked {key} path {relative}"
                )
        if not all(path.is_file() for path in resolved.values()):
            continue
        fields = parse_provenance(resolved["path"], errors)
        expected = {
            "provenance_schema": "1",
            "renderer_implementation": str(row.get("renderer_implementation", "")),
            "capture_tool": str(row.get("capture_tool", "")),
            "backend": str(row.get("backend", "")),
            "adapter_device": str(row.get("adapter_device", "")),
            "case_id": record_id,
            "runtime_revision": upstream_ref,
            "replay_sha256": str(row.get("replay_sha256", "")),
            "reference_input_manifest_sha256": str(
                row.get("reference_input_manifest_sha256", "")
            ),
            "stream_sha256": sha256_file(resolved["stream"]),
            "png_sha256": sha256_file(resolved["reference"]),
            "frame": str(row.get("frame", "")),
            "frame_width": str(row.get("frame_width", "")),
            "frame_height": str(row.get("frame_height", "")),
            "mode": str(row.get("mode", "")),
            "sample_count": str(row.get("sample_count", "")),
        }
        for key, expected_value in expected.items():
            actual = fields.get(key)
            if actual != expected_value:
                errors.append(
                    f"reference provenance {record_id} {key} `{actual}` does not match `{expected_value}`"
                )
        for key in ("replay_sha256", "reference_input_manifest_sha256"):
            value = fields.get(key, "")
            if len(value) != 64 or any(character not in "0123456789abcdef" for character in value):
                errors.append(
                    f"reference provenance {record_id} {key} must be 64 lowercase hex characters"
                )


def expand_source_scope(
    upstream_root: pathlib.Path, globs: list[str], excludes: list[str]
) -> set[str]:
    excluded = set(excludes)
    return {
        path.relative_to(upstream_root).as_posix()
        for pattern in globs
        for path in upstream_root.glob(pattern)
        if path.is_file() and path.relative_to(upstream_root).as_posix() not in excluded
    }


def validate_render_context_file_map(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    relative = str(manifest.get("render_context_file_map", ""))
    path = repo_root / relative
    if not relative or not path.is_file():
        errors.append(f"missing render-context file map {relative}")
        return
    if not git_tracked_file(repo_root, relative):
        errors.append(f"untracked render-context file map {relative}")

    try:
        with path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = tuple(reader.fieldnames or ())
            rows = list(reader)
    except (OSError, csv.Error) as error:
        errors.append(f"cannot read render-context file map {relative}: {error}")
        return
    if fieldnames != RENDER_CONTEXT_FILE_MAP_COLUMNS:
        errors.append(
            "render-context file map schema must be: "
            + "\t".join(RENDER_CONTEXT_FILE_MAP_COLUMNS)
        )
        return

    upstream_ref = str(manifest.get("upstream_ref", ""))
    rows_by_source: dict[str, list[tuple[int, int, int]]] = collections.defaultdict(
        list
    )
    for line_number, row in enumerate(rows, 2):
        if None in row:
            errors.append(
                f"render-context file map line {line_number} has surplus columns"
            )
        upstream_file = str(row.get("upstream_file", ""))
        if row.get("version") != "1":
            errors.append(
                f"render-context file map line {line_number} has invalid version"
            )
        if row.get("upstream_sha") != upstream_ref:
            errors.append(
                f"render-context file map line {line_number} pin does not match upstream_ref"
            )
        if upstream_file not in RENDER_CONTEXT_FILE_MAP_SOURCES:
            errors.append(
                f"render-context file map line {line_number} names unexpected source {upstream_file}"
            )
        status = str(row.get("status", ""))
        if status not in RENDER_CONTEXT_FILE_MAP_STATUSES:
            errors.append(
                f"render-context file map line {line_number} has invalid status `{status}`"
            )
        if not str(row.get("symbol", "")).strip():
            errors.append(
                f"render-context file map line {line_number} has an empty symbol"
            )
        if not str(row.get("remaining", "")).strip():
            errors.append(
                f"render-context file map line {line_number} has empty remaining work"
            )
        rust_owner = str(row.get("rust_owner", ""))
        if status == "missing":
            if rust_owner != "-":
                errors.append(
                    f"render-context file map line {line_number} marks missing work with a Rust owner"
                )
        elif rust_owner == "-":
            errors.append(
                f"render-context file map line {line_number} is {status} without a Rust owner"
            )
        elif not (repo_root / rust_owner).is_file():
            errors.append(
                f"render-context file map line {line_number} names missing Rust owner {rust_owner}"
            )
        elif not git_tracked_file(repo_root, rust_owner):
            errors.append(
                f"render-context file map line {line_number} names untracked Rust owner {rust_owner}"
            )

        range_match = re.fullmatch(r"(\d+)-(\d+)", str(row.get("lines", "")))
        if range_match is None:
            errors.append(
                f"render-context file map line {line_number} has invalid line range"
            )
            continue
        start, end = (int(value) for value in range_match.groups())
        if start < 1 or end < start:
            errors.append(
                f"render-context file map line {line_number} has invalid line range {start}-{end}"
            )
            continue
        rows_by_source[upstream_file].append((line_number, start, end))

    mapped_sources = set(rows_by_source)
    if mapped_sources != RENDER_CONTEXT_FILE_MAP_SOURCES:
        missing = sorted(RENDER_CONTEXT_FILE_MAP_SOURCES - mapped_sources)
        extra = sorted(mapped_sources - RENDER_CONTEXT_FILE_MAP_SOURCES)
        if missing:
            errors.append("render-context file map omits sources: " + ", ".join(missing))
        if extra:
            errors.append(
                "render-context file map includes extra sources: " + ", ".join(extra)
            )

    for upstream_file in sorted(RENDER_CONTEXT_FILE_MAP_SOURCES):
        source_path = upstream_root / upstream_file
        if not source_path.is_file():
            errors.append(
                f"render-context file map source does not exist: {upstream_file}"
            )
            continue
        with source_path.open(encoding="utf-8", errors="replace") as source:
            line_count = sum(1 for _ in source)
        expected_start = 1
        for line_number, start, end in rows_by_source.get(upstream_file, []):
            if start != expected_start:
                errors.append(
                    "render-context file map does not continuously cover "
                    f"{upstream_file}: line {line_number} starts at {start}, "
                    f"expected {expected_start}"
                )
            if end > line_count:
                errors.append(
                    f"render-context file map line {line_number} ends outside {upstream_file}"
                )
            expected_start = end + 1
        if expected_start != line_count + 1:
            errors.append(
                "render-context file map does not reach the end of "
                f"{upstream_file}: stopped at {expected_start - 1}, expected {line_count}"
            )


def extract_render_context_field_declarations(
    upstream_root: pathlib.Path,
    errors: list[str],
) -> dict[tuple[str, str, str], int]:
    declarations: dict[tuple[str, str, str], int] = {}
    source_lines: dict[str, list[str]] = {}
    for upstream_file, cpp_type, start, end, pattern in RENDER_CONTEXT_FIELD_DECLARATION_SPANS:
        if upstream_file not in source_lines:
            path = upstream_root / upstream_file
            try:
                source_lines[upstream_file] = path.read_text(encoding="utf-8").splitlines()
            except OSError as error:
                errors.append(
                    f"cannot read render-context field source {upstream_file}: {error}"
                )
                continue
        lines = source_lines[upstream_file]
        if end > len(lines):
            errors.append(
                f"render-context field span {upstream_file}:{start}-{end} exceeds source EOF"
            )
            continue
        for line_number in range(start, end + 1):
            for match in pattern.finditer(lines[line_number - 1]):
                key = (upstream_file, cpp_type, match.group(1))
                if key in declarations:
                    errors.append(
                        "render-context field declaration appears more than once: "
                        + ":".join(key)
                    )
                declarations[key] = line_number
    return declarations


def compare_render_context_field_rows(
    rows: list[dict[str, str]],
    declarations: dict[tuple[str, str, str], int],
    errors: list[str],
) -> None:
    ledger: dict[tuple[str, str, str], tuple[int, int]] = {}
    for line_number, row in enumerate(rows, 2):
        key = (row["upstream_file"], row["cpp_type"], row["cpp_field"])
        try:
            declaration_line = int(row["declaration_line"])
        except ValueError:
            errors.append(
                f"render-context field map line {line_number} has invalid declaration line"
            )
            continue
        if key in ledger:
            errors.append("duplicate render-context field row: " + ":".join(key))
        ledger[key] = (line_number, declaration_line)

    missing = sorted(set(declarations) - set(ledger))
    extra = sorted(set(ledger) - set(declarations))
    if missing:
        errors.append(
            "render-context field map omits declarations: "
            + ", ".join(":".join(key) for key in missing)
        )
    if extra:
        errors.append(
            "render-context field map invents declarations: "
            + ", ".join(":".join(key) for key in extra)
        )
    for key in sorted(set(declarations) & set(ledger)):
        line_number, actual_line = ledger[key]
        expected_line = declarations[key]
        if actual_line != expected_line:
            errors.append(
                f"render-context field map line {line_number} locates "
                f"{':'.join(key)} at {actual_line}, expected {expected_line}"
            )


def validate_render_context_field_map(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    relative = str(manifest.get("render_context_field_map", ""))
    path = repo_root / relative
    if not relative or not path.is_file():
        errors.append(f"missing render-context field map {relative}")
        return
    if not git_tracked_file(repo_root, relative):
        errors.append(f"untracked render-context field map {relative}")

    try:
        with path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = tuple(reader.fieldnames or ())
            rows = [
                {str(key): str(value or "") for key, value in row.items() if key is not None}
                for row in reader
            ]
    except (OSError, csv.Error) as error:
        errors.append(f"cannot read render-context field map {relative}: {error}")
        return
    if fieldnames != RENDER_CONTEXT_FIELD_MAP_COLUMNS:
        errors.append(
            "render-context field map schema must be: "
            + "\t".join(RENDER_CONTEXT_FIELD_MAP_COLUMNS)
        )
        return

    declarations = extract_render_context_field_declarations(upstream_root, errors)
    compare_render_context_field_rows(rows, declarations, errors)
    upstream_ref = str(manifest.get("upstream_ref", ""))
    required_prose = (
        "configuration",
        "rust_field",
        "construction_and_publication",
        "mutation_thread",
        "submission_and_completion",
        "destruction_order",
        "null_and_failure",
        "safe_rust_adaptation",
    )
    for line_number, row in enumerate(rows, 2):
        if row["version"] != "1":
            errors.append(f"render-context field map line {line_number} has invalid version")
        if row["upstream_sha"] != upstream_ref:
            errors.append(
                f"render-context field map line {line_number} pin does not match upstream_ref"
            )
        status = row["status"]
        if status not in RENDER_CONTEXT_FIELD_MAP_STATUSES:
            errors.append(
                f"render-context field map line {line_number} has invalid status `{status}`"
            )
        for column in required_prose:
            if not row[column].strip():
                errors.append(
                    f"render-context field map line {line_number} has empty {column}"
                )
        rust_owner = row["rust_owner"]
        if rust_owner == "-":
            if status != "review-needed":
                errors.append(
                    f"render-context field map line {line_number} lacks a Rust owner but is {status}"
                )
        elif not (repo_root / rust_owner).is_file():
            errors.append(
                f"render-context field map line {line_number} names missing Rust owner {rust_owner}"
            )
        elif not git_tracked_file(repo_root, rust_owner):
            errors.append(
                f"render-context field map line {line_number} names untracked Rust owner {rust_owner}"
            )
        evidence = [value.strip() for value in row["evidence"].split(";") if value.strip()]
        if status in {"prepared", "verified"} and not evidence:
            errors.append(
                f"render-context field map line {line_number} is {status} without evidence"
            )
        for citation in evidence:
            validate_evidence_citation(citation, repo_root, upstream_root, errors)


def extract_render_context_configuration_blocks(
    upstream_root: pathlib.Path, errors: list[str]
) -> dict[tuple[str, int, int], tuple[int, ...]]:
    blocks: dict[tuple[str, int, int], tuple[int, ...]] = {}
    opening = re.compile(r"^\s*#(?:if\s|ifdef\s|ifndef\s)")
    branch = re.compile(r"^\s*#(?:elif\s|else(?:\s|$))")
    closing = re.compile(r"^\s*#endif(?:\s|$)")
    for relative in sorted(RENDER_CONTEXT_CONFIGURATION_MAP_SOURCES):
        path = upstream_root / relative
        if not path.is_file():
            errors.append(f"missing pinned configuration source {relative}")
            continue
        stack: list[tuple[int, list[int]]] = []
        for line_number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
            if opening.match(line):
                stack.append((line_number, [line_number]))
            elif branch.match(line):
                if not stack:
                    errors.append(
                        f"orphan preprocessor branch in {relative}:{line_number}"
                    )
                else:
                    stack[-1][1].append(line_number)
            elif closing.match(line):
                if not stack:
                    errors.append(f"orphan #endif in {relative}:{line_number}")
                    continue
                start, branches = stack.pop()
                blocks[(relative, start, line_number)] = tuple(branches)
        if stack:
            errors.append(
                f"unterminated preprocessor blocks in {relative}: "
                + ", ".join(str(start) for start, _ in stack)
            )
    return blocks


def compare_render_context_configuration_rows(
    rows: list[dict[str, str]],
    blocks: dict[tuple[str, int, int], tuple[int, ...]],
    errors: list[str],
) -> None:
    ledger: dict[tuple[str, int, int], tuple[int, tuple[int, ...]]] = {}
    for line_number, row in enumerate(rows, 2):
        match = re.fullmatch(r"(\d+)-(\d+)", row.get("lines", ""))
        if not match:
            errors.append(
                f"render-context configuration map line {line_number} has invalid range"
            )
            continue
        start, end = map(int, match.groups())
        try:
            branches = tuple(
                int(value) for value in row.get("branch_lines", "").split(",")
            )
        except ValueError:
            errors.append(
                f"render-context configuration map line {line_number} has invalid branch lines"
            )
            continue
        key = (row.get("upstream_file", ""), start, end)
        if key in ledger:
            errors.append(
                "duplicate render-context configuration row: "
                + f"{key[0]}:{start}-{end}"
            )
        ledger[key] = (line_number, branches)

    missing = sorted(set(blocks) - set(ledger))
    extra = sorted(set(ledger) - set(blocks))
    if missing:
        errors.append(
            "render-context configuration map omits blocks: "
            + ", ".join(f"{path}:{start}-{end}" for path, start, end in missing)
        )
    if extra:
        errors.append(
            "render-context configuration map invents blocks: "
            + ", ".join(f"{path}:{start}-{end}" for path, start, end in extra)
        )
    for key in sorted(set(blocks) & set(ledger)):
        line_number, actual = ledger[key]
        expected = blocks[key]
        if actual != expected:
            errors.append(
                f"render-context configuration map line {line_number} has branch lines "
                f"{actual}, expected {expected} for {key[0]}:{key[1]}-{key[2]}"
            )


def validate_render_context_configuration_map(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    relative = str(manifest.get("render_context_configuration_map", ""))
    path = repo_root / relative
    if not relative or not path.is_file():
        errors.append(f"missing render-context configuration map {relative}")
        return
    if not git_tracked_file(repo_root, relative):
        errors.append(f"untracked render-context configuration map {relative}")
    try:
        with path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = tuple(reader.fieldnames or ())
            rows = [
                {str(key): str(value or "") for key, value in row.items() if key is not None}
                for row in reader
            ]
    except (OSError, csv.Error) as error:
        errors.append(f"cannot read render-context configuration map {relative}: {error}")
        return
    if fieldnames != RENDER_CONTEXT_CONFIGURATION_MAP_COLUMNS:
        errors.append(
            "render-context configuration map schema must be: "
            + "\t".join(RENDER_CONTEXT_CONFIGURATION_MAP_COLUMNS)
        )
        return

    blocks = extract_render_context_configuration_blocks(upstream_root, errors)
    compare_render_context_configuration_rows(rows, blocks, errors)
    upstream_ref = str(manifest.get("upstream_ref", ""))
    required_prose = (
        "block",
        "configurations",
        "source_behavior",
        "rust_configuration",
        "remaining",
    )
    block_names: set[str] = set()
    for line_number, row in enumerate(rows, 2):
        if row["version"] != "1":
            errors.append(
                f"render-context configuration map line {line_number} has invalid version"
            )
        if row["upstream_sha"] != upstream_ref:
            errors.append(
                f"render-context configuration map line {line_number} pin does not match upstream_ref"
            )
        if row["block"] in block_names:
            errors.append(
                f"duplicate render-context configuration block name `{row['block']}`"
            )
        block_names.add(row["block"])
        status = row["status"]
        if status not in RENDER_CONTEXT_CONFIGURATION_MAP_STATUSES:
            errors.append(
                f"render-context configuration map line {line_number} has invalid status `{status}`"
            )
        for column in required_prose:
            if not row[column].strip():
                errors.append(
                    f"render-context configuration map line {line_number} has empty {column}"
                )
        rust_owner = row["rust_owner"]
        if rust_owner == "-":
            if status != "review-needed":
                errors.append(
                    f"render-context configuration map line {line_number} lacks a Rust owner but is {status}"
                )
        elif not (repo_root / rust_owner).is_file():
            errors.append(
                f"render-context configuration map line {line_number} names missing Rust owner {rust_owner}"
            )
        elif not git_tracked_file(repo_root, rust_owner):
            errors.append(
                f"render-context configuration map line {line_number} names untracked Rust owner {rust_owner}"
            )
        evidence = [value.strip() for value in row["evidence"].split(";") if value.strip()]
        if status in {"prepared", "verified"} and not evidence:
            errors.append(
                f"render-context configuration map line {line_number} is {status} without evidence"
            )
        for citation in evidence:
            validate_evidence_citation(citation, repo_root, upstream_root, errors)


def compare_translation_convention_ids(ids: list[str], errors: list[str]) -> None:
    actual_ids = set(ids)
    missing = sorted(TRANSLATION_CONVENTION_IDS - actual_ids)
    extra = sorted(actual_ids - TRANSLATION_CONVENTION_IDS)
    if len(ids) != len(actual_ids):
        errors.append("Metal translation conventions contain duplicate IDs")
    if missing:
        errors.append("Metal translation conventions omit: " + ", ".join(missing))
    if extra:
        errors.append("Metal translation conventions invent: " + ", ".join(extra))


def validate_translation_conventions(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    relative = str(manifest.get("translation_conventions", ""))
    path = repo_root / relative
    if not relative or not path.is_file():
        errors.append(f"missing Metal translation conventions {relative}")
        return
    if not git_tracked_file(repo_root, relative):
        errors.append(f"untracked Metal translation conventions {relative}")
    try:
        with path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = tuple(reader.fieldnames or ())
            rows = [
                {str(key): str(value or "") for key, value in row.items() if key is not None}
                for row in reader
            ]
    except (OSError, csv.Error) as error:
        errors.append(f"cannot read Metal translation conventions {relative}: {error}")
        return
    if fieldnames != TRANSLATION_CONVENTION_COLUMNS:
        errors.append(
            "Metal translation convention schema must be: "
            + "\t".join(TRANSLATION_CONVENTION_COLUMNS)
        )
        return
    compare_translation_convention_ids(
        [row["convention"] for row in rows], errors
    )
    for line_number, row in enumerate(rows, 2):
        if row["version"] != "1":
            errors.append(
                f"Metal translation convention line {line_number} has invalid version"
            )
        if row["status"] not in TRANSLATION_CONVENTION_STATUSES:
            errors.append(
                f"Metal translation convention line {line_number} has invalid status `{row['status']}`"
            )
        for column in ("cpp_shape", "rust_rule", "invariant", "forbidden"):
            if not row[column].strip():
                errors.append(
                    f"Metal translation convention line {line_number} has empty {column}"
                )
        evidence = [value.strip() for value in row["evidence"].split(";") if value.strip()]
        if row["status"] in {"frozen", "verified"} and not evidence:
            errors.append(
                f"Metal translation convention line {line_number} is {row['status']} without evidence"
            )
        for citation in evidence:
            validate_evidence_citation(citation, repo_root, upstream_root, errors)


def validate_citation(
    citation: str,
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    match = CITATION_RE.fullmatch(citation)
    if match is None:
        errors.append(f"invalid citation (expected cpp|rust:path:line): {citation}")
        return
    root_kind, relative, start_text, end_text = match.groups()
    root = upstream_root if root_kind == "cpp" else repo_root
    source = root / relative
    if not source.is_file():
        errors.append(f"citation file does not exist: {citation}")
        return
    with source.open(encoding="utf-8", errors="replace") as lines:
        line_count = sum(1 for _ in lines)
    start = int(start_text)
    end = int(end_text or start_text)
    if start < 1 or end < start or end > line_count:
        errors.append(
            f"citation line is outside {relative} (1..{line_count}): {citation}"
        )


def validate_evidence_citation(
    citation: str,
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    head, separator, ranges = citation.rpartition(":")
    parts = ranges.split(",") if separator else []
    if len(parts) > 1 and all(re.fullmatch(r"\d+(?:-\d+)?", part) for part in parts):
        for line_range in parts:
            validate_citation(
                f"{head}:{line_range}", repo_root, upstream_root, errors
            )
        return
    validate_citation(citation, repo_root, upstream_root, errors)


def validate_source_rows(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> collections.Counter[str]:
    expected = expand_source_scope(
        upstream_root,
        [str(value) for value in manifest.get("source_globs", [])],
        [str(value) for value in manifest.get("source_excludes", [])],
    )
    rows = list(manifest.get("source", []))
    paths = [str(row.get("upstream", "")) for row in rows]
    duplicates = duplicate_values(paths)
    if duplicates:
        errors.append(f"duplicate source rows: {', '.join(duplicates)}")
    actual = set(paths)
    missing = sorted(expected - actual)
    extra = sorted(actual - expected)
    if missing:
        errors.append("untracked upstream Metal sources: " + ", ".join(missing))
    if extra:
        errors.append("out-of-scope source rows: " + ", ".join(extra))

    counts: collections.Counter[str] = collections.Counter()
    for row in rows:
        path = str(row.get("upstream", ""))
        status = str(row.get("status", ""))
        issue = str(row.get("issue", ""))
        lane = str(row.get("lane", ""))
        rust_modules = [str(value) for value in row.get("rust_modules", [])]
        evidence = [str(value) for value in row.get("evidence", [])]
        parity_evidence = [str(value) for value in row.get("parity_evidence", [])]
        if status not in SOURCE_STATUSES:
            errors.append(f"{path} has invalid status `{status}`")
        else:
            counts[status] += 1
        if not re.fullmatch(r"UNIV-\d+", issue):
            errors.append(f"{path} has invalid or missing issue `{issue}`")
        if lane not in {"renderer-platform", "ore-metal", "platform-shaders"}:
            errors.append(f"{path} has invalid lane `{lane}`")
        if status in VERIFIED_STATUSES:
            if not rust_modules:
                errors.append(f"{path} is {status} without a Rust module")
            if not evidence:
                errors.append(f"{path} is {status} without verification evidence")
            for relative in rust_modules:
                if not (repo_root / relative).is_file():
                    errors.append(f"{path} names missing Rust module {relative}")
                elif not git_tracked_file(repo_root, relative):
                    errors.append(f"{path} names untracked Rust module {relative}")
            for relative in evidence:
                if not (repo_root / relative).is_file():
                    errors.append(f"{path} names missing evidence path {relative}")
                elif not git_tracked_file(repo_root, relative):
                    errors.append(f"{path} names untracked evidence path {relative}")
        if status == "verified":
            if not parity_evidence:
                errors.append(f"{path} is verified without parity evidence")
            for relative in parity_evidence:
                if not (repo_root / relative).is_file():
                    errors.append(f"{path} names missing parity evidence path {relative}")
                elif not git_tracked_file(repo_root, relative):
                    errors.append(f"{path} names untracked parity evidence path {relative}")
    return counts


def validate_translation_units(
    manifest: dict[str, Any], errors: list[str]
) -> list[dict[str, Any]]:
    units = list(manifest.get("translation_unit", []))
    unit_ids = [str(unit.get("id", "")) for unit in units]
    duplicates = duplicate_values(unit_ids)
    if duplicates:
        errors.append(f"duplicate translation-unit ids: {', '.join(duplicates)}")

    source_rows = list(manifest.get("source", []))
    ore_sources = {
        str(row.get("upstream", ""))
        for row in source_rows
        if row.get("lane") == "ore-metal"
    }
    pending_ore_sources = {
        str(row.get("upstream", ""))
        for row in source_rows
        if row.get("lane") == "ore-metal" and row.get("status") == "pending"
    }
    assigned_sources = [
        str(source)
        for unit in units
        for source in list(unit.get("sources", []))
    ]
    overlapping_sources = duplicate_values(assigned_sources)
    if overlapping_sources:
        errors.append(
            "overlapping translation-unit sources: "
            + ", ".join(overlapping_sources)
        )
    missing_sources = sorted(pending_ore_sources - set(assigned_sources))
    if missing_sources:
        errors.append("missing pending ORE sources: " + ", ".join(missing_sources))
    outside_sources = sorted(set(assigned_sources) - ore_sources)
    if outside_sources:
        errors.append(
            "translation-unit sources outside the ORE lane: "
            + ", ".join(outside_sources)
        )

    upstream_ref = str(manifest.get("upstream_ref", ""))
    rust_target_owners: dict[str, list[str]] = collections.defaultdict(list)
    worker_claims: list[str] = []
    unit_by_id = {str(unit.get("id", "")): unit for unit in units}
    dependency_graph: dict[str, list[str]] = {}
    for unit in units:
        unit_id = str(unit.get("id", ""))
        sources = [str(source) for source in unit.get("sources", [])]
        dependencies = [str(value) for value in unit.get("dependencies", [])]
        rust_targets = [str(value) for value in unit.get("rust_targets", [])]
        phase = str(unit.get("phase", ""))
        status = str(unit.get("status", ""))
        worker_claim = str(unit.get("worker_claim", ""))
        if not re.fullmatch(r"[a-z][a-z0-9-]*", unit_id):
            errors.append(f"translation unit has invalid id `{unit_id}`")
        if not sources:
            errors.append(f"translation unit {unit_id} has no sources")
        if duplicate_values(sources):
            errors.append(f"translation unit {unit_id} repeats a source")
        if phase not in TRANSLATION_PHASES:
            errors.append(f"translation unit {unit_id} has invalid phase `{phase}`")
        if status not in TRANSLATION_STATUSES:
            errors.append(f"translation unit {unit_id} has invalid status `{status}`")
        if str(unit.get("base_ref", "")) != upstream_ref:
            errors.append(
                f"translation unit {unit_id} base_ref does not match upstream_ref"
            )
        if unit.get("worker_role") not in TRANSLATION_WORKER_ROLES:
            errors.append(f"translation unit {unit_id} has invalid worker role")
        if worker_claim != "unclaimed" and not re.fullmatch(
            r"[a-z][a-z0-9-]*", worker_claim
        ):
            errors.append(f"translation unit {unit_id} has invalid worker claim")
        if status != "pending" and worker_claim == "unclaimed":
            errors.append(
                f"translation unit {unit_id} is {status} without a worker claim"
            )
        if worker_claim and worker_claim != "unclaimed":
            worker_claims.append(worker_claim)
        for field in ("source_reviewer_role", "ownership_reviewer_role"):
            if unit.get(field) not in TRANSLATION_REVIEWER_ROLES:
                errors.append(
                    f"translation unit {unit_id} has invalid {field.replace('_', ' ')}"
                )
        if unit.get("fixer_role") not in TRANSLATION_FIXER_ROLES:
            errors.append(f"translation unit {unit_id} has invalid fixer role")
        if unit.get("requires_lifetime_rows") is not True:
            errors.append(
                f"translation unit {unit_id} must require lifetime rows"
            )
        if not rust_targets:
            errors.append(f"translation unit {unit_id} has no Rust targets")
        for target in rust_targets:
            path = pathlib.PurePosixPath(target)
            canonical_target = path.as_posix()
            if (
                path.is_absolute()
                or ".." in path.parts
                or target in {"", "."}
                or canonical_target != target
                or path.suffix != ".rs"
            ):
                errors.append(
                    f"translation unit {unit_id} Rust target must be a canonical .rs file: {target}"
                )
            if not target.startswith("crates/nuxie-ore-metal/src/"):
                errors.append(
                    f"translation unit {unit_id} Rust target is outside "
                    f"crates/nuxie-ore-metal/src: {target}"
                )
            rust_target_owners[canonical_target].append(unit_id)
        if duplicate_values(dependencies):
            errors.append(f"translation unit {unit_id} repeats a dependency")
        if unit_id in dependencies:
            errors.append(f"translation unit {unit_id} depends on itself")
        dependency_graph[unit_id] = dependencies

    for target, owners in sorted(rust_target_owners.items()):
        if len(owners) > 1:
            errors.append(
                f"Rust target {target} is owned by multiple translation units: "
                + ", ".join(owners)
            )
    duplicate_claims = duplicate_values(worker_claims)
    if duplicate_claims:
        errors.append("duplicate worker claims: " + ", ".join(duplicate_claims))
    for unit_id, dependencies in dependency_graph.items():
        missing_dependencies = sorted(set(dependencies) - set(unit_by_id))
        if missing_dependencies:
            errors.append(
                f"translation unit {unit_id} has unknown dependencies: "
                + ", ".join(missing_dependencies)
            )

    visit_state: dict[str, int] = {}

    def visit(unit_id: str, trail: list[str]) -> None:
        state = visit_state.get(unit_id, 0)
        if state == 2:
            return
        if state == 1:
            cycle_start = trail.index(unit_id) if unit_id in trail else 0
            cycle = trail[cycle_start:] + [unit_id]
            errors.append("translation-unit dependency cycle: " + " -> ".join(cycle))
            return
        visit_state[unit_id] = 1
        for dependency in dependency_graph.get(unit_id, []):
            if dependency in dependency_graph:
                visit(dependency, trail + [unit_id])
        visit_state[unit_id] = 2

    for unit_id in unit_ids:
        visit(unit_id, [])

    trial_units = {
        str(unit.get("id", "")): {str(source) for source in unit.get("sources", [])}
        for unit in units
        if unit.get("phase") == "trial"
    }
    if trial_units != FOUNDATION_TRIAL_UNITS:
        errors.append(
            "trial translation units must be the compileable ore-types, "
            "ore-rstb-container, and ore-binding-map foundations"
        )
    for unit_id in FOUNDATION_TRIAL_UNITS:
        unit = unit_by_id.get(unit_id)
        if unit is not None:
            if unit.get("dependencies"):
                errors.append(
                    f"foundation trial unit {unit_id} must have no dependencies"
                )
            if unit.get("worker_role") != "luna-extra-high":
                errors.append(
                    f"foundation trial unit {unit_id} must use luna-extra-high"
                )
            targets = {str(target) for target in unit.get("rust_targets", [])}
            if targets != FOUNDATION_TRIAL_TARGETS[unit_id]:
                errors.append(
                    f"foundation trial unit {unit_id} has drifted Rust targets"
                )
    gpu_resource = unit_by_id.get("gpu-resource")
    if gpu_resource is not None and gpu_resource.get("worker_role") != "sol-high":
        errors.append("gpu-resource must use sol-high for purgatory adaptation")
    return units


def validate_lifetime_ledger(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> list[dict[str, str]]:
    relative = str(manifest.get("lifetime_ledger", ""))
    path = repo_root / relative
    if not relative or not path.is_file():
        errors.append(f"missing lifetime ledger {relative}")
        return []
    if not git_tracked_file(repo_root, relative):
        errors.append(f"untracked lifetime ledger {relative}")

    try:
        with path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = tuple(reader.fieldnames or ())
            rows = []
            for line_number, row in enumerate(reader, 2):
                if None in row:
                    errors.append(
                        f"lifetime ledger line {line_number} has surplus columns"
                    )
                rows.append(
                    {
                        str(key): str(value or "")
                        for key, value in row.items()
                        if key is not None
                    }
                )
    except (OSError, csv.Error) as error:
        errors.append(f"cannot read lifetime ledger {relative}: {error}")
        return []
    if fieldnames != LIFETIME_COLUMNS:
        errors.append(
            "lifetime ledger schema must be: " + "\t".join(LIFETIME_COLUMNS)
        )
        return rows

    units = list(manifest.get("translation_unit", []))
    units_by_id = {str(unit.get("id", "")): unit for unit in units}
    source_rows = list(manifest.get("source", []))
    ore_sources = {
        str(row.get("upstream", ""))
        for row in source_rows
        if row.get("lane") == "ore-metal"
    }
    upstream_ref = str(manifest.get("upstream_ref", ""))
    ledger_keys: list[str] = []
    rows_by_unit: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    for line_number, row in enumerate(rows, 2):
        unit_id = row["unit"].strip()
        upstream_path = row["upstream_path"].strip()
        field = row["field"].strip()
        status = row["status"].strip()
        row_key = f"{unit_id}:{upstream_path}:{field}"
        ledger_keys.append(row_key)
        if row["schema_version"].strip() != "1":
            errors.append(f"lifetime ledger line {line_number} has invalid schema version")
        if row["upstream_ref"].strip() != upstream_ref:
            errors.append(f"lifetime ledger line {line_number} pin does not match upstream_ref")
        unit = units_by_id.get(unit_id)
        if unit is None:
            errors.append(f"lifetime ledger line {line_number} names unknown unit {unit_id}")
        else:
            rows_by_unit[unit_id].append(row)
            unit_sources = {str(source) for source in unit.get("sources", [])}
            if upstream_path not in unit_sources:
                errors.append(
                    f"lifetime ledger line {line_number} source is not owned by unit {unit_id}: {upstream_path}"
                )
        if upstream_path not in ore_sources:
            errors.append(
                f"lifetime ledger line {line_number} source is not in the ORE manifest: {upstream_path}"
            )
        if not field:
            errors.append(f"lifetime ledger line {line_number} has an empty field")
        for column in (
            "cpp_ownership",
            "rust_shape",
            "threading",
            "concrete_native_downcast_seam",
            "release_invariant",
            "failure_invariant",
        ):
            if not row[column].strip():
                errors.append(
                    f"lifetime ledger line {line_number} has an empty {column}"
                )
        if status not in LIFETIME_STATUSES:
            errors.append(
                f"lifetime ledger line {line_number} has invalid status `{status}`"
            )
        evidence = [
            value.strip() for value in row["evidence"].split(";") if value.strip()
        ]
        if status in {"prepared", "verified"} and not evidence:
            errors.append(
                f"lifetime ledger line {line_number} is {status} without evidence"
            )
        for citation in evidence:
            validate_evidence_citation(citation, repo_root, upstream_root, errors)
            head, _, _ = citation.rpartition(":")
            root_kind, separator, cited_path = head.partition(":")
            if (
                separator
                and root_kind == "rust"
                and not git_tracked_file(repo_root, cited_path)
            ):
                errors.append(
                    f"lifetime ledger line {line_number} cites untracked Rust evidence {cited_path}"
                )

    duplicates = duplicate_values(ledger_keys)
    if duplicates:
        errors.append("duplicate lifetime ledger rows: " + ", ".join(duplicates))
    for unit in units:
        unit_id = str(unit.get("id", ""))
        unit_rows = rows_by_unit.get(unit_id, [])
        if not unit_rows:
            errors.append(f"translation unit {unit_id} has no lifetime rows")
            continue
        covered_sources = {row["upstream_path"] for row in unit_rows}
        missing_sources = sorted(
            {str(source) for source in unit.get("sources", [])} - covered_sources
        )
        if missing_sources:
            errors.append(
                f"translation unit {unit_id} has sources without lifetime rows: "
                + ", ".join(missing_sources)
            )
        if unit.get("status") != "pending":
            unprepared = [
                row["field"]
                for row in unit_rows
                if row["status"] not in {"prepared", "verified"}
            ]
            if unprepared:
                errors.append(
                    f"translation unit {unit_id} advanced before lifetime preparation: "
                    + ", ".join(unprepared)
                )
    return rows


def validate_owner_rows(
    ownership: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> tuple[list[dict[str, Any]], collections.Counter[str]]:
    owners = list(ownership.get("owner", []))
    owner_ids = [str(row.get("id", "")) for row in owners]
    duplicates = duplicate_values(owner_ids)
    if duplicates:
        errors.append(f"duplicate ownership rows: {', '.join(duplicates)}")
    counts: collections.Counter[str] = collections.Counter()
    for row in owners:
        owner_id = str(row.get("id", ""))
        status = str(row.get("status", ""))
        issue = str(row.get("issue", ""))
        tests = [str(value) for value in row.get("required_tests", [])]
        citations = [str(value) for value in row.get("citations", [])]
        evidence_paths = [str(value) for value in row.get("evidence_paths", [])]
        if not owner_id:
            errors.append("ownership row has an empty id")
        if status not in OWNER_STATUSES:
            errors.append(f"ownership row {owner_id} has invalid status `{status}`")
        else:
            counts[status] += 1
        if not re.fullmatch(r"UNIV-\d+", issue):
            errors.append(f"ownership row {owner_id} has invalid issue `{issue}`")
        if not tests:
            errors.append(f"ownership row {owner_id} has no required tests")
        if not citations:
            errors.append(f"ownership row {owner_id} has no upstream citations")
        for citation in citations:
            validate_citation(citation, repo_root, upstream_root, errors)
        if status in VERIFIED_STATUSES:
            if not evidence_paths:
                errors.append(
                    f"ownership row {owner_id} is {status} without concrete evidence paths"
                )
            for relative in evidence_paths:
                if not (repo_root / relative).is_file():
                    errors.append(
                        f"ownership row {owner_id} names missing evidence path {relative}"
                    )
                elif not git_tracked_file(repo_root, relative):
                    errors.append(
                        f"ownership row {owner_id} names untracked evidence path {relative}"
                    )
    return owners, counts


def check(
    *,
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    manifest_path: pathlib.Path,
    ownership_path: pathlib.Path,
) -> str:
    manifest = read_toml(manifest_path)
    ownership = read_toml(ownership_path)
    errors: list[str] = []

    if manifest.get("version") != 1:
        errors.append("Metal source manifest version must be 1")
    if ownership.get("version") != 1:
        errors.append("Metal ownership inventory version must be 1")
    upstream_ref = str(manifest.get("upstream_ref", ""))
    if not re.fullmatch(r"[0-9a-f]{40}", upstream_ref):
        errors.append("Metal source manifest upstream_ref must be a full 40-hex SHA")
    else:
        actual_ref = git_head(upstream_root)
        if actual_ref != upstream_ref:
            errors.append(
                f"upstream checkout is {actual_ref}; Metal source manifest pins {upstream_ref}"
            )
    if ownership.get("upstream_ref") != upstream_ref:
        errors.append("Metal source manifest and ownership inventory pin different refs")

    guide = repo_root / str(manifest.get("porting_guide", ""))
    if not guide.is_file():
        errors.append(f"Metal porting guide does not exist: {guide}")

    source_counts = validate_source_rows(manifest, repo_root, upstream_root, errors)
    validate_render_context_file_map(manifest, repo_root, upstream_root, errors)
    validate_render_context_field_map(manifest, repo_root, upstream_root, errors)
    validate_render_context_configuration_map(
        manifest, repo_root, upstream_root, errors
    )
    validate_translation_conventions(manifest, repo_root, upstream_root, errors)
    units = validate_translation_units(manifest, errors)
    validate_lifetime_ledger(manifest, repo_root, upstream_root, errors)
    validate_reference_provenance(manifest, repo_root, errors)
    expected_counts = {
        str(key): int(value)
        for key, value in dict(manifest.get("expected_status_counts", {})).items()
    }
    if dict(source_counts) != {key: value for key, value in expected_counts.items() if value}:
        errors.append(
            f"source status counts drifted: expected {expected_counts}, got {dict(source_counts)}"
        )

    owners, owner_counts = validate_owner_rows(
        ownership, repo_root, upstream_root, errors
    )
    expected_owner_counts = {
        str(key): int(value)
        for key, value in dict(ownership.get("expected_status_counts", {})).items()
    }
    if dict(owner_counts) != {
        key: value for key, value in expected_owner_counts.items() if value
    }:
        errors.append(
            "ownership status counts drifted: "
            f"expected {expected_owner_counts}, got {dict(owner_counts)}"
        )

    if errors:
        raise CheckFailure("\n".join(f"- {error}" for error in errors))
    return (
        "Metal port campaign check passed: "
        f"sources={sum(source_counts.values())} "
        f"pending={source_counts['pending']} "
        f"in-progress={source_counts['in-progress']} "
        f"ported={source_counts['ported']} "
        f"verified={source_counts['verified']} owners={len(owners)} "
        f"translation-units={len(units)}"
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument("--upstream-root", type=pathlib.Path, required=True)
    parser.add_argument("--manifest", type=pathlib.Path, required=True)
    parser.add_argument("--ownership", type=pathlib.Path, required=True)
    args = parser.parse_args()
    try:
        print(
            check(
                repo_root=args.repo_root.resolve(),
                upstream_root=args.upstream_root.resolve(),
                manifest_path=args.manifest.resolve(),
                ownership_path=args.ownership.resolve(),
            )
        )
    except CheckFailure as error:
        print(f"Metal port campaign check failed:\n{error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
