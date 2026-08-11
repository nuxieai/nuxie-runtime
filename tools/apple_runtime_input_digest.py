#!/usr/bin/env python3
"""Compute the audited functional-input identity of the Apple runtime distribution."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import re
import subprocess
import sys
import tomllib
from collections.abc import Iterable, Mapping, Sequence
from typing import Any


SCHEMA_VERSION = 1
ROOT_PACKAGE = "nux-apple-product-extension"
DEFAULT_FEATURES = ("apple-runtime",)
DEFAULT_TARGETS = (
    "aarch64-apple-ios",
    "aarch64-apple-ios-sim",
    "x86_64-apple-ios",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
)
PACKAGING_INPUTS = (
    "LICENSE",
    "THIRD_PARTY_NOTICES.md",
    "crates/nux-capi/abi-layout-v3.json",
    "crates/nux-capi/exports-v3-apple-metal-extension.txt",
    "crates/nux-capi/exports-v3-portable.txt",
    "crates/nux-capi/include/nux_capi.generated.h",
    "crates/nux-capi/include/nux_capi.h",
    "crates/nux-capi/include/nux_capi_apple.h",
    "crates/nux-capi/size-baseline-apple-runtime-v0.4.0.json",
    "crates/nux-capi/size-budgets-v3.json",
    "crates/nux-capi/smoke/distribution_consumer.c",
    "crates/nux-capi/smoke/distribution_consumer.swift",
    "crates/nux-capi/smoke/capi_metal_smoke.c",
    "crates/nux-capi/smoke/capi_metal_smoke.swift",
    "crates/nux-capi/smoke/composed_script_asset.riv.base64",
    "crates/nux-apple-product-extension/exports-v1-product-extension.txt",
    "crates/nux-apple-product-extension/include/module.modulemap",
    "crates/nux-apple-product-extension/include/nux_product_extension.h",
    "crates/nux-apple-product-extension/smoke/product_extension_consumer.swift",
    "tools/apple_runtime_contract.py",
    "tools/apple_runtime_input_digest.py",
    "tools/build-nux-capi-xcframeworks.sh",
    "tools/check-nux-capi-surface.py",
    "tools/check-nux-capi-layout.py",
    "tools/json-scalar.py",
    "tools/publish-nux-capi-release.sh",
    "tools/verify-nux-capi-xcframeworks.sh",
)
EXCLUDED_DIRECTORY_NAMES = {".git", "target", "__pycache__"}
NON_BUILD_PACKAGE_DIRECTORIES = {".github", "benches", "docs", "examples", "smoke", "tests"}
NON_BUILD_PACKAGE_FILE_PREFIXES = ("CHANGELOG", "CONTRIBUTING", "LICENSE", "README")
BUILD_ENVIRONMENT_KEYS = {
    "AR",
    "ARFLAGS",
    "CC",
    "CFLAGS",
    "CPPFLAGS",
    "CPATH",
    "C_INCLUDE_PATH",
    "CXX",
    "CXXFLAGS",
    "LD",
    "LDFLAGS",
    "LIBRARY_PATH",
    "OBJC_INCLUDE_PATH",
    "PKG_CONFIG_PATH",
    "RANLIB",
    "RUSTC",
    "RUSTC_BOOTSTRAP",
    "RUSTDOCFLAGS",
    "RUSTC_LINKER",
    "RUSTDOC",
    "RUSTC_WRAPPER",
    "RUSTC_WORKSPACE_WRAPPER",
    "RUSTFLAGS",
    "SOURCE_DATE_EPOCH",
    "SDKROOT",
    "STRIP",
    "CARGO_BUILD_RUSTFLAGS",
    "CARGO_BUILD_RUSTC",
    "CARGO_BUILD_RUSTC_WRAPPER",
    "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
    "CARGO_BUILD_INCREMENTAL",
    "CARGO_BUILD_RUSTDOC",
    "CARGO_ENCODED_RUSTFLAGS",
    "CARGO_INCREMENTAL",
}


class InputDigestError(ValueError):
    pass


_CFG_TOKEN = re.compile(r'\s*([A-Za-z_][A-Za-z0-9_]*|=|\(|\)|,|"(?:[^"\\]|\\.)*")')


def _target_matches(
    expression: str | None,
    target: str,
    target_cfg: set[tuple[str, str | None]],
) -> bool:
    if expression is None:
        return True
    if not expression.startswith("cfg("):
        return expression == target
    tokens = [match.group(1) for match in _CFG_TOKEN.finditer(expression)]
    if "".join(tokens) != re.sub(r"\s+", "", expression):
        raise InputDigestError(f"cannot tokenize Cargo target expression: {expression}")
    position = 0

    def take(expected: str | None = None) -> str:
        nonlocal position
        if position >= len(tokens):
            raise InputDigestError(f"truncated Cargo target expression: {expression}")
        token = tokens[position]
        if expected is not None and token != expected:
            raise InputDigestError(
                f"expected {expected!r} in Cargo target expression: {expression}"
            )
        position += 1
        return token

    def evaluate() -> bool:
        name = take()
        if position < len(tokens) and tokens[position] == "(":
            take("(")
            values: list[bool] = []
            if position < len(tokens) and tokens[position] != ")":
                while True:
                    values.append(evaluate())
                    if position >= len(tokens) or tokens[position] != ",":
                        break
                    take(",")
            take(")")
            if name == "all":
                return all(values)
            if name == "any":
                return any(values)
            if name == "not" and len(values) == 1:
                return not values[0]
            raise InputDigestError(f"unsupported Cargo cfg operator in: {expression}")
        if position < len(tokens) and tokens[position] == "=":
            take("=")
            encoded_value = take()
            if not encoded_value.startswith('"'):
                raise InputDigestError(f"Cargo cfg value is not quoted: {expression}")
            value = json.loads(encoded_value)
            return (name, value) in target_cfg
        return (name, None) in target_cfg

    take("cfg")
    take("(")
    result = evaluate()
    take(")")
    if position != len(tokens):
        raise InputDigestError(f"trailing Cargo target expression tokens: {expression}")
    return result


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _canonical_json(document: object) -> bytes:
    return (json.dumps(document, sort_keys=True, separators=(",", ":")) + "\n").encode()


def _relative_path(repo_root: pathlib.Path, path: pathlib.Path) -> str:
    try:
        return path.resolve().relative_to(repo_root.resolve()).as_posix()
    except ValueError as error:
        raise InputDigestError(f"local dependency escapes the audited repository: {path}") from error


def _read_file(repo_root: pathlib.Path, relative_path: str, kind: str) -> dict[str, str]:
    path = repo_root / relative_path
    if not path.is_file():
        raise InputDigestError(f"required {kind} input is missing: {relative_path}")
    if path.is_symlink():
        raise InputDigestError(f"audited input must not be a symlink: {relative_path}")
    return {
        "kind": kind,
        "path": relative_path,
        "sha256": _sha256(path.read_bytes()),
    }


def _package_files(repo_root: pathlib.Path, package_root: pathlib.Path) -> Iterable[str]:
    for path in sorted(package_root.rglob("*")):
        if path.is_symlink():
            raise InputDigestError(f"audited package input must not be a symlink: {path}")
        if not path.is_file():
            continue
        relative_to_package = path.relative_to(package_root)
        if any(part in EXCLUDED_DIRECTORY_NAMES for part in relative_to_package.parts):
            continue
        if relative_to_package.parts[0] in NON_BUILD_PACKAGE_DIRECTORIES:
            continue
        if (
            len(relative_to_package.parts) == 1
            and relative_to_package.name.upper().startswith(
                NON_BUILD_PACKAGE_FILE_PREFIXES
            )
        ):
            continue
        yield _relative_path(repo_root, path)


def _directory_content_hash(root: pathlib.Path) -> str:
    if root.is_symlink() or not root.is_dir():
        raise InputDigestError(f"resolved source root must be a regular directory: {root}")
    records: list[dict[str, str]] = []
    for path in sorted(root.rglob("*")):
        if path.is_symlink():
            raise InputDigestError(f"resolved source input must not be a symlink: {path}")
        if not path.is_file():
            continue
        relative = path.relative_to(root)
        if any(part in EXCLUDED_DIRECTORY_NAMES for part in relative.parts):
            continue
        if relative.as_posix() == ".cargo-ok":
            continue
        records.append({"path": relative.as_posix(), "sha256": _sha256(path.read_bytes())})
    if not records:
        raise InputDigestError(f"resolved source root contains no audited files: {root}")
    return _sha256(_canonical_json(records))


def _directory_direct_files_hash(root: pathlib.Path) -> str:
    if root.is_symlink() or not root.is_dir():
        raise InputDigestError(f"toolchain library root must be a regular directory: {root}")
    records: list[dict[str, str]] = []
    for path in sorted(root.iterdir()):
        if path.is_symlink():
            raise InputDigestError(f"toolchain library must not be a symlink: {path}")
        if path.is_file():
            records.append({"path": path.name, "sha256": _sha256(path.read_bytes())})
    if not records:
        raise InputDigestError(f"toolchain library root contains no files: {root}")
    return _sha256(_canonical_json(records))


def _reachable_package_ids(
    metadata: Mapping[str, Any],
    root_package: str,
    target: str,
    target_cfg: set[tuple[str, str | None]],
) -> set[str]:
    packages = metadata.get("packages")
    resolve = metadata.get("resolve")
    if not isinstance(packages, list) or not isinstance(resolve, dict):
        raise InputDigestError("Cargo metadata is missing packages or resolve")
    roots = [package for package in packages if package.get("name") == root_package]
    if len(roots) != 1:
        raise InputDigestError(
            f"expected exactly one {root_package!r} package, found {len(roots)}"
        )
    nodes = resolve.get("nodes")
    if not isinstance(nodes, list):
        raise InputDigestError("Cargo metadata resolve graph has no nodes")
    nodes_by_id = {node["id"]: node for node in nodes}
    pending = [roots[0]["id"]]
    reachable: set[str] = set()
    while pending:
        package_id = pending.pop()
        if package_id in reachable:
            continue
        node = nodes_by_id.get(package_id)
        if node is None:
            raise InputDigestError(f"Cargo resolve graph is missing node {package_id}")
        reachable.add(package_id)
        for dependency in node.get("deps", []):
            dependency_kinds = dependency.get("dep_kinds", [])
            if dependency_kinds and not any(
                dependency_kind.get("kind") != "dev"
                and _target_matches(dependency_kind.get("target"), target, target_cfg)
                for dependency_kind in dependency_kinds
            ):
                continue
            pending.append(dependency["pkg"])
    return reachable


def _lock_packages(repo_root: pathlib.Path) -> dict[tuple[str, str, str | None], dict[str, object]]:
    try:
        lockfile = tomllib.loads((repo_root / "Cargo.lock").read_text(encoding="utf-8"))
    except (OSError, UnicodeError, tomllib.TOMLDecodeError) as error:
        raise InputDigestError(f"cannot parse the exact Cargo.lock: {error}") from error
    packages: dict[tuple[str, str, str | None], dict[str, object]] = {}
    for package in lockfile.get("package", []):
        source = package.get("source")
        packages[(package["name"], package["version"], source)] = package
    return packages


def _lock_entry_projection(lock_entry: Mapping[str, object] | None) -> dict[str, object] | None:
    if lock_entry is None:
        return None
    # Dependency edges and selected features come from Cargo's target-specific
    # resolved graph below. Keeping them out of this projection prevents an
    # unused/dev-only lock edge from invalidating an otherwise identical build.
    return {
        key: lock_entry[key]
        for key in ("name", "version", "source", "checksum")
        if key in lock_entry
    }


def _cargo_config_paths(repo_root: pathlib.Path) -> set[pathlib.Path]:
    repo_root = repo_root.resolve()
    cargo_home = pathlib.Path(os.environ.get("CARGO_HOME", pathlib.Path.home() / ".cargo")).resolve()
    external_roots = [cargo_home]
    external_roots.extend(parent / ".cargo" for parent in repo_root.parents)
    for cargo_config_root in external_roots:
        if cargo_config_root == repo_root / ".cargo":
            continue
        for name in ("config", "config.toml"):
            candidate = cargo_config_root / name
            if candidate.is_file():
                raise InputDigestError(
                    f"external Cargo configuration is not an audited release input: {candidate}"
                )

    pending = [repo_root / ".cargo" / name for name in ("config", "config.toml")]
    result: set[pathlib.Path] = set()
    while pending:
        unresolved_path = pending.pop().absolute()
        if not unresolved_path.exists():
            continue
        candidate = unresolved_path
        while candidate != repo_root and repo_root in candidate.parents:
            if candidate.is_symlink():
                raise InputDigestError(f"Cargo config input must not traverse a symlink: {candidate}")
            candidate = candidate.parent
        path = unresolved_path.resolve()
        if path in result:
            continue
        try:
            path.relative_to(repo_root)
        except ValueError as error:
            raise InputDigestError(f"Cargo config include escapes the repository: {path}") from error
        if not path.is_file():
            raise InputDigestError(f"Cargo config input must be a regular file: {path}")
        try:
            document = tomllib.loads(path.read_text(encoding="utf-8"))
        except (OSError, UnicodeError, tomllib.TOMLDecodeError) as error:
            raise InputDigestError(f"cannot parse Cargo config {path}: {error}") from error
        result.add(path)
        includes = document.get("include", [])
        if isinstance(includes, str):
            includes = [includes]
        if not isinstance(includes, list) or not all(isinstance(value, str) for value in includes):
            raise InputDigestError(f"Cargo config include must be a string or string list: {path}")
        pending.extend(path.parent / value for value in includes)
    return result


def _build_environment() -> dict[str, str]:
    overrides = {
        key: value
        for key, value in sorted(os.environ.items())
        if value
        and (
            key in BUILD_ENVIRONMENT_KEYS
            or re.fullmatch(
                r"(?:HOST_|TARGET_)?(?:AR|CC|CFLAGS|CXX|CXXFLAGS|RANLIB)(?:_.+)?",
                key,
            )
            is not None
            or key.startswith("CARGO_PROFILE_RELEASE_APPLE_")
            or (
                key.startswith("CARGO_TARGET_")
                and key.endswith(("_LINKER", "_RUNNER", "_RUSTFLAGS"))
            )
        )
    }
    if overrides:
        raise InputDigestError(
            "release build has unaudited compiler/linker overrides: "
            + ", ".join(overrides)
        )
    return {}


def _tool_identities(encoded_tools: Sequence[str]) -> dict[str, str]:
    identities: dict[str, str] = {}
    for encoded_tool in encoded_tools:
        role, separator, encoded_path = encoded_tool.partition("=")
        if not separator or not role or not encoded_path or role in identities:
            raise InputDigestError(f"invalid or duplicate tool identity: {encoded_tool}")
        path = pathlib.Path(encoded_path).resolve()
        if not path.is_file():
            raise InputDigestError(f"build tool is not a regular file: {encoded_path}")
        identities[role] = _sha256(path.read_bytes())
    if not identities:
        raise InputDigestError("release build supplied no audited tool binaries")
    return dict(sorted(identities.items()))


def _rust_library_identities(cargo: str, targets: Iterable[str]) -> dict[str, str]:
    rustc = str(pathlib.Path(cargo).with_name("rustc"))
    completed = subprocess.run(
        [rustc, "--print", "sysroot"], check=False, capture_output=True, text=True
    )
    if completed.returncode != 0:
        raise InputDigestError("cannot determine the pinned Rust sysroot")
    rustlib = pathlib.Path(completed.stdout.strip()) / "lib" / "rustlib"
    identities: dict[str, str] = {
        "compiler-libraries": _directory_direct_files_hash(rustlib.parent)
    }
    for target in sorted(set(targets)):
        target_root = rustlib / target
        identities[target] = _directory_content_hash(target_root / "lib")
        codegen_backends = target_root / "codegen-backends"
        if codegen_backends.exists():
            identities[f"{target}:codegen-backends"] = _directory_content_hash(
                codegen_backends
            )
    return identities


def build_manifest(
    repo_root: pathlib.Path,
    metadata_by_target: Mapping[str, Mapping[str, Any]],
    configuration: Mapping[str, object],
    target_cfg_by_target: Mapping[str, set[tuple[str, str | None]]] | None = None,
    host_metadata: Mapping[str, Any] | None = None,
    host_target: str | None = None,
    host_cfg: set[tuple[str, str | None]] | None = None,
    resolutions_by_context: Mapping[str, Mapping[str, Sequence[str]]] | None = None,
    root_package: str = ROOT_PACKAGE,
    features: Sequence[str] = DEFAULT_FEATURES,
) -> dict[str, object]:
    repo_root = repo_root.resolve()
    lock_packages = _lock_packages(repo_root)
    package_records: dict[tuple[str, str, str, str], dict[str, object]] = {}
    local_package_roots: set[pathlib.Path] = set()

    contexts: list[tuple[str, Mapping[str, Any], str, set[tuple[str, str | None]]]] = []
    for target in sorted(metadata_by_target):
        target_cfg = (
            target_cfg_by_target[target]
            if target_cfg_by_target is not None
            else _fallback_apple_target_cfg(target)
        )
        contexts.append((target, metadata_by_target[target], target, target_cfg))
    if host_metadata is not None:
        if host_target is None or host_cfg is None:
            raise InputDigestError("host metadata requires an exact host target and cfg set")
        contexts.append(("host", host_metadata, host_target, host_cfg))

    for context_label, metadata, target, target_cfg in contexts:
        resolution = (
            resolutions_by_context.get(context_label)
            if resolutions_by_context is not None
            else None
        )
        if resolutions_by_context is not None and resolution is None:
            raise InputDigestError(f"missing exact Cargo tree resolution for {context_label}")
        reachable = (
            set(resolution)
            if resolution is not None
            else _reachable_package_ids(metadata, root_package, target, target_cfg)
        )
        packages_by_id = {package["id"]: package for package in metadata["packages"]}
        nodes_by_id = {node["id"]: node for node in metadata["resolve"]["nodes"]}
        for package_id in sorted(reachable):
            package = packages_by_id[package_id]
            manifest_path = pathlib.Path(package["manifest_path"]).resolve()
            source = package.get("source")
            lock_entry = lock_packages.get((package["name"], package["version"], source))
            checksum = lock_entry.get("checksum") if lock_entry is not None else None
            if source is not None and lock_entry is None:
                raise InputDigestError(
                    f"external dependency is missing its Cargo.lock entry: {package_id}"
                )
            if source is not None and source.startswith("registry+") and checksum is None:
                raise InputDigestError(
                    f"registry dependency is missing its Cargo.lock checksum: {package_id}"
                )
            relative_manifest = None
            if source is None:
                relative_manifest = _relative_path(repo_root, manifest_path)
                local_package_roots.add(manifest_path.parent)
            key = (
                package["name"],
                package["version"],
                source or "path",
                relative_manifest or "",
            )
            record = package_records.setdefault(
                key,
                {
                    "checksum": checksum,
                    "lockEntryHash": (
                        _sha256(_canonical_json(_lock_entry_projection(lock_entry)))
                        if lock_entry
                        else None
                    ),
                    "manifestPath": relative_manifest,
                    "name": package["name"],
                    "resolvedSourceHash": (
                        _directory_content_hash(manifest_path.parent)
                        if source is not None
                        else None
                    ),
                    "source": source,
                    "targets": {},
                    "version": package["version"],
                },
            )
            record_targets = record["targets"]
            assert isinstance(record_targets, dict)
            record_targets[context_label] = sorted(
                resolution[package_id]
                if resolution is not None
                else nodes_by_id[package_id].get("features", [])
            )

    file_records: dict[str, dict[str, str]] = {}
    for package_root in sorted(local_package_roots):
        for relative_path in _package_files(repo_root, package_root):
            file_records[relative_path] = _read_file(
                repo_root, relative_path, "dependency-closure"
            )

    required_paths = {"Cargo.toml", *PACKAGING_INPUTS}
    for pattern in ("rust-toolchain", "rust-toolchain.toml"):
        required_paths.update(
            path.relative_to(repo_root).as_posix()
            for path in repo_root.glob(pattern)
            if path.is_file()
        )
    required_paths.update(_relative_path(repo_root, path) for path in _cargo_config_paths(repo_root))
    for relative_path in sorted(required_paths):
        kind = "cargo-resolution" if relative_path == "Cargo.toml" or relative_path.startswith(".cargo/") else "distribution"
        file_records[relative_path] = _read_file(repo_root, relative_path, kind)

    return {
        "configuration": dict(configuration),
        "features": sorted(features),
        "files": [file_records[path] for path in sorted(file_records)],
        "packages": [package_records[key] for key in sorted(package_records)],
        "rootPackage": root_package,
        "schemaVersion": SCHEMA_VERSION,
        "targets": sorted(metadata_by_target),
    }


def _cargo_metadata(
    cargo: str,
    repo_root: pathlib.Path,
    target: str,
    root_package: str = ROOT_PACKAGE,
    features: Sequence[str] = DEFAULT_FEATURES,
) -> Mapping[str, Any]:
    command = [
        cargo,
        "metadata",
        "--locked",
        "--format-version",
        "1",
        "--manifest-path",
        str(repo_root / "Cargo.toml"),
        "--no-default-features",
        "--features",
        ",".join(f"{root_package}/{feature}" for feature in features),
        "--filter-platform",
        target,
    ]
    completed = subprocess.run(command, check=False, capture_output=True, cwd=repo_root)
    if completed.returncode != 0:
        raise InputDigestError(
            f"Cargo metadata failed for {target}:\n"
            + completed.stderr.decode(errors="replace")
        )
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise InputDigestError(f"Cargo metadata returned invalid JSON for {target}") from error


_TREE_PACKAGE = re.compile(r"(?P<name>\S+) v(?P<version>\S+)(?: \((?P<location>.*)\))?\Z")


def _cargo_tree_resolution(
    cargo: str,
    repo_root: pathlib.Path,
    target: str,
    metadata: Mapping[str, Any],
    root_package: str = ROOT_PACKAGE,
    features: Sequence[str] = DEFAULT_FEATURES,
) -> dict[str, list[str]]:
    command = [
        cargo,
        "tree",
        "--locked",
        "--manifest-path",
        str(repo_root / "Cargo.toml"),
        "--package",
        root_package,
        "--no-default-features",
        "--features",
        ",".join(features),
        "--target",
        target,
        "--edges",
        "normal,build",
        "--prefix",
        "none",
        "--format",
        "{p}\t{f}",
    ]
    completed = subprocess.run(command, check=False, capture_output=True, text=True, cwd=repo_root)
    if completed.returncode != 0:
        raise InputDigestError(f"Cargo tree failed for {target}:\n{completed.stderr}")
    packages = metadata.get("packages")
    if not isinstance(packages, list):
        raise InputDigestError("Cargo metadata is missing its package inventory")
    resolved: dict[str, set[str]] = {}
    for line in completed.stdout.splitlines():
        line = line.removesuffix(" (*)")
        display, separator, encoded_features = line.partition("\t")
        match = _TREE_PACKAGE.fullmatch(display)
        if not separator or match is None:
            raise InputDigestError(f"cannot parse Cargo tree record: {line}")
        candidates = [
            package
            for package in packages
            if package.get("name") == match["name"]
            and package.get("version") == match["version"]
        ]
        location = match["location"]
        if location == "proc-macro":
            location = None
        if location is not None:
            local_candidates = [
                package
                for package in candidates
                if package.get("source") is None
                and pathlib.Path(package["manifest_path"]).resolve().parent
                == pathlib.Path(location).resolve()
            ]
            source_candidates = [
                package
                for package in candidates
                if isinstance(package.get("source"), str)
                and location in package["source"]
            ]
            candidates = local_candidates or source_candidates
        if len(candidates) != 1:
            raise InputDigestError(
                f"Cargo tree package is ambiguous in metadata: {display} ({len(candidates)} matches)"
            )
        package_id = candidates[0]["id"]
        resolved.setdefault(package_id, set()).update(
            feature for feature in encoded_features.split(",") if feature
        )
    return {package_id: sorted(features) for package_id, features in sorted(resolved.items())}


def _fallback_apple_target_cfg(target: str) -> set[tuple[str, str | None]]:
    components = target.split("-")
    arch = components[0]
    os_name = "macos" if target.endswith("apple-darwin") else "ios"
    environment = "sim" if target.endswith("-sim") else ""
    return {
        ("target_arch", arch),
        ("target_env", environment),
        ("target_family", "unix"),
        ("target_os", os_name),
        ("target_vendor", "apple"),
        ("unix", None),
    }


def _rustc_target_cfg(cargo: str, target: str) -> set[tuple[str, str | None]]:
    rustc = str(pathlib.Path(cargo).with_name("rustc"))
    completed = subprocess.run(
        [rustc, "--print", "cfg", "--target", target],
        check=False,
        capture_output=True,
    )
    if completed.returncode != 0:
        raise InputDigestError(
            f"rustc target cfg failed for {target}:\n"
            + completed.stderr.decode(errors="replace")
        )
    result: set[tuple[str, str | None]] = set()
    for encoded_line in completed.stdout.splitlines():
        line = encoded_line.decode(errors="strict")
        if "=" in line:
            key, encoded_value = line.split("=", 1)
            result.add((key, json.loads(encoded_value)))
        else:
            result.add((line, None))
    return result


def _rustc_host_target(cargo: str) -> str:
    rustc = str(pathlib.Path(cargo).with_name("rustc"))
    completed = subprocess.run([rustc, "-vV"], check=False, capture_output=True, text=True)
    if completed.returncode != 0:
        raise InputDigestError("cannot determine the pinned Rust host target")
    for line in completed.stdout.splitlines():
        if line.startswith("host: "):
            return line.removeprefix("host: ")
    raise InputDigestError("pinned rustc did not report a host target")


def _configuration(
    arguments: argparse.Namespace, targets: Sequence[str]
) -> dict[str, object]:
    return {
        "buildEnvironment": _build_environment(),
        "buildProfile": arguments.build_profile,
        "cargo": arguments.cargo_version,
        "minimumIOSVersion": arguments.minimum_ios_version,
        "minimumMacOSVersion": arguments.minimum_macos_version,
        "rustToolchain": arguments.rust_toolchain,
        "rustc": arguments.rustc_version,
        "hostTarget": _rustc_host_target(arguments.cargo),
        "rustLibraries": _rust_library_identities(
            arguments.cargo, (*targets, _rustc_host_target(arguments.cargo))
        ),
        "toolBinaries": _tool_identities(arguments.tool),
        "sdk": {
            "iphoneOS": arguments.iphoneos_sdk,
            "iphoneSimulator": arguments.iphonesimulator_sdk,
            "macOS": arguments.macos_sdk,
        },
        "xcode": {
            "build": arguments.xcode_build,
            "version": arguments.xcode_version,
        },
    }


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("write", "verify"))
    parser.add_argument("manifest", type=pathlib.Path)
    parser.add_argument("--repo-root", required=True, type=pathlib.Path)
    parser.add_argument("--cargo", required=True)
    parser.add_argument("--root-package", default=ROOT_PACKAGE)
    parser.add_argument("--feature", action="append", dest="features")
    parser.add_argument("--target", action="append", dest="targets")
    parser.add_argument("--build-profile", required=True)
    parser.add_argument("--rust-toolchain", required=True)
    parser.add_argument("--rustc-version", required=True)
    parser.add_argument("--cargo-version", required=True)
    parser.add_argument("--xcode-version", required=True)
    parser.add_argument("--xcode-build", required=True)
    parser.add_argument("--iphoneos-sdk", required=True)
    parser.add_argument("--iphonesimulator-sdk", required=True)
    parser.add_argument("--macos-sdk", required=True)
    parser.add_argument("--minimum-ios-version", required=True)
    parser.add_argument("--minimum-macos-version", required=True)
    parser.add_argument(
        "--tool",
        action="append",
        default=[],
        help="audited build tool as ROLE=/absolute/path (repeatable)",
    )
    return parser


def main(arguments: Sequence[str]) -> int:
    parsed = _parser().parse_args(arguments)
    repo_root = parsed.repo_root.resolve()
    features = tuple(parsed.features or DEFAULT_FEATURES)
    targets = tuple(parsed.targets or DEFAULT_TARGETS)
    metadata_by_target = {
        target: _cargo_metadata(
            parsed.cargo, repo_root, target, parsed.root_package, features
        )
        for target in targets
    }
    target_cfg_by_target = {
        target: _rustc_target_cfg(parsed.cargo, target) for target in targets
    }
    host_target = _rustc_host_target(parsed.cargo)
    host_metadata = _cargo_metadata(
        parsed.cargo, repo_root, host_target, parsed.root_package, features
    )
    host_cfg = _rustc_target_cfg(parsed.cargo, host_target)
    resolutions_by_context = {
        target: _cargo_tree_resolution(
            parsed.cargo,
            repo_root,
            target,
            metadata_by_target[target],
            parsed.root_package,
            features,
        )
        for target in targets
    }
    resolutions_by_context["host"] = _cargo_tree_resolution(
        parsed.cargo,
        repo_root,
        host_target,
        host_metadata,
        parsed.root_package,
        features,
    )
    manifest = build_manifest(
        repo_root,
        metadata_by_target,
        _configuration(parsed, targets),
        target_cfg_by_target,
        host_metadata,
        host_target,
        host_cfg,
        resolutions_by_context,
        parsed.root_package,
        features,
    )
    encoded = _canonical_json(manifest)
    digest = _sha256(encoded)
    if parsed.command == "write":
        parsed.manifest.parent.mkdir(parents=True, exist_ok=True)
        parsed.manifest.write_bytes(encoded)
    else:
        try:
            existing = parsed.manifest.read_bytes()
        except OSError as error:
            raise InputDigestError(f"cannot read build-input manifest: {error}") from error
        if existing != encoded:
            raise InputDigestError(
                "build-input manifest does not match the current audited dependency closure"
            )
    print(digest)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except (InputDigestError, OSError) as error:
        raise SystemExit(f"apple-runtime-input-digest: {error}") from error
