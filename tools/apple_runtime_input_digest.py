#!/usr/bin/env python3
"""Audit the configured Bazel input closure of the Apple runtime distribution."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys

try:
    from .bazel_runtime_build import APPLE_LABEL, BazelRuntime, canonical, digest
except ImportError:
    from bazel_runtime_build import APPLE_LABEL, BazelRuntime, canonical, digest

SCHEMA_VERSION = 2
ROOT_PACKAGE = "nux-apple-product-extension"
DEFAULT_FEATURES = ("apple-runtime",)
DEFAULT_TARGETS = (
    "aarch64-apple-ios", "aarch64-apple-ios-sim", "x86_64-apple-ios",
    "aarch64-apple-darwin", "x86_64-apple-darwin",
)
PACKAGING_INPUTS = (
    "LICENSE",
    "THIRD_PARTY_NOTICES.md",
    "crates/nux-capi/abi-layout-v4.json",
    "crates/nux-capi/exports-v4-apple-metal-extension.txt",
    "crates/nux-capi/exports-v4-portable.txt",
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
    "crates/nux-apple-product-extension/smoke/product_extension_consumer.c",
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

class InputDigestError(ValueError):
    pass


def build_manifest(repo_root, configuration, *, bazel=None):
    bazel = bazel or BazelRuntime(repo_root)
    evidence = bazel.evidence(APPLE_LABEL, DEFAULT_TARGETS, PACKAGING_INPUTS,
                              ios=configuration["minimumIOSVersion"],
                              macos=configuration["minimumMacOSVersion"])
    configuration = dict(configuration)
    tools = dict(evidence["toolBinaries"])
    tools.update(configuration.pop("packagingTools", {}))
    return {
        "configuration": {
            **configuration,
            "buildSystem": "bazel", "bazel": evidence["bazel"],
            "bazelTarget": APPLE_LABEL, "bazelPlatforms": evidence["platforms"],
            "rustc": evidence["rustc"], "rustLibraries": evidence["rustLibraries"],
            "toolBinaries": tools,
        },
        "features": list(DEFAULT_FEATURES), "files": evidence["files"],
        "packages": evidence["packages"], "rootPackage": ROOT_PACKAGE,
        "schemaVersion": SCHEMA_VERSION, "targets": sorted(DEFAULT_TARGETS),
    }


def _tool_identities(encoded_tools):
    tools = {}
    for encoded in encoded_tools:
        role, separator, path = encoded.partition("=")
        if not separator or not role or role in tools or not Path(path).is_absolute():
            raise InputDigestError("packaging tools must have unique roles and absolute paths")
        tools[role] = digest(path)
    return tools


def main(arguments):
    parser = argparse.ArgumentParser()
    parser.add_argument("command", choices=("write", "verify"))
    parser.add_argument("manifest", type=Path)
    parser.add_argument("--repo-root", required=True, type=Path)
    parser.add_argument("--build-profile", default="release-apple", choices=("release-apple",))
    parser.add_argument("--rust-toolchain", default="1.94.1", choices=("1.94.1",))
    parser.add_argument("--xcode-version", required=True)
    parser.add_argument("--xcode-build", required=True)
    parser.add_argument("--iphoneos-sdk", required=True)
    parser.add_argument("--iphonesimulator-sdk", required=True)
    parser.add_argument("--macos-sdk", required=True)
    parser.add_argument("--minimum-ios-version", default="15.0")
    parser.add_argument("--minimum-macos-version", default="12.0")
    parser.add_argument("--tool", action="append", default=[])
    args = parser.parse_args(arguments)
    configuration = {
        "buildEnvironment": {}, "buildProfile": args.build_profile,
        "rustToolchain": args.rust_toolchain,
        "minimumIOSVersion": args.minimum_ios_version,
        "minimumMacOSVersion": args.minimum_macos_version,
        "packagingTools": _tool_identities(args.tool),
        "sdk": {"iphoneOS": args.iphoneos_sdk, "iphoneSimulator": args.iphonesimulator_sdk,
                "macOS": args.macos_sdk},
        "xcode": {"version": args.xcode_version, "build": args.xcode_build},
    }
    manifest = build_manifest(args.repo_root, configuration)
    encoded = canonical(manifest)
    if args.command == "write":
        args.manifest.parent.mkdir(parents=True, exist_ok=True)
        args.manifest.write_bytes(encoded)
    elif args.manifest.read_bytes() != encoded:
        raise InputDigestError("build-input manifest differs from the current configured Bazel closure")
    print(hashlib.sha256(encoded).hexdigest())
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except (InputDigestError, ValueError, OSError) as error:
        raise SystemExit(f"apple-runtime-input-digest: {error}") from error
