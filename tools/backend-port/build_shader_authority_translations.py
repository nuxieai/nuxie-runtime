#!/usr/bin/env python3
"""Generate exact compiled owners for every pinned shader source input."""

from __future__ import annotations

import argparse
import csv
import hashlib
import re
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class Output:
    path: Path
    content: bytes


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--upstream-root", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    return parser.parse_args()


def rows(path: Path) -> list[dict[str, str]]:
    with path.open(newline="") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def sha256(content: bytes) -> str:
    return hashlib.sha256(content).hexdigest()


def logical_line_count(content: bytes) -> int:
    return len(content.splitlines())


def rust_string(value: str) -> str:
    return '"' + value.replace("\\", "\\\\").replace('"', '\\"') + '"'


def target_content(owner: dict[str, str], snapshot_name: str, content: bytes) -> bytes:
    line_count = logical_line_count(content)
    text = f'''//! Exact generated-input translation of {owner["source_path"]}.
//!
//! Shader behavior is retained as the unchanged pinned byte program. Backend
//! compilers consume generated artifacts from this authority; no Rust or
//! legacy-WGPU shader is substituted here.

#![allow(dead_code)]

pub const PINNED_UPSTREAM_COMMIT: &str = "4ac7b32798da0482e441ef09304dc3b480ed3ee5";
pub const PINNED_SOURCE_PATH: &str = {rust_string(owner["source_path"])};
pub const PINNED_SOURCE_SHA256: &str = {rust_string(owner["source_sha256"])};
pub const OWNERSHIP_UNIT: &str = {rust_string(owner["ownership_unit"])};
pub const PINNED_SOURCE_LINE_COUNT: usize = {line_count};
pub const PINNED_SOURCE_BYTE_COUNT: usize = {len(content)};
pub const PINNED_SOURCE: &[u8] = include_bytes!({rust_string("source/" + snapshot_name)});

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
'''
    return text.encode()


def receipt_content(
    owner: dict[str, str],
    snapshot_path: str,
    target_hash: str,
    content: bytes,
    dependency_units: list[str],
    configuration_count: int,
) -> bytes:
    dependencies = ", ".join(rust_string(value) for value in dependency_units)
    text = f'''schema_version = 1
campaign = "shader-build-authority"
ownership_unit = {rust_string(owner["ownership_unit"])}
translation_kind = "complete-source-owner"
source_path = {rust_string(owner["source_path"])}
source_sha256 = {rust_string(owner["source_sha256"])}
target_path = {rust_string(owner["target_path"])}
target_sha256 = {rust_string(target_hash)}
source_snapshot_path = {rust_string(snapshot_path)}
source_snapshot_sha256 = {rust_string(owner["source_sha256"])}
dependency_units = [{dependencies}]
source_lines = {logical_line_count(content)}
source_bytes = {len(content)}
configuration_authorities = {configuration_count}
compile_evidence = "cargo check -p nuxie-renderer --no-default-features"
scope_note = "Preserves the complete shader program byte-for-byte; generated outputs remain governed by the frozen 520-row artifact ledger."
'''
    return text.encode()


def render(repo: Path, upstream: Path, manifest: dict) -> list[Output]:
    ownership = rows(repo / manifest["ownership_inventory"])
    order = {
        row["ownership_unit"]: row
        for row in rows(repo / manifest["ownership_unit_order"])
    }
    configurations = rows(repo / manifest["configuration_inventory"])
    configuration_counts: dict[str, int] = {}
    for row in configurations:
        configuration_counts[row["source_path"]] = (
            configuration_counts.get(row["source_path"], 0) + 1
        )
    shader_owners = sorted(
        (
            owner
            for owner in ownership
            if owner["campaign"] == "shader-build-authority"
            and owner["source_role"] == "generated-input"
        ),
        key=lambda owner: owner["source_path"],
    )
    if len(shader_owners) != 78:
        raise ValueError(f"shader source denominator drift: {len(shader_owners)} != 78")

    target_dir = repo / "crates/nuxie-renderer/src/mechanical_port/shader-build-authority"
    source_dir = target_dir / "source"
    receipt_dir = repo / manifest["translation_receipt_directory"]
    outputs: list[Output] = []
    inventory_lines = [
        "//! @generated exact shader source-owner module inventory.",
        "#![allow(dead_code)]",
        "",
    ]
    for index, owner in enumerate(shader_owners):
        source = upstream / owner["source_path"]
        content = source.read_bytes()
        if sha256(content) != owner["source_sha256"]:
            raise ValueError(f"upstream shader source drift: {owner['source_path']}")
        target = repo / owner["target_path"]
        stem = target.stem
        snapshot_name = f"{stem}.source"
        snapshot = source_dir / snapshot_name
        translated = target_content(owner, snapshot_name, content)
        unit_order = order[owner["ownership_unit"]]
        dependency_units = [
            value for value in unit_order["dependency_units"].split(";") if value
        ]
        receipt_name = f"shader-input-{stem}.translation.toml"
        receipt = receipt_dir / receipt_name
        outputs.extend(
            [
                Output(snapshot, content),
                Output(target, translated),
                Output(
                    receipt,
                    receipt_content(
                        owner,
                        str(snapshot.relative_to(repo)),
                        sha256(translated),
                        content,
                        dependency_units,
                        configuration_counts.get(owner["source_path"], 0),
                    ),
                ),
            ]
        )
        module_name = f"shader_source_owner_{index:03d}"
        inventory_lines.extend(
            [
                f'#[path = "shader-build-authority/{target.name}"]',
                f"mod {module_name};",
                "",
            ]
        )
    inventory = repo / "crates/nuxie-renderer/src/mechanical_port/backend_shader_authority_inventory.rs"
    outputs.append(Output(inventory, ("\n".join(inventory_lines)).encode()))
    return outputs


def main() -> int:
    args = parse_args()
    repo = args.repo_root.resolve()
    upstream = args.upstream_root.resolve()
    manifest_path = args.manifest if args.manifest.is_absolute() else repo / args.manifest
    manifest = tomllib.loads(manifest_path.read_text())
    outputs = render(repo, upstream, manifest)
    stale = [output for output in outputs if not output.path.is_file() or output.path.read_bytes() != output.content]
    if args.check:
        if stale:
            print("backend shader authority translations are stale:", file=sys.stderr)
            for output in stale:
                print(f"- {output.path.relative_to(repo)}", file=sys.stderr)
            return 1
        print("backend shader authority translations clean: 78 exact source owners")
        return 0
    for output in outputs:
        output.path.parent.mkdir(parents=True, exist_ok=True)
        output.path.write_bytes(output.content)
    print(f"wrote {len(outputs)} exact shader authority artifacts for 78 source owners")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, tomllib.TOMLDecodeError) as error:
        print(f"backend shader authority generation failure: {error}", file=sys.stderr)
        raise SystemExit(1)
