#!/usr/bin/env python3
"""Verify the archive-pinned shader tree before executing its build recipes."""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath


def verify_inputs(manifest_path: Path, source_root: Path, revision: str) -> dict:
    manifest = json.loads(manifest_path.read_text())
    if manifest.get("revision") != revision:
        raise ValueError("shader source revision does not match generator pin")
    expected = manifest.get("files")
    if not isinstance(expected, dict) or not expected:
        raise ValueError("shader source manifest is empty")
    for name, digest in expected.items():
        path = PurePosixPath(name)
        if (path.is_absolute() or ".." in path.parts
                or not name.startswith("renderer/src/shaders/")
                or not isinstance(digest, str) or len(digest) != 64):
            raise ValueError(f"invalid shader source manifest entry: {name}")
    actual = {
        path.relative_to(source_root).as_posix()
        for path in (source_root / "renderer/src/shaders").rglob("*")
        if path.is_file()
    }
    if actual != expected.keys():
        raise ValueError("shader source file set differs from pinned manifest")
    for name, digest in expected.items():
        if hashlib.sha256((source_root / name).read_bytes()).hexdigest() != digest:
            raise ValueError(f"shader source digest mismatch: {name}")
    return manifest


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--source-root", required=True, type=Path)
    parser.add_argument("--revision", required=True)
    args = parser.parse_args()
    verify_inputs(args.manifest, args.source_root, args.revision)
