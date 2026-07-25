#!/usr/bin/env python3
"""Fail-closed checker for the Editor Next runtime-defect atlas."""

from __future__ import annotations

import argparse
import collections
import hashlib
import pathlib
import re
import subprocess
import sys
import tomllib
from typing import Any, Iterable


SCHEMA = "nuxie.editor-next.runtime-defect-atlas/v1"
CORRECTIONS_SCHEMA = "nuxie.editor-next.runtime-defect-corrections/v1"
FIXTURES_SCHEMA = "nuxie.editor-next.runtime-defect-fixtures/v1"
RUNTIME_IDS = {f"RT-ED-{value:03d}" for value in range(1, 8)}
LOCAL_IDS = {
    *(f"LOC-{value:03d}" for value in range(1, 10)),
    *(f"LOC-{value:03d}" for value in range(11, 20)),
}
EXPECTED_IDS = RUNTIME_IDS | LOCAL_IDS
EXPECTED_CORRECTION_IDS = {f"COR-{value:02d}" for value in range(1, 13)}
EXPECTED_CORRECTIONS_SHA256 = (
    "d5e3c41d43db53b925f4c01834deb73c51669e5df5fec1f3db7a28393aab83a7"
)
EXPECTED_CHILDREN = {
    "RT-ED-001": (set(), set(), set()),
    "RT-ED-002": (set(), set(), set()),
    "RT-ED-003": ({"P04-C01", "P19-C03"}, set(), set()),
    "RT-ED-004": (
        {"P04-C01", "P05-C01", "P10-C01", "P12-C01", "P15-C01"},
        set(),
        set(),
    ),
    "RT-ED-005": ({"P09-C01"}, set(), set()),
    "RT-ED-006": (set(), set(), set()),
    "RT-ED-007": ({"P19-C09"}, set(), set()),
    "LOC-001": (set(), {"P13-C07"}, set()),
    "LOC-002": (
        set(),
        {"P04-C11", "P09-C01", "P09-C03", "P09-C06"},
        set(),
    ),
    "LOC-003": (set(), set(), set()),
    "LOC-004": (set(), set(), set()),
    "LOC-005": (set(), {"P09-C05"}, set()),
    "LOC-006": (set(), {"P09-C04"}, set()),
    "LOC-007": (set(), {"P11-C12"}, set()),
    "LOC-008": (set(), {"P08-C06"}, set()),
    "LOC-009": (set(), {"P14-C01"}, set()),
    "LOC-011": (set(), {"P08-C06"}, set()),
    "LOC-012": (set(), {"P19-C08"}, set()),
    "LOC-013": (set(), {"P08-C08"}, set()),
    "LOC-014": (set(), {"P08-C09"}, set()),
    "LOC-015": (set(), {"P18-C01", "P18-C04", "P18-C05", "P18-C07"}, set()),
    "LOC-016": (set(), {"P18-C01", "P18-C04"}, set()),
    "LOC-017": (set(), {"P18-C07"}, set()),
    "LOC-018": (set(), {"P04-C12", "P07-C04"}, set()),
    "LOC-019": (set(), {"P14-C06"}, set()),
}
EXPECTED_LEASE = {
    "refreshed": "2026-07-24",
    "active_wave": "FL-A",
    "branch": "levi/fl-a",
    "reserved_files": {
        "crates/nuxie-graph/src/lib.rs",
        "crates/nuxie-runtime/src/artboard.rs",
        "crates/nuxie-runtime/src/artboard_data_bind.rs",
        "crates/nuxie-runtime/src/components.rs",
        "crates/nuxie-runtime/src/constraints.rs",
        "crates/nuxie-runtime/src/draw.rs",
        "crates/nuxie-runtime/src/focus.rs",
        "crates/nuxie-runtime/src/lib.rs",
        "crates/nuxie-runtime/src/objects.rs",
        "crates/nuxie-runtime/src/retained_data_bind.rs",
        "crates/nuxie-runtime/src/text.rs",
        "docs/runtime-frame-loop-gaps.toml",
    },
    "future_files": {
        "crates/nuxie-runtime/src/animation.rs",
        "crates/nuxie-runtime/src/state_machine.rs",
        "crates/nuxie-runtime/src/state_machine/**",
    },
    "shared_ledgers": {
        "docs/runtime-frame-loop-ownership.toml",
        "docs/runtime-frame-loop-status.md",
        "file-correspondence-manifest.toml",
    },
}
OWNER_CLASSES = {"runtime", "api", "renderer", "editor", "artifact"}
CLASSIFICATIONS = {
    "unqualified",
    "tracked-gap",
    "structural-mistranslation",
    "local-translation-defect",
    "api-surface-gap",
    "verification-gap",
    "editor-integration-defect",
    "upstream-drift",
    "additive-product-feature",
    "stale-oracle",
    "retracted",
}
STATES = {
    "reported",
    "reproduced",
    "qualified",
    "mapped",
    "executor-green",
    "orchestrator-verified",
    "handoff-ready",
    "editor-consumed",
    "user-decided",
    "stale-oracle",
    "retracted",
    "closed",
}
TRANSITIONS = {
    "reported": {"reproduced"},
    "reproduced": {"qualified", "user-decided", "stale-oracle", "retracted"},
    "qualified": {"mapped", "handoff-ready"},
    "mapped": {"executor-green"},
    "executor-green": {"orchestrator-verified"},
    "orchestrator-verified": {"handoff-ready"},
    "handoff-ready": {"editor-consumed"},
    "editor-consumed": {"closed"},
    "user-decided": {"closed"},
    "stale-oracle": {"closed"},
    "retracted": {"closed"},
    "closed": set(),
}
RESULT_STATUSES = {"pending", "pass", "fail", "not-applicable"}
FIXTURE_KINDS = {
    "artifact",
    "browser-renderer",
    "cpp-runtime",
    "editor-product",
    "historical",
    "rust-runtime",
    "three-layer",
}
FIXTURE_STATUSES = {"registered", "implemented", "qualified", "historical"}
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
CHILD_RE = re.compile(r"^P\d{2}-C\d{2}$")
TICKET_RE = re.compile(r"^F-ED-(?:00|0[1-9]|1[0-4])$")
MINIMUM_FLOORS = {
    "runtime_tests": 414,
    "nuxie_tests": 140,
    "cpp_probe_tests": 721,
    "golden_entries": 317,
    "golden_segments": 647,
    "scripted_entries": 317,
    "scripted_segments": 647,
    "renderer_pixels": 1468,
}
MAXIMUM_CEILINGS = {"maximum_sdk_bytes": 9_437_184}
KNOWN_FLOORS = set(MINIMUM_FLOORS) | set(MAXIMUM_CEILINGS)
ARTIFACT_HASH_KEYS = {"proposal", "runtime_defects", "parity_ledger"}
STIMULUS_ROOTS = {"repo", "rive", "editor"}
REVISION_KEYS = {
    "original_localization_rust_sha",
    "editor_last_consumed_runtime_sha",
    "investigation_head_sha",
    "merged_repair_sha",
    "consumed_runtime_sha",
    "consumed_superproject_sha",
}
EARLY_STATES = {"reported", "reproduced"}
NORMAL_PIPELINE_STATES = {
    "reported",
    "reproduced",
    "qualified",
    "mapped",
    "executor-green",
    "orchestrator-verified",
    "handoff-ready",
    "editor-consumed",
    "closed",
}
QUALIFIED_OR_LATER = NORMAL_PIPELINE_STATES - {"reported", "reproduced"}
IMPLEMENTED_FIXTURE_STATES = NORMAL_PIPELINE_STATES - {"reported"}
QUALIFIED_FIXTURE_STATES = QUALIFIED_OR_LATER


class CheckFailure(Exception):
    """Raised when the atlas is incomplete or inconsistent."""


def read_toml(path: pathlib.Path) -> dict[str, Any]:
    try:
        with path.open("rb") as source:
            return tomllib.load(source)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise CheckFailure(f"cannot read {path}: {error}") from error


def duplicate_values(values: Iterable[str]) -> list[str]:
    counts = collections.Counter(values)
    return sorted(value for value, count in counts.items() if count > 1)


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


def validate_result(
    defect_id: str,
    layer: str,
    result: Any,
    state: str,
    errors: list[str],
) -> None:
    if not isinstance(result, dict):
        errors.append(f"{defect_id} has no {layer}_result table")
        return
    status = str(result.get("status", ""))
    if status not in RESULT_STATUSES:
        errors.append(f"{defect_id} {layer}_result has invalid status {status!r}")
        return
    if status in {"pending", "not-applicable"}:
        if not str(result.get("reason", "")).strip():
            errors.append(f"{defect_id} {layer}_result {status} has no reason")
    else:
        if not str(result.get("command", "")).strip():
            errors.append(f"{defect_id} {layer}_result {status} has no command")
        if not str(result.get("evidence", "")).strip():
            errors.append(f"{defect_id} {layer}_result {status} has no evidence")
    if state not in {"reported", "reproduced"} and status == "pending":
        errors.append(
            f"{defect_id} is {state} but {layer}_result is still pending"
        )


def validate_history(
    defect_id: str, state: str, history: Any, errors: list[str]
) -> None:
    if not isinstance(history, list) or not history:
        errors.append(f"{defect_id} has no state history")
        return
    history_states = [str(row.get("state", "")) for row in history]
    if history_states[0] != "reported":
        errors.append(f"{defect_id} history must begin at reported")
    if history_states[-1] != state:
        errors.append(
            f"{defect_id} state {state!r} does not match history tail "
            f"{history_states[-1]!r}"
        )
    for row in history:
        row_state = str(row.get("state", ""))
        if row_state not in STATES:
            errors.append(f"{defect_id} history has invalid state {row_state!r}")
        if not str(row.get("actor", "")).strip():
            errors.append(f"{defect_id} history state {row_state} has no actor")
        if not str(row.get("evidence", "")).strip():
            errors.append(f"{defect_id} history state {row_state} has no evidence")
        if (
            row_state == "orchestrator-verified"
            and row.get("actor") != "independent-orchestrator"
        ):
            errors.append(
                f"{defect_id} orchestrator-verified was not promoted by "
                "independent-orchestrator"
            )
    for previous, current in zip(history_states, history_states[1:]):
        if current not in TRANSITIONS.get(previous, set()):
            errors.append(
                f"{defect_id} has illegal state transition {previous} -> {current}"
            )


def validate_children(
    defect_id: str,
    field: str,
    values: Any,
    errors: list[str],
) -> set[str]:
    if not isinstance(values, list):
        errors.append(f"{defect_id} {field} is not a list")
        return set()
    normalized = [str(value) for value in values]
    duplicates = duplicate_values(normalized)
    if duplicates:
        errors.append(
            f"{defect_id} {field} contains duplicates: {', '.join(duplicates)}"
        )
    for value in normalized:
        if CHILD_RE.fullmatch(value) is None:
            errors.append(f"{defect_id} {field} has invalid child {value!r}")
    return set(normalized)


def validate_corrections(
    corrections: dict[str, Any],
    upstream_ref: str,
    pin_content: bool,
    errors: list[str],
) -> int:
    if corrections.get("schema") != CORRECTIONS_SCHEMA:
        errors.append(f"corrections schema must be {CORRECTIONS_SCHEMA}")
    if corrections.get("version") != 1:
        errors.append("corrections version must be 1")
    if corrections.get("source_pin") != upstream_ref:
        errors.append("corrections and atlas pin different upstream refs")
    rows = list(corrections.get("correction", []))
    ids = [str(row.get("id", "")) for row in rows]
    duplicates = duplicate_values(ids)
    if duplicates:
        errors.append(f"duplicate correction ids: {', '.join(duplicates)}")
    actual_ids = set(ids)
    if actual_ids != EXPECTED_CORRECTION_IDS:
        missing = ", ".join(sorted(EXPECTED_CORRECTION_IDS - actual_ids)) or "none"
        extra = ", ".join(sorted(actual_ids - EXPECTED_CORRECTION_IDS)) or "none"
        errors.append(
            "correction ids must be exactly COR-01..COR-12; "
            f"missing: {missing}; extra: {extra}"
        )
    for row in rows:
        correction_id = str(row.get("id", ""))
        if not re.fullmatch(r"COR-\d{2}", correction_id):
            errors.append(f"invalid correction id {correction_id!r}")
        if row.get("status") not in {"open", "resolved", "versioned"}:
            errors.append(f"{correction_id} has invalid correction status")
        if not str(row.get("description", "")).strip():
            errors.append(f"{correction_id} has no description")
        if not str(row.get("resolution", "")).strip():
            errors.append(f"{correction_id} has no resolution")
    canonical_parts = [
        str(row.get(field, ""))
        for row in rows
        for field in ("id", "status", "description", "resolution")
    ]
    content_digest = hashlib.sha256(
        "\0".join(canonical_parts).encode("utf-8")
    ).hexdigest()
    if pin_content and content_digest != EXPECTED_CORRECTIONS_SHA256:
        errors.append(
            f"correction content digest is {content_digest}; "
            f"expected {EXPECTED_CORRECTIONS_SHA256}"
        )
    expected = corrections.get("expected_corrections")
    if expected != len(rows):
        errors.append(
            f"correction count ratchet says {expected}, actual is {len(rows)}"
        )
    return len(rows)


def fixture_digest(row: dict[str, Any]) -> str:
    parts = [
        str(row.get(field, ""))
        for field in ("id", "defect_id", "kind", "status", "driver")
    ]
    for artifact in row.get("stimulus_files", []):
        if isinstance(artifact, dict):
            parts.extend(
                str(artifact.get(field, ""))
                for field in ("root", "path", "sha256")
            )
    canonical = "\0".join(parts)
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def validate_stimulus_files(
    fixture_id: str,
    status: str,
    artifacts: Any,
    roots: dict[str, pathlib.Path | None],
    verify_files: bool,
    errors: list[str],
) -> None:
    if not isinstance(artifacts, list):
        errors.append(f"fixture {fixture_id} stimulus_files must be a list")
        return
    if status == "qualified" and not artifacts:
        errors.append(
            f"fixture {fixture_id} is qualified but has no hashed stimulus files"
        )
    seen: set[tuple[str, str]] = set()
    for artifact in artifacts:
        if not isinstance(artifact, dict):
            errors.append(f"fixture {fixture_id} has a non-table stimulus file")
            continue
        if set(artifact) != {"root", "path", "sha256"}:
            errors.append(
                f"fixture {fixture_id} stimulus file keys must be exactly "
                "root, path, sha256"
            )
            continue
        root_name = str(artifact.get("root", ""))
        relative_text = str(artifact.get("path", ""))
        expected_hash = str(artifact.get("sha256", ""))
        if root_name not in STIMULUS_ROOTS:
            errors.append(
                f"fixture {fixture_id} has unknown stimulus root {root_name!r}"
            )
            continue
        relative = pathlib.PurePosixPath(relative_text)
        if (
            not relative_text
            or relative.is_absolute()
            or ".." in relative.parts
        ):
            errors.append(
                f"fixture {fixture_id} has unsafe stimulus path {relative_text!r}"
            )
            continue
        key = (root_name, relative_text)
        if key in seen:
            errors.append(
                f"fixture {fixture_id} repeats stimulus file "
                f"{root_name}:{relative_text}"
            )
        seen.add(key)
        if SHA256_RE.fullmatch(expected_hash) is None:
            errors.append(
                f"fixture {fixture_id} stimulus {root_name}:{relative_text} "
                "has invalid sha256"
            )
            continue
        if not verify_files:
            continue
        root = roots.get(root_name)
        if root is None:
            errors.append(
                f"fixture {fixture_id} cannot resolve stimulus root {root_name}"
            )
            continue
        path = root / pathlib.Path(relative_text)
        if not path.is_file():
            errors.append(
                f"fixture {fixture_id} stimulus does not exist at {path}"
            )
            continue
        actual_hash = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual_hash != expected_hash:
            errors.append(
                f"fixture {fixture_id} stimulus {root_name}:{relative_text} "
                f"hash is {actual_hash}; registry records {expected_hash}"
            )


def validate_cpp_probe_provenance(
    cpp_probe: pathlib.Path,
    repo_root: pathlib.Path,
    upstream_ref: str,
    errors: list[str],
) -> None:
    stamp_path = pathlib.Path(f"{cpp_probe}.provenance")
    if not stamp_path.is_file():
        errors.append(f"C++ probe provenance stamp is missing at {stamp_path}")
        return
    fields: dict[str, str] = {}
    for line in stamp_path.read_text().splitlines():
        key, separator, value = line.partition("=")
        if not separator or not key or key in fields:
            errors.append(f"C++ probe provenance stamp has invalid line {line!r}")
            continue
        fields[key] = value
    required = {
        "upstream_ref",
        "compiler",
        "flags",
        "source",
        "source_sha256",
        "executable_sha256",
    }
    if set(fields) != required:
        errors.append(
            "C++ probe provenance keys must be exactly "
            + ", ".join(sorted(required))
        )
        return
    if fields["upstream_ref"] != upstream_ref:
        errors.append(
            f"C++ probe provenance pins {fields['upstream_ref']}; "
            f"atlas pins {upstream_ref}"
        )
    if fields["flags"] != "-std=c++20 -Wall -Wextra -Werror":
        errors.append("C++ probe provenance records unexpected compiler flags")
    expected_source = "tools/editor-next-runtime-defects/cpp_probe/registry.cpp"
    if fields["source"] != expected_source:
        errors.append(
            f"C++ probe provenance source is {fields['source']!r}; "
            f"expected {expected_source!r}"
        )
    source_path = repo_root / expected_source
    if source_path.is_file():
        actual_source_hash = hashlib.sha256(source_path.read_bytes()).hexdigest()
        if fields["source_sha256"] != actual_source_hash:
            errors.append(
                "C++ probe provenance source hash does not match registry.cpp"
            )
    else:
        errors.append(f"C++ probe source does not exist at {source_path}")
    actual_executable_hash = hashlib.sha256(cpp_probe.read_bytes()).hexdigest()
    if fields["executable_sha256"] != actual_executable_hash:
        errors.append(
            "C++ probe provenance executable hash does not match the executable"
        )


def run_cpp_probe(
    cpp_probe: pathlib.Path,
    repo_root: pathlib.Path,
    upstream_ref: str,
    verify_provenance: bool,
    errors: list[str],
) -> set[str]:
    if not cpp_probe.is_file():
        errors.append(f"C++ probe executable does not exist at {cpp_probe}")
        return set()
    if verify_provenance:
        validate_cpp_probe_provenance(
            cpp_probe,
            repo_root,
            upstream_ref,
            errors,
        )
    result = subprocess.run(
        [str(cpp_probe), "--list"],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        errors.append(
            f"C++ probe --list failed ({result.returncode}): "
            f"{result.stderr.strip()}"
        )
        return set()
    values = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    duplicates = duplicate_values(values)
    if duplicates:
        errors.append(
            f"C++ probe --list returned duplicates: {', '.join(duplicates)}"
        )
    return set(values)


def validate_fixtures(
    fixtures: dict[str, Any],
    upstream_ref: str,
    atlas_fixtures: dict[str, str],
    atlas_states: dict[str, str],
    cpp_probe: pathlib.Path | None,
    repo_root: pathlib.Path,
    stimulus_roots: dict[str, pathlib.Path | None],
    verify_stimulus_files: bool,
    errors: list[str],
) -> tuple[int, dict[str, dict[str, Any]]]:
    if fixtures.get("schema") != FIXTURES_SCHEMA:
        errors.append(f"fixtures schema must be {FIXTURES_SCHEMA}")
    if fixtures.get("version") != 1:
        errors.append("fixtures version must be 1")
    if fixtures.get("upstream_ref") != upstream_ref:
        errors.append("fixtures and atlas pin different upstream refs")

    rows = list(fixtures.get("fixture", []))
    ids = [str(row.get("id", "")) for row in rows]
    defect_ids = [str(row.get("defect_id", "")) for row in rows]
    duplicate_ids = duplicate_values(ids)
    if duplicate_ids:
        errors.append(f"duplicate fixture registry ids: {', '.join(duplicate_ids)}")
    duplicate_defects = duplicate_values(defect_ids)
    if duplicate_defects:
        errors.append(
            "duplicate fixture registry defect ids: "
            f"{', '.join(duplicate_defects)}"
        )

    registry_fixtures: dict[str, str] = {}
    fixture_rows: dict[str, dict[str, Any]] = {}
    expected_cpp_probe_ids: set[str] = set()
    for row in rows:
        fixture_id = str(row.get("id", ""))
        defect_id = str(row.get("defect_id", ""))
        if not fixture_id:
            errors.append("fixture registry row has an empty id")
        elif fixture_id not in registry_fixtures:
            registry_fixtures[fixture_id] = defect_id
            fixture_rows[fixture_id] = row
        if defect_id not in EXPECTED_IDS:
            errors.append(
                f"fixture {fixture_id or '<empty>'} has invalid defect_id "
                f"{defect_id!r}"
            )
        if row.get("kind") not in FIXTURE_KINDS:
            errors.append(f"fixture {fixture_id} has invalid kind")
        status = str(row.get("status", ""))
        if status not in FIXTURE_STATUSES:
            errors.append(f"fixture {fixture_id} has invalid status")
        driver = str(row.get("driver", "")).strip()
        if not driver:
            errors.append(f"fixture {fixture_id} has no driver")
        if driver == "cpp_probe/registry.cpp":
            expected_cpp_probe_ids.add(fixture_id)
        if status in {"implemented", "qualified"} and (
            driver.startswith("pending:") or driver.startswith("evidence-only:")
        ):
            errors.append(
                f"fixture {fixture_id} is {status} but uses non-executable "
                f"driver {driver!r}"
            )
        validate_stimulus_files(
            fixture_id,
            status,
            row.get("stimulus_files", []),
            stimulus_roots,
            verify_stimulus_files,
            errors,
        )
        atlas_state = atlas_states.get(defect_id, "")
        if status == "implemented" and atlas_state not in IMPLEMENTED_FIXTURE_STATES:
            errors.append(
                f"fixture {fixture_id} is implemented but atlas row {defect_id} "
                f"is {atlas_state or '<missing>'}"
            )
        if status == "qualified" and atlas_state not in QUALIFIED_FIXTURE_STATES:
            errors.append(
                f"fixture {fixture_id} is qualified but atlas row {defect_id} "
                f"is {atlas_state or '<missing>'}"
            )

    registered_ids = set(registry_fixtures)
    atlas_ids = set(atlas_fixtures)
    missing = sorted(atlas_ids - registered_ids)
    extra = sorted(registered_ids - atlas_ids)
    if missing:
        errors.append(
            f"atlas fixture ids missing from registry: {', '.join(missing)}"
        )
    if extra:
        errors.append(
            f"fixture registry has ids absent from atlas: {', '.join(extra)}"
        )
    for fixture_id in sorted(atlas_ids & registered_ids):
        expected_defect = atlas_fixtures[fixture_id]
        actual_defect = registry_fixtures[fixture_id]
        if actual_defect != expected_defect:
            errors.append(
                f"fixture {fixture_id} maps to {actual_defect}; "
                f"atlas assigns it to {expected_defect}"
            )

    expected = fixtures.get("expected_fixtures")
    if expected != len(rows):
        errors.append(
            f"fixture count ratchet says {expected}, actual is {len(rows)}"
        )
    if cpp_probe is not None:
        actual_cpp_probe_ids = run_cpp_probe(
            cpp_probe,
            repo_root,
            upstream_ref,
            verify_stimulus_files,
            errors,
        )
        if actual_cpp_probe_ids != expected_cpp_probe_ids:
            missing = ", ".join(
                sorted(expected_cpp_probe_ids - actual_cpp_probe_ids)
            ) or "none"
            extra = ", ".join(
                sorted(actual_cpp_probe_ids - expected_cpp_probe_ids)
            ) or "none"
            errors.append(
                "C++ probe registry must exactly match fixtures driven by "
                "cpp_probe/registry.cpp; "
                f"missing: {missing}; extra: {extra}"
            )
    return len(rows), fixture_rows


def validate_pending_value(
    defect_id: str,
    field: str,
    value: Any,
    state: str,
    errors: list[str],
    *,
    pending_allowed_late: bool = False,
) -> None:
    if isinstance(value, str):
        normalized = value.strip()
        pending = normalized.lower().startswith("pending")
        if not normalized:
            errors.append(f"{defect_id} closure field {field} is empty")
        elif pending:
            pending_reason = normalized.partition(":")[2].strip()
            if not normalized.lower().startswith("pending:") or not pending_reason:
                errors.append(
                    f"{defect_id} closure field {field} has no pending reason"
                )
            elif state not in EARLY_STATES and not pending_allowed_late:
                errors.append(
                    f"{defect_id} is {state} but closure field {field} is pending"
                )
        return
    if isinstance(value, dict) and value.get("status") == "pending":
        if not str(value.get("reason", "")).strip():
            errors.append(f"{defect_id} closure field {field} has no pending reason")
        if state not in EARLY_STATES and not pending_allowed_late:
            errors.append(
                f"{defect_id} is {state} but closure field {field} is pending"
            )
        return
    if value is None:
        errors.append(f"{defect_id} has no closure field {field}")
    else:
        errors.append(f"{defect_id} closure field {field} must be a string")


def validate_revision(
    defect_id: str,
    field: str,
    value: Any,
    state: str,
    no_repair_path: bool,
    errors: list[str],
) -> None:
    pending_allowed_late = (
        field == "merged_repair_sha"
        and state
        not in {"orchestrator-verified", "handoff-ready", "editor-consumed", "closed"}
    ) or (
        field == "consumed_runtime_sha"
        and state not in {"editor-consumed", "closed"}
    ) or (field == "consumed_superproject_sha" and state != "closed")
    if no_repair_path and field in {
        "merged_repair_sha",
        "consumed_runtime_sha",
        "consumed_superproject_sha",
    }:
        pending_allowed_late = True
    if isinstance(value, str) and SHA_RE.fullmatch(value):
        return
    if isinstance(value, dict) and value.get("status") == "pending":
        validate_pending_value(
            defect_id,
            f"revisions.{field}",
            value,
            state,
            errors,
            pending_allowed_late=pending_allowed_late,
        )
        return
    errors.append(
        f"{defect_id} revisions.{field} must be a full SHA or pending with a reason"
    )


def validate_verification(
    defect_id: str,
    field: str,
    value: Any,
    state: str,
    errors: list[str],
) -> None:
    if not isinstance(value, dict):
        errors.append(f"{defect_id} {field} must be a table")
        return
    status = str(value.get("status", ""))
    if status == "pending":
        validate_pending_value(defect_id, field, value, state, errors)
    elif status == "pass":
        if not str(value.get("command", "")).strip():
            errors.append(f"{defect_id} {field} pass has no command")
        if not str(value.get("evidence", "")).strip():
            errors.append(f"{defect_id} {field} pass has no evidence")
        if field == "orchestrator_verification" and (
            value.get("actor") != "independent-orchestrator"
        ):
            errors.append(
                f"{defect_id} orchestrator_verification pass lacks independent actor"
            )
    elif status == "not-applicable":
        if not str(value.get("reason", "")).strip():
            errors.append(f"{defect_id} {field} not-applicable has no reason")
    else:
        errors.append(f"{defect_id} {field} has invalid status {status!r}")


def validate_closure_schema(
    row: dict[str, Any],
    fixture_row: dict[str, Any] | None,
    source_artifact_hashes: dict[str, str],
    errors: list[str],
) -> None:
    defect_id = str(row.get("id", ""))
    state = str(row.get("state", ""))
    for field in (
        "source_class",
        "preliminary_disposition",
        "rust_stimulus",
        "cpp_stimulus",
        "rust_owner",
        "displaced_mechanism",
        "owning_ledger",
        "adaptation_rule",
        "decision_row",
    ):
        validate_pending_value(defect_id, field, row.get(field), state, errors)

    hashes = row.get("artifact_hashes")
    if not isinstance(hashes, dict):
        errors.append(f"{defect_id} artifact_hashes must be a table")
    else:
        if set(hashes) != ARTIFACT_HASH_KEYS:
            errors.append(
                f"{defect_id} artifact_hashes keys must be exactly "
                f"{', '.join(sorted(ARTIFACT_HASH_KEYS))}"
            )
        for field in ARTIFACT_HASH_KEYS:
            if SHA256_RE.fullmatch(str(hashes.get(field, ""))) is None:
                errors.append(
                    f"{defect_id} artifact_hashes.{field} must be a SHA256"
                )
            elif hashes.get(field) != source_artifact_hashes.get(field):
                errors.append(
                    f"{defect_id} artifact_hashes.{field} does not match "
                    "the pinned source artifact"
                )

    revisions = row.get("revisions")
    if not isinstance(revisions, dict):
        errors.append(f"{defect_id} revisions must be a table")
    else:
        if set(revisions) != REVISION_KEYS:
            errors.append(
                f"{defect_id} revisions keys must be exactly "
                f"{', '.join(sorted(REVISION_KEYS))}"
            )
        no_repair_path = any(
            history.get("state") in {"stale-oracle", "retracted"}
            for history in row.get("history", [])
            if isinstance(history, dict)
        ) or row.get("classification") == "additive-product-feature"
        for field in REVISION_KEYS:
            validate_revision(
                defect_id,
                field,
                revisions.get(field),
                state,
                no_repair_path,
                errors,
            )

    for field in ("source_files", "source_members", "lifecycle_phases"):
        values = row.get(field)
        if not isinstance(values, list) or not values:
            errors.append(f"{defect_id} {field} must be a nonempty list")
        elif any(not str(value).strip() for value in values):
            errors.append(f"{defect_id} {field} contains an empty value")
        else:
            for value in values:
                validate_pending_value(defect_id, field, value, state, errors)

    for field in ("dependencies", "target_tests"):
        values = row.get(field)
        if not isinstance(values, list):
            errors.append(f"{defect_id} {field} must be a list")
        elif any(not str(value).strip() for value in values):
            errors.append(f"{defect_id} {field} contains an empty value")

    renderer_row = row.get("owner_class") == "renderer" or (
        fixture_row is not None
        and fixture_row.get("kind") == "browser-renderer"
    )
    floors = row.get("required_floors")
    if not isinstance(floors, list) or not floors:
        errors.append(f"{defect_id} required_floors must be a nonempty list")
    else:
        unknown = sorted({str(value) for value in floors} - KNOWN_FLOORS)
        if unknown:
            errors.append(
                f"{defect_id} required_floors contains unknown floors: "
                f"{', '.join(unknown)}"
            )
        if (
            renderer_row
            and state in QUALIFIED_OR_LATER
            and "renderer_pixels" not in floors
        ):
            errors.append(
                f"{defect_id} is qualified renderer work but omits "
                "the renderer_pixels floor"
            )

    renderer = row.get("renderer_provenance")
    if not isinstance(renderer, dict):
        errors.append(f"{defect_id} renderer_provenance must be a table")
    else:
        status = str(renderer.get("status", ""))
        if (
            renderer_row
            and state in QUALIFIED_OR_LATER
            and status != "complete"
        ):
            errors.append(
                f"{defect_id} is qualified renderer work but renderer "
                "provenance is not complete"
            )
        if status in {"pending", "not-applicable"}:
            if not str(renderer.get("reason", "")).strip():
                errors.append(
                    f"{defect_id} renderer_provenance {status} has no reason"
                )
            if status == "pending" and state not in EARLY_STATES:
                errors.append(
                    f"{defect_id} is {state} but renderer_provenance is pending"
                )
        elif status == "complete":
            for field in (
                "backend",
                "dawn_revision",
                "mode",
                "feature_flags",
                "surface",
                "reference_executable",
                "reference_stamp_sha256",
                "command",
                "evidence",
            ):
                if not str(renderer.get(field, "")).strip():
                    errors.append(
                        f"{defect_id} renderer_provenance complete has no {field}"
                    )
            reference_stamp = str(renderer.get("reference_stamp_sha256", ""))
            if (
                reference_stamp
                and SHA256_RE.fullmatch(reference_stamp) is None
            ):
                errors.append(
                    f"{defect_id} renderer_provenance reference stamp "
                    "must be a SHA256"
                )
        else:
            errors.append(
                f"{defect_id} renderer_provenance has invalid status {status!r}"
            )

    validate_verification(
        defect_id, "executor_verification", row.get("executor_verification"), state, errors
    )
    validate_verification(
        defect_id,
        "orchestrator_verification",
        row.get("orchestrator_verification"),
        state,
        errors,
    )

    reproduction = str(row.get("reproduction_sha256", ""))
    if SHA256_RE.fullmatch(reproduction) is None:
        errors.append(f"{defect_id} reproduction_sha256 must be a SHA256")
    elif fixture_row is None:
        errors.append(f"{defect_id} cannot resolve fixture for reproduction hash")
    else:
        expected = fixture_digest(fixture_row)
        if reproduction != expected:
            errors.append(
                f"{defect_id} reproduction_sha256 is {reproduction}; "
                f"fixture digest is {expected}"
            )


def validate_artifacts(
    artifacts: list[dict[str, Any]],
    source_root: pathlib.Path | None,
    errors: list[str],
) -> None:
    ids = [str(row.get("id", "")) for row in artifacts]
    duplicates = duplicate_values(ids)
    if duplicates:
        errors.append(f"duplicate artifact ids: {', '.join(duplicates)}")
    if len(artifacts) != 3:
        errors.append(f"expected 3 source artifacts, found {len(artifacts)}")
    for row in artifacts:
        artifact_id = str(row.get("id", ""))
        relative = str(row.get("path", ""))
        digest = str(row.get("sha256", ""))
        if not artifact_id or not relative:
            errors.append("source artifact has an empty id or path")
            continue
        if SHA256_RE.fullmatch(digest) is None:
            errors.append(f"artifact {artifact_id} has invalid sha256")
            continue
        if source_root is None:
            continue
        path = source_root / relative
        if not path.is_file():
            errors.append(f"artifact {artifact_id} does not exist at {path}")
            continue
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != digest:
            errors.append(
                f"artifact {artifact_id} hash is {actual}, atlas records {digest}"
            )


def validate_floors(floors: Any, errors: list[str]) -> None:
    if not isinstance(floors, dict):
        errors.append("atlas has no floors table")
        return
    expected_fields = set(MINIMUM_FLOORS) | set(MAXIMUM_CEILINGS)
    extra_fields = sorted(set(floors) - expected_fields)
    if extra_fields:
        errors.append(f"unknown floor fields: {', '.join(extra_fields)}")
    for field, minimum in MINIMUM_FLOORS.items():
        actual = floors.get(field)
        if not isinstance(actual, int):
            errors.append(f"floor {field} must be an integer")
        elif actual < minimum:
            errors.append(f"floor {field} is {actual}; minimum is {minimum}")
    for field, maximum in MAXIMUM_CEILINGS.items():
        actual = floors.get(field)
        if not isinstance(actual, int):
            errors.append(f"ceiling {field} must be an integer")
        elif actual > maximum:
            errors.append(f"ceiling {field} is {actual}; maximum is {maximum}")


def check(
    *,
    repo_root: pathlib.Path,
    atlas_path: pathlib.Path,
    corrections_path: pathlib.Path,
    fixtures_path: pathlib.Path,
    source_root: pathlib.Path | None,
    expected_upstream_ref: str,
    rive_runtime_dir: pathlib.Path | None,
    cpp_probe: pathlib.Path | None,
    require_closed: bool,
    validate_source_snapshot_git: bool,
) -> str:
    atlas = read_toml(atlas_path)
    corrections = read_toml(corrections_path)
    fixtures = read_toml(fixtures_path)
    errors: list[str] = []

    if atlas.get("schema") != SCHEMA:
        errors.append(f"atlas schema must be {SCHEMA}")
    if atlas.get("version") != 1:
        errors.append("atlas version must be 1")
    upstream_ref = str(atlas.get("upstream_ref", ""))
    if SHA_RE.fullmatch(upstream_ref) is None:
        errors.append("atlas upstream_ref must be a full 40-hex SHA")
    if upstream_ref != expected_upstream_ref:
        errors.append(
            f"atlas pins {upstream_ref}; expected {expected_upstream_ref}"
        )
    if rive_runtime_dir is not None:
        actual = git_head(rive_runtime_dir)
        if actual != upstream_ref:
            errors.append(
                f"upstream checkout is {actual}; atlas pins {upstream_ref}"
            )
    for field in ("editor_consumed_runtime_ref", "investigation_base_ref"):
        if SHA_RE.fullmatch(str(atlas.get(field, ""))) is None:
            errors.append(f"atlas {field} must be a full 40-hex SHA")
    source_snapshot_status = str(atlas.get("source_snapshot_status", ""))
    source_snapshot_ref = str(atlas.get("source_snapshot_ref", ""))
    if source_snapshot_status == "landed":
        if SHA_RE.fullmatch(source_snapshot_ref) is None:
            errors.append("landed snapshot must have a full source_snapshot_ref")
        elif validate_source_snapshot_git and source_root is not None:
            actual_source_ref = git_head(source_root)
            if actual_source_ref != source_snapshot_ref:
                errors.append(
                    f"Editor source checkout is {actual_source_ref}; "
                    f"atlas pins {source_snapshot_ref}"
                )
    elif source_snapshot_status == "pending-editor-commit":
        if source_snapshot_ref:
            errors.append(
                "pending-editor-commit snapshot must have an empty "
                "source_snapshot_ref"
            )
    else:
        errors.append(
            "source_snapshot_status must be landed or pending-editor-commit"
        )
    declared_corrections = (
        repo_root / str(atlas.get("corrections_file", ""))
    ).resolve()
    if declared_corrections != corrections_path:
        errors.append(
            f"atlas corrections_file resolves to {declared_corrections}, "
            f"but checker received {corrections_path}"
        )
    declared_fixtures = (
        repo_root / str(atlas.get("fixtures_file", ""))
    ).resolve()
    if declared_fixtures != fixtures_path:
        errors.append(
            f"atlas fixtures_file resolves to {declared_fixtures}, "
            f"but checker received {fixtures_path}"
        )

    correction_count = validate_corrections(
        corrections,
        upstream_ref,
        validate_source_snapshot_git,
        errors,
    )
    artifact_rows = list(atlas.get("artifact", []))
    validate_artifacts(artifact_rows, source_root, errors)
    artifact_id_to_field = {
        "cutover-proposal": "proposal",
        "runtime-defects": "runtime_defects",
        "parity-ledger": "parity_ledger",
    }
    source_artifact_hashes = {
        artifact_id_to_field[str(row.get("id", ""))]: str(row.get("sha256", ""))
        for row in artifact_rows
        if str(row.get("id", "")) in artifact_id_to_field
    }
    if set(source_artifact_hashes) != ARTIFACT_HASH_KEYS:
        errors.append(
            "source artifact ids must be exactly cutover-proposal, "
            "runtime-defects, parity-ledger"
        )
    validate_floors(atlas.get("floors"), errors)

    reserved_ids = set(str(value) for value in atlas.get("reserved_ids", []))
    if reserved_ids != {"LOC-010"}:
        errors.append("reserved_ids must contain only LOC-010")

    lease = atlas.get("lease")
    if not isinstance(lease, dict):
        errors.append("atlas has no lease table")
        lease = {}
    for field in ("refreshed", "active_wave", "branch"):
        if lease.get(field) != EXPECTED_LEASE[field]:
            errors.append(
                f"lease {field} is {lease.get(field)!r}; "
                f"expected {EXPECTED_LEASE[field]!r}"
            )
    for field in ("reserved_files", "future_files", "shared_ledgers"):
        actual_paths = {str(value) for value in lease.get(field, [])}
        expected_paths = EXPECTED_LEASE[field]
        if actual_paths != expected_paths:
            missing = ", ".join(sorted(expected_paths - actual_paths)) or "none"
            extra = ", ".join(sorted(actual_paths - expected_paths)) or "none"
            errors.append(
                f"lease {field} differs from the pinned coordination contract; "
                f"missing: {missing}; extra: {extra}"
            )

    rows = list(atlas.get("defect", []))
    ids = [str(row.get("id", "")) for row in rows]
    duplicates = duplicate_values(ids)
    if duplicates:
        errors.append(f"duplicate defect ids: {', '.join(duplicates)}")
    actual_ids = set(ids)
    missing = sorted(EXPECTED_IDS - actual_ids)
    extra = sorted(actual_ids - EXPECTED_IDS)
    if missing:
        errors.append(f"atlas is missing defect ids: {', '.join(missing)}")
    if extra:
        errors.append(f"atlas has unexpected defect ids: {', '.join(extra)}")
    if atlas.get("expected_defects") != len(rows):
        errors.append(
            f"defect count ratchet says {atlas.get('expected_defects')}, "
            f"actual is {len(rows)}"
        )

    fixture_ids: list[str] = []
    atlas_fixtures: dict[str, str] = {}
    atlas_states = {
        str(row.get("id", "")): str(row.get("state", "")) for row in rows
    }
    formal_children: set[str] = set()
    candidate_children: set[str] = set()
    disputed_children: set[str] = set()
    state_counts: collections.Counter[str] = collections.Counter()
    reserved_paths = {
        str(value)
        for field in ("reserved_files", "future_files", "shared_ledgers")
        for value in lease.get(field, [])
    }

    for row in rows:
        defect_id = str(row.get("id", ""))
        state = str(row.get("state", ""))
        state_counts[state] += 1
        if state not in STATES:
            errors.append(f"{defect_id} has invalid state {state!r}")
        if row.get("owner_class") not in OWNER_CLASSES:
            errors.append(f"{defect_id} has invalid owner_class")
        if row.get("classification") not in CLASSIFICATIONS:
            errors.append(f"{defect_id} has invalid classification")
        ticket = str(row.get("ticket", ""))
        if TICKET_RE.fullmatch(ticket) is None:
            errors.append(f"{defect_id} has invalid ticket {ticket!r}")
        if not str(row.get("title", "")).strip():
            errors.append(f"{defect_id} has no title")

        fixture_id = str(row.get("fixture_id", ""))
        if not fixture_id:
            errors.append(f"{defect_id} has no fixture_id")
        elif fixture_id not in atlas_fixtures:
            atlas_fixtures[fixture_id] = defect_id
        fixture_ids.append(fixture_id)

        validate_history(defect_id, state, row.get("history"), errors)
        for layer in ("cpp", "rust", "editor"):
            validate_result(
                defect_id,
                layer,
                row.get(f"{layer}_result"),
                state,
                errors,
            )

        touch = {str(value) for value in row.get("touch", [])}
        declared_dont_touch = {
            str(value) for value in row.get("dont_touch", [])
        }
        dont_touch = (
            reserved_paths
            if declared_dont_touch == {"@active-fl-lease"}
            else declared_dont_touch
        )
        overlap = sorted(touch & dont_touch)
        if overlap:
            errors.append(
                f"{defect_id} TOUCH and DON'T TOUCH overlap: {', '.join(overlap)}"
            )
        if not reserved_paths.issubset(dont_touch):
            missing_locks = sorted(reserved_paths - dont_touch)
            errors.append(
                f"{defect_id} omits active lease locks: {', '.join(missing_locks)}"
            )

        row_formal = validate_children(
            defect_id, "formal_children", row.get("formal_children"), errors
        )
        row_candidate = validate_children(
            defect_id, "candidate_children", row.get("candidate_children"), errors
        )
        row_disputed = validate_children(
            defect_id, "disputed_children", row.get("disputed_children"), errors
        )
        formal_children |= row_formal
        candidate_children |= row_candidate
        disputed_children |= row_disputed
        expected_children = EXPECTED_CHILDREN.get(defect_id)
        if expected_children is not None and (
            row_formal,
            row_candidate,
            row_disputed,
        ) != expected_children:
            errors.append(
                f"{defect_id} child mapping differs from the pinned exact map"
            )

    fixture_duplicates = duplicate_values(fixture_ids)
    if fixture_duplicates:
        errors.append(f"duplicate fixture ids: {', '.join(fixture_duplicates)}")
    fixture_count, fixture_rows = validate_fixtures(
        fixtures,
        upstream_ref,
        atlas_fixtures,
        atlas_states,
        cpp_probe,
        repo_root,
        {
            "repo": repo_root,
            "rive": rive_runtime_dir,
            "editor": source_root.parent if source_root is not None else None,
        },
        validate_source_snapshot_git,
        errors,
    )
    for row in rows:
        fixture_id = str(row.get("fixture_id", ""))
        validate_closure_schema(
            row,
            fixture_rows.get(fixture_id),
            source_artifact_hashes,
            errors,
        )
    if atlas.get("expected_formal_children") != len(formal_children):
        errors.append(
            "formal-child count ratchet says "
            f"{atlas.get('expected_formal_children')}, actual is "
            f"{len(formal_children)}"
        )
    if atlas.get("expected_candidate_children") != len(candidate_children):
        errors.append(
            "candidate-child count ratchet says "
            f"{atlas.get('expected_candidate_children')}, actual is "
            f"{len(candidate_children)}"
        )
    union_children = formal_children | candidate_children
    if atlas.get("expected_union_children") != len(union_children):
        errors.append(
            "union-child count ratchet says "
            f"{atlas.get('expected_union_children')}, actual is "
            f"{len(union_children)}"
        )
    overlap_children = formal_children & candidate_children
    expected_overlap = {
        str(value) for value in atlas.get("expected_overlap_children", [])
    }
    if expected_overlap != overlap_children:
        expected_text = ", ".join(sorted(expected_overlap)) or "empty"
        actual_text = ", ".join(sorted(overlap_children)) or "empty"
        errors.append(
            f"child-overlap ratchet names {expected_text}, "
            f"actual overlap is {actual_text}"
        )
    if not disputed_children.issubset(formal_children | candidate_children):
        errors.append("disputed children must also be formal or candidate children")
    if require_closed:
        open_rows = sorted(
            str(row.get("id", ""))
            for row in rows
            if row.get("state") != "closed"
        )
        if open_rows:
            errors.append(f"rows remain open: {', '.join(open_rows)}")

    if errors:
        raise CheckFailure("\n".join(f"- {error}" for error in errors))
    counts = ",".join(f"{key}:{state_counts[key]}" for key in sorted(state_counts))
    return (
        f"editor-next-runtime-defects: defects={len(rows)} "
        f"corrections={correction_count} fixtures={fixture_count} "
        f"states={counts} "
        f"formal_children={len(formal_children)} "
        f"candidate_children={len(candidate_children)} "
        f"union_children={len(union_children)}"
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument("--atlas", type=pathlib.Path, required=True)
    parser.add_argument("--corrections", type=pathlib.Path, required=True)
    parser.add_argument("--fixtures", type=pathlib.Path, required=True)
    parser.add_argument("--source-root", type=pathlib.Path)
    parser.add_argument("--rive-runtime-dir", type=pathlib.Path)
    parser.add_argument("--cpp-probe", type=pathlib.Path)
    parser.add_argument(
        "--test-mode",
        action="store_true",
        help="permit omitted production provenance inputs for isolated unit fixtures",
    )
    parser.add_argument("--expected-upstream-ref", required=True)
    parser.add_argument("--require-closed", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    canonical_atlas = (
        args.repo_root.resolve()
        / "docs"
        / "editor-next-runtime-defect-atlas.toml"
    )
    if args.test_mode and args.atlas.resolve() == canonical_atlas:
        print(
            "editor-next-runtime-defect-check failed:\n"
            "- --test-mode cannot validate the repository atlas",
            file=sys.stderr,
        )
        return 2
    if not args.test_mode:
        missing = [
            flag
            for flag, value in (
                ("--source-root", args.source_root),
                ("--rive-runtime-dir", args.rive_runtime_dir),
                ("--cpp-probe", args.cpp_probe),
            )
            if value is None
        ]
        if missing:
            print(
                "editor-next-runtime-defect-check failed:\n"
                "- production mode requires provenance inputs: "
                + ", ".join(missing),
                file=sys.stderr,
            )
            return 2
    try:
        summary = check(
            repo_root=args.repo_root.resolve(),
            atlas_path=args.atlas.resolve(),
            corrections_path=args.corrections.resolve(),
            fixtures_path=args.fixtures.resolve(),
            source_root=args.source_root.resolve() if args.source_root else None,
            expected_upstream_ref=args.expected_upstream_ref,
            rive_runtime_dir=(
                args.rive_runtime_dir.resolve() if args.rive_runtime_dir else None
            ),
            cpp_probe=args.cpp_probe.resolve() if args.cpp_probe else None,
            require_closed=args.require_closed,
            validate_source_snapshot_git=not args.test_mode,
        )
    except CheckFailure as error:
        print(f"editor-next-runtime-defect-check failed:\n{error}", file=sys.stderr)
        return 1
    print(summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
