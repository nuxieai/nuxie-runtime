#!/usr/bin/env python3
"""Freeze the current Rust-WGPU implementation that must be retired at cutover."""

from __future__ import annotations

import argparse
import csv
import sys
from pathlib import Path


HEADER = (
    "source_path",
    "source_sha256",
    "evidence_kind",
    "evidence_count",
    "cutover_disposition",
    "cutover_status",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    return parser.parse_args()


def validate_cutover(repo_root: Path, inventory: Path) -> int:
    if not inventory.is_file():
        raise ValueError(f"missing frozen legacy Rust-WGPU inventory: {inventory}")
    with inventory.open(newline="") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or ()) != HEADER:
            raise ValueError("legacy Rust-WGPU inventory header changed")
        rows = list(reader)
    if not rows:
        raise ValueError("legacy Rust-WGPU inventory is empty")

    failures: list[str] = []
    for row in rows:
        path = repo_root / row["source_path"]
        disposition = row["cutover_disposition"]
        if row["cutover_status"] != "complete":
            failures.append(f"{row['source_path']}: cutover is not complete")
        if disposition == "delete" and path.exists():
            failures.append(f"{row['source_path']}: legacy owner still exists")
        elif disposition != "delete" and not path.is_file():
            failures.append(f"{row['source_path']}: retained exact-source input is missing")
    if failures:
        raise ValueError("\n".join(failures))
    return len(rows)


def main() -> int:
    args = parse_args()
    output = args.output if args.output.is_absolute() else args.repo_root / args.output
    try:
        count = validate_cutover(args.repo_root, output)
    except ValueError as error:
        print(error, file=sys.stderr)
        return 1
    action = "checked" if args.check else "verified"
    print(f"{action} {count} frozen legacy Rust-WGPU cutover rows: complete")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
