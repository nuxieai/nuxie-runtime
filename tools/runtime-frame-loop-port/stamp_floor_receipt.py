#!/usr/bin/env python3
"""Copy a floor receipt with its immutable candidate SHA inside the file."""

from __future__ import annotations

import argparse
import os
import pathlib
import re
import subprocess
import tempfile


TREE_SHA_PREFIX = b"FLOOR_RECEIPT_TREE_SHA="
TREE_SHA_RE = re.compile(r"[0-9a-f]{40}")
EVIDENCE_DIR = pathlib.PurePosixPath("docs/runtime-frame-loop-fl-c5-evidence")


def tracked_floor_receipts(repo_root: pathlib.Path) -> list[pathlib.Path]:
    completed = subprocess.run(
        ["git", "ls-files", "-z", "--", EVIDENCE_DIR.as_posix()],
        cwd=repo_root,
        check=True,
        capture_output=True,
    )
    tracked = completed.stdout.decode("utf-8").split("\0")
    return sorted(
        repo_root / path
        for path in tracked
        if path
        and pathlib.PurePosixPath(path).name.startswith("floor")
        and pathlib.PurePosixPath(path).suffix == ".log"
    )


def validate_tracked_floor_receipts(repo_root: pathlib.Path) -> list[str]:
    errors = []
    for receipt in tracked_floor_receipts(repo_root):
        relative = receipt.relative_to(repo_root).as_posix()
        lines = receipt.read_bytes().splitlines()
        first_line = lines[0] if lines else b""
        if not first_line.startswith(TREE_SHA_PREFIX):
            errors.append(f"{relative}: missing receipt tree SHA")
            continue
        tree_sha_bytes = first_line.removeprefix(TREE_SHA_PREFIX)
        try:
            tree_sha = tree_sha_bytes.decode("ascii")
        except UnicodeDecodeError:
            errors.append(f"{relative}: malformed receipt tree SHA")
            continue
        if TREE_SHA_RE.fullmatch(tree_sha) is None:
            errors.append(f"{relative}: malformed receipt tree SHA")
            continue
        resolved = subprocess.run(
            ["git", "cat-file", "-e", f"{tree_sha}^{{commit}}"],
            cwd=repo_root,
            capture_output=True,
        )
        if resolved.returncode != 0:
            errors.append(f"{relative}: unresolvable receipt tree SHA {tree_sha}")
    return errors


def resolve_tree_sha(repo_root: pathlib.Path, requested: str | None) -> str:
    if requested is None:
        completed = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=repo_root,
            check=True,
            capture_output=True,
            text=True,
        )
        requested = completed.stdout.strip()
    if TREE_SHA_RE.fullmatch(requested) is None:
        raise ValueError("tree SHA must be exactly 40 lowercase hexadecimal characters")
    return requested


def stamp_receipt(
    source: pathlib.Path,
    destination: pathlib.Path,
    tree_sha: str,
) -> None:
    if TREE_SHA_RE.fullmatch(tree_sha) is None:
        raise ValueError("tree SHA must be exactly 40 lowercase hexadecimal characters")
    payload = source.read_bytes()
    first_line, separator, remainder = payload.partition(b"\n")
    if first_line.startswith(TREE_SHA_PREFIX):
        payload = remainder if separator else b""
    stamped = TREE_SHA_PREFIX + tree_sha.encode("ascii") + b"\n" + payload

    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary_path: pathlib.Path | None = None
    try:
        with tempfile.NamedTemporaryFile(
            dir=destination.parent,
            prefix=f".{destination.name}.",
            suffix=".stamp",
            delete=False,
        ) as temporary:
            temporary.write(stamped)
            temporary.flush()
            os.fsync(temporary.fileno())
            temporary_path = pathlib.Path(temporary.name)
        os.replace(temporary_path, destination)
        temporary_path = None
    finally:
        if temporary_path is not None and temporary_path.exists():
            temporary_path.unlink()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument("--source", type=pathlib.Path, required=True)
    parser.add_argument("--destination", type=pathlib.Path, required=True)
    parser.add_argument("--tree-sha")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = args.repo_root.resolve()
    tree_sha = resolve_tree_sha(repo_root, args.tree_sha)
    stamp_receipt(args.source.resolve(), args.destination.resolve(), tree_sha)
    print(f"stamped {args.destination} with tree SHA {tree_sha}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
