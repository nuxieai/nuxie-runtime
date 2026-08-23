#!/usr/bin/env python3
"""Validate source-owner translation receipts against frozen campaign authority."""

from __future__ import annotations

import argparse
import csv
import hashlib
import subprocess
import sys
import tomllib
from pathlib import Path


TRANSLATION_QUEUES = {
    "translation-admission",
    "vulkan-translation",
    "webgpu-translation",
    "webgl2-translation",
}
QUEUE_CAMPAIGNS = {
    "translation-admission": {"shader-build-authority"},
    "vulkan-translation": {"shader-build-authority", "vulkan"},
    "webgpu-translation": {"shader-build-authority", "vulkan", "webgpu"},
    "webgl2-translation": {
        "shader-build-authority",
        "vulkan",
        "webgpu",
        "webgl2",
    },
}
PRIOR_CAMPAIGNS = {
    "translation-admission": set(),
    "vulkan-translation": {"shader-build-authority"},
    "webgpu-translation": {"shader-build-authority", "vulkan"},
    "webgl2-translation": {"shader-build-authority", "vulkan", "webgpu"},
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--upstream-root", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    return parser.parse_args()


def read_rows(path: Path) -> list[dict[str, str]]:
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
    upstream = args.upstream_root.resolve()
    manifest_path = args.manifest if args.manifest.is_absolute() else repo / args.manifest
    manifest = tomllib.loads(manifest_path.read_text())
    require(manifest["active_queue"] in TRANSLATION_QUEUES, "active queue is not translation")
    require(manifest["preparation_status"] == "green", "preparation is not green")
    require(manifest["ignored_skills"] == ["implement", "tdd"], "ignored-skill contract drift")
    actual_ref = subprocess.run(
        ["git", "-C", str(upstream), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    require(actual_ref == manifest["upstream_ref"], "upstream revision drift")

    ownership = read_rows(repo / manifest["ownership_inventory"])
    order = read_rows(repo / manifest["ownership_unit_order"])
    owner_by_source = {row["source_path"]: row for row in ownership}
    unit_order = {row["ownership_unit"]: row for row in order}
    require(len(owner_by_source) == len(ownership), "duplicate frozen source owner")
    require(len(unit_order) == len(order), "duplicate frozen ownership unit")

    receipt_dir = repo / manifest["translation_receipt_directory"]
    receipts = [] if not receipt_dir.exists() else sorted(receipt_dir.glob("*.translation.toml"))
    completed_sources: set[str] = set()
    completed_units: set[str] = set()
    target_paths: set[str] = set()
    for receipt_path in receipts:
        receipt = tomllib.loads(receipt_path.read_text())
        require(receipt.get("schema_version") == 1, f"invalid receipt schema: {receipt_path}")
        source_path = receipt["source_path"]
        require(source_path in owner_by_source, f"receipt invents source: {source_path}")
        owner = owner_by_source[source_path]
        require(source_path not in completed_sources, f"duplicate source receipt: {source_path}")
        require(receipt["campaign"] == owner["campaign"], f"receipt campaign drift: {source_path}")
        require(
            receipt["campaign"] in QUEUE_CAMPAIGNS[manifest["active_queue"]],
            f"receipt campaign is ahead of active queue: {source_path}",
        )
        require(receipt["ownership_unit"] == owner["ownership_unit"], f"receipt unit drift: {source_path}")
        require(receipt["source_sha256"] == owner["source_sha256"], f"receipt source hash drift: {source_path}")
        require(digest(upstream / source_path) == owner["source_sha256"], f"upstream source drift: {source_path}")
        require(receipt["target_path"] == owner["target_path"], f"receipt target drift: {source_path}")
        target = repo / owner["target_path"]
        require(target.is_file(), f"translated target is missing: {target}")
        require(digest(target) == receipt["target_sha256"], f"translated target hash drift: {target}")
        snapshot = repo / receipt["source_snapshot_path"]
        require(snapshot.is_file(), f"translated source snapshot is missing: {snapshot}")
        require(
            digest(snapshot) == receipt["source_snapshot_sha256"] == owner["source_sha256"],
            f"translated source snapshot drift: {snapshot}",
        )
        require(
            owner["port_disposition"]
            in {"translate", "shared-authority", "dependency-authority"},
            f"non-translation source received a translation: {source_path}",
        )
        require(owner["target_path"] not in target_paths, f"translated target overlap: {target}")
        require(receipt["translation_kind"] == "complete-source-owner", f"partial translation receipt: {source_path}")
        expected_dependencies = {
            value
            for value in unit_order[owner["ownership_unit"]]["dependency_units"].split(";")
            if value
        }
        require(
            set(receipt["dependency_units"]) == expected_dependencies,
            f"receipt dependency drift: {source_path}",
        )
        completed_sources.add(source_path)
        target_paths.add(owner["target_path"])

    sources_by_unit: dict[str, set[str]] = {}
    for owner in ownership:
        if owner["port_disposition"] not in {
            "translate",
            "shared-authority",
            "dependency-authority",
        }:
            continue
        sources_by_unit.setdefault(owner["ownership_unit"], set()).add(owner["source_path"])
    for unit, sources in sources_by_unit.items():
        translated = sources & completed_sources
        require(not translated or translated == sources, f"partial ownership unit translated: {unit}")
        if translated:
            completed_units.add(unit)

    for unit in completed_units:
        row = unit_order[unit]
        dependencies = {value for value in row["dependency_units"].split(";") if value}
        require(dependencies <= completed_units, f"translated before dependencies: {unit}")
    for prior_campaign in PRIOR_CAMPAIGNS[manifest["active_queue"]]:
        incomplete = {
            unit
            for unit, source_paths in sources_by_unit.items()
            if unit_order[unit]["campaign"] == prior_campaign
            and not source_paths <= completed_sources
        }
        require(
            not incomplete,
            f"active queue opened before {prior_campaign} closed",
        )

    print(
        "backend translation receipts clean: "
        f"sources={len(completed_sources)}/{sum(len(v) for v in sources_by_unit.values())}, "
        f"units={len(completed_units)}/{len(sources_by_unit)}, queue={manifest['active_queue']}"
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, tomllib.TOMLDecodeError) as error:
        print(f"backend translation failure: {error}", file=sys.stderr)
        raise SystemExit(1)
