#!/usr/bin/env python3
"""Fail closed until every renderer-port preparation denominator is reviewed."""

from __future__ import annotations

import argparse
import csv
import hashlib
import sys
import tomllib
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--upstream-root", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--denominators-only", action="store_true")
    return parser.parse_args()


def rows(path: Path) -> list[dict[str, str]]:
    with path.open(newline="") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def main() -> int:
    args = parse_args()
    repo = args.repo_root.resolve()
    manifest_path = args.manifest if args.manifest.is_absolute() else repo / args.manifest
    manifest = tomllib.loads(manifest_path.read_text())
    require(manifest["active_queue"] == "preparation", "active queue is not preparation")
    actual_ref = __import__("subprocess").run(
        ["git", "-C", str(args.upstream_root), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    require(actual_ref == manifest["upstream_ref"], "upstream revision drift")

    paths = {
        key: repo / manifest[key]
        for key in (
            "source_inventory",
            "ownership_inventory",
            "dependency_inventory",
            "ownership_unit_order",
            "generated_artifact_inventory",
            "configuration_inventory",
            "field_inventory",
            "lifecycle_inventory",
            "legacy_wgpu_inventory",
        )
    }
    tables = {key: rows(path) for key, path in paths.items()}
    expected = manifest["denominator"]
    count_keys = {
        "source_inventory": "sources",
        "ownership_inventory": "ownership_rows",
        "dependency_inventory": "dependency_edges",
        "ownership_unit_order": "ownership_units",
        "generated_artifact_inventory": "generated_artifacts",
        "configuration_inventory": "configuration_rows",
        "field_inventory": "fields",
        "lifecycle_inventory": "lifecycle_events",
        "legacy_wgpu_inventory": "legacy_wgpu_files",
    }
    for table, denominator in count_keys.items():
        require(
            len(tables[table]) == expected[denominator],
            f"{table} denominator drift: {len(tables[table])} != {expected[denominator]}",
        )

    sources = tables["source_inventory"]
    ownership = tables["ownership_inventory"]
    source_paths = {row["source_path"] for row in sources}
    require(len(source_paths) == len(sources), "duplicate source inventory path")
    require(
        {row["source_path"] for row in ownership} == source_paths,
        "ownership/source denominator mismatch",
    )
    targets = [row["target_path"] for row in ownership]
    require(len(set(targets)) == len(targets), "overlapping target ownership")
    require(all(row["mapping_status"] == "exclusive" for row in ownership), "nonexclusive mapping")
    require(all(row["translation_status"] == "pending" for row in ownership), "translation started before admission")
    units = {row["ownership_unit"] for row in ownership}
    ordered_units = {row["ownership_unit"] for row in tables["ownership_unit_order"]}
    require(units == ordered_units, "ownership order omits or invents units")
    for table in ("configuration_inventory", "field_inventory", "lifecycle_inventory"):
        require(
            {row["ownership_unit"] for row in tables[table]} <= units,
            f"{table} references invented ownership units",
        )

    oracle_path = repo / manifest["oracle_contract"]
    oracle = tomllib.loads(oracle_path.read_text())
    corpus = repo / oracle["corpus"]
    require(digest(corpus) == oracle["corpus_sha256"], "oracle corpus hash drift")
    corpus_rows = tomllib.loads(corpus.read_text())["entry"]
    require(len(corpus_rows) == oracle["corpus_entries"] == expected["corpus_entries"], "corpus count drift")
    require(not oracle["corpus_exclusions"], "corpus exclusions introduced")
    require(not oracle["candidate_derived_tolerances"], "candidate-derived tolerances enabled")

    print(
        "backend preparation denominators clean: "
        + ", ".join(f"{key}={expected[key]}" for key in sorted(expected))
    )
    if args.denominators_only:
        return 0

    blockers: list[str] = []
    if manifest["preparation_status"] != "green":
        blockers.append(f"campaign preparation_status={manifest['preparation_status']}")
    toolchain = tomllib.loads((repo / manifest["toolchain_authority"]).read_text())
    if toolchain["hermetic_bootstrap_status"] != "green":
        blockers.append("shader toolchain is not hermetically bootstrapped")
    if oracle["tolerance_status"] != "frozen-source-repeatability":
        blockers.append("primary-source repeatability budgets are not frozen")
    for backend in oracle["backend"]:
        if backend["root_status"] != "green":
            blockers.append(f"{backend['id']} primary/candidate root is {backend['root_status']}")
    for platform in oracle["platform"]:
        if platform["requirement"] == "local-blocking" and platform["status"] != "green":
            blockers.append(f"local platform {platform['id']} is {platform['status']}")
    for table, status_column in (
        ("configuration_inventory", "review_status"),
        ("field_inventory", "ownership_review"),
        ("lifecycle_inventory", "review_status"),
    ):
        pending = sum("review-required" in row[status_column] for row in tables[table])
        if pending:
            blockers.append(f"{table} has {pending} review-required rows")
    if blockers:
        print("backend preparation gate RED:", file=sys.stderr)
        for blocker in blockers:
            print(f"- {blocker}", file=sys.stderr)
        return 1
    print("backend preparation gate GREEN; translation admission may begin")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError) as error:
        print(f"backend preparation denominator failure: {error}", file=sys.stderr)
        raise SystemExit(1)
