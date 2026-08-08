#!/usr/bin/env python3

import hashlib
import json
import pathlib
import re
import sys
import tempfile
import unicodedata


METADATA_KEYS = {
    "schemaVersion",
    "runtimeVersion",
    "buildSourceRevision",
    "releaseRevision",
    "buildInputsHash",
    "buildInputsManifestPath",
    "runtimeIdentity",
    "contractFingerprint",
    "luaurVersion",
    "buildProfile",
    "rustToolchain",
    "xcodeVersion",
    "xcodeBuild",
    "iphoneOSSDKVersion",
    "iphoneOSSDKBuild",
    "iphoneSimulatorSDKVersion",
    "iphoneSimulatorSDKBuild",
    "macOSSDKVersion",
    "macOSSDKBuild",
    "minimumIOSVersion",
    "minimumMacOSVersion",
    "thirdPartyNoticesPath",
    "swiftPackageChecksum",
}
BUILD_INPUT_KEYS = {
    "configuration",
    "features",
    "files",
    "packages",
    "rootPackage",
    "schemaVersion",
    "targets",
}
BUILD_TARGETS = {
    "aarch64-apple-darwin",
    "aarch64-apple-ios",
    "aarch64-apple-ios-sim",
    "x86_64-apple-darwin",
    "x86_64-apple-ios",
}
SEMVER = re.compile(r"[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?\Z")
SOURCE_REVISION = re.compile(r"[0-9a-f]{40}(?:-dirty\.[0-9a-f]{64})?\Z")
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
HEADER_FUNCTION = re.compile(r"\b(nux_[a-z0-9_]+)\s*\(")
EXPORTED_FUNCTION = re.compile(r"_nux_[a-z0-9_]+\Z")
FULL_APPLE_TARGETS = BUILD_TARGETS
IOS_TARGETS = {
    "aarch64-apple-ios",
    "aarch64-apple-ios-sim",
    "x86_64-apple-ios",
}
FORBIDDEN_HEADER_IDENTIFIERS = {
    "NUX_FLOW_SESSION_ABI_MINOR",
    "NUX_RUNTIME_ABI_MAJOR",
    "NUX_RUNTIME_ABI_MINOR",
    "NUX_STATUS_ABI_MISMATCH",
    "minimum_abi_minor",
    # Keep the hard-cut sentinel while honoring the repository-wide rule that
    # retired lowercase ABI spellings do not appear verbatim in code.
    "nux_" "flow_runtime_context_create",
    "nux_runtime_abi_major",
    "nux_runtime_abi_minor",
    "nux_runtime_require_abi",
    "required_abi_major",
}


class ContractError(ValueError):
    pass


def _string(document: dict[str, object], key: str) -> str:
    value = document.get(key)
    if not isinstance(value, str):
        raise ContractError(f"{key} must be a string")
    if any(unicodedata.category(character) == "Cc" for character in value):
        raise ContractError(f"{key} contains a control character")
    return value


def validate_metadata(document: object) -> None:
    if not isinstance(document, dict):
        raise ContractError("artifact metadata must be a top-level object")
    actual_keys = set(document)
    if actual_keys != METADATA_KEYS:
        missing = sorted(METADATA_KEYS - actual_keys)
        extra = sorted(actual_keys - METADATA_KEYS)
        raise ContractError(f"metadata keys differ: missing={missing}, extra={extra}")
    if document["schemaVersion"] != 5:
        raise ContractError("schemaVersion must be exactly 5")

    runtime_version = _string(document, "runtimeVersion")
    build_source_revision = _string(document, "buildSourceRevision")
    release_revision = _string(document, "releaseRevision")
    runtime_identity = _string(document, "runtimeIdentity")
    contract_fingerprint = _string(document, "contractFingerprint")
    swift_package_checksum = _string(document, "swiftPackageChecksum")
    build_inputs_manifest_path = _string(document, "buildInputsManifestPath")
    for key in METADATA_KEYS - {"schemaVersion"}:
        _string(document, key)

    build_inputs_hash = document["buildInputsHash"]
    if not isinstance(build_inputs_hash, str) or SHA256.fullmatch(build_inputs_hash) is None:
        raise ContractError("buildInputsHash must be a lowercase SHA-256")
    if build_inputs_manifest_path != "NuxieRuntime.xcframework/BUILD_INPUTS.json":
        raise ContractError("buildInputsManifestPath is not the canonical artifact path")

    if SEMVER.fullmatch(runtime_version) is None:
        raise ContractError("runtimeVersion is not canonical SemVer")
    if SOURCE_REVISION.fullmatch(build_source_revision) is None:
        raise ContractError("buildSourceRevision is not an exact source identity")
    if re.fullmatch(r"[0-9a-f]{40}", release_revision) is None:
        raise ContractError("releaseRevision is not an exact clean source identity")
    if runtime_identity != f"{runtime_version}@{build_source_revision}":
        raise ContractError("runtimeIdentity does not match version and build revision")
    if SHA256.fullmatch(contract_fingerprint) is None:
        raise ContractError("contractFingerprint is not a lowercase SHA-256")
    if SHA256.fullmatch(swift_package_checksum) is None:
        raise ContractError("swiftPackageChecksum is not a lowercase SHA-256")


def expected_symbols(header: str) -> set[str]:
    identifiers = set(re.findall(r"\b[A-Za-z_][A-Za-z0-9_]*\b", header))
    forbidden = sorted(identifiers & FORBIDDEN_HEADER_IDENTIFIERS)
    if forbidden:
        raise ContractError(
            f"generated header exposes removed client ABI identifiers: {forbidden}"
        )
    symbols = {f"_{name}" for name in HEADER_FUNCTION.findall(header)}
    if not symbols:
        raise ContractError("generated header declares no public nux_* functions")
    return symbols


def validate_symbols(header: str, exported: str) -> None:
    expected = expected_symbols(header)
    actual = {
        line.strip()
        for line in exported.splitlines()
        if EXPORTED_FUNCTION.fullmatch(line.strip()) is not None
    }
    if actual != expected:
        missing = sorted(expected - actual)
        extra = sorted(actual - expected)
        raise ContractError(
            f"public symbol set differs: missing={missing}, extra={extra}"
        )


def validate_symbol_partitions(
    manifests: dict[str, str], exported: str
) -> None:
    if not manifests:
        raise ContractError("symbol partitions are empty")
    partitions: dict[str, set[str]] = {}
    owner: dict[str, str] = {}
    for name, encoded in manifests.items():
        lines = encoded.splitlines()
        if not lines or lines != sorted(set(lines)):
            raise ContractError(f"symbol partition {name} is not unique and sorted")
        if any(re.fullmatch(r"nux_[a-z0-9_]+", symbol) is None for symbol in lines):
            raise ContractError(f"symbol partition {name} has a malformed export")
        partition = set(lines)
        for symbol in partition:
            previous = owner.get(symbol)
            if previous is not None:
                raise ContractError(
                    f"symbol partition overlap: {symbol} belongs to {previous} and {name}"
                )
            owner[symbol] = name
        partitions[name] = partition

    expected = {f"_{symbol}" for partition in partitions.values() for symbol in partition}
    actual = {
        line.strip()
        for line in exported.splitlines()
        if EXPORTED_FUNCTION.fullmatch(line.strip()) is not None
    }
    if actual != expected:
        missing = sorted(expected - actual)
        extra = sorted(actual - expected)
        raise ContractError(
            f"partitioned public symbol set differs: missing={missing}, extra={extra}"
        )


def validate_distribution_metadata(document: object) -> None:
    if not isinstance(document, dict):
        raise ContractError("distribution metadata must be a top-level object")
    expected_keys = {
        "schemaVersion",
        "runtimeVersion",
        "buildSourceRevision",
        "releaseRevision",
        "runtimeIdentity",
        "contractFingerprint",
        "buildInputsHash",
        "artifacts",
    }
    if set(document) != expected_keys:
        raise ContractError("distribution metadata has an incomplete or unknown schema")
    if document["schemaVersion"] != 6:
        raise ContractError("distribution schemaVersion must be exactly 6")
    runtime_version = _string(document, "runtimeVersion")
    build_revision = _string(document, "buildSourceRevision")
    release_revision = _string(document, "releaseRevision")
    if SEMVER.fullmatch(runtime_version) is None:
        raise ContractError("runtimeVersion is not canonical SemVer")
    if SOURCE_REVISION.fullmatch(build_revision) is None:
        raise ContractError("buildSourceRevision is not an exact source identity")
    if re.fullmatch(r"[0-9a-f]{40}", release_revision) is None:
        raise ContractError("releaseRevision is not an exact clean source identity")
    if document["runtimeIdentity"] != f"{runtime_version}@{build_revision}":
        raise ContractError("runtimeIdentity does not match version and build revision")
    for key in ("contractFingerprint", "buildInputsHash"):
        if not isinstance(document[key], str) or SHA256.fullmatch(document[key]) is None:
            raise ContractError(f"{key} must be a lowercase SHA-256")

    artifacts = document["artifacts"]
    if not isinstance(artifacts, list) or len(artifacts) != 2:
        raise ContractError("distribution must describe exactly two artifacts")
    expected = {
        "full-apple": (
            "NuxieRuntime.xcframework.zip",
            FULL_APPLE_TARGETS,
        ),
        "ios-only": (
            "NuxieRuntime-iOS.xcframework.zip",
            IOS_TARGETS,
        ),
    }
    if [artifact.get("kind") for artifact in artifacts if isinstance(artifact, dict)] != [
        "full-apple",
        "ios-only",
    ]:
        raise ContractError("distribution artifacts must be ordered full-apple then ios-only")
    checksums: set[str] = set()
    for artifact in artifacts:
        if not isinstance(artifact, dict) or set(artifact) != {
            "kind",
            "archiveName",
            "bundleName",
            "swiftPackageChecksum",
            "targets",
        }:
            raise ContractError("distribution artifact has a malformed schema")
        kind = artifact["kind"]
        if kind not in expected:
            raise ContractError(f"unknown distribution artifact kind: {kind}")
        archive_name, targets = expected[kind]
        if artifact["archiveName"] != archive_name:
            raise ContractError(f"{kind} has a noncanonical archive name")
        if artifact["bundleName"] != "NuxieRuntime.xcframework":
            raise ContractError(f"{kind} has a noncanonical bundle name")
        checksum = artifact["swiftPackageChecksum"]
        if not isinstance(checksum, str) or SHA256.fullmatch(checksum) is None:
            raise ContractError(f"{kind} has a malformed SwiftPM checksum")
        checksums.add(checksum)
        artifact_targets = artifact["targets"]
        if not isinstance(artifact_targets, list) or artifact_targets != sorted(targets):
            raise ContractError(f"{kind} does not contain its exact target set")
    if len(checksums) != 2:
        raise ContractError("distribution artifacts must have distinct archive checksums")


def validate_layout_oracle(document: object) -> None:
    if not isinstance(document, dict) or set(document) != {
        "schemaVersion",
        "dataModel",
        "types",
    }:
        raise ContractError("layout oracle has an incomplete or unknown schema")
    if document["schemaVersion"] != 1 or document["dataModel"] != "apple-lp64":
        raise ContractError("layout oracle must describe schema 1 Apple LP64")
    types = document["types"]
    if not isinstance(types, list) or not types:
        raise ContractError("layout oracle has no public value types")
    names = [record.get("name") for record in types if isinstance(record, dict)]
    if len(names) != len(types) or names != sorted(set(names)):
        raise ContractError("layout oracle type names are not unique and sorted")
    for record in types:
        if set(record) != {"name", "size", "alignment", "fields"}:
            raise ContractError("layout oracle has a malformed type record")
        if not isinstance(record["size"], int) or record["size"] <= 0:
            raise ContractError(f"{record['name']} has an invalid size")
        if not isinstance(record["alignment"], int) or record["alignment"] <= 0:
            raise ContractError(f"{record['name']} has an invalid alignment")
        fields = record["fields"]
        if not isinstance(fields, list) or not fields:
            raise ContractError(f"{record['name']} has no field layout")
        field_names = [field.get("name") for field in fields if isinstance(field, dict)]
        offsets = [field.get("offset") for field in fields if isinstance(field, dict)]
        if (
            len(field_names) != len(fields)
            or len(set(field_names)) != len(fields)
            or not all(isinstance(offset, int) and offset >= 0 for offset in offsets)
            or offsets != sorted(offsets)
        ):
            raise ContractError(f"{record['name']} field offsets are malformed")
        if any(set(field) != {"name", "offset"} for field in fields):
            raise ContractError(f"{record['name']} has a malformed field record")


def validate_size_report(report: object, budgets: object, *, release: bool) -> None:
    if not isinstance(report, dict) or set(report) != {"schemaVersion", "artifacts"}:
        raise ContractError("size report has an incomplete or unknown schema")
    if report["schemaVersion"] != 1 or not isinstance(report["artifacts"], dict):
        raise ContractError("size report must use schema 1")
    if set(report["artifacts"]) != {"full-apple", "ios-only"}:
        raise ContractError("size report must contain both distribution artifacts")
    metric_keys = {
        "compressedBytes",
        "expandedBytes",
        "representativeLinkedBytes",
        "sliceBytes",
    }
    for kind, metrics in report["artifacts"].items():
        if not isinstance(metrics, dict) or set(metrics) != metric_keys:
            raise ContractError(f"{kind} size record is malformed")
        if not all(
            isinstance(metrics[key], int) and metrics[key] >= 0
            for key in metric_keys - {"sliceBytes"}
        ):
            raise ContractError(f"{kind} has an invalid byte measurement")
        if (
            not isinstance(metrics["sliceBytes"], dict)
            or not metrics["sliceBytes"]
            or not all(
                isinstance(target, str)
                and isinstance(value, int)
                and value >= 0
                for target, value in metrics["sliceBytes"].items()
            )
        ):
            raise ContractError(f"{kind} has malformed per-slice measurements")

    if not isinstance(budgets, dict) or set(budgets) != {
        "schemaVersion",
        "mode",
        "maximums",
    } or budgets["schemaVersion"] != 1:
        raise ContractError("size budgets have an incomplete or unknown schema")
    if budgets["mode"] == "candidate" and budgets["maximums"] is None:
        if release:
            raise ContractError("release size budgets have not been frozen")
        return
    if budgets["mode"] != "release" or not isinstance(budgets["maximums"], dict):
        raise ContractError("size budgets have an invalid mode")
    if set(budgets["maximums"]) != {"full-apple", "ios-only"}:
        raise ContractError("size budgets must cover both distribution artifacts")
    for kind, measured in report["artifacts"].items():
        maximum = budgets["maximums"][kind]
        if not isinstance(maximum, dict) or set(maximum) != metric_keys:
            raise ContractError(f"{kind} size budget is malformed")
        for metric in metric_keys - {"sliceBytes"}:
            limit = maximum[metric]
            if not isinstance(limit, int) or limit < 0:
                raise ContractError(f"{kind}.{metric} budget is invalid")
            if measured[metric] > limit:
                raise ContractError(
                    f"{kind}.{metric} exceeds budget: {measured[metric]} > {limit}"
                )
        if set(maximum["sliceBytes"]) != set(measured["sliceBytes"]):
            raise ContractError(f"{kind}.sliceBytes budget target set differs")
        for target, value in measured["sliceBytes"].items():
            limit = maximum["sliceBytes"][target]
            if not isinstance(limit, int) or value > limit:
                raise ContractError(f"{kind}.sliceBytes.{target} exceeds budget")


def validate_build_inputs(document: object, encoded: bytes, expected_hash: str) -> None:
    if not isinstance(document, dict) or set(document) != BUILD_INPUT_KEYS:
        raise ContractError("build-input manifest has an incomplete or unknown schema")
    if document["schemaVersion"] != 1:
        raise ContractError("build-input manifest schemaVersion must be exactly 1")
    if document["rootPackage"] != "nux-apple-runtime":
        raise ContractError("build-input manifest has the wrong root package")
    if document["features"] != ["apple-product"]:
        raise ContractError("build-input manifest has the wrong feature set")
    targets = document["targets"]
    if not isinstance(targets, list) or set(targets) != BUILD_TARGETS or targets != sorted(targets):
        raise ContractError("build-input manifest does not cover the exact Apple target set")
    configuration = document["configuration"]
    if not isinstance(configuration, dict) or set(configuration) != {
        "buildProfile",
        "buildEnvironment",
        "cargo",
        "minimumIOSVersion",
        "minimumMacOSVersion",
        "rustToolchain",
        "rustc",
        "hostTarget",
        "rustLibraries",
        "toolBinaries",
        "sdk",
        "xcode",
    }:
        raise ContractError("build-input manifest has incomplete toolchain configuration")
    if not all(
        isinstance(value, str) and value
        for key, value in configuration.items()
        if key
        not in {"buildEnvironment", "rustLibraries", "sdk", "toolBinaries", "xcode"}
    ):
        raise ContractError("build-input manifest has an empty toolchain value")
    if configuration["buildEnvironment"] != {}:
        raise ContractError("build-input manifest contains forbidden environment overrides")
    if (
        not isinstance(configuration["toolBinaries"], dict)
        or not configuration["toolBinaries"]
        or not all(
            isinstance(role, str)
            and role
            and isinstance(digest, str)
            and SHA256.fullmatch(digest) is not None
            for role, digest in configuration["toolBinaries"].items()
        )
    ):
        raise ContractError("build-input manifest has malformed tool-binary provenance")
    if (
        not isinstance(configuration["rustLibraries"], dict)
        or not configuration["rustLibraries"]
        or not all(
            isinstance(target, str)
            and target
            and isinstance(digest, str)
            and SHA256.fullmatch(digest) is not None
            for target, digest in configuration["rustLibraries"].items()
        )
    ):
        raise ContractError("build-input manifest has malformed Rust-library provenance")
    if not isinstance(configuration["sdk"], dict) or set(configuration["sdk"]) != {
        "iphoneOS",
        "iphoneSimulator",
        "macOS",
    }:
        raise ContractError("build-input manifest has incomplete SDK provenance")
    if not isinstance(configuration["xcode"], dict) or set(configuration["xcode"]) != {
        "build",
        "version",
    }:
        raise ContractError("build-input manifest has incomplete Xcode provenance")

    files = document["files"]
    if not isinstance(files, list) or not files:
        raise ContractError("build-input manifest has no file inputs")
    paths: list[str] = []
    for record in files:
        if not isinstance(record, dict) or set(record) != {"kind", "path", "sha256"}:
            raise ContractError("build-input manifest has a malformed file record")
        path = record["path"]
        if not isinstance(path, str) or not path or path.startswith("/") or ".." in pathlib.PurePosixPath(path).parts:
            raise ContractError("build-input manifest has an unsafe file path")
        if not isinstance(record["kind"], str) or not record["kind"]:
            raise ContractError("build-input manifest has an empty file classification")
        if (
            not isinstance(record["sha256"], str)
            or SHA256.fullmatch(record["sha256"]) is None
        ):
            raise ContractError("build-input manifest has a malformed file hash")
        paths.append(path)
    if paths != sorted(set(paths)):
        raise ContractError("build-input manifest file paths are not unique and sorted")

    packages = document["packages"]
    if not isinstance(packages, list) or not packages:
        raise ContractError("build-input manifest has no dependency closure")
    for package in packages:
        if not isinstance(package, dict) or set(package) != {
            "checksum",
            "lockEntryHash",
            "manifestPath",
            "name",
            "resolvedSourceHash",
            "source",
            "targets",
            "version",
        }:
            raise ContractError("build-input manifest has a malformed package record")
        if not all(isinstance(package[key], str) and package[key] for key in ("name", "version")):
            raise ContractError("build-input manifest has an unnamed dependency")
        if package["source"] is None:
            if not isinstance(package["manifestPath"], str) or not package["manifestPath"]:
                raise ContractError("local dependency is missing its manifest path")
        elif not isinstance(package["source"], str) or not package["source"]:
            raise ContractError("dependency source is malformed")
        else:
            if package["source"].startswith("registry+") and (
                not isinstance(package["checksum"], str)
                or SHA256.fullmatch(package["checksum"]) is None
            ):
                raise ContractError("registry dependency is missing its lockfile checksum")
            if (
                not isinstance(package["lockEntryHash"], str)
                or SHA256.fullmatch(package["lockEntryHash"]) is None
            ):
                raise ContractError("external dependency is missing its lock-entry hash")
            if (
                not isinstance(package["resolvedSourceHash"], str)
                or SHA256.fullmatch(package["resolvedSourceHash"]) is None
            ):
                raise ContractError("external dependency is missing its resolved-source hash")
        if package["source"] is None and package["resolvedSourceHash"] is not None:
            raise ContractError("local dependency has an unexpected resolved-source hash")
        if package["source"] is None and package["lockEntryHash"] is not None and (
            not isinstance(package["lockEntryHash"], str)
            or SHA256.fullmatch(package["lockEntryHash"]) is None
        ):
            raise ContractError("local dependency has a malformed lock-entry hash")
        package_targets = package["targets"]
        if (
            not isinstance(package_targets, dict)
            or not package_targets
            or not set(package_targets) <= BUILD_TARGETS | {"host"}
        ):
            raise ContractError("dependency has an invalid target closure")

    canonical = (json.dumps(document, sort_keys=True, separators=(",", ":")) + "\n").encode()
    if canonical != encoded:
        raise ContractError("build-input manifest is not canonical JSON")
    if SHA256.fullmatch(expected_hash) is None:
        raise ContractError("expected build-input hash is malformed")
    if hashlib.sha256(encoded).hexdigest() != expected_hash:
        raise ContractError("build-input manifest does not match buildInputsHash")


def _load_json(path: pathlib.Path) -> object:
    try:
        with path.open(encoding="utf-8") as source:
            return json.load(source)
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ContractError(f"cannot read {path}: {error}") from error


def qualify_release_metadata(path: pathlib.Path, release_revision: str) -> None:
    document = _load_json(path)
    validate_metadata(document)
    if re.fullmatch(r"[0-9a-f]{40}", release_revision) is None:
        raise ContractError("release revision is not an exact clean source identity")
    assert isinstance(document, dict)
    document["releaseRevision"] = release_revision
    encoded = (json.dumps(document, indent=2) + "\n").encode()
    temporary_path: pathlib.Path | None = None
    try:
        with tempfile.NamedTemporaryFile(
            prefix=f".{path.name}.", dir=path.parent, delete=False
        ) as destination:
            destination.write(encoded)
            temporary_path = pathlib.Path(destination.name)
        temporary_path.replace(path)
    except OSError as error:
        raise ContractError(f"cannot qualify release metadata {path}: {error}") from error
    finally:
        if temporary_path is not None:
            temporary_path.unlink(missing_ok=True)


def main(arguments: list[str]) -> int:
    if len(arguments) == 2 and arguments[0] == "metadata":
        validate_metadata(_load_json(pathlib.Path(arguments[1])))
        return 0
    if len(arguments) == 3 and arguments[0] == "release":
        qualify_release_metadata(pathlib.Path(arguments[1]), arguments[2])
        return 0
    if len(arguments) == 3 and arguments[0] == "symbols":
        try:
            header = pathlib.Path(arguments[1]).read_text(encoding="utf-8")
            exported = pathlib.Path(arguments[2]).read_text(encoding="utf-8")
        except (OSError, UnicodeError) as error:
            raise ContractError(f"cannot read symbol inputs: {error}") from error
        validate_symbols(header, exported)
        return 0
    if len(arguments) == 3 and arguments[0] == "inputs":
        path = pathlib.Path(arguments[1])
        try:
            encoded = path.read_bytes()
            document = json.loads(encoded)
        except (OSError, UnicodeError, json.JSONDecodeError) as error:
            raise ContractError(f"cannot read {path}: {error}") from error
        validate_build_inputs(document, encoded, arguments[2])
        return 0
    raise ContractError(
        "usage: apple_runtime_contract.py "
        "metadata <artifact.json> | release <artifact.json> <revision> | "
        "inputs <BUILD_INPUTS.json> <sha256> | "
        "symbols <header> <nm-output>"
    )


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        raise SystemExit(f"apple-runtime-contract: {error}") from error
