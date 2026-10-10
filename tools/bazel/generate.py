#!/usr/bin/env python3
"""Generate runtime BUILD files and crate_universe resolution metadata."""

from pathlib import Path
import argparse
import json
import tempfile
import tomllib

from cargo_graph import collect_packages, graph_feature_union, resolve_features, resolve_native_features, write_registry_workspace
from emit import render_package
from runtime_features import NATIVE_TOOLS_ROOTS


ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="report generated files that need updating without modifying them")
    parser.add_argument("--reset-lock", action="store_true", help="seed the flattened dependency lock from Cargo.lock before repinning")
    args = parser.parse_args()
    if args.check and args.reset_lock:
        parser.error("--check cannot reset dependency pins")
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text())
    patches = {name: (ROOT / spec["path"]).resolve() for name, spec in workspace["patch"]["crates-io"].items()}
    manifests = [ROOT / path / "Cargo.toml" for path in workspace["workspace"]["members"]]
    manifests.extend(path / "Cargo.toml" for path in patches.values())
    packages = collect_packages(set(manifests), patches)
    patch_names = {name: next(package.name for package in packages.values() if package.directory == directory) for name, directory in patches.items()}
    workspace_names = {package.name for package in packages.values() if package.directory.relative_to(ROOT).parts[0] != "vendor"}
    defaults = {name: ["default"] for name in workspace_names}
    authoring = {
        "nuxie": ["scripting"], "nuxie-scripting": ["luau"],
        "nuxie-binary": ["test-support"], "nuxie-ore-metal": ["with-rive-tools"],
        "nuxie-renderer": [], "nuxie-render-stream": [], "nuxie-sriv": [],
        "nuxie-project-data": [], "nuxie-project-data-scripting": [],
        "luaur-compiler": ["default"], "luaur-ast": ["default"],
        "luaur-bytecode": ["default"], "luaur-common": ["default"],
    }
    browser = {
        "nuxie": ["scripting"], "nuxie-scripting": ["luau", "async", "js-host-seed"],
        "nuxie-binary": ["test-support"], "nuxie-render-stream": [],
        "nuxie-project-data": [], "nuxie-project-data-scripting": [],
        "nuxie-video-host": [],
    }
    graphs = {
        "": resolve_native_features(packages, defaults),
        "test": resolve_native_features(packages, defaults, include_dev=True),
        "host": resolve_native_features(packages, host=True),
        "editor": resolve_native_features(packages, authoring),
        "publisher": resolve_features(packages, authoring, platform="wasm"),
        "webgpu": resolve_features(packages, {**browser, "nuxie-renderer": ["renderer-webgpu"]}, platform="wasm"),
        "webgl2": resolve_features(packages, {**browser, "nuxie-renderer": ["renderer-webgl2"]}, platform="wasm"),
        "apple": resolve_features(packages, {"nux-apple-product-extension": ["apple-runtime"]}, platform="apple"),
        "android": resolve_features(packages, {"nux-capi": ["android-vulkan", "scripting", "android-authored-wgsl"]}, platform="android"),
    }

    outputs = {}
    # Cargo.Bazel.lock is a separate crate_universe pin. Regeneration retains
    # the flattened Cargo lock that produced it; --reset-lock is explicit.
    registry_graphs = {
        "cargo": graphs,
        "native-tools-cargo": {**graphs, "native-tools": resolve_native_features(packages, NATIVE_TOOLS_ROOTS)},
    }
    for registry, selected_graphs in registry_graphs.items():
        with tempfile.TemporaryDirectory() as scratch:
            destination = Path(scratch)
            seed = ROOT / "bazel" / registry / "Cargo.lock"
            if args.reset_lock or not seed.exists():
                seed = ROOT / "Cargo.lock"
            write_registry_workspace(packages, destination, seed, patch_names,
                                     workspace_members=workspace_names,
                                     enabled_features=graph_feature_union(selected_graphs))
            for path in destination.rglob("*"):
                if path.is_file():
                    outputs[ROOT / "bazel" / registry / path.relative_to(destination)] = path.read_text()
        outputs[ROOT / "bazel" / registry / "BUILD.bazel"] = '\n'.join([
            'package(default_visibility = ["//visibility:public"])',
            'exports_files(glob(["**/Cargo.toml", "Cargo.lock", "Cargo.Bazel.lock", "**/lib.rs", "**/build.rs"]))',
        ]) + '\n'

    def label_for(name, variant):
        package = packages[name]
        suffix = "__" + variant if variant else ""
        return f"//{package.directory.relative_to(ROOT).as_posix()}:{name}{suffix}"

    for package in packages.values():
        content = render_package(
            package, graphs, label_for, fixture_labels=["//:fixtures"],
            unit_tests=package.name in workspace_names,
            integration_tests=package.name in workspace_names,
            examples=package.name in workspace_names,
        )
        outputs[package.directory / "BUILD.bazel"] = content
    root_lines = [
        "# Generated by tools/bazel/generate.py from the Cargo workspace.",
        'load("//bazel:source-path.bzl", "package_label")',
        'package(default_visibility = ["//visibility:public"])',
        'exports_files(["Cargo.toml", "Cargo.lock"])',
        'filegroup(name = "fixtures", srcs = glob(["fixtures/**", "defs/**", "silver-corpus.toml", "tools/renderer-timing-gate.sh"], allow_empty = True) + [package_label("crates/nuxie-schema:src/generated/schema.rs")])',
        'filegroup(name = "workspace", srcs = [package_label(label) for label in ' + json.dumps(sorted(label_for(name, "")[2:] for name in workspace_names)) + '])',
        'test_suite(name = "unit_tests", tests = [package_label(label) for label in ' + json.dumps(sorted(
            f"{package.directory.relative_to(ROOT).as_posix()}:{package.name}__unit_test"
            for package in packages.values()
            if package.directory.parent.name == "crates" and (package.directory / package.manifest.get("lib", {}).get("path", "src/lib.rs")).is_file()
        )) + '])',
    ]
    outputs[ROOT / "BUILD.bazel"] = "\n\n".join(root_lines) + "\n"
    changed = sorted(path for path, content in outputs.items() if not path.exists() or path.read_text() != content)
    if args.check:
        for path in changed:
            print(f"stale generated file: {path.relative_to(ROOT)}")
        return 1 if changed else 0
    for path in changed:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(outputs[path])
    print(f"Generated direct targets for {len(packages)} local packages and {len(graphs)} feature graphs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
