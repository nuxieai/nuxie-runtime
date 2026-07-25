#!/usr/bin/env python3
"""Deterministic source fingerprint for Rust frame-loop trace evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import subprocess
from typing import Any


FINGERPRINT_SCHEMA = "nuxie-runtime-frame-loop-rust-source/v1"
RUST_RUNNER_PROVENANCE_SCHEMA = (
    "nuxie-runtime-frame-loop-rust-runner/v1"
)
RUST_RUNNER_PROVENANCE_SUFFIX = ".frame-loop-trace-provenance"
CANONICAL_TRACE_PATH = pathlib.PurePosixPath(
    "docs/runtime-frame-loop-trace.json"
)
LOCAL_FIXTURE_LINKS = {
    pathlib.PurePosixPath(f"fixtures/{name}")
    for name in ("animation", "flow", "graph", "minimal")
}


class SourceFingerprintError(RuntimeError):
    """Raised when the candidate source tree cannot be fingerprinted."""


def _candidate_paths(repo_root: pathlib.Path) -> list[bytes]:
    result = subprocess.run(
        [
            "git",
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ],
        cwd=repo_root,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise SourceFingerprintError(
            "cannot enumerate candidate source files: "
            + result.stderr.decode("utf-8", errors="replace").strip()
        )
    return sorted(path for path in result.stdout.split(b"\0") if path)


def _relative_evidence_path(
    repo_root: pathlib.Path, evidence_path: pathlib.Path
) -> pathlib.PurePosixPath | None:
    try:
        relative = evidence_path.resolve().relative_to(repo_root.resolve())
    except ValueError:
        return None
    return pathlib.PurePosixPath(relative.as_posix())


def _is_excluded(
    relative: pathlib.PurePosixPath,
    evidence_relative: pathlib.PurePosixPath | None,
) -> bool:
    if relative == CANONICAL_TRACE_PATH or relative == evidence_relative:
        return True
    # These four developer-only convenience links point at fixture directories
    # outside the repository. They are never candidate additions and must not
    # make local evidence differ from a clean/orchestrator checkout.
    if relative in LOCAL_FIXTURE_LINKS:
        return True
    if (
        "__pycache__" in relative.parts
        or relative.suffix in {".pyc", ".pyo", ".profraw", ".profdata"}
        or relative.name == ".DS_Store"
        or relative.parts[:1] == ("target",)
    ):
        return True
    return (
        len(relative.parts) > 1
        and relative.parts[0] == "docs"
        and relative.name.endswith("-status.md")
    )


def candidate_source_fingerprint(
    repo_root: pathlib.Path, *, evidence_path: pathlib.Path
) -> dict[str, Any]:
    """Hash the tracked and intended-untracked candidate source tree.

    Git's standard ignore rules remove generated build and trace artifacts.
    Trace output and mutable status documents are excluded explicitly so writing
    evidence about a candidate does not change the candidate's identity.
    """

    repo_root = repo_root.resolve()
    evidence_relative = _relative_evidence_path(repo_root, evidence_path)
    digest = hashlib.sha256()
    digest.update((FINGERPRINT_SCHEMA + "\0").encode())
    file_count = 0

    for raw_relative in _candidate_paths(repo_root):
        relative_text = os.fsdecode(raw_relative)
        relative = pathlib.PurePosixPath(relative_text)
        if _is_excluded(relative, evidence_relative):
            continue

        path = repo_root / pathlib.Path(relative_text)
        if path.is_symlink():
            kind = b"symlink"
            payload = os.fsencode(os.readlink(path))
            executable = b"0"
        elif path.is_file():
            kind = b"file"
            payload = path.read_bytes()
            executable = b"1" if path.stat().st_mode & 0o111 else b"0"
        elif path.exists():
            kind = b"other"
            payload = b""
            executable = b"0"
        else:
            # Deleted tracked files remain in `git ls-files`; retain the
            # deletion in the candidate identity.
            kind = b"missing"
            payload = b""
            executable = b"0"

        for value in (raw_relative, kind, executable, payload):
            digest.update(len(value).to_bytes(8, byteorder="big"))
            digest.update(value)
        file_count += 1

    return {
        "schema": FINGERPRINT_SCHEMA,
        "sha256": digest.hexdigest(),
        "file_count": file_count,
    }


def rust_runner_provenance(
    candidate_source: dict[str, Any],
) -> dict[str, Any]:
    """Describe the only Rust runner configuration accepted by trace capture."""

    return {
        "schema": RUST_RUNNER_PROVENANCE_SCHEMA,
        "candidate_source": candidate_source,
        "cargo_target_dir": "target/frame-loop-coverage",
        "package": "rust-golden-runner",
        "features": ["coverage-trace"],
        "rustflags": ["-Cinstrument-coverage"],
    }


def rust_runner_provenance_path(runner: pathlib.Path) -> pathlib.Path:
    return runner.with_name(runner.name + RUST_RUNNER_PROVENANCE_SUFFIX)


def require_rust_runner_provenance(
    runner: pathlib.Path, candidate_source: dict[str, Any]
) -> dict[str, Any]:
    """Require the runner stamp to match the current candidate exactly."""

    provenance_path = rust_runner_provenance_path(runner)
    try:
        actual = json.loads(provenance_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise SourceFingerprintError(
            f"cannot read Rust trace runner provenance {provenance_path}: {error}"
        ) from error
    expected = rust_runner_provenance(candidate_source)
    if actual != expected:
        raise SourceFingerprintError(
            "Rust trace runner provenance is stale; rebuild with "
            "`make runtime-frame-loop-trace-runners`"
        )
    return actual


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument("--evidence-path", type=pathlib.Path, required=True)
    parser.add_argument("--runner-provenance", action="store_true")
    args = parser.parse_args()
    candidate_source = candidate_source_fingerprint(
        args.repo_root.resolve(), evidence_path=args.evidence_path.resolve()
    )
    value = (
        rust_runner_provenance(candidate_source)
        if args.runner_provenance
        else candidate_source
    )
    print(json.dumps(value, sort_keys=True, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
