#!/usr/bin/env python3
"""Freeze the current Rust-WGPU implementation that must be retired at cutover."""

from __future__ import annotations

import argparse
import hashlib
import re
import sys
from dataclasses import dataclass
from pathlib import Path


HEADER = (
    "source_path",
    "source_sha256",
    "evidence_kind",
    "evidence_count",
    "cutover_disposition",
    "cutover_status",
)
DIRECT = re.compile(r"\b(?:wgpu|Wgpu|WGPU|WebGPU|rust-wgpu|WGSL)\b")


@dataclass(frozen=True, order=True)
class Row:
    source_path: str
    source_sha256: str
    evidence_kind: str
    evidence_count: int
    cutover_disposition: str
    cutover_status: str = "pending"

    def tsv(self) -> str:
        return "\t".join(str(getattr(self, column)) for column in HEADER)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    return parser.parse_args()


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def render(repo_root: Path) -> str:
    source_root = repo_root / "crates/nuxie-renderer/src"
    rows: list[Row] = []
    for path in sorted(item for item in source_root.rglob("*") if item.is_file()):
        relative = path.relative_to(repo_root).as_posix()
        if "/native_metal/" in relative or "/mechanical_port/" in relative:
            continue
        if path.suffix == ".wgsl":
            rows.append(Row(relative, digest(path), "wgsl-artifact", 1, "delete-or-replace"))
            continue
        if path.suffix != ".rs":
            continue
        matches = DIRECT.findall(path.read_text(errors="replace"))
        if matches:
            rows.append(
                Row(relative, digest(path), "direct-rust-wgpu-owner-or-callsite", len(matches), "delete-or-replace")
            )
    if not rows:
        raise ValueError("legacy Rust-WGPU inventory is empty")
    return "\n".join(("\t".join(HEADER), *(row.tsv() for row in rows))) + "\n"


def main() -> int:
    args = parse_args()
    output = args.output if args.output.is_absolute() else args.repo_root / args.output
    rendered = render(args.repo_root)
    count = len(rendered.splitlines()) - 1
    if args.check:
        if not output.is_file() or output.read_text() != rendered:
            print("legacy Rust-WGPU inventory is stale", file=sys.stderr)
            return 1
        print(f"legacy Rust-WGPU inventory clean: {count} files")
        return 0
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(rendered)
    print(f"wrote {count} legacy Rust-WGPU rows to {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
