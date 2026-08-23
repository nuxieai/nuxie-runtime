#!/usr/bin/env python3
"""Fail-closed validator for the native Metal mechanical-port campaign."""

from __future__ import annotations

import argparse
import collections
import csv
import hashlib
import importlib.util
import os
import pathlib
import re
import subprocess
import sys
import tomllib
from typing import Any, Iterable


REPLAY_RECEIPT_COMMANDS = False
_RECEIPT_COMMAND_CACHE: dict[tuple[str, str], tuple[int, str, str]] = {}


EXHAUSTIVE_AUTHORITY_LEDGERS = {
    "preprocessor_authority": pathlib.Path("docs/metal-port-preprocessor-authority.tsv"),
    "direct_include_authority": pathlib.Path("docs/metal-port-include-authority.tsv"),
    "source_dependency_authority": pathlib.Path("docs/metal-port-source-dependencies.tsv"),
    "dispatch_prerequisite_authority": pathlib.Path("docs/metal-port-dispatch-prerequisites.tsv"),
    "build_branch_authority": pathlib.Path("docs/metal-port-build-branch-authority.tsv"),
}


SOURCE_STATUSES = {"pending", "in-progress", "ported", "verified"}
OWNER_STATUSES = {"pending", "in-progress", "ported", "verified"}
VERIFIED_STATUSES = {"ported", "verified"}
TRANSLATION_STATUSES = {
    "pending",
    "ready",
    "in-progress",
    "translated",
    "reviewed",
    "fixed",
    "compiled",
    "verified",
}
TRANSLATION_PHASES = {"trial", "bulk"}
TRANSLATION_WORKER_ROLES = {"luna-extra-high", "sol-high"}
TRANSLATION_REVIEWER_ROLES = {"sol-high"}
TRANSLATION_FIXER_ROLES = {"sol-high"}
TRANSLATION_RECEIPT_FIELDS = (
    "translation_receipt",
    "source_review_receipt",
    "ownership_review_receipt",
    "fix_receipt",
    "compile_receipt",
    "verification_receipt",
)
TRANSLATION_RECEIPT_SUFFIXES = {
    "translation_receipt": "translation.toml",
    "source_review_receipt": "source-review.toml",
    "ownership_review_receipt": "ownership-review.toml",
    "fix_receipt": "fix.toml",
    "compile_receipt": "compile.toml",
    "verification_receipt": "verification.toml",
}
MECHANICAL_TRANSLATION_WORKFLOW = {
    "methodology": "https://bun.com/blog/bun-in-rust",
    "queue": "complete-pinned-files-in-parallel-waves",
    "translator_role": "luna-extra-high",
    "source_reviewer_role": "sol-high",
    "ownership_reviewer_role": "sol-high",
    "fixer_driver_role": "sol-high",
    "adversarial_reviews_per_translation": 2,
    "bulk_translation_before_review_queue": True,
    "source_review_pass_before_ownership_review_pass": True,
    "ownership_review_pass_before_fix_queue": True,
    "bulk_translation_before_compiler_queue": True,
    "compiler_queue_before_behavior_queue": True,
    "cleanup_only_after_full_parity": True,
    "feature_or_fixture_work_items_forbidden": True,
    "stubs_and_skipped_tests_forbidden": True,
}
LIFETIME_STATUSES = {"review-needed", "prepared", "verified"}
LIFETIME_COLUMNS = (
    "schema_version",
    "upstream_ref",
    "unit",
    "upstream_path",
    "field",
    "cpp_ownership",
    "rust_shape",
    "threading",
    "concrete_native_downcast_seam",
    "release_invariant",
    "failure_invariant",
    "status",
    "evidence",
)
RENDER_CONTEXT_FILE_MAP_COLUMNS = (
    "version",
    "upstream_sha",
    "upstream_file",
    "lines",
    "symbol",
    "status",
    "rust_owner",
    "remaining",
)
RENDER_CONTEXT_FILE_MAP_STATUSES = {"ported", "partial", "missing"}
RENDER_CONTEXT_FILE_MAP_SOURCES = {
    "renderer/include/rive/renderer/metal/render_context_metal_impl.h",
    "renderer/src/metal/render_context_metal_impl.mm",
}
RENDER_CONTEXT_FIELD_MAP_COLUMNS = (
    "version",
    "upstream_sha",
    "upstream_file",
    "cpp_type",
    "cpp_field",
    "cpp_declared_type",
    "declaration_line",
    "configuration",
    "rust_owner",
    "rust_field",
    "construction_and_publication",
    "mutation_thread",
    "submission_and_completion",
    "destruction_order",
    "null_and_failure",
    "safe_rust_adaptation",
    "mapping_status",
    "translation_status",
    "evidence",
)
RENDER_CONTEXT_FIELD_MAPPING_STATUSES = {"review-needed", "prepared"}
RENDER_CONTEXT_FIELD_TRANSLATION_STATUSES = {"pending", "translated", "verified"}
RENDER_CONTEXT_FIELD_PLACEHOLDER_MARKERS = (
    "is declared source state; preserve",
    "preserve the source mutation sites",
    "use an ownership-safe Rust representation matching",
    "option<box<_>> or",
    "option<nonnull<_>> when nullable",
    "otherwise a lifetime-bound",
    "needs ownership review",
    "not implemented",
    "borrow or nonnull",
    "or nonnull handle",
    "future rust",
    "no stored shadow buffer",
    "source-only inert member",
    "an exact-width rust scalar/enum/record value",
    "hashmap<_, _>",
)
RENDER_CONTEXT_FIELD_EXTRACTOR = "tools/metal-port/extract_field_authority.py"
RENDER_CONTEXT_CONFIGURATION_MAP_COLUMNS = (
    "version",
    "upstream_sha",
    "upstream_file",
    "block",
    "lines",
    "branch_lines",
    "configurations",
    "source_behavior",
    "rust_owner",
    "rust_configuration",
    "mapping_status",
    "translation_status",
    "translation_disposition",
    "validation_disposition",
    "exclusion_reason",
    "remaining",
    "evidence",
)
RENDER_CONTEXT_CONFIGURATION_MAPPING_STATUSES = {
    "review-needed",
    "prepared",
}
RENDER_CONTEXT_CONFIGURATION_TRANSLATION_STATUSES = {
    "pending",
    "translated",
    "verified",
}
RENDER_CONTEXT_CONFIGURATION_TRANSLATION_DISPOSITIONS = {"required", "excluded"}
RENDER_CONTEXT_CONFIGURATION_VALIDATION_DISPOSITIONS = {
    "executable",
    "compile-link-only",
    "checked-exclusion",
}
RENDER_CONTEXT_CONFIGURATION_MAP_SOURCES = {
    "renderer/include/rive/renderer/metal/render_context_metal_impl.h",
    "renderer/src/metal/render_context_metal_impl.mm",
    "renderer/src/metal/background_shader_compiler.h",
    "renderer/src/metal/background_shader_compiler.mm",
}
RENDER_CONTEXT_DEPENDENCY_MAP_COLUMNS = (
    "version",
    "upstream_sha",
    "upstream_file",
    "source_role",
    "rust_owners",
    "translation_unit",
    "translation_target",
    "field_coverage",
    "mapping_status",
    "translation_status",
    "remaining",
    "evidence",
)
RENDER_CONTEXT_INCLUDE_MAP_COLUMNS = (
    "version",
    "upstream_sha",
    "upstream_file",
    "include_line",
    "include_token",
    "include_kind",
    "configuration",
    "source_resolution",
    "correspondence_owner",
    "mapping_status",
    "remaining",
    "evidence",
)
RENDER_CONTEXT_DEPENDENCY_SOURCES = (
    "include/rive/rive_types.hpp",
    "include/rive/refcnt.hpp",
    "include/utils/lite_rtti.hpp",
    "include/rive/renderer.hpp",
    "src/renderer.cpp",
    "include/rive/gpu_texture_format.hpp",
    "include/rive/shapes/paint/image_sampler.hpp",
    "decoders/include/rive/decoders/astc_footprints.hpp",
    "renderer/include/rive/renderer/gpu.hpp",
    "renderer/src/gpu.cpp",
    "renderer/include/rive/renderer/buffer_ring.hpp",
    "renderer/include/rive/renderer/render_target.hpp",
    "renderer/include/rive/renderer/texture.hpp",
    "renderer/include/rive/renderer/rive_render_image.hpp",
    "renderer/src/rive_render_image.cpp",
    "renderer/include/rive/renderer/rive_render_buffer.hpp",
    "renderer/include/rive/renderer/render_canvas.hpp",
    "renderer/include/rive/renderer/render_context.hpp",
    "renderer/include/rive/renderer/render_context_impl.hpp",
    "renderer/include/rive/renderer/render_context_helper_impl.hpp",
    "renderer/src/render_context_helper_impl.cpp",
    "renderer/include/rive/renderer/draw.hpp",
    "renderer/src/draw.cpp",
    "renderer/src/render_context.cpp",
    "renderer/src/rive_render_path.hpp",
    "renderer/src/rive_render_path.cpp",
    "renderer/src/rive_render_paint.hpp",
    "renderer/src/rive_render_paint.cpp",
    "renderer/include/rive/renderer/rive_renderer.hpp",
    "renderer/src/rive_renderer.cpp",
    "include/rive/factory.hpp",
    "renderer/include/rive/renderer/rive_render_factory.hpp",
    "renderer/src/rive_render_factory.cpp",
    "renderer/src/gradient.hpp",
    "renderer/src/gradient.cpp",
)
RENDER_CONTEXT_CONFIGURATION_MAP_SOURCES.update(RENDER_CONTEXT_DEPENDENCY_SOURCES)
RENDER_CONTEXT_INCLUDE_TOOLCHAIN_OWNERS = {
    "rust-std",
    "apple-platform-cfg",
    "emscripten-target",
}
GENERIC_TRANSLATION_UNIT_PLAN = {
    "generic-rive-types": (4, (), ("include/rive/rive_types.hpp",)),
    "generic-refcnt": (5, ("generic-rive-types",), ("include/rive/refcnt.hpp",)),
    "generic-lite-rtti": (15, ("generic-refcnt",), ("include/utils/lite_rtti.hpp",)),
    "generic-image-sampler": (16, (), ("include/rive/shapes/paint/image_sampler.hpp",)),
    "generic-renderer-contract": (17, ("generic-rive-types", "generic-refcnt", "generic-lite-rtti", "generic-image-sampler"), ("include/rive/renderer.hpp",)),
    "generic-renderer-implementation": (18, ("generic-renderer-contract",), ("src/renderer.cpp",)),
    "generic-gpu-texture-format": (19, (), ("include/rive/gpu_texture_format.hpp",)),
    "generic-astc-footprints": (20, (), ("decoders/include/rive/decoders/astc_footprints.hpp",)),
    "generic-gpu-contract": (21, ("generic-rive-types", "generic-image-sampler", "generic-renderer-contract"), ("renderer/include/rive/renderer/gpu.hpp",)),
    "generic-buffer-ring": (22, ("generic-gpu-contract",), ("renderer/include/rive/renderer/buffer_ring.hpp",)),
    "generic-render-target": (23, ("generic-refcnt",), ("renderer/include/rive/renderer/render_target.hpp",)),
    "generic-texture-image": (24, ("generic-refcnt", "generic-lite-rtti", "generic-renderer-contract"), ("renderer/include/rive/renderer/texture.hpp", "renderer/include/rive/renderer/rive_render_image.hpp", "renderer/src/rive_render_image.cpp")),
    "generic-rive-render-buffer": (25, ("generic-renderer-contract", "generic-gpu-contract"), ("renderer/include/rive/renderer/rive_render_buffer.hpp",)),
    "generic-render-canvas": (26, ("generic-refcnt", "generic-render-target", "generic-texture-image"), ("renderer/include/rive/renderer/render_canvas.hpp",)),
    "generic-render-context-contract": (27, ("generic-renderer-contract", "generic-gpu-contract", "generic-render-target", "generic-render-canvas", "ore-context-render-pass"), ("renderer/include/rive/renderer/render_context.hpp",)),
    "generic-render-context-impl-contract": (28, ("generic-gpu-texture-format", "generic-render-context-contract"), ("renderer/include/rive/renderer/render_context_impl.hpp",)),
    "generic-render-context-helper": (29, ("generic-buffer-ring", "generic-texture-image", "generic-render-context-impl-contract"), ("renderer/include/rive/renderer/render_context_helper_impl.hpp", "renderer/src/render_context_helper_impl.cpp")),
    "generic-gpu-implementation": (30, ("generic-renderer-contract", "generic-gpu-contract", "generic-render-target", "generic-texture-image", "generic-render-context-contract"), ("renderer/src/gpu.cpp",)),
    "generic-render-context-implementation": (31, ("generic-refcnt", "generic-renderer-implementation", "generic-gpu-texture-format", "generic-gpu-contract", "generic-texture-image", "generic-render-canvas", "generic-render-context-contract", "generic-render-context-impl-contract", "generic-gpu-implementation"), ("renderer/include/rive/renderer/draw.hpp", "renderer/src/draw.cpp", "renderer/src/render_context.cpp")),
    "generic-rive-render-path": (36, ("generic-renderer-contract", "generic-gpu-contract", "generic-render-context-implementation", "metal-shader-source-batch"), ("renderer/src/rive_render_path.hpp", "renderer/src/rive_render_path.cpp")),
    "generic-rive-render-paint": (37, ("generic-renderer-contract", "generic-gpu-contract", "generic-texture-image", "generic-render-context-contract"), ("renderer/src/rive_render_paint.hpp", "renderer/src/rive_render_paint.cpp")),
    "generic-rive-renderer": (38, ("generic-renderer-contract", "generic-gpu-contract", "generic-render-context-contract", "generic-render-context-implementation", "generic-texture-image", "generic-rive-render-path", "generic-rive-render-paint"), ("renderer/include/rive/renderer/rive_renderer.hpp", "renderer/src/rive_renderer.cpp")),
    "generic-factory-contract": (39, ("generic-refcnt", "generic-renderer-contract"), ("include/rive/factory.hpp",)),
    "generic-gradient": (40, ("generic-gpu-contract", "generic-renderer-contract"), ("renderer/src/gradient.hpp", "renderer/src/gradient.cpp")),
    "generic-rive-render-factory": (41, ("generic-factory-contract", "generic-gradient", "generic-rive-render-paint", "generic-rive-render-path", "generic-rive-renderer"), ("renderer/include/rive/renderer/rive_render_factory.hpp", "renderer/src/rive_render_factory.cpp")),
}
METAL_TRANSLATION_UNIT_PLAN = {
    "metal-render-context-api": (33, ("generic-buffer-ring", "generic-render-context-impl-contract", "ore-context-render-pass"), ("renderer/include/rive/renderer/metal/render_context_metal_impl.h",)),
    "metal-background-shader-compiler": (34, ("metal-render-context-api", "metal-shader-source-batch"), ("renderer/src/metal/background_shader_compiler.h", "renderer/src/metal/background_shader_compiler.mm")),
    "metal-render-context-implementation": (35, ("generic-buffer-ring", "generic-gpu-contract", "generic-gpu-implementation", "generic-render-target", "generic-texture-image", "generic-rive-render-buffer", "generic-render-canvas", "generic-render-context-contract", "generic-render-context-impl-contract", "generic-render-context-helper", "generic-render-context-implementation", "metal-shader-source-batch", "metal-render-context-api", "metal-background-shader-compiler", "ore-context-render-pass"), ("renderer/src/metal/render_context_metal_impl.mm",)),
}
DIVERGENCE_LEDGER_COLUMNS = (
    "version",
    "upstream_sha",
    "id",
    "upstream_source",
    "source_behavior",
    "rust_owner",
    "rust_behavior",
    "observability",
    "proposed_disposition",
    "status",
    "source_review_receipt",
    "ownership_review_receipt",
    "correction_receipt",
    "evidence",
)
DIVERGENCE_IDS = {
    "transactional-atomic-plane-publication",
    "eager-command-queue-injection",
    "context-options-cache-owner",
    "transactional-resource-generation",
    "paired-atlas-pipeline-publication",
    "deep-compatible-pipeline-cache",
}
DIVERGENCE_STATUSES = {"review-needed", "accepted", "rejected", "resolved"}
DIVERGENCE_DISPOSITIONS = {
    "review-safety-correction",
    "review-ownership-adaptation",
}
SHADER_SOURCE_INVENTORY_COLUMNS = (
    "version",
    "upstream_sha",
    "ordinal",
    "stage",
    "source",
    "line_count",
    "sha256",
    "disposition",
)
METAL_SHADER_ARTIFACT_TARGETS = (
    "crates/nuxie-renderer/build.rs",
    "crates/nuxie-renderer/src/native_metal/draw_combinations.rs",
    "crates/nuxie-renderer/src/native_metal/shaders/advanced_blend.minified.glsl",
    "crates/nuxie-renderer/src/native_metal/shaders/bezier_utils.minified.glsl",
    "crates/nuxie-renderer/src/native_metal/shaders/color_ramp.metal",
    "crates/nuxie-renderer/src/native_metal/shaders/color_ramp.minified.glsl",
    "crates/nuxie-renderer/src/native_metal/shaders/common.minified.glsl",
    "crates/nuxie-renderer/src/native_metal/shaders/constants.minified.glsl",
    "crates/nuxie-renderer/src/native_metal/shaders/draw.metal",
    "crates/nuxie-renderer/src/native_metal/shaders/draw_combinations.metal",
    "crates/nuxie-renderer/src/native_metal/shaders/draw_image_mesh.minified.vert",
    "crates/nuxie-renderer/src/native_metal/shaders/draw_mesh.minified.frag",
    "crates/nuxie-renderer/src/native_metal/shaders/draw_path.minified.vert",
    "crates/nuxie-renderer/src/native_metal/shaders/draw_path_common.minified.glsl",
    "crates/nuxie-renderer/src/native_metal/shaders/draw_raster_order_path.minified.frag",
    "crates/nuxie-renderer/src/native_metal/shaders/flush_uniforms.minified.glsl",
    "crates/nuxie-renderer/src/native_metal/shaders/metal.minified.glsl",
    "crates/nuxie-renderer/src/native_metal/shaders/render_atlas.minified.glsl",
    "crates/nuxie-renderer/src/native_metal/shaders/tessellate.metal",
    "crates/nuxie-renderer/src/native_metal/shaders/tessellate.minified.glsl",
)
TRANSLATION_CONVENTION_COLUMNS = (
    "version",
    "convention",
    "cpp_shape",
    "rust_rule",
    "invariant",
    "forbidden",
    "status",
    "evidence",
)
TRANSLATION_CONVENTION_STATUSES = {"review-needed", "frozen", "verified"}
TRANSLATION_CONVENTION_IDS = {
    "objc-retained-nullable",
    "intrusive-reference-counting",
    "byte-ranges-and-alignment",
    "enums-flags-slots-formats",
    "assertions-and-errors",
    "callbacks-workers-completion",
    "preprocessor-configurations",
    "destruction-and-drop-order",
}
RENDER_CONTEXT_FIELD_OWNER_INPUT = "docs/render-context-metal-field-owners.tsv"
RENDER_CONTEXT_FIELD_SOURCE_INPUT = "docs/render-context-metal-field-sources.tsv"
RENDER_CONTEXT_FIELD_EXPECTED_COUNT = 455
ORE_TRANSLATION_UNIT_ORDER = (
    "ore-types",
    "ore-rstb-container",
    "ore-binding-map",
    "gpu-resource",
    "ore-bind-group-layout",
    "ore-buffer",
    "ore-texture",
    "ore-sampler",
    "ore-shader-module",
    "ore-pipeline",
    "ore-bind-group",
    "ore-context-render-pass",
)
MECHANICAL_DISPATCH_ORDER = (
    "ore-types",
    "ore-rstb-container",
    "ore-binding-map",
    "generic-rive-types",
    "generic-refcnt",
    "gpu-resource",
    "ore-bind-group-layout",
    "ore-buffer",
    "ore-texture",
    "ore-sampler",
    "ore-shader-module",
    "ore-pipeline",
    "ore-bind-group",
    "ore-context-render-pass",
    "generic-lite-rtti",
    "generic-image-sampler",
    "generic-renderer-contract",
    "generic-renderer-implementation",
    "generic-gpu-texture-format",
    "generic-astc-footprints",
    "generic-gpu-contract",
    "generic-buffer-ring",
    "generic-render-target",
    "generic-texture-image",
    "generic-rive-render-buffer",
    "generic-render-canvas",
    "generic-render-context-contract",
    "generic-render-context-impl-contract",
    "generic-render-context-helper",
    "generic-gpu-implementation",
    "generic-render-context-implementation",
    "metal-shader-source-batch",
    "metal-render-context-api",
    "metal-background-shader-compiler",
    "metal-render-context-implementation",
    "generic-rive-render-path",
    "generic-rive-render-paint",
    "generic-rive-renderer",
    "generic-factory-contract",
    "generic-gradient",
    "generic-rive-render-factory",
)
MECHANICAL_DISPATCH_ORDINALS = {
    unit_id: ordinal for ordinal, unit_id in enumerate(MECHANICAL_DISPATCH_ORDER, 1)
}
FOUNDATION_TRIAL_UNITS = {
    "ore-types": {"renderer/include/rive/renderer/ore/ore_types.hpp"},
    "ore-rstb-container": {
        "renderer/include/rive/renderer/ore/ore_rstb_entry_container.hpp"
    },
    "ore-binding-map": {
        "renderer/include/rive/renderer/ore/ore_binding_map.hpp",
        "renderer/src/ore/ore_binding_map.cpp",
    },
}
CITATION_RE = re.compile(r"^(cpp|rust):(.+):(\d+)(?:-(\d+))?$")


class CheckFailure(Exception):
    """Raised when the port campaign documents are incomplete or inconsistent."""


def read_toml(path: pathlib.Path) -> dict[str, Any]:
    try:
        with path.open("rb") as source:
            return tomllib.load(source)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise CheckFailure(f"cannot read {path}: {error}") from error


def git_head(path: pathlib.Path) -> str:
    result = subprocess.run(
        ["git", "-C", str(path), "rev-parse", "HEAD"],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise CheckFailure(
            f"cannot resolve upstream HEAD at {path}: {result.stderr.strip()}"
        )
    return result.stdout.strip()


def git_tracked_file(repo_root: pathlib.Path, relative: str) -> bool:
    if not (repo_root / relative).is_file():
        return False
    result = subprocess.run(
        ["git", "-C", str(repo_root), "ls-files", "--error-unmatch", "--", relative],
        text=True,
        capture_output=True,
        check=False,
    )
    return result.returncode == 0


def duplicate_values(values: Iterable[str]) -> list[str]:
    counts = collections.Counter(values)
    return sorted(value for value, count in counts.items() if count > 1)


def derived_coverage_status(unit: dict[str, Any] | None) -> str:
    """Map receipt-gated unit state onto mutable row-level coverage."""

    status = str((unit or {}).get("status", "pending"))
    if status in {"translated", "reviewed", "fixed", "compiled"}:
        return "translated"
    if status == "verified":
        return "verified"
    return "pending"


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def parse_provenance(path: pathlib.Path, errors: list[str]) -> dict[str, str]:
    fields: dict[str, str] = {}
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as error:
        errors.append(f"cannot read reference provenance {path}: {error}")
        return fields
    for line_number, raw_line in enumerate(lines, 1):
        line = raw_line.strip()
        if not line:
            continue
        if "=" not in line:
            errors.append(f"{path} line {line_number} is not key=value provenance")
            continue
        key, value = (part.strip() for part in line.split("=", 1))
        if not key or not value:
            errors.append(f"{path} line {line_number} has an empty key or value")
        elif key in fields:
            errors.append(f"{path} repeats provenance field `{key}`")
        else:
            fields[key] = value
    return fields


def validate_reference_provenance(
    manifest: dict[str, Any], repo_root: pathlib.Path, errors: list[str]
) -> None:
    rows = list(manifest.get("reference_provenance", []))
    duplicates = duplicate_values(str(row.get("id", "")) for row in rows)
    if duplicates:
        errors.append(f"duplicate reference provenance rows: {', '.join(duplicates)}")
    upstream_ref = str(manifest.get("upstream_ref", ""))
    for row in rows:
        record_id = str(row.get("id", ""))
        relative_paths = {
            key: str(row.get(key, "")) for key in ("path", "stream", "reference")
        }
        resolved: dict[str, pathlib.Path] = {}
        for key, relative in relative_paths.items():
            path = repo_root / relative
            resolved[key] = path
            if not relative or not path.is_file():
                errors.append(
                    f"reference provenance {record_id} names missing {key} path {relative}"
                )
            elif not git_tracked_file(repo_root, relative):
                errors.append(
                    f"reference provenance {record_id} names untracked {key} path {relative}"
                )
        if not all(path.is_file() for path in resolved.values()):
            continue
        fields = parse_provenance(resolved["path"], errors)
        expected = {
            "provenance_schema": "1",
            "renderer_implementation": str(row.get("renderer_implementation", "")),
            "capture_tool": str(row.get("capture_tool", "")),
            "backend": str(row.get("backend", "")),
            "adapter_device": str(row.get("adapter_device", "")),
            "case_id": record_id,
            "runtime_revision": upstream_ref,
            "replay_sha256": str(row.get("replay_sha256", "")),
            "reference_input_manifest_sha256": str(
                row.get("reference_input_manifest_sha256", "")
            ),
            "stream_sha256": sha256_file(resolved["stream"]),
            "png_sha256": sha256_file(resolved["reference"]),
            "frame": str(row.get("frame", "")),
            "frame_width": str(row.get("frame_width", "")),
            "frame_height": str(row.get("frame_height", "")),
            "mode": str(row.get("mode", "")),
            "sample_count": str(row.get("sample_count", "")),
        }
        for key, expected_value in expected.items():
            actual = fields.get(key)
            if actual != expected_value:
                errors.append(
                    f"reference provenance {record_id} {key} `{actual}` does not match `{expected_value}`"
                )
        for key in ("replay_sha256", "reference_input_manifest_sha256"):
            value = fields.get(key, "")
            if len(value) != 64 or any(character not in "0123456789abcdef" for character in value):
                errors.append(
                    f"reference provenance {record_id} {key} must be 64 lowercase hex characters"
                )


def expand_source_scope(
    upstream_root: pathlib.Path, globs: list[str], excludes: list[str]
) -> set[str]:
    excluded = set(excludes)
    return {
        path.relative_to(upstream_root).as_posix()
        for pattern in globs
        for path in upstream_root.glob(pattern)
        if path.is_file() and path.relative_to(upstream_root).as_posix() not in excluded
    }


def expected_metal_shader_source_batch(
    upstream_root: pathlib.Path,
) -> list[tuple[str, str]]:
    shader_root = upstream_root / "renderer/src/shaders"
    batch: list[tuple[pathlib.Path, str]] = [
        (shader_root / "Makefile", "build-recipe"),
        (shader_root / "minify.py", "batch-minifier"),
    ]
    for pattern, stage in (
        ("*.glsl", "minify-input-glsl"),
        ("*.vert", "minify-input-vert"),
        ("*.frag", "minify-input-frag"),
    ):
        batch.extend((path, stage) for path in sorted(shader_root.glob(pattern)))
    batch.append(
        (
            shader_root / "metal/generate_draw_combinations.py",
            "draw-combination-generator",
        )
    )
    batch.extend(
        (path, "metal-input")
        for path in sorted((shader_root / "metal").glob("*.metal"))
    )
    return [
        (path.relative_to(upstream_root).as_posix(), stage) for path, stage in batch
    ]


def validate_metal_shader_translation_unit(
    manifest: dict[str, Any], expected_sources: list[str], errors: list[str]
) -> None:
    units = {
        str(unit.get("id", "")): unit for unit in manifest.get("translation_unit", [])
    }
    unit = units.get("metal-shader-source-batch")
    if unit is None:
        errors.append("missing metal-shader-source-batch translation unit")
        return
    if [str(source) for source in unit.get("sources", [])] != expected_sources:
        errors.append(
            "metal-shader-source-batch sources must exactly match inventory order"
        )
    artifact_targets = tuple(
        str(target) for target in unit.get("artifact_targets", [])
    )
    if artifact_targets != METAL_SHADER_ARTIFACT_TARGETS:
        errors.append(
            "metal-shader-source-batch artifact targets must match the frozen target inventory"
        )
    if unit.get("dispatch_ordinal") != 32:
        errors.append("metal-shader-source-batch dispatch ordinal must be 32")
    if unit.get("worker_role") != "luna-extra-high":
        errors.append("metal-shader-source-batch must use luna-extra-high")
    if unit.get("source_reviewer_role") != "sol-high":
        errors.append("metal-shader-source-batch source reviewer must use sol-high")
    if unit.get("ownership_reviewer_role") != "sol-high":
        errors.append("metal-shader-source-batch ownership reviewer must use sol-high")
    if unit.get("fixer_role") != "sol-high":
        errors.append("metal-shader-source-batch fixer must use sol-high")
    validate_translation_receipts(unit, errors)


def validate_metal_shader_source_inventory(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    relative = str(manifest.get("shader_source_inventory", ""))
    path = repo_root / relative
    if not relative or not path.is_file():
        errors.append(f"missing Metal shader source inventory {relative}")
        return
    if not git_tracked_file(repo_root, relative):
        errors.append(f"untracked Metal shader source inventory {relative}")
    try:
        with path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = tuple(reader.fieldnames or ())
            rows = list(reader)
    except (OSError, csv.Error) as error:
        errors.append(f"cannot read Metal shader source inventory {relative}: {error}")
        return
    if fieldnames != SHADER_SOURCE_INVENTORY_COLUMNS:
        errors.append(
            "Metal shader source inventory schema must be: "
            + "\t".join(SHADER_SOURCE_INVENTORY_COLUMNS)
        )
        return

    expected = expected_metal_shader_source_batch(upstream_root)
    actual_sources = [str(row.get("source", "")) for row in rows]
    expected_sources = [source for source, _ in expected]
    if actual_sources != expected_sources:
        errors.append(
            "Metal shader source inventory must match Makefile wildcard order: "
            "Makefile, minify.py, sorted *.glsl, sorted *.vert, sorted *.frag, "
            "generate_draw_combinations.py, sorted metal/*.metal"
        )
    if len(rows) != 40:
        errors.append(
            f"Metal shader source inventory must contain 40 rows, got {len(rows)}"
        )

    upstream_ref = str(manifest.get("upstream_ref", ""))
    expected_stages = [stage for _, stage in expected]
    for index, row in enumerate(rows, 1):
        source_name = str(row.get("source", ""))
        if row.get("version") != "1":
            errors.append(f"Metal shader source inventory row {index} has invalid version")
        if row.get("upstream_sha") != upstream_ref:
            errors.append(
                f"Metal shader source inventory row {index} pin does not match upstream_ref"
            )
        if row.get("ordinal") != str(index):
            errors.append(
                f"Metal shader source inventory row {index} has ordinal {row.get('ordinal')!r}"
            )
        if index <= len(expected_stages) and row.get("stage") != expected_stages[index - 1]:
            errors.append(
                f"Metal shader source inventory row {index} has wrong stage {row.get('stage')!r}"
            )
        if row.get("disposition") != "full-translation-source":
            errors.append(
                f"Metal shader source inventory row {index} must be a full-translation-source"
            )
        source_path = upstream_root / source_name
        if not source_path.is_file():
            errors.append(
                f"Metal shader source inventory row {index} names missing source {source_name}"
            )
            continue
        text = source_path.read_text(encoding="utf-8")
        if row.get("line_count") != str(len(text.splitlines())):
            errors.append(
                f"Metal shader source inventory row {index} line count drifted for {source_name}"
            )
        if row.get("sha256") != sha256_file(source_path):
            errors.append(
                f"Metal shader source inventory row {index} hash drifted for {source_name}"
            )

    platform_shader_sources = [
        str(row.get("upstream", ""))
        for row in manifest.get("source", [])
        if row.get("lane") == "platform-shaders"
    ]
    if set(platform_shader_sources) != set(expected_sources):
        errors.append(
            "platform-shaders source rows must exactly match the Metal shader source inventory"
        )
    validate_metal_shader_translation_unit(manifest, expected_sources, errors)


def validate_render_context_file_map(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    relative = str(manifest.get("render_context_file_map", ""))
    path = repo_root / relative
    if not relative or not path.is_file():
        errors.append(f"missing render-context file map {relative}")
        return
    if not git_tracked_file(repo_root, relative):
        errors.append(f"untracked render-context file map {relative}")

    try:
        with path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = tuple(reader.fieldnames or ())
            rows = list(reader)
    except (OSError, csv.Error) as error:
        errors.append(f"cannot read render-context file map {relative}: {error}")
        return
    if fieldnames != RENDER_CONTEXT_FILE_MAP_COLUMNS:
        errors.append(
            "render-context file map schema must be: "
            + "\t".join(RENDER_CONTEXT_FILE_MAP_COLUMNS)
        )
        return

    upstream_ref = str(manifest.get("upstream_ref", ""))
    source_units = {
        str(source): unit
        for unit in manifest.get("translation_unit", [])
        for source in unit.get("sources", [])
    }
    rows_by_source: dict[str, list[tuple[int, int, int]]] = collections.defaultdict(
        list
    )
    for line_number, row in enumerate(rows, 2):
        if None in row:
            errors.append(
                f"render-context file map line {line_number} has surplus columns"
            )
        upstream_file = str(row.get("upstream_file", ""))
        if row.get("version") != "1":
            errors.append(
                f"render-context file map line {line_number} has invalid version"
            )
        if row.get("upstream_sha") != upstream_ref:
            errors.append(
                f"render-context file map line {line_number} pin does not match upstream_ref"
            )
        if upstream_file not in RENDER_CONTEXT_FILE_MAP_SOURCES:
            errors.append(
                f"render-context file map line {line_number} names unexpected source {upstream_file}"
            )
        status = str(row.get("status", ""))
        if status not in RENDER_CONTEXT_FILE_MAP_STATUSES:
            errors.append(
                f"render-context file map line {line_number} has invalid status `{status}`"
            )
        if status == "ported" and (
            upstream_file not in source_units
            or source_units[upstream_file].get("status")
            not in {"fixed", "compiled", "verified"}
        ):
            errors.append(
                f"render-context file map line {line_number} ported status outruns owning unit receipts"
            )
        if not str(row.get("symbol", "")).strip():
            errors.append(
                f"render-context file map line {line_number} has an empty symbol"
            )
        if not str(row.get("remaining", "")).strip():
            errors.append(
                f"render-context file map line {line_number} has empty remaining work"
            )
        rust_owner = str(row.get("rust_owner", ""))
        if status == "missing":
            if rust_owner != "-":
                errors.append(
                    f"render-context file map line {line_number} marks missing work with a Rust owner"
                )
        elif rust_owner == "-":
            errors.append(
                f"render-context file map line {line_number} is {status} without a Rust owner"
            )
        elif not (repo_root / rust_owner).is_file():
            errors.append(
                f"render-context file map line {line_number} names missing Rust owner {rust_owner}"
            )
        elif not git_tracked_file(repo_root, rust_owner):
            errors.append(
                f"render-context file map line {line_number} names untracked Rust owner {rust_owner}"
            )

        range_match = re.fullmatch(r"(\d+)-(\d+)", str(row.get("lines", "")))
        if range_match is None:
            errors.append(
                f"render-context file map line {line_number} has invalid line range"
            )
            continue
        start, end = (int(value) for value in range_match.groups())
        if start < 1 or end < start:
            errors.append(
                f"render-context file map line {line_number} has invalid line range {start}-{end}"
            )
            continue
        rows_by_source[upstream_file].append((line_number, start, end))

    mapped_sources = set(rows_by_source)
    if mapped_sources != RENDER_CONTEXT_FILE_MAP_SOURCES:
        missing = sorted(RENDER_CONTEXT_FILE_MAP_SOURCES - mapped_sources)
        extra = sorted(mapped_sources - RENDER_CONTEXT_FILE_MAP_SOURCES)
        if missing:
            errors.append("render-context file map omits sources: " + ", ".join(missing))
        if extra:
            errors.append(
                "render-context file map includes extra sources: " + ", ".join(extra)
            )

    for upstream_file in sorted(RENDER_CONTEXT_FILE_MAP_SOURCES):
        source_path = upstream_root / upstream_file
        if not source_path.is_file():
            errors.append(
                f"render-context file map source does not exist: {upstream_file}"
            )
            continue
        with source_path.open(encoding="utf-8", errors="replace") as source:
            line_count = sum(1 for _ in source)
        expected_start = 1
        for line_number, start, end in rows_by_source.get(upstream_file, []):
            if start != expected_start:
                errors.append(
                    "render-context file map does not continuously cover "
                    f"{upstream_file}: line {line_number} starts at {start}, "
                    f"expected {expected_start}"
                )
            if end > line_count:
                errors.append(
                    f"render-context file map line {line_number} ends outside {upstream_file}"
                )
            expected_start = end + 1
        if expected_start != line_count + 1:
            errors.append(
                "render-context file map does not reach the end of "
                f"{upstream_file}: stopped at {expected_start - 1}, expected {line_count}"
            )


def extract_render_context_field_declarations(
    upstream_root: pathlib.Path,
    errors: list[str],
) -> dict[tuple[str, str, str], tuple[str, int, str]]:
    extractor = pathlib.Path(__file__).with_name("extract_field_authority.py")
    owner_input = pathlib.Path(__file__).parents[2] / RENDER_CONTEXT_FIELD_OWNER_INPUT
    process = subprocess.run(
        [
            sys.executable,
            str(extractor),
            "--upstream-root",
            str(upstream_root),
            "--owners",
            str(owner_input),
            "--sources",
            str(pathlib.Path(__file__).parents[2] / RENDER_CONTEXT_FIELD_SOURCE_INPUT),
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if process.returncode:
        errors.append("Clang render-context field extraction failed: " + process.stderr.strip())
        return {}
    reader = csv.DictReader(process.stdout.splitlines(), delimiter="\t")
    declarations = {
        (row["upstream_file"], row["cpp_type"], row["cpp_field"]): (
            row["cpp_declared_type"],
            int(row["declaration_line"]),
            row["configuration"],
        )
        for row in reader
    }
    if len(declarations) != RENDER_CONTEXT_FIELD_EXPECTED_COUNT:
        errors.append(
            "Clang render-context field authority has "
            f"{len(declarations)} declarations, expected {RENDER_CONTEXT_FIELD_EXPECTED_COUNT}"
        )
    return declarations


def compare_render_context_field_rows(
    rows: list[dict[str, str]],
    declarations: dict[tuple[str, str, str], tuple[str, int, str]],
    errors: list[str],
) -> None:
    ledger: dict[tuple[str, str, str], tuple[int, str, int, str]] = {}
    for line_number, row in enumerate(rows, 2):
        key = (row["upstream_file"], row["cpp_type"], row["cpp_field"])
        try:
            declaration_line = int(row["declaration_line"])
        except ValueError:
            errors.append(
                f"render-context field map line {line_number} has invalid declaration line"
            )
            continue
        if key in ledger:
            errors.append("duplicate render-context field row: " + ":".join(key))
        ledger[key] = (
            line_number,
            row.get("cpp_declared_type", ""),
            declaration_line,
            row.get("configuration", ""),
        )

    missing = sorted(set(declarations) - set(ledger))
    extra = sorted(set(ledger) - set(declarations))
    if missing:
        errors.append(
            "render-context field map omits declarations: "
            + ", ".join(":".join(key) for key in missing)
        )
    if extra:
        errors.append(
            "render-context field map invents declarations: "
            + ", ".join(":".join(key) for key in extra)
        )
    for key in sorted(set(declarations) & set(ledger)):
        line_number, actual_type, actual_line, actual_configuration = ledger[key]
        expected_type, expected_line, expected_configuration = declarations[key]
        if actual_type != expected_type:
            errors.append(
                f"render-context field map line {line_number} types "
                f"{':'.join(key)} as {actual_type!r}, expected {expected_type!r}"
            )
        if actual_line != expected_line:
            errors.append(
                f"render-context field map line {line_number} locates "
                f"{':'.join(key)} at {actual_line}, expected {expected_line}"
            )
        if actual_configuration != expected_configuration:
            errors.append(
                f"render-context field map line {line_number} configures "
                f"{':'.join(key)} as {actual_configuration!r}, "
                f"expected {expected_configuration!r}"
            )


def validate_render_context_field_map(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    for manifest_key, expected in (
        ("render_context_field_owner_authority", RENDER_CONTEXT_FIELD_OWNER_INPUT),
        ("render_context_field_source_authority", RENDER_CONTEXT_FIELD_SOURCE_INPUT),
        ("render_context_field_extractor", RENDER_CONTEXT_FIELD_EXTRACTOR),
    ):
        configured = str(manifest.get(manifest_key, ""))
        if configured != expected:
            errors.append(
                f"{manifest_key} must be {expected}, got {configured or '<missing>'}"
            )
            continue
        authority_path = repo_root / expected
        if not authority_path.is_file():
            errors.append(f"missing render-context field authority {expected}")
        elif not git_tracked_file(repo_root, expected):
            errors.append(f"untracked render-context field authority {expected}")
    relative = str(manifest.get("render_context_field_map", ""))
    path = repo_root / relative
    if not relative or not path.is_file():
        errors.append(f"missing render-context field map {relative}")
        return
    if not git_tracked_file(repo_root, relative):
        errors.append(f"untracked render-context field map {relative}")

    try:
        with path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = tuple(reader.fieldnames or ())
            rows = [
                {str(key): str(value or "") for key, value in row.items() if key is not None}
                for row in reader
            ]
    except (OSError, csv.Error) as error:
        errors.append(f"cannot read render-context field map {relative}: {error}")
        return
    if fieldnames != RENDER_CONTEXT_FIELD_MAP_COLUMNS:
        errors.append(
            "render-context field map schema must be: "
            + "\t".join(RENDER_CONTEXT_FIELD_MAP_COLUMNS)
        )
        return

    declarations = extract_render_context_field_declarations(upstream_root, errors)
    compare_render_context_field_rows(rows, declarations, errors)
    upstream_ref = str(manifest.get("upstream_ref", ""))
    source_unit = {
        str(source): unit
        for unit in manifest.get("translation_unit", [])
        for source in unit.get("sources", [])
    }
    required_prose = (
        "configuration",
        "rust_field",
        "construction_and_publication",
        "mutation_thread",
        "submission_and_completion",
        "destruction_order",
        "null_and_failure",
        "safe_rust_adaptation",
    )
    for line_number, row in enumerate(rows, 2):
        if row["version"] != "1":
            errors.append(f"render-context field map line {line_number} has invalid version")
        if row["upstream_sha"] != upstream_ref:
            errors.append(
                f"render-context field map line {line_number} pin does not match upstream_ref"
            )
        mapping_status = row["mapping_status"]
        if mapping_status not in RENDER_CONTEXT_FIELD_MAPPING_STATUSES:
            errors.append(
                "render-context field map line "
                f"{line_number} has invalid mapping status `{mapping_status}`"
            )
        translation_status = row["translation_status"]
        if translation_status not in RENDER_CONTEXT_FIELD_TRANSLATION_STATUSES:
            errors.append(
                "render-context field map line "
                f"{line_number} has invalid translation status `{translation_status}`"
            )
        owning_unit = source_unit.get(row["upstream_file"])
        expected_translation_status = derived_coverage_status(owning_unit)
        if translation_status != expected_translation_status:
            errors.append(
                f"render-context field map line {line_number} translation status "
                f"must be {expected_translation_status} for its receipt-gated owning unit"
            )
        if translation_status in {"translated", "verified"} and (
            owning_unit is None
            or owning_unit.get("status")
            not in {"translated", "reviewed", "fixed", "compiled", "verified"}
        ):
            errors.append(
                f"render-context field map line {line_number} translation outruns its owning unit receipts"
            )
        if (
            translation_status == "verified"
            and owning_unit is not None
            and owning_unit.get("status") not in {"compiled", "verified"}
        ):
            errors.append(
                f"render-context field map line {line_number} verification outruns its owning unit"
            )
        for column in required_prose:
            if not row[column].strip():
                errors.append(
                    f"render-context field map line {line_number} has empty {column}"
                )
        normalized_mapping = " ".join(
            row[column].lower() for column in required_prose
        )
        declared_type = row["cpp_declared_type"]
        if "std::vector<" in declared_type and not any(
            term in normalized_mapping for term in ("vec<", "vec ", "vector", "container")
        ):
            errors.append(f"render-context field map line {line_number} collapses a Clang vector")
        if "std::vector<" in declared_type and "unique_ptr" in declared_type and not any(
            term in normalized_mapping for term in ("box<", "owned", "owner")
        ):
            errors.append(f"render-context field map line {line_number} loses vector element ownership")
        if all(token not in declared_type for token in ("*", "&", "std::vector<")) and "nonnull" in row["rust_field"].lower():
            errors.append(f"render-context field map line {line_number} invents pointer ownership from source text")
        if "TrivialArrayAllocator" in declared_type and "arena" not in row["rust_field"].lower():
            errors.append(f"render-context field map line {line_number} loses allocator ownership")
        if mapping_status == "prepared" and any(
            marker in normalized_mapping
            for marker in RENDER_CONTEXT_FIELD_PLACEHOLDER_MARKERS
        ):
            errors.append(
                f"render-context field map line {line_number} marks placeholder prose prepared"
            )
        if mapping_status == "prepared" and re.search(
            r"(?:HashMap|BTreeMap)<[^>]*\b_\b[^>]*>",
            " ".join(row[column] for column in required_prose),
        ):
            errors.append(
                f"render-context field map line {line_number} leaves map key/value types inferred"
            )
        if (
            mapping_status == "prepared"
            and declared_type in {
                "std::vector<DrawSortEntry>",
                "std::vector<gpu::TwoTexelRamp>",
                "std::vector<ClipInfo>",
            }
            and any(term in normalized_mapping for term in ("pointer elements", "borrowed links"))
        ):
            errors.append(
                f"render-context field map line {line_number} invents pointer semantics for a value vector"
            )
        if (
            mapping_status == "prepared"
            and declared_type == "volatile uint8_t[152]"
            and "[" not in row["rust_field"]
        ):
            errors.append(
                f"render-context field map line {line_number} loses fixed-array shape"
            )
        exact_field_contracts = {
            ("RenderContext::LogicalFlush", "m_ctx"): "&'ctx RenderContext",
            ("RenderContext::TessellationWriter", "m_flush"): "&'flush mut LogicalFlush",
            ("RenderContext::TessellationWriter", "m_tessSpanData"): "&'span mut WriteOnlyMappedMemory",
            ("RenderContext::LogicalFlush", "m_drawList"): "ArenaLinkedList<'flush, DrawBatch>",
            ("RenderContext::LogicalFlush", "m_firstDstBlendBarrier"): "Option<NonNull<DrawBatch>>",
            ("FlushUniforms", "m_padTo256Bytes"): "[u8; 152]",
        }
        exact_fragment = exact_field_contracts.get((row["cpp_type"], row["cpp_field"]))
        if mapping_status == "prepared" and exact_fragment and exact_fragment not in row["rust_field"]:
            errors.append(
                f"render-context field map line {line_number} violates exact field contract for "
                f"{row['cpp_type']}.{row['cpp_field']}"
            )
        if (
            mapping_status == "prepared"
            and "mechanical_port/source/" in row["rust_field"]
        ):
            if not any(
                term in normalized_mapping
                for term in (
                    "owner",
                    "borrow",
                    "pointer",
                    "atomic",
                    "container",
                    "copied value",
                    "mapped-memory",
                    "allocator",
                    "value state",
                    "scalar",
                    "enum",
                    "record",
                    "bitflags",
                    "volatile",
                    "abi",
                    "fixed-size",
                )
            ):
                errors.append(
                    f"render-context field map line {line_number} lacks substantive ownership semantics"
                )
        rust_owner = row["rust_owner"]
        if rust_owner == "-":
            if translation_status != "pending":
                errors.append(
                    "render-context field map line "
                    f"{line_number} lacks a Rust owner but translation is {translation_status}"
                )
        elif not (repo_root / rust_owner).is_file():
            errors.append(
                f"render-context field map line {line_number} names missing Rust owner {rust_owner}"
            )
        elif not git_tracked_file(repo_root, rust_owner):
            errors.append(
                f"render-context field map line {line_number} names untracked Rust owner {rust_owner}"
            )
        evidence = [value.strip() for value in row["evidence"].split(";") if value.strip()]
        if mapping_status == "prepared" and not evidence:
            errors.append(
                f"render-context field map line {line_number} is prepared without evidence"
            )
        if translation_status in {"translated", "verified"} and not evidence:
            errors.append(
                "render-context field map line "
                f"{line_number} is {translation_status} without evidence"
            )
        if mapping_status == "review-needed" and translation_status != "pending":
            errors.append(
                "render-context field map line "
                f"{line_number} cannot translate an unresolved mapping"
            )
        for citation in evidence:
            validate_evidence_citation(citation, repo_root, upstream_root, errors)


def extract_render_context_configuration_blocks(
    upstream_root: pathlib.Path, errors: list[str]
) -> dict[tuple[str, int, int], tuple[int, ...]]:
    blocks: dict[tuple[str, int, int], tuple[int, ...]] = {}
    opening = re.compile(r"^\s*#\s*(?:if\s|ifdef\s|ifndef\s)")
    branch = re.compile(r"^\s*#\s*(?:elif\s|else(?:\s|$))")
    closing = re.compile(r"^\s*#\s*endif(?:\s|$)")
    for relative in sorted(RENDER_CONTEXT_CONFIGURATION_MAP_SOURCES):
        path = upstream_root / relative
        if not path.is_file():
            errors.append(f"missing pinned configuration source {relative}")
            continue
        stack: list[tuple[int, list[int]]] = []
        lines = path.read_text(encoding="utf-8").splitlines()
        for line_number, line in enumerate(lines, 1):
            if opening.match(line):
                stack.append((line_number, [line_number]))
            elif branch.match(line):
                if not stack:
                    errors.append(
                        f"orphan preprocessor branch in {relative}:{line_number}"
                    )
                else:
                    stack[-1][1].append(line_number)
            elif closing.match(line):
                if not stack:
                    errors.append(f"orphan #endif in {relative}:{line_number}")
                    continue
                start, branches = stack.pop()
                opening_line = lines[start - 1].strip()
                macro_match = re.fullmatch(r"#\s*ifndef\s+([A-Za-z_][A-Za-z0-9_]*)", opening_line)
                is_outer_guard = (
                    start <= 10
                    and line_number == len(lines)
                    and macro_match is not None
                    and start < len(lines)
                    and re.fullmatch(
                        rf"#\s*define\s+{re.escape(macro_match.group(1))}",
                        lines[start].strip(),
                    )
                )
                if not is_outer_guard:
                    blocks[(relative, start, line_number)] = tuple(branches)
        if stack:
            errors.append(
                f"unterminated preprocessor blocks in {relative}: "
                + ", ".join(str(start) for start, _ in stack)
            )
    return blocks


def compare_render_context_configuration_rows(
    rows: list[dict[str, str]],
    blocks: dict[tuple[str, int, int], tuple[int, ...]],
    errors: list[str],
) -> None:
    ledger: dict[tuple[str, int, int], tuple[int, tuple[int, ...]]] = {}
    for line_number, row in enumerate(rows, 2):
        match = re.fullmatch(r"(\d+)-(\d+)", row.get("lines", ""))
        if not match:
            errors.append(
                f"render-context configuration map line {line_number} has invalid range"
            )
            continue
        start, end = map(int, match.groups())
        try:
            branches = tuple(
                int(value) for value in row.get("branch_lines", "").split(",")
            )
        except ValueError:
            errors.append(
                f"render-context configuration map line {line_number} has invalid branch lines"
            )
            continue
        key = (row.get("upstream_file", ""), start, end)
        if key in ledger:
            errors.append(
                "duplicate render-context configuration row: "
                + f"{key[0]}:{start}-{end}"
            )
        ledger[key] = (line_number, branches)

    missing = sorted(set(blocks) - set(ledger))
    extra = sorted(set(ledger) - set(blocks))
    if missing:
        errors.append(
            "render-context configuration map omits blocks: "
            + ", ".join(f"{path}:{start}-{end}" for path, start, end in missing)
        )
    if extra:
        errors.append(
            "render-context configuration map invents blocks: "
            + ", ".join(f"{path}:{start}-{end}" for path, start, end in extra)
        )
    for key in sorted(set(blocks) & set(ledger)):
        line_number, actual = ledger[key]
        expected = blocks[key]
        if actual != expected:
            errors.append(
                f"render-context configuration map line {line_number} has branch lines "
                f"{actual}, expected {expected} for {key[0]}:{key[1]}-{key[2]}"
            )


def validate_render_context_configuration_map(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    relative = str(manifest.get("render_context_configuration_map", ""))
    path = repo_root / relative
    if not relative or not path.is_file():
        errors.append(f"missing render-context configuration map {relative}")
        return
    if not git_tracked_file(repo_root, relative):
        errors.append(f"untracked render-context configuration map {relative}")
    try:
        with path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = tuple(reader.fieldnames or ())
            rows = [
                {str(key): str(value or "") for key, value in row.items() if key is not None}
                for row in reader
            ]
    except (OSError, csv.Error) as error:
        errors.append(f"cannot read render-context configuration map {relative}: {error}")
        return
    if fieldnames != RENDER_CONTEXT_CONFIGURATION_MAP_COLUMNS:
        errors.append(
            "render-context configuration map schema must be: "
            + "\t".join(RENDER_CONTEXT_CONFIGURATION_MAP_COLUMNS)
        )
        return

    blocks = extract_render_context_configuration_blocks(upstream_root, errors)
    compare_render_context_configuration_rows(rows, blocks, errors)
    upstream_ref = str(manifest.get("upstream_ref", ""))
    required_prose = (
        "block",
        "configurations",
        "source_behavior",
        "rust_configuration",
        "remaining",
    )
    block_names: set[str] = set()
    source_units = {
        str(source): unit
        for unit in manifest.get("translation_unit", [])
        for source in unit.get("sources", [])
    }
    for line_number, row in enumerate(rows, 2):
        if row["version"] != "1":
            errors.append(
                f"render-context configuration map line {line_number} has invalid version"
            )
        if row["upstream_sha"] != upstream_ref:
            errors.append(
                f"render-context configuration map line {line_number} pin does not match upstream_ref"
            )
        if row["block"] in block_names:
            errors.append(
                f"duplicate render-context configuration block name `{row['block']}`"
            )
        block_names.add(row["block"])
        mapping_status = row["mapping_status"]
        if mapping_status not in RENDER_CONTEXT_CONFIGURATION_MAPPING_STATUSES:
            errors.append(
                "render-context configuration map line "
                f"{line_number} has invalid mapping status `{mapping_status}`"
            )
        translation_status = row["translation_status"]
        if translation_status not in RENDER_CONTEXT_CONFIGURATION_TRANSLATION_STATUSES:
            errors.append(
                "render-context configuration map line "
                f"{line_number} has invalid translation status `{translation_status}`"
            )
        expected_translation_status = derived_coverage_status(
            source_units.get(row["upstream_file"])
        )
        if translation_status != expected_translation_status:
            errors.append(
                f"render-context configuration map line {line_number} translation status "
                f"must be {expected_translation_status} for its receipt-gated owning unit"
            )
        translation_disposition = row["translation_disposition"]
        if (
            translation_disposition
            not in RENDER_CONTEXT_CONFIGURATION_TRANSLATION_DISPOSITIONS
        ):
            errors.append(
                "render-context configuration map line "
                f"{line_number} has invalid translation disposition "
                f"`{translation_disposition}`"
            )
        validation_disposition = row["validation_disposition"]
        if (
            validation_disposition
            not in RENDER_CONTEXT_CONFIGURATION_VALIDATION_DISPOSITIONS
        ):
            errors.append(
                "render-context configuration map line "
                f"{line_number} has invalid validation disposition "
                f"`{validation_disposition}`"
            )
        if translation_disposition == "required" and row["exclusion_reason"] != "-":
            errors.append(
                "render-context configuration map line "
                f"{line_number} is required but supplies an exclusion reason"
            )
        if translation_disposition == "excluded":
            if translation_status != "verified":
                errors.append(
                    "render-context configuration map line "
                    f"{line_number} is excluded without verified evidence"
                )
            if validation_disposition != "checked-exclusion":
                errors.append(
                    "render-context configuration map line "
                    f"{line_number} excludes translation without a checked-exclusion validation"
                )
            if row["exclusion_reason"] == "-":
                errors.append(
                    "render-context configuration map line "
                    f"{line_number} excludes translation without a reason"
                )
        for column in required_prose:
            if not row[column].strip():
                errors.append(
                    f"render-context configuration map line {line_number} has empty {column}"
                )
        rust_owner = row["rust_owner"]
        if rust_owner == "-":
            if mapping_status != "review-needed" or translation_status != "pending":
                errors.append(
                    "render-context configuration map line "
                    f"{line_number} lacks a Rust owner but is "
                    f"{mapping_status}/{translation_status}"
                )
        elif not (repo_root / rust_owner).is_file():
            errors.append(
                f"render-context configuration map line {line_number} names missing Rust owner {rust_owner}"
            )
        elif not git_tracked_file(repo_root, rust_owner):
            errors.append(
                f"render-context configuration map line {line_number} names untracked Rust owner {rust_owner}"
            )
        evidence = [value.strip() for value in row["evidence"].split(";") if value.strip()]
        if translation_status in {"translated", "verified"} and not evidence:
            errors.append(
                "render-context configuration map line "
                f"{line_number} is {translation_status} without evidence"
            )
        if mapping_status == "review-needed" and translation_status != "pending":
            errors.append(
                "render-context configuration map line "
                f"{line_number} cannot translate an unresolved mapping"
            )
        for citation in evidence:
            validate_evidence_citation(citation, repo_root, upstream_root, errors)


def read_campaign_tsv(
    manifest: dict[str, Any],
    key: str,
    columns: tuple[str, ...],
    label: str,
    repo_root: pathlib.Path,
    errors: list[str],
) -> list[dict[str, str]]:
    relative = str(manifest.get(key, ""))
    path = repo_root / relative
    if not relative or not path.is_file():
        errors.append(f"missing {label} {relative}")
        return []
    if not git_tracked_file(repo_root, relative):
        errors.append(f"untracked {label} {relative}")
    try:
        with path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = tuple(reader.fieldnames or ())
            rows = list(reader)
    except (OSError, csv.Error) as error:
        errors.append(f"cannot read {label} {relative}: {error}")
        return []
    if fieldnames != columns:
        errors.append(f"{label} schema must be: " + "\t".join(columns))
        return []
    return rows


def validate_render_context_dependency_map(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> list[dict[str, str]]:
    rows = read_campaign_tsv(
        manifest,
        "render_context_dependency_map",
        RENDER_CONTEXT_DEPENDENCY_MAP_COLUMNS,
        "render-context dependency map",
        repo_root,
        errors,
    )
    if not rows:
        return rows
    actual_sources = [row["upstream_file"] for row in rows]
    if tuple(actual_sources) != RENDER_CONTEXT_DEPENDENCY_SOURCES:
        missing = sorted(set(RENDER_CONTEXT_DEPENDENCY_SOURCES) - set(actual_sources))
        extra = sorted(set(actual_sources) - set(RENDER_CONTEXT_DEPENDENCY_SOURCES))
        errors.append(
            f"render-context dependency map must contain the {len(RENDER_CONTEXT_DEPENDENCY_SOURCES)} pinned sources in source order"
            + (f"; missing: {', '.join(missing)}" if missing else "")
            + (f"; invented: {', '.join(extra)}" if extra else "")
        )
    if duplicate_values(actual_sources):
        errors.append("render-context dependency map repeats sources")

    field_relative = str(manifest.get("render_context_field_map", ""))
    try:
        with (repo_root / field_relative).open(encoding="utf-8", newline="") as source:
            field_rows = list(csv.DictReader(source, delimiter="\t"))
    except (OSError, csv.Error):
        field_rows = []
    field_counts = collections.Counter(row.get("upstream_file", "") for row in field_rows)
    units = {
        str(unit.get("id", "")): unit for unit in manifest.get("translation_unit", [])
    }
    upstream_ref = str(manifest.get("upstream_ref", ""))
    for line_number, row in enumerate(rows, 2):
        source_name = row["upstream_file"]
        if row["version"] != "1" or row["upstream_sha"] != upstream_ref:
            errors.append(f"render-context dependency map line {line_number} has a bad version or pin")
        if not (upstream_root / source_name).is_file():
            errors.append(f"render-context dependency map line {line_number} names missing source {source_name}")
        if not row["source_role"].strip() or not row["remaining"].strip():
            errors.append(f"render-context dependency map line {line_number} has empty role or remaining work")
        if row["mapping_status"] not in RENDER_CONTEXT_FIELD_MAPPING_STATUSES:
            errors.append(f"render-context dependency map line {line_number} has invalid mapping status")
        if row["translation_status"] not in RENDER_CONTEXT_FIELD_TRANSLATION_STATUSES:
            errors.append(f"render-context dependency map line {line_number} has invalid translation status")
        if row["mapping_status"] == "review-needed" and row["translation_status"] != "pending":
            errors.append(f"render-context dependency map line {line_number} translates an unresolved mapping")
        unit = units.get(row["translation_unit"])
        if unit is None:
            errors.append(f"render-context dependency map line {line_number} names missing unit {row['translation_unit']}")
        else:
            expected_translation_status = derived_coverage_status(unit)
            if row["translation_status"] != expected_translation_status:
                errors.append(
                    f"render-context dependency map line {line_number} translation status "
                    f"must be {expected_translation_status} for its receipt-gated owning unit"
                )
            if source_name not in {str(value) for value in unit.get("sources", [])}:
                errors.append(f"render-context dependency map line {line_number} source is not owned by unit {row['translation_unit']}")
            if row["translation_target"] not in {str(value) for value in unit.get("rust_targets", [])}:
                errors.append(f"render-context dependency map line {line_number} target is not owned by unit {row['translation_unit']}")
        count = field_counts[source_name]
        expected_coverage = f"fields:{count}" if count else "state-free"
        if source_name == "renderer/src/rive_render_image.cpp":
            expected_coverage = "source-static:textureResourceHashCounter"
        if row["field_coverage"] != expected_coverage:
            errors.append(
                f"render-context dependency map line {line_number} field coverage is {row['field_coverage']!r}, expected {expected_coverage!r}"
            )
        for owner in (value.strip() for value in row["rust_owners"].split(";")):
            if not owner or not (repo_root / owner).is_file():
                errors.append(f"render-context dependency map line {line_number} names missing Rust owner {owner}")
            elif not git_tracked_file(repo_root, owner):
                errors.append(f"render-context dependency map line {line_number} names untracked Rust owner {owner}")
        evidence = [value.strip() for value in row["evidence"].split(";") if value.strip()]
        if row["mapping_status"] == "prepared" and not evidence:
            errors.append(f"render-context dependency map line {line_number} is prepared without evidence")
        for citation in evidence:
            validate_evidence_citation(citation, repo_root, upstream_root, errors)
        if row["translation_status"] in {"translated", "verified"}:
            target = row["translation_target"]
            if not (repo_root / target).is_file() or not git_tracked_file(repo_root, target):
                errors.append(f"render-context dependency map line {line_number} claims translation without tracked target {target}")
    return rows


def extract_direct_include_occurrences(
    upstream_root: pathlib.Path, errors: list[str]
) -> dict[tuple[str, int, str], tuple[str, str]]:
    configuration_blocks = extract_render_context_configuration_blocks(upstream_root, errors)
    by_source: dict[str, list[tuple[int, int]]] = collections.defaultdict(list)
    for source_name, start, end in configuration_blocks:
        by_source[source_name].append((start, end))
    occurrences: dict[tuple[str, int, str], tuple[str, str]] = {}
    include_pattern = re.compile(r'^\s*#include\s*([<"])([^>"]+)[>"]')
    for source_name in RENDER_CONTEXT_DEPENDENCY_SOURCES:
        path = upstream_root / source_name
        lines = path.read_text(encoding="utf-8").splitlines()
        for line_number, line in enumerate(lines, 1):
            match = include_pattern.match(line)
            if match is None:
                continue
            delimiter, token = match.groups()
            kind = "system" if delimiter == "<" else "project"
            if token.startswith("generated/"):
                kind = "generated"
            configs = []
            for start, end in by_source[source_name]:
                if start < line_number < end:
                    opening = lines[start - 1].strip()
                    configs.append(opening.removeprefix("#"))
            configuration = " && ".join(configs) if configs else "all"
            occurrences[(source_name, line_number, token)] = (kind, configuration)
    return occurrences


def resolve_project_include(
    upstream_root: pathlib.Path, source_name: str, token: str
) -> str | None:
    source_path = upstream_root / source_name
    candidates = (
        source_path.parent / token,
        upstream_root / "include" / token,
        upstream_root / "renderer/include" / token,
        upstream_root / "renderer/src" / token,
        upstream_root / "decoders/include" / token,
    )
    for candidate in candidates:
        if candidate.is_file():
            return candidate.relative_to(upstream_root).as_posix()
    return None


def validate_render_context_include_map(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> list[dict[str, str]]:
    rows = read_campaign_tsv(
        manifest,
        "render_context_include_map",
        RENDER_CONTEXT_INCLUDE_MAP_COLUMNS,
        "render-context include map",
        repo_root,
        errors,
    )
    expected = extract_direct_include_occurrences(upstream_root, errors)
    actual: dict[tuple[str, int, str], tuple[int, dict[str, str]]] = {}
    unit_ids = {str(unit.get("id", "")) for unit in manifest.get("translation_unit", [])}
    upstream_ref = str(manifest.get("upstream_ref", ""))
    for ledger_line, row in enumerate(rows, 2):
        try:
            include_line = int(row["include_line"])
        except ValueError:
            errors.append(f"render-context include map line {ledger_line} has invalid include line")
            continue
        key = (row["upstream_file"], include_line, row["include_token"])
        if key in actual:
            errors.append(f"render-context include map repeats {key[0]}:{key[1]}:{key[2]}")
        actual[key] = (ledger_line, row)
        if row["version"] != "1" or row["upstream_sha"] != upstream_ref:
            errors.append(f"render-context include map line {ledger_line} has a bad version or pin")
    missing = sorted(set(expected) - set(actual))
    extra = sorted(set(actual) - set(expected))
    if missing:
        errors.append("render-context include map omits occurrences: " + ", ".join(f"{s}:{n}:{t}" for s, n, t in missing))
    if extra:
        errors.append("render-context include map invents occurrences: " + ", ".join(f"{s}:{n}:{t}" for s, n, t in extra))
    expected_occurrence_count = len(expected)
    expected_token_count = len({key[2] for key in expected})
    if len(rows) != expected_occurrence_count or len({row["include_token"] for row in rows}) != expected_token_count:
        errors.append(
            "render-context include map must contain "
            f"{expected_occurrence_count} occurrences and {expected_token_count} canonical tokens"
        )
    for key in sorted(set(expected) & set(actual)):
        ledger_line, row = actual[key]
        kind, configuration = expected[key]
        if row["include_kind"] != kind or row["configuration"] != configuration:
            errors.append(f"render-context include map line {ledger_line} kind or configuration drifted")
        token = row["include_token"]
        if kind == "project":
            resolution = resolve_project_include(upstream_root, row["upstream_file"], token)
            if resolution is None or row["source_resolution"] != resolution:
                errors.append(f"render-context include map line {ledger_line} has invalid project resolution")
        elif kind == "system" and row["source_resolution"] != f"system:{token}":
            errors.append(f"render-context include map line {ledger_line} has invalid system resolution")
        elif kind == "generated" and row["source_resolution"] != f"generated:renderer/src/{token}":
            errors.append(f"render-context include map line {ledger_line} has invalid generated resolution")
        owner_kind, separator, owner_value = row["correspondence_owner"].partition(":")
        if not separator:
            errors.append(f"render-context include map line {ledger_line} has malformed owner")
        elif owner_kind == "unit" and owner_value not in unit_ids:
            errors.append(f"render-context include map line {ledger_line} names missing unit {owner_value}")
        elif owner_kind == "rust":
            if not (repo_root / owner_value).is_file() or not git_tracked_file(repo_root, owner_value):
                errors.append(f"render-context include map line {ledger_line} names missing or untracked Rust owner {owner_value}")
        elif owner_kind == "toolchain" and owner_value not in RENDER_CONTEXT_INCLUDE_TOOLCHAIN_OWNERS:
            errors.append(f"render-context include map line {ledger_line} names invalid toolchain owner {owner_value}")
        elif owner_kind not in {"unit", "rust", "toolchain"}:
            errors.append(f"render-context include map line {ledger_line} has invalid owner kind {owner_kind}")
        if row["mapping_status"] != "prepared" or not row["remaining"].strip():
            errors.append(f"render-context include map line {ledger_line} is not prepared or has empty remaining work")
        expected_evidence = f"cpp:{key[0]}:{key[1]}-{key[1]}"
        if row["evidence"] != expected_evidence:
            errors.append(f"render-context include map line {ledger_line} must cite {expected_evidence}")
        validate_evidence_citation(row["evidence"], repo_root, upstream_root, errors)
    return rows


def compare_translation_convention_ids(ids: list[str], errors: list[str]) -> None:
    actual_ids = set(ids)
    missing = sorted(TRANSLATION_CONVENTION_IDS - actual_ids)
    extra = sorted(actual_ids - TRANSLATION_CONVENTION_IDS)
    if len(ids) != len(actual_ids):
        errors.append("Metal translation conventions contain duplicate IDs")
    if missing:
        errors.append("Metal translation conventions omit: " + ", ".join(missing))
    if extra:
        errors.append("Metal translation conventions invent: " + ", ".join(extra))


def validate_divergence_ledger(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> list[dict[str, str]]:
    relative = str(manifest.get("divergence_ledger", ""))
    path = repo_root / relative
    if not relative or not path.is_file():
        errors.append(f"missing Metal divergence ledger {relative}")
        return []
    if not git_tracked_file(repo_root, relative):
        errors.append(f"untracked Metal divergence ledger {relative}")
    try:
        with path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = tuple(reader.fieldnames or ())
            rows = [
                {str(key): str(value or "") for key, value in row.items() if key is not None}
                for row in reader
            ]
    except (OSError, csv.Error) as error:
        errors.append(f"cannot read Metal divergence ledger {relative}: {error}")
        return []
    if fieldnames != DIVERGENCE_LEDGER_COLUMNS:
        errors.append(
            "Metal divergence ledger schema must be: "
            + "\t".join(DIVERGENCE_LEDGER_COLUMNS)
        )
        return

    ids = [row["id"] for row in rows]
    if len(ids) != len(set(ids)):
        errors.append("Metal divergence ledger contains duplicate IDs")
    missing = sorted(DIVERGENCE_IDS - set(ids))
    extra = sorted(set(ids) - DIVERGENCE_IDS)
    if missing:
        errors.append("Metal divergence ledger omits: " + ", ".join(missing))
    if extra:
        errors.append("Metal divergence ledger invents: " + ", ".join(extra))

    upstream_ref = str(manifest.get("upstream_ref", ""))
    prose_columns = (
        "source_behavior",
        "rust_behavior",
        "observability",
    )
    for line_number, row in enumerate(rows, 2):
        if row["version"] != "1":
            errors.append(f"Metal divergence ledger line {line_number} has invalid version")
        if row["upstream_sha"] != upstream_ref:
            errors.append(f"Metal divergence ledger line {line_number} pin drifted")
        for column in prose_columns:
            if not row[column].strip():
                errors.append(
                    f"Metal divergence ledger line {line_number} has empty {column}"
                )
        if row["proposed_disposition"] not in DIVERGENCE_DISPOSITIONS:
            errors.append(
                f"Metal divergence ledger line {line_number} has invalid disposition"
            )
        status = row["status"]
        if status not in DIVERGENCE_STATUSES:
            errors.append(
                f"Metal divergence ledger line {line_number} has invalid status `{status}`"
            )
        upstream_source = row["upstream_source"]
        rust_owner = row["rust_owner"]
        if not (upstream_root / upstream_source).is_file():
            errors.append(
                f"Metal divergence ledger line {line_number} names missing upstream source {upstream_source}"
            )
        if not (repo_root / rust_owner).is_file():
            errors.append(
                f"Metal divergence ledger line {line_number} names missing Rust owner {rust_owner}"
            )
        elif not git_tracked_file(repo_root, rust_owner):
            errors.append(
                f"Metal divergence ledger line {line_number} names untracked Rust owner {rust_owner}"
            )
        citations = [
            value.strip() for value in row["evidence"].split(";") if value.strip()
        ]
        if not citations:
            errors.append(f"Metal divergence ledger line {line_number} lacks evidence")
        if not any(value.startswith(f"cpp:{upstream_source}:") for value in citations):
            errors.append(
                f"Metal divergence ledger line {line_number} lacks source evidence"
            )
        if not any(value.startswith(f"rust:{rust_owner}:") for value in citations):
            errors.append(
                f"Metal divergence ledger line {line_number} lacks Rust-owner evidence"
            )
        for citation in citations:
            validate_evidence_citation(citation, repo_root, upstream_root, errors)

        receipts = (
            row["source_review_receipt"],
            row["ownership_review_receipt"],
            row["correction_receipt"],
        )
        if status == "review-needed" and any(value != "-" for value in receipts):
            errors.append(
                f"Metal divergence ledger line {line_number} has receipts before review"
            )
        if status in {"accepted", "rejected", "resolved"}:
            if any(value in {"", "-", "pending"} for value in receipts[:2]):
                errors.append(
                    f"Metal divergence ledger line {line_number} lacks both Sol review receipts"
                )
            divergence_unit = f"divergence-{row['id']}"
            cpp_ranges = [
                value for value in citations if value.startswith(f"cpp:{upstream_source}:")
            ]
            cited_cpp_sources = sorted(
                {
                    value.removeprefix("cpp:").rsplit(":", 1)[0]
                    for value in citations
                    if value.startswith("cpp:")
                }
            )
            for field, value in zip(
                ("source_review_receipt", "ownership_review_receipt"), receipts[:2]
            ):
                expected = canonical_receipt_path(divergence_unit, field)
                if value != expected or not git_tracked_file(repo_root, expected):
                    errors.append(
                        f"Metal divergence ledger line {line_number} requires canonical tracked {field}"
                    )
                else:
                    validate_receipt_contents(
                        repo_root / expected,
                        divergence_unit,
                        field,
                        upstream_ref,
                        errors,
                        repo_root=repo_root,
                        upstream_root=upstream_root,
                        expected_sources=cited_cpp_sources,
                        expected_artifacts=[rust_owner],
                        required_cpp_ranges=cpp_ranges,
                        required_rust_owner=rust_owner,
                        require_scoped_evidence=True,
                    )
            review_ids: list[str] = []
            for value in receipts[:2]:
                try:
                    with (repo_root / value).open("rb") as receipt_source:
                        review_ids.append(
                            str(tomllib.load(receipt_source).get("review_run_id", ""))
                        )
                except (OSError, tomllib.TOMLDecodeError):
                    continue
            if len(review_ids) == 2 and review_ids[0] == review_ids[1]:
                errors.append(
                    f"Metal divergence ledger line {line_number} review receipts must use distinct review_run_id values"
                )
        if status == "resolved" and receipts[2] in {"", "-", "pending"}:
            errors.append(
                f"Metal divergence ledger line {line_number} lacks a correction receipt"
            )
        if status == "resolved":
            divergence_unit = f"divergence-{row['id']}"
            expected = canonical_receipt_path(divergence_unit, "fix_receipt")
            if receipts[2] != expected or not git_tracked_file(repo_root, expected):
                errors.append(
                    f"Metal divergence ledger line {line_number} requires canonical tracked correction receipt"
                )
            else:
                validate_receipt_contents(
                    repo_root / expected,
                    divergence_unit,
                    "fix_receipt",
                    upstream_ref,
                    errors,
                    repo_root=repo_root,
                    upstream_root=upstream_root,
                    expected_artifacts=[rust_owner],
                    required_cpp_ranges=[
                        value
                        for value in citations
                        if value.startswith(f"cpp:{upstream_source}:")
                    ],
                    required_rust_owner=rust_owner,
                    require_scoped_evidence=True,
                )
    return rows


def validate_divergence_promotions(
    manifest: dict[str, Any],
    ownership: dict[str, Any],
    rows: Iterable[dict[str, str]],
    errors: list[str],
) -> None:
    """Prevent implementation/ownership promotion past unresolved divergences."""

    source_units = {
        str(source): unit
        for unit in manifest.get("translation_unit", [])
        for source in unit.get("sources", [])
    }
    source_rows = {
        str(row.get("upstream", "")): row for row in manifest.get("source", [])
    }
    owner_rows = list(ownership.get("owner", []))
    for row in rows:
        status = row.get("status", "")
        resolved = status in {"accepted", "resolved"}
        if resolved:
            continue
        upstream_source = row.get("upstream_source", "")
        divergence_id = row.get("id", "")
        unit = source_units.get(upstream_source)
        if unit is not None and unit.get("status") in {"fixed", "compiled", "verified"}:
            errors.append(
                f"translation unit {unit.get('id')} cannot promote while divergence {divergence_id} is {status}"
            )
        source_row = source_rows.get(upstream_source)
        if source_row is not None and source_row.get("status") in {"ported", "verified"}:
            errors.append(
                f"source {upstream_source} cannot promote while divergence {divergence_id} is {status}"
            )
        for owner in owner_rows:
            citations = [str(value) for value in owner.get("citations", [])]
            if (
                owner.get("status") in {"ported", "verified"}
                and any(value.startswith(f"cpp:{upstream_source}:") for value in citations)
            ):
                errors.append(
                    f"ownership row {owner.get('id')} cannot promote while divergence {divergence_id} is {status}"
                )


def validate_translation_conventions(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    relative = str(manifest.get("translation_conventions", ""))
    path = repo_root / relative
    if not relative or not path.is_file():
        errors.append(f"missing Metal translation conventions {relative}")
        return
    if not git_tracked_file(repo_root, relative):
        errors.append(f"untracked Metal translation conventions {relative}")
    try:
        with path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = tuple(reader.fieldnames or ())
            rows = [
                {str(key): str(value or "") for key, value in row.items() if key is not None}
                for row in reader
            ]
    except (OSError, csv.Error) as error:
        errors.append(f"cannot read Metal translation conventions {relative}: {error}")
        return
    if fieldnames != TRANSLATION_CONVENTION_COLUMNS:
        errors.append(
            "Metal translation convention schema must be: "
            + "\t".join(TRANSLATION_CONVENTION_COLUMNS)
        )
        return
    compare_translation_convention_ids(
        [row["convention"] for row in rows], errors
    )
    for line_number, row in enumerate(rows, 2):
        if row["version"] != "1":
            errors.append(
                f"Metal translation convention line {line_number} has invalid version"
            )
        if row["status"] not in TRANSLATION_CONVENTION_STATUSES:
            errors.append(
                f"Metal translation convention line {line_number} has invalid status `{row['status']}`"
            )
        for column in ("cpp_shape", "rust_rule", "invariant", "forbidden"):
            if not row[column].strip():
                errors.append(
                    f"Metal translation convention line {line_number} has empty {column}"
                )
        evidence = [value.strip() for value in row["evidence"].split(";") if value.strip()]
        if row["status"] in {"frozen", "verified"} and not evidence:
            errors.append(
                f"Metal translation convention line {line_number} is {row['status']} without evidence"
            )
        for citation in evidence:
            validate_evidence_citation(citation, repo_root, upstream_root, errors)


def validate_citation(
    citation: str,
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    match = CITATION_RE.fullmatch(citation)
    if match is None:
        errors.append(f"invalid citation (expected cpp|rust:path:line): {citation}")
        return
    root_kind, relative, start_text, end_text = match.groups()
    root = upstream_root if root_kind == "cpp" else repo_root
    source = root / relative
    if not source.is_file():
        errors.append(f"citation file does not exist: {citation}")
        return
    with source.open(encoding="utf-8", errors="replace") as lines:
        line_count = sum(1 for _ in lines)
    start = int(start_text)
    end = int(end_text or start_text)
    if start < 1 or end < start or end > line_count:
        errors.append(
            f"citation line is outside {relative} (1..{line_count}): {citation}"
        )


def validate_evidence_citation(
    citation: str,
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    head, separator, ranges = citation.rpartition(":")
    parts = ranges.split(",") if separator else []
    if len(parts) > 1 and all(re.fullmatch(r"\d+(?:-\d+)?", part) for part in parts):
        for line_range in parts:
            validate_citation(
                f"{head}:{line_range}", repo_root, upstream_root, errors
            )
        return
    validate_citation(citation, repo_root, upstream_root, errors)


def validate_source_rows(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> collections.Counter[str]:
    expected = expand_source_scope(
        upstream_root,
        [str(value) for value in manifest.get("source_globs", [])],
        [str(value) for value in manifest.get("source_excludes", [])],
    )
    rows = list(manifest.get("source", []))
    paths = [str(row.get("upstream", "")) for row in rows]
    duplicates = duplicate_values(paths)
    if duplicates:
        errors.append(f"duplicate source rows: {', '.join(duplicates)}")
    actual = set(paths)
    missing = sorted(expected - actual)
    extra = sorted(actual - expected)
    if missing:
        errors.append("untracked upstream Metal sources: " + ", ".join(missing))
    if extra:
        errors.append("out-of-scope source rows: " + ", ".join(extra))

    translation_targets_by_source: dict[str, set[str]] = collections.defaultdict(set)
    for unit in manifest.get("translation_unit", []):
        targets = {str(value) for value in unit.get("rust_targets", [])}
        for source in unit.get("sources", []):
            translation_targets_by_source[str(source)].update(targets)

    counts: collections.Counter[str] = collections.Counter()
    for row in rows:
        path = str(row.get("upstream", ""))
        status = str(row.get("status", ""))
        issue = str(row.get("issue", ""))
        lane = str(row.get("lane", ""))
        rust_modules = [str(value) for value in row.get("rust_modules", [])]
        evidence = [str(value) for value in row.get("evidence", [])]
        parity_evidence = [str(value) for value in row.get("parity_evidence", [])]
        owned_targets = translation_targets_by_source.get(path, set())
        if owned_targets and not owned_targets.intersection(rust_modules):
            errors.append(
                f"{path} source card Rust modules do not overlap its translation owner: "
                + ", ".join(sorted(owned_targets))
            )
        if status not in SOURCE_STATUSES:
            errors.append(f"{path} has invalid status `{status}`")
        else:
            counts[status] += 1
        if not re.fullmatch(r"UNIV-\d+", issue):
            errors.append(f"{path} has invalid or missing issue `{issue}`")
        if lane not in {"renderer-core", "renderer-platform", "ore-metal", "platform-shaders"}:
            errors.append(f"{path} has invalid lane `{lane}`")
        if status in VERIFIED_STATUSES:
            if not rust_modules:
                errors.append(f"{path} is {status} without a Rust module")
            if not evidence:
                errors.append(f"{path} is {status} without verification evidence")
            for relative in rust_modules:
                if not (repo_root / relative).is_file():
                    errors.append(f"{path} names missing Rust module {relative}")
                elif not git_tracked_file(repo_root, relative):
                    errors.append(f"{path} names untracked Rust module {relative}")
            for relative in evidence:
                if not (repo_root / relative).is_file():
                    errors.append(f"{path} names missing evidence path {relative}")
                elif not git_tracked_file(repo_root, relative):
                    errors.append(f"{path} names untracked evidence path {relative}")
        if status == "verified":
            if not parity_evidence:
                errors.append(f"{path} is verified without parity evidence")
            for relative in parity_evidence:
                if not (repo_root / relative).is_file():
                    errors.append(f"{path} names missing parity evidence path {relative}")
                elif not git_tracked_file(repo_root, relative):
                    errors.append(f"{path} names untracked parity evidence path {relative}")
    return counts


def validate_mechanical_translation_workflow(
    manifest: dict[str, Any], errors: list[str]
) -> None:
    workflow = manifest.get("mechanical_translation_workflow")
    if not isinstance(workflow, dict):
        errors.append("missing mechanical_translation_workflow contract")
        return
    missing = sorted(set(MECHANICAL_TRANSLATION_WORKFLOW) - set(workflow))
    extra = sorted(set(workflow) - set(MECHANICAL_TRANSLATION_WORKFLOW))
    if missing:
        errors.append(
            "mechanical translation workflow is missing keys: " + ", ".join(missing)
        )
    if extra:
        errors.append(
            "mechanical translation workflow has invented keys: " + ", ".join(extra)
        )
    for key, expected in MECHANICAL_TRANSLATION_WORKFLOW.items():
        actual = workflow.get(key)
        if actual != expected:
            errors.append(
                f"mechanical translation workflow {key} must be {expected!r}, "
                f"got {actual!r}"
            )


def canonical_receipt_path(unit_id: str, field: str) -> str:
    return f"docs/metal-port-receipts/{unit_id}.{TRANSLATION_RECEIPT_SUFFIXES[field]}"


def _citation_ranges_for_path(
    values: Iterable[Any], root_kind: str, relative: str
) -> list[tuple[int, int]]:
    prefix = f"{root_kind}:{relative}:"
    ranges: list[tuple[int, int]] = []
    for value in values:
        if not isinstance(value, str) or not value.startswith(prefix):
            continue
        suffix = value[len(prefix):]
        for item in suffix.split(","):
            match = re.fullmatch(r"(\d+)(?:-(\d+))?", item)
            if match is None:
                continue
            start = int(match.group(1))
            end = int(match.group(2) or match.group(1))
            ranges.append((start, end))
    return ranges


def _citations_cover_entire_file(
    values: Iterable[Any], root_kind: str, relative: str, root: pathlib.Path
) -> bool:
    path = root / relative
    if not path.is_file():
        return False
    with path.open(encoding="utf-8", errors="replace") as source:
        line_count = sum(1 for _ in source)
    if line_count == 0:
        return False
    ranges = sorted(_citation_ranges_for_path(values, root_kind, relative))
    next_line = 1
    for start, end in ranges:
        if start > next_line:
            return False
        if end >= next_line:
            next_line = end + 1
        if next_line > line_count:
            return True
    return next_line > line_count


def _receipt_command_result_count(stdout: str) -> int:
    lines = [line.strip() for line in stdout.splitlines() if line.strip()]
    if len(lines) == 1 and re.fullmatch(r"\d+", lines[0]):
        return int(lines[0])
    return len(lines)


def _replay_receipt_command(
    receipt_path: pathlib.Path,
    declaration: str,
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    marker = " :: exit=0 :: count="
    if marker not in declaration:
        return
    command, expected_text = declaration.rsplit(marker, 1)
    if not expected_text.isdigit():
        return
    cache_key = (str(repo_root), command)
    cached = _RECEIPT_COMMAND_CACHE.get(cache_key)
    if cached is None:
        environment = os.environ.copy()
        environment["RIVE_RUNTIME_DIR"] = str(upstream_root)
        environment["METAL_PORT_RECEIPT_REPLAY_CHILD"] = "1"
        try:
            result = subprocess.run(
                command,
                cwd=repo_root,
                env=environment,
                executable="/bin/zsh",
                shell=True,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=180,
            )
            cached = (result.returncode, result.stdout, result.stderr)
        except subprocess.TimeoutExpired as error:
            cached = (124, error.stdout or "", error.stderr or "")
        _RECEIPT_COMMAND_CACHE[cache_key] = cached
    returncode, stdout, stderr = cached
    if returncode != 0:
        detail = stderr.strip().splitlines()
        suffix = f": {detail[-1]}" if detail else ""
        errors.append(
            f"receipt {receipt_path} command replay exited {returncode}{suffix}: {command}"
        )
        return
    expected = int(expected_text)
    actual = _receipt_command_result_count(stdout)
    if actual != expected:
        errors.append(
            f"receipt {receipt_path} command replay count is {actual}, claimed {expected}: {command}"
        )


def validate_receipt_contents(
    path: pathlib.Path,
    unit_id: str,
    field: str,
    upstream_ref: str,
    errors: list[str],
    *,
    repo_root: pathlib.Path | None = None,
    upstream_root: pathlib.Path | None = None,
    expected_sources: Iterable[str] = (),
    expected_artifacts: Iterable[str] = (),
    required_cpp_ranges: Iterable[str] = (),
    required_rust_owner: str = "",
    require_scoped_evidence: bool = False,
) -> None:
    try:
        with path.open("rb") as source:
            receipt = tomllib.load(source)
    except (OSError, tomllib.TOMLDecodeError) as error:
        errors.append(f"invalid receipt {path}: {error}")
        return
    kind = TRANSLATION_RECEIPT_SUFFIXES[field].removesuffix(".toml")
    common_keys = {
        "schema_version", "unit", "receipt_kind", "upstream_ref",
        "workspace_base_ref", "role", "open_findings", "commands",
        "evidence", "artifact_digests",
    }
    kind_keys = {
        "translation_receipt": {
            "source_digests", "omitted_lines", "omitted_declarations",
            "omitted_conditionals", "omitted_include_owners",
        },
        "source_review_receipt": {"findings", "citations", "review_run_id", "coverage"},
        "ownership_review_receipt": {"findings", "citations", "review_run_id", "coverage"},
        "fix_receipt": {"resolutions"},
        "compile_receipt": {"compiler_diagnostics"},
        "verification_receipt": {"suite_reports"},
    }[field]
    missing_keys = sorted((common_keys | kind_keys) - set(receipt))
    extra_keys = sorted(set(receipt) - (common_keys | kind_keys))
    if missing_keys:
        errors.append(f"receipt {path} is missing schema keys: " + ", ".join(missing_keys))
    if extra_keys:
        errors.append(f"receipt {path} invents schema keys: " + ", ".join(extra_keys))
    expected_role = "luna-extra-high" if field == "translation_receipt" else "sol-high"
    required = {
        "schema_version": 1,
        "unit": unit_id,
        "receipt_kind": kind,
        "upstream_ref": upstream_ref,
        "role": expected_role,
    }
    for key, expected in required.items():
        if receipt.get(key) != expected:
            errors.append(f"receipt {path} {key} must be {expected!r}")
    workspace_ref = str(receipt.get("workspace_base_ref", ""))
    if not re.fullmatch(r"[0-9a-f]{40}", workspace_ref):
        errors.append(f"receipt {path} requires a full workspace_base_ref")
    elif repo_root is not None:
        commit = subprocess.run(
            ["git", "-C", str(repo_root), "cat-file", "-e", f"{workspace_ref}^{{commit}}"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        if commit.returncode:
            errors.append(f"receipt {path} workspace_base_ref is not a repository commit")
        else:
            ancestor = subprocess.run(
                ["git", "-C", str(repo_root), "merge-base", "--is-ancestor", workspace_ref, "HEAD"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            if ancestor.returncode:
                errors.append(
                    f"receipt {path} workspace_base_ref must be an ancestor of current HEAD"
                )

    commands = receipt.get("commands")
    if not isinstance(commands, list) or not commands:
        errors.append(f"receipt {path} requires nonempty commands")
    elif any(
        not isinstance(command, str)
        or not re.fullmatch(r"\S.* :: exit=0 :: count=[1-9]\d*", command)
        for command in commands
    ):
        errors.append(
            f"receipt {path} commands must truthfully claim success as "
            "'<command> :: exit=0 :: count=<positive verified count>'"
        )
    elif (
        REPLAY_RECEIPT_COMMANDS
        and repo_root is not None
        and upstream_root is not None
        and os.environ.get("METAL_PORT_RECEIPT_REPLAY_CHILD") != "1"
    ):
        for command in commands:
            _replay_receipt_command(path, command, repo_root, upstream_root, errors)

    evidence = receipt.get("evidence")
    if not isinstance(evidence, list) or not evidence:
        errors.append(f"receipt {path} requires nonempty evidence")
    elif any(not isinstance(value, str) or not value.strip() for value in evidence):
        errors.append(f"receipt {path} evidence entries must be nonempty strings")
    elif repo_root is not None and upstream_root is not None:
        for value in evidence:
            if value.startswith(("cpp:", "rust:")):
                validate_evidence_citation(value, repo_root, upstream_root, errors)
            elif not (repo_root / value).is_file() or not git_tracked_file(repo_root, value):
                errors.append(f"receipt {path} evidence path is missing or untracked: {value}")
    if require_scoped_evidence:
        scoped_evidence = evidence if isinstance(evidence, list) else []
        if not scoped_evidence or any(
            not isinstance(value, str) or not value.startswith(("cpp:", "rust:"))
            for value in scoped_evidence
        ):
            errors.append(
                f"receipt {path} divergence evidence must consist only of scoped C++/Rust citations"
            )
        required_ranges = {str(value) for value in required_cpp_ranges}
        if required_ranges and not required_ranges <= set(scoped_evidence):
            errors.append(f"receipt {path} must evidence the exact divergence C++ range")
        rust_evidence = {
            value.removeprefix("rust:").rsplit(":", 1)[0]
            for value in scoped_evidence
            if isinstance(value, str) and value.startswith("rust:")
        }
        if required_rust_owner and rust_evidence != {required_rust_owner}:
            errors.append(f"receipt {path} must evidence only the divergence Rust owner")

    artifact_digests = receipt.get("artifact_digests")
    if not isinstance(artifact_digests, dict) or not artifact_digests:
        errors.append(f"receipt {path} requires nonempty artifact_digests")
    else:
        artifact_keys = {str(key) for key in artifact_digests}
        required_artifacts = {str(value) for value in expected_artifacts}
        if required_artifacts and artifact_keys != required_artifacts:
            errors.append(
                f"receipt {path} artifact_digests must exactly cover unit outputs: "
                + ", ".join(sorted(required_artifacts))
            )
        elif not required_artifacts and isinstance(evidence, list):
            allowed = {
                value for value in evidence
                if isinstance(value, str) and not value.startswith(("cpp:", "rust:"))
            }
            if not artifact_keys <= allowed:
                errors.append(f"receipt {path} artifact_digests names non-evidence artifacts")
        for relative, digest in artifact_digests.items():
            if not isinstance(relative, str) or not re.fullmatch(r"[0-9a-f]{64}", str(digest)):
                errors.append(f"receipt {path} has malformed artifact digest for {relative}")
                continue
            artifact = repo_root / relative if repo_root is not None else None
            if artifact is not None and (
                not artifact.is_file() or not git_tracked_file(repo_root, relative)
            ):
                errors.append(f"receipt {path} artifact is missing or untracked: {relative}")
            elif artifact is not None and hashlib.sha256(artifact.read_bytes()).hexdigest() != digest:
                errors.append(f"receipt {path} artifact digest mismatches bytes: {relative}")
    if not isinstance(receipt.get("open_findings"), int):
        errors.append(f"receipt {path} requires integer open_findings")
    elif receipt["open_findings"] != 0:
        errors.append(f"receipt {path} open_findings must be zero before promotion")
    if field == "translation_receipt":
        source_digests = receipt.get("source_digests")
        if not isinstance(source_digests, dict) or not source_digests:
            errors.append(f"receipt {path} requires source_digests")
        else:
            required_sources = {str(value) for value in expected_sources}
            if required_sources and set(source_digests) != required_sources:
                errors.append(
                    f"receipt {path} source_digests must exactly cover unit sources: "
                    + ", ".join(sorted(required_sources))
                )
            for relative, digest in source_digests.items():
                if not isinstance(relative, str) or not re.fullmatch(r"[0-9a-f]{64}", str(digest)):
                    errors.append(f"receipt {path} has malformed source digest for {relative}")
                    continue
                source = upstream_root / relative if upstream_root is not None else None
                if source is not None and not source.is_file():
                    errors.append(f"receipt {path} source is missing: {relative}")
                elif source is not None and hashlib.sha256(source.read_bytes()).hexdigest() != digest:
                    errors.append(f"receipt {path} source digest mismatches pinned bytes: {relative}")
        for key in ("omitted_lines", "omitted_declarations", "omitted_conditionals", "omitted_include_owners"):
            if receipt.get(key) != 0:
                errors.append(f"receipt {path} {key} must be zero")
    elif field in {"source_review_receipt", "ownership_review_receipt"}:
        findings = receipt.get("findings")
        citations = receipt.get("citations")
        review_run_id = receipt.get("review_run_id")
        coverage = receipt.get("coverage")
        expected_coverage = (
            {"owned-source-lines", "declarations", "conditionals", "include-owners", "source-semantics"}
            if field == "source_review_receipt"
            else {"fields", "lifetimes", "threads", "retain-release", "drop-order", "unsafe-invariants", "divergences"}
        )
        if not isinstance(review_run_id, str) or not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._:-]{7,}", review_run_id):
            errors.append(f"receipt {path} requires a unique non-placeholder review_run_id")
        if not isinstance(coverage, list) or set(coverage) != expected_coverage or len(coverage) != len(expected_coverage):
            errors.append(
                f"receipt {path} coverage must exactly record the {field.removesuffix('_receipt')} review contract"
            )
        if not isinstance(findings, list) or not isinstance(citations, list) or not citations:
            errors.append(f"receipt {path} requires findings and citations lists")
        elif findings:
            errors.append(f"receipt {path} cannot promote with recorded findings")
        elif repo_root is not None and upstream_root is not None:
            for citation in citations:
                if not isinstance(citation, str):
                    errors.append(f"receipt {path} citations must be strings")
                else:
                    validate_evidence_citation(citation, repo_root, upstream_root, errors)
            scoped_values = [
                value
                for value in [*citations, *(evidence if isinstance(evidence, list) else [])]
                if isinstance(value, str) and value.startswith(("cpp:", "rust:"))
            ]
            cpp_paths = {
                value.removeprefix("cpp:").rsplit(":", 1)[0]
                for value in scoped_values
                if value.startswith("cpp:")
            }
            rust_paths = {
                value.removeprefix("rust:").rsplit(":", 1)[0]
                for value in scoped_values
                if value.startswith("rust:")
            }
            owned_sources = {str(value) for value in expected_sources}
            rust_artifacts = {
                str(value) for value in expected_artifacts if str(value).endswith(".rs")
            }
            if cpp_paths != owned_sources:
                errors.append(
                    f"receipt {path} cpp citations/evidence must exactly cover owned unit sources"
                )
            if rust_artifacts and rust_paths != rust_artifacts:
                errors.append(
                    f"receipt {path} Rust citations/evidence must exactly cover reviewed Rust artifacts"
                )
            all_scoped_values = [
                value
                for value in [*citations, *(evidence if isinstance(evidence, list) else [])]
                if isinstance(value, str) and value.startswith(("cpp:", "rust:"))
            ]
            for relative in sorted(owned_sources):
                if not _citations_cover_entire_file(
                    all_scoped_values, "cpp", relative, upstream_root
                ):
                    errors.append(
                        f"receipt {path} citations/evidence do not cover every current source line: {relative}"
                    )
            for relative in sorted(rust_artifacts):
                if not _citations_cover_entire_file(
                    all_scoped_values, "rust", relative, repo_root
                ):
                    errors.append(
                        f"receipt {path} citations/evidence do not cover every current Rust artifact line: {relative}"
                    )
            required_ranges = {str(value) for value in required_cpp_ranges}
            if required_ranges and not required_ranges <= set(scoped_values):
                errors.append(
                    f"receipt {path} must cite the exact divergence C++ range"
                )
            if required_rust_owner and rust_paths != {required_rust_owner}:
                errors.append(
                    f"receipt {path} must cite only the divergence Rust owner"
                )
            if require_scoped_evidence:
                if not isinstance(evidence, list) or not evidence or any(
                    not isinstance(value, str)
                    or not value.startswith(("cpp:", "rust:"))
                    for value in evidence
                ):
                    errors.append(
                        f"receipt {path} divergence evidence must consist only of scoped C++/Rust citations"
                    )
    elif field == "fix_receipt":
        resolutions = receipt.get("resolutions")
        if not isinstance(resolutions, list) or not resolutions or any(
            not isinstance(value, str)
            or not re.fullmatch(r"(?:NO_FINDINGS|[A-Za-z0-9][A-Za-z0-9._-]{2,}):\s+\S.*", value)
            for value in resolutions
        ):
            errors.append(
                f"receipt {path} resolutions must preserve stable finding IDs or an exact NO_FINDINGS: audit"
            )
    elif field == "compile_receipt":
        if receipt.get("compiler_diagnostics") != 0:
            errors.append(f"receipt {path} compiler_diagnostics must be zero")
    else:
        suite_reports = receipt.get("suite_reports")
        expected_suites = {f"V{index}" for index in range(10)}
        if not isinstance(suite_reports, dict) or set(suite_reports) != expected_suites:
            errors.append(f"receipt {path} suite_reports must exactly cover V0-V9")
        elif len(set(suite_reports.values())) != 10:
            errors.append(f"receipt {path} V0-V9 suite reports must be distinct")
        elif repo_root is not None:
            for suite, relative in suite_reports.items():
                if (
                    not isinstance(relative, str)
                    or not (repo_root / relative).is_file()
                    or not git_tracked_file(repo_root, relative)
                ):
                    errors.append(
                        f"receipt {path} {suite} report is missing or untracked: {relative}"
                    )


def validate_translation_receipts(
    unit: dict[str, Any],
    errors: list[str],
    repo_root: pathlib.Path | None = None,
    upstream_root: pathlib.Path | None = None,
) -> None:
    unit_id = str(unit.get("id", ""))
    status = str(unit.get("status", ""))
    translation_loop_receipts = (
        "translation_receipt",
        "source_review_receipt",
        "ownership_review_receipt",
        "fix_receipt",
    )
    required_by_status = {
        "pending": (),
        "ready": (),
        "in-progress": (),
        "translated": ("translation_receipt",),
        "reviewed": (
            "translation_receipt",
            "source_review_receipt",
            "ownership_review_receipt",
        ),
        "fixed": translation_loop_receipts,
        "compiled": (*translation_loop_receipts, "compile_receipt"),
        "verified": TRANSLATION_RECEIPT_FIELDS,
    }
    required = set(required_by_status.get(status, ()))
    for field in TRANSLATION_RECEIPT_FIELDS:
        value = str(unit.get(field, "pending"))
        if field not in required:
            if value != "pending":
                errors.append(
                    f"translation unit {unit_id} {status} {field} must be pending"
                )
            continue
        expected = canonical_receipt_path(unit_id, field)
        if value != expected:
            errors.append(
                f"translation unit {unit_id} {status} {field} must be canonical `{expected}`"
            )
            continue
        if repo_root is not None and not git_tracked_file(repo_root, expected):
            errors.append(
                f"translation unit {unit_id} {status} {field} receipt does not exist as a tracked file: {expected}"
            )
        elif repo_root is not None:
            validate_receipt_contents(
                repo_root / expected,
                unit_id,
                field,
                str(unit.get("base_ref", "")),
                errors,
                repo_root=repo_root,
                upstream_root=upstream_root,
                expected_sources=unit.get("sources", []),
                expected_artifacts=[
                    *unit.get("rust_targets", []),
                    *unit.get("artifact_targets", []),
                ],
            )
    if repo_root is not None and {
        "source_review_receipt",
        "ownership_review_receipt",
    } <= required:
        review_ids: list[str] = []
        for field in ("source_review_receipt", "ownership_review_receipt"):
            receipt_path = repo_root / str(unit.get(field, ""))
            try:
                with receipt_path.open("rb") as source:
                    review_ids.append(str(tomllib.load(source).get("review_run_id", "")))
            except (OSError, tomllib.TOMLDecodeError):
                continue
        if len(review_ids) == 2 and review_ids[0] == review_ids[1]:
            errors.append(
                f"translation unit {unit_id} source and ownership reviews must use distinct review_run_id values"
            )
    # Review findings are expected between translation and correction. They
    # remain visible on the unit until the Sol fix/rereview loop closes; only
    # promotion to a fixed or later state requires a zero balance.
    if status in {"fixed", "compiled", "verified"}:
        if unit.get("open_findings") != 0:
            errors.append(
                f"translation unit {unit_id} {status} must have zero open findings"
            )


def mechanical_ore_target(source_name: str) -> str:
    suffix = pathlib.PurePosixPath(source_name).suffix
    stem = source_name[: -len(suffix)] if suffix else source_name
    suffix_map = {".hpp": "_hpp.rs", ".cpp": "_cpp.rs", ".h": "_h.rs", ".mm": "_mm.rs"}
    return "crates/nuxie-ore-metal/src/mechanical_port/source/" + stem + suffix_map[suffix]


def validate_source_unit_promotion(
    manifest: dict[str, Any], repo_root: pathlib.Path, errors: list[str]
) -> None:
    """Keep whole-source and row-level promotion bidirectional."""

    source_rows = {
        str(row.get("upstream", "")): row for row in manifest.get("source", [])
    }
    file_map_path = repo_root / str(manifest.get("render_context_file_map", ""))
    try:
        with file_map_path.open(encoding="utf-8", newline="") as source:
            file_rows = list(csv.DictReader(source, delimiter="\t"))
    except (OSError, csv.Error):
        file_rows = []
    file_statuses: dict[str, set[str]] = collections.defaultdict(set)
    for row in file_rows:
        file_statuses[str(row.get("upstream_file", ""))].add(str(row.get("status", "")))

    for unit in manifest.get("translation_unit", []):
        unit_status = str(unit.get("status", "pending"))
        expected_source_status = (
            "verified"
            if unit_status == "verified"
            else "ported"
            if unit_status == "compiled"
            else "in-progress"
            if unit_status in {"in-progress", "translated", "reviewed", "fixed"}
            else "pending"
        )
        for source_name in (str(value) for value in unit.get("sources", [])):
            source_row = source_rows.get(source_name)
            source_status = source_row.get("status") if source_row is not None else None
            if unit_status == "in-progress" and source_status in {"pending", "in-progress"}:
                # File-primary parallel waves may claim only part of a
                # multi-file integration group at a time.
                pass
            elif source_row is not None and source_status != expected_source_status:
                errors.append(
                    f"source {source_name} status must be {expected_source_status} "
                    f"when owning unit {unit.get('id')} is {unit_status}"
                )
            if unit_status in {"compiled", "verified"} and source_name in file_statuses:
                incomplete = file_statuses[source_name] - {"ported"}
                if incomplete:
                    errors.append(
                        f"translation unit {unit.get('id')} cannot be {unit_status} while "
                        f"file-map rows for {source_name} remain {', '.join(sorted(incomplete))}"
                    )


def validate_translation_units(
    manifest: dict[str, Any],
    errors: list[str],
    repo_root: pathlib.Path | None = None,
    upstream_root: pathlib.Path | None = None,
) -> list[dict[str, Any]]:
    units = list(manifest.get("translation_unit", []))
    unit_ids = [str(unit.get("id", "")) for unit in units]
    duplicates = duplicate_values(unit_ids)
    if duplicates:
        errors.append(f"duplicate translation-unit ids: {', '.join(duplicates)}")

    source_rows = list(manifest.get("source", []))
    all_sources = {
        str(row.get("upstream", "")) for row in source_rows
    }
    assigned_sources = [
        str(source)
        for unit in units
        for source in list(unit.get("sources", []))
    ]
    overlapping_sources = duplicate_values(assigned_sources)
    if overlapping_sources:
        errors.append(
            "overlapping translation-unit sources: "
            + ", ".join(overlapping_sources)
        )
    missing_sources = sorted(all_sources - set(assigned_sources))
    if missing_sources:
        errors.append("missing active manifest sources: " + ", ".join(missing_sources))
    outside_sources = sorted(set(assigned_sources) - all_sources)
    if outside_sources:
        errors.append(
            "translation-unit sources outside the source manifest: "
            + ", ".join(outside_sources)
        )

    upstream_ref = str(manifest.get("upstream_ref", ""))
    rust_target_owners: dict[str, list[str]] = collections.defaultdict(list)
    artifact_target_owners: dict[str, list[str]] = collections.defaultdict(list)
    worker_claims: list[str] = []
    unit_by_id = {str(unit.get("id", "")): unit for unit in units}
    source_dependency_graph: dict[str, list[str]] = {}
    dispatch_graph: dict[str, list[str]] = {}
    for unit in units:
        unit_id = str(unit.get("id", ""))
        sources = [str(source) for source in unit.get("sources", [])]
        source_dependencies = [
            str(value) for value in unit.get("source_dependencies", [])
        ]
        dispatch_prerequisites = [
            str(value) for value in unit.get("dispatch_prerequisites", [])
        ]
        rust_targets = [str(value) for value in unit.get("rust_targets", [])]
        artifact_targets = [str(value) for value in unit.get("artifact_targets", [])]
        phase = str(unit.get("phase", ""))
        status = str(unit.get("status", ""))
        worker_claim = str(unit.get("worker_claim", ""))
        if not re.fullmatch(r"[a-z][a-z0-9-]*", unit_id):
            errors.append(f"translation unit has invalid id `{unit_id}`")
        if not sources:
            errors.append(f"translation unit {unit_id} has no sources")
        if duplicate_values(sources):
            errors.append(f"translation unit {unit_id} repeats a source")
        if phase not in TRANSLATION_PHASES:
            errors.append(f"translation unit {unit_id} has invalid phase `{phase}`")
        if status not in TRANSLATION_STATUSES:
            errors.append(f"translation unit {unit_id} has invalid status `{status}`")
        if str(unit.get("base_ref", "")) != upstream_ref:
            errors.append(
                f"translation unit {unit_id} base_ref does not match upstream_ref"
            )
        if unit.get("worker_role") not in TRANSLATION_WORKER_ROLES:
            errors.append(f"translation unit {unit_id} has invalid worker role")
        if worker_claim != "unclaimed" and not re.fullmatch(
            r"[a-z][a-z0-9-]*", worker_claim
        ):
            errors.append(f"translation unit {unit_id} has invalid worker claim")
        if status != "pending" and worker_claim == "unclaimed":
            errors.append(
                f"translation unit {unit_id} is {status} without a worker claim"
            )
        if worker_claim and worker_claim != "unclaimed":
            worker_claims.append(worker_claim)
        for field in ("source_reviewer_role", "ownership_reviewer_role"):
            if unit.get(field) not in TRANSLATION_REVIEWER_ROLES:
                errors.append(
                    f"translation unit {unit_id} has invalid {field.replace('_', ' ')}"
                )
        if unit.get("fixer_role") not in TRANSLATION_FIXER_ROLES:
            errors.append(f"translation unit {unit_id} has invalid fixer role")
        validate_translation_receipts(unit, errors, repo_root, upstream_root)
        missing_receipt_fields = [
            field for field in TRANSLATION_RECEIPT_FIELDS if field not in unit
        ]
        if missing_receipt_fields:
            errors.append(
                f"translation unit {unit_id} omits receipt fields: "
                + ", ".join(missing_receipt_fields)
            )
        if not isinstance(unit.get("open_findings"), int) or int(unit.get("open_findings", -1)) < 0:
            errors.append(f"translation unit {unit_id} has invalid open_findings")
        is_shader_batch = unit_id == "metal-shader-source-batch"
        expected_lifetime_rows = not is_shader_batch
        if unit.get("requires_lifetime_rows") is not expected_lifetime_rows:
            if expected_lifetime_rows:
                errors.append(
                    f"translation unit {unit_id} must require lifetime rows"
                )
            else:
                errors.append(
                    f"translation unit {unit_id} must not require field lifetime rows"
                )
        if not rust_targets and not artifact_targets:
            errors.append(f"translation unit {unit_id} has no Rust targets")
        for target in rust_targets:
            path = pathlib.PurePosixPath(target)
            canonical_target = path.as_posix()
            if (
                path.is_absolute()
                or ".." in path.parts
                or target in {"", "."}
                or canonical_target != target
                or path.suffix != ".rs"
            ):
                errors.append(
                    f"translation unit {unit_id} Rust target must be a canonical .rs file: {target}"
                )
            allowed_renderer_target = (
                unit_id
                in set(GENERIC_TRANSLATION_UNIT_PLAN)
                | set(METAL_TRANSLATION_UNIT_PLAN)
                | {"metal-shader-source-batch"}
                and target.startswith("crates/nuxie-renderer/src/mechanical_port/source/")
            )
            if not target.startswith("crates/nuxie-ore-metal/src/") and not allowed_renderer_target:
                errors.append(
                    f"translation unit {unit_id} Rust target is outside "
                    f"an allowed mechanical source namespace: {target}"
                )
            rust_target_owners[canonical_target].append(unit_id)
        for target in artifact_targets:
            path = pathlib.PurePosixPath(target)
            canonical_target = path.as_posix()
            if (
                path.is_absolute()
                or ".." in path.parts
                or target in {"", "."}
                or canonical_target != target
            ):
                errors.append(
                    f"translation unit {unit_id} artifact target must be a canonical repo path: {target}"
                )
            artifact_target_owners[canonical_target].append(unit_id)
        if "dependencies" in unit:
            errors.append(
                f"translation unit {unit_id} uses ambiguous dependencies instead of "
                "source_dependencies and dispatch_prerequisites"
            )
        if "source_dependencies" not in unit:
            errors.append(f"translation unit {unit_id} is missing source_dependencies")
        if "dispatch_prerequisites" not in unit:
            errors.append(f"translation unit {unit_id} is missing dispatch_prerequisites")
        if duplicate_values(source_dependencies):
            errors.append(f"translation unit {unit_id} repeats a source dependency")
        if duplicate_values(dispatch_prerequisites):
            errors.append(f"translation unit {unit_id} repeats a dispatch prerequisite")
        if unit_id in source_dependencies:
            errors.append(f"translation unit {unit_id} source-depends on itself")
        if unit_id in dispatch_prerequisites:
            errors.append(f"translation unit {unit_id} dispatch-depends on itself")
        source_dependency_graph[unit_id] = source_dependencies
        dispatch_graph[unit_id] = dispatch_prerequisites

    statuses = {str(unit.get("id", "")): str(unit.get("status", "")) for unit in units}
    if any(status in {"compiled", "verified"} for status in statuses.values()):
        incomplete_translation_loop = sorted(
            unit_id
            for unit_id, status in statuses.items()
            if status not in {"fixed", "compiled", "verified"}
        )
        if incomplete_translation_loop:
            errors.append(
                f"compiler queue cannot start until all {len(MECHANICAL_DISPATCH_ORDER)} units complete the Luna, "
                "two-review, and fix loop; not fixed: "
                + ", ".join(incomplete_translation_loop)
            )
    if any(status == "verified" for status in statuses.values()):
        uncompiled = sorted(
            unit_id
            for unit_id, status in statuses.items()
            if status not in {"compiled", "verified"}
        )
        if uncompiled:
            errors.append(
                f"behavior verification cannot start until all {len(MECHANICAL_DISPATCH_ORDER)} units are compiled; "
                "not compiled: "
                + ", ".join(uncompiled)
            )

    for target, owners in sorted(rust_target_owners.items()):
        if len(owners) > 1:
            errors.append(
                f"Rust target {target} is owned by multiple translation units: "
                + ", ".join(owners)
            )
    for target, owners in sorted(artifact_target_owners.items()):
        if len(owners) > 1:
            errors.append(
                f"artifact target {target} is owned by multiple translation units: "
                + ", ".join(owners)
            )
    duplicate_claims = duplicate_values(worker_claims)
    if duplicate_claims:
        errors.append("duplicate worker claims: " + ", ".join(duplicate_claims))
    for graph_name, graph in (
        ("source dependencies", source_dependency_graph),
        ("dispatch prerequisites", dispatch_graph),
    ):
        for unit_id, dependencies in graph.items():
            missing_dependencies = sorted(set(dependencies) - set(unit_by_id))
            if not missing_dependencies:
                continue
            errors.append(
                f"translation unit {unit_id} has unknown {graph_name}: "
                + ", ".join(missing_dependencies)
            )

    visit_state: dict[str, int] = {}

    def visit(unit_id: str, trail: list[str]) -> None:
        state = visit_state.get(unit_id, 0)
        if state == 2:
            return
        if state == 1:
            cycle_start = trail.index(unit_id) if unit_id in trail else 0
            cycle = trail[cycle_start:] + [unit_id]
            errors.append(
                "translation-unit dispatch prerequisite cycle: "
                + " -> ".join(cycle)
            )
            return
        visit_state[unit_id] = 1
        for dependency in dispatch_graph.get(unit_id, []):
            if dependency in dispatch_graph:
                visit(dependency, trail + [unit_id])
        visit_state[unit_id] = 2

    for unit_id in unit_ids:
        visit(unit_id, [])

    dispatch_ids = set(MECHANICAL_DISPATCH_ORDER)
    dispatch_ordinals = {
        str(unit.get("id", "")): unit.get("dispatch_ordinal")
        for unit in units
        if str(unit.get("id", "")) in dispatch_ids
    }
    enforce_campaign_dispatch = any(
        row.get("lane") in {"renderer-core", "renderer-platform", "platform-shaders"}
        for row in source_rows
    )
    if enforce_campaign_dispatch and (
        set(dispatch_ordinals) != dispatch_ids
        or set(dispatch_ordinals.values())
        != set(range(1, len(MECHANICAL_DISPATCH_ORDER) + 1))
    ):
        errors.append(
            "mechanical dispatch units must own exact ordinals 1 through "
            f"{len(MECHANICAL_DISPATCH_ORDER)}"
        )

    for unit_id in ORE_TRANSLATION_UNIT_ORDER:
        if not enforce_campaign_dispatch:
            break
        unit = unit_by_id.get(unit_id)
        if unit is None:
            errors.append(f"missing requeued ORE translation unit {unit_id}")
            continue
        expected_ordinal = MECHANICAL_DISPATCH_ORDINALS[unit_id]
        if unit.get("dispatch_ordinal") != expected_ordinal:
            errors.append(
                f"translation unit {unit_id} dispatch ordinal must be {expected_ordinal}"
            )
        if unit.get("worker_role") != "luna-extra-high":
            errors.append(f"requeued ORE unit {unit_id} must use luna-extra-high")
        expected_targets = [mechanical_ore_target(str(source)) for source in unit.get("sources", [])]
        if [str(value) for value in unit.get("rust_targets", [])] != expected_targets:
            errors.append(f"requeued ORE unit {unit_id} mechanical targets drifted")
    planned_units = {
        **GENERIC_TRANSLATION_UNIT_PLAN,
        **METAL_TRANSLATION_UNIT_PLAN,
    }

    # The Bun-style bulk pass claims complete source files in parallel waves.
    # Dispatch prerequisites and earlier ordinals must already be claimed, but
    # reviews/fixes intentionally do not serialize later transliteration.
    translation_started_states = {
        "in-progress",
        "translated",
        "reviewed",
        "fixed",
        "compiled",
        "verified",
    }
    for unit in units:
        unit_id = str(unit.get("id", ""))
        advancing = unit.get("worker_claim") != "unclaimed" or unit.get("status") != "pending"
        if not advancing:
            continue
        for prerequisite in unit.get("dispatch_prerequisites", []):
            prerequisite_unit = unit_by_id.get(str(prerequisite))
            if prerequisite_unit is not None and prerequisite_unit.get("status") not in translation_started_states:
                errors.append(
                    f"translation unit {unit_id} advances before prerequisite {prerequisite} is claimed"
                )
        ordinal = int(unit.get("dispatch_ordinal", 0))
        for prior in units:
            prior_ordinal = int(prior.get("dispatch_ordinal", 0))
            if 0 < prior_ordinal < ordinal and prior.get("status") not in translation_started_states:
                errors.append(
                    f"translation unit {unit_id} advances before ordinal {prior_ordinal} {prior.get('id')} is claimed"
                )
    for unit_id, (expected_ordinal, dependencies, sources) in (
        planned_units.items() if enforce_campaign_dispatch else ()
    ):
        unit = unit_by_id.get(unit_id)
        if unit is None:
            errors.append(f"missing planned translation unit {unit_id}")
            continue
        if unit.get("dispatch_ordinal") != expected_ordinal:
            errors.append(f"translation unit {unit_id} dispatch ordinal must be {expected_ordinal}")
        if (
            tuple(str(value) for value in unit.get("dispatch_prerequisites", []))
            != dependencies
        ):
            errors.append(
                f"translation unit {unit_id} dispatch prerequisites drifted"
            )
        if tuple(str(value) for value in unit.get("sources", [])) != sources:
            errors.append(f"translation unit {unit_id} sources drifted")
        expected_authority = (
            "render-context-dependency-map"
            if unit_id in GENERIC_TRANSLATION_UNIT_PLAN
            else "source-manifest"
        )
        if unit.get("source_authority") != expected_authority:
            errors.append(f"translation unit {unit_id} source authority drifted")
        if unit.get("lifetime_authority") != "render-context-field-map":
            errors.append(f"translation unit {unit_id} must use render-context-field-map lifetime authority")
        expected_targets = []
        for source_name in sources:
            suffix = pathlib.PurePosixPath(source_name).suffix
            stem = source_name[: -len(suffix)] if suffix else source_name
            expected_targets.append(
                "crates/nuxie-renderer/src/mechanical_port/source/"
                + stem
                + {".hpp": "_hpp.rs", ".cpp": "_cpp.rs", ".h": "_h.rs", ".mm": "_mm.rs"}[suffix]
            )
        if [str(value) for value in unit.get("rust_targets", [])] != expected_targets:
            errors.append(f"translation unit {unit_id} mechanical targets drifted")
    for unit_id, prerequisites in dispatch_graph.items():
        unit_ordinal = unit_by_id.get(unit_id, {}).get("dispatch_ordinal")
        if not isinstance(unit_ordinal, int):
            continue
        for prerequisite in prerequisites:
            prerequisite_ordinal = unit_by_id.get(prerequisite, {}).get("dispatch_ordinal")
            if isinstance(prerequisite_ordinal, int) and prerequisite_ordinal >= unit_ordinal:
                errors.append(
                    f"translation unit {unit_id} dispatch prerequisite {prerequisite} "
                    f"ordinal {prerequisite_ordinal} must precede consumer ordinal {unit_ordinal}"
                )
    shader_unit = unit_by_id.get("metal-shader-source-batch")
    if shader_unit is not None and shader_unit.get("lifetime_authority") != "artifact-only":
        errors.append("metal-shader-source-batch must use artifact-only lifetime authority")
    for unit_id in ORE_TRANSLATION_UNIT_ORDER:
        unit = unit_by_id.get(unit_id)
        if unit is not None and unit.get("lifetime_authority") != "ore-port-lifetimes":
            errors.append(f"requeued ORE unit {unit_id} must use ore-port-lifetimes lifetime authority")

    trial_units = {
        str(unit.get("id", "")): {str(source) for source in unit.get("sources", [])}
        for unit in units
        if unit.get("phase") == "trial"
    }
    if trial_units != FOUNDATION_TRIAL_UNITS:
        errors.append(
            "trial translation units must be the compileable ore-types, "
            "ore-rstb-container, and ore-binding-map foundations"
        )
    for unit_id in FOUNDATION_TRIAL_UNITS:
        unit = unit_by_id.get(unit_id)
        if unit is not None:
            if unit.get("dispatch_prerequisites"):
                errors.append(
                    f"foundation trial unit {unit_id} must have no dispatch prerequisites"
                )
            if unit.get("worker_role") != "luna-extra-high":
                errors.append(
                    f"foundation trial unit {unit_id} must use luna-extra-high"
                )
            targets = {str(target) for target in unit.get("rust_targets", [])}
            expected_targets = {
                mechanical_ore_target(source)
                for source in FOUNDATION_TRIAL_UNITS[unit_id]
            }
            if targets != expected_targets:
                errors.append(
                    f"foundation trial unit {unit_id} has drifted Rust targets"
                )

    source_owner = {
        str(source): unit
        for unit in units
        for source in unit.get("sources", [])
    }
    for row in source_rows:
        source_name = str(row.get("upstream", ""))
        source_status = str(row.get("status", ""))
        unit = source_owner.get(source_name)
        if unit is None:
            continue
        unit_status = str(unit.get("status", ""))
        unit_id = str(unit.get("id", ""))
        if source_status == "in-progress" and (
            unit_status not in TRANSLATION_STATUSES - {"pending"}
            or unit.get("worker_claim") == "unclaimed"
        ):
            errors.append(
                f"source {source_name} is in-progress ahead of owning unit {unit_id}"
            )
        if source_status == "ported" and unit_status not in {"compiled", "verified"}:
            errors.append(
                f"source {source_name} is {source_status} ahead of owning unit {unit_id} receipts"
            )
        if source_status == "verified" and unit_status != "verified":
            errors.append(
                f"source {source_name} is verified ahead of owning unit {unit_id}"
            )
        if unit_status in {"translated", "reviewed", "fixed"} and source_status == "pending":
            errors.append(f"translation unit {unit_id} advances while source {source_name} is pending")
        if unit_status == "compiled" and source_status not in {"ported", "verified"}:
            errors.append(f"compiled translation unit {unit_id} has unported source {source_name}")
        if unit_status == "verified" and source_status != "verified":
            errors.append(f"verified translation unit {unit_id} has unverified source {source_name}")
    return units


def validate_manifest_targets_are_compiled_modules(
    manifest: dict[str, Any], repo_root: pathlib.Path, errors: list[str]
) -> None:
    """Reject ledger-only targets by requiring an exact compiled import inventory."""

    targets_by_crate: dict[str, set[str]] = collections.defaultdict(set)
    for unit in manifest.get("translation_unit", []):
        for target in unit.get("rust_targets", []):
            relative = str(target)
            match = re.fullmatch(
                r"crates/([^/]+)/src/mechanical_port/source/.+/(?:([^/]+)\.rs)",
                relative,
            )
            if match is not None:
                targets_by_crate[match.group(1)].add(relative)

    for crate, targets in sorted(targets_by_crate.items()):
        crate_root = repo_root / "crates" / crate / "src"
        lib_path = crate_root / "lib.rs"
        module_path = crate_root / "mechanical_port.rs"
        inventory_path = crate_root / "mechanical_port" / "target_inventory.rs"
        if not lib_path.is_file():
            errors.append(f"manifest target crate {crate} has no src/lib.rs")
            continue
        lib_source = lib_path.read_text(encoding="utf-8", errors="replace")
        if re.search(r"\b(?:pub(?:\([^)]*\))?\s+)?mod\s+mechanical_port\s*;", lib_source) is None:
            errors.append(
                f"manifest Rust targets for crate {crate} are compiler-inert: src/lib.rs does not root mechanical_port"
            )
            continue
        if not module_path.is_file():
            errors.append(
                f"manifest Rust targets for crate {crate} have no src/mechanical_port.rs module tree"
            )
            continue
        module_source = module_path.read_text(encoding="utf-8", errors="replace")
        if re.search(r"\bmod\s+target_inventory\s*;", module_source) is None:
            errors.append(
                f"manifest Rust targets for crate {crate} lack the compiled target_inventory module"
            )
            continue
        inventory_lines = [
            "//! @generated by the Metal campaign authority; do not edit by hand.",
            "#![allow(unused_imports)]",
            "",
        ]
        prefix = f"crates/{crate}/src/mechanical_port/"
        for target in sorted(targets):
            module_path_parts = pathlib.PurePosixPath(target.removeprefix(prefix)).with_suffix("").parts
            inventory_lines.append(
                "use crate::mechanical_port::" + "::".join(module_path_parts) + " as _;"
            )
        inventory_lines.extend(
            [
                "",
                f"pub(crate) const MANIFEST_TARGET_COUNT: usize = {len(targets)};",
                "const _: [(); MANIFEST_TARGET_COUNT] = [(); MANIFEST_TARGET_COUNT];",
                "",
            ]
        )
        expected = "\n".join(inventory_lines)
        if not inventory_path.is_file():
            errors.append(
                f"manifest Rust targets for crate {crate} lack generated compiled inventory {inventory_path.relative_to(repo_root)}"
            )
        elif inventory_path.read_text(encoding="utf-8") != expected:
            errors.append(
                f"compiled manifest target inventory drifted for crate {crate}: {inventory_path.relative_to(repo_root)}"
            )


def validate_lifetime_ledger(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> list[dict[str, str]]:
    relative = str(manifest.get("lifetime_ledger", ""))
    path = repo_root / relative
    if not relative or not path.is_file():
        errors.append(f"missing lifetime ledger {relative}")
        return []
    if not git_tracked_file(repo_root, relative):
        errors.append(f"untracked lifetime ledger {relative}")

    try:
        with path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = tuple(reader.fieldnames or ())
            rows = []
            for line_number, row in enumerate(reader, 2):
                if None in row:
                    errors.append(
                        f"lifetime ledger line {line_number} has surplus columns"
                    )
                rows.append(
                    {
                        str(key): str(value or "")
                        for key, value in row.items()
                        if key is not None
                    }
                )
    except (OSError, csv.Error) as error:
        errors.append(f"cannot read lifetime ledger {relative}: {error}")
        return []
    if fieldnames != LIFETIME_COLUMNS:
        errors.append(
            "lifetime ledger schema must be: " + "\t".join(LIFETIME_COLUMNS)
        )
        return rows

    units = list(manifest.get("translation_unit", []))
    units_by_id = {str(unit.get("id", "")): unit for unit in units}
    source_rows = list(manifest.get("source", []))
    ore_sources = {
        str(row.get("upstream", ""))
        for row in source_rows
        if row.get("lane") == "ore-metal"
    }
    upstream_ref = str(manifest.get("upstream_ref", ""))
    ledger_keys: list[str] = []
    rows_by_unit: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    for line_number, row in enumerate(rows, 2):
        unit_id = row["unit"].strip()
        upstream_path = row["upstream_path"].strip()
        field = row["field"].strip()
        status = row["status"].strip()
        row_key = f"{unit_id}:{upstream_path}:{field}"
        ledger_keys.append(row_key)
        if row["schema_version"].strip() != "1":
            errors.append(f"lifetime ledger line {line_number} has invalid schema version")
        if row["upstream_ref"].strip() != upstream_ref:
            errors.append(f"lifetime ledger line {line_number} pin does not match upstream_ref")
        unit = units_by_id.get(unit_id)
        if unit is None:
            errors.append(f"lifetime ledger line {line_number} names unknown unit {unit_id}")
        else:
            rows_by_unit[unit_id].append(row)
            unit_sources = {str(source) for source in unit.get("sources", [])}
            dependency_sources = {
                str(source)
                for dependency in unit.get("source_dependencies", [])
                for source in units_by_id.get(str(dependency), {}).get("sources", [])
            }
            if upstream_path not in unit_sources | dependency_sources:
                errors.append(
                    f"lifetime ledger line {line_number} source is not owned by unit {unit_id}: {upstream_path}"
                )
        if upstream_path not in ore_sources:
            errors.append(
                f"lifetime ledger line {line_number} source is not in the ORE manifest: {upstream_path}"
            )
        if not field:
            errors.append(f"lifetime ledger line {line_number} has an empty field")
        for column in (
            "cpp_ownership",
            "rust_shape",
            "threading",
            "concrete_native_downcast_seam",
            "release_invariant",
            "failure_invariant",
        ):
            if not row[column].strip():
                errors.append(
                    f"lifetime ledger line {line_number} has an empty {column}"
                )
        if status not in LIFETIME_STATUSES:
            errors.append(
                f"lifetime ledger line {line_number} has invalid status `{status}`"
            )
        evidence = [
            value.strip() for value in row["evidence"].split(";") if value.strip()
        ]
        if status in {"prepared", "verified"} and not evidence:
            errors.append(
                f"lifetime ledger line {line_number} is {status} without evidence"
            )
        for citation in evidence:
            validate_evidence_citation(citation, repo_root, upstream_root, errors)
            head, _, _ = citation.rpartition(":")
            root_kind, separator, cited_path = head.partition(":")
            if (
                separator
                and root_kind == "rust"
                and not git_tracked_file(repo_root, cited_path)
            ):
                errors.append(
                    f"lifetime ledger line {line_number} cites untracked Rust evidence {cited_path}"
                )

    duplicates = duplicate_values(ledger_keys)
    if duplicates:
        errors.append("duplicate lifetime ledger rows: " + ", ".join(duplicates))
    for unit in units:
        if unit.get("lifetime_authority") != "ore-port-lifetimes":
            continue
        unit_id = str(unit.get("id", ""))
        unit_rows = rows_by_unit.get(unit_id, [])
        if not unit_rows:
            errors.append(f"translation unit {unit_id} has no lifetime rows")
            continue
        covered_sources = {row["upstream_path"] for row in unit_rows}
        missing_sources = sorted(
            {str(source) for source in unit.get("sources", [])} - covered_sources
        )
        if missing_sources:
            errors.append(
                f"translation unit {unit_id} has sources without lifetime rows: "
                + ", ".join(missing_sources)
            )
        if unit.get("status") != "pending":
            unprepared = [
                row["field"]
                for row in unit_rows
                if row["status"] not in {"prepared", "verified"}
            ]
            if unprepared:
                errors.append(
                    f"translation unit {unit_id} advanced before lifetime preparation: "
                    + ", ".join(unprepared)
                )
    return rows


def validate_owner_rows(
    ownership: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
    manifest: dict[str, Any] | None = None,
) -> tuple[list[dict[str, Any]], collections.Counter[str]]:
    owners = list(ownership.get("owner", []))
    owner_ids = [str(row.get("id", "")) for row in owners]
    duplicates = duplicate_values(owner_ids)
    if duplicates:
        errors.append(f"duplicate ownership rows: {', '.join(duplicates)}")
    counts: collections.Counter[str] = collections.Counter()
    source_units = {
        str(source): unit
        for unit in (manifest or {}).get("translation_unit", [])
        for source in unit.get("sources", [])
    }
    for row in owners:
        owner_id = str(row.get("id", ""))
        status = str(row.get("status", ""))
        issue = str(row.get("issue", ""))
        tests = [str(value) for value in row.get("required_tests", [])]
        citations = [str(value) for value in row.get("citations", [])]
        evidence_paths = [str(value) for value in row.get("evidence_paths", [])]
        if not owner_id:
            errors.append("ownership row has an empty id")
        if status not in OWNER_STATUSES:
            errors.append(f"ownership row {owner_id} has invalid status `{status}`")
        else:
            counts[status] += 1
        if not re.fullmatch(r"UNIV-\d+", issue):
            errors.append(f"ownership row {owner_id} has invalid issue `{issue}`")
        if not tests:
            errors.append(f"ownership row {owner_id} has no required tests")
        if not citations:
            errors.append(f"ownership row {owner_id} has no upstream citations")
        for citation in citations:
            validate_citation(citation, repo_root, upstream_root, errors)
        if status in VERIFIED_STATUSES and manifest is not None:
            cited_sources = {
                match.group(1)
                for citation in citations
                if (match := re.fullmatch(r"cpp:(.+):\d+(?:-\d+)?", citation))
            }
            relevant_units = {
                str(source_units[source].get("id", "")): source_units[source]
                for source in cited_sources
                if source in source_units
            }
            if not relevant_units:
                errors.append(
                    f"ownership row {owner_id} has no cited campaign unit to receipt-gate promotion"
                )
            required_statuses = (
                {"verified"}
                if status == "verified"
                else {"fixed", "compiled", "verified"}
            )
            incomplete = sorted(
                unit_id
                for unit_id, unit in relevant_units.items()
                if unit.get("status") not in required_statuses
            )
            if incomplete:
                errors.append(
                    f"ownership row {owner_id} promotion outruns unit receipts: "
                    + ", ".join(incomplete)
                )
        if status in VERIFIED_STATUSES:
            if not evidence_paths:
                errors.append(
                    f"ownership row {owner_id} is {status} without concrete evidence paths"
                )
            for relative in evidence_paths:
                if not (repo_root / relative).is_file():
                    errors.append(
                        f"ownership row {owner_id} names missing evidence path {relative}"
                    )
                elif not git_tracked_file(repo_root, relative):
                    errors.append(
                        f"ownership row {owner_id} names untracked evidence path {relative}"
                    )
    return owners, counts


def load_authority_builder() -> Any:
    path = pathlib.Path(__file__).with_name("build_authority_ledgers.py")
    spec = importlib.util.spec_from_file_location("metal_port_authority_builder", path)
    if spec is None or spec.loader is None:
        raise ValueError(f"cannot load exhaustive authority builder {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def compare_exhaustive_authority_ledgers(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    expected: dict[pathlib.Path, str],
    errors: list[str],
    *,
    require_tracked: bool,
) -> None:
    expected_by_path = {path.as_posix(): content for path, content in expected.items()}
    for manifest_key, canonical_path in EXHAUSTIVE_AUTHORITY_LEDGERS.items():
        configured = str(manifest.get(manifest_key, ""))
        if configured != canonical_path.as_posix():
            errors.append(
                f"{manifest_key} must be {canonical_path.as_posix()}, got {configured or '<missing>'}"
            )
            continue
        path = repo_root / canonical_path
        if not path.is_file():
            errors.append(f"missing exhaustive authority ledger {canonical_path}")
            continue
        if require_tracked and not git_tracked_file(repo_root, canonical_path.as_posix()):
            errors.append(f"untracked exhaustive authority ledger {canonical_path}")
        actual = path.read_text(encoding="utf-8")
        authoritative = expected_by_path.get(canonical_path.as_posix())
        if authoritative is None:
            errors.append(f"authority builder did not render {canonical_path}")
        elif actual != authoritative:
            errors.append(
                f"exhaustive authority ledger drifted from pinned sources: {canonical_path}"
            )


def validate_exhaustive_authority_ledgers(
    manifest: dict[str, Any],
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    errors: list[str],
) -> None:
    builder_relative = "tools/metal-port/build_authority_ledgers.py"
    builder_path = repo_root / builder_relative
    if not builder_path.is_file():
        errors.append(f"missing exhaustive authority builder {builder_relative}")
        return
    if not git_tracked_file(repo_root, builder_relative):
        errors.append(f"untracked exhaustive authority builder {builder_relative}")
    try:
        builder = load_authority_builder()
        expected = builder.build(repo_root, upstream_root)
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        errors.append(f"cannot derive exhaustive Metal authority: {error}")
        return
    compare_exhaustive_authority_ledgers(
        manifest,
        repo_root,
        expected,
        errors,
        require_tracked=True,
    )


def validate_progress_promotion_claims(
    manifest: dict[str, Any],
    ownership: dict[str, Any],
    progress: dict[str, Any],
    errors: list[str],
) -> None:
    """Reject dashboard completion claims that outrun immutable evidence.

    The progress page is a projection of the campaign authorities. It must not
    become an independent source of truth that can label a phase or closeout
    gate green while the corresponding unit receipts, source rows, or owner
    rows remain incomplete.
    """

    units = list(manifest.get("translation_unit", []))
    sources = list(manifest.get("source", []))
    owners = list(ownership.get("owner", []))

    phase_receipts = {
        "translation": ("translation_receipt",),
        "source-review": ("source_review_receipt",),
        "ownership-review": ("ownership_review_receipt",),
        "correction": ("fix_receipt",),
        "compiler": ("compile_receipt",),
        "behavior": ("verification_receipt",),
        "review": TRANSLATION_RECEIPT_FIELDS,
        "promotion": TRANSLATION_RECEIPT_FIELDS,
    }

    def incomplete_receipts(fields: Iterable[str]) -> list[str]:
        incomplete: list[str] = []
        for unit in units:
            unit_id = str(unit.get("id", "<missing-id>"))
            for field in fields:
                expected = canonical_receipt_path(unit_id, field)
                if unit.get(field) != expected:
                    incomplete.append(f"{unit_id}.{field}")
        return incomplete

    phases = {
        str(phase.get("id", "")): phase
        for phase in progress.get("phase", [])
        if isinstance(phase, dict)
    }
    for phase_id, fields in phase_receipts.items():
        if phases.get(phase_id, {}).get("status") != "complete":
            continue
        incomplete = incomplete_receipts(fields)
        if incomplete:
            errors.append(
                f"{phase_id} phase cannot be complete with incomplete receipts: "
                + ", ".join(incomplete)
            )
        if phase_id == "ownership-review":
            incomplete_owners = [
                str(owner.get("id", "<missing-id>"))
                for owner in owners
                if owner.get("status") != "verified"
            ]
            if incomplete_owners:
                errors.append(
                    "ownership-review phase cannot be complete with incomplete owners: "
                    + ", ".join(incomplete_owners)
                )

    suites = {
        str(suite.get("id", "")): suite
        for suite in progress.get("suite", [])
        if isinstance(suite, dict)
    }

    def require_verified_campaign(suite_id: str, receipt_fields: Iterable[str]) -> None:
        if suites.get(suite_id, {}).get("status") != "green":
            return
        incomplete_sources = [
            str(source.get("upstream", "<missing-upstream>"))
            for source in sources
            if source.get("status") != "verified"
        ]
        incomplete_owners = [
            str(owner.get("id", "<missing-id>"))
            for owner in owners
            if owner.get("status") != "verified"
        ]
        incomplete = incomplete_receipts(receipt_fields)
        if incomplete_sources:
            errors.append(
                f"{suite_id} cannot be green with incomplete sources: "
                + ", ".join(incomplete_sources)
            )
        if incomplete_owners:
            errors.append(
                f"{suite_id} cannot be green with incomplete owners: "
                + ", ".join(incomplete_owners)
            )
        if incomplete:
            errors.append(
                f"{suite_id} cannot be green with incomplete receipts: "
                + ", ".join(incomplete)
            )
        if suite_id == "V9":
            incomplete_units = [
                str(unit.get("id", "<missing-id>"))
                for unit in units
                if unit.get("status") != "verified"
            ]
            if incomplete_units:
                errors.append(
                    "V9 cannot be green with incomplete translation units: "
                    + ", ".join(incomplete_units)
                )

    require_verified_campaign(
        "V1",
        (
            "translation_receipt",
            "source_review_receipt",
            "ownership_review_receipt",
            "fix_receipt",
        ),
    )
    require_verified_campaign("V9", TRANSLATION_RECEIPT_FIELDS)


def check(
    *,
    repo_root: pathlib.Path,
    upstream_root: pathlib.Path,
    manifest_path: pathlib.Path,
    ownership_path: pathlib.Path,
) -> str:
    manifest = read_toml(manifest_path)
    ownership = read_toml(ownership_path)
    errors: list[str] = []

    if manifest.get("version") != 1:
        errors.append("Metal source manifest version must be 1")
    if ownership.get("version") != 1:
        errors.append("Metal ownership inventory version must be 1")
    upstream_ref = str(manifest.get("upstream_ref", ""))
    if not re.fullmatch(r"[0-9a-f]{40}", upstream_ref):
        errors.append("Metal source manifest upstream_ref must be a full 40-hex SHA")
    else:
        actual_ref = git_head(upstream_root)
        if actual_ref != upstream_ref:
            errors.append(
                f"upstream checkout is {actual_ref}; Metal source manifest pins {upstream_ref}"
            )
    if ownership.get("upstream_ref") != upstream_ref:
        errors.append("Metal source manifest and ownership inventory pin different refs")

    guide = repo_root / str(manifest.get("porting_guide", ""))
    if not guide.is_file():
        errors.append(f"Metal porting guide does not exist: {guide}")

    source_counts = validate_source_rows(manifest, repo_root, upstream_root, errors)
    validate_mechanical_translation_workflow(manifest, errors)
    validate_metal_shader_source_inventory(
        manifest, repo_root, upstream_root, errors
    )
    validate_render_context_file_map(manifest, repo_root, upstream_root, errors)
    validate_render_context_field_map(manifest, repo_root, upstream_root, errors)
    validate_render_context_configuration_map(
        manifest, repo_root, upstream_root, errors
    )
    validate_render_context_dependency_map(
        manifest, repo_root, upstream_root, errors
    )
    validate_render_context_include_map(
        manifest, repo_root, upstream_root, errors
    )
    validate_exhaustive_authority_ledgers(
        manifest, repo_root, upstream_root, errors
    )
    divergence_rows = validate_divergence_ledger(
        manifest, repo_root, upstream_root, errors
    )
    validate_divergence_promotions(manifest, ownership, divergence_rows, errors)
    validate_translation_conventions(manifest, repo_root, upstream_root, errors)
    units = validate_translation_units(manifest, errors, repo_root, upstream_root)
    validate_manifest_targets_are_compiled_modules(manifest, repo_root, errors)
    validate_source_unit_promotion(manifest, repo_root, errors)
    validate_lifetime_ledger(manifest, repo_root, upstream_root, errors)
    validate_reference_provenance(manifest, repo_root, errors)
    expected_counts = {
        str(key): int(value)
        for key, value in dict(manifest.get("expected_status_counts", {})).items()
    }
    if dict(source_counts) != {key: value for key, value in expected_counts.items() if value}:
        errors.append(
            f"source status counts drifted: expected {expected_counts}, got {dict(source_counts)}"
        )

    owners, owner_counts = validate_owner_rows(
        ownership, repo_root, upstream_root, errors, manifest
    )
    expected_owner_counts = {
        str(key): int(value)
        for key, value in dict(ownership.get("expected_status_counts", {})).items()
    }
    if dict(owner_counts) != {
        key: value for key, value in expected_owner_counts.items() if value
    }:
        errors.append(
            "ownership status counts drifted: "
            f"expected {expected_owner_counts}, got {dict(owner_counts)}"
        )

    progress_path = repo_root / "docs/metal-renderer-progress.toml"
    if not progress_path.is_file():
        errors.append("missing Metal renderer progress authority docs/metal-renderer-progress.toml")
    else:
        try:
            progress = read_toml(progress_path)
        except (OSError, tomllib.TOMLDecodeError) as error:
            errors.append(f"cannot read Metal renderer progress authority: {error}")
        else:
            if progress.get("upstream_ref") != upstream_ref:
                errors.append("Metal renderer progress authority pins a different upstream ref")
            validate_progress_promotion_claims(manifest, ownership, progress, errors)

    if errors:
        raise CheckFailure("\n".join(f"- {error}" for error in errors))
    return (
        "Metal port campaign check passed: "
        f"sources={sum(source_counts.values())} "
        f"pending={source_counts['pending']} "
        f"in-progress={source_counts['in-progress']} "
        f"ported={source_counts['ported']} "
        f"verified={source_counts['verified']} owners={len(owners)} "
        f"translation-units={len(units)}"
    )


def main() -> int:
    global REPLAY_RECEIPT_COMMANDS
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument("--upstream-root", type=pathlib.Path, required=True)
    parser.add_argument("--manifest", type=pathlib.Path, required=True)
    parser.add_argument("--ownership", type=pathlib.Path, required=True)
    parser.add_argument(
        "--replay-receipt-commands",
        action="store_true",
        help="execute every receipt command and verify its declared result count",
    )
    args = parser.parse_args()
    REPLAY_RECEIPT_COMMANDS = args.replay_receipt_commands
    try:
        print(
            check(
                repo_root=args.repo_root.resolve(),
                upstream_root=args.upstream_root.resolve(),
                manifest_path=args.manifest.resolve(),
                ownership_path=args.ownership.resolve(),
            )
        )
    except CheckFailure as error:
        print(f"Metal port campaign check failed:\n{error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
