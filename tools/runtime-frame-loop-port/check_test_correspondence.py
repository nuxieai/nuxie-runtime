#!/usr/bin/env python3
"""Validate the pinned upstream unit-test correspondence manifest."""

from __future__ import annotations

import argparse
import collections
import dataclasses
import json
import os
import pathlib
import re
import subprocess
import sys
import tomllib
from typing import Any


STATUSES = (
    "ported-differential",
    "ported-direct",
    "partial",
    "pending",
    "n-a",
)
CASE_STATUSES = ("pending", "direct", "differential", "adapted")
CASE_OUTCOMES = ("unverified", "pass", "expected-red", "not-applicable")
CASE_EVIDENCE_KINDS = ("rust-test", "live-differential")
CASE_ADAPTATION_KINDS = (
    "cxx-language-only",
    "rust-safety",
    "taffy",
    "native-audio",
    "native-scripting",
)
SOURCE_GLOBS = (
    "tests/unit_tests/runtime/*.cpp",
    "tests/unit_tests/runtime/scripting/*.cpp",
)
TEST_CASE_START_RE = re.compile(r"\bTEST_CASE\s*\(")


class CheckFailure(Exception):
    """Raised when the manifest is stale or internally inconsistent."""


@dataclasses.dataclass(frozen=True)
class CheckSummary:
    files: int
    test_cases: int
    status_counts: dict[str, int]
    case_status_counts: dict[str, int]
    case_outcome_counts: dict[str, int]


@dataclasses.dataclass(frozen=True)
class UpstreamCase:
    upstream: str
    ordinal: int
    line: int
    name: str

    @property
    def case_id(self) -> str:
        return f"{self.upstream}#{self.ordinal}"


def strip_cpp_comments(source: str) -> str:
    """Blank C++ comments while preserving strings, offsets, and newlines."""

    output = list(source)
    index = 0
    state = "code"
    while index < len(source):
        if state == "code":
            if source.startswith("//", index):
                output[index : index + 2] = "  "
                index += 2
                state = "line-comment"
            elif source.startswith("/*", index):
                output[index : index + 2] = "  "
                index += 2
                state = "block-comment"
            elif source[index] == '"':
                index += 1
                state = "string"
            elif source[index] == "'":
                index += 1
                state = "character"
            else:
                index += 1
        elif state == "line-comment":
            if source[index] == "\n":
                state = "code"
            else:
                output[index] = " "
            index += 1
        elif state == "block-comment":
            if source.startswith("*/", index):
                output[index : index + 2] = "  "
                index += 2
                state = "code"
            else:
                if source[index] != "\n":
                    output[index] = " "
                index += 1
        else:
            quote = '"' if state == "string" else "'"
            if source[index] == "\\":
                index = min(index + 2, len(source))
            elif source[index] == quote:
                index += 1
                state = "code"
            else:
                index += 1
    return "".join(output)


def test_case_entries(source: str) -> list[tuple[int, str]]:
    """Return (line, name) for active Catch2 TEST_CASE declarations."""

    uncommented = strip_cpp_comments(source)
    entries: list[tuple[int, str]] = []
    for match in TEST_CASE_START_RE.finditer(uncommented):
        cursor = match.end()
        fragments: list[str] = []
        while True:
            while cursor < len(uncommented) and uncommented[cursor].isspace():
                cursor += 1
            if cursor >= len(uncommented) or uncommented[cursor] != '"':
                break
            cursor += 1
            start = cursor
            fragment: list[str] = []
            while cursor < len(uncommented):
                if uncommented[cursor] == "\\" and cursor + 1 < len(uncommented):
                    fragment.append(uncommented[start:cursor])
                    fragment.append(uncommented[cursor : cursor + 2])
                    cursor += 2
                    start = cursor
                elif uncommented[cursor] == '"':
                    fragment.append(uncommented[start:cursor])
                    cursor += 1
                    break
                else:
                    cursor += 1
            fragments.append("".join(fragment))
        if not fragments:
            raise CheckFailure(
                "TEST_CASE at line "
                f"{uncommented.count(chr(10), 0, match.start()) + 1} "
                "does not start with a string literal name"
            )
        name = "".join(fragments).replace(r'\"', '"').replace(r"\\", "\\")
        entries.append((uncommented.count("\n", 0, match.start()) + 1, name))
    return entries


def test_case_names(source: str) -> list[str]:
    """Return active Catch2 TEST_CASE names in source order."""

    return [name for _, name in test_case_entries(source)]


def upstream_case_census(sources: dict[str, str]) -> list[UpstreamCase]:
    """Return the stable case denominator in path/source order."""

    return [
        UpstreamCase(path, ordinal, line, name)
        for path, source in sorted(sources.items())
        for ordinal, (line, name) in enumerate(test_case_entries(source), start=1)
    ]


def load_json_object(path: pathlib.Path, label: str) -> dict[str, Any]:
    try:
        document = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise CheckFailure(f"cannot read {label} {path}: {error}") from error
    if not isinstance(document, dict):
        raise CheckFailure(f"{label} must be a JSON object")
    return document


def require_string(mapping: dict[str, Any], key: str, context: str) -> str:
    value = mapping.get(key)
    if not isinstance(value, str) or not value.strip():
        raise CheckFailure(f"{context}.{key} must be a non-empty string")
    return value


RUST_FUNCTION_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\b"
)
RUST_TEST_ATTRIBUTE_RE = re.compile(
    r"#\s*\[\s*(?:[A-Za-z_][A-Za-z0-9_]*::)*test(?:\s*\([^]]*\))?\s*\]"
)
RUST_IGNORE_ATTRIBUTE_RE = re.compile(
    r'''#\s*\[\s*ignore(?:\s*=\s*"([^"]*)")?\s*\]'''
)


@dataclasses.dataclass(frozen=True)
class RustTestLocator:
    ignored: bool
    ignore_reason: str | None


def resolve_rust_test(
    repo_root: pathlib.Path, evidence: dict[str, Any], context: str
) -> RustTestLocator:
    relative = require_string(evidence, "path", context)
    candidate = pathlib.PurePosixPath(relative)
    if candidate.is_absolute() or ".." in candidate.parts:
        raise CheckFailure(f"{context}.path must be a repo-relative path")
    if candidate.suffix != ".rs" or not candidate.parts or candidate.parts[0] == "docs":
        raise CheckFailure(f"{context} rust-test evidence must point to a Rust source file")
    source_path = repo_root / candidate
    if not source_path.is_file():
        raise CheckFailure(f"{context} Rust test path does not exist: {relative}")
    line = require_int(evidence, "line", context)
    if line < 1:
        raise CheckFailure(f"{context}.line must be at least 1")
    symbol = require_string(evidence, "symbol", context)
    expected_name = symbol.rsplit("::", 1)[-1]
    lines = source_path.read_text().splitlines()
    if line > len(lines):
        raise CheckFailure(f"{context} Rust test line {line} is past end of {relative}")
    match = RUST_FUNCTION_RE.match(lines[line - 1])
    if match is None or match.group(1) != expected_name:
        raise CheckFailure(
            f"{context} does not resolve {symbol} at {relative}:{line}"
        )
    attributes: list[str] = []
    cursor = line - 2
    while cursor >= 0:
        stripped = lines[cursor].strip()
        if not stripped:
            cursor -= 1
            continue
        if stripped.startswith("#["):
            attributes.append(stripped)
            cursor -= 1
            continue
        break
    attribute_text = "\n".join(reversed(attributes))
    if RUST_TEST_ATTRIBUTE_RE.search(attribute_text) is None:
        raise CheckFailure(f"{context} locator is not a discovered Rust test")
    ignored = RUST_IGNORE_ATTRIBUTE_RE.search(attribute_text)
    return RustTestLocator(
        ignored=ignored is not None,
        ignore_reason=ignored.group(1) if ignored is not None else None,
    )


def validate_live_differential(
    repo_root: pathlib.Path, evidence: dict[str, Any], context: str
) -> None:
    relative = require_string(evidence, "harness_path", context)
    candidate = pathlib.PurePosixPath(relative)
    if candidate.is_absolute() or ".." in candidate.parts:
        raise CheckFailure(f"{context}.harness_path must be repo-relative")
    if (
        candidate.suffix not in {".rs", ".py", ".sh"}
        or not candidate.parts
        or candidate.parts[0] == "docs"
    ):
        raise CheckFailure(
            f"{context} live-differential must point to an executable source harness"
        )
    if not (repo_root / candidate).is_file():
        raise CheckFailure(
            f"{context} differential harness does not exist: {relative}"
        )
    require_string(evidence, "differential_id", context)
    require_string(evidence, "cpp_entry", context)
    require_string(evidence, "rust_entry", context)
    command = evidence.get("command")
    if not isinstance(command, list) or not command or not all(
        isinstance(part, str) and part.strip() for part in command
    ):
        raise CheckFailure(f"{context}.command must be a non-empty argv string list")


def validate_case_evidence(
    repo_root: pathlib.Path,
    row: dict[str, Any],
    case: UpstreamCase,
) -> None:
    context = case.case_id
    status = row["status"]
    outcome = row["outcome"]
    evidence = row.get("evidence")
    if not isinstance(evidence, list):
        raise CheckFailure(f"{context}.evidence must be a list")
    if status == "pending":
        if outcome != "unverified" or evidence:
            raise CheckFailure(
                f"{context} pending case requires outcome unverified and no evidence"
            )
        if row.get("adaptation") is not None or row.get("note") is not None:
            raise CheckFailure(f"{context} pending case cannot claim adaptation or note")
        return

    note = row.get("note")
    if not isinstance(note, str) or not note.strip():
        raise CheckFailure(f"{context} proven/adapted case requires a non-empty note")
    adaptation = row.get("adaptation")
    if status == "adapted":
        if not isinstance(adaptation, dict):
            raise CheckFailure(f"{context} adapted case requires adaptation metadata")
        kind = adaptation.get("kind")
        if kind not in CASE_ADAPTATION_KINDS:
            raise CheckFailure(f"{context} has invalid adaptation kind {kind!r}")
        require_string(adaptation, "rationale", f"{context}.adaptation")
        require_string(
            adaptation, "inapplicable_observable", f"{context}.adaptation"
        )
        if outcome == "not-applicable":
            if kind != "cxx-language-only" or evidence:
                raise CheckFailure(
                    f"{context} not-applicable requires cxx-language-only adaptation "
                    "and no evidence"
                )
            return
    elif adaptation is not None:
        raise CheckFailure(f"{context} may only declare adaptation with status adapted")

    if outcome not in {"pass", "expected-red"}:
        raise CheckFailure(
            f"{context} {status} case requires outcome pass or expected-red"
        )
    if len(evidence) != 1 or not isinstance(evidence[0], dict):
        raise CheckFailure(f"{context} requires exactly one typed evidence locator")
    locator = evidence[0]
    kind = locator.get("kind")
    if kind not in CASE_EVIDENCE_KINDS:
        raise CheckFailure(f"{context} has invalid evidence kind {kind!r}")
    if status == "direct" and kind != "rust-test":
        raise CheckFailure(f"{context} direct case requires rust-test evidence")
    if status == "differential" and kind != "live-differential":
        raise CheckFailure(
            f"{context} differential case requires live-differential evidence"
        )
    if kind == "live-differential":
        if outcome == "expected-red":
            raise CheckFailure(
                f"{context} expected-red must resolve to an explicitly ignored Rust test"
            )
        validate_live_differential(repo_root, locator, f"{context}.evidence[0]")
        return
    rust_test = resolve_rust_test(repo_root, locator, f"{context}.evidence[0]")
    supporting = locator.get("supporting_rust_tests", [])
    if not isinstance(supporting, list) or not all(
        isinstance(item, dict) for item in supporting
    ):
        raise CheckFailure(
            f"{context}.evidence[0].supporting_rust_tests must be a list of Rust test locators"
        )
    for index, supporting_locator in enumerate(supporting):
        supporting_test = resolve_rust_test(
            repo_root,
            supporting_locator,
            f"{context}.evidence[0].supporting_rust_tests[{index}]",
        )
        if supporting_test.ignored:
            raise CheckFailure(
                f"{context} supporting Rust test must be an executable passing assertion body"
            )
    if outcome == "pass" and rust_test.ignored:
        raise CheckFailure(f"{context} pass evidence points to an ignored Rust test")
    if outcome == "expected-red":
        reason = row.get("expected_red_reason")
        if not isinstance(reason, str) or not reason.startswith("expected-red: "):
            raise CheckFailure(
                f"{context} expected-red requires expected_red_reason naming missing behavior"
            )
        if not rust_test.ignored:
            raise CheckFailure(f"{context} expected-red Rust test is not #[ignore]")
        if rust_test.ignore_reason != reason:
            raise CheckFailure(
                f"{context} expected-red reason does not match the Rust #[ignore] reason"
            )
    elif row.get("expected_red_reason") is not None:
        raise CheckFailure(f"{context} may only set expected_red_reason for expected-red")


def git_output(repo: pathlib.Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=repo,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode:
        raise CheckFailure(
            f"git {' '.join(args)} failed in {repo}: {result.stderr.strip()}"
        )
    return result.stdout


def pinned_sources(upstream: pathlib.Path, ref: str) -> dict[str, str]:
    paths = git_output(upstream, "ls-tree", "-r", "--name-only", ref).splitlines()
    selected = sorted(
        path
        for path in paths
        if (
            pathlib.PurePosixPath(path).match(SOURCE_GLOBS[0])
            or pathlib.PurePosixPath(path).match(SOURCE_GLOBS[1])
        )
    )
    return {
        path: git_output(upstream, "show", f"{ref}:{path}") for path in selected
    }


def require_int(mapping: dict[str, Any], key: str, context: str) -> int:
    value = mapping.get(key)
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise CheckFailure(f"{context}.{key} must be a non-negative integer")
    return value


STATUS_RANK = {
    "pending": 0,
    "partial": 1,
    "ported-direct": 2,
    "ported-differential": 2,
}


def historical_row_floors(
    repo_root: pathlib.Path, manifest_path: pathlib.Path
) -> dict[str, str] | None:
    """Return each row's highest-ranked historical status keyed by upstream path.

    Statuses may only move in the pending -> partial -> ported direction, so
    every row is ratcheted against the best status it ever held. n-a rows are
    excluded: zero-case support files and explicitly adjudicated C++-language
    tests have no Rust behavior-coverage rank. Walks --full-history so merge
    simplification cannot prune a discarded parent's promotions, and fails
    closed on shallow clones or unreadable history, where the floors would be
    understated.
    """

    try:
        relative = manifest_path.resolve().relative_to(repo_root.resolve()).as_posix()
    except ValueError:
        return None
    shallow = subprocess.run(
        ["git", "rev-parse", "--is-shallow-repository"],
        cwd=repo_root,
        check=False,
        capture_output=True,
        text=True,
    )
    if shallow.returncode == 0 and shallow.stdout.strip() == "true":
        raise CheckFailure(
            "repository clone is shallow, so the status ratchet cannot see the "
            "manifest's full history; run `git fetch --unshallow` first"
        )
    history = subprocess.run(
        ["git", "log", "--full-history", "--format=%H", "--", relative],
        cwd=repo_root,
        check=False,
        capture_output=True,
        text=True,
    )
    if history.returncode != 0:
        raise CheckFailure(
            f"cannot read {relative} history for the status ratchet: "
            f"{history.stderr.strip()}"
        )
    if not history.stdout.strip():
        return None
    floors: dict[str, str] = {}
    for revision in history.stdout.splitlines():
        baseline = git_output(repo_root, "show", f"{revision}:{relative}")
        try:
            previous = tomllib.loads(baseline)
        except tomllib.TOMLDecodeError as error:
            raise CheckFailure(
                f"tracked manifest {revision} is invalid: {error}"
            ) from error
        rows = previous.get("file")
        if not isinstance(rows, list):
            raise CheckFailure(f"tracked manifest {revision} has no [[file]] rows")
        for row in rows:
            if not isinstance(row, dict):
                continue
            path = row.get("upstream")
            status = row.get("status")
            if not isinstance(path, str) or status not in STATUS_RANK:
                continue
            best = floors.get(path)
            if best is None or STATUS_RANK[status] > STATUS_RANK[best]:
                floors[path] = status
    return floors


@dataclasses.dataclass(frozen=True)
class HistoricalCaseFloor:
    max_pending: int
    proven_ids: frozenset[str]


def historical_case_floor(
    repo_root: pathlib.Path, ledger_path: pathlib.Path
) -> HistoricalCaseFloor | None:
    """Return the lowest pending ratchet and cases ever promoted in history."""

    try:
        relative = ledger_path.resolve().relative_to(repo_root.resolve()).as_posix()
    except ValueError:
        return None
    history = subprocess.run(
        ["git", "log", "--full-history", "--format=%H", "--", relative],
        cwd=repo_root,
        check=False,
        capture_output=True,
        text=True,
    )
    if history.returncode != 0:
        raise CheckFailure(
            f"cannot read {relative} history for the case ratchet: "
            f"{history.stderr.strip()}"
        )
    if not history.stdout.strip():
        return None
    minimum: int | None = None
    proven: set[str] = set()
    for revision in history.stdout.splitlines():
        baseline = git_output(repo_root, "show", f"{revision}:{relative}")
        try:
            previous = json.loads(baseline)
        except json.JSONDecodeError as error:
            raise CheckFailure(f"tracked case ledger {revision} is invalid: {error}") from error
        if not isinstance(previous, dict):
            raise CheckFailure(f"tracked case ledger {revision} is not an object")
        ratchet = previous.get("ratchet")
        if not isinstance(ratchet, dict):
            raise CheckFailure(f"tracked case ledger {revision} has no ratchet")
        maximum = require_int(ratchet, "max_pending", f"tracked ledger {revision}.ratchet")
        minimum = maximum if minimum is None else min(minimum, maximum)
        cases = previous.get("cases")
        if not isinstance(cases, list):
            raise CheckFailure(f"tracked case ledger {revision} has no cases")
        for row in cases:
            if isinstance(row, dict) and row.get("status") in CASE_STATUSES[1:]:
                case_id = row.get("id")
                if isinstance(case_id, str):
                    proven.add(case_id)
    assert minimum is not None
    return HistoricalCaseFloor(minimum, frozenset(proven))


def check_case_ledger(
    repo_root: pathlib.Path,
    ledger_path: pathlib.Path,
    ref: str,
    expected_cases: list[UpstreamCase],
) -> tuple[dict[str, int], dict[str, int]]:
    ledger = load_json_object(ledger_path, "case ledger")
    if ledger.get("schema") != "nuxie-test-case-correspondence/v1":
        raise CheckFailure("unsupported test-case correspondence schema")
    if ledger.get("schema_version") != 1:
        raise CheckFailure("case ledger schema_version must be 1")
    if ledger.get("upstream_ref") != ref:
        raise CheckFailure("case ledger upstream_ref must match the file manifest pin")
    if tuple(ledger.get("source_globs", ())) != SOURCE_GLOBS:
        raise CheckFailure(f"case ledger source_globs must be {list(SOURCE_GLOBS)!r}")
    if tuple(ledger.get("status_values", ())) != CASE_STATUSES:
        raise CheckFailure(f"case ledger status_values must be {list(CASE_STATUSES)!r}")
    if tuple(ledger.get("outcome_values", ())) != CASE_OUTCOMES:
        raise CheckFailure(f"case ledger outcome_values must be {list(CASE_OUTCOMES)!r}")
    if tuple(ledger.get("evidence_kinds", ())) != CASE_EVIDENCE_KINDS:
        raise CheckFailure(
            f"case ledger evidence_kinds must be {list(CASE_EVIDENCE_KINDS)!r}"
        )
    if tuple(ledger.get("adaptation_kinds", ())) != CASE_ADAPTATION_KINDS:
        raise CheckFailure(
            f"case ledger adaptation_kinds must be {list(CASE_ADAPTATION_KINDS)!r}"
        )
    declared_count = require_int(ledger, "case_count", "case ledger")
    if declared_count != len(expected_cases):
        raise CheckFailure(
            f"case ledger case_count={declared_count}, pin has {len(expected_cases)}"
        )
    rows = ledger.get("cases")
    if not isinstance(rows, list) or not all(isinstance(row, dict) for row in rows):
        raise CheckFailure("case ledger cases must be a list of objects")
    if len(rows) != declared_count:
        raise CheckFailure(
            f"case ledger declares {declared_count} cases but contains {len(rows)} rows"
        )

    declared_keys: list[tuple[str, int]] = []
    for index, row in enumerate(rows):
        context = f"case ledger row {index + 1}"
        upstream = require_string(row, "upstream", context)
        ordinal = require_int(row, "ordinal", context)
        if ordinal < 1:
            raise CheckFailure(f"{context}.ordinal must be at least 1")
        declared_keys.append((upstream, ordinal))
    if declared_keys != sorted(declared_keys):
        raise CheckFailure("case ledger rows must be sorted by upstream path and ordinal")
    duplicates = [
        key for key, count in collections.Counter(declared_keys).items() if count > 1
    ]
    if duplicates:
        raise CheckFailure(f"duplicate case identity in case ledger: {duplicates[0]}")
    expected_keys = [(case.upstream, case.ordinal) for case in expected_cases]
    missing = sorted(set(expected_keys) - set(declared_keys))
    extra = sorted(set(declared_keys) - set(expected_keys))
    if missing or extra:
        raise CheckFailure(f"case census mismatch: missing={missing}, extra={extra}")

    status_counts: collections.Counter[str] = collections.Counter()
    outcome_counts: collections.Counter[str] = collections.Counter()
    current_by_id: dict[str, str] = {}
    for row, expected in zip(rows, expected_cases, strict=True):
        context = expected.case_id
        if row.get("id") != expected.case_id:
            raise CheckFailure(
                f"{context} has stale id {row.get('id')!r}; expected {expected.case_id!r}"
            )
        line = require_int(row, "line", context)
        if line != expected.line:
            raise CheckFailure(
                f"{context} has stale line {line}; pinned source line is {expected.line}"
            )
        name = row.get("name")
        if name != expected.name:
            raise CheckFailure(
                f"{context} has stale name {name!r}; pinned name is {expected.name!r}"
            )
        status = row.get("status")
        if status not in CASE_STATUSES:
            raise CheckFailure(f"{context} has invalid case status {status!r}")
        outcome = row.get("outcome")
        if outcome not in CASE_OUTCOMES:
            raise CheckFailure(f"{context} has invalid case outcome {outcome!r}")
        status_counts[status] += 1
        outcome_counts[outcome] += 1
        current_by_id[expected.case_id] = status
        validate_case_evidence(repo_root, row, expected)

    ratchet = ledger.get("ratchet")
    if not isinstance(ratchet, dict):
        raise CheckFailure("case ledger is missing ratchet")
    max_pending = require_int(ratchet, "max_pending", "case ledger.ratchet")
    if status_counts["pending"] > max_pending:
        raise CheckFailure(
            f"case pending count {status_counts['pending']} exceeds case ratchet {max_pending}"
        )
    floor = historical_case_floor(repo_root, ledger_path)
    if floor is not None:
        if max_pending > floor.max_pending:
            raise CheckFailure(
                f"case max_pending {max_pending} regressed from historical "
                f"{floor.max_pending}; the case ratchet may only decrease"
            )
        for case_id in sorted(floor.proven_ids):
            if current_by_id.get(case_id) == "pending":
                raise CheckFailure(
                    f"{case_id} regressed from historical case proof to pending"
                )
    return (
        {status: status_counts[status] for status in CASE_STATUSES},
        {outcome: outcome_counts[outcome] for outcome in CASE_OUTCOMES},
    )


def check_manifest(
    repo_root: pathlib.Path,
    upstream: pathlib.Path,
    manifest_path: pathlib.Path,
) -> CheckSummary:
    try:
        with manifest_path.open("rb") as source:
            manifest = tomllib.load(source)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise CheckFailure(f"cannot read {manifest_path}: {error}") from error

    if manifest.get("schema") != "nuxie-test-correspondence/v1":
        raise CheckFailure("unsupported test-correspondence schema")
    if manifest.get("schema_version") != 1:
        raise CheckFailure("schema_version must be 1")
    if tuple(manifest.get("source_globs", ())) != SOURCE_GLOBS:
        raise CheckFailure(f"source_globs must be {list(SOURCE_GLOBS)!r}")
    if tuple(manifest.get("status_values", ())) != STATUSES:
        raise CheckFailure(f"status_values must be {list(STATUSES)!r}")

    ref = manifest.get("upstream_ref")
    if not isinstance(ref, str) or not re.fullmatch(r"[0-9a-f]{40}", ref):
        raise CheckFailure("upstream_ref must be a full lowercase commit SHA")
    actual_ref = git_output(upstream, "rev-parse", "HEAD").strip()
    if actual_ref != ref:
        raise CheckFailure(f"upstream pin mismatch: manifest={ref}, checkout={actual_ref}")

    sources = pinned_sources(upstream, ref)
    upstream_cases = {path: test_case_names(source) for path, source in sources.items()}
    case_census = upstream_case_census(sources)
    rows = manifest.get("file")
    if not isinstance(rows, list):
        raise CheckFailure("manifest must contain [[file]] rows")
    declared_paths = [row.get("upstream") for row in rows if isinstance(row, dict)]
    if len(declared_paths) != len(rows) or not all(
        isinstance(path, str) for path in declared_paths
    ):
        raise CheckFailure("every [[file]] row must have a string upstream path")
    if declared_paths != sorted(declared_paths):
        raise CheckFailure("[[file]] rows must be sorted by upstream path")
    if len(set(declared_paths)) != len(declared_paths):
        raise CheckFailure("duplicate upstream path in [[file]] rows")
    missing = sorted(set(sources) - set(declared_paths))
    extra = sorted(set(declared_paths) - set(sources))
    if missing or extra:
        raise CheckFailure(f"upstream census mismatch: missing={missing}, extra={extra}")

    declared_row_count = require_int(manifest, "row_count", "manifest")
    if declared_row_count != len(rows):
        raise CheckFailure(
            f"row_count={declared_row_count}, actual manifest rows={len(rows)}"
        )
    declared_case_count = require_int(manifest, "test_case_count", "manifest")
    actual_case_count = sum(map(len, upstream_cases.values()))
    if declared_case_count != actual_case_count:
        raise CheckFailure(
            f"test_case_count={declared_case_count}, pin has {actual_case_count}"
        )
    case_ledger_name = manifest.get("case_ledger")
    if not isinstance(case_ledger_name, str) or not case_ledger_name.strip():
        raise CheckFailure("manifest.case_ledger must name the machine case ledger")
    case_ledger_candidate = pathlib.PurePosixPath(case_ledger_name)
    if case_ledger_candidate.is_absolute() or ".." in case_ledger_candidate.parts:
        raise CheckFailure("manifest.case_ledger must be a repo-relative path")
    case_ledger_path = repo_root / case_ledger_candidate

    status_counts: collections.Counter[str] = collections.Counter()
    for row in rows:
        path = row["upstream"]
        actual_names = upstream_cases[path]
        declared_count = require_int(row, "test_case_count", path)
        if declared_count != len(actual_names):
            raise CheckFailure(
                f"{path} declares {declared_count} TEST_CASEs; pin has {len(actual_names)}"
            )
        status = row.get("status")
        if status not in STATUSES:
            raise CheckFailure(f"{path} has invalid status {status!r}")
        status_counts[status] += 1
        evidence = row.get("evidence")
        if not isinstance(evidence, list) or not all(
            isinstance(item, str) and item.strip() for item in evidence
        ):
            raise CheckFailure(f"{path}.evidence must be a list of non-empty strings")
        if status in {"ported-differential", "ported-direct", "partial"} and not evidence:
            raise CheckFailure(f"{path} status {status} requires evidence")
        note = row.get("note")
        if not isinstance(note, str) or not note.strip():
            raise CheckFailure(f"{path}.note must be a non-empty string")
        adaptation = row.get("adaptation")
        if status == "n-a":
            if declared_count == 0:
                if adaptation is not None:
                    raise CheckFailure(
                        f"{path} zero-case n-a row must not declare adaptation"
                    )
            elif adaptation != "cxx-language-only":
                raise CheckFailure(
                    f"{path} nonzero n-a row requires adaptation = "
                    '"cxx-language-only"'
                )
        else:
            if declared_count == 0:
                raise CheckFailure(f"{path} zero-case row must use n-a")
            if adaptation is not None:
                raise CheckFailure(f"{path} may only declare adaptation when status is n-a")

        covered = row.get("covered_test_cases")
        if status == "partial":
            if not isinstance(covered, list) or not covered or not all(
                isinstance(item, str) and item for item in covered
            ):
                raise CheckFailure(
                    f"{path} partial row requires non-empty covered_test_cases"
                )
            unknown = sorted(set(covered) - set(actual_names))
            if unknown:
                raise CheckFailure(f"{path} has unknown covered_test_cases: {unknown}")
            if len(set(covered)) != len(covered):
                raise CheckFailure(f"{path} has duplicate covered_test_cases")
            if len(covered) >= declared_count:
                raise CheckFailure(
                    f"{path} partial row must name a strict subset of TEST_CASEs"
                )
        elif covered is not None:
            raise CheckFailure(f"{path} may only set covered_test_cases when partial")

    if "expected_status_counts" in manifest:
        raise CheckFailure(
            "[expected_status_counts] was replaced by the [ratchet] floor and the "
            "recensused monotonic guards; delete the block"
        )

    ratchet = manifest.get("ratchet")
    if not isinstance(ratchet, dict):
        raise CheckFailure("missing [ratchet]")
    max_pending = require_int(ratchet, "max_pending", "ratchet")
    if status_counts["pending"] > max_pending:
        raise CheckFailure(
            f"pending count {status_counts['pending']} exceeds ratchet {max_pending}"
        )
    floors = historical_row_floors(repo_root, manifest_path)
    if floors is not None:
        for row in rows:
            path = row["upstream"]
            status = row["status"]
            floor = floors.get(path)
            if (
                floor is not None
                and status in STATUS_RANK
                and STATUS_RANK[status] < STATUS_RANK[floor]
            ):
                raise CheckFailure(
                    f"{path} status {status} regressed from historical {floor}; "
                    "rows may only move pending -> partial -> ported"
                )

    case_status_counts, case_outcome_counts = check_case_ledger(
        repo_root, case_ledger_path, ref, case_census
    )

    return CheckSummary(
        files=len(rows),
        test_cases=actual_case_count,
        status_counts={status: status_counts[status] for status in STATUSES},
        case_status_counts=case_status_counts,
        case_outcome_counts=case_outcome_counts,
    )


def parse_args(argv: list[str]) -> argparse.Namespace:
    default_root = pathlib.Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=pathlib.Path, default=default_root)
    parser.add_argument(
        "--rive-runtime-dir",
        type=pathlib.Path,
        default=pathlib.Path(
            os.environ.get("RIVE_RUNTIME_DIR", "/Users/levi/dev/oss/rive-runtime")
        ),
    )
    parser.add_argument("--manifest", type=pathlib.Path)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    manifest = args.manifest or args.repo_root / "test-correspondence-manifest.toml"
    try:
        summary = check_manifest(args.repo_root, args.rive_runtime_dir, manifest)
    except CheckFailure as error:
        print(f"test-correspondence check failed: {error}", file=sys.stderr)
        return 1
    file_counts = ", ".join(
        f"{status}={summary.status_counts[status]}" for status in STATUSES
    )
    case_counts = ", ".join(
        f"{status}={summary.case_status_counts[status]}" for status in CASE_STATUSES
    )
    outcome_counts = ", ".join(
        f"{outcome}={summary.case_outcome_counts[outcome]}"
        for outcome in CASE_OUTCOMES
    )
    print(
        "test-correspondence check passed: "
        f"files={summary.files}, TEST_CASEs={summary.test_cases}; "
        f"file classifications: {file_counts}; "
        f"case proof: {case_counts}; case outcomes: {outcome_counts}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
