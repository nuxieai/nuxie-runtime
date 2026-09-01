#!/usr/bin/env python3
"""Import the exact pinned inputs owned by Vulkan's shader registry."""

from __future__ import annotations

import argparse
import hashlib
import re
import shutil
import subprocess
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
    parser.add_argument("--generated-root", type=Path)
    args = parser.parse_args()

    repo = args.repo_root.resolve()
    upstream = args.upstream_root.resolve()
    source_cpp = upstream / "renderer/src/vulkan/vulkan_shaders.cpp"
    generated_root = args.generated_root or upstream / "renderer/src/shaders/out/generated/spirv"
    destination = (
        repo
        / "crates/nuxie-renderer/src/mechanical_port/vulkan/generated/spirv"
    )
    snapshot_root = (
        repo / "crates/nuxie-renderer/src/mechanical_port/vulkan/source"
    )
    checksums = {name: digest for digest, name in (
        line.split() for line in Path(__file__).with_name("vulkan-spirv.sha256").read_text().splitlines()
    )}
    pinned_ref = "2cfa84e8103aeeeff4c2bfee92839ab580521660"

    includes = [
        match.decode("utf-8")
        for match in INCLUDE_RE.findall(source_cpp.read_bytes())
    ]
    if len(includes) != 93 or len(set(includes)) != 93:
        raise SystemExit(
            f"expected 93 unique vulkan_shaders.cpp headers, got "
            f"{len(includes)} includes/{len(set(includes))} unique"
        )

    if set(includes) != set(checksums):
        raise SystemExit("Vulkan shader includes differ from retained checksums")

    snapshot_root.mkdir(parents=True, exist_ok=True)
    for source_path in (
        "renderer/src/vulkan/vulkan_shaders.hpp",
        "renderer/src/vulkan/vulkan_shaders.cpp",
    ):
        source = upstream / source_path
        data = source.read_bytes()
        pinned = subprocess.check_output(["git", "-C", str(upstream), "show", f"{pinned_ref}:{source_path}"])
        if data != pinned:
            raise SystemExit(f"pinned source mismatch for {source_path}")
        target = snapshot_root / source_path.replace("/", "_")
        shutil.copyfile(source, target)
        if target.read_bytes() != data:
            raise SystemExit(f"byte copy mismatch for {source_path}")

    destination.mkdir(parents=True, exist_ok=True)
    for name in includes:
        source = generated_root / name
        data = source.read_bytes()
        actual = sha256(data)
        expected = checksums.get(name)
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
