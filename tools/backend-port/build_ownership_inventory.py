#!/usr/bin/env python3
"""Classify every pinned backend source into one exclusive ownership unit."""

from __future__ import annotations

import argparse
import csv
import re
import sys
from dataclasses import dataclass
from pathlib import Path


HEADER = (
    "campaign",
    "source_path",
    "source_sha256",
    "ownership_unit",
    "source_role",
    "port_disposition",
    "target_path",
    "mapping_status",
    "translation_status",
)


@dataclass(frozen=True, order=True)
class OwnershipRow:
    campaign: str
    source_path: str
    source_sha256: str
    ownership_unit: str
    source_role: str
    port_disposition: str
    target_path: str
    mapping_status: str = "exclusive"
    translation_status: str = "pending"

    def tsv(self) -> str:
        return "\t".join(str(getattr(self, column)) for column in HEADER)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--source-inventory", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    return parser.parse_args()


def semantic_stem(path: Path) -> str:
    stem = path.stem
    for suffix in ("_vulkan", "_wgpu", "_gl"):
        if stem.endswith(suffix):
            stem = stem[: -len(suffix)]
    return stem.replace("-", "_")


def backend_layer(source_path: str) -> str:
    if "/ore/" in source_path:
        return "ore"
    if "/wagyu-port/" in source_path:
        return "wagyu"
    return "renderer"


def shared_unit(source_path: str) -> str:
    fixed = {
        "renderer/make_dawn.sh": "dependency:dawn",
        "renderer/make_moltenvk.sh": "dependency:moltenvk",
        "renderer/make_swiftshader.sh": "dependency:swiftshader",
        "renderer/premake5.lua": "build:renderer",
        "renderer/premake5_pls_renderer.lua": "build:pls_renderer",
        "renderer/src/shaders/Makefile": "shader:build_graph",
        "renderer/src/shaders/minify.py": "shader:minifier",
        "renderer/src/shaders/spirv_binary_to_header.py": "shader:spirv_header_generator",
        "renderer/src/shaders/wgsl_to_header.py": "shader:wgsl_header_generator",
    }
    return fixed.get(source_path, f"shader:source:{semantic_stem(Path(source_path))}")


def ownership_unit(campaign: str, source_path: str) -> str:
    if campaign == "shader-build-authority":
        return shared_unit(source_path)
    layer = backend_layer(source_path)
    stem = semantic_stem(Path(source_path))
    if stem.startswith("ore_"):
        stem = stem[4:]
    return f"{campaign}:{layer}:{stem}"


def source_role(source_kind: str, source_path: str) -> str:
    if source_kind == "header":
        return "declaration"
    if source_kind == "implementation":
        return "implementation"
    if source_kind == "objective-cpp-implementation":
        return "platform-implementation"
    if source_kind == "shader-input":
        return "generated-input"
    if source_kind == "generator":
        return "generator"
    if source_kind == "nonsemantic-support":
        return "nonsemantic-evidence"
    if "/wagyu-port/" in source_path:
        return "compatibility-build-input"
    return "build-input"


def port_disposition(campaign: str, role: str, source_path: str) -> str:
    if role == "nonsemantic-evidence":
        return "evidence-only"
    if campaign == "shader-build-authority":
        return "shared-authority"
    if "/wagyu-port/" in source_path:
        return "dependency-authority"
    webgl2_exclusions = {
        "renderer/src/gl/load_gles_extensions.cpp",
        "renderer/src/gl/pls_impl_ext_native.cpp",
        "renderer/src/gl/pls_impl_rw_texture.cpp",
    }
    if campaign == "webgl2" and (
        role == "platform-implementation" or source_path in webgl2_exclusions
    ):
        return "source-exclusion-non-webgl2-build"
    return "translate"


def target_path(campaign: str, source_path: str, role: str) -> str:
    safe = re.sub(r"[^a-zA-Z0-9_]+", "_", source_path).strip("_").lower()
    role_suffix = {
        "declaration": "decl",
        "implementation": "impl",
        "platform-implementation": "platform_impl",
        "generated-input": "generated_input",
        "generator": "generator",
        "build-input": "build_input",
        "compatibility-build-input": "compat_build_input",
        "nonsemantic-evidence": "evidence",
    }[role]
    return f"crates/nuxie-renderer/src/mechanical_port/{campaign}/{safe}__{role_suffix}.rs"


def load_sources(path: Path) -> list[dict[str, str]]:
    with path.open(newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    if not rows:
        raise ValueError("source inventory is empty")
    expected = {
        "campaign",
        "source_kind",
        "source_path",
        "source_sha256",
        "classification_status",
    }
    if set(rows[0]) != expected:
        raise ValueError(f"unexpected source inventory columns: {set(rows[0])}")
    return rows


def render(source_inventory: Path) -> str:
    ownership_rows: list[OwnershipRow] = []
    seen_sources: set[str] = set()
    seen_targets: set[str] = set()
    for source in load_sources(source_inventory):
        source_path_value = source["source_path"]
        if source_path_value in seen_sources:
            raise ValueError(f"duplicate source ownership: {source_path_value}")
        seen_sources.add(source_path_value)
        campaign = source["campaign"]
        role = source_role(source["source_kind"], source_path_value)
        target = target_path(campaign, source_path_value, role)
        if target in seen_targets:
            raise ValueError(f"overlapping target ownership: {target}")
        seen_targets.add(target)
        ownership_rows.append(
            OwnershipRow(
                campaign=campaign,
                source_path=source_path_value,
                source_sha256=source["source_sha256"],
                ownership_unit=ownership_unit(campaign, source_path_value),
                source_role=role,
                port_disposition=port_disposition(campaign, role, source_path_value),
                target_path=target,
                translation_status=(
                    "excluded-by-pinned-build"
                    if port_disposition(campaign, role, source_path_value)
                    == "source-exclusion-non-webgl2-build"
                    else "pending"
                ),
            )
        )
    ownership_rows.sort()
    return "\n".join(("\t".join(HEADER), *(row.tsv() for row in ownership_rows))) + "\n"


def main() -> int:
    args = parse_args()
    source_inventory = args.source_inventory
    if not source_inventory.is_absolute():
        source_inventory = args.repo_root / source_inventory
    output = args.output
    if not output.is_absolute():
        output = args.repo_root / output
    rendered = render(source_inventory)
    row_count = len(rendered.splitlines()) - 1
    if args.check:
        if not output.is_file() or output.read_text() != rendered:
            print("backend ownership inventory is stale", file=sys.stderr)
            return 1
        print(f"backend ownership inventory clean: {row_count} exclusive rows")
        return 0
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(rendered)
    print(f"wrote {row_count} exclusive ownership rows to {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
