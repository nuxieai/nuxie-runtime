#!/usr/bin/env python3
"""Check runtime file coverage, not semantic parity or review completion.

The pinned upstream tree is the inventory. Header/implementation pairs share
one Rust owner; five upstream naming exceptions follow their Rust modules.
No generated ledger or stored completion count is required.
"""

import argparse
from pathlib import Path, PurePosixPath
import subprocess
import sys


OWNER_ROOT = Path("crates/nuxie-runtime/src/mechanical_port/source")
RENAMED_OWNERS = {
    "animation/state_machine_fire_event": "generated/animation/state_machine_fire_event",
    "nested_animation": "animation/nested_animation",
    "property_recorder": "animation/property_recorder",
    "shapes/shape_paint_path": "shapes/paint/shape_paint_path",
    "text_engine": "text/text_engine",
}


def upstream_owners(paths: list[str]) -> set[str]:
    owners = set()
    for path in paths:
        for prefix, extension in (("include/rive/", ".hpp"), ("src/", ".cpp")):
            if path.startswith(prefix) and path.endswith(extension):
                owners.add(str(PurePosixPath(path[len(prefix):]).with_suffix("")))
    return owners


def missing_owners(repo: Path, owners: set[str]) -> list[str]:
    missing = []
    for owner in sorted(owners):
        target = OWNER_ROOT / (RENAMED_OWNERS.get(owner, owner) + ".rs")
        if not (repo / target).is_file() or not (repo / target).read_text().strip():
            missing.append(f"{owner} -> {target}")
    return missing


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--upstream-root", type=Path, required=True)
    parser.add_argument("--upstream-ref", required=True)
    args = parser.parse_args()
    paths = subprocess.check_output(
        ["git", "-C", str(args.upstream_root), "ls-tree", "-r", "--name-only",
         args.upstream_ref, "--", "include/rive", "src"], text=True,
    ).splitlines()
    owners = upstream_owners(paths)
    if not owners:
        raise ValueError("pinned upstream contains no runtime header/source owners")
    missing = missing_owners(args.repo_root, owners)
    if missing:
        print("Missing Rust source owners:\n" + "\n".join(missing), file=sys.stderr)
        return 1
    print(f"Runtime source correspondence: {len(owners)} pinned owners have Rust files "
          "(structural coverage only; tests and review establish behavior).")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"Runtime source correspondence failed: {error}", file=sys.stderr)
        raise SystemExit(1)
