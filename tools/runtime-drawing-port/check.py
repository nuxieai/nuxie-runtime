#!/usr/bin/env python3
"""Fail-closed checker for the C++ runtime drawing ownership port.

The file-correspondence manifest is intentionally coarse: one C++ file may
contain several independently owned objects.  This checker makes the smaller
ownership/lifecycle rows executable and ratchets the old scene-cache surface
down to zero without pretending that the current pre-removal state is closed.
"""

from __future__ import annotations

import argparse
import collections
import pathlib
import re
import subprocess
import sys
import tomllib
from typing import Any, Iterable


STATUSES = {"exact", "adapted", "pending", "compensation"}
CLOSED_STATUSES = {"exact", "adapted"}
LIFECYCLE_PHASES = ("construct", "update", "draw", "clone_drop")
CITATION_RE = re.compile(r"^(cpp|rust):(.+):(\d+)(?:-(\d+))?$")


class CheckFailure(Exception):
    """Raised when the ownership proof is incomplete or internally invalid."""


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


def topological_order(
    batches: list[dict[str, Any]], errors: list[str]
) -> list[str]:
    ids = [str(batch.get("id", "")) for batch in batches]
    duplicates = duplicate_values(ids)
    if duplicates:
        errors.append(f"duplicate batch ids: {', '.join(duplicates)}")
    known = set(ids)
    dependencies: dict[str, set[str]] = {}
    sequence: dict[str, int] = {}
    for batch in batches:
        batch_id = str(batch.get("id", ""))
        if not batch_id:
            errors.append("batch has an empty id")
            continue
        deps = {str(value) for value in batch.get("depends_on", [])}
        missing = sorted(deps - known)
        if missing:
            errors.append(
                f"batch {batch_id} has unknown dependencies: {', '.join(missing)}"
            )
        dependencies[batch_id] = deps & known
        try:
            sequence[batch_id] = int(batch["sequence"])
        except (KeyError, TypeError, ValueError):
            errors.append(f"batch {batch_id} has no integer sequence")
    sequence_duplicates = duplicate_values(str(value) for value in sequence.values())
    if sequence_duplicates:
        errors.append(
            "duplicate batch sequence values: " + ", ".join(sequence_duplicates)
        )
    for batch_id, deps in dependencies.items():
        for dependency in deps:
            if sequence.get(dependency, 0) >= sequence.get(batch_id, 0):
                errors.append(
                    f"batch {batch_id} must follow dependency {dependency} in sequence"
                )

    incoming = {key: set(value) for key, value in dependencies.items()}
    order: list[str] = []
    ready = sorted(
        (key for key, value in incoming.items() if not value),
        key=lambda key: sequence.get(key, 0),
    )
    while ready:
        current = ready.pop(0)
        order.append(current)
        for key in incoming:
            if current in incoming[key]:
                incoming[key].remove(current)
                if not incoming[key] and key not in order and key not in ready:
                    ready.append(key)
                    ready.sort(key=lambda value: sequence.get(value, 0))
    if len(order) != len(incoming):
        cycle = sorted(key for key, value in incoming.items() if value)
        errors.append(f"batch dependency cycle: {', '.join(cycle)}")
    return order


def matching_files(repo_root: pathlib.Path, globs: list[str]) -> list[pathlib.Path]:
    files: set[pathlib.Path] = set()
    for pattern in globs:
        files.update(path for path in repo_root.glob(pattern) if path.is_file())
    return sorted(files)


def count_pattern(
    repo_root: pathlib.Path, globs: list[str], pattern: str
) -> tuple[int, list[str]]:
    try:
        regex = re.compile(pattern)
    except re.error as error:
        raise CheckFailure(f"invalid ratchet regex {pattern!r}: {error}") from error
    count = 0
    hits: list[str] = []
    for path in matching_files(repo_root, globs):
        relative = path.relative_to(repo_root)
        for line_number, line in enumerate(
            path.read_text(encoding="utf-8", errors="replace").splitlines(), start=1
        ):
            matches = list(regex.finditer(line))
            if matches:
                count += len(matches)
                hits.append(f"{relative}:{line_number}")
    return count, hits


def check(
    *,
    repo_root: pathlib.Path,
    rive_runtime_dir: pathlib.Path,
    ledger_path: pathlib.Path,
    gaps_path: pathlib.Path,
    require_closed: bool,
) -> str:
    ledger = read_toml(ledger_path)
    gaps = read_toml(gaps_path)
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
                f"upstream checkout is {actual}; ownership ledger pins {upstream_ref}"
            )
    if gaps.get("upstream_ref") != upstream_ref:
        errors.append("gap inventory and ownership ledger pin different upstream refs")

    porting_path = repo_root / str(
        ledger.get("porting_rules_file", "docs/PORTING.md")
    )
    try:
        porting_rules = porting_path.read_text(encoding="utf-8")
    except OSError as error:
        raise CheckFailure(f"cannot read porting rules {porting_path}: {error}") from error

    batches = list(ledger.get("batch", []))
    batch_order = topological_order(batches, errors)
    batch_ids = {str(batch.get("id", "")) for batch in batches}

    ratchets = list(ledger.get("ratchet", []))
    ratchet_ids = [str(row.get("id", "")) for row in ratchets]
    duplicates = duplicate_values(ratchet_ids)
    if duplicates:
        errors.append(f"duplicate ratchet ids: {', '.join(duplicates)}")
    known_ratchets = set(ratchet_ids)

    owners = list(ledger.get("owner", []))
    owner_ids = [str(row.get("id", "")) for row in owners]
    duplicates = duplicate_values(owner_ids)
    if duplicates:
        errors.append(f"duplicate owner ids: {', '.join(duplicates)}")
    known_owners = set(owner_ids)
    owner_batches = {
        str(row.get("id", "")): str(row.get("batch", "")) for row in owners
    }
    batch_sequences = {
        str(batch.get("id", "")): int(batch.get("sequence", 0))
        for batch in batches
        if isinstance(batch.get("sequence"), int)
    }
    status_counts: collections.Counter[str] = collections.Counter()

    for row in owners:
        owner_id = str(row.get("id", ""))
        if not owner_id:
            errors.append("owner row has an empty id")
            continue
        status = str(row.get("status", ""))
        if status not in STATUSES:
            errors.append(f"owner {owner_id} has invalid status {status!r}")
        else:
            status_counts[status] += 1
        batch = str(row.get("batch", ""))
        if batch not in batch_ids:
            errors.append(f"owner {owner_id} has unknown batch {batch!r}")
        rule = str(row.get("rule", ""))
        if not re.fullmatch(r"RF-\d+", rule):
            errors.append(f"owner {owner_id} has invalid rule {rule!r}")
        elif f"**{rule} " not in porting_rules:
            errors.append(f"owner {owner_id} cites missing PORTING.md rule {rule}")
        lifecycle = row.get("lifecycle")
        if not isinstance(lifecycle, dict):
            errors.append(f"owner {owner_id} has no lifecycle table")
            lifecycle = {}
        for phase in LIFECYCLE_PHASES:
            citations = lifecycle.get(phase, [])
            if not isinstance(citations, list) or not citations:
                errors.append(f"owner {owner_id} lifecycle {phase} is empty")
                continue
            for citation in citations:
                validate_citation(
                    str(citation), repo_root, rive_runtime_dir, errors
                )
        rust_file = repo_root / str(row.get("rust_file", ""))
        anchor = str(row.get("rust_anchor", ""))
        if not rust_file.is_file():
            errors.append(f"owner {owner_id} Rust file does not exist: {rust_file}")
        elif not anchor:
            errors.append(f"owner {owner_id} has an empty rust_anchor")
        elif anchor not in rust_file.read_text(encoding="utf-8", errors="replace"):
            errors.append(
                f"owner {owner_id} anchor {anchor!r} is absent from "
                f"{rust_file.relative_to(repo_root)}"
            )
        dependencies = {str(value) for value in row.get("depends_on", [])}
        missing_dependencies = sorted(dependencies - known_owners)
        if missing_dependencies:
            errors.append(
                f"owner {owner_id} has unknown owner dependencies: "
                + ", ".join(missing_dependencies)
            )
        for dependency in sorted(dependencies & known_owners):
            dependency_batch = owner_batches[dependency]
            if batch_sequences.get(dependency_batch, 0) > batch_sequences.get(batch, 0):
                errors.append(
                    f"owner {owner_id} in batch {batch} depends on later owner "
                    f"{dependency} in batch {dependency_batch}"
                )
        legacy = {str(value) for value in row.get("legacy_ratchets", [])}
        missing_legacy = sorted(legacy - known_ratchets)
        if missing_legacy:
            errors.append(
                f"owner {owner_id} has unknown legacy ratchets: "
                + ", ".join(missing_legacy)
            )
        if status in CLOSED_STATUSES and legacy:
            errors.append(
                f"owner {owner_id} is {status} but still names legacy ratchets"
            )

    expected = ledger.get("expected_status_counts", {})
    for status in sorted(STATUSES):
        try:
            expected_count = int(expected[status])
        except (KeyError, TypeError, ValueError):
            errors.append(f"expected_status_counts.{status} is missing or invalid")
            continue
        if status_counts[status] != expected_count:
            errors.append(
                f"owner status count {status}={status_counts[status]}, "
                f"expected {expected_count}"
            )

    ratchet_results: list[tuple[str, int, int]] = []
    for row in ratchets:
        ratchet_id = str(row.get("id", ""))
        globs = [str(value) for value in row.get("globs", [])]
        pattern = str(row.get("pattern", ""))
        try:
            maximum = int(row["max_occurrences"])
        except (KeyError, TypeError, ValueError):
            errors.append(f"ratchet {ratchet_id} has no integer max_occurrences")
            continue
        if not globs or not pattern:
            errors.append(f"ratchet {ratchet_id} must provide globs and pattern")
            continue
        try:
            count, hits = count_pattern(repo_root, globs, pattern)
        except CheckFailure as error:
            errors.append(str(error))
            continue
        ratchet_results.append((ratchet_id, count, maximum))
        if count > maximum:
            sample = ", ".join(hits[:8])
            errors.append(
                f"ratchet {ratchet_id} increased to {count} > {maximum}; "
                f"first hits: {sample}"
            )
        if require_closed and count != 0:
            errors.append(
                f"closed ownership required but ratchet {ratchet_id} has {count} hits"
            )

    gap_rows = list(gaps.get("gap", []))
    gap_ids = [str(row.get("id", "")) for row in gap_rows]
    duplicates = duplicate_values(gap_ids)
    if duplicates:
        errors.append(f"duplicate gap ids: {', '.join(duplicates)}")
    for row in gap_rows:
        gap_id = str(row.get("id", ""))
        rule = str(row.get("rule", ""))
        if not gap_id:
            errors.append("gap row has an empty id")
        if not re.fullmatch(r"RF-\d+", rule) or f"**{rule} " not in porting_rules:
            errors.append(f"gap {gap_id} cites missing PORTING.md rule {rule!r}")
        citations = row.get("citations", [])
        if not isinstance(citations, list) or not citations:
            errors.append(f"gap {gap_id} has no citations")
        else:
            for citation in citations:
                validate_citation(
                    str(citation), repo_root, rive_runtime_dir, errors
                )
        if not str(row.get("decision", "")).strip():
            errors.append(f"gap {gap_id} has no decision")
        if not str(row.get("closure_test", "")).strip():
            errors.append(f"gap {gap_id} has no closure_test")

    if require_closed:
        open_rows = sorted(
            str(row.get("id", ""))
            for row in owners
            if str(row.get("status", "")) not in CLOSED_STATUSES
        )
        if open_rows:
            errors.append(
                "closed ownership required but rows remain open: "
                + ", ".join(open_rows)
            )

    if errors:
        raise CheckFailure("\n".join(f"- {error}" for error in errors))

    ratchet_summary = ", ".join(
        f"{ratchet_id}={count}/{maximum}"
        for ratchet_id, count, maximum in ratchet_results
    )
    status_summary = ", ".join(
        f"{status}={status_counts[status]}" for status in sorted(STATUSES)
    )
    return (
        "runtime-drawing-port: "
        f"owners={len(owners)} ({status_summary}); "
        f"batches={' -> '.join(batch_order)}; "
        f"gaps={len(gap_rows)}; ratchets[{ratchet_summary}]"
    )


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser()
    result.add_argument("--repo-root", type=pathlib.Path, required=True)
    result.add_argument("--rive-runtime-dir", type=pathlib.Path, required=True)
    result.add_argument("--ledger", type=pathlib.Path, required=True)
    result.add_argument("--gaps", type=pathlib.Path, required=True)
    result.add_argument(
        "--require-closed",
        action="store_true",
        help="fail unless all ownership rows and legacy ratchets are closed",
    )
    return result


def main() -> int:
    args = parser().parse_args()
    try:
        summary = check(
            repo_root=args.repo_root.resolve(),
            rive_runtime_dir=args.rive_runtime_dir.resolve(),
            ledger_path=args.ledger.resolve(),
            gaps_path=args.gaps.resolve(),
            require_closed=args.require_closed,
        )
    except CheckFailure as error:
        print(f"runtime-drawing-port check failed:\n{error}", file=sys.stderr)
        return 1
    print(summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
