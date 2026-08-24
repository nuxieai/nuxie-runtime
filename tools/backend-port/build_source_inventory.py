#!/usr/bin/env python3
"""Build the exact pinned source-candidate inventory for future GPU ports."""

from __future__ import annotations

import argparse
import hashlib
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True, order=True)
class SourceRow:
    campaign: str
    source_kind: str
    source_path: str
    source_sha256: str
    classification_status: str = "ownership-ledger"

    def tsv(self) -> str:
        return "\t".join(
            (
                self.campaign,
                self.source_kind,
                self.source_path,
                self.source_sha256,
                self.classification_status,
            )
        )


HEADER = "\t".join(
    (
        "campaign",
        "source_kind",
        "source_path",
        "source_sha256",
        "classification_status",
    )
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--upstream-root", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    return parser.parse_args()


def git_output(root: Path, *args: str) -> str:
    return subprocess.run(
        ["git", "-C", str(root), *args],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def classify(path: Path) -> str:
    name = path.name
    suffix = path.suffix.lower()
    if suffix in {".hpp", ".h"}:
        return "header"
    if suffix in {".cpp", ".c"}:
        return "implementation"
    if suffix == ".mm":
        return "objective-cpp-implementation"
    if suffix in {".glsl", ".vert", ".frag", ".main"}:
        return "shader-input"
    if suffix == ".py":
        return "generator"
    if suffix in {".lua", ".js"} or name == "Makefile" or name.startswith("make_"):
        return "build-script"
    if suffix == ".md" or name.startswith("."):
        return "nonsemantic-support"
    raise ValueError(f"unclassified source kind for {path}")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def collect_tree(upstream_root: Path, relative_root: str) -> list[Path]:
    root = upstream_root / relative_root
    if not root.is_dir():
        raise FileNotFoundError(f"missing source root: {relative_root}")
    return sorted(
        path
        for path in root.rglob("*")
        if path.is_file()
        and "__pycache__" not in path.parts
        and path.suffix.lower() != ".pyc"
    )


def add_row(
    rows_by_path: dict[str, SourceRow],
    upstream_root: Path,
    campaign: str,
    path: Path,
) -> None:
    if not path.is_file():
        raise FileNotFoundError(f"missing source: {path.relative_to(upstream_root)}")
    relative = path.relative_to(upstream_root).as_posix()
    row = SourceRow(campaign, classify(path), relative, digest(path))
    previous = rows_by_path.get(relative)
    if previous is not None and previous.campaign != campaign:
        raise ValueError(
            f"source belongs to multiple campaigns: {relative}: "
            f"{previous.campaign}, {campaign}"
        )
    rows_by_path[relative] = row


def render_inventory(manifest: dict, upstream_root: Path) -> str:
    expected_ref = manifest["upstream_ref"]
    actual_ref = git_output(upstream_root, "rev-parse", "HEAD")
    if actual_ref != expected_ref:
        raise ValueError(f"upstream drift: expected {expected_ref}, got {actual_ref}")
    if git_output(upstream_root, "status", "--porcelain", "--untracked-files=no"):
        raise ValueError("upstream checkout has tracked changes")

    rows_by_path: dict[str, SourceRow] = {}
    for backend in manifest["backend"]:
        campaign = backend["id"]
        for relative_root in backend["source_roots"]:
            for path in collect_tree(upstream_root, relative_root):
                add_row(rows_by_path, upstream_root, campaign, path)
        for relative in backend["extra_sources"]:
            add_row(rows_by_path, upstream_root, campaign, upstream_root / relative)

    for shared in manifest["shared_source_set"]:
        campaign = shared["id"]
        for relative_root in shared["roots"]:
            for path in collect_tree(upstream_root, relative_root):
                add_row(rows_by_path, upstream_root, campaign, path)
        top_level_root = upstream_root / shared["top_level_root"]
        extensions = {f".{extension}" for extension in shared["top_level_extensions"]}
        for path in sorted(top_level_root.iterdir()):
            if path.is_file() and path.suffix.lower() in extensions:
                add_row(rows_by_path, upstream_root, campaign, path)
        for relative in shared["extra_sources"]:
            add_row(rows_by_path, upstream_root, campaign, upstream_root / relative)

    rows = sorted(rows_by_path.values())
    return "\n".join([HEADER, *(row.tsv() for row in rows)]) + "\n"


def main() -> int:
    args = parse_args()
    manifest_path = args.manifest
    if not manifest_path.is_absolute():
        manifest_path = args.repo_root / manifest_path
    output_path = args.output
    if not output_path.is_absolute():
        output_path = args.repo_root / output_path

    manifest = tomllib.loads(manifest_path.read_text())
    rendered = render_inventory(manifest, args.upstream_root.resolve())
    if args.check:
        if not output_path.is_file():
            print(f"missing generated inventory: {output_path}", file=sys.stderr)
            return 1
        current = output_path.read_text()
        if current != rendered:
            print(
                "backend source inventory is stale; regenerate with "
                "tools/backend-port/build_source_inventory.py",
                file=sys.stderr,
            )
            return 1
        print(f"backend source inventory clean: {len(rendered.splitlines()) - 1} rows")
        return 0

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(rendered)
    print(f"wrote {len(rendered.splitlines()) - 1} rows to {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
