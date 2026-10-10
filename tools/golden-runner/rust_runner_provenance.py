#!/usr/bin/env python3
"""Bind staged Rust golden runners to their source and build configuration.

The gate consumes binaries from the conventional target directory. A stamp
binds each variant/profile binary hash to source digests, dependency pins,
Bazel targets, and the Rust toolchain. A matching stamp reuses the binary;
otherwise tools/bazel/runtime.py compiles the authored crates directly with
Bazel and publishes the fresh executable atomically.

Cargo metadata still discovers workspace members and the target directory.
Changed member digests clear any legacy Cargo artifacts in that directory,
so pre-migration artifacts cannot survive a source change. Bazel's own build
cache uses content fingerprints and does not depend on those Cargo mtimes.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

DIGEST_SCHEMA = "nuxie-golden-rust-sources/v1"
STAMP_SCHEMA = "nuxie-golden-rust-runner/v1"
RUNNER_PACKAGE = "rust-golden-runner"

VARIANTS = {
    "ordinary": {
        "features": [],
        "artifact": "rust-golden-runner-ordinary",
    },
    "scripted": {
        "features": ["scripting"],
        "artifact": "rust-golden-runner-scripted",
    },
}


class ProvenanceError(Exception):
    pass


def run(command, cwd, capture=False):
    result = subprocess.run(
        command,
        cwd=cwd,
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
        text=True,
    )
    if result.returncode != 0:
        detail = f": {result.stderr.strip()}" if capture and result.stderr else ""
        raise ProvenanceError(
            f"command failed ({result.returncode}): {' '.join(command)}{detail}"
        )
    return result.stdout if capture else None


def sha256_path(path: Path) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def workspace_members(repo_root: Path) -> dict[str, Path]:
    """Map package name -> source directory for every package whose sources
    live inside the repository: workspace members plus vendored/path
    dependencies (vendor/). Anything compiled from in-repo files can carry
    the poisoned-mtime state, so all of it participates in the digest."""
    # `--offline` keeps resolution pinned to Cargo.lock, but it also refuses to
    # reach the registry -- so a dependency added since the runner's cargo home
    # was last warmed makes metadata fail outright. Populate the cache from the
    # lockfile first (a no-op when it is already warm); `--locked` keeps this
    # from resolving anything the lockfile does not already name. Best-effort:
    # a workspace with no lockfile or no network still gets the sharper error
    # from the offline metadata call below.
    try:
        run(["cargo", "fetch", "--locked"], cwd=repo_root, capture=True)
    except ProvenanceError:
        pass
    stdout = run(
        ["cargo", "metadata", "--format-version", "1", "--offline"],
        cwd=repo_root,
        capture=True,
    )
    metadata = json.loads(stdout)
    members = {}
    for package in metadata["packages"]:
        directory = Path(package["manifest_path"]).parent
        try:
            members[package["name"]] = directory.relative_to(repo_root)
        except ValueError:
            continue  # registry dependency outside the repository
    if RUNNER_PACKAGE not in members:
        raise ProvenanceError(f"workspace has no member named {RUNNER_PACKAGE}")
    return members


def cargo_target_directory(repo_root: Path) -> Path:
    """Use Cargo's resolution of CARGO_TARGET_DIR and .cargo configuration.

    Running metadata from repo_root also gives relative environment paths the
    same base as the clean/build commands below.
    """
    metadata = json.loads(
        run(
            ["cargo", "metadata", "--format-version", "1", "--no-deps", "--offline"],
            cwd=repo_root,
            capture=True,
        )
    )
    return (repo_root / Path(metadata["target_directory"])).resolve()


def member_digest(repo_root: Path, directory: Path) -> str:
    """Digest of every non-hidden file under the member directory.

    Hidden files and target/ trees are excluded so editor droppings and
    build output do not churn the digest; everything else participates so a
    rewritten source can never be invisible to the gate.
    """
    entries = []
    base = repo_root / directory
    for current, directories, files in os.walk(base):
        directories[:] = sorted(
            d for d in directories if d != "target" and not d.startswith(".")
        )
        for name in sorted(files):
            if name.startswith("."):
                continue
            path = Path(current) / name
            entries.append(
                f"{path.relative_to(base)}\0{sha256_path(path)}"
            )
    digest = hashlib.sha256()
    for entry in entries:
        digest.update(entry.encode())
        digest.update(b"\n")
    return digest.hexdigest()


def current_digest_state(repo_root: Path, members: dict[str, Path]) -> dict:
    workspace = hashlib.sha256()
    inputs = {repo_root / name for name in ("Cargo.toml", "Cargo.lock", "MODULE.bazel", "MODULE.bazel.lock", ".bazelversion", ".bazelrc", "BUILD.bazel")}
    inputs.update(path for directory in (repo_root / "bazel", repo_root / "tools/bazel")
                  for path in directory.rglob("*") if path.is_file() and "__pycache__" not in path.parts)
    for path in sorted(inputs):
        if path.is_file():
            workspace.update(str(path.relative_to(repo_root)).encode())
            workspace.update(sha256_path(path).encode())
    return {
        "schema": DIGEST_SCHEMA,
        "rustc": run(["rustc", "--version"], cwd=repo_root, capture=True).strip(),
        "workspace": workspace.hexdigest(),
        "members": {
            name: member_digest(repo_root, directory)
            for name, directory in sorted(members.items())
        },
    }


def digest_state_id(state: dict) -> str:
    return hashlib.sha256(
        json.dumps(state, sort_keys=True).encode()
    ).hexdigest()


def load_json(path: Path):
    try:
        with open(path, encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, json.JSONDecodeError):
        return None


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    handle = tempfile.NamedTemporaryFile(
        "w", dir=path.parent, prefix=path.name, suffix=".tmp", delete=False
    )
    try:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")
        handle.close()
        os.replace(handle.name, path)
    finally:
        if os.path.exists(handle.name):
            os.unlink(handle.name)


def changed_members(state: dict, recorded) -> list[str]:
    """Members whose digests differ from the recorded state.

    A missing, corrupt, or schema/toolchain/workspace-level mismatch keeps
    this fail-closed: every member counts as changed.
    """
    if not isinstance(recorded, dict):
        return sorted(state["members"])
    for key in ("schema", "rustc", "workspace"):
        if recorded.get(key) != state[key]:
            return sorted(state["members"])
    recorded_members = recorded.get("members")
    if not isinstance(recorded_members, dict):
        return sorted(state["members"])
    return sorted(
        name
        for name, digest in state["members"].items()
        if recorded_members.get(name) != digest
    )


def restore_sources_invariant(
    repo_root: Path, members: dict[str, Path], state: dict, digests_path: Path
) -> list[str]:
    changed = changed_members(state, load_json(digests_path))
    for name in changed:
        run(["cargo", "clean", "-p", name], cwd=repo_root, capture=True)
    return changed


def verify_quiescent(repo_root: Path, members: dict[str, Path], state: dict) -> None:
    if current_digest_state(repo_root, members) != state:
        raise ProvenanceError(
            "workspace sources changed while the golden gate build was running; "
            "re-run the gate once the tree is quiescent"
        )


def ensure_sources(repo_root: Path) -> None:
    repo_root = repo_root.resolve()
    members = workspace_members(repo_root)
    target_dir = cargo_target_directory(repo_root)
    state = current_digest_state(repo_root, members)
    digests_path = target_dir / "golden-gate/rust-sources.json"
    changed = restore_sources_invariant(repo_root, members, state, digests_path)
    verify_quiescent(repo_root, members, state)
    write_json(digests_path, state)
    if changed:
        print(
            "rust source provenance: invalidated stale artifacts for "
            f"{len(changed)} member(s): {', '.join(changed)}"
        )
    else:
        print("rust source provenance: workspace artifacts verified")


def ensure_runner(repo_root: Path, variant: str, profile: str) -> None:
    repo_root = repo_root.resolve()
    settings = VARIANTS[variant]
    members = workspace_members(repo_root)
    target_dir = cargo_target_directory(repo_root)
    state = current_digest_state(repo_root, members)
    state_id = digest_state_id(state)

    digests_path = target_dir / "golden-gate/rust-sources.json"
    stamp_path = target_dir / f"golden-gate/{variant}-{profile}.json"
    uplift = target_dir / profile / RUNNER_PACKAGE
    artifact = target_dir / profile / settings["artifact"]

    changed = restore_sources_invariant(repo_root, members, state, digests_path)

    stamp = load_json(stamp_path)
    reusable = (
        not changed
        and isinstance(stamp, dict)
        and stamp.get("schema") == STAMP_SCHEMA
        and stamp.get("digest_state") == state_id
        and artifact.is_file()
        and sha256_path(artifact) == stamp.get("binary_sha256")
    )
    if reusable:
        # The shared uplift path is scratch: the other variant's build
        # legitimately overwrites it. Consumers of this variant read the
        # stamped copy or the uplift path, so put the verified binary back.
        if variant == "ordinary" and (
            not uplift.is_file() or sha256_path(uplift) != stamp["binary_sha256"]
        ):
            run(["cp", str(artifact), str(uplift)], cwd=repo_root)
        print(
            f"rust runner provenance: reusing verified {variant} runner "
            f"({stamp['binary_sha256'][:16]})"
        )
        return

    command = [*runner_build_command(repo_root), "build", "--quiet", "-p", RUNNER_PACKAGE]
    if profile == "release":
        command.append("--release")
    for feature in settings["features"]:
        command.extend(["--features", feature])
    uplift.unlink(missing_ok=True)
    run(command, cwd=repo_root)
    if not uplift.is_file():
        raise ProvenanceError(f"Bazel build produced no runner at {uplift}")
    run(["cp", str(uplift), str(artifact)], cwd=repo_root)

    verify_quiescent(repo_root, members, state)
    write_json(digests_path, state)
    write_json(
        stamp_path,
        {
            "schema": STAMP_SCHEMA,
            "variant": variant,
            "profile": profile,
            "digest_state": state_id,
            "rustc": state["rustc"],
            "binary_sha256": sha256_path(artifact),
        },
    )
    print(
        f"rust runner provenance: rebuilt {variant} runner"
        + (f" after invalidating {', '.join(changed)}" if changed else "")
    )


def runner_build_command(repo_root: Path) -> list[str]:
    """Select the compiler entrypoint separately from the content stamp guard."""
    return [str(repo_root / "tools/bazel/runtime.py")]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parent.parent.parent,
    )
    subcommands = parser.add_subparsers(dest="command", required=True)
    ensure = subcommands.add_parser(
        "ensure", help="verify or rebuild one runner variant"
    )
    ensure.add_argument("--variant", choices=sorted(VARIANTS), required=True)
    ensure.add_argument("--profile", choices=["debug", "release"], required=True)
    subcommands.add_parser(
        "ensure-sources",
        help="verify workspace artifacts against source content only",
    )
    arguments = parser.parse_args()

    try:
        if arguments.command == "ensure":
            ensure_runner(
                arguments.repo_root.resolve(), arguments.variant, arguments.profile
            )
        else:
            ensure_sources(arguments.repo_root.resolve())
    except ProvenanceError as error:
        print(f"rust runner provenance: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
