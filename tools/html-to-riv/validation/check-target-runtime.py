#!/usr/bin/env python3
"""Reject compiler changes outside its module against the fixed runtime target.

This is a source-identity gate, not a substitute for recording the actual native
build/dependency graph and testing files on that build.
"""
import json
from pathlib import Path
import subprocess
import sys

BASELINE = "6c7ac16617835b5f581784ff08a9e779bb52faf3"
MODULE = "tools/html-to-riv/"


def git(root, *args):
    return subprocess.check_output(
        ["git", "-c", "core.filemode=true", *args], cwd=root
    ).decode("utf-8", errors="surrogateescape")


def audit(root, baseline=BASELINE):
    # Check index as well: an unstaged undo must not conceal a protected change
    # that would still be included by `git commit`.
    layers = {}
    for label, options in [("index", ["--cached"]), ("worktree", [])]:
        paths = git(root, "diff", *options, "--name-only", "--no-renames", "-z", baseline, "--").split("\0")
        layers[label] = sorted(p for p in paths if p and not p.startswith(MODULE))
    untracked = git(root, "ls-files", "--others", "--exclude-standard", "-z").split("\0")
    layers["untracked"] = sorted(p for p in untracked if p and not p.startswith(MODULE))
    return {
        "baseline": baseline,
        "baselineTree": git(root, "rev-parse", baseline + "^{tree}").strip(),
        "status": "pass" if not any(layers.values()) else "fail",
        "changesOutsideCompiler": layers,
        "scope": "Tracked source, index and nonignored additions; build resolution must be verified separately",
    }


if __name__ == "__main__":
    root = Path(__file__).resolve().parents[3]
    try:
        receipt = audit(root)
    except subprocess.CalledProcessError as error:
        sys.exit(f"Cannot verify immutable runtime: git exited {error.returncode}")
    print(json.dumps(receipt, indent=2))
    sys.exit(0 if receipt["status"] == "pass" else 1)
