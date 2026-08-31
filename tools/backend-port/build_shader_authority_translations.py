#!/usr/bin/env python3
"""Generate exact compiled owners for every pinned shader source input."""

from __future__ import annotations

import argparse
import hashlib
import re
import sys
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
    parser.add_argument("--check", action="store_true")
    return parser.parse_args()


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

pub const PINNED_UPSTREAM_COMMIT: &str = {rust_string(owner["upstream_commit"])};
pub const PINNED_SOURCE_PATH: &str = {rust_string(owner["source_path"])};
pub const PINNED_SOURCE_SHA256: &str = {rust_string(owner["source_sha256"])};
pub const OWNERSHIP_UNIT: &str = {rust_string(owner["ownership_unit"])};
pub const PINNED_SOURCE_LINE_COUNT: usize = {line_count};
pub const PINNED_SOURCE_BYTE_COUNT: usize = {len(content)};
pub const PINNED_SOURCE: &[u8] = include_bytes!({rust_string("source/" + snapshot_name)});

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
'''
    return text.encode()


def render(repo: Path, upstream: Path) -> list[Output]:
    # The translated shader owners retain their own source paths and hashes.
    # Reuse that executable provenance instead of a completed campaign ledger.
    target_dir = repo / "crates/nuxie-renderer/src/mechanical_port/shader-build-authority"
    shader_owners = []
    for target in target_dir.glob("*__generated_input.rs"):
        text = target.read_text()
        def constant(name: str) -> str:
            match = re.search(rf'pub const {name}: &str = "([^"\n]+)";', text)
            if not match:
                raise ValueError(f"missing {name} in {target}")
            return match[1]
        shader_owners.append({
            "upstream_commit": constant("PINNED_UPSTREAM_COMMIT"),
            "source_path": constant("PINNED_SOURCE_PATH"),
            "source_sha256": constant("PINNED_SOURCE_SHA256"),
            "ownership_unit": constant("OWNERSHIP_UNIT"),
            "target_path": str(target.relative_to(repo)),
        })
    if len(shader_owners) != 78:
        raise ValueError(f"pinned shader source owner count drift: {len(shader_owners)} != 78")
    outputs: list[Output] = []
    inventory_lines = [
        "//! @generated exact shader source-owner module inventory.",
        "#![allow(dead_code)]", "",
    ]
    for index, owner in enumerate(sorted(shader_owners, key=lambda row: row["source_path"])):
        content = (upstream / owner["source_path"]).read_bytes()
        if sha256(content) != owner["source_sha256"]:
            raise ValueError(f"upstream shader source drift: {owner['source_path']}")
        target = repo / owner["target_path"]
        snapshot_name = f"{target.stem}.source"
        outputs.extend([
            Output(target_dir / "source" / snapshot_name, content),
            Output(target, target_content(owner, snapshot_name, content)),
        ])
        inventory_lines.extend([
            f'#[path = "shader-build-authority/{target.name}"]',
            f"mod shader_source_owner_{index:03d};", "",
        ])
    inventory = repo / "crates/nuxie-renderer/src/mechanical_port/backend_shader_authority_inventory.rs"
    outputs.append(Output(inventory, ("\n".join(inventory_lines)).encode()))
    return outputs


def main() -> int:
    args = parse_args()
    repo = args.repo_root.resolve()
    upstream = args.upstream_root.resolve()
    outputs = render(repo, upstream)
    stale = [output for output in outputs if not output.path.is_file() or output.path.read_bytes() != output.content]
    if args.check:
        if stale:
            print("backend shader authority translations are stale:", file=sys.stderr)
            for output in stale:
                print(f"- {output.path.relative_to(repo)}", file=sys.stderr)
            return 1
        print("backend shader authority translations clean: exact source owners")
        return 0
    for output in outputs:
        output.path.parent.mkdir(parents=True, exist_ok=True)
        output.path.write_bytes(output.content)
    print(f"wrote {len(outputs)} exact shader authority artifacts from translated source owners")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError) as error:
        print(f"backend shader authority generation failure: {error}", file=sys.stderr)
        raise SystemExit(1)
