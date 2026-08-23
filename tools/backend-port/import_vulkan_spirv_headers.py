#!/usr/bin/env python3
"""Import the exact pinned inputs owned by Vulkan's shader registry."""

from __future__ import annotations

import argparse
import csv
import hashlib
import re
import shutil
from pathlib import Path


INCLUDE_RE = re.compile(
    rb'#include "generated/shaders/spirv/([^"\r\n]+)"'
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", required=True, type=Path)
    parser.add_argument("--upstream-root", required=True, type=Path)
    args = parser.parse_args()

    repo = args.repo_root.resolve()
    upstream = args.upstream_root.resolve()
    source_cpp = upstream / "renderer/src/vulkan/vulkan_shaders.cpp"
    generated_root = upstream / "renderer/src/shaders/out/generated/spirv"
    destination = (
        repo
        / "crates/nuxie-renderer/src/mechanical_port/vulkan/generated/spirv"
    )
    snapshot_root = (
        repo / "crates/nuxie-renderer/src/mechanical_port/vulkan/source"
    )
    ledger_path = repo / "docs/backend-port-generated-artifacts.tsv"
    inventory_path = repo / "docs/backend-port-source-inventory.tsv"

    includes = [
        match.decode("utf-8")
        for match in INCLUDE_RE.findall(source_cpp.read_bytes())
    ]
    if len(includes) != 93 or len(set(includes)) != 93:
        raise SystemExit(
            f"expected 93 unique vulkan_shaders.cpp headers, got "
            f"{len(includes)} includes/{len(set(includes))} unique"
        )

    with ledger_path.open(newline="", encoding="utf-8") as ledger_file:
        ledger = {
            row["artifact_path"]: row
            for row in csv.DictReader(ledger_file, delimiter="\t")
        }

    with inventory_path.open(newline="", encoding="utf-8") as inventory_file:
        inventory = {
            row["source_path"]: row
            for row in csv.DictReader(inventory_file, delimiter="\t")
        }

    snapshot_root.mkdir(parents=True, exist_ok=True)
    for source_path in (
        "renderer/src/vulkan/vulkan_shaders.hpp",
        "renderer/src/vulkan/vulkan_shaders.cpp",
    ):
        source = upstream / source_path
        data = source.read_bytes()
        row = inventory.get(source_path)
        if row is None or sha256(data) != row["source_sha256"]:
            raise SystemExit(f"source inventory mismatch for {source_path}")
        target = snapshot_root / source_path.replace("/", "_")
        shutil.copyfile(source, target)
        if target.read_bytes() != data:
            raise SystemExit(f"byte copy mismatch for {source_path}")

    destination.mkdir(parents=True, exist_ok=True)
    for name in includes:
        artifact_path = f"out/generated/spirv/{name}"
        row = ledger.get(artifact_path)
        if row is None:
            raise SystemExit(f"missing frozen generated-artifact row: {artifact_path}")
        if row["retention"] != "retained" or row["direct_include_count"] != "1":
            raise SystemExit(f"unexpected frozen retention row: {row}")

        source = generated_root / name
        data = source.read_bytes()
        actual = sha256(data)
        expected = row["artifact_sha256"]
        if actual != expected:
            raise SystemExit(
                f"generated header hash mismatch for {name}: "
                f"expected {expected}, got {actual}"
            )
        target = destination / name
        shutil.copyfile(source, target)
        if target.read_bytes() != data:
            raise SystemExit(f"byte copy mismatch for {name}")

    print(
        "imported 2 source snapshots and "
        f"{len(includes)} frozen Vulkan SPIR-V headers"
    )


if __name__ == "__main__":
    main()
