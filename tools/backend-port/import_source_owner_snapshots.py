#!/usr/bin/env python3
"""Import exact source snapshots for one frozen backend ownership unit."""

from __future__ import annotations

import argparse
import csv
import hashlib
import shutil
from pathlib import Path


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", required=True, type=Path)
    parser.add_argument("--upstream-root", required=True, type=Path)
    parser.add_argument("--ownership-unit", required=True)
    args = parser.parse_args()

    repo = args.repo_root.resolve()
    upstream = args.upstream_root.resolve()
    ledger_path = repo / "docs/backend-port-source-ownership.tsv"
    with ledger_path.open(newline="", encoding="utf-8") as ledger_file:
        owners = [
            row
            for row in csv.DictReader(ledger_file, delimiter="\t")
            if row["ownership_unit"] == args.ownership_unit
        ]
    if not owners:
        raise SystemExit(f"no frozen sources for {args.ownership_unit}")

    outputs: list[str] = []
    for owner in owners:
        source_path = owner["source_path"]
        data = (upstream / source_path).read_bytes()
        actual = sha256(data)
        if actual != owner["source_sha256"]:
            raise SystemExit(
                f"source hash mismatch for {source_path}: "
                f"expected {owner['source_sha256']}, got {actual}"
            )
        target_parent = (repo / owner["target_path"]).parent
        snapshot = target_parent / "source" / source_path.replace("/", "_")
        snapshot.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(upstream / source_path, snapshot)
        if snapshot.read_bytes() != data:
            raise SystemExit(f"byte copy mismatch for {source_path}")
        outputs.append(str(snapshot.relative_to(repo)))

    print(f"imported {len(outputs)} exact snapshots for {args.ownership_unit}")
    for output in sorted(outputs):
        print(output)


if __name__ == "__main__":
    main()
