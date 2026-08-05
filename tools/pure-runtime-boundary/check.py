#!/usr/bin/env python3
"""Ratchet baseline/ABI/oracle packages against product-owned imports.

Nuxie-only architecture guard; no pinned C++ behavior or correspondence row.
See docs/pure-runtime-boundary.md for the ratified contract and the exact migration
debt that this checker prevents from spreading.
"""

from __future__ import annotations

import argparse
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

# All workspace packages are protected by default. These named packages are
# current/future owners above the baseline, plus the one browser consumer that
# is allowed to depend on the browser adapter. Adding another exemption is an
# architecture-policy change reviewed in the same diff as the new package.
UNPROTECTED_WORKSPACE_PACKAGES = {
    "browser-renderer-smoke",
    "nux-container",
    "nuxie",  # mixed facade until the extraction completes
    "nuxie-apple-adapter",
    "nuxie-authoring",
    "nuxie-browser-adapter",
    "nuxie-flow",
    "nuxie-product",
    "nuxie-product-scripting",
    "nuxie-project-data",
}
# Exemption means a package is an upward-facing owner or consumer. Protected
# packages must therefore never depend on any exempt package, even when its
# name does not follow one of the reserved product prefixes.
FORBIDDEN_DEPENDENCIES.update(UNPROTECTED_WORKSPACE_PACKAGES)

# The portable ABI currently reaches baseline facade symbols through the mixed
# nuxie crate. Only this exact dependency form is grandfathered. In particular,
# default features or explicit dependency features could activate product trust
# and scripting, so either change is rejected.
MIXED_FACADE_DEBT = ("nux-capi", "nuxie")
MIXED_FACADE_ALLOWED_FORWARDED_FEATURES = {"renderer"}
MIXED_FACADE_ALLOWED_PROVIDER_FEATURES = {
    "renderer": {"dep:nuxie-renderer"},
}
MIXED_FACADE_ALLOWED_PROVIDER_DEPENDENCIES = {
    "nuxie-renderer": "nuxie-renderer",
}
MIXED_FACADE_ALLOWED_SYMBOLS = {
    "ApplePresentationCompletion",
    "AppleSurface",
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
MIXED_FACADE_PRODUCT_METHOD = re.compile(
    r"\b(?:prepare_flow_[A-Za-z0-9_]*|import_with_(?:trusted_scripts|"
    r"trusted_scripts_and_limits|script_capability|unsigned_scripts)|"
    r"FlowSession[A-Za-z0-9_]*|"
    r"Scene(?:Tx)?[A-Za-z0-9_]*|ProjectData[A-Za-z0-9_]*)\b"
)
DIRECT_NUXIE_PATH = re.compile(r"\bnuxie\s*::\s*(?P<symbol>[A-Za-z_][A-Za-z0-9_]*)")
RUST_USE_STATEMENT = re.compile(r"\buse\b(?P<body>[^;]*);", re.DOTALL)
NUXIE_EXTERN_CRATE = re.compile(r"\bextern\s+crate\s+nuxie\b")
FILE_ASSOCIATED_ITEM = re.compile(
    r"(?:<\s*)?\bFile\b(?:\s*>)?\s*::\s*(?P<item>[A-Za-z_][A-Za-z0-9_]*)"
)
MIXED_FACADE_TYPE_ALIAS = re.compile(
    r"\btype\s+[A-Za-z_][A-Za-z0-9_]*(?:\s*<[^;=]*>)?\s*=\s*"
    r"(?:::)?(?:nuxie\s*::\s*)?(?:" + "|".join(sorted(MIXED_FACADE_ALLOWED_SYMBOLS)) + r")\b"
)
MIXED_FACADE_ALLOWED_FILE_ASSOCIATED_ITEMS = {"import"}

# These are file-level ratchet exceptions, not compliant dependencies. A new
# file containing either marker family fails. Deleting entries is allowed and
# should happen as the migration in docs/pure-runtime-boundary.md proceeds.
INTERNAL_DEBT_FILES = {
    "apple-image-admission": {
        "crates/nux-capi/src/size_report_roots.rs",
        "crates/nuxie-renderer/src/lib.rs",
    },
    "apple-presentation": {
        "crates/nux-capi/src/size_report_roots.rs",
        "crates/nuxie-renderer/src/lib.rs",
        "crates/nuxie-renderer/src/surface.rs",
    },
    "binary-authoring": {
        "crates/nuxie-binary/src/lib.rs",
        "crates/nuxie-runtime/src/artboard/tests.rs",
        "crates/nuxie-runtime/src/assets/font_asset.rs",
        "crates/nuxie-runtime/src/data_bind/data_bind_context.rs",
        "crates/nuxie-runtime/src/draw.rs",
        "crates/nuxie-runtime/src/shapes/list_path.rs",
        "crates/nuxie-runtime/src/state_machine/bindables.rs",
        "crates/nuxie-runtime/src/state_machine/data_bind_template.rs",
        "crates/nuxie-runtime/src/state_machine/focus_action_target.rs",
        "crates/nuxie-runtime/src/state_machine/focus_action_traversal.rs",
        "crates/nuxie-runtime/src/state_machine/listener_action.rs",
        "crates/nuxie-runtime/src/state_machine/listener_align_target.rs",
        "crates/nuxie-runtime/src/state_machine/listener_input_change.rs",
        "crates/nuxie-runtime/src/state_machine/listener_types/listener_input_type_viewmodel.rs",
        "crates/nuxie-runtime/src/state_machine/state_machine.rs",
        "crates/nuxie-runtime/src/state_machine/state_machine_fire_action.rs",
        "crates/nuxie-runtime/src/state_machine/state_machine_instance/tests/scripted_listener_actions.rs",
        "crates/nuxie-runtime/src/state_machine/state_machine_instance/tests/view_model_listener.rs",
        "crates/nuxie-runtime/src/state_machine/state_machine_listener.rs",
        "crates/nuxie-runtime/src/state_machine/transition_duration_binding.rs",
        "crates/nuxie-runtime/src/text.rs",
        "crates/nuxie-runtime/src/view_model.rs",
        "crates/nuxie-runtime/src/view_model_cell.rs",
        "crates/nuxie-runtime/src/viewmodel/runtime/viewmodel_instance_list_index_runtime.rs",
        "crates/nuxie-runtime/src/viewmodel/runtime/viewmodel_instance_runtime.rs",
        "crates/nuxie-runtime/src/viewmodel/viewmodel_instance.rs",
        "crates/nuxie-binary/tests/authoring_records.rs",
        "tools/nuxie-codegen/src/main.rs",
    },
    "browser-presentation": {
        "crates/nuxie-renderer/src/browser.rs",
        "crates/nuxie-renderer/src/lib.rs",
    },
    "project-data": {
        "crates/nuxie-runtime/src/project_data_converter.rs",
        "crates/nuxie-runtime/src/lib.rs",
        "crates/nuxie-runtime/src/data_bind/context/context_value.rs",
        "crates/nuxie-runtime/src/data_bind/converters/data_converter_number_to_list.rs",
        "crates/nuxie-runtime/src/data_bind/data_bind_context.rs",
        "crates/nuxie-runtime/tests/project_data_converter.rs",
        "crates/nuxie-runtime/tests/simple_array_adaptation.rs",
    },
    "product-host-commands": {
        "crates/nuxie-scripting/src/vm.rs",
        "crates/nuxie-scripting/src/vm/host_commands.rs",
        "crates/nuxie-scripting/src/vm/resource_limits.rs",
        "crates/nuxie-scripting/tests/nuxie_host_commands.rs",
    },
}
INTERNAL_DEBT_MARKERS = {
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
    for key, child in value.items():
        child_path = (*path, str(key))
        # A non-virtual workspace root is both a package manifest and the
        # workspace manifest. Central declarations are not dependency edges of
        # that root package; only member tables which inherit them are edges.
        if not path and str(key) == "workspace":
            continue
        if str(key) in {"dependencies", "dev-dependencies", "build-dependencies"}:
            if isinstance(child, dict):
                yield child_path, child
            continue
        yield from dependency_tables(child, child_path)


def dependency_package(dependency_name: str, specification: object) -> str:
    if isinstance(specification, dict):
        package = specification.get("package")
        if isinstance(package, str):
            return package
    return dependency_name


def workspace_packages(
    repo_root: pathlib.Path,
) -> tuple[
    list[tuple[str, str, dict[str, object]]], dict[str, object], list[str]
]:
    errors: list[str] = []
    workspace_path = repo_root / "Cargo.toml"
    try:
        workspace_manifest = tomllib.loads(workspace_path.read_text())
    except (OSError, tomllib.TOMLDecodeError) as error:
        return [], {}, [f"Cargo.toml: cannot parse workspace manifest: {error}"]

    workspace = workspace_manifest.get("workspace")
    members = workspace.get("members") if isinstance(workspace, dict) else None
    if not isinstance(members, list) or not all(
        isinstance(member, str) for member in members
    ):
        return [], {}, ["Cargo.toml: [workspace].members must be a string array"]

    workspace_dependencies = workspace.get("dependencies", {})
    if not isinstance(workspace_dependencies, dict):
        return [], {}, ["Cargo.toml: [workspace.dependencies] must be a table"]
    excluded = workspace.get("exclude", [])
    if not isinstance(excluded, list) or not all(
        isinstance(pattern, str) for pattern in excluded
    ):
        return [], {}, ["Cargo.toml: [workspace].exclude must be a string array"]

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

    return packages, workspace_dependencies, errors


def mixed_facade_debt_error(
    package_name: str,
    table_path: tuple[str, ...],
    dependency_name: str,
    resolved_name: str,
    specification: object,
) -> str | None:
    if (package_name, normalized_package_name(resolved_name)) != MIXED_FACADE_DEBT:
        return "not-grandfathered"
    if table_path != ("dependencies",) or not isinstance(specification, dict):
        return "mixed-facade debt edge expanded outside [dependencies]"
    if normalized_package_name(dependency_name) != "nuxie":
        return "mixed-facade debt edge must use dependency key 'nuxie'"
    if specification.get("default-features") is not False:
        return "mixed-facade debt edge expanded by enabling default features"
    features = specification.get("features", [])
    if features not in (None, []) and features != ():
        return "mixed-facade debt edge expanded with dependency features"
    return None


def _blank_non_newlines(characters: list[str], start: int, end: int) -> None:
    for index in range(start, end):
        if characters[index] not in "\r\n":
            characters[index] = " "


def strip_rust_non_code(source: str) -> str:
    """Blank comments and string literals while preserving line positions."""

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
            _blank_non_newlines(characters, index, min(end, length))
            index = end
            continue
        index += 1
    return "".join(characters)


def mixed_facade_feature_errors(
    package: str, package_name: str, manifest: dict[str, object]
) -> list[str]:
    if package_name != MIXED_FACADE_DEBT[0]:
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
            if match and match.group(1) not in MIXED_FACADE_ALLOWED_FORWARDED_FEATURES:
                errors.append(
                    f"{package}/Cargo.toml: feature {feature_name!r} forwards "
                    f"forbidden mixed-facade feature {match.group(1)!r}"
                )
    return errors


def mixed_facade_provider_feature_errors(
    packages: list[tuple[str, str, dict[str, object]]],
    workspace_dependencies: dict[str, object],
) -> list[str]:
    provider = next(
        (
            (package, manifest)
            for package, package_name, manifest in packages
            if package_name == MIXED_FACADE_DEBT[1]
        ),
        None,
    )
    if provider is None:
        return []
    package, manifest = provider
    features = manifest.get("features")
    if not isinstance(features, dict):
        return [f"{package}/Cargo.toml: mixed facade [features] must be a table"]
    errors = []
    for feature_name, approved_activations in MIXED_FACADE_ALLOWED_PROVIDER_FEATURES.items():
        activations = features.get(feature_name)
        if not isinstance(activations, list) or not all(
            isinstance(activation, str) for activation in activations
        ):
            errors.append(
                f"{package}/Cargo.toml: mixed facade feature {feature_name!r} "
                "must be a string array"
            )
            continue
        actual = set(activations)
        if actual != approved_activations:
            errors.append(
                f"{package}/Cargo.toml: mixed facade feature {feature_name!r} "
                f"activations changed from {sorted(approved_activations)!r} to "
                f"{sorted(actual)!r}"
            )
    dependencies = manifest.get("dependencies")
    if not isinstance(dependencies, dict):
        errors.append(f"{package}/Cargo.toml: mixed facade [dependencies] must be a table")
        return errors
    for dependency_name, approved_package in MIXED_FACADE_ALLOWED_PROVIDER_DEPENDENCIES.items():
        specification = dependencies.get(dependency_name)
        effective = specification
        if isinstance(specification, dict) and specification.get("workspace") is True:
            effective, inheritance_error = inherited_dependency_specification(
                dependency_name, specification, workspace_dependencies
            )
            if inheritance_error is not None:
                errors.append(f"{package}/Cargo.toml: {inheritance_error}")
                continue
        resolved_package = dependency_package(dependency_name, effective)
        if resolved_package != approved_package:
            errors.append(
                f"{package}/Cargo.toml: mixed facade dependency {dependency_name!r} "
                f"resolves to {resolved_package!r}, expected {approved_package!r}"
            )
        if not isinstance(effective, dict):
            errors.append(
                f"{package}/Cargo.toml: mixed facade dependency {dependency_name!r} "
                "must use an explicit table"
            )
            continue
        features = effective.get("features", [])
        defaults = effective.get("default-features", True)
        if effective.get("optional") is not True or features not in (None, []) or defaults is not True:
            errors.append(
                f"{package}/Cargo.toml: mixed facade dependency {dependency_name!r} "
                "must remain optional with default features and no explicit features"
            )
    return errors


def mixed_facade_source_errors(relative: str, source: str) -> list[str]:
    errors = []
    for match in NUXIE_EXTERN_CRATE.finditer(source):
        line = source.count("\n", 0, match.start()) + 1
        errors.append(
            f"{relative}:{line}: mixed facade extern-crate imports are not approved"
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
                f"{relative}:{line}: mixed facade use tree is not an approved flat import"
            )
            continue
        for symbol in imported_symbols:
            if symbol not in MIXED_FACADE_ALLOWED_SYMBOLS:
                errors.append(
                    f"{relative}:{line}: mixed facade symbol {symbol!r} is not approved"
                )
    for match in MIXED_FACADE_TYPE_ALIAS.finditer(source):
        line = source.count("\n", 0, match.start()) + 1
        errors.append(
            f"{relative}:{line}: type aliases of mixed facade symbols are not approved"
        )
    for match in DIRECT_NUXIE_PATH.finditer(source):
        symbol = match.group("symbol")
        if symbol not in MIXED_FACADE_ALLOWED_SYMBOLS:
            line = source.count("\n", 0, match.start()) + 1
            errors.append(
                f"{relative}:{line}: mixed facade symbol {symbol!r} is not approved"
            )
    for match in re.finditer(r"\bimpl\b(?P<header>[^{};]*)\{", source, re.DOTALL):
        header = match.group("header")
        symbol_pattern = "|".join(sorted(MIXED_FACADE_ALLOWED_SYMBOLS))
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
                f"{relative}:{line}: impls targeting mixed facade symbols are not approved"
            )
    for match in FILE_ASSOCIATED_ITEM.finditer(source):
        item = match.group("item")
        if item not in MIXED_FACADE_ALLOWED_FILE_ASSOCIATED_ITEMS:
            line = source.count("\n", 0, match.start()) + 1
            errors.append(
                f"{relative}:{line}: File associated item {item!r} is not in the "
                "approved baseline facade surface"
            )
    for match in MIXED_FACADE_PRODUCT_METHOD.finditer(source):
        line = source.count("\n", 0, match.start()) + 1
        errors.append(
            f"{relative}:{line}: mixed facade product method/type {match.group(0)!r} "
            "is not approved"
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
    package_root: pathlib.Path, manifest: dict[str, object]
) -> Iterator[pathlib.Path]:
    sources: set[pathlib.Path] = set()
    package = manifest.get("package")
    build = package.get("build") if isinstance(package, dict) else None
    if isinstance(build, str):
        sources.add((package_root / build).resolve())
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
            sources.add((package_root / target_path).resolve())

    for directory in ("src", "tests", "examples", "benches"):
        source_root = package_root / directory
        if source_root.is_dir():
            sources.update(path.resolve() for path in source_root.rglob("*.rs"))
    yield from sorted(sources)


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
) -> tuple[list[str], dict[str, set[str]], int, int, int]:
    packages, workspace_dependencies, errors = workspace_packages(repo_root)
    observed_debt = {family: set() for family in INTERNAL_DEBT_FILES}
    reported_debt_spread: set[tuple[str, str]] = set()
    manifest_debt_count = 0
    dependency_table_count = 0
    protected_count = 0

    errors.extend(
        mixed_facade_provider_feature_errors(packages, workspace_dependencies)
    )

    for package, package_name, manifest in packages:
        if package_name in UNPROTECTED_WORKSPACE_PACKAGES:
            continue
        protected_count += 1
        package_root = repo_root / package
        errors.extend(mixed_facade_feature_errors(package, package_name, manifest))

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
                if is_forbidden_dependency(dependency_name) or is_forbidden_dependency(
                    resolved_name
                ):
                    table = ".".join(table_path)
                    debt_error = mixed_facade_debt_error(
                        package_name,
                        table_path,
                        dependency_name,
                        resolved_name,
                        effective_specification,
                    )
                    if debt_error is None:
                        manifest_debt_count += 1
                        continue
                    if debt_error != "not-grandfathered":
                        errors.append(
                            f"{package}/Cargo.toml: {debt_error}: "
                            f"{dependency_name!r} through [{table}]"
                        )
                        continue
                    errors.append(
                        f"{package}/Cargo.toml: protected package imports product "
                        f"dependency {dependency_name!r} (package {resolved_name!r}) "
                        f"through [{table}]"
                    )

        for source_path in package_rust_sources(package_root, manifest):
            try:
                relative = source_path.relative_to(repo_root).as_posix()
            except ValueError:
                errors.append(
                    f"{package}/Cargo.toml: Rust target source escapes repository: "
                    f"{source_path}"
                )
                continue
            try:
                source = strip_rust_non_code(source_path.read_text())
            except OSError as error:
                errors.append(f"{relative}: cannot read source: {error}")
                continue
            if package_name == MIXED_FACADE_DEBT[0]:
                errors.extend(mixed_facade_source_errors(relative, source))
            lines = source.splitlines()
            for line_number, line in enumerate(lines, 1):
                if EXPLICIT_PRODUCT_PATH.search(line) or LOCAL_PRODUCT_MODULE.search(line):
                    errors.append(
                        f"{relative}:{line_number}: protected source imports a "
                        f"product/authoring module: {line.strip()}"
                    )
                for family, marker in INTERNAL_DEBT_MARKERS.items():
                    if not marker.search(line):
                        continue
                    observed_debt[family].add(relative)
                    spread = (family, relative)
                    if (
                        relative not in INTERNAL_DEBT_FILES[family]
                        and spread not in reported_debt_spread
                    ):
                        reported_debt_spread.add(spread)
                        errors.append(
                            f"{relative}:{line_number}: {family} boundary debt spread "
                            "outside its grandfathered files"
                        )

    errors.extend(
        missing_debt_exception_errors(repo_root, observed_debt, INTERNAL_DEBT_FILES)
    )
    for family, exceptions in INTERNAL_DEBT_FILES.items():
        for relative in sorted(exceptions):
            if (repo_root / relative).is_file() and relative not in observed_debt[family]:
                errors.append(
                    f"{relative}: stale {family} boundary debt exception; remove the "
                    "allowlist entry with the debt or restore the marker classification"
                )

    return (
        errors,
        observed_debt,
        manifest_debt_count,
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
        manifest_debt_count,
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
        "manifest migration debt: "
        f"portable-c-abi-mixed-facade={manifest_debt_count}; "
        f"internal migration debt: {debt_summary}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
