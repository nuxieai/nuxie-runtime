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
