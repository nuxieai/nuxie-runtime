#!/usr/bin/env python3
"""Apply a digest-guarded patch to upstream rive-runtime sources.

A patch here is a unified diff, which `git apply` also reads, whose preamble
names the SHA-256 of each file it changes before and after:

    upstream-sha256 <digest> <path>
    patched-sha256 <digest> <path>

A file whose digest is not the upstream one is refused, never applied with
fuzz or offsets. Hunks apply at their stated lines with exact context, and the
result must have the patched digest.

usage: apply_patch.py --root <upstream tree> <patch>
"""
from __future__ import annotations

import argparse
from dataclasses import dataclass, field
import hashlib
from pathlib import Path, PurePosixPath
import re
import stat
import sys

DIGEST = re.compile(rb"^(upstream|patched)-sha256 ([0-9a-f]{64}) (\S+)$")
HUNK = re.compile(rb"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")


@dataclass
class Hunk:
    old_start: int
    old: list[bytes]
    new: list[bytes]


@dataclass
class FilePatch:
    path: str
    upstream_sha256: str = ""
    patched_sha256: str = ""
    hunks: list[Hunk] = field(default_factory=list)


def sha256(content: bytes) -> str:
    return hashlib.sha256(content).hexdigest()


def relative_path(name: bytes, patch: Path) -> str:
    path = name.decode()
    pure = PurePosixPath(path)
    if not path or pure.is_absolute() or ".." in pure.parts:
        raise ValueError(f"{patch}: unsafe path {path!r}")
    return path


def load(patch: Path) -> dict[str, FilePatch]:
    """Every file the patch changes, with its digests and hunks."""
    lines = patch.read_bytes().split(b"\n")
    if lines and lines[-1] == b"":
        lines.pop()
    files: dict[str, FilePatch] = {}
    index = 0
    # The preamble runs to the first file header; only digest lines matter.
    while index < len(lines) and not lines[index].startswith((b"diff --git ", b"--- ")):
        match = DIGEST.match(lines[index])
        if match:
            path = relative_path(match[3], patch)
            entry = files.setdefault(path, FilePatch(path))
            slot = match[1].decode() + "_sha256"
            if getattr(entry, slot):
                raise ValueError(f"{patch}: two {slot} lines for {path}")
            setattr(entry, slot, match[2].decode())
        index += 1
    diffed = set()
    current = None
    while index < len(lines):
        line = lines[index]
        if line.startswith((b"diff --git ", b"index ")):
            index += 1
        elif line.startswith(b"--- "):
            following = lines[index + 1] if index + 1 < len(lines) else b""
            if not (line.startswith(b"--- a/") and following.startswith(b"+++ b/") and line[6:] == following[6:]):
                raise ValueError(f"{patch}: expected matching --- a/ and +++ b/ headers at line {index + 1}")
            path = relative_path(line[6:], patch)
            if path not in files or path in diffed:
                raise ValueError(f"{patch}: {path} needs one diff and its two digests")
            diffed.add(path)
            current = files[path]
            index += 2
        elif line.startswith(b"@@ "):
            match = HUNK.match(line)
            if not match or current is None:
                raise ValueError(f"{patch}: malformed hunk header at line {index + 1}")
            old_count = int(match[2] or b"1")
            new_count = int(match[4] or b"1")
            old: list[bytes] = []
            new: list[bytes] = []
            index += 1
            while len(old) < old_count or len(new) < new_count:
                if index >= len(lines) or lines[index][:1] not in (b" ", b"-", b"+"):
                    raise ValueError(f"{patch}: hunk at line {index} ends before its counts")
                if lines[index][:1] in (b" ", b"-"):
                    old.append(lines[index][1:])
                if lines[index][:1] in (b" ", b"+"):
                    new.append(lines[index][1:])
                index += 1
            if len(old) != old_count or len(new) != new_count:
                raise ValueError(f"{patch}: hunk before line {index + 1} overruns its counts")
            current.hunks.append(Hunk(int(match[1]), old, new))
        else:
            raise ValueError(f"{patch}: unexpected line {index + 1}: {line[:60]!r}")
    for entry in files.values():
        if not (entry.upstream_sha256 and entry.patched_sha256 and entry.path in diffed and entry.hunks):
            raise ValueError(f"{patch}: {entry.path} needs one diff and its two digests")
    return files


def apply_bytes(entry: FilePatch, content: bytes) -> bytes:
    """`content` patched, refusing any input but the recorded upstream file."""
    actual = sha256(content)
    if actual != entry.upstream_sha256:
        raise ValueError(
            f"refusing {entry.path}: its sha256 is {actual}, and this patch applies only "
            f"to upstream's {entry.upstream_sha256}"
        )
    lines = content.split(b"\n")
    result: list[bytes] = []
    position = 0
    for hunk in entry.hunks:
        start = hunk.old_start - 1 if hunk.old else hunk.old_start
        if start < position or lines[start:start + len(hunk.old)] != hunk.old:
            raise ValueError(f"{entry.path}: hunk at line {hunk.old_start} does not match exactly")
        result.extend(lines[position:start])
        result.extend(hunk.new)
        position = start + len(hunk.old)
    result.extend(lines[position:])
    patched = b"\n".join(result)
    if sha256(patched) != entry.patched_sha256:
        raise ValueError(
            f"{entry.path}: the patched sha256 is {sha256(patched)}, not the recorded "
            f"{entry.patched_sha256}"
        )
    return patched


def apply_tree(patch: Path, root: Path) -> list[str]:
    """Patch each file under `root`, a copy of upstream's tree, in place; the
    patched paths."""
    files = load(patch)
    for entry in files.values():
        target = root / entry.path
        patched = apply_bytes(entry, target.read_bytes())
        # A copy of read-only build inputs keeps their mode.
        target.chmod(target.stat().st_mode | stat.S_IWUSR)
        target.write_bytes(patched)
    return sorted(files)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("patch", type=Path)
    args = parser.parse_args()
    try:
        for path in apply_tree(args.patch, args.root):
            print(f"patched {path} with {args.patch.name}")
    except (OSError, ValueError) as error:
        print(f"rive-runtime patch failure: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
