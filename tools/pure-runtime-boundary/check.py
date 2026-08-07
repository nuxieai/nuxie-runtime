#!/usr/bin/env python3
"""Ratchet baseline/ABI/oracle packages against product-owned imports.

Nuxie-only architecture guard; no pinned C++ behavior or correspondence row.
See docs/pure-runtime-boundary.md for the ratified contract and the exact migration
debt that this checker prevents from spreading.
"""

from __future__ import annotations

import argparse
import ast
import functools
import pathlib
import re
import sys
import tomllib
from collections.abc import Iterator


FORBIDDEN_DEPENDENCIES = {
    "nuxie",
    "nux-container",
    "nuxie-apple-adapter",
    "nuxie-authoring",
    "nuxie-browser-adapter",
    "nuxie-flow",
    "nuxie-product",
    "nuxie-product-scripting",
    "nuxie-project-data",
}
FORBIDDEN_DEPENDENCY_PREFIXES = (
    "nuxie-apple-",
    "nuxie-authoring-",
    "nuxie-browser-",
    "nuxie-flow-",
    "nuxie-product-",
    "nuxie-project-",
)

RUNTIME_REPOSITORY = "https://github.com/nuxieai/nuxie-runtime"
PRODUCT_REPOSITORY = "https://github.com/nuxieai/nuxie-product"
FULL_GIT_REVISION = re.compile(r"^[0-9a-f]{40}$")
AUDITED_SELF_PATCHES = {
    "nuxie": "crates/nuxie",
    "nuxie-scripting": "crates/nuxie-scripting",
}
EXTRACTED_PRODUCT_PATHS = (
    "crates/nux-container",
    "crates/nuxie-product-scripting",
)
# These packages are owned by product repositories and may not be restored as
# runtime workspace members even if they happen to avoid a forbidden edge.
FORBIDDEN_RUNTIME_WORKSPACE_PACKAGES = {"nuxie-apple-adapter"}

PRODUCT_ROOT_REEXPORT = re.compile(r"\bpub\s+use\b[^;]*;", re.DOTALL)

# All workspace packages are protected by default. These named packages are
# current/future owners above the baseline, plus the browser parity executable
# that deliberately drives the protected facade and generated oracle fixture.
# The separate browser-WebGPU guard prevents that tool from owning product
# lifecycle policy. Adding another exemption is an architecture-policy change
# reviewed in the same diff as the new package.
UNPROTECTED_WORKSPACE_PACKAGES = {
    "browser-renderer-smoke",
    "nuxie-authoring",
    "nuxie-flow",
    "nuxie-product",
    "nuxie-project-data",
}
EXTERNAL_OWNER_PACKAGES = {
    "nux-container",
    "nuxie-browser-adapter",
    "nuxie-product-scripting",
}
# Exemption means a package is an upward-facing owner or consumer. Protected
# packages must therefore never depend on any exempt package, even when its
# name does not follow one of the reserved product prefixes.
FORBIDDEN_DEPENDENCIES.update(UNPROTECTED_WORKSPACE_PACKAGES)

# The portable ABI reaches baseline facade symbols through nuxie. This is a
# permanent, narrow baseline edge rather than migration debt: the whole nuxie
# package is protected, while the C consumer is restricted to this exact edge
# and approved symbols.
PORTABLE_ABI_FACADE_EDGE = ("nux-capi", "nuxie")
PORTABLE_ABI_FACADE_ALLOWED_FORWARDED_FEATURES = {"renderer"}
PORTABLE_ABI_FACADE_ALLOWED_SYMBOLS = {
    "Artboard",
    "ArtboardInstance",
    "BlendMode",
    "ColorInt",
    "Factory",
    "File",
    "FillRule",
    "GpuCanvasPassState",
    "GpuCanvasPipelineShaders",
    "GpuCanvasPipelineState",
    "GpuCanvasPlan",
    "GpuCanvasShader",
    "ImageDecodeError",
    "ImageFilter",
    "ImageSampler",
    "ImageWrap",
    "Mat2D",
    "RawPath",
    "RenderBuffer",
    "RenderBufferFlags",
    "RenderBufferType",
    "RenderImage",
    "RenderMode",
    "RenderPaint",
    "RenderPaintStyle",
    "RenderPath",
    "Renderer",
    "RenderShader",
    "StateMachineInstance",
    "StrokeCap",
    "StrokeJoin",
    "ViewModelInstance",
    "WgpuFactory",
    "WgpuFrame",
}
PORTABLE_ABI_FACADE_PRODUCT_METHOD = re.compile(
    r"\b(?:prepare_flow_[A-Za-z0-9_]*|import_with_(?:trusted_scripts|"
    r"trusted_scripts_and_limits|script_capability|unsigned_scripts)|"
    r"FlowSession[A-Za-z0-9_]*|"
    r"Scene(?:Tx)?[A-Za-z0-9_]*|ProjectData[A-Za-z0-9_]*)\b"
)
PORTABLE_ABI_FORBIDDEN_VOCABULARY = re.compile(
    r"(?-i:\b[A-Za-z0-9_]*(?:Apple|CAMetal(?:Layer|Drawable)?|FlowSession|"
    r"NuxExperience|Experience(?:Context|Package|Session)|Project(?:DO|Data)|"
    r"PackageSession|NuxPackage|NuxArtifact)(?:[A-Z_][A-Za-z0-9_]*)?\b)|"
    r"(?-i:\b(?:NuxProduct[A-Za-z0-9_]*|[A-Za-z0-9_]*Product(?:Session|"
    r"Context|Package|ABI|Api|Host|Runtime|Operation|Result|Value)"
    r"[A-Za-z0-9_]*)\b)|"
    r"(?i:(?<![A-Za-z0-9])(?:[a-z0-9]+_)*(?:nux_)?(?:apple|flow_session|"
    r"experience|project_(?:do|data)|package_session|nux_package|nux_artifact)"
    r"(?:_[a-z0-9]+)*(?![A-Za-z0-9]))|"
    r"(?i:(?<![A-Za-z0-9])(?:[a-z0-9]+_)*(?:nux_)?product_(?:session|"
    r"context|package|abi|api|host|runtime|operation|result|value)"
    r"(?:_[a-z0-9]+)*(?![A-Za-z0-9]))|"
    r"(?i:\bnuxie[-_]product(?:[-_][A-Za-z0-9_-]+)?\b|"
    r"\bnux[-_]container\b)",
)
PORTABLE_ABI_CONTRACT_SUFFIXES = {
    ".c",
    ".cc",
    ".cpp",
    ".h",
    ".inc",
    ".m",
    ".md",
    ".mm",
    ".modulemap",
    ".rs",
    ".toml",
}
DIRECT_NUXIE_PATH = re.compile(r"\bnuxie\s*::\s*(?P<symbol>[A-Za-z_][A-Za-z0-9_]*)")
RUST_USE_STATEMENT = re.compile(r"\buse\b(?P<body>[^;]*);", re.DOTALL)
NUXIE_EXTERN_CRATE = re.compile(r"\bextern\s+crate\s+nuxie\b")
FILE_ASSOCIATED_ITEM = re.compile(
    r"(?:<\s*)?\bFile\b(?:\s*>)?\s*::\s*(?P<item>[A-Za-z_][A-Za-z0-9_]*)"
)
PORTABLE_ABI_FACADE_TYPE_ALIAS = re.compile(
    r"\btype\s+[A-Za-z_][A-Za-z0-9_]*(?:\s*<[^;=]*>)?\s*=\s*"
    r"(?:::)?(?:nuxie\s*::\s*)?(?:"
    + "|".join(sorted(PORTABLE_ABI_FACADE_ALLOWED_SYMBOLS))
    + r")\b"
)
PORTABLE_ABI_FACADE_ALLOWED_FILE_ASSOCIATED_ITEMS = {"import"}

# Exact third-party path providers intentionally excluded from the root Cargo
# workspace. New entries require an architecture-policy review; a blanket
# vendor/ prefix would let first-party helpers hide from the protected scan.
AUDITED_UNSCANNED_THIRD_PARTY_PATHS = {
    "vendor/luaur-ast-0.1.8",
    "vendor/luaur-bytecode-0.1.8",
    "vendor/luaur-common-0.1.8",
    "vendor/luaur-compiler-0.1.8",
    "vendor/luaur-rt-0.1.8",
    "vendor/luaur-vm-0.1.8",
    "vendor/wgpu-30.0.0",
    "vendor/wgpu-core-30.0.0",
    "vendor/wgpu-core-deps-apple-30.0.0",
    "vendor/wgpu-core-deps-emscripten-30.0.0",
    "vendor/wgpu-core-deps-wasm-30.0.0",
    "vendor/wgpu-core-deps-windows-linux-android-30.0.0",
    "vendor/wgpu-hal-30.0.0",
}

# These are file-level ratchet exceptions, not compliant dependencies. A new
# file containing either marker family fails. Deleting entries is allowed and
# should happen as the migration in docs/pure-runtime-boundary.md proceeds.
INTERNAL_DEBT_FILES = {
    "editor-gpu-tooling": set(),
    "apple-image-admission": set(),
    "apple-presentation": set(),
    # A single compatibility shim keeps pinned nuxie-dev authoring builds
    # source-compatible. It is compiled only with nuxie-binary's non-default
    # test-support feature and must not spread into protected consumers.
    "binary-authoring": {"crates/nuxie-binary/src/legacy_test_support.rs"},
    "browser-presentation": set(),
    "project-data": set(),
    "product-host-commands": set(),
}
INTERNAL_DEBT_MARKERS = {
    "editor-gpu-tooling": re.compile(
        r"\bGpuCanvas(?:Program|RenderPlan)\b|"
        r"\bpub\s+fn\s+(?:eval|register_source_module)\b|"
        r"\bpub\s+fn\s+load\s*\([^)]*\bsource\s*:\s*&str"
    ),
    "apple-image-admission": re.compile(
        r"\bAPPLE_SAFE_IMAGE_|\bvalidate_image_bytes\b"
    ),
    "apple-presentation": re.compile(
        r"\bApple(?:PresentationCompletion|Surface)\b|"
        r"\bSurfaceDisposition\b|\bCAMetalDrawable\b"
    ),
    "binary-authoring": re.compile(
        r"\bAuthoring(?:Property|Record|Value)\b|\bfrom_authoring_records\b"
    ),
    "browser-presentation": re.compile(
        r"\bBrowser(?:Factory|Frame|ResizeError)\b|\bbrowser_surface_lifecycle\b"
    ),
    "project-data": re.compile(r"\bProjectData|\bproject_data_converter\b"),
    "product-host-commands": re.compile(
        r"\bhost_commands\b|\bHost(?:Command|Value|CycleCheckpoint|EffectCheckpoint)\b|"
        r"\b(?:begin|rollback)_host_cycle\b|\b(?:checkpoint|rollback)_host_effects\b|"
        r"\bdrain_(?:flow_)?host_commands\b|\bhost_cycle_active\b|"
        r"\b(?:HostIdentifier|HostString|HostDepth|HostNodes|HostEdges|HostValueBytes|"
        r"Commands|CommandContent)\b|"
        r"\bScriptResourceLimit::(?:Host(?:Identifier|String|Depth|Nodes|Edges|ValueBytes)|"
        r"Commands|CommandContent)\b"
    ),
}

EXPLICIT_PRODUCT_PATH = re.compile(
    r"\b(?:nuxie::(?:flow_session|scene)|"
    r"nuxie_(?:authoring|flow|product(?:_scripting)?|project_data)|nux_container)::"
)
LOCAL_PRODUCT_MODULE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:mod|use)\s+"
    r"(?:(?:crate|self|super)::)*(?:authoring|flow_session)(?:::|\s*;)"
)
RUST_STRING_LITERAL = (
    r'(?P<literal>"(?:\\.|[^"\\])*"|'
    r'r(?P<hashes>#{0,255})"(?P<raw>.*?)"(?P=hashes))'
)
RUST_PATH_SOURCE_EDGE = re.compile(
    rf"\bpath\s*=\s*{RUST_STRING_LITERAL}", re.DOTALL
)
RUST_INCLUDE_SOURCE_EDGE = re.compile(
    rf"\binclude\s*!\s*\(\s*{RUST_STRING_LITERAL}\s*\)", re.DOTALL
)
RUST_INCLUDE_INVOCATION = re.compile(r"\binclude\s*!\s*[\(\[\{]")
RUST_DATA_INCLUDE_SOURCE_EDGE = re.compile(
    rf"\b(?P<macro>include_(?:bytes|str))\s*!\s*\(\s*"
    rf"{RUST_STRING_LITERAL}\s*\)",
    re.DOTALL,
)
RUST_DATA_INCLUDE_MANIFEST_EDGE = re.compile(
    rf"\b(?P<macro>include_(?:bytes|str))\s*!\s*\(\s*concat\s*!\s*\(\s*"
    rf'env\s*!\s*\(\s*"CARGO_MANIFEST_DIR"\s*\)\s*,\s*'
    rf"{RUST_STRING_LITERAL}\s*\)\s*\)",
    re.DOTALL,
)
RUST_DATA_INCLUDE_INVOCATION = re.compile(
    r"\binclude_(?:bytes|str)\s*!\s*[\(\[\{]"
)
RUST_PATH_ASSIGNMENT = re.compile(r"\bpath\s*=")
CFG_ATTRIBUTE_START = re.compile(r"#\s*\[\s*cfg\s*\(")
RUST_MODULE_AFTER_CFG = re.compile(
    r"\s*(?:#\s*\[[^\]]*\]\s*)*"
    r"(?:pub(?:\([^)]*\))?\s+)?mod\s+[A-Za-z_][A-Za-z0-9_]*\s*\{"
)
ALLOWED_DYNAMIC_INCLUDES = {
    "crates/nuxie-runtime/src/objects.rs": re.compile(
        r'include!\s*\(\s*concat!\s*\(\s*env!\s*\(\s*"OUT_DIR"\s*\)\s*,'
        r'\s*"/runtime_objects\.rs"\s*\)\s*\)'
    ),
}

BINARY_AUTHORING_COMPATIBILITY_FILE = (
    "crates/nuxie-binary/src/legacy_test_support.rs"
)
BINARY_AUTHORING_COMPATIBILITY_MODULE = re.compile(
    r'#\s*\[\s*cfg\s*\(\s*feature\s*=\s*"test-support"\s*\)\s*\]\s*'
    r"mod\s+legacy_test_support\s*;"
)


def normalized_package_name(name: str) -> str:
    return name.strip().replace("_", "-")


def is_forbidden_dependency(name: str) -> bool:
    normalized = normalized_package_name(name)
    return normalized in FORBIDDEN_DEPENDENCIES or normalized.startswith(
        FORBIDDEN_DEPENDENCY_PREFIXES
    )


def dependency_tables(
    value: object, path: tuple[str, ...] = ()
) -> Iterator[tuple[tuple[str, ...], dict[str, object]]]:
    if not isinstance(value, dict):
        return
    dependency_table_names = (
        "dependencies",
        "dev-dependencies",
        "build-dependencies",
    )
    for table_name in dependency_table_names:
        table = value.get(table_name)
        if isinstance(table, dict):
            yield (*path, table_name), table
    # Cargo dependency edges may additionally occur directly below
    # [target.<selector>]. Arbitrary package/tool metadata can also contain a
    # table named `dependencies`; it is data, not a Cargo edge.
    if path:
        return
    targets = value.get("target")
    if not isinstance(targets, dict):
        return
    for selector, target in targets.items():
        if not isinstance(target, dict):
            continue
        for table_name in dependency_table_names:
            table = target.get(table_name)
            if isinstance(table, dict):
                yield ("target", str(selector), table_name), table


def dependency_package(dependency_name: str, specification: object) -> str:
    if isinstance(specification, dict):
        package = specification.get("package")
        if isinstance(package, str):
            return package
    return dependency_name


def workspace_packages(
    repo_root: pathlib.Path,
) -> tuple[
    list[tuple[str, str, dict[str, object]]],
    dict[str, object],
    set[str],
    list[str],
]:
    errors: list[str] = []
    workspace_path = repo_root / "Cargo.toml"
    try:
        workspace_manifest = tomllib.loads(workspace_path.read_text())
    except (OSError, tomllib.TOMLDecodeError) as error:
        return [], {}, set(), [
            f"Cargo.toml: cannot parse workspace manifest: {error}"
        ]

    workspace = workspace_manifest.get("workspace")
    members = workspace.get("members") if isinstance(workspace, dict) else None
    if not isinstance(members, list) or not all(
        isinstance(member, str) for member in members
    ):
        return [], {}, set(), [
            "Cargo.toml: [workspace].members must be a string array"
        ]

    workspace_dependencies = workspace.get("dependencies", {})
    if not isinstance(workspace_dependencies, dict):
        return [], {}, set(), [
            "Cargo.toml: [workspace.dependencies] must be a table"
        ]
    excluded = workspace.get("exclude", [])
    if not isinstance(excluded, list) or not all(
        isinstance(pattern, str) for pattern in excluded
    ):
        return [], {}, set(), [
            "Cargo.toml: [workspace].exclude must be a string array"
        ]

    excluded_paths: set[str] = set()
    for excluded_path in excluded:
        resolved_excluded = (repo_root / excluded_path).resolve()
        try:
            excluded_paths.add(resolved_excluded.relative_to(repo_root).as_posix())
        except ValueError:
            errors.append(
                f"Cargo.toml: workspace exclude escapes repository: {excluded_path}"
            )

    def is_excluded(relative: str) -> bool:
        # Cargo treats exclude entries as literal paths, even though workspace
        # member entries support globs.
        return relative in excluded_paths

    member_paths: set[str] = set()
    if isinstance(workspace_manifest.get("package"), dict):
        member_paths.add(".")

    # Cargo configuration can override dependency providers with higher
    # precedence than the workspace manifest. Keep committed provider changes
    # in the root Cargo.toml, where this checker can resolve and audit them as
    # part of the protected graph.
    for cargo_config_relative in (".cargo/config.toml", ".cargo/config"):
        cargo_config_path = repo_root / cargo_config_relative
        if not cargo_config_path.is_file():
            continue
        try:
            cargo_config = tomllib.loads(cargo_config_path.read_text())
        except (OSError, tomllib.TOMLDecodeError) as error:
            errors.append(
                f"{cargo_config_relative}: cannot parse Cargo configuration: {error}"
            )
            continue
        for override_name in ("patch", "paths", "source"):
            if cargo_config.get(override_name):
                errors.append(
                    f"{cargo_config_relative}: dependency provider override "
                    f"[{override_name}] is not allowed; declare an audited "
                    "[patch] in Cargo.toml instead"
                )

    patches = workspace_manifest.get("patch", {})
    if not isinstance(patches, dict):
        errors.append("Cargo.toml: [patch] must be a table")
        patches = {}
    runtime_patches = patches.get(RUNTIME_REPOSITORY)
    if runtime_patches is not None:
        if not isinstance(runtime_patches, dict) or set(runtime_patches) != set(
            AUDITED_SELF_PATCHES
        ):
            errors.append(
                f"Cargo.toml: [patch.{RUNTIME_REPOSITORY!r}] must contain exactly "
                f"{sorted(AUDITED_SELF_PATCHES)!r}"
            )
        else:
            for package_name, expected_path in AUDITED_SELF_PATCHES.items():
                if runtime_patches.get(package_name) != {"path": expected_path}:
                    errors.append(
                        f"Cargo.toml: runtime self-patch {package_name!r} must resolve "
                        f"exactly to {expected_path!r}"
                    )
    for source_name, source_patches in patches.items():
        if not isinstance(source_patches, dict):
            errors.append(f"Cargo.toml: [patch.{source_name}] must be a table")
            continue
        for patch_name, specification in source_patches.items():
            if not isinstance(specification, dict):
                continue
            patch_path = specification.get("path")
            if not isinstance(patch_path, str):
                continue
            resolved_patch = (repo_root / patch_path).resolve()
            try:
                relative = resolved_patch.relative_to(repo_root).as_posix()
            except ValueError:
                errors.append(
                    f"Cargo.toml: path patch {patch_name!r} escapes repository: "
                    f"{patch_path}"
                )
                continue
            patch_manifest_path = resolved_patch / "Cargo.toml"
            try:
                patch_manifest = tomllib.loads(patch_manifest_path.read_text())
            except (OSError, tomllib.TOMLDecodeError) as error:
                errors.append(
                    f"Cargo.toml: cannot parse path patch {patch_name!r} provider "
                    f"{relative}/Cargo.toml: {error}"
                )
                continue
            patch_package = patch_manifest.get("package")
            patch_package_name = (
                patch_package.get("name") if isinstance(patch_package, dict) else None
            )
            if not isinstance(patch_package_name, str) or not patch_package_name.strip():
                errors.append(
                    f"Cargo.toml: path patch {patch_name!r} provider "
                    f"{relative}/Cargo.toml requires [package].name"
                )
                continue
            audited_self_patch = (
                source_name == RUNTIME_REPOSITORY
                and AUDITED_SELF_PATCHES.get(patch_name) == relative
                and patch_package_name == patch_name
            )
            if is_forbidden_dependency(patch_package_name) and not audited_self_patch:
                errors.append(
                    f"Cargo.toml: path patch {patch_name!r} resolves to product "
                    f"package {patch_package_name!r}"
                )
            if is_excluded(relative):
                if relative not in AUDITED_UNSCANNED_THIRD_PARTY_PATHS:
                    errors.append(
                        f"Cargo.toml: path patch {patch_name!r} resolves to excluded "
                        f"provider {relative!r} outside the protected workspace scan"
                    )
                continue
            member_paths.add(relative)

    replacements = workspace_manifest.get("replace", {})
    if not isinstance(replacements, dict):
        errors.append("Cargo.toml: [replace] must be a table")
    else:
        for replacement_name in replacements:
            errors.append(
                f"Cargo.toml: deprecated [replace] override {replacement_name!r} "
                "is not allowed; use an audited [patch] or workspace dependency"
            )

    for member in members:
        matches = (
            sorted(repo_root.glob(member))
            if any(character in member for character in "*?[")
            else [repo_root / member]
        )
        if not matches:
            errors.append(
                f"Cargo.toml: workspace member pattern {member!r} matched nothing"
            )
            continue
        for match in matches:
            try:
                relative = match.relative_to(repo_root).as_posix()
            except ValueError:
                errors.append(f"Cargo.toml: workspace member escapes repository: {member}")
                continue
            if not is_excluded(relative):
                member_paths.add(relative)

    packages: list[tuple[str, str, dict[str, object]]] = []
    parsed_paths: set[str] = set()
    pending_paths = sorted(member_paths)
    while pending_paths:
        relative = pending_paths.pop(0)
        if relative in parsed_paths:
            continue
        parsed_paths.add(relative)
        manifest_path = repo_root / relative / "Cargo.toml"
        try:
            manifest = tomllib.loads(manifest_path.read_text())
        except (OSError, tomllib.TOMLDecodeError) as error:
            errors.append(f"{relative}/Cargo.toml: cannot parse manifest: {error}")
            continue
        package = manifest.get("package")
        name = package.get("name") if isinstance(package, dict) else None
        if not isinstance(name, str) or not name.strip():
            errors.append(f"{relative}/Cargo.toml: [package].name is required")
            continue
        packages.append((relative, normalized_package_name(name), manifest))

        for _, dependencies in dependency_tables(manifest):
            for dependency_name, specification in dependencies.items():
                effective_specification = specification
                inherited = (
                    isinstance(specification, dict)
                    and specification.get("workspace") is True
                )
                if inherited:
                    effective_specification, inheritance_error = (
                        inherited_dependency_specification(
                            dependency_name, specification, workspace_dependencies
                        )
                    )
                    if inheritance_error is not None:
                        continue
                if not isinstance(effective_specification, dict):
                    continue
                dependency_path = effective_specification.get("path")
                if not isinstance(dependency_path, str):
                    continue
                # Cargo resolves paths declared in [workspace.dependencies]
                # from the workspace root, not from the inheriting member.
                dependency_base = repo_root if inherited else manifest_path.parent
                resolved_path = (dependency_base / dependency_path).resolve()
                try:
                    implicit_relative = resolved_path.relative_to(repo_root).as_posix()
                except ValueError:
                    continue
                if not (resolved_path / "Cargo.toml").is_file():
                    continue
                if is_excluded(implicit_relative):
                    continue
                if implicit_relative not in parsed_paths:
                    pending_paths.append(implicit_relative)
                    pending_paths.sort()

    return packages, workspace_dependencies, excluded_paths, errors


def portable_abi_facade_edge_error(
    package_name: str,
    table_path: tuple[str, ...],
    dependency_name: str,
    resolved_name: str,
    specification: object,
    resolved_path: str | None,
) -> str | None:
    if (package_name, normalized_package_name(resolved_name)) != PORTABLE_ABI_FACADE_EDGE:
        return "not-approved"
    if table_path != ("dependencies",) or not isinstance(specification, dict):
        return "portable ABI facade edge is only approved in [dependencies]"
    if normalized_package_name(dependency_name) != "nuxie":
        return "portable ABI facade edge must use dependency key 'nuxie'"
    if resolved_path != "crates/nuxie":
        return "portable ABI facade edge must resolve to local crates/nuxie"
    if specification.get("default-features") is not False:
        return "portable ABI facade edge must disable default features"
    features = specification.get("features", [])
    if features not in (None, []) and features != ():
        return "portable ABI facade edge cannot enable dependency features"
    return None


def nuxie_self_test_dependency_error(
    package_name: str,
    package_path: str,
    table_path: tuple[str, ...],
    dependency_name: str,
    resolved_name: str,
    specification: object,
    resolved_path: str | None,
) -> str | None:
    if package_name != "nuxie" or normalized_package_name(resolved_name) != "nuxie":
        return "not-approved"
    if resolved_path != package_path:
        return "not-approved"
    if table_path != ("dev-dependencies",) or not isinstance(specification, dict):
        return "nuxie self edge is only approved in [dev-dependencies]"
    if normalized_package_name(dependency_name) != "nuxie":
        return "nuxie self edge must use dependency key 'nuxie'"
    if specification.get("default-features") is not False:
        return "nuxie self edge must disable default features"
    if specification.get("features") != ["test-support"]:
        return "nuxie self edge may enable only the test-support feature"
    return None


def _blank_non_newlines(characters: list[str], start: int, end: int) -> None:
    for index in range(start, end):
        if characters[index] not in "\r\n":
            characters[index] = " "


def _strip_rust(source: str, *, blank_literals: bool) -> str:
    """Blank comments and optionally literals while preserving positions."""

    characters = list(source)
    length = len(source)
    index = 0
    while index < length:
        if source.startswith("//", index):
            end = source.find("\n", index + 2)
            end = length if end < 0 else end
            _blank_non_newlines(characters, index, end)
            index = end
            continue
        if source.startswith("/*", index):
            depth = 1
            end = index + 2
            while end < length and depth:
                if source.startswith("/*", end):
                    depth += 1
                    end += 2
                elif source.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            _blank_non_newlines(characters, index, end)
            index = end
            continue

        char_start = index
        if source.startswith("b'", index):
            char_start += 1
        if source[char_start] == "'":
            end = char_start + 1
            if end < length and source[end] == "\\":
                end += 2
            else:
                end += 1
            if end < length and source[end] == "'":
                end += 1
                if blank_literals:
                    _blank_non_newlines(characters, index, end)
                index = end
                continue

        raw_start = index
        if source.startswith(("br", "cr"), index):
            raw_start += 1
        if source.startswith("r", raw_start):
            quote = raw_start + 1
            while quote < length and source[quote] == "#":
                quote += 1
            if quote < length and source[quote] == '"':
                hashes = source[raw_start + 1 : quote]
                delimiter = '"' + hashes
                end = source.find(delimiter, quote + 1)
                end = length if end < 0 else end + len(delimiter)
                if blank_literals:
                    _blank_non_newlines(characters, index, end)
                index = end
                continue

        quote = index
        if source[index] in "bc" and index + 1 < length and source[index + 1] == '"':
            quote += 1
        if source[quote] == '"':
            end = quote + 1
            while end < length:
                if source[end] == "\\":
                    end += 2
                    continue
                end += 1
                if source[end - 1] == '"':
                    break
            if blank_literals:
                _blank_non_newlines(characters, index, min(end, length))
            index = end
            continue
        index += 1
    return "".join(characters)


def strip_rust_non_code(source: str) -> str:
    """Blank comments and string literals while preserving line positions."""

    return _strip_rust(source, blank_literals=True)


def strip_rust_comments(source: str) -> str:
    """Blank comments while preserving literals and line positions."""

    return _strip_rust(source, blank_literals=False)


def rust_code_pattern_is_present(pattern: re.Pattern[str], source: str) -> bool:
    """Match code while retaining literals needed to interpret attributes."""

    comments_removed = strip_rust_comments(source)
    code_mask = strip_rust_non_code(source)
    return any(
        not code_mask[match.start()].isspace()
        for match in pattern.finditer(comments_removed)
    )


def feature_reaches(
    features: dict[str, object], start: str, target: str, visited: set[str] | None = None
) -> bool:
    """Follow Cargo's local feature aliases without chasing dependency features."""

    if start == target:
        return True
    if visited is None:
        visited = set()
    if start in visited:
        return False
    visited.add(start)
    activations = features.get(start)
    if not isinstance(activations, list):
        return False
    return any(
        isinstance(activation, str)
        and "/" not in activation
        and not activation.startswith("dep:")
        and feature_reaches(features, activation, target, visited)
        for activation in activations
    )


def matching_delimiter(
    source: str, opening_index: int, opening: str, closing: str
) -> int | None:
    depth = 0
    for index in range(opening_index, len(source)):
        character = source[index]
        if character == opening:
            depth += 1
        elif character == closing:
            depth -= 1
            if depth == 0:
                return index
    return None


def split_cfg_arguments(arguments: str) -> list[str]:
    parts = []
    start = 0
    depth = 0
    for index, character in enumerate(arguments):
        if character == "(":
            depth += 1
        elif character == ")":
            depth = max(0, depth - 1)
        elif character == "," and depth == 0:
            parts.append(arguments[start:index].strip())
            start = index + 1
    parts.append(arguments[start:].strip())
    return [part for part in parts if part]


def cfg_predicate_implies_test(predicate: str) -> bool:
    predicate = predicate.strip()
    if predicate == "test":
        return True
    call = re.fullmatch(r"(?P<operator>all|any|not)\s*\((?P<body>.*)\)", predicate, re.DOTALL)
    if call is None:
        return False
    arguments = split_cfg_arguments(call.group("body"))
    implications = [cfg_predicate_implies_test(argument) for argument in arguments]
    if call.group("operator") == "all":
        return any(implications)
    if call.group("operator") == "any":
        return bool(implications) and all(implications)
    # Conservatively reject `not`, including double negation, rather than
    # attempting to prove a more complex predicate test-only.
    return False


def cfg_test_module_ranges(source: str) -> list[tuple[int, int]]:
    """Return ranges for modules whose cfg predicate necessarily requires test."""

    ranges = []
    for cfg_match in CFG_ATTRIBUTE_START.finditer(source):
        opening_parenthesis = source.find("(", cfg_match.start(), cfg_match.end())
        closing_parenthesis = matching_delimiter(
            source, opening_parenthesis, "(", ")"
        )
        if closing_parenthesis is None:
            continue
        predicate = source[opening_parenthesis + 1 : closing_parenthesis]
        if not cfg_predicate_implies_test(predicate):
            continue
        closing_bracket = source.find("]", closing_parenthesis + 1)
        if closing_bracket < 0:
            continue
        module_match = RUST_MODULE_AFTER_CFG.match(source, closing_bracket + 1)
        if module_match is None:
            continue
        opening_brace = source.find("{", module_match.start(), module_match.end())
        if opening_brace < 0:
            continue
        closing_brace = matching_delimiter(source, opening_brace, "{", "}")
        if closing_brace is not None:
            ranges.append((cfg_match.start(), closing_brace + 1))
    return ranges


def debt_match_is_within_approved_scope(
    family: str,
    relative: str,
    match: re.Match[str],
    test_module_ranges: list[tuple[int, int]],
) -> bool:
    if family != "binary-authoring" or relative != "crates/nuxie/src/lib.rs":
        return True
    return any(
        start <= match.start() < end for start, end in test_module_ranges
    )


def portable_abi_facade_feature_errors(
    package: str, package_name: str, manifest: dict[str, object]
) -> list[str]:
    if package_name != PORTABLE_ABI_FACADE_EDGE[0]:
        return []
    features = manifest.get("features", {})
    if not isinstance(features, dict):
        return [f"{package}/Cargo.toml: [features] must be a table"]
    errors = []
    for feature_name, activations in features.items():
        if not isinstance(activations, list):
            errors.append(
                f"{package}/Cargo.toml: feature {feature_name!r} must be an array"
            )
            continue
        for activation in activations:
            if not isinstance(activation, str):
                continue
            match = re.fullmatch(r"nuxie\??/([A-Za-z0-9_-]+)", activation)
            if (
                match
                and match.group(1)
                not in PORTABLE_ABI_FACADE_ALLOWED_FORWARDED_FEATURES
            ):
                errors.append(
                    f"{package}/Cargo.toml: feature {feature_name!r} forwards "
                    f"forbidden portable ABI facade feature {match.group(1)!r}"
                )
    return errors


def portable_abi_facade_source_errors(relative: str, source: str) -> list[str]:
    errors = []
    for match in NUXIE_EXTERN_CRATE.finditer(source):
        line = source.count("\n", 0, match.start()) + 1
        errors.append(
            f"{relative}:{line}: portable ABI facade extern-crate imports are not approved"
        )
    for match in RUST_USE_STATEMENT.finditer(source):
        body = match.group("body").strip()
        if re.search(r"\bnuxie\b", body) is None:
            continue
        imported_symbols: list[str] | None = None
        direct = re.fullmatch(r"nuxie\s*::\s*([A-Za-z_][A-Za-z0-9_]*)", body)
        grouped = re.fullmatch(r"nuxie\s*::\s*\{([^{}]*)\}", body, re.DOTALL)
        if direct is not None:
            imported_symbols = [direct.group(1)]
        elif grouped is not None:
            items = [item.strip() for item in grouped.group(1).split(",")]
            if all(re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", item) for item in items if item):
                imported_symbols = [item for item in items if item]
        line = source.count("\n", 0, match.start()) + 1
        if imported_symbols is None:
            errors.append(
                f"{relative}:{line}: portable ABI facade use tree is not an approved flat import"
            )
            continue
        for symbol in imported_symbols:
            if symbol not in PORTABLE_ABI_FACADE_ALLOWED_SYMBOLS:
                errors.append(
                    f"{relative}:{line}: portable ABI facade symbol {symbol!r} is not approved"
                )
    for match in PORTABLE_ABI_FACADE_TYPE_ALIAS.finditer(source):
        line = source.count("\n", 0, match.start()) + 1
        errors.append(
            f"{relative}:{line}: type aliases of portable ABI facade symbols are not approved"
        )
    for match in DIRECT_NUXIE_PATH.finditer(source):
        symbol = match.group("symbol")
        if symbol not in PORTABLE_ABI_FACADE_ALLOWED_SYMBOLS:
            line = source.count("\n", 0, match.start()) + 1
            errors.append(
                f"{relative}:{line}: portable ABI facade symbol {symbol!r} is not approved"
            )
    for match in re.finditer(r"\bimpl\b(?P<header>[^{};]*)\{", source, re.DOTALL):
        header = match.group("header")
        symbol_pattern = "|".join(sorted(PORTABLE_ABI_FACADE_ALLOWED_SYMBOLS))
        target = re.search(
            rf"\bfor\s+(?:\(\s*)*(?:::)?(?:nuxie\s*::\s*)?"
            rf"(?:{symbol_pattern})\b",
            header,
        )
        inherent = None
        if re.search(r"\bfor\b", header) is None:
            inherent = re.match(
                rf"\s*(?:<[^>]*>\s*)?(?:::)?(?:nuxie\s*::\s*)?"
                rf"(?:{symbol_pattern})\b",
                header,
            )
        if target is not None or inherent is not None:
            line = source.count("\n", 0, match.start()) + 1
            errors.append(
                f"{relative}:{line}: impls targeting portable ABI facade symbols are not approved"
            )
    for match in FILE_ASSOCIATED_ITEM.finditer(source):
        item = match.group("item")
        if item not in PORTABLE_ABI_FACADE_ALLOWED_FILE_ASSOCIATED_ITEMS:
            line = source.count("\n", 0, match.start()) + 1
            errors.append(
                f"{relative}:{line}: File associated item {item!r} is not in the "
                "approved baseline facade surface"
            )
    for match in PORTABLE_ABI_FACADE_PRODUCT_METHOD.finditer(source):
        line = source.count("\n", 0, match.start()) + 1
        errors.append(
            f"{relative}:{line}: portable ABI facade product method/type {match.group(0)!r} "
            "is not approved"
        )
    return errors


def portable_abi_vocabulary_errors(
    repo_root: pathlib.Path, package_root: pathlib.Path
) -> list[str]:
    """Reject product/platform terms anywhere in the portable ABI contract.

    Unlike the general Rust source scan, this intentionally includes comments,
    headers, smoke sources, module maps, and manifests. Naming the portable ABI
    as an Apple or product surface is itself an ownership regression even when
    the token does not compile into a dependency edge.
    """

    errors = []
    for path in sorted(package_root.rglob("*")):
        if not path.is_file() or path.suffix not in PORTABLE_ABI_CONTRACT_SUFFIXES:
            continue
        package_relative_parts = path.relative_to(package_root).parts
        if package_relative_parts and package_relative_parts[0] == "target":
            continue
        relative = path.relative_to(repo_root).as_posix()
        try:
            source = path.read_text()
        except OSError as error:
            errors.append(f"{relative}: cannot read portable ABI contract: {error}")
            continue
        for match in PORTABLE_ABI_FORBIDDEN_VOCABULARY.finditer(source):
            line = source.count("\n", 0, match.start()) + 1
            errors.append(
                f"{relative}:{line}: portable ABI contains product/Apple vocabulary "
                f"{match.group(0)!r}"
            )
    return errors


def missing_debt_exception_errors(
    repo_root: pathlib.Path,
    observed_debt: dict[str, set[str]],
    debt_files: dict[str, set[str]],
) -> list[str]:
    if not (repo_root / ".git").exists():
        return []
    return [
        f"{relative}: missing {family} boundary debt exception file; remove the allowlist entry"
        for family, exceptions in debt_files.items()
        for relative in sorted(exceptions)
        if not (repo_root / relative).is_file()
        and relative not in observed_debt.get(family, set())
    ]


def package_rust_sources(
    package_root: pathlib.Path,
    manifest: dict[str, object],
    separate_package_roots: set[pathlib.Path] | None = None,
) -> Iterator[pathlib.Path]:
    package_root = package_root.resolve()
    separate_package_roots = separate_package_roots or {package_root}
    sources: set[pathlib.Path] = set()
    source_tree_roots = {
        (package_root / conventional).resolve()
        for conventional in ("src", "examples", "tests", "benches")
    }
    package = manifest.get("package")
    build = package.get("build") if isinstance(package, dict) else None
    if isinstance(build, str):
        build_path = (package_root / build).resolve()
        sources.add(build_path)
        source_tree_roots.add(build_path.parent)
    elif build is not False:
        build_script = package_root / "build.rs"
        if build_script.is_file():
            sources.add(build_script.resolve())

    target_tables: list[object] = [manifest.get("lib")]
    for table_name in ("bin", "example", "test", "bench"):
        table = manifest.get(table_name)
        target_tables.extend(table if isinstance(table, list) else [table])
    for target in target_tables:
        target_path = target.get("path") if isinstance(target, dict) else None
        if isinstance(target_path, str):
            resolved_target = (package_root / target_path).resolve()
            sources.add(resolved_target)
            source_tree_roots.add(resolved_target.parent)

    # A custom target can place its module tree anywhere below the package
    # root. Scan every Rust file there, not only Cargo's conventional folders.
    for path in package_root.rglob("*.rs"):
        relative_parts = path.relative_to(package_root).parts
        if relative_parts and relative_parts[0] in {"target", ".git"}:
            continue
        # A non-virtual workspace root may contain member or explicitly
        # excluded package roots. Outside this package's actual source trees,
        # their sources are separately owned rather than claimed by the root.
        separate_owner = next(
            (
                owner
                for owner in separate_package_roots
                if package_root in owner.parents and owner in path.parents
            ),
            None,
        )
        if separate_owner is not None and not any(
            tree == separate_owner
            or tree in separate_owner.parents
            or separate_owner in tree.parents
            for tree in source_tree_roots
        ):
            continue
        sources.add(path.resolve())
    yield from sorted(sources)


@functools.cache
def manifest_declares_package(manifest_path: pathlib.Path) -> bool:
    if not manifest_path.is_file():
        return False
    try:
        manifest = tomllib.loads(manifest_path.read_text())
    except (OSError, tomllib.TOMLDecodeError):
        return False
    return isinstance(manifest.get("package"), dict)


def rust_string_literal_value(match: re.Match[str]) -> str | None:
    raw = match.group("raw")
    if raw is not None:
        return raw
    try:
        value = ast.literal_eval(match.group("literal"))
    except (SyntaxError, ValueError):
        return None
    return value if isinstance(value, str) else None


def rust_attribute_ranges(code_mask: str) -> Iterator[tuple[int, int]]:
    for match in re.finditer(r"#\s*\[", code_mask):
        open_bracket = code_mask.find("[", match.start(), match.end())
        depth = 1
        cursor = open_bracket + 1
        while cursor < len(code_mask) and depth:
            if code_mask[cursor] == "[":
                depth += 1
            elif code_mask[cursor] == "]":
                depth -= 1
            cursor += 1
        if depth == 0:
            yield match.start(), cursor


def source_edge_boundary_error(
    relative: str,
    source_path: pathlib.Path,
    package_root: pathlib.Path,
    source: str,
    match: re.Match[str],
    edge_kind: str,
) -> str | None:
    source_edge = rust_string_literal_value(match)
    line = source.count("\n", 0, match.start()) + 1
    if source_edge is None:
        return (
            f"{relative}:{line}: protected source {edge_kind} path "
            "could not be verified"
        )
    resolved = (source_path.parent / source_edge).resolve()
    try:
        resolved.relative_to(package_root)
    except ValueError:
        crosses_boundary = True
    else:
        crosses_boundary = any(
            manifest_declares_package(parent / "Cargo.toml")
            for parent in resolved.parents
            if parent != package_root and package_root in parent.parents
        )
    if crosses_boundary:
        return (
            f"{relative}:{line}: protected source {edge_kind} crosses "
            f"a package boundary to {resolved}"
        )
    return None


def cross_package_source_edge_errors(
    relative: str,
    source_path: pathlib.Path,
    package_root: pathlib.Path,
    repo_root: pathlib.Path,
    source: str,
) -> list[str]:
    errors = []
    package_root = package_root.resolve()
    code_mask = strip_rust_non_code(source)
    attribute_ranges = list(rust_attribute_ranges(code_mask))
    verified_path_starts = set()
    for match in RUST_PATH_SOURCE_EDGE.finditer(source):
        if code_mask[match.start()].isspace():
            continue
        if not any(start <= match.start() < end for start, end in attribute_ranges):
            continue
        verified_path_starts.add(match.start())
        preceding_code = code_mask[: match.start()]
        if preceding_code.count("{") != preceding_code.count("}"):
            line = source.count("\n", 0, match.start()) + 1
            errors.append(
                f"{relative}:{line}: protected source path attribute inside an "
                "inline/block context could not be verified"
            )
            continue
        error = source_edge_boundary_error(
            relative, source_path, package_root, source, match, "path attribute"
        )
        if error is not None:
            errors.append(error)
    for start, end in attribute_ranges:
        for assignment in RUST_PATH_ASSIGNMENT.finditer(code_mask, start, end):
            if assignment.start() not in verified_path_starts:
                line = source.count("\n", 0, assignment.start()) + 1
                errors.append(
                    f"{relative}:{line}: protected source path attribute form "
                    "could not be verified"
                )

    verified_include_starts = set()
    for match in RUST_INCLUDE_SOURCE_EDGE.finditer(source):
        if code_mask[match.start()].isspace():
            continue
        verified_include_starts.add(match.start())
        error = source_edge_boundary_error(
            relative, source_path, package_root, source, match, "include!"
        )
        if error is not None:
            errors.append(error)
    allowed_dynamic = ALLOWED_DYNAMIC_INCLUDES.get(relative)
    for invocation in RUST_INCLUDE_INVOCATION.finditer(code_mask):
        if invocation.start() in verified_include_starts:
            continue
        if allowed_dynamic is not None and allowed_dynamic.match(
            source, invocation.start()
        ):
            continue
        line = source.count("\n", 0, invocation.start()) + 1
        errors.append(
            f"{relative}:{line}: protected source include! form could not be verified"
        )

    verified_data_include_starts = set()
    data_include_edges = [
        (match, (source_path.parent / rust_string_literal_value(match)).resolve())
        for match in RUST_DATA_INCLUDE_SOURCE_EDGE.finditer(source)
        if not code_mask[match.start()].isspace()
        and rust_string_literal_value(match) is not None
    ]
    data_include_edges.extend(
        (
            match,
            pathlib.Path(f"{package_root}{rust_string_literal_value(match)}").resolve(),
        )
        for match in RUST_DATA_INCLUDE_MANIFEST_EDGE.finditer(source)
        if not code_mask[match.start()].isspace()
        and rust_string_literal_value(match) is not None
    )
    for match, resolved in data_include_edges:
        verified_data_include_starts.add(match.start())
        if code_mask[match.start()].isspace():
            continue
        line = source.count("\n", 0, match.start()) + 1
        edge_kind = f"{match.group('macro')}!"
        try:
            resolved.relative_to(repo_root)
        except ValueError:
            errors.append(
                f"{relative}:{line}: protected source {edge_kind} escapes "
                f"the repository to {resolved}"
            )
            continue
        owning_package = next(
            (
                parent
                for parent in (resolved, *resolved.parents)
                if parent == repo_root or repo_root in parent.parents
                if manifest_declares_package(parent / "Cargo.toml")
            ),
            None,
        )
        if owning_package is not None and owning_package != package_root:
            errors.append(
                f"{relative}:{line}: protected source {edge_kind} crosses "
                f"a package boundary to {resolved}"
            )
    for invocation in RUST_DATA_INCLUDE_INVOCATION.finditer(code_mask):
        if invocation.start() in verified_data_include_starts:
            continue
        line = source.count("\n", 0, invocation.start()) + 1
        errors.append(
            f"{relative}:{line}: protected source data include form could not be verified"
        )
    return errors


def inherited_dependency_specification(
    dependency_name: str,
    specification: dict[str, object],
    workspace_dependencies: dict[str, object],
) -> tuple[object | None, str | None]:
    inherited = workspace_dependencies.get(dependency_name)
    if inherited is None:
        return None, (
            f"inherited dependency {dependency_name!r} is absent from "
            "[workspace.dependencies]"
        )
    if not isinstance(inherited, dict):
        inherited = {"version": inherited}
    effective = dict(inherited)
    # Cargo permits member declarations to add features and optionality, but
    # `default-features = false` cannot turn off defaults unless the workspace
    # declaration also disables them. Model the resolved edge, not the text of
    # the inheriting table.
    if "optional" in specification:
        effective["optional"] = specification["optional"]
    workspace_defaults = inherited.get("default-features", True) is not False
    local_enables_defaults = specification.get("default-features") is True
    effective["default-features"] = workspace_defaults or local_enables_defaults
    inherited_features = inherited.get("features", [])
    local_features = specification.get("features", [])
    if not isinstance(inherited_features, list) or not isinstance(local_features, list):
        effective["features"] = local_features
    else:
        effective["features"] = list(
            dict.fromkeys([*inherited_features, *local_features])
        )
    return effective, None


def check_repository(
    repo_root: pathlib.Path,
) -> tuple[list[str], dict[str, set[str]], int, int]:
    packages, workspace_dependencies, excluded_paths, errors = workspace_packages(
        repo_root
    )
    separate_package_roots = {
        (repo_root / package).resolve() for package, _, _ in packages
    }
    for excluded in excluded_paths:
        resolved_excluded = (repo_root / excluded).resolve()
        if manifest_declares_package(resolved_excluded / "Cargo.toml"):
            separate_package_roots.add(resolved_excluded)
    observed_debt = {family: set() for family in INTERNAL_DEBT_FILES}
    reported_debt_spread: set[tuple[str, str]] = set()
    dependency_table_count = 0
    protected_count = 0
    package_paths = {package for package, _, _ in packages}

    for package, package_name, _ in packages:
        if package_name in EXTERNAL_OWNER_PACKAGES:
            errors.append(
                f"{package}/Cargo.toml: package {package_name!r} belongs to its "
                "external product/platform owner, not nuxie-runtime"
            )

    # This repository's ratified ownership contract makes these paths and the
    # remaining staging host's provider shape part of the live boundary. Unit
    # fixtures omit the contract document and continue to exercise the generic
    # package scanner independently.
    if (repo_root / "docs/product-crate-seams.md").is_file():
        for extracted_path in EXTRACTED_PRODUCT_PATHS:
            if (repo_root / extracted_path).exists():
                errors.append(
                    f"{extracted_path}: extracted product source must remain owned "
                    f"by {PRODUCT_REPOSITORY}"
                )
        product_package = next(
            (
                manifest
                for package, package_name, manifest in packages
                if package == "crates/nuxie-product" and package_name == "nuxie-product"
            ),
            None,
        )
        if product_package is not None:
            dependencies = product_package.get("dependencies", {})
            product_scripting = (
                dependencies.get("nuxie-product-scripting")
                if isinstance(dependencies, dict)
                else None
            )
            valid_product_scripting = (
                isinstance(product_scripting, dict)
                and product_scripting.get("git") == PRODUCT_REPOSITORY
                and isinstance(product_scripting.get("rev"), str)
                and FULL_GIT_REVISION.fullmatch(product_scripting["rev"]) is not None
                and product_scripting.get("optional") is True
                and set(product_scripting) == {"git", "rev", "optional"}
            )
            if not valid_product_scripting:
                errors.append(
                    "crates/nuxie-product/Cargo.toml: nuxie-product-scripting must be "
                    "an optional exact-revision dependency on the canonical product repository"
                )
            root_manifest = tomllib.loads((repo_root / "Cargo.toml").read_text())
            if root_manifest.get("patch", {}).get(RUNTIME_REPOSITORY) != {
                name: {"path": path} for name, path in AUDITED_SELF_PATCHES.items()
            }:
                errors.append(
                    "Cargo.toml: the remaining product host requires the exact audited "
                    "runtime self-patches"
                )

    for package, package_name, _ in packages:
        if package_name in FORBIDDEN_RUNTIME_WORKSPACE_PACKAGES:
            errors.append(
                f"{package}/Cargo.toml: {package_name} is owned by nuxie-ios, "
                "not the runtime workspace"
            )

    for package, package_name, manifest in packages:
        if package_name in UNPROTECTED_WORKSPACE_PACKAGES:
            continue
        protected_count += 1
        package_root = repo_root / package
        if package_name == "nuxie-binary":
            features = manifest.get("features", {})
            if isinstance(features, dict) and feature_reaches(
                features, "default", "test-support"
            ):
                errors.append(
                    f"{package}/Cargo.toml: test-support must remain outside "
                    "the default shipping feature set"
                )
        errors.extend(
            portable_abi_facade_feature_errors(package, package_name, manifest)
        )
        if package_name == PORTABLE_ABI_FACADE_EDGE[0]:
            errors.extend(portable_abi_vocabulary_errors(repo_root, package_root))

        for table_path, dependencies in dependency_tables(manifest):
            dependency_table_count += 1
            for dependency_name, specification in dependencies.items():
                effective_specification = specification
                if isinstance(specification, dict) and specification.get("workspace") is True:
                    effective_specification, inheritance_error = (
                        inherited_dependency_specification(
                            dependency_name, specification, workspace_dependencies
                        )
                    )
                    if inheritance_error is not None:
                        errors.append(
                            f"{package}/Cargo.toml: {inheritance_error}"
                        )
                        continue
                resolved_name = dependency_package(
                    dependency_name, effective_specification
                )
                inherited = (
                    isinstance(specification, dict)
                    and specification.get("workspace") is True
                )
                dependency_path = (
                    effective_specification.get("path")
                    if isinstance(effective_specification, dict)
                    else None
                )
                resolved_path = None
                if isinstance(dependency_path, str):
                    dependency_base = repo_root if inherited else package_root
                    try:
                        resolved_path = (
                            (dependency_base / dependency_path)
                            .resolve()
                            .relative_to(repo_root)
                            .as_posix()
                        )
                    except ValueError:
                        pass
                if (
                    resolved_path is not None
                    and (repo_root / resolved_path / "Cargo.toml").is_file()
                    and resolved_path not in package_paths
                    and resolved_path not in AUDITED_UNSCANNED_THIRD_PARTY_PATHS
                ):
                    errors.append(
                        f"{package}/Cargo.toml: in-repo path dependency "
                        f"{dependency_name!r} resolves to {resolved_path!r}, which is "
                        "outside the protected workspace scan"
                    )
                if is_forbidden_dependency(dependency_name) or is_forbidden_dependency(
                    resolved_name
                ):
                    table = ".".join(table_path)
                    edge_error = portable_abi_facade_edge_error(
                        package_name,
                        table_path,
                        dependency_name,
                        resolved_name,
                        effective_specification,
                        resolved_path,
                    )
                    if edge_error is None:
                        continue
                    if edge_error != "not-approved":
                        errors.append(
                            f"{package}/Cargo.toml: {edge_error}: "
                            f"{dependency_name!r} through [{table}]"
                        )
                        continue
                    self_edge_error = nuxie_self_test_dependency_error(
                        package_name,
                        package,
                        table_path,
                        dependency_name,
                        resolved_name,
                        effective_specification,
                        resolved_path,
                    )
                    if self_edge_error is None:
                        continue
                    if self_edge_error != "not-approved":
                        errors.append(
                            f"{package}/Cargo.toml: {self_edge_error}: "
                            f"{dependency_name!r} through [{table}]"
                        )
                        continue
                    errors.append(
                        f"{package}/Cargo.toml: protected package imports product "
                        f"dependency {dependency_name!r} (package {resolved_name!r}) "
                        f"through [{table}]"
                    )

        for source_path in package_rust_sources(
            package_root, manifest, separate_package_roots
        ):
            try:
                relative = source_path.relative_to(repo_root).as_posix()
            except ValueError:
                errors.append(
                    f"{package}/Cargo.toml: Rust target source escapes repository: "
                    f"{source_path}"
                )
                continue
            try:
                raw_source = source_path.read_text()
            except OSError as error:
                errors.append(f"{relative}: cannot read source: {error}")
                continue
            errors.extend(
                cross_package_source_edge_errors(
                    relative, source_path, package_root, repo_root, raw_source
                )
            )
            source = strip_rust_non_code(raw_source)
            if package_name == PORTABLE_ABI_FACADE_EDGE[0]:
                errors.extend(portable_abi_facade_source_errors(relative, source))
            test_module_ranges = cfg_test_module_ranges(source)
            lines = source.splitlines()
            for line_number, line in enumerate(lines, 1):
                if EXPLICIT_PRODUCT_PATH.search(line) or LOCAL_PRODUCT_MODULE.search(line):
                    errors.append(
                        f"{relative}:{line_number}: protected source imports a "
                        f"product/authoring module: {line.strip()}"
                    )
            for family, marker in INTERNAL_DEBT_MARKERS.items():
                matches = list(marker.finditer(source))
                if not matches:
                    continue
                observed_debt[family].add(relative)
                spread = (family, relative)
                out_of_scope = next(
                    (
                        match
                        for match in matches
                        if not debt_match_is_within_approved_scope(
                            family, relative, match, test_module_ranges
                        )
                    ),
                    None,
                )
                if out_of_scope is not None and spread not in reported_debt_spread:
                    reported_debt_spread.add(spread)
                    line_number = source.count("\n", 0, out_of_scope.start()) + 1
                    errors.append(
                        f"{relative}:{line_number}: {family} test-only boundary debt "
                        "escaped its test-only cfg module"
                    )
                    continue
                if (
                    relative not in INTERNAL_DEBT_FILES[family]
                    and spread not in reported_debt_spread
                ):
                    reported_debt_spread.add(spread)
                    line_number = source.count("\n", 0, matches[0].start()) + 1
                    errors.append(
                        f"{relative}:{line_number}: {family} boundary debt spread "
                        "outside its grandfathered files"
                    )

    errors.extend(
        missing_debt_exception_errors(repo_root, observed_debt, INTERNAL_DEBT_FILES)
    )
    if BINARY_AUTHORING_COMPATIBILITY_FILE in observed_debt["binary-authoring"]:
        binary_lib = repo_root / "crates/nuxie-binary/src/lib.rs"
        try:
            binary_source = binary_lib.read_text()
        except OSError as error:
            errors.append(f"crates/nuxie-binary/src/lib.rs: cannot read source: {error}")
        else:
            if not rust_code_pattern_is_present(
                BINARY_AUTHORING_COMPATIBILITY_MODULE, binary_source
            ):
                errors.append(
                    "crates/nuxie-binary/src/lib.rs: legacy authoring compatibility "
                    "must remain gated by the non-default test-support feature"
                )
    for family, exceptions in INTERNAL_DEBT_FILES.items():
        for relative in sorted(exceptions):
            if (repo_root / relative).is_file() and relative not in observed_debt[family]:
                errors.append(
                    f"{relative}: stale {family} boundary debt exception; remove the "
                    "allowlist entry with the debt or restore the marker classification"
                )

    product_lib = repo_root / "crates/nuxie-product/src/lib.rs"
    if product_lib.is_file():
        try:
            product_source = strip_rust_non_code(product_lib.read_text())
        except OSError as error:
            errors.append(f"crates/nuxie-product/src/lib.rs: cannot read source: {error}")
        else:
            for match in PRODUCT_ROOT_REEXPORT.finditer(product_source):
                prefix = product_source[: match.start()]
                if prefix.count("{") != prefix.count("}"):
                    continue
                line_number = product_source.count("\n", 0, match.start()) + 1
                errors.append(
                    "crates/nuxie-product/src/lib.rs:"
                    f"{line_number}: product vocabulary must remain namespaced; "
                    "crate-root compatibility exports cannot return"
                )

    return (
        errors,
        observed_debt,
        protected_count,
        dependency_table_count,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--repo-root",
        type=pathlib.Path,
        default=pathlib.Path(__file__).resolve().parents[2],
    )
    args = parser.parse_args()
    repo_root = args.repo_root.resolve()

    (
        errors,
        observed_debt,
        protected_count,
        dependency_table_count,
    ) = check_repository(repo_root)
    if errors:
        for error in errors:
            print(f"pure-runtime boundary check failed: {error}", file=sys.stderr)
        return 1

    debt_summary = ", ".join(
        f"{family}={len(paths)} grandfathered file(s)"
        for family, paths in sorted(observed_debt.items())
    )
    print(
        "pure-runtime boundary check passed; "
        f"protected workspace packages={protected_count}; "
        f"declared dependency tables={dependency_table_count}; "
        f"internal migration debt: {debt_summary}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
