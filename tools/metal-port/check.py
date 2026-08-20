#!/usr/bin/env python3
"""Fail-closed validator for the native Metal mechanical-port campaign."""

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


SOURCE_STATUSES = {"pending", "in-progress", "ported", "verified"}
OWNER_STATUSES = {"pending", "in-progress", "ported", "verified"}
VERIFIED_STATUSES = {"ported", "verified"}
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
        f"verified={source_counts['verified']} owners={len(owners)}"
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
