#!/usr/bin/env python3
"""Build evidence and fail-closed verification for the Android C runtime cut."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import re
import stat
import struct
import subprocess
import sys
import tempfile
import zipfile
import zlib
from collections.abc import Sequence


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
ARTIFACT_NAME = "NuxieRuntimeAndroid.zip"
METADATA_NAME = "NuxieRuntimeAndroid.json"
BUILD_INPUTS_NAME = "NuxieRuntimeAndroid-BUILD_INPUTS.json"
SIZE_REPORT_NAME = "NuxieRuntimeAndroid-SIZE_REPORT.json"
ARTIFACT_VERSION = "0.3.9"
RELEASE_TAG = f"android-runtime-v{ARTIFACT_VERSION}"
RUST_TOOLCHAIN = "1.94.1"
CARGO_NDK_VERSION = "4.1.2"
ANDROID_NDK_VERSION = "29.0.14206865"
ANDROID_API_LEVEL = 23
ANDROID_LOAD_ALIGNMENT = 0x4000
ROOT_PACKAGE = "nux-capi"
FEATURES = ["android-authored-wgsl", "android-vulkan", "scripting"]
EMBEDDED_FEATURES = "android-vulkan,android-authored-wgsl,scripting"
TARGETS = ["aarch64-linux-android", "x86_64-linux-android"]
ABIS = ["arm64-v8a", "x86_64"]
ABI_TARGETS = {
    "arm64-v8a": ("aarch64-linux-android", 183),
    "x86_64": ("x86_64-linux-android", 62),
}
EXPECTED_FILES = (
    "include/nux_capi.generated.h",
    "jniLibs/arm64-v8a/libc++_shared.so",
    "jniLibs/arm64-v8a/libnux_capi.so",
    "jniLibs/x86_64/libc++_shared.so",
    "jniLibs/x86_64/libnux_capi.so",
)
CONTRACT_INPUTS = (
    "crates/nux-capi/abi-layout-v4.json",
    "crates/nux-capi/exports-v4-android-authored-wgsl-extension.txt",
    "crates/nux-capi/exports-v4-android-vulkan-extension.txt",
    "crates/nux-capi/exports-v4-apple-metal-extension.txt",
    "crates/nux-capi/exports-v4-portable.txt",
    "crates/nux-capi/include/nux_capi.generated.h",
)
DISTRIBUTION_INPUTS = (
    "Cargo.lock",
    "Cargo.toml",
    "crates/nux-capi/Cargo.toml",
    "crates/nux-capi/build.rs",
    "crates/nux-capi/cbindgen.toml",
    *CONTRACT_INPUTS,
    "tools/android-runtime-size-budget-v4.json",
    "tools/android_runtime_contract.py",
    "tools/build-nux-capi-android.sh",
    "tools/publish-nux-capi-android-release.sh",
)
TOOL_ROLES = {
    "cargo",
    "cargo-ndk",
    "ndk-aarch64-api23-clang",
    "ndk-clang",
    "ndk-ld.lld",
    "ndk-llvm-ar",
    "ndk-llvm-nm",
    "ndk-llvm-readelf",
    "ndk-llvm-strings",
    "ndk-llvm-strip",
    "ndk-x86_64-api23-clang",
    "python",
    "rustc",
}
ZIP_TIMESTAMP = (1980, 1, 1, 0, 0, 0)
SHA256 = re.compile(r"[0-9a-f]{64}\Z")
SOURCE_REVISION = re.compile(r"[0-9a-f]{40}\Z")
SEMVER = re.compile(r"[0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?\Z")
HEADER_FUNCTION = re.compile(r"\b(nux_[a-z0-9_]+)\s*\(")
COMMENTS = re.compile(r"/\*.*?\*/|//[^\r\n]*", re.DOTALL)
PROVENANCE = re.compile(
    r'\{"schemaVersion":6,"rootPackage":"nux-capi"[^{}\r\n]*\}'
)
EXPECTED_NEEDED = {
    "libnux_capi.so": {"libc++_shared.so", "libc.so", "libdl.so", "libm.so"},
    "libc++_shared.so": {"libc.so", "libdl.so", "libm.so"},
}


class ContractError(ValueError):
    """The candidate does not satisfy the immutable Android contract."""


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: pathlib.Path) -> str:
    return sha256_bytes(path.read_bytes())


def canonical_json(document: object) -> bytes:
    return (json.dumps(document, sort_keys=True, separators=(",", ":")) + "\n").encode()


def write_json(path: pathlib.Path, document: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(document, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def require_exact_keys(document: object, keys: set[str], label: str) -> dict[str, object]:
    if not isinstance(document, dict) or set(document) != keys:
        raise ContractError(f"{label} has an incomplete or unknown schema")
    return document


def require_sha256(value: object, label: str) -> str:
    if not isinstance(value, str) or SHA256.fullmatch(value) is None:
        raise ContractError(f"{label} must be a lowercase SHA-256")
    return value


def require_source_revision(value: object, label: str) -> str:
    if not isinstance(value, str) or SOURCE_REVISION.fullmatch(value) is None:
        raise ContractError(f"{label} must be a full lowercase 40-character Git SHA")
    return value


def require_semver(value: object, label: str) -> str:
    if not isinstance(value, str) or SEMVER.fullmatch(value) is None:
        raise ContractError(f"{label} must be canonical SemVer")
    return value


def contract_records(repo_root: pathlib.Path) -> list[dict[str, str]]:
    records: list[dict[str, str]] = []
    for relative in CONTRACT_INPUTS:
        path = repo_root / relative
        if not path.is_file() or path.is_symlink():
            raise ContractError(f"contract input is missing or not regular: {relative}")
        records.append({"path": relative, "sha256": sha256_file(path)})
    return records


def contract_fingerprint(repo_root: pathlib.Path) -> str:
    return sha256_bytes(canonical_json(contract_records(repo_root)))


def directory_digest(path: pathlib.Path) -> str:
    if not path.is_dir() or path.is_symlink():
        raise ContractError(f"toolchain library directory is not regular: {path}")
    records: list[dict[str, str]] = []
    for candidate in sorted(path.rglob("*")):
        if candidate.is_symlink():
            raise ContractError(f"toolchain library input is a symlink: {candidate}")
        if candidate.is_file():
            records.append(
                {
                    "path": candidate.relative_to(path).as_posix(),
                    "sha256": sha256_file(candidate),
                }
            )
    if not records:
        raise ContractError(f"toolchain library directory is empty: {path}")
    return sha256_bytes(canonical_json(records))


def command_output(command: Sequence[str]) -> str:
    completed = subprocess.run(command, check=False, capture_output=True, text=True)
    if completed.returncode != 0:
        raise ContractError(
            f"command failed ({' '.join(command)}):\n{completed.stderr.strip()}"
        )
    return completed.stdout.strip()


def tool_record(role: str, path: pathlib.Path, version: str) -> dict[str, str]:
    if not path.is_file():
        raise ContractError(f"{role} tool is not a file: {path}")
    return {
        "name": path.name,
        "role": role,
        "sha256": sha256_file(path),
        "version": version,
    }


def build_input_document(
    *,
    repo_root: pathlib.Path,
    source_revision: str,
    runtime_version: str,
    source_date_epoch: int,
    rustc: pathlib.Path,
    cargo: pathlib.Path,
    cargo_ndk: pathlib.Path,
    ndk_root: pathlib.Path,
    ndk_host_tag: str,
) -> dict[str, object]:
    require_source_revision(source_revision, "source revision")
    require_semver(runtime_version, "runtime version")
    if source_date_epoch < 0:
        raise ContractError("SOURCE_DATE_EPOCH must be non-negative")

    prebuilt = ndk_root / "toolchains/llvm/prebuilt" / ndk_host_tag
    bin_root = prebuilt / "bin"
    sysroot_lib = prebuilt / "sysroot/usr/lib"
    source_properties = ndk_root / "source.properties"
    if not source_properties.is_file():
        raise ContractError("Android NDK source.properties is missing")
    ndk_revision_match = re.search(
        r"^Pkg\.Revision\s*=\s*(\S+)\s*$",
        source_properties.read_text(encoding="utf-8"),
        re.MULTILINE,
    )
    ndk_revision = ndk_revision_match.group(1) if ndk_revision_match else ""
    if ndk_revision != ANDROID_NDK_VERSION:
        raise ContractError(
            f"Android NDK must be {ANDROID_NDK_VERSION}, found {ndk_revision or 'unknown'}"
        )

    rustc_version = command_output([str(rustc), "-vV"]).replace("\n", " ")
    cargo_version = command_output([str(cargo), "-Vv"]).replace("\n", " ")
    cargo_ndk_version = command_output([str(cargo), "ndk", "--version"])
    if not rustc_version.startswith(f"rustc {RUST_TOOLCHAIN} "):
        raise ContractError(f"rustc must be {RUST_TOOLCHAIN}: {rustc_version}")
    if not cargo_version.startswith(f"cargo {RUST_TOOLCHAIN} "):
        raise ContractError(f"cargo must be {RUST_TOOLCHAIN}: {cargo_version}")
    if cargo_ndk_version != f"cargo-ndk {CARGO_NDK_VERSION}":
        raise ContractError(
            f"cargo-ndk must be {CARGO_NDK_VERSION}: {cargo_ndk_version}"
        )

    ndk_version = command_output([str(bin_root / "clang"), "--version"]).splitlines()[0]
    shared_version = ndk_version
    tool_paths = {
        "cargo": cargo,
        "cargo-ndk": cargo_ndk,
        "ndk-aarch64-api23-clang": bin_root / "aarch64-linux-android23-clang",
        "ndk-clang": bin_root / "clang",
        "ndk-ld.lld": bin_root / "ld.lld",
        "ndk-llvm-ar": bin_root / "llvm-ar",
        "ndk-llvm-nm": bin_root / "llvm-nm",
        "ndk-llvm-readelf": bin_root / "llvm-readelf",
        "ndk-llvm-strings": bin_root / "llvm-strings",
        "ndk-llvm-strip": bin_root / "llvm-strip",
        "ndk-x86_64-api23-clang": bin_root / "x86_64-linux-android23-clang",
        "python": pathlib.Path(sys.executable),
        "rustc": rustc,
    }
    python_identity = f"Python {sys.version.split()[0]}; zlib {zlib.ZLIB_VERSION}"
    tool_versions = {
        "cargo": cargo_version,
        "cargo-ndk": cargo_ndk_version,
        "python": python_identity,
        "rustc": rustc_version,
    }
    tools = [
        tool_record(role, path, tool_versions.get(role, shared_version))
        for role, path in sorted(tool_paths.items())
    ]

    files: list[dict[str, str]] = []
    for relative in sorted(set(DISTRIBUTION_INPUTS)):
        path = repo_root / relative
        if not path.is_file() or path.is_symlink():
            raise ContractError(f"distribution input is missing or not regular: {relative}")
        files.append({"path": relative, "sha256": sha256_file(path)})

    rust_libraries: dict[str, str] = {}
    for target in TARGETS:
        target_libdir = pathlib.Path(
            command_output([str(rustc), "--print", "target-libdir", "--target", target])
        )
        rust_libraries[target] = directory_digest(target_libdir)

    ndk_runtime_libraries = {
        abi: sha256_file(sysroot_lib / target / "libc++_shared.so")
        for abi, (target, _) in ABI_TARGETS.items()
    }
    return {
        "configuration": {
            "androidApiLevel": ANDROID_API_LEVEL,
            "androidNdk": ndk_revision,
            "androidNdkHostTag": ndk_host_tag,
            "androidNdkSourcePropertiesSha256": sha256_file(source_properties),
            "buildEnvironment": {},
            "buildProfile": "release",
            "cargo": cargo_version,
            "cargoNdk": cargo_ndk_version,
            "python": python_identity,
            "rustToolchain": RUST_TOOLCHAIN,
            "rustc": rustc_version,
            "sourceDateEpoch": source_date_epoch,
        },
        "features": FEATURES,
        "files": files,
        "ndkRuntimeLibraries": ndk_runtime_libraries,
        "rootPackage": ROOT_PACKAGE,
        "runtimeVersion": runtime_version,
        "rustLibraries": rust_libraries,
        "schemaVersion": 1,
        "sourceRevision": source_revision,
        "targets": TARGETS,
        "tools": tools,
    }


def validate_build_inputs(document: object, encoded: bytes | None = None) -> dict[str, object]:
    value = require_exact_keys(
        document,
        {
            "configuration",
            "features",
            "files",
            "ndkRuntimeLibraries",
            "rootPackage",
            "runtimeVersion",
            "rustLibraries",
            "schemaVersion",
            "sourceRevision",
            "targets",
            "tools",
        },
        "Android build inputs",
    )
    if encoded is not None and encoded != canonical_json(value):
        raise ContractError("Android build inputs are not canonical JSON")
    if value["schemaVersion"] != 1 or value["rootPackage"] != ROOT_PACKAGE:
        raise ContractError("Android build inputs have the wrong schema or root package")
    require_source_revision(value["sourceRevision"], "build-input sourceRevision")
    require_semver(value["runtimeVersion"], "build-input runtimeVersion")
    if value["features"] != FEATURES or value["targets"] != TARGETS:
        raise ContractError("Android build inputs have the wrong target or feature set")

    configuration = require_exact_keys(
        value["configuration"],
        {
            "androidApiLevel",
            "androidNdk",
            "androidNdkHostTag",
            "androidNdkSourcePropertiesSha256",
            "buildEnvironment",
            "buildProfile",
            "cargo",
            "cargoNdk",
            "python",
            "rustToolchain",
            "rustc",
            "sourceDateEpoch",
        },
        "Android build-input configuration",
    )
    if (
        configuration["androidApiLevel"] != ANDROID_API_LEVEL
        or configuration["androidNdk"] != ANDROID_NDK_VERSION
        or configuration["buildEnvironment"] != {}
        or configuration["buildProfile"] != "release"
        or configuration["cargoNdk"] != f"cargo-ndk {CARGO_NDK_VERSION}"
        or configuration["rustToolchain"] != RUST_TOOLCHAIN
    ):
        raise ContractError("Android build-input configuration is not the pinned release cut")
    if not isinstance(configuration["sourceDateEpoch"], int) or configuration["sourceDateEpoch"] < 0:
        raise ContractError("sourceDateEpoch must be a non-negative integer")
    if not isinstance(configuration["cargo"], str) or not configuration["cargo"].startswith(
        f"cargo {RUST_TOOLCHAIN} "
    ):
        raise ContractError("build-input Cargo version is not pinned")
    if not isinstance(configuration["rustc"], str) or not configuration["rustc"].startswith(
        f"rustc {RUST_TOOLCHAIN} "
    ):
        raise ContractError("build-input rustc version is not pinned")
    for label in ("androidNdkHostTag", "python"):
        if not isinstance(configuration[label], str) or not configuration[label]:
            raise ContractError(f"build-input {label} is empty")
    require_sha256(
        configuration["androidNdkSourcePropertiesSha256"],
        "build-input Android NDK source.properties",
    )

    files = value["files"]
    if not isinstance(files, list) or not files:
        raise ContractError("Android build inputs contain no file identities")
    file_paths: list[str] = []
    for record in files:
        item = require_exact_keys(record, {"path", "sha256"}, "build-input file")
        path = item["path"]
        if not isinstance(path, str) or not path or path.startswith("/") or ".." in pathlib.PurePosixPath(path).parts:
            raise ContractError("build-input file path is not repository-relative")
        file_paths.append(path)
        require_sha256(item["sha256"], f"build-input file {path}")
    if file_paths != sorted(set(file_paths)):
        raise ContractError("build-input file identities are not unique and sorted")
    missing_distribution_inputs = sorted(set(DISTRIBUTION_INPUTS) - set(file_paths))
    if missing_distribution_inputs:
        raise ContractError(f"build inputs omit distribution files: {missing_distribution_inputs}")

    tools = value["tools"]
    if not isinstance(tools, list):
        raise ContractError("Android build inputs tools must be a list")
    roles: list[str] = []
    for record in tools:
        item = require_exact_keys(
            record, {"name", "role", "sha256", "version"}, "build-input tool"
        )
        if not all(isinstance(item[key], str) and item[key] for key in ("name", "role", "version")):
            raise ContractError("build-input tool identity is incomplete")
        roles.append(item["role"])
        require_sha256(item["sha256"], f"build-input tool {item['role']}")
    if roles != sorted(TOOL_ROLES):
        raise ContractError("build-input tool identity does not cover the exact tool set")
    tools_by_role = {record["role"]: record for record in tools}
    if tools_by_role["cargo-ndk"]["version"] != configuration["cargoNdk"]:
        raise ContractError("cargo-ndk tool identity differs from configuration")
    if tools_by_role["cargo"]["version"] != configuration["cargo"]:
        raise ContractError("Cargo tool identity differs from configuration")
    if tools_by_role["rustc"]["version"] != configuration["rustc"]:
        raise ContractError("rustc tool identity differs from configuration")
    if tools_by_role["python"]["version"] != configuration["python"]:
        raise ContractError("Python tool identity differs from configuration")

    for key, exact_keys in (
        ("rustLibraries", set(TARGETS)),
        ("ndkRuntimeLibraries", set(ABIS)),
    ):
        mapping = value[key]
        if not isinstance(mapping, dict) or set(mapping) != exact_keys:
            raise ContractError(f"{key} does not cover the exact Android target set")
        for label, digest in mapping.items():
            require_sha256(digest, f"{key}.{label}")
    return value


def regular_files(root: pathlib.Path) -> list[str]:
    if not root.is_dir() or root.is_symlink():
        raise ContractError(f"artifact root is not a regular directory: {root}")
    result: list[str] = []
    for candidate in sorted(root.rglob("*")):
        if candidate.is_symlink():
            raise ContractError(f"artifact file must not be a symlink: {candidate}")
        if candidate.is_file():
            result.append(candidate.relative_to(root).as_posix())
    return result


def validate_prebuilt_tree(root: pathlib.Path) -> None:
    actual = regular_files(root)
    if actual != list(EXPECTED_FILES):
        missing = sorted(set(EXPECTED_FILES) - set(actual))
        extra = sorted(set(actual) - set(EXPECTED_FILES))
        raise ContractError(f"Android artifact tree differs: missing={missing}, extra={extra}")


def create_deterministic_zip(root: pathlib.Path, archive: pathlib.Path) -> None:
    validate_prebuilt_tree(root)
    archive.parent.mkdir(parents=True, exist_ok=True)
    temporary = archive.with_name(f".{archive.name}.tmp")
    if temporary.exists():
        temporary.unlink()
    with zipfile.ZipFile(
        temporary, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9
    ) as output:
        output.comment = b""
        for relative in EXPECTED_FILES:
            information = zipfile.ZipInfo(relative, date_time=ZIP_TIMESTAMP)
            information.compress_type = zipfile.ZIP_DEFLATED
            information.create_system = 3
            information.external_attr = (stat.S_IFREG | 0o644) << 16
            information.extra = b""
            information.comment = b""
            output.writestr(information, (root / relative).read_bytes(), compresslevel=9)
    temporary.replace(archive)


def validate_zip(archive: pathlib.Path) -> dict[str, bytes]:
    if not archive.is_file() or archive.is_symlink():
        raise ContractError(f"Android archive is missing or not regular: {archive}")
    with zipfile.ZipFile(archive) as packaged:
        if packaged.comment:
            raise ContractError("Android archive must not have a ZIP comment")
        records = packaged.infolist()
        names = [record.filename for record in records]
        if names != list(EXPECTED_FILES):
            raise ContractError(f"Android ZIP entries differ from the exact five-file tree: {names}")
        contents: dict[str, bytes] = {}
        for record in records:
            path = pathlib.PurePosixPath(record.filename)
            mode = record.external_attr >> 16
            if (
                path.is_absolute()
                or ".." in path.parts
                or record.is_dir()
                or record.date_time != ZIP_TIMESTAMP
                or record.compress_type != zipfile.ZIP_DEFLATED
                or record.create_system != 3
                or record.create_version != 20
                or record.extract_version != 20
                or record.reserved != 0
                or record.volume != 0
                or record.internal_attr != 0
                or record.external_attr != (stat.S_IFREG | 0o644) << 16
                or record.extra
                or record.comment
                or record.flag_bits & 0x1
                or not stat.S_ISREG(mode)
                or stat.S_IMODE(mode) != 0o644
            ):
                raise ContractError(f"Android ZIP entry is not canonical: {record.filename}")
            contents[record.filename] = packaged.read(record)
        return contents


def file_records(contents: dict[str, bytes]) -> list[dict[str, object]]:
    return [
        {
            "path": path,
            "sha256": sha256_bytes(contents[path]),
            "sizeBytes": len(contents[path]),
        }
        for path in EXPECTED_FILES
    ]


def validate_budget(document: object) -> dict[str, object]:
    value = require_exact_keys(
        document, {"maximums", "releaseTag", "schemaVersion"}, "Android size budget"
    )
    if value["schemaVersion"] != 1 or value["releaseTag"] != RELEASE_TAG:
        raise ContractError("Android size budget has the wrong schema or release tag")
    maximums = require_exact_keys(
        value["maximums"],
        {"archiveBytes", "expandedBytes", "fileBytes"},
        "Android size maximums",
    )
    for key in ("archiveBytes", "expandedBytes"):
        if not isinstance(maximums[key], int) or maximums[key] <= 0:
            raise ContractError(f"Android size maximum {key} is invalid")
    file_maximums = maximums["fileBytes"]
    if not isinstance(file_maximums, dict) or list(file_maximums) != list(EXPECTED_FILES):
        raise ContractError("Android per-file size budget does not cover the exact artifact tree")
    if not all(isinstance(value, int) and value > 0 for value in file_maximums.values()):
        raise ContractError("Android per-file size budget contains an invalid maximum")
    return value


def size_report(
    archive: pathlib.Path,
    records: list[dict[str, object]],
    budget: dict[str, object],
    budget_sha256: str,
) -> dict[str, object]:
    file_bytes = {record["path"]: record["sizeBytes"] for record in records}
    archive_bytes = archive.stat().st_size
    expanded_bytes = sum(file_bytes.values())
    maximums = budget["maximums"]
    return {
        "artifactName": ARTIFACT_NAME,
        "budgetSha256": budget_sha256,
        "headroomBytes": {
            "archiveBytes": maximums["archiveBytes"] - archive_bytes,
            "expandedBytes": maximums["expandedBytes"] - expanded_bytes,
            "fileBytes": {
                path: maximums["fileBytes"][path] - file_bytes[path]
                for path in EXPECTED_FILES
            },
        },
        "maximums": maximums,
        "measurements": {
            "archiveBytes": archive_bytes,
            "expandedBytes": expanded_bytes,
            "fileBytes": file_bytes,
        },
        "releaseTag": RELEASE_TAG,
        "schemaVersion": 1,
    }


def validate_size_report(
    document: object,
    budget: dict[str, object],
    budget_sha256: str,
    archive: pathlib.Path,
    records: list[dict[str, object]],
) -> None:
    expected = size_report(archive, records, budget, budget_sha256)
    if document != expected:
        raise ContractError("Android size report does not match artifact bytes and budget")
    headroom = expected["headroomBytes"]
    if headroom["archiveBytes"] < 0 or headroom["expandedBytes"] < 0:
        raise ContractError("Android aggregate artifact size exceeds the committed budget")
    over = [path for path, value in headroom["fileBytes"].items() if value < 0]
    if over:
        raise ContractError(f"Android artifact files exceed the committed budget: {over}")


def metadata_document(
    *,
    runtime_version: str,
    source_revision: str,
    fingerprint: str,
    build_inputs_hash: str,
    archive: pathlib.Path,
    records: list[dict[str, object]],
) -> dict[str, object]:
    return {
        "android": {
            "abis": ABIS,
            "apiLevel": ANDROID_API_LEVEL,
            "features": FEATURES,
            "rustTargets": TARGETS,
        },
        "artifact": {
            "archiveBytes": archive.stat().st_size,
            "expandedBytes": sum(record["sizeBytes"] for record in records),
            "files": records,
            "name": ARTIFACT_NAME,
            "sha256": sha256_file(archive),
        },
        "artifactVersion": ARTIFACT_VERSION,
        "buildInputsHash": build_inputs_hash,
        "buildSourceRevision": source_revision,
        "contractFingerprint": fingerprint,
        "releaseRevision": source_revision,
        "releaseTag": RELEASE_TAG,
        "runtimeIdentity": f"{runtime_version}@{source_revision}",
        "runtimeVersion": runtime_version,
        "schemaVersion": 1,
    }


def validate_metadata(
    document: object,
    *,
    archive: pathlib.Path,
    records: list[dict[str, object]],
    build_inputs: dict[str, object],
    build_inputs_hash: str,
    fingerprint: str,
    release_revision: str | None,
) -> dict[str, object]:
    value = require_exact_keys(
        document,
        {
            "android",
            "artifact",
            "artifactVersion",
            "buildInputsHash",
            "buildSourceRevision",
            "contractFingerprint",
            "releaseRevision",
            "releaseTag",
            "runtimeIdentity",
            "runtimeVersion",
            "schemaVersion",
        },
        "Android artifact metadata",
    )
    if (
        value["schemaVersion"] != 1
        or value["artifactVersion"] != ARTIFACT_VERSION
        or value["releaseTag"] != RELEASE_TAG
    ):
        raise ContractError("Android artifact metadata has the wrong schema or release identity")
    runtime_version = require_semver(value["runtimeVersion"], "metadata runtimeVersion")
    source_revision = require_source_revision(
        value["buildSourceRevision"], "metadata buildSourceRevision"
    )
    recorded_release_revision = require_source_revision(
        value["releaseRevision"], "metadata releaseRevision"
    )
    if recorded_release_revision != source_revision:
        raise ContractError("Android metadata releaseRevision differs from build source")
    if release_revision is not None and recorded_release_revision != release_revision:
        raise ContractError("Android metadata was not built from the release-tag revision")
    if value["runtimeIdentity"] != f"{runtime_version}@{source_revision}":
        raise ContractError("Android runtimeIdentity does not bind version to source")
    if build_inputs["runtimeVersion"] != runtime_version or build_inputs["sourceRevision"] != source_revision:
        raise ContractError("Android metadata and build inputs identify different sources")
    if value["buildInputsHash"] != build_inputs_hash:
        raise ContractError("Android metadata has the wrong build-input hash")
    if value["contractFingerprint"] != fingerprint:
        raise ContractError("Android metadata has the wrong ABI-v4 contract fingerprint")

    android = require_exact_keys(
        value["android"], {"abis", "apiLevel", "features", "rustTargets"}, "Android target metadata"
    )
    if android != {
        "abis": ABIS,
        "apiLevel": ANDROID_API_LEVEL,
        "features": FEATURES,
        "rustTargets": TARGETS,
    }:
        raise ContractError("Android target metadata differs from the release cut")

    artifact = require_exact_keys(
        value["artifact"],
        {"archiveBytes", "expandedBytes", "files", "name", "sha256"},
        "Android archive metadata",
    )
    expected_artifact = {
        "archiveBytes": archive.stat().st_size,
        "expandedBytes": sum(record["sizeBytes"] for record in records),
        "files": records,
        "name": ARTIFACT_NAME,
        "sha256": sha256_file(archive),
    }
    if artifact != expected_artifact:
        raise ContractError("Android archive metadata does not match exact bytes")
    return value


def export_partitions(repo_root: pathlib.Path) -> dict[str, list[str]]:
    paths = {
        "androidAuthoredWgsl": "crates/nux-capi/exports-v4-android-authored-wgsl-extension.txt",
        "androidVulkan": "crates/nux-capi/exports-v4-android-vulkan-extension.txt",
        "appleMetal": "crates/nux-capi/exports-v4-apple-metal-extension.txt",
        "portable": "crates/nux-capi/exports-v4-portable.txt",
    }
    partitions: dict[str, list[str]] = {}
    owners: dict[str, str] = {}
    for name, relative in paths.items():
        lines = (repo_root / relative).read_text(encoding="utf-8").splitlines()
        if not lines or lines != sorted(set(lines)):
            raise ContractError(f"ABI-v4 export partition {name} is not unique and sorted")
        if any(re.fullmatch(r"nux_[a-z0-9_]+", symbol) is None for symbol in lines):
            raise ContractError(f"ABI-v4 export partition {name} has a malformed symbol")
        for symbol in lines:
            previous = owners.get(symbol)
            if previous is not None and {previous, name} != {"appleMetal", "androidVulkan"}:
                raise ContractError(
                    f"ABI-v4 export partition overlap: {symbol} belongs to {previous} and {name}"
                )
            owners.setdefault(symbol, name)
        partitions[name] = lines
    return partitions


def symbols_from_header(source: str) -> set[str]:
    return set(HEADER_FUNCTION.findall(COMMENTS.sub("", source)))


def public_structs(header: str) -> dict[str, list[str]]:
    structs: dict[str, list[str]] = {}
    for name, body in re.findall(
        r"typedef struct (Nux\w+)\s*\{(.*?)\}\s*\1\s*;", header, re.DOTALL
    ):
        body = COMMENTS.sub("", body)
        fields: list[str] = []
        for declaration in body.split(";"):
            declaration = declaration.strip()
            if not declaration:
                continue
            match = re.search(r"\(\*([A-Za-z_]\w*)\)", declaration)
            if match is None:
                match = re.search(r"([A-Za-z_]\w*)\s*(?:\[[^]]*\])?\s*$", declaration)
            if match is None:
                raise ContractError(
                    f"cannot identify a field in public struct {name}: {declaration}"
                )
            fields.append(match.group(1))
        structs[name] = fields
    if not structs:
        raise ContractError("ABI-v4 header declares no public value structs")
    return structs


def layout_assertions(
    header: str, oracle: object, selected_header: str | None = None
) -> str:
    structs = public_structs(header)
    selected_structs = (
        public_structs(selected_header) if selected_header is not None else structs
    )
    if not set(selected_structs).issubset(structs):
        raise ContractError("selected Android header contains an unknown value struct")
    document = require_exact_keys(
        oracle, {"schemaVersion", "dataModel", "types"}, "ABI-v4 layout oracle"
    )
    if document["schemaVersion"] != 1 or document["dataModel"] not in {
        "apple-lp64",
        "android-lp64",
        "lp64",
    }:
        raise ContractError("ABI-v4 layout oracle must describe schema-1 LP64")
    records = document["types"]
    if not isinstance(records, list) or not records:
        raise ContractError("ABI-v4 layout oracle contains no type records")
    names = [record.get("name") for record in records if isinstance(record, dict)]
    if names != sorted(set(names)) or set(names) != set(structs):
        raise ContractError(
            "ABI-v4 layout oracle type set differs from the generated header"
        )
    lines = ["#include <stddef.h>", '#include "nux_capi.generated.h"']
    for record in records:
        item = require_exact_keys(
            record, {"alignment", "fields", "name", "size"}, "ABI-v4 layout type"
        )
        name = item["name"]
        if (
            not isinstance(item["size"], int)
            or item["size"] <= 0
            or not isinstance(item["alignment"], int)
            or item["alignment"] <= 0
        ):
            raise ContractError(f"ABI-v4 layout type {name} has invalid size/alignment")
        fields = item["fields"]
        if not isinstance(fields, list):
            raise ContractError(f"ABI-v4 layout type {name} fields must be a list")
        field_names: list[str] = []
        for field in fields:
            value = require_exact_keys(field, {"name", "offset"}, "ABI-v4 layout field")
            if (
                not isinstance(value["name"], str)
                or not isinstance(value["offset"], int)
                or value["offset"] < 0
            ):
                raise ContractError(f"ABI-v4 layout field for {name} is malformed")
            field_names.append(value["name"])
        if field_names != structs[name] or len(field_names) != len(set(field_names)):
            raise ContractError(f"ABI-v4 layout fields differ for {name}")
        if name not in selected_structs:
            continue
        if selected_structs[name] != field_names:
            raise ContractError(f"selected Android layout fields differ for {name}")
        lines.append(f'_Static_assert(sizeof({name}) == {item["size"]}, "{name} size");')
        lines.append(
            f'_Static_assert(_Alignof({name}) == {item["alignment"]}, "{name} alignment");'
        )
        for field in fields:
            lines.append(
                f'_Static_assert(offsetof({name}, {field["name"]}) == '
                f'{field["offset"]}, "{name}.{field["name"]} offset");'
            )
    return "\n".join(lines) + "\n"


def validate_headers(
    raw_header: str, selected_header: str, partitions: dict[str, list[str]]
) -> set[str]:
    if re.search(r"^#define\s+NUX_CAPI_ABI_VERSION\s+4(?:[uU])?\s*$", raw_header, re.MULTILINE) is None:
        raise ContractError("packaged header is not ABI version 4")
    full_expected = set().union(*(set(lines) for lines in partitions.values()))
    full_actual = symbols_from_header(raw_header)
    if full_actual != full_expected:
        raise ContractError(
            "packaged ABI-v4 header differs from the full export inventory: "
            f"missing={sorted(full_expected - full_actual)}, extra={sorted(full_actual - full_expected)}"
        )
    android_expected = set(partitions["portable"]) | set(partitions["androidVulkan"]) | set(
        partitions["androidAuthoredWgsl"]
    )
    selected_actual = symbols_from_header(selected_header)
    if selected_actual != android_expected:
        raise ContractError(
            "Android-selected ABI-v4 header differs from the artifact export union: "
            f"missing={sorted(android_expected - selected_actual)}, "
            f"extra={sorted(selected_actual - android_expected)}"
        )
    return android_expected


def parse_nm_symbols(output: str) -> set[str]:
    symbols: set[str] = set()
    for line in output.splitlines():
        fields = line.split()
        if fields and re.fullmatch(r"nux_[A-Za-z0-9_]+", fields[-1]):
            symbols.add(fields[-1])
    return symbols


def parse_needed(output: str) -> set[str]:
    return set(re.findall(r"\(NEEDED\).*?\[([^]]+)\]", output))


def validate_elf_header(data: bytes, expected_machine: int, label: str) -> None:
    if len(data) < 64 or data[:4] != b"\x7fELF":
        raise ContractError(f"{label} is not an ELF object")
    if data[4] != 2 or data[5] != 1 or data[6] != 1:
        raise ContractError(f"{label} must be a current little-endian ELF64 object")
    elf_type, machine = struct.unpack_from("<HH", data, 16)
    if elf_type != 3 or machine != expected_machine:
        raise ContractError(
            f"{label} has the wrong ELF type or architecture: type={elf_type}, machine={machine}"
        )


def validate_load_segment(
    *, offset: int, virtual_address: int, alignment: int, label: str
) -> None:
    if alignment < ANDROID_LOAD_ALIGNMENT:
        raise ContractError(
            f"{label} LOAD alignment {alignment:#x} is below "
            f"the required {ANDROID_LOAD_ALIGNMENT:#x}"
        )
    if alignment & (alignment - 1):
        raise ContractError(f"{label} LOAD alignment {alignment:#x} is not a power of two")
    if offset % ANDROID_LOAD_ALIGNMENT != virtual_address % ANDROID_LOAD_ALIGNMENT:
        raise ContractError(
            f"{label} LOAD offset {offset:#x} and virtual address {virtual_address:#x} "
            f"are not congruent modulo {ANDROID_LOAD_ALIGNMENT:#x}"
        )


def validate_load_segment_alignment(output: str, label: str) -> None:
    load_count = 0
    for line in output.splitlines():
        fields = line.split()
        if not fields or fields[0] != "LOAD":
            continue
        if (
            len(fields) < 8
            or re.fullmatch(r"0x[0-9A-Fa-f]+", fields[1]) is None
            or re.fullmatch(r"0x[0-9A-Fa-f]+", fields[2]) is None
            or re.fullmatch(r"0x[0-9A-Fa-f]+", fields[-1]) is None
        ):
            raise ContractError(f"{label} has a malformed LOAD program header: {line.strip()}")
        load_count += 1
        validate_load_segment(
            offset=int(fields[1], 16),
            virtual_address=int(fields[2], 16),
            alignment=int(fields[-1], 16),
            label=label,
        )
    if load_count == 0:
        raise ContractError(f"{label} has no LOAD program headers")


def validate_elf_load_segments(data: bytes, label: str) -> None:
    if len(data) < 64:
        raise ContractError(f"{label} has a truncated ELF header")
    program_header_offset = struct.unpack_from("<Q", data, 32)[0]
    program_header_size, program_header_count = struct.unpack_from("<HH", data, 54)
    if program_header_size < 56 or program_header_count == 0:
        raise ContractError(f"{label} has a malformed ELF program-header table")
    table_end = program_header_offset + program_header_size * program_header_count
    if program_header_offset < 64 or table_end > len(data):
        raise ContractError(f"{label} has a truncated ELF program-header table")

    load_count = 0
    for index in range(program_header_count):
        entry = program_header_offset + index * program_header_size
        if struct.unpack_from("<I", data, entry)[0] != 1:
            continue
        load_count += 1
        validate_load_segment(
            offset=struct.unpack_from("<Q", data, entry + 8)[0],
            virtual_address=struct.unpack_from("<Q", data, entry + 16)[0],
            alignment=struct.unpack_from("<Q", data, entry + 48)[0],
            label=label,
        )
    if load_count == 0:
        raise ContractError(f"{label} has no ELF LOAD program headers")


def validate_packaged_load_segment_alignments(outputs: dict[str, str]) -> None:
    expected = {path for path in EXPECTED_FILES if path.endswith(".so")}
    actual = set(outputs)
    if actual != expected:
        raise ContractError(
            "Android LOAD alignment evidence differs from the packaged shared libraries: "
            f"missing={sorted(expected - actual)}, extra={sorted(actual - expected)}"
        )
    for path in sorted(expected):
        validate_load_segment_alignment(outputs[path], path)


def validate_provenance(
    strings_output: str,
    *,
    metadata: dict[str, object],
    build_inputs: dict[str, object],
    target: str,
) -> None:
    occurrences = PROVENANCE.findall(strings_output)
    if len(occurrences) != 1:
        raise ContractError(
            f"{target} must contain exactly one schema-6 nux-capi provenance record; "
            f"found {len(occurrences)}"
        )
    try:
        actual = json.loads(occurrences[0])
    except json.JSONDecodeError as error:
        raise ContractError(f"{target} contains malformed build provenance") from error
    expected = {
        "schemaVersion": 6,
        "rootPackage": ROOT_PACKAGE,
        "runtimeVersion": metadata["runtimeVersion"],
        "buildSourceRevision": metadata["buildSourceRevision"],
        "target": target,
        "profile": "release",
        "features": EMBEDDED_FEATURES,
        "rustc": build_inputs["configuration"]["rustc"],
        "buildInputsHash": metadata["buildInputsHash"],
        "contractFingerprint": metadata["contractFingerprint"],
    }
    if actual != expected:
        differing = sorted(
            key for key in set(actual) | set(expected) if actual.get(key) != expected.get(key)
        )
        raise ContractError(f"{target} embedded provenance differs: {differing}")


def resolve_ndk_root(encoded: str | None) -> pathlib.Path:
    candidate = encoded or os.environ.get("ANDROID_NDK_HOME") or os.environ.get("ANDROID_NDK_ROOT")
    if not candidate:
        android_home = os.environ.get("ANDROID_HOME")
        if android_home:
            candidate = str(pathlib.Path(android_home) / "ndk" / ANDROID_NDK_VERSION)
    if not candidate:
        raise ContractError(
            f"set ANDROID_NDK_HOME to Android NDK {ANDROID_NDK_VERSION}"
        )
    root = pathlib.Path(candidate).resolve()
    properties = root / "source.properties"
    if not properties.is_file():
        raise ContractError(f"Android NDK source.properties is missing under {root}")
    match = re.search(
        r"^Pkg\.Revision\s*=\s*(\S+)\s*$",
        properties.read_text(encoding="utf-8"),
        re.MULTILINE,
    )
    if match is None or match.group(1) != ANDROID_NDK_VERSION:
        raise ContractError(f"Android NDK must be exactly {ANDROID_NDK_VERSION}")
    return root


def resolve_ndk_prebuilt(ndk_root: pathlib.Path, expected_host_tag: str | None = None) -> pathlib.Path:
    root = ndk_root / "toolchains/llvm/prebuilt"
    candidates = sorted(path for path in root.iterdir() if path.is_dir()) if root.is_dir() else []
    if expected_host_tag is not None:
        candidates = [path for path in candidates if path.name == expected_host_tag]
    if len(candidates) != 1:
        raise ContractError(
            f"Android NDK must expose exactly one matching host prebuilt, found {[p.name for p in candidates]}"
        )
    return candidates[0]


def verify_current_inputs(
    repo_root: pathlib.Path,
    document: dict[str, object],
    ndk_root: pathlib.Path,
    prebuilt: pathlib.Path,
) -> None:
    current_revision = command_output(
        ["git", "-C", str(repo_root), "rev-parse", "--verify", "HEAD"]
    )
    if current_revision != document["sourceRevision"]:
        raise ContractError("current checkout differs from the artifact source revision")
    for record in document["files"]:
        path = repo_root / record["path"]
        if not path.is_file() or sha256_file(path) != record["sha256"]:
            raise ContractError(f"current source differs from build input {record['path']}")
    rustc_path = pathlib.Path(
        command_output(["rustup", "which", "--toolchain", RUST_TOOLCHAIN, "rustc"])
    )
    cargo_path = pathlib.Path(
        command_output(["rustup", "which", "--toolchain", RUST_TOOLCHAIN, "cargo"])
    )
    cargo_home = pathlib.Path(os.environ.get("CARGO_HOME", pathlib.Path.home() / ".cargo"))
    cargo_ndk_path = pathlib.Path(
        os.environ.get("NUX_ANDROID_CARGO_NDK", cargo_home / "bin/cargo-ndk")
    )
    tool_paths = {
        "cargo": cargo_path,
        "cargo-ndk": cargo_ndk_path,
        "ndk-aarch64-api23-clang": prebuilt / "bin/aarch64-linux-android23-clang",
        "ndk-clang": prebuilt / "bin/clang",
        "ndk-ld.lld": prebuilt / "bin/ld.lld",
        "ndk-llvm-ar": prebuilt / "bin/llvm-ar",
        "ndk-llvm-nm": prebuilt / "bin/llvm-nm",
        "ndk-llvm-readelf": prebuilt / "bin/llvm-readelf",
        "ndk-llvm-strings": prebuilt / "bin/llvm-strings",
        "ndk-llvm-strip": prebuilt / "bin/llvm-strip",
        "ndk-x86_64-api23-clang": prebuilt / "bin/x86_64-linux-android23-clang",
        "python": pathlib.Path(sys.executable),
        "rustc": rustc_path,
    }
    records = {record["role"]: record for record in document["tools"]}
    for role, path in tool_paths.items():
        if not path.is_file() or sha256_file(path) != records[role]["sha256"]:
            raise ContractError(f"current Android NDK tool differs from build input {role}")
    properties_hash = sha256_file(ndk_root / "source.properties")
    if properties_hash != document["configuration"]["androidNdkSourcePropertiesSha256"]:
        raise ContractError("current Android NDK source.properties differs from build input")
    for target in TARGETS:
        target_libdir = pathlib.Path(
            command_output(
                [str(rustc_path), "--print", "target-libdir", "--target", target]
            )
        )
        if directory_digest(target_libdir) != document["rustLibraries"][target]:
            raise ContractError(f"current Rust target libraries differ for {target}")
    sysroot_lib = prebuilt / "sysroot/usr/lib"
    for abi, (target, _) in ABI_TARGETS.items():
        if (
            sha256_file(sysroot_lib / target / "libc++_shared.so")
            != document["ndkRuntimeLibraries"][abi]
        ):
            raise ContractError(f"current Android NDK runtime library differs for {abi}")


def run_checked(command: Sequence[str], label: str) -> str:
    completed = subprocess.run(command, check=False, capture_output=True, text=True)
    if completed.returncode != 0:
        raise ContractError(f"{label} failed:\n{completed.stderr.strip()}")
    return completed.stdout


def verify_artifact(
    *,
    repo_root: pathlib.Path,
    artifact_root: pathlib.Path,
    ndk_root: pathlib.Path,
    release_revision: str | None,
) -> None:
    archive = artifact_root / ARTIFACT_NAME
    metadata_path = artifact_root / METADATA_NAME
    inputs_path = artifact_root / BUILD_INPUTS_NAME
    size_path = artifact_root / SIZE_REPORT_NAME
    budget_path = repo_root / "tools/android-runtime-size-budget-v4.json"
    for path in (archive, metadata_path, inputs_path, size_path, budget_path):
        if not path.is_file() or path.is_symlink():
            raise ContractError(f"required Android release evidence is missing: {path}")

    input_bytes = inputs_path.read_bytes()
    build_inputs = validate_build_inputs(json.loads(input_bytes), input_bytes)
    prebuilt = resolve_ndk_prebuilt(
        ndk_root, build_inputs["configuration"]["androidNdkHostTag"]
    )
    verify_current_inputs(repo_root, build_inputs, ndk_root, prebuilt)
    contents = validate_zip(archive)
    committed_header = (repo_root / "crates/nux-capi/include/nux_capi.generated.h").read_bytes()
    if contents["include/nux_capi.generated.h"] != committed_header:
        raise ContractError("packaged ABI-v4 header differs byte-for-byte from its source")
    records = file_records(contents)
    fingerprint = contract_fingerprint(repo_root)
    metadata = validate_metadata(
        json.loads(metadata_path.read_text(encoding="utf-8")),
        archive=archive,
        records=records,
        build_inputs=build_inputs,
        build_inputs_hash=sha256_bytes(input_bytes),
        fingerprint=fingerprint,
        release_revision=release_revision,
    )
    budget_bytes = budget_path.read_bytes()
    budget = validate_budget(json.loads(budget_bytes))
    validate_size_report(
        json.loads(size_path.read_text(encoding="utf-8")),
        budget,
        sha256_bytes(budget_bytes),
        archive,
        records,
    )

    partitions = export_partitions(repo_root)
    bin_root = prebuilt / "bin"
    with tempfile.TemporaryDirectory(prefix="nux-capi-android-verify-") as temporary:
        extracted = pathlib.Path(temporary)
        for relative, data in contents.items():
            destination = extracted / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
        packaged_header = extracted / "include/nux_capi.generated.h"
        selected_header = run_checked(
            [
                str(bin_root / "aarch64-linux-android23-clang"),
                "-E",
                "-P",
                "-DNUX_CAPI_ANDROID_VULKAN",
                "-DNUX_CAPI_ANDROID_AUTHORED_WGSL",
                "-x",
                "c",
                str(packaged_header),
            ],
            "Android header preprocessing",
        )
        expected_exports = validate_headers(
            packaged_header.read_text(encoding="utf-8"), selected_header, partitions
        )
        layout_source = extracted / "layout.c"
        layout_source.write_text(
            layout_assertions(
                packaged_header.read_text(encoding="utf-8"),
                json.loads(
                    (repo_root / "crates/nux-capi/abi-layout-v4.json").read_text(
                        encoding="utf-8"
                    )
                ),
                selected_header,
            ),
            encoding="utf-8",
        )
        run_checked(
            [
                str(bin_root / "aarch64-linux-android23-clang"),
                "-std=c11",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-DNUX_CAPI_ANDROID_VULKAN",
                "-DNUX_CAPI_ANDROID_AUTHORED_WGSL",
                f"-I{packaged_header.parent}",
                "-fsyntax-only",
                str(layout_source),
            ],
            "ABI-v4 Android LP64 layout verification",
        )

        load_segment_outputs: dict[str, str] = {}
        for abi, (target, machine) in ABI_TARGETS.items():
            for library_name in ("libc++_shared.so", "libnux_capi.so"):
                relative = f"jniLibs/{abi}/{library_name}"
                library = extracted / relative
                validate_elf_header(contents[relative], machine, relative)
                validate_elf_load_segments(contents[relative], relative)
                load_segment_outputs[relative] = run_checked(
                    [str(bin_root / "llvm-readelf"), "-lW", str(library)],
                    f"LOAD segment inspection for {relative}",
                )
                dynamic = run_checked(
                    [str(bin_root / "llvm-readelf"), "--dynamic", str(library)],
                    f"DT_NEEDED inspection for {relative}",
                )
                needed = parse_needed(dynamic)
                if needed != EXPECTED_NEEDED[library_name]:
                    raise ContractError(
                        f"{relative} DT_NEEDED differs: expected={sorted(EXPECTED_NEEDED[library_name])}, "
                        f"actual={sorted(needed)}"
                    )
            nux_library = extracted / f"jniLibs/{abi}/libnux_capi.so"
            exported = parse_nm_symbols(
                run_checked(
                    [
                        str(bin_root / "llvm-nm"),
                        "--dynamic",
                        "--defined-only",
                        "--extern-only",
                        str(nux_library),
                    ],
                    f"export inspection for {abi}",
                )
            )
            if exported != expected_exports:
                raise ContractError(
                    f"{abi} nux-capi exports differ from ABI-v4 Android union: "
                    f"missing={sorted(expected_exports - exported)}, "
                    f"extra={sorted(exported - expected_exports)}"
                )
            strings = run_checked(
                [str(bin_root / "llvm-strings"), str(nux_library)],
                f"provenance inspection for {abi}",
            )
            validate_provenance(
                strings, metadata=metadata, build_inputs=build_inputs, target=target
            )
            libcxx = extracted / f"jniLibs/{abi}/libc++_shared.so"
            expected_libcxx_hash = build_inputs["ndkRuntimeLibraries"][abi]
            if sha256_file(libcxx) != expected_libcxx_hash:
                raise ContractError(f"{abi} libc++_shared.so is not the pinned NDK input")
        validate_packaged_load_segment_alignments(load_segment_outputs)


def package_artifact(
    *,
    repo_root: pathlib.Path,
    artifact_root: pathlib.Path,
    prebuilt_root: pathlib.Path,
    build_inputs_path: pathlib.Path,
    source_revision: str,
    runtime_version: str,
) -> None:
    validate_prebuilt_tree(prebuilt_root)
    input_bytes = build_inputs_path.read_bytes()
    inputs = validate_build_inputs(json.loads(input_bytes), input_bytes)
    if inputs["sourceRevision"] != source_revision or inputs["runtimeVersion"] != runtime_version:
        raise ContractError("package source/version differs from build-input identity")
    artifact_root.mkdir(parents=True, exist_ok=True)
    archive = artifact_root / ARTIFACT_NAME
    create_deterministic_zip(prebuilt_root, archive)
    contents = validate_zip(archive)
    records = file_records(contents)
    budget_path = repo_root / "tools/android-runtime-size-budget-v4.json"
    budget_bytes = budget_path.read_bytes()
    budget = validate_budget(json.loads(budget_bytes))
    metadata = metadata_document(
        runtime_version=runtime_version,
        source_revision=source_revision,
        fingerprint=contract_fingerprint(repo_root),
        build_inputs_hash=sha256_bytes(input_bytes),
        archive=archive,
        records=records,
    )
    report = size_report(
        archive, records, budget, sha256_bytes(budget_bytes)
    )
    write_json(artifact_root / METADATA_NAME, metadata)
    write_json(artifact_root / SIZE_REPORT_NAME, report)
    (artifact_root / BUILD_INPUTS_NAME).write_bytes(input_bytes)
    validate_size_report(report, budget, sha256_bytes(budget_bytes), archive, records)


def create_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)

    fingerprint = subparsers.add_parser("fingerprint")
    fingerprint.add_argument("--repo-root", type=pathlib.Path, default=REPO_ROOT)

    inputs = subparsers.add_parser("inputs")
    inputs.add_argument("output", type=pathlib.Path)
    inputs.add_argument("--repo-root", type=pathlib.Path, default=REPO_ROOT)
    inputs.add_argument("--source-revision", required=True)
    inputs.add_argument("--runtime-version", required=True)
    inputs.add_argument("--source-date-epoch", required=True, type=int)
    inputs.add_argument("--rustc", required=True, type=pathlib.Path)
    inputs.add_argument("--cargo", required=True, type=pathlib.Path)
    inputs.add_argument("--cargo-ndk", required=True, type=pathlib.Path)
    inputs.add_argument("--ndk-root", required=True, type=pathlib.Path)
    inputs.add_argument("--ndk-host-tag", required=True)

    package = subparsers.add_parser("package")
    package.add_argument("--repo-root", type=pathlib.Path, default=REPO_ROOT)
    package.add_argument("--artifact-root", required=True, type=pathlib.Path)
    package.add_argument("--prebuilt-root", required=True, type=pathlib.Path)
    package.add_argument("--build-inputs", required=True, type=pathlib.Path)
    package.add_argument("--source-revision", required=True)
    package.add_argument("--runtime-version", required=True)

    verify = subparsers.add_parser("verify")
    verify.add_argument("--repo-root", type=pathlib.Path, default=REPO_ROOT)
    verify.add_argument("--artifact-root", required=True, type=pathlib.Path)
    verify.add_argument("--ndk-root")
    verify.add_argument("--release-revision")
    return parser


def main(arguments: Sequence[str]) -> int:
    parsed = create_parser().parse_args(arguments)
    if parsed.command == "fingerprint":
        print(contract_fingerprint(parsed.repo_root.resolve()))
    elif parsed.command == "inputs":
        document = build_input_document(
            repo_root=parsed.repo_root.resolve(),
            source_revision=parsed.source_revision,
            runtime_version=parsed.runtime_version,
            source_date_epoch=parsed.source_date_epoch,
            rustc=parsed.rustc.resolve(),
            cargo=parsed.cargo.resolve(),
            cargo_ndk=parsed.cargo_ndk.resolve(),
            ndk_root=parsed.ndk_root.resolve(),
            ndk_host_tag=parsed.ndk_host_tag,
        )
        encoded = canonical_json(document)
        parsed.output.parent.mkdir(parents=True, exist_ok=True)
        parsed.output.write_bytes(encoded)
        print(sha256_bytes(encoded))
    elif parsed.command == "package":
        package_artifact(
            repo_root=parsed.repo_root.resolve(),
            artifact_root=parsed.artifact_root.resolve(),
            prebuilt_root=parsed.prebuilt_root.resolve(),
            build_inputs_path=parsed.build_inputs.resolve(),
            source_revision=parsed.source_revision,
            runtime_version=parsed.runtime_version,
        )
    elif parsed.command == "verify":
        release_revision = parsed.release_revision
        if release_revision is not None:
            require_source_revision(release_revision, "requested release revision")
        verify_artifact(
            repo_root=parsed.repo_root.resolve(),
            artifact_root=parsed.artifact_root.resolve(),
            ndk_root=resolve_ndk_root(parsed.ndk_root),
            release_revision=release_revision,
        )
        print("NuxieRuntimeAndroid.zip satisfies the immutable ABI-v4 Android contract")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except (ContractError, OSError, UnicodeError, json.JSONDecodeError, zipfile.BadZipFile) as error:
        raise SystemExit(f"android-runtime-contract: {error}") from error
