#!/usr/bin/env python3
"""Fail-closed checker for the C++-corresponding runtime frame-loop port."""

from __future__ import annotations

import argparse
import collections
import fnmatch
import json
import pathlib
import re
import subprocess
import sys
import tomllib
from typing import Any, Iterable


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


class CheckFailure(Exception):
    """Raised when the frame-loop proof is incomplete or inconsistent."""


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
        if not re.fullmatch(r"(?:AF|RF)-\d+", rule):
            errors.append(f"{subject} is adapted without an AF/RF rule")
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
    if trace.get("schema") != "nuxie-runtime-frame-loop-trace/v1":
        errors.append("trace evidence schema is not v1")
    if trace.get("upstream_ref") != upstream_ref:
        errors.append("trace evidence pins a different upstream ref")
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
    operations = trace.get("golden_stream_operations", {})
    if operations.get("cpp") != operations.get("rust"):
        errors.append("trace evidence golden-stream work counts differ")
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

    ratchet_results: list[tuple[str, int, int]] = []
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
        for name, counts in trace.get("landmarks", {}).items()
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

    for row in gaps.get("ratchet", []):
        ratchet_id = str(row.get("id", ""))
        pattern_text = str(row.get("pattern", ""))
        globs = [str(value) for value in row.get("globs", [])]
        maximum = row.get("max_occurrences")
        if not ratchet_id or not pattern_text or not globs or not isinstance(maximum, int):
            errors.append(f"ratchet {ratchet_id!r} is incomplete")
            continue
        try:
            pattern = re.compile(pattern_text)
        except re.error as error:
            errors.append(f"ratchet {ratchet_id} has invalid regex: {error}")
            continue
        count = 0
        hits: list[str] = []
        for glob in globs:
            for path in sorted(repo_root.glob(glob)):
                if not path.is_file():
                    continue
                for line_number, line in enumerate(
                    path.read_text(encoding="utf-8", errors="replace").splitlines(),
                    start=1,
                ):
                    found = list(pattern.finditer(line))
                    count += len(found)
                    if found:
                        hits.append(f"{path.relative_to(repo_root)}:{line_number}")
        ratchet_results.append((ratchet_id, count, maximum))
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
        f"{ratchet_id}={count}/{maximum}"
        for ratchet_id, count, maximum in ratchet_results
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
