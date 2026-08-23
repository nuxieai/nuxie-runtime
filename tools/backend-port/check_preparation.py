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
            "repeatability_inventory",
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
        "repeatability_inventory": "repeatability_rows",
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
    require(
        all(
            row["translation_status"]
            in {"pending", "excluded-by-pinned-build"}
            for row in ownership
        ),
        "translation started before admission",
    )
    require(
        all(
            (row["port_disposition"] == "source-exclusion-non-webgl2-build")
            == (row["translation_status"] == "excluded-by-pinned-build")
            for row in ownership
        ),
        "source exclusion and translation status disagree",
    )
    units = {row["ownership_unit"] for row in ownership}
    ordered_units = {row["ownership_unit"] for row in tables["ownership_unit_order"]}
    require(units == ordered_units, "ownership order omits or invents units")
    owner_contracts = tomllib.loads((repo / manifest["owner_contracts"]).read_text())
    reviewed_units: set[str] = set()
    for family in owner_contracts["family"]:
        require(family["review_status"] == "reviewed", f"owner family not reviewed: {family['id']}")
        matched = {
            unit
            for unit in units
            if any(unit.startswith(prefix) for prefix in family["unit_prefixes"])
            and any(row["campaign"] == family["campaign"] and row["ownership_unit"] == unit for row in ownership)
        }
        require(matched, f"owner family matches no units: {family['id']}")
        require(not (reviewed_units & matched), f"owner family overlap: {family['id']}")
        reviewed_units.update(matched)
    require(reviewed_units == units, "owner contracts omit or invent ownership units")
    for table in ("configuration_inventory", "field_inventory", "lifecycle_inventory"):
        require(
            {row["ownership_unit"] for row in tables[table]} <= units,
            f"{table} references invented ownership units",
        )

    oracle_path = repo / manifest["oracle_contract"]
    oracle = tomllib.loads(oracle_path.read_text())
    webgl2 = next(backend for backend in oracle["backend"] if backend["id"] == "webgl2")
    webgl2_root_path = repo / webgl2["source_root_manifest"]
    webgl2_root = tomllib.loads(webgl2_root_path.read_text())
    require(
        webgl2_root["source_runtime_revision"] == oracle["upstream_ref"],
        "WebGL2 source root revision drift",
    )
    for artifact in webgl2_root["artifact"]:
        artifact_path = repo / artifact["path"]
        require(
            artifact_path.is_file() and digest(artifact_path) == artifact["sha256"],
            f"WebGL2 source root artifact drift: {artifact['path']}",
        )
    archive_paths = {
        "rive_pls_renderer": "renderer/out/cpp-webgl2-oracle/librive_pls_renderer.a",
        "rive": "renderer/out/cpp-webgl2-oracle/librive.a",
        "rive_decoders": "renderer/out/cpp-webgl2-oracle/librive_decoders.a",
        "libpng": "renderer/out/cpp-webgl2-oracle/liblibpng.a",
        "zlib": "renderer/out/cpp-webgl2-oracle/libzlib.a",
        "libjpeg": "renderer/out/cpp-webgl2-oracle/liblibjpeg.a",
        "libwebp": "renderer/out/cpp-webgl2-oracle/liblibwebp.a",
        "rive_harfbuzz": "renderer/out/cpp-webgl2-oracle/librive_harfbuzz.a",
        "rive_sheenbidi": "renderer/out/cpp-webgl2-oracle/librive_sheenbidi.a",
        "rive_yoga": "renderer/out/cpp-webgl2-oracle/librive_yoga.a",
    }
    frozen_archives = {archive["name"]: archive for archive in webgl2_root["upstream_archive"]}
    require(
        set(frozen_archives) == set(archive_paths),
        "WebGL2 source root archive denominator drift",
    )
    for name, relative_path in archive_paths.items():
        archive_path = args.upstream_root / relative_path
        require(
            archive_path.is_file()
            and digest(archive_path) == frozen_archives[name]["sha256"],
            f"WebGL2 source root archive drift: {name}",
        )
    corpus = repo / oracle["corpus"]
    require(digest(corpus) == oracle["corpus_sha256"], "oracle corpus hash drift")
    corpus_rows = tomllib.loads(corpus.read_text())["entry"]
    require(len(corpus_rows) == oracle["corpus_entries"] == expected["corpus_entries"], "corpus count drift")
    require(not oracle["corpus_exclusions"], "corpus exclusions introduced")
    require(not oracle["candidate_derived_tolerances"], "candidate-derived tolerances enabled")
    corpus_identity = {row["id"]: row["mode"] for row in corpus_rows}
    primary_backends = {
        backend["id"]: backend["primary_backend_id"] for backend in oracle["backend"]
    }
    repeatability = tables["repeatability_inventory"]
    repeatability_identity = {
        (row["campaign"], row["corpus_entry"]) for row in repeatability
    }
    require(
        len(repeatability_identity) == len(repeatability),
        "duplicate source repeatability rows",
    )
    require(
        repeatability_identity
        == {
            (campaign, entry)
            for campaign in primary_backends
            for entry in corpus_identity
        },
        "source repeatability/corpus denominator mismatch",
    )
    for row in repeatability:
        campaign = row["campaign"]
        require(
            row["primary_backend_id"] == primary_backends[campaign],
            f"source repeatability backend drift: {campaign}",
        )
        require(
            row["mode"] == corpus_identity[row["corpus_entry"]],
            f"source repeatability mode drift: {row['corpus_entry']}",
        )
        require(
            row["candidate_output_observed"] == "false",
            f"candidate output entered source repeatability: {row['corpus_entry']}",
        )
        if row["status"] == "frozen-source-repeatability":
            require(
                row["run_a_replay_sha256"] == row["run_b_replay_sha256"],
                f"source repeatability used different binaries: {row['corpus_entry']}",
            )
            require(
                row["observed_different_pixels"]
                == row["frozen_max_different_pixels"]
                and row["observed_max_channel_delta"]
                == row["frozen_max_channel_delta"],
                f"repeatability budget is not source-derived: {row['corpus_entry']}",
            )
        else:
            require(
                row["status"] == "pending-source-capture",
                f"invalid source repeatability status: {row['status']}",
            )

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
    if toolchain["source_bootstrap_status"] != "green":
        blockers.append("shader toolchain source bootstrap is not green")
    if oracle["tolerance_status"] != "frozen-source-repeatability":
        blockers.append("primary-source repeatability budgets are not frozen")
    for campaign in primary_backends:
        pending_repeatability = sum(
            row["campaign"] == campaign
            and row["status"] != "frozen-source-repeatability"
            for row in repeatability
        )
        if pending_repeatability:
            blockers.append(
                f"{campaign} source repeatability has {pending_repeatability} pending rows"
            )
    for backend in oracle["backend"]:
        if backend["primary_root_status"] != "green":
            blockers.append(
                f"{backend['id']} primary root is {backend['primary_root_status']}"
            )
        if backend["candidate_root_contract_status"] != "frozen":
            blockers.append(
                f"{backend['id']} candidate root contract is "
                f"{backend['candidate_root_contract_status']}"
            )
    for platform in oracle["platform"]:
        if (
            platform["requirement"] == "local-blocking"
            and platform["primary_status"] != "green"
        ):
            blockers.append(
                f"local primary platform {platform['id']} is "
                f"{platform['primary_status']}"
            )
    for table, status_column in (
        ("configuration_inventory", "review_status"),
        ("field_inventory", "ownership_review"),
        ("lifecycle_inventory", "review_status"),
    ):
        pending = sum(
            "review-required" in row[status_column]
            and row["ownership_unit"] not in reviewed_units
            for row in tables[table]
        )
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
