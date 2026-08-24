#!/usr/bin/env python3
"""Validate the pinned upstream unit-test correspondence manifest."""

from __future__ import annotations

import argparse
import collections
import dataclasses
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

    return CheckSummary(
        files=len(rows),
        test_cases=actual_case_count,
        status_counts={status: status_counts[status] for status in STATUSES},
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
    counts = ", ".join(
        f"{status}={summary.status_counts[status]}" for status in STATUSES
    )
    print(
        "test-correspondence check passed: "
        f"files={summary.files}, TEST_CASEs={summary.test_cases}, {counts}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
