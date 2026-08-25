#!/usr/bin/env python3
"""Ratchet one primary Rust source owner per applicable upstream source row."""

from __future__ import annotations

import argparse
import collections
import pathlib
import re
import tomllib


SHARED_OWNER_MARKER = re.compile(r"\b(?:MR(?:-\d+)?|exception)\b", re.IGNORECASE)


def modules(row: dict[str, object]) -> list[str]:
    return [
        value.strip()
        for value in str(row.get("rust_module", "")).split(";")
        if value.strip()
    ]


def direct_primary_owner(row: dict[str, object]) -> str | None:
    upstream_stem = pathlib.PurePosixPath(str(row.get("upstream", ""))).stem
    return next(
        (
            module
            for module in modules(row)
            if pathlib.PurePosixPath(module).stem == upstream_stem
        ),
        None,
    )


def check(repo_root: pathlib.Path, manifest_path: pathlib.Path) -> tuple[list[str], dict[str, int]]:
    manifest = tomllib.loads(manifest_path.read_text())
    rows = list(manifest.get("file", []))
    ratchet = manifest.get("source_correspondence_ratchet")
    errors: list[str] = []
    if not isinstance(ratchet, dict):
        return ["source_correspondence_ratchet table is missing"], {}

    explicit_values = ratchet.get("explicit_owner_exceptions", [])
    if not isinstance(explicit_values, list):
        return ["explicit_owner_exceptions must be an array"], {}
    explicit = [str(value) for value in explicit_values]
    duplicates = sorted(
        value
        for value, count in collections.Counter(explicit).items()
        if count > 1
    )
    if duplicates:
        errors.append("duplicate explicit owner exceptions: " + ", ".join(duplicates))
    explicit_set = set(explicit)

    seen_upstream: set[str] = set()
    primary_to_upstream: dict[str, str] = {}
    counts = {
        "applicable_rows": 0,
        "direct_primary_owner_rows": 0,
        "adjudicated_shared_owner_rows": 0,
        "explicit_owner_exception_rows": 0,
        "pending_rows": 0,
    }
    for row in rows:
        upstream = str(row.get("upstream", ""))
        if not upstream:
            errors.append("source correspondence row has an empty upstream path")
            continue
        if upstream in seen_upstream:
            errors.append(f"duplicate upstream source row: {upstream}")
            continue
        seen_upstream.add(upstream)

        row_modules = modules(row)
        for module in row_modules:
            if not (repo_root / module).is_file():
                errors.append(f"declared Rust owner does not exist: {upstream} -> {module}")

        if str(row.get("status", "")) == "pending":
            counts["pending_rows"] += 1
            if row_modules:
                errors.append(f"pending row declares a Rust owner: {upstream}")
            if upstream in explicit_set:
                errors.append(f"pending row cannot be an owner exception: {upstream}")
            continue

        counts["applicable_rows"] += 1
        primary = direct_primary_owner(row)
        if primary is not None:
            counts["direct_primary_owner_rows"] += 1
            previous = primary_to_upstream.get(primary)
            if previous is not None:
                errors.append(
                    f"primary Rust owner is not bijective: {primary} owns {previous} and {upstream}"
                )
            primary_to_upstream[primary] = upstream
            if upstream in explicit_set:
                errors.append(f"direct owner is redundantly excepted: {upstream}")
            continue

        if upstream in explicit_set:
            counts["explicit_owner_exception_rows"] += 1
            continue
        if SHARED_OWNER_MARKER.search(str(row.get("note", ""))) is not None:
            counts["adjudicated_shared_owner_rows"] += 1
            continue
        errors.append(f"applicable row has no direct primary owner or adjudication: {upstream}")

    unknown_exceptions = sorted(explicit_set - seen_upstream)
    if unknown_exceptions:
        errors.append("owner exceptions name unknown rows: " + ", ".join(unknown_exceptions))

    expected_keys = tuple(counts)
    for key in expected_keys:
        expected = ratchet.get(key)
        if expected != counts[key]:
            errors.append(
                f"source correspondence ratchet {key} expected {expected!r}, got {counts[key]}"
            )
    if len(rows) != counts["applicable_rows"] + counts["pending_rows"]:
        errors.append("source correspondence row census does not close")
    return errors, counts


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument("--manifest", type=pathlib.Path, required=True)
    args = parser.parse_args()
    errors, counts = check(args.repo_root, args.manifest)
    if errors:
        raise SystemExit(
            "Source correspondence check failed: "
            + "\nSource correspondence check failed: ".join(errors)
        )
    print(
        "Source correspondence: "
        f"{counts['applicable_rows']} applicable = "
        f"{counts['direct_primary_owner_rows']} direct primary + "
        f"{counts['adjudicated_shared_owner_rows']} shared adjudications + "
        f"{counts['explicit_owner_exception_rows']} explicit exceptions; "
        f"{counts['pending_rows']} pending absent rows"
    )


if __name__ == "__main__":
    main()
