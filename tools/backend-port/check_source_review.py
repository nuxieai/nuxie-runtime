#!/usr/bin/env python3
"""Validate the global backend source-semantics review evidence."""

from __future__ import annotations

import argparse
import csv
import hashlib
import re
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable


TRANSLATED_DISPOSITIONS = {"translate", "shared-authority", "dependency-authority"}
BACKEND_CAMPAIGNS = {"vulkan", "webgpu", "webgl2"}
EXPECTED_UPSTREAM_REF = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"
EXPECTED_WORKSPACE_BASE_REF = "5967e72706cf63702ecbc84a982e4172b8d6245f"
EXPECTED_ADMISSION_BASE_REF = "aa25e76acdbe0ad0f4099f8a360937bd74d856f9"
EXPECTED_SOURCE_REVIEW_PATHS = {
    "source_review_plan": "docs/backend-port-source-review-plan.toml",
    "source_review_schema": "docs/backend-port-source-review-schema.md",
    "source_review_receipt_directory": "docs/backend-port-source-reviews",
    "source_review_support_inventory": "docs/backend-port-source-review-support.tsv",
}
EXPECTED_OWNERSHIP_REVIEW_PATHS = {
    "ownership_review_plan": "docs/backend-port-ownership-review-plan.toml",
    "ownership_review_schema": "docs/backend-port-ownership-review-schema.md",
    "ownership_review_receipt_directory": "docs/backend-port-ownership-reviews",
}
OWNERSHIP_COMPLETION_PIN_KEYS = {
    "ownership_review_barrier_ref",
    "ownership_review_receipt_tree_sha256",
    "ownership_review_receipt_logical_lines",
    "ownership_review_receipt_bytes",
    "ownership_review_finding_total",
    "ownership_review_p0_findings",
    "ownership_review_p1_findings",
    "ownership_review_p2_findings",
    "ownership_review_p3_findings",
    "ownership_review_finding_id_sha256",
}
EXPECTED_MANIFEST_KEYS = {
    "schema_version", "upstream_ref", "active_queue", "preparation_status",
    "ignored_skills", "translation_receipt_directory", "source_review_plan",
    "source_review_schema", "source_review_receipt_directory",
    "source_review_support_inventory", "source_review_status",
    "ownership_review_plan", "ownership_review_schema",
    "ownership_review_receipt_directory", "ownership_review_launch_ref",
    "ownership_review_status", "queue_order",
    "shared_generic_authority", "shared_ownership_authority", "source_inventory",
    "ownership_inventory", "dependency_inventory", "ownership_unit_order",
    "toolchain_authority", "generated_artifact_inventory", "configuration_inventory",
    "field_profiles", "field_inventory", "lifecycle_inventory", "oracle_contract",
    "legacy_wgpu_inventory", "owner_contracts", "repeatability_inventory",
    "cutover_contract", "denominator", "backend", "shared_source_set",
}
EXPECTED_PLAN_SHA256 = "015cc3763be315e3bcfe0fe7793f25d45c9ccea90e0ad20a2ba6b0f154da4ddb"
EXPECTED_SUPPORT_INVENTORY_SHA256 = \
    "4b26bd58f1ff7da06267c3d485c91db13adea7a1995c6051d337cd7058fccf3e"
EXPECTED_SCHEMA_SHA256 = "f5d449bd67fac1de88be554cab042321146fe7a5dcbeb9f46495b077cb09d748"
CLASSIFICATION_BUILD_PREDICATES = {
    "renderer/src/gl/load_gles_extensions.cpp": (408, 414, "android-only-not-emscripten"),
    "renderer/src/gl/pls_impl_ext_native.cpp": (408, 414, "android-only-not-emscripten"),
    "renderer/src/gl/pls_impl_rw_texture.cpp": (
        397, 405, "windows-or-macosx-or-linux-only-not-emscripten",
    ),
    "renderer/src/ore/gl/ore_buffer_gl.mm": (465, 470, "macosx-only-not-emscripten"),
    "renderer/src/ore/gl/ore_context_gl.mm": (465, 470, "macosx-only-not-emscripten"),
    "renderer/src/ore/gl/ore_pipeline_gl.mm": (465, 470, "macosx-only-not-emscripten"),
    "renderer/src/ore/gl/ore_render_pass_gl.mm": (465, 470, "macosx-only-not-emscripten"),
    "renderer/src/ore/gl/ore_sampler_gl.mm": (465, 470, "macosx-only-not-emscripten"),
    "renderer/src/ore/gl/ore_shader_module_gl.mm": (
        465, 470, "macosx-only-not-emscripten",
    ),
    "renderer/src/ore/gl/ore_texture_gl.mm": (465, 470, "macosx-only-not-emscripten"),
}
CLASSIFICATION_BUILD_SELECTIONS = {
    "renderer/src/gl/pls_impl_webgl.cpp": (613, 616, "emscripten"),
    **{
        f"renderer/src/ore/gl/ore_{name}_gl.cpp": (
            537, 540,
            "emscripten-with-rive-canvas-not-with-wagyu-not-no-gl-not-for-unreal",
        )
        for name in (
            "buffer", "context", "pipeline", "render_pass", "sampler",
            "shader_module", "texture",
        )
    },
}
BROWSER_FACTORY_SOURCES = {
    "vulkan": (
        "renderer/src/vulkan/render_context_vulkan_impl.cpp",
        "RenderContextVulkanImpl",
    ),
    "webgl2": (
        "renderer/src/gl/render_context_gl_impl.cpp",
        "RenderContextGLImpl",
    ),
    "webgpu": (
        "renderer/src/webgpu/render_context_webgpu_impl.cpp",
        "RenderContextWebGPUImpl",
    ),
}
BROWSER_FACTORY_HEADERS = {
    "vulkan": "renderer/include/rive/renderer/vulkan/render_context_vulkan_impl.hpp",
    "webgl2": "renderer/include/rive/renderer/gl/render_context_gl_impl.hpp",
    "webgpu": "renderer/include/rive/renderer/webgpu/render_context_webgpu_impl.hpp",
}
BROWSER_COMPONENT_SOURCE_PATHS = {
    "renderer/src/vulkan/render_context_vulkan_impl.cpp",
    "renderer/include/rive/renderer/vulkan/render_context_vulkan_impl.hpp",
    "renderer/include/rive/renderer/gl/gles3.hpp",
    "renderer/src/gl/gl_utils.cpp",
    "renderer/src/gl/render_context_gl_impl.cpp",
    "renderer/include/rive/renderer/gl/render_context_gl_impl.hpp",
    "renderer/src/gl/pls_impl_webgl.cpp",
    "renderer/src/webgpu/wagyu-port/include/webgpu/webgpu.h",
    "renderer/src/webgpu/wagyu-port/include/webgpu/webgpu_wagyu.h",
    "renderer/src/webgpu/render_context_webgpu_impl.cpp",
    "renderer/include/rive/renderer/webgpu/render_context_webgpu_impl.hpp",
    "renderer/src/webgpu/wagyu-port/src/library_webgpu_stubs.js",
    "renderer/src/webgpu/wagyu-port/src/library_webgpu_wagyu_stubs.js",
    "renderer/src/webgpu/wagyu-port/webgpu-port.py",
}
EXPECTED_BROWSER_COMPONENT_IDS = {
    "component-080", "component-083", "component-090", "component-093",
    "component-097", "component-098", "component-109", "component-112",
    "component-113", "component-114",
}
EXPECTED_BROWSER_SUPPORT_PATHS = {
    "Cargo.lock", "Cargo.toml", "crates/nuxie-renderer/Cargo.toml",
    "crates/nuxie-renderer/src/draw.rs", "crates/nuxie-renderer/src/lib.rs",
    "crates/nuxie-renderer/src/mechanical_port.rs",
}
BROWSER_FEATURES = (
    "default", "native-vulkan-experimental", "native-webgpu-experimental", "ore-gl",
)
BROWSER_MODULE_FEATURES = {
    "webgl2": "native-webgpu-experimental",
    "webgpu": "native-webgpu-experimental",
    "vulkan": "native-vulkan-experimental",
}
WEBGL_EM_JS_SEMANTICS = {
    "webgl_shader_source": "ShaderSourceBypassingEmscripten",
    "enable_WEBGL_shader_pixel_local_storage_coherent":
        "enableWebGLShaderPixelLocalStorageCoherent",
    "framebufferTexturePixelLocalStorageWEBGL":
        "FramebufferTexturePixelLocalStorageANGLE",
    "framebufferPixelLocalClearValuefvWEBGL":
        "FramebufferPixelLocalClearValuefvANGLE",
    "beginPixelLocalStorageWEBGL": "BeginPixelLocalStorageANGLE",
    "endPixelLocalStorageWEBGL": "EndPixelLocalStorageANGLE",
    "getFramebufferPixelLocalStorageParameterivWEBGL":
        "getFramebufferPixelLocalStorageParameter",
    "enable_WEBGL_provoking_vertex": "enableWebGLProvokingVertex",
    "provokingVertexWEBGL": "ProvokingVertex",
}
WEBGPU_JS_STUB_PATHS = (
    "renderer/src/webgpu/wagyu-port/src/library_webgpu_stubs.js",
    "renderer/src/webgpu/wagyu-port/src/library_webgpu_wagyu_stubs.js",
)
WEBGPU_HEADER_PATHS = (
    "renderer/src/webgpu/wagyu-port/include/webgpu/webgpu.h",
    "renderer/src/webgpu/wagyu-port/include/webgpu/webgpu_wagyu.h",
)
WEBGPU_PORT_PATH = "renderer/src/webgpu/wagyu-port/webgpu-port.py"
EXPECTED_COVERAGE = [
    "owned-source-lines",
    "translated-target-lines",
    "declarations",
    "conditionals",
    "include-owners",
    "source-semantics",
    "pinned-build-exclusions",
]
EXPECTED_OVERLAY_COVERAGE = [*EXPECTED_COVERAGE, "cross-backend-overlays"]
EXPECTED_SEVERITIES = ["P0", "P1", "P2", "P3"]
EXPECTED_OVERLAY_IDS = [
    "shared-authority-consumers",
    "webgpu-to-webgl2-load-store",
    "generated-authority",
    "webgpu-abi",
    "shared-ore-contracts",
    "shared-renderer-contracts",
    "vulkan-vma-adaptation",
    "backend-identity-and-browser-bridges",
    "classification-boundary",
]
EXPECTED_CUTOVER_CONTRACT = {
    "ported_renderers": ["vulkan", "webgpu", "webgl2"],
    "editor_browser_renderers": ["webgpu", "webgl2"],
    "editor_selection": "explicit-user-selected-no-automatic-fallback",
    "legacy_wgpu": "retain-until-each-ported-renderer-independently-passes-frozen-closeout-then-delete",
}
EXPECTED_RULES = {
    "authority": "Pinned source, frozen build/configuration/generated authority, and exclusive source-owner mapping are primary. Other backends and existing tests are diagnostic only.",
    "read_only": "Review contexts do not edit source, target, receipts, or authority and do not use compiler diagnostics, fixtures, or features to choose review work.",
    "complete_lines": "Every owned source and every translated target receives one exact full-file 1-N citation and current SHA-256 binding.",
    "complete_semantics": "Review declarations, executable bodies, conditionals, include/import owners, configuration branches, generated inputs/outputs, failure order, and source-visible behavior.",
    "exclusions": "Every nontranslated row is independently revalidated against the pinned build graph; an exclusion has no translated target evidence and cannot hide a semantic owner compiled by the selected backend.",
    "no_phase_jump": "A contract-shaped executor seam may defer physical platform execution to its declared later gate, but disconnected types, inert owners, stubs, placeholders, success-returning no-ops, and missing source branches are findings.",
    "finding_stability": "Every finding receives a stable ID, severity, summary, and exact source/target citations; no finding is corrected until both global review passes finish.",
    "skill_exclusion": "Actively ignore and do not use the implement or tdd skills.",
    "wave_order": "Review order_group 0 through 6 in order. Within one wave, parallelize only by complete component_id; never split an SCC. No backend-specific completion can advance the global gate.",
    "component_atomicity": "One canonical receipt covers the complete source and target union of one frozen SCC component; a component is never issued, cited, or closed as separate ownership units.",
    "overlay_gate": "After all seven component waves and support review close structurally, one explicit overlay receipt covers all nine derived cross-seam authorities before the global source pass closes.",
    "product_scope": "Port Vulkan, WebGPU, and WebGL2 exactly; expose WebGPU and WebGL2 as explicit editor choices; retain legacy Rust-WGPU until all three ports independently pass frozen closeout, then delete it.",
}
EXPECTED_CATEGORY_ORDER = [
    "translated_target",
    "source_snapshot",
    "dependency_tree_member",
    "dependency_file",
    "source_review_support",
    "campaign_documentation",
    "campaign_tooling",
    "ownership_only_evidence",
    "explicit_deletion",
]
FROZEN_CAMPAIGN_PATH_KEYS = (
    "shared_generic_authority",
    "shared_ownership_authority",
    "source_inventory",
    "ownership_inventory",
    "dependency_inventory",
    "ownership_unit_order",
    "toolchain_authority",
    "generated_artifact_inventory",
    "configuration_inventory",
    "field_profiles",
    "field_inventory",
    "lifecycle_inventory",
    "oracle_contract",
    "legacy_wgpu_inventory",
    "owner_contracts",
    "repeatability_inventory",
)
FROZEN_MANIFEST_KEYS = {
    "upstream_ref",
    "preparation_status",
    "ignored_skills",
    "translation_receipt_directory",
    "queue_order",
    "denominator",
    "backend",
    "shared_source_set",
    *FROZEN_CAMPAIGN_PATH_KEYS,
}
PLAN_KEYS = {
    "schema_version", "upstream_ref", "workspace_base_ref", "review_kind",
    "review_mode", "receipt_directory", "source_denominator",
    "translated_source_denominator", "excluded_source_denominator",
    "logical_source_line_denominator", "translated_logical_source_line_denominator",
    "excluded_logical_source_line_denominator", "source_byte_denominator",
    "translated_source_byte_denominator", "excluded_source_byte_denominator",
    "unit_denominator", "translated_unit_denominator", "excluded_only_unit_denominator",
    "component_denominator", "translated_component_denominator",
    "excluded_only_component_denominator", "component_receipt_denominator",
    "semantic_dependency_rows", "semantic_configuration_rows", "generated_artifacts",
    "retained_generated_artifacts", "ephemeral_generated_artifacts",
    "retained_generated_logical_lines", "retained_generated_bytes",
    "pinned_external_dependency_files", "pinned_external_dependency_logical_lines",
    "pinned_external_dependency_bytes", "semantic_generated_owner_edges",
    "translation_snapshot_denominator",
    "translation_dependency_file_denominator", "translation_dependency_tree_denominator",
    "translation_dependency_tree_file_denominator", "support_artifact_denominator",
    "support_artifact_logical_line_denominator", "overlay_denominator", "coverage",
    "severity_order", "finding_id_rule", "rules", "changed_byte_closure", "wave", "overlay",
}
CLOSURE_KEYS = {
    "base_ref", "head_ref", "diff_filter", "category_order", "campaign_tooling_paths",
    "ownership_only_paths", "explicit_deleted_paths", "changed_path_denominator",
    *EXPECTED_CATEGORY_ORDER,
}
WAVE_KEYS = {
    "id", "order_group", "source_count", "translated_source_count", "excluded_source_count",
    "logical_source_lines", "translated_logical_source_lines", "excluded_logical_source_lines",
    "unit_count", "translated_unit_count", "excluded_only_unit_count", "component_count",
    "translated_component_count", "excluded_only_component_count",
}
OVERLAY_PLAN_KEYS = {
    "id", "rule", "component_count", "support_count", "dependency_record_count",
    "semantic_dependency_record_count", "configuration_record_count",
    "build_predicate_record_count", "generated_record_count",
    "browser_bridge_record_count",
    "physical_generated_record_count", "external_record_count", "artifact_record_count",
    "tree_count", "excluded_source_count", "authority_record_count", "authority_sha256",
}
COMPONENT_RECEIPT_KEYS = {
    "schema_version", "component_id", "units", "receipt_kind", "upstream_ref",
    "workspace_base_ref", "role", "review_run_id", "review_wave", "coverage",
    "sources", "targets", "findings", "open_findings",
}
SUPPORT_RECEIPT_KEYS = {
    "schema_version", "receipt_kind", "upstream_ref", "workspace_base_ref", "role",
    "review_run_id", "review_wave", "coverage", "artifacts", "findings", "open_findings",
}
OVERLAY_RECEIPT_KEYS = {
    "schema_version", "receipt_kind", "upstream_ref", "workspace_base_ref", "role",
    "review_run_id", "review_wave", "coverage", "overlays", "findings", "open_findings",
}
SOURCE_RECORD_KEYS = {"path", "sha256", "citation", "disposition"}
TARGET_RECORD_KEYS = {"path", "sha256", "citation"}
SUPPORT_RECORD_KEYS = {
    "path", "sha256", "logical_lines", "citation", "artifact_role", "review_overlay",
    "source_authority", "disposition",
}
OVERLAY_RECORD_KEYS = {
    "id", "authority_record_count", "authority_sha256", "component_ids",
    "support_paths", "tree_bindings", "external_bindings", "generated_bindings",
    "authority_keys", "component_receipts", "support_receipts", "attestation",
}
TREE_BINDING_KEYS = {"path", "tree_sha256"}
UPSTREAM_FILE_BINDING_KEYS = {"path", "sha256", "logical_lines"}
RECEIPT_BINDING_KEYS = {"id", "path", "sha256"}
FINDING_KEYS = {"id", "severity", "summary", "citations"}
OVERLAY_FINDING_KEYS = {"id", "overlay_id", "severity", "summary", "citations"}


class ReviewError(ValueError):
    """An evidence or authority contract failed."""


@dataclass(frozen=True)
class Owner:
    campaign: str
    source_path: str
    source_sha256: str
    unit: str
    disposition: str
    target_path: str

    @property
    def translated(self) -> bool:
        return self.disposition in TRANSLATED_DISPOSITIONS


@dataclass(frozen=True)
class Unit:
    unit: str
    campaign: str
    order_group: int
    component_id: str
    source_count: int
    dependency_units: tuple[str, ...]

    @property
    def review_wave(self) -> str:
        return f"g{self.order_group}"


@dataclass(frozen=True)
class Component:
    component_id: str
    order_group: int
    units: tuple[str, ...]

    @property
    def review_wave(self) -> str:
        return f"g{self.order_group}"


@dataclass(frozen=True)
class Translation:
    source_path: str
    unit: str
    target_path: str
    source_sha256: str
    target_sha256: str
    snapshot_path: str
    snapshot_sha256: str


@dataclass(frozen=True)
class FileArtifact:
    path: str
    sha256: str


@dataclass(frozen=True)
class TreeArtifact:
    path: str
    tree_sha256: str
    members: tuple[str, ...]


@dataclass(frozen=True)
class SupportArtifact:
    path: str
    sha256: str
    logical_lines: int
    artifact_role: str
    review_overlay: str
    source_authority: str
    disposition: str


@dataclass(frozen=True)
class UpstreamFileAuthority:
    path: str
    sha256: str
    logical_lines: int
    byte_count: int


@dataclass(frozen=True)
class OverlayExpectation:
    id: str
    component_ids: tuple[str, ...]
    support_paths: tuple[str, ...]
    dependency_record_count: int
    semantic_dependency_record_count: int
    configuration_record_count: int
    build_predicate_record_count: int
    generated_record_count: int
    browser_bridge_record_count: int
    generated_paths: tuple[str, ...]
    external_paths: tuple[str, ...]
    artifact_paths: tuple[str, ...]
    tree_bindings: tuple[tuple[str, str], ...]
    excluded_source_count: int
    authority_keys: tuple[str, ...]
    authority_record_count: int
    authority_sha256: str


@dataclass
class Authority:
    repo: Path
    upstream: Path
    manifest: dict[str, Any]
    plan: dict[str, Any]
    owners: dict[str, Owner]
    units: dict[str, Unit]
    components: dict[str, Component]
    owners_by_component: dict[str, list[Owner]]
    translations: dict[str, Translation]
    file_artifacts: dict[str, FileArtifact]
    tree_artifacts: dict[str, TreeArtifact]
    support_artifacts: dict[str, SupportArtifact]
    generated_outputs: dict[str, UpstreamFileAuthority]
    external_authorities: dict[str, UpstreamFileAuthority]
    overlays: dict[str, OverlayExpectation]
    receipt_directory: Path
    source_reviewer_role: str
    changed_path_count: int


@dataclass(frozen=True)
class ComponentReceiptResult:
    component_id: str
    review_wave: str
    review_run_id: str
    unit_ids: tuple[str, ...]
    source_paths: frozenset[str]
    target_paths: frozenset[str]
    finding_ids: tuple[str, ...]
    open_findings: int


@dataclass(frozen=True)
class SupportReceiptResult:
    review_run_id: str
    artifact_paths: frozenset[str]
    finding_ids: tuple[str, ...]
    open_findings: int


@dataclass(frozen=True)
class OverlayReceiptResult:
    review_run_id: str
    overlay_ids: tuple[str, ...]
    finding_ids: tuple[str, ...]
    open_findings: int


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--upstream-root", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--receipt", type=Path)
    mode.add_argument("--admission", action="store_true")
    return parser.parse_args()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ReviewError(message)


def require_exact_keys(value: dict[str, Any], expected: set[str], label: str) -> None:
    missing = sorted(expected - set(value))
    extra = sorted(set(value) - expected)
    require(not missing, f"{label} is missing keys: {', '.join(missing)}")
    require(not extra, f"{label} invents keys: {', '.join(extra)}")


def read_tsv(path: Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as source:
        return list(csv.DictReader(source, delimiter="\t"))


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def logical_lines(path: Path) -> int:
    return len(path.read_bytes().splitlines())


def tree_digest(path: Path) -> str:
    hasher = hashlib.sha256()
    for child in sorted(candidate for candidate in path.rglob("*") if candidate.is_file()):
        hasher.update(child.relative_to(path).as_posix().encode())
        hasher.update(b"\0")
        hasher.update(child.read_bytes())
        hasher.update(b"\0")
    return hasher.hexdigest()


def canonical_row(prefix: str, row: dict[str, str]) -> str:
    fields = "\x1f".join(f"{key}={row[key]}" for key in sorted(row))
    return f"{prefix}:{fields}"


def canonical_digest(records: Iterable[str]) -> tuple[int, str]:
    ordered = sorted(set(records))
    return len(ordered), hashlib.sha256("\n".join(ordered).encode()).hexdigest()


def run_git(root: Path, *arguments: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", "-C", str(root), *arguments], capture_output=True, text=True)


def git_head(root: Path) -> str:
    result = run_git(root, "rev-parse", "HEAD")
    require(result.returncode == 0, f"cannot resolve Git HEAD for {root}")
    return result.stdout.strip()


def git_tracked(repo: Path, path: Path) -> bool:
    try:
        relative = path.resolve().relative_to(repo).as_posix()
    except ValueError:
        return False
    return run_git(repo, "ls-files", "--error-unmatch", "--", relative).returncode == 0


def require_revision(repo: Path, revision: str, label: str) -> None:
    require(bool(re.fullmatch(r"[0-9a-f]{40}", revision)), f"{label} must be a full Git revision")
    require(run_git(repo, "cat-file", "-e", f"{revision}^{{commit}}").returncode == 0,
            f"{label} is not a repository commit")


def require_ancestor(repo: Path, revision: str, label: str) -> None:
    require_revision(repo, revision, label)
    require(run_git(repo, "merge-base", "--is-ancestor", revision, "HEAD").returncode == 0,
            f"{label} is not an ancestor of current HEAD")


def require_frozen_scopes(repo: Path, revision: str, paths: Iterable[str]) -> None:
    """Require tracked review inputs to equal their bytes at the launch checkpoint."""
    scopes = sorted(set(paths))
    require(scopes, "frozen review-byte scope is empty")
    for scope in scopes:
        path = repo_path(repo, scope, "frozen review-byte scope")
        require(path.exists(), f"frozen review-byte scope is missing: {scope}")
        require(run_git(repo, "ls-files", "--error-unmatch", "--", scope).returncode == 0,
                f"frozen review-byte scope is not tracked: {scope}")
        require(run_git(repo, "cat-file", "-e", f"{revision}:{scope}").returncode == 0,
                f"frozen review-byte scope was absent at workspace_base_ref: {scope}")
    # Chunk the argument list so the complete tree remains portable across host limits.
    for offset in range(0, len(scopes), 100):
        chunk = scopes[offset:offset + 100]
        result = run_git(repo, "diff", "--quiet", "--no-ext-diff", revision, "--", *chunk)
        require(result.returncode == 0,
                "review bytes drifted from workspace_base_ref in: "
                + ", ".join(chunk))


def load_toml_at_revision(repo: Path, revision: str, relative: str,
                          label: str) -> dict[str, Any]:
    result = subprocess.run(
        ["git", "-C", str(repo), "show", f"{revision}:{relative}"],
        capture_output=True,
    )
    require(result.returncode == 0, f"missing {label} at {revision}: {relative}")
    value = tomllib.loads(result.stdout.decode("utf-8"))
    require(isinstance(value, dict), f"{label} at {revision} must be a TOML table")
    return value


def repo_path(repo: Path, relative: str, label: str) -> Path:
    require(bool(relative) and not Path(relative).is_absolute(), f"{label} must be repository-relative")
    path = (repo / relative).resolve()
    try:
        path.relative_to(repo)
    except ValueError as error:
        raise ReviewError(f"{label} escapes the repository: {relative}") from error
    return path


def upstream_path(upstream: Path, relative: str, label: str) -> Path:
    require(bool(relative) and not Path(relative).is_absolute(), f"{label} must be upstream-relative")
    path = (upstream / relative).resolve()
    try:
        path.relative_to(upstream)
    except ValueError as error:
        raise ReviewError(f"{label} escapes the upstream tree: {relative}") from error
    return path


def load_toml(path: Path, label: str) -> dict[str, Any]:
    require(path.is_file(), f"missing {label}: {path}")
    with path.open("rb") as source:
        value = tomllib.load(source)
    require(isinstance(value, dict), f"{label} must be a TOML table")
    return value


def validate_queue(manifest: dict[str, Any]) -> None:
    queue = manifest.get("active_queue")
    order = manifest.get("queue_order")
    require(isinstance(order, list) and all(isinstance(item, str) for item in order), "invalid queue_order")
    require(len(order) == len(set(order)), "queue_order contains duplicates")
    require("source-review" in order and queue in order, "source-review queue or active queue is unknown")
    require(order.index(str(queue)) >= order.index("source-review"), "active queue precedes source-review")


def validate_plan_shape(plan: dict[str, Any], manifest: dict[str, Any], repo: Path) -> None:
    require_exact_keys(plan, PLAN_KEYS, "source-review plan")
    require(plan["schema_version"] == 1, "invalid source-review plan schema")
    require(plan["upstream_ref"] == EXPECTED_UPSTREAM_REF,
            "source-review plan is not pinned to the expected upstream revision")
    require(plan["workspace_base_ref"] == EXPECTED_WORKSPACE_BASE_REF,
            "source-review plan is not pinned to the expected workspace base")
    require(plan["upstream_ref"] == manifest["upstream_ref"], "source-review plan upstream drift")
    require(plan["review_kind"] == "global-source-semantics", "source-review plan kind drift")
    require(plan["review_mode"] == "independent-read-only-scc-waves", "source-review mode drift")
    require(plan["coverage"] == EXPECTED_COVERAGE, "source-review coverage contract drift")
    require(plan["severity_order"] == EXPECTED_SEVERITIES, "source-review severity contract drift")
    require(plan["finding_id_rule"] == "SR-C<component number>-<two-digit nonzero ordinal>",
            "source-review finding ID rule drift")
    require(plan["rules"] == EXPECTED_RULES, "source-review rules drift")
    require_ancestor(repo, str(plan["workspace_base_ref"]), "plan workspace_base_ref")
    require(plan["receipt_directory"] == manifest["source_review_receipt_directory"],
            "source-review receipt directory drift")


def load_role(repo: Path, manifest: dict[str, Any]) -> str:
    shared = load_toml(repo_path(repo, str(manifest["shared_generic_authority"]),
                                 "shared generic authority"), "shared generic authority")
    role = shared.get("mechanical_translation_workflow", {}).get("source_reviewer_role")
    require(isinstance(role, str) and role, "shared authority has no source reviewer role")
    return role


def validate_waves(plan: dict[str, Any], owners_by_component: dict[str, list[Owner]],
                   units: dict[str, Unit], components: dict[str, Component], upstream: Path) -> None:
    waves = plan["wave"]
    require(isinstance(waves, list) and len(waves) == 7, "source-review plan must define seven waves")
    require([wave.get("id") for wave in waves] == [f"g{i}" for i in range(7)],
            "source-review wave order drift")
    require([wave.get("order_group") for wave in waves] == list(range(7)),
            "source-review order_group drift")
    for wave in waves:
        require(isinstance(wave, dict), "source-review wave must be a table")
        wave_id = str(wave["id"])
        require_exact_keys(wave, WAVE_KEYS, f"wave {wave_id}")
        members = [component for component in components.values()
                   if component.order_group == wave["order_group"]]
        member_units = [units[unit_id] for component in members for unit_id in component.units]
        member_owners = [owner for component in members
                         for owner in owners_by_component[component.component_id]]
        translated = [owner for owner in member_owners if owner.translated]
        excluded = [owner for owner in member_owners if not owner.translated]
        translated_unit_ids = {owner.unit for owner in translated}
        translated_units = [unit for unit in member_units if unit.unit in translated_unit_ids]
        excluded_units = [unit for unit in member_units if unit.unit not in translated_unit_ids]
        translated_components = [component for component in members
                                 if any(owner.translated for owner in owners_by_component[component.component_id])]
        excluded_components = [component for component in members if component not in translated_components]
        derived = {
            "source_count": len(member_owners),
            "translated_source_count": len(translated),
            "excluded_source_count": len(excluded),
            "logical_source_lines": sum(logical_lines(upstream_path(upstream, owner.source_path,
                                                                      "wave source"))
                                        for owner in member_owners),
            "translated_logical_source_lines": sum(logical_lines(upstream_path(upstream, owner.source_path,
                                                                                 "wave source"))
                                                   for owner in translated),
            "excluded_logical_source_lines": sum(logical_lines(upstream_path(upstream, owner.source_path,
                                                                               "wave source"))
                                                 for owner in excluded),
            "unit_count": len(member_units),
            "translated_unit_count": len(translated_units),
            "excluded_only_unit_count": len(excluded_units),
            "component_count": len(members),
            "translated_component_count": len(translated_components),
            "excluded_only_component_count": len(excluded_components),
        }
        for key, value in derived.items():
            require(wave[key] == value, f"wave {wave_id} {key} drift: expected {value}")


def load_translation_closure(
    repo: Path,
    upstream: Path,
    manifest: dict[str, Any],
    plan: dict[str, Any],
    owners: dict[str, Owner],
    units: dict[str, Unit],
) -> tuple[dict[str, Translation], dict[str, FileArtifact], dict[str, TreeArtifact]]:
    receipt_directory = repo_path(repo, str(manifest["translation_receipt_directory"]),
                                  "translation receipt directory")
    paths = sorted(receipt_directory.glob("*.translation.toml"))
    translations: dict[str, Translation] = {}
    target_owners: dict[str, str] = {}
    file_artifacts: dict[str, FileArtifact] = {}
    tree_artifacts: dict[str, TreeArtifact] = {}
    for path in paths:
        require(git_tracked(repo, path), f"translation receipt is not tracked: {path}")
        receipt = load_toml(path, "translation receipt")
        source_path = str(receipt.get("source_path", ""))
        require(source_path in owners, f"translation receipt invents source: {path}")
        owner = owners[source_path]
        require(owner.translated, f"nontranslated source has a translation receipt: {source_path}")
        require(source_path not in translations, f"duplicate translation receipt: {source_path}")
        require(receipt.get("schema_version") == 1, f"invalid translation receipt schema: {path}")
        require(receipt.get("campaign") == owner.campaign, f"translation campaign drift: {source_path}")
        require(receipt.get("ownership_unit") == owner.unit, f"translation unit drift: {source_path}")
        require(receipt.get("translation_kind") == "complete-source-owner",
                f"partial translation receipt: {source_path}")
        require(receipt.get("source_sha256") == owner.source_sha256,
                f"translation source hash drift: {source_path}")
        source = upstream_path(upstream, source_path, "translated source")
        require(sha256(source) == owner.source_sha256,
                f"translated source bytes drift: {source_path}")
        require(receipt.get("source_lines") == logical_lines(source),
                f"translation source-line drift: {source_path}")
        require(receipt.get("source_bytes") == source.stat().st_size,
                f"translation source-byte drift: {source_path}")
        require(receipt.get("target_path") == owner.target_path,
                f"translation target drift: {source_path}")
        target_path = str(receipt.get("target_path", ""))
        require(target_path not in target_owners, f"translation target overlap: {target_path}")
        target = repo_path(repo, target_path, "translated target")
        require(target.is_file(), f"translated target is missing: {target_path}")
        target_sha = str(receipt.get("target_sha256", ""))
        require(sha256(target) == target_sha, f"translated target hash drift: {target_path}")
        snapshot_path = str(receipt.get("source_snapshot_path", ""))
        snapshot = repo_path(repo, snapshot_path, "translated source snapshot")
        require(snapshot.is_file(), f"translated source snapshot is missing: {snapshot_path}")
        require(git_tracked(repo, snapshot), f"translated source snapshot is not tracked: {snapshot_path}")
        snapshot_sha = str(receipt.get("source_snapshot_sha256", ""))
        require(sha256(snapshot) == snapshot_sha == owner.source_sha256,
                f"translated source snapshot drift: {snapshot_path}")
        dependency_units = receipt.get("dependency_units")
        require(isinstance(dependency_units, list),
                f"translation dependencies are not an array: {source_path}")
        require(set(dependency_units) == set(units[owner.unit].dependency_units),
                f"translation dependency drift: {source_path}")
        artifacts = receipt.get("dependency_artifacts", [])
        require(isinstance(artifacts, list),
                f"translation dependency artifacts are not an array: {source_path}")
        for index, artifact in enumerate(artifacts):
            require(isinstance(artifact, dict),
                    f"invalid translation dependency artifact {source_path}:{index}")
            artifact_path = str(artifact.get("path", ""))
            absolute = repo_path(repo, artifact_path, "translation dependency artifact")
            if "tree_sha256" in artifact:
                require_exact_keys(artifact, {"path", "tree_sha256"},
                                   f"translation dependency tree {source_path}:{index}")
                require(absolute.is_dir(), f"translation dependency tree is missing: {artifact_path}")
                tree_sha = str(artifact["tree_sha256"])
                require(tree_digest(absolute) == tree_sha,
                        f"translation dependency tree drift: {artifact_path}")
                members = tuple(sorted(child.relative_to(repo).as_posix()
                                       for child in absolute.rglob("*") if child.is_file()))
                require(members, f"translation dependency tree is empty: {artifact_path}")
                require(all(git_tracked(repo, repo / member) for member in members),
                        f"translation dependency tree has untracked members: {artifact_path}")
                tree = TreeArtifact(artifact_path, tree_sha, members)
                require(artifact_path not in tree_artifacts or tree_artifacts[artifact_path] == tree,
                        f"conflicting translation dependency tree: {artifact_path}")
                tree_artifacts[artifact_path] = tree
            else:
                require_exact_keys(artifact, {"path", "sha256"},
                                   f"translation dependency file {source_path}:{index}")
                require(absolute.is_file(), f"translation dependency file is missing: {artifact_path}")
                require(git_tracked(repo, absolute),
                        f"translation dependency file is not tracked: {artifact_path}")
                artifact_sha = str(artifact["sha256"])
                require(sha256(absolute) == artifact_sha,
                        f"translation dependency file drift: {artifact_path}")
                file_artifact = FileArtifact(artifact_path, artifact_sha)
                require(artifact_path not in file_artifacts or file_artifacts[artifact_path] == file_artifact,
                        f"conflicting translation dependency file: {artifact_path}")
                file_artifacts[artifact_path] = file_artifact
        translations[source_path] = Translation(
            source_path, owner.unit, target_path, owner.source_sha256, target_sha,
            snapshot_path, snapshot_sha,
        )
        target_owners[target_path] = source_path

    translated_sources = {owner.source_path for owner in owners.values() if owner.translated}
    require(set(translations) == translated_sources,
            "translation receipt denominator does not exactly cover translated sources")
    require(len({item.snapshot_path for item in translations.values()})
            == plan["translation_snapshot_denominator"],
            "translation snapshot denominator drift")
    require(len(file_artifacts) == plan["translation_dependency_file_denominator"],
            "translation dependency-file denominator drift")
    require(len(tree_artifacts) == plan["translation_dependency_tree_denominator"],
            "translation dependency-tree denominator drift")
    require(len({member for tree in tree_artifacts.values() for member in tree.members})
            == plan["translation_dependency_tree_file_denominator"],
            "translation dependency-tree file denominator drift")
    return translations, file_artifacts, tree_artifacts


def load_support_artifacts(repo: Path, manifest: dict[str, Any], plan: dict[str, Any],
                           translated_targets: set[str]) -> dict[str, SupportArtifact]:
    inventory_path = repo_path(repo, str(manifest["source_review_support_inventory"]),
                               "source-review support inventory")
    require(git_tracked(repo, inventory_path), "source-review support inventory is not tracked")
    rows = read_tsv(inventory_path)
    expected_header = [
        "artifact_path", "artifact_sha256", "logical_lines", "artifact_role",
        "review_overlay", "source_authority", "disposition",
    ]
    if rows:
        require(list(rows[0]) == expected_header, "source-review support inventory header drift")
    support: dict[str, SupportArtifact] = {}
    for row in rows:
        artifact = SupportArtifact(
            row["artifact_path"], row["artifact_sha256"], int(row["logical_lines"]),
            row["artifact_role"], row["review_overlay"], row["source_authority"],
            row["disposition"],
        )
        require(artifact.path not in support, f"duplicate support artifact: {artifact.path}")
        require(artifact.path not in translated_targets,
                f"support artifact overlaps a translated target: {artifact.path}")
        require(artifact.disposition == "review-full-source-semantics",
                f"support artifact disposition drift: {artifact.path}")
        require(artifact.review_overlay in EXPECTED_OVERLAY_IDS,
                f"support artifact has unknown overlay: {artifact.path}")
        require(artifact.artifact_role.strip() != "", f"support artifact has no role: {artifact.path}")
        require(artifact.source_authority.strip() != "",
                f"support artifact has no source authority: {artifact.path}")
        path = repo_path(repo, artifact.path, "source-review support artifact")
        require(path.is_file(), f"support artifact is missing: {artifact.path}")
        require(git_tracked(repo, path), f"support artifact is not tracked: {artifact.path}")
        require(sha256(path) == artifact.sha256, f"support artifact hash drift: {artifact.path}")
        require(logical_lines(path) == artifact.logical_lines,
                f"support artifact line-count drift: {artifact.path}")
        support[artifact.path] = artifact
    require(len(support) == plan["support_artifact_denominator"],
            "source-review support artifact denominator drift")
    require(sum(item.logical_lines for item in support.values())
            == plan["support_artifact_logical_line_denominator"],
            "source-review support logical-line denominator drift")
    return support


def load_dependency_authorities(
    upstream: Path,
    dependencies: list[dict[str, str]],
    owners: dict[str, Owner],
    plan: dict[str, Any],
) -> dict[str, UpstreamFileAuthority]:
    expected_kinds = {
        "owned-source", "generated-from-owned-source", "pinned-source-external",
        "external-sdk-or-system", "external-tool-module",
    }
    external: dict[str, UpstreamFileAuthority] = {}
    for index, row in enumerate(dependencies):
        kind = row["resolution_kind"]
        require(kind in expected_kinds,
                f"dependency row {index} has unknown resolution kind: {kind}")
        resolved_path = row["resolved_path"]
        resolved_sha = row["resolved_sha256"]
        if kind in {"owned-source", "generated-from-owned-source"}:
            require(resolved_path in owners,
                    f"dependency row {index} resolves outside owned source: {resolved_path}")
            require(resolved_sha == owners[resolved_path].source_sha256,
                    f"dependency row {index} owned-source hash drift: {resolved_path}")
        elif kind == "pinned-source-external":
            path = upstream_path(upstream, resolved_path, "pinned external dependency")
            require(path.is_file(), f"pinned external dependency is missing: {resolved_path}")
            require(sha256(path) == resolved_sha,
                    f"pinned external dependency hash drift: {resolved_path}")
            authority = UpstreamFileAuthority(
                resolved_path, resolved_sha, logical_lines(path), path.stat().st_size,
            )
            require(resolved_path not in external or external[resolved_path] == authority,
                    f"conflicting pinned external dependency: {resolved_path}")
            external[resolved_path] = authority
        else:
            require(resolved_path == resolved_sha == "-",
                    f"external dependency row {index} invents pinned bytes")
    require(len(external) == plan["pinned_external_dependency_files"],
            "pinned external dependency denominator drift")
    require(sum(item.logical_lines for item in external.values())
            == plan["pinned_external_dependency_logical_lines"],
            "pinned external dependency logical-line denominator drift")
    require(sum(item.byte_count for item in external.values())
            == plan["pinned_external_dependency_bytes"],
            "pinned external dependency byte denominator drift")
    return external


def load_generated_outputs(
    repo: Path,
    upstream: Path,
    manifest: dict[str, Any],
    generated: list[dict[str, str]],
    plan: dict[str, Any],
) -> dict[str, UpstreamFileAuthority]:
    toolchain = load_toml(
        repo_path(repo, str(manifest["toolchain_authority"]), "toolchain authority"),
        "toolchain authority",
    )
    shader_directory = str(toolchain.get("shader_directory", ""))
    require(shader_directory == "renderer/src/shaders",
            "generated shader directory authority drift")
    retained_inventory_paths = {
        row["artifact_path"] for row in generated if row["retention"] == "retained"
    }
    outputs: dict[str, UpstreamFileAuthority] = {}
    ephemeral_count = 0
    seen_inventory_paths: set[str] = set()
    for index, row in enumerate(generated):
        artifact_path = row["artifact_path"]
        require(artifact_path not in seen_inventory_paths,
                f"duplicate generated artifact row: {artifact_path}")
        seen_inventory_paths.add(artifact_path)
        require(row["stage"].strip() != "", f"generated row {index} has no stage")
        require(row["direct_include_count"].isdigit(),
                f"generated row {index} has invalid include count")
        full_path = f"{shader_directory}/{artifact_path}"
        path = upstream_path(upstream, full_path, "generated output")
        if row["retention"] == "retained":
            require(path.is_file(), f"retained generated output is missing: {full_path}")
            require(sha256(path) == row["artifact_sha256"],
                    f"retained generated output hash drift: {full_path}")
            outputs[full_path] = UpstreamFileAuthority(
                full_path, row["artifact_sha256"], logical_lines(path), path.stat().st_size,
            )
        else:
            require(row["retention"] == "ephemeral-final-header-retained",
                    f"generated row {index} has unknown retention")
            require(row["artifact_sha256"] == "-" and not path.exists(),
                    f"ephemeral generated output unexpectedly exists: {full_path}")
            final_header = Path(artifact_path).with_suffix(".hpp").as_posix()
            require(final_header in retained_inventory_paths,
                    f"ephemeral generated output has no retained header: {artifact_path}")
            ephemeral_count += 1
    require(len(generated) == plan["generated_artifacts"],
            "generated artifact denominator drift")
    require(len(outputs) == plan["retained_generated_artifacts"],
            "retained generated artifact denominator drift")
    require(ephemeral_count == plan["ephemeral_generated_artifacts"],
            "ephemeral generated artifact denominator drift")
    require(sum(item.logical_lines for item in outputs.values())
            == plan["retained_generated_logical_lines"],
            "retained generated logical-line denominator drift")
    require(sum(item.byte_count for item in outputs.values())
            == plan["retained_generated_bytes"],
            "retained generated byte denominator drift")
    return outputs


def derive_browser_bridge_authority(
    repo: Path,
    upstream: Path,
    owners: dict[str, Owner],
    units: dict[str, Unit],
) -> tuple[set[str], set[str]]:
    """Derive exact current factory/module/WebGL/WebGPU host seam records."""
    unit_components = {unit.unit: unit.component_id for unit in units.values()}
    require(BROWSER_COMPONENT_SOURCE_PATHS <= set(owners),
            "browser bridge authority source membership drift")
    component_ids = {
        unit_components[owners[source_path].unit]
        for source_path in BROWSER_COMPONENT_SOURCE_PATHS
    }
    require(component_ids == EXPECTED_BROWSER_COMPONENT_IDS,
            "browser bridge authority component membership drift")

    tokens: set[str] = set()

    factory_counts: dict[str, int] = {}
    for backend, (source_path, class_name) in BROWSER_FACTORY_SOURCES.items():
        text = upstream_path(upstream, source_path, "browser factory source").read_text(
            encoding="utf-8", errors="replace"
        )
        matches = list(re.finditer(
            rf"\b{re.escape(class_name)}::(MakeContext|makeOreContext)\s*\(", text
        ))
        factory_counts[backend] = len(matches)
        for match in matches:
            line = text.count("\n", 0, match.start()) + 1
            tokens.add(
                f"browser-factory:{backend}:{source_path}:{line}:"
                f"{class_name}::{match.group(1)}"
            )
    require(factory_counts == {"vulkan": 2, "webgl2": 3, "webgpu": 2},
            "browser factory denominator drift")

    factory_declaration_counts: dict[str, int] = {}
    factory_declaration_pattern = re.compile(
        r"\bstatic\s+std::unique_ptr<RenderContext>\s+(MakeContext)\s*\("
        r"|\bstd::unique_ptr<rive::ore::Context>\s+(makeOreContext)\s*"
        r"\(\s*\)\s*override\s*;",
        re.DOTALL,
    )
    for backend, source_path in BROWSER_FACTORY_HEADERS.items():
        text = upstream_path(
            upstream, source_path, "browser factory declaration source"
        ).read_text(encoding="utf-8", errors="replace")
        matches = list(factory_declaration_pattern.finditer(text))
        factory_declaration_counts[backend] = len(matches)
        for match in matches:
            method = match.group(1) or match.group(2)
            line = text.count("\n", 0, match.start()) + 1
            tokens.add(
                f"browser-factory-decl:{backend}:{source_path}:{line}:{method}"
            )
    require(factory_declaration_counts == {"vulkan": 3, "webgl2": 4, "webgpu": 2},
            "browser factory declaration denominator drift")

    cargo_path = repo_path(repo, "crates/nuxie-renderer/Cargo.toml",
                           "browser feature authority")
    cargo = load_toml(cargo_path, "browser feature authority")
    features = cargo.get("features")
    require(isinstance(features, dict), "browser feature authority has no features table")
    for feature in BROWSER_FEATURES:
        values = features.get(feature)
        require(isinstance(values, list) and all(isinstance(value, str) for value in values),
                f"browser feature authority drift: {feature}")
        tokens.add(
            "browser-feature:crates/nuxie-renderer/Cargo.toml:"
            f"{feature}={','.join(values)}"
        )

    module_path = "crates/nuxie-renderer/src/mechanical_port.rs"
    module_text = repo_path(repo, module_path, "browser module authority").read_text(
        encoding="utf-8", errors="replace"
    )
    for module, feature in BROWSER_MODULE_FEATURES.items():
        pattern = re.compile(
            rf"(?m)^#\[cfg\(feature = \"{re.escape(feature)}\"\)\]\n"
            rf"(?P<module>pub\(crate\) mod {re.escape(module)} \{{)"
        )
        matches = list(pattern.finditer(module_text))
        require(len(matches) == 1, f"browser module gate drift: {module}")
        line = module_text.count("\n", 0, matches[0].start("module")) + 1
        tokens.add(f"browser-module:{module_path}:{line}:{module}:cfg={feature}")

    member_count = 0
    for source_path in sorted(BROWSER_COMPONENT_SOURCE_PATHS):
        owner = owners[source_path]
        require(owner.translated and owner.target_path,
                f"browser module member is not translated: {source_path}")
        target_basename = Path(owner.target_path).name
        member_pattern = re.compile(
            rf'(?m)^(?P<indent>[ \t]*)#\[path = "{re.escape(target_basename)}"\]\n'
            rf'(?P=indent)pub\(crate\) mod (?P<module>[A-Za-z_]\w*);'
        )
        matches = list(member_pattern.finditer(module_text))
        require(len(matches) == 1,
                f"browser module member wiring drift: {source_path}")
        line = module_text.count("\n", 0, matches[0].start()) + 1
        tokens.add(
            f"browser-module-member:{source_path}->{owner.target_path}:"
            f"support={module_path}:{line}:module={matches[0].group('module')}"
        )
        member_count += 1
    require(member_count == 14, "browser module member denominator drift")

    lib_path = "crates/nuxie-renderer/src/lib.rs"
    lib_text = repo_path(repo, lib_path, "browser root module authority").read_text(
        encoding="utf-8", errors="replace"
    )
    root_gate_pattern = re.compile(
        r"#\[cfg\(any\(\s*"
        r"feature = \"native-vulkan-experimental\",\s*"
        r"feature = \"native-webgpu-experimental\",\s*"
        r"all\(\s*feature = \"native-metal-experimental\",\s*"
        r"any\(\s*target_os = \"ios\",\s*target_os = \"macos\",\s*"
        r"target_os = \"tvos\",\s*target_os = \"visionos\"\s*\)\s*\)\s*"
        r"\)\)\]\s*mod mechanical_port;"
    )
    require(len(root_gate_pattern.findall(lib_text)) == 1,
            "browser root module gate drift")
    tokens.add(
        "browser-module:crates/nuxie-renderer/src/lib.rs:mechanical_port:"
        "cfg=native-vulkan-experimental|native-webgpu-experimental|"
        "native-metal-experimental@(ios|macos|tvos|visionos)"
    )

    gles3_source = "renderer/include/rive/renderer/gl/gles3.hpp"
    gles3_owner = owners[gles3_source]
    require(gles3_owner.translated, "browser GL command authority is not translated")
    expected_gles3_target = (
        "crates/nuxie-renderer/src/mechanical_port/webgl2/"
        "renderer_include_rive_renderer_gl_gles3_hpp__decl.rs"
    )
    require(gles3_owner.target_path == expected_gles3_target,
            "browser GL command target drift")
    gles3_lines = repo_path(repo, expected_gles3_target,
                            "browser GL command authority").read_text(
                                encoding="utf-8", errors="replace"
                            ).splitlines()

    command_starts = [index for index, line in enumerate(gles3_lines)
                      if "enum GLCommand {" in line]
    require(len(command_starts) == 1, "browser GL command enum drift")
    depth = 0
    command_names: set[str] = set()
    command_closed = False
    for index in range(command_starts[0], len(gles3_lines)):
        line = gles3_lines[index]
        prior_depth = depth
        depth += line.count("{") - line.count("}")
        if index > command_starts[0] and prior_depth == 1:
            match = re.match(r"    ([A-Z][A-Za-z0-9_]*)\s*(?:\(|\{|,)", line)
            if match:
                command_names.add(match.group(1))
        if index > command_starts[0] and depth == 0:
            command_closed = True
            break
    require(command_closed and len(command_names) == 113,
            "browser GL command variant denominator drift")
    tokens.update(f"webgl-command:{name}" for name in command_names)

    query_starts = [index for index, line in enumerate(gles3_lines)
                    if "trait GLExecutionProvider" in line]
    require(len(query_starts) == 1, "browser GL query trait drift")
    depth = 0
    query_lines: dict[str, list[int]] = {}
    query_closed = False
    for index in range(query_starts[0], len(gles3_lines)):
        line = gles3_lines[index]
        depth += line.count("{") - line.count("}")
        if index > query_starts[0]:
            match = re.match(r"\s*(?:unsafe\s+)?fn\s+([A-Za-z_]\w*)\s*\(", line)
            if match:
                query_lines.setdefault(match.group(1), []).append(index + 1)
        if index > query_starts[0] and depth == 0:
            query_closed = True
            break
    require(query_closed and len(query_lines) == 23,
            "browser GL query denominator drift")
    tokens.update(
        f"webgl-query:{name}:lines={','.join(str(line) for line in lines)}"
        for name, lines in query_lines.items()
    )

    extension_source_path = "renderer/src/gl/render_context_gl_impl.cpp"
    extension_source_text = upstream_path(
        upstream, extension_source_path, "WebGL extension query authority"
    ).read_text(encoding="utf-8", errors="replace")
    extension_target_path = (
        "crates/nuxie-renderer/src/mechanical_port/webgl2/"
        "renderer_src_gl_render_context_gl_impl_cpp__impl.rs"
    )
    require(owners[extension_source_path].target_path == extension_target_path,
            "WebGL extension query target drift")
    extension_target_text = repo_path(
        repo, extension_target_path, "WebGL extension query target authority"
    ).read_text(encoding="utf-8", errors="replace")
    extension_queries: set[str] = set()
    for match in re.finditer(
        r"emscripten_webgl_enable_extension\s*\(\s*"
        r"emscripten_webgl_get_current_context\s*\(\s*\)\s*,\s*"
        r"\"([^\"]+)\"\s*\)",
        extension_source_text,
        re.DOTALL,
    ):
        extension = match.group(1)
        require(extension not in extension_queries,
                f"duplicate WebGL extension query: {extension}")
        extension_queries.add(extension)
        target_matches = list(re.finditer(
            rf'enableWebGLExtension\("{re.escape(extension)}"\)',
            extension_target_text,
        ))
        require(len(target_matches) == 1,
                f"WebGL extension query translation drift: {extension}")
        source_line = extension_source_text.count("\n", 0, match.start()) + 1
        target_line = extension_target_text.count(
            "\n", 0, target_matches[0].start()
        ) + 1
        tokens.add(
            f"webgl-extension-query:{extension_source_path}:{source_line}:{extension}->"
            f"{extension_target_path}:{target_line}"
        )
    require(extension_queries == {
        "WEBGL_debug_renderer_info", "WEBGL_clip_cull_distance",
        "EXT_color_buffer_half_float", "OES_texture_half_float_linear",
        "EXT_color_buffer_float", "EXT_float_blend", "KHR_parallel_shader_compile",
        "WEBGL_compressed_texture_s3tc", "EXT_texture_compression_bptc",
        "WEBGL_compressed_texture_astc",
    }, "WebGL extension query denominator drift")

    em_js_symbols: set[str] = set()
    for source_path in ("renderer/src/gl/gl_utils.cpp", "renderer/src/gl/pls_impl_webgl.cpp"):
        text = upstream_path(upstream, source_path, "browser EM_JS authority").read_text(
            encoding="utf-8", errors="replace"
        )
        for match in re.finditer(
            r"\bEM_JS\s*\(\s*[^,]+,\s*([A-Za-z_]\w*)\s*,", text, re.DOTALL
        ):
            symbol = match.group(1)
            require(symbol in WEBGL_EM_JS_SEMANTICS,
                    f"browser EM_JS semantic mapping missing: {symbol}")
            require(symbol not in em_js_symbols, f"duplicate browser EM_JS symbol: {symbol}")
            em_js_symbols.add(symbol)
            target_semantic = WEBGL_EM_JS_SEMANTICS[symbol]
            require(target_semantic in command_names or target_semantic in query_lines,
                    f"browser EM_JS target semantic missing: {target_semantic}")
            line = text.count("\n", 0, match.start()) + 1
            tokens.add(
                f"webgl-em-js:{source_path}:{line}:{symbol}->{target_semantic}"
            )
    require(em_js_symbols == set(WEBGL_EM_JS_SEMANTICS),
            "browser EM_JS symbol denominator drift")

    js_symbols: dict[str, tuple[str, int]] = {}
    for source_path in WEBGPU_JS_STUB_PATHS:
        lines = upstream_path(upstream, source_path, "WebGPU JS host authority").read_text(
            encoding="utf-8", errors="replace"
        ).splitlines()
        for line_number, line in enumerate(lines, 1):
            match = re.match(r"\s*(wgpu\w+):\s*undefined,?", line)
            if match:
                symbol = match.group(1)
                require(symbol not in js_symbols,
                        f"duplicate WebGPU JS host symbol: {symbol}")
                js_symbols[symbol] = (source_path, line_number)

    declaration_symbols: dict[str, tuple[str, int]] = {}
    for source_path in WEBGPU_HEADER_PATHS:
        lines = upstream_path(upstream, source_path,
                              "WebGPU host declaration authority").read_text(
                                  encoding="utf-8", errors="replace"
                              ).splitlines()
        for line_number, line in enumerate(lines, 1):
            for match in re.finditer(r"\b(wgpu\w+)\s*\(", line):
                symbol = match.group(1)
                require(symbol not in declaration_symbols,
                        f"duplicate WebGPU host declaration: {symbol}")
                declaration_symbols[symbol] = (source_path, line_number)

    require(len(js_symbols) == 273 and len(declaration_symbols) == 272,
            "WebGPU host symbol denominator drift")
    require(set(js_symbols) - set(declaration_symbols) == {
        "wgpuGetInstanceCapabilities",
        "wgpuWagyuRenderPassEncoderSetShaderPixelLocalStorageEnabled",
    }, "WebGPU JS-only host symbol drift")
    require(set(declaration_symbols) - set(js_symbols) == {
        "wgpuSupportedInstanceFeaturesFreeMembers",
    }, "WebGPU declaration-only host symbol drift")
    for symbol in set(js_symbols) | set(declaration_symbols):
        js = js_symbols.get(symbol)
        declaration = declaration_symbols.get(symbol)
        symbol_class = "matched" if js and declaration else ("js-only" if js else "decl-only")
        js_binding = f"{js[0]}:{js[1]}" if js else "-"
        declaration_binding = (f"{declaration[0]}:{declaration[1]}"
                               if declaration else "-")
        tokens.add(
            f"webgpu-host-symbol:{symbol}:class={symbol_class}:js={js_binding}:"
            f"decl={declaration_binding}"
        )

    registrations: set[tuple[str, str]] = set()
    for source_path in WEBGPU_JS_STUB_PATHS:
        lines = upstream_path(upstream, source_path, "WebGPU JS registration authority").read_text(
            encoding="utf-8", errors="replace"
        ).splitlines()
        for line_number, line in enumerate(lines, 1):
            match = re.search(r"addToLibrary\((Library\w+)\)", line)
            if match:
                registrations.add((source_path, match.group(1)))
                tokens.add(
                    f"webgpu-js-register:{source_path}:{line_number}:{match.group(1)}"
                )
    require(registrations == {
        (WEBGPU_JS_STUB_PATHS[0], "LibraryWebGPU"),
        (WEBGPU_JS_STUB_PATHS[1], "LibraryWebGPUExtensions"),
    }, "WebGPU JS registration authority drift")

    port_lines = upstream_path(upstream, WEBGPU_PORT_PATH,
                               "WebGPU port wiring authority").read_text(
                                   encoding="utf-8", errors="replace"
                               ).splitlines()
    wired_libraries: set[str] = set()
    no_fallback_lines: list[int] = []
    for index, line in enumerate(port_lines):
        wire = re.search(
            r"JS_LIBRARIES \+= \[ os\.path\.join\(src_dir, '([^']+)'\) \]", line
        )
        if wire:
            library = wire.group(1)
            wired_libraries.add(library)
            tokens.add(f"webgpu-js-wire:{WEBGPU_PORT_PATH}:{index + 1}:{library}")
        if re.search(r"if settings\.USE_WEBGPU:", line):
            require(index + 1 < len(port_lines)
                    and "raise Exception('webgpu-port is not compatible with deprecated "
                        "Emscripten USE_WEBGPU option')" in port_lines[index + 1],
                    "WebGPU USE_WEBGPU rejection body drift")
            no_fallback_lines.append(index + 1)
            tokens.add(
                "webgpu-no-deprecated-emscripten:"
                f"{WEBGPU_PORT_PATH}:{index + 1}:reject=USE_WEBGPU"
            )
    require(wired_libraries == {
        "library_webgpu_stubs.js", "library_webgpu_wagyu_stubs.js",
    }, "WebGPU JS library wiring drift")
    require(no_fallback_lines == [112], "WebGPU USE_WEBGPU rejection denominator drift")

    expected_prefix_counts = {
        "browser-factory": 7,
        "browser-factory-decl": 9,
        "browser-feature": 4,
        "browser-module": 4,
        "browser-module-member": 14,
        "webgl-command": 113,
        "webgl-query": 23,
        "webgl-extension-query": 10,
        "webgl-em-js": 9,
        "webgpu-host-symbol": 274,
        "webgpu-js-register": 2,
        "webgpu-js-wire": 2,
        "webgpu-no-deprecated-emscripten": 1,
    }
    prefix_counts = {
        prefix: sum(token.startswith(f"{prefix}:") for token in tokens)
        for prefix in expected_prefix_counts
    }
    require(prefix_counts == expected_prefix_counts and len(tokens) == 472,
            "browser bridge typed-record denominator drift")
    return component_ids, tokens


def overlay_expectations(
    plan: dict[str, Any],
    repo: Path,
    upstream: Path,
    owners: dict[str, Owner],
    units: dict[str, Unit],
    dependencies: list[dict[str, str]],
    configurations: list[dict[str, str]],
    generated: list[dict[str, str]],
    generated_outputs: dict[str, UpstreamFileAuthority],
    external_authorities: dict[str, UpstreamFileAuthority],
    file_artifacts: dict[str, FileArtifact],
    tree_artifacts: dict[str, TreeArtifact],
    support: dict[str, SupportArtifact],
    validate_plan: bool = True,
) -> dict[str, OverlayExpectation]:
    unit_component = {unit.unit: unit.component_id for unit in units.values()}
    translated_sources = {owner.source_path for owner in owners.values() if owner.translated}
    browser_components, browser_bridge_tokens = derive_browser_bridge_authority(
        repo, upstream, owners, units
    )
    shared_rows = [
        row for row in dependencies
        if row["source_unit"] in units
        and row["dependency_unit"] in units
        and units[row["source_unit"]].campaign in BACKEND_CAMPAIGNS
        and units[row["dependency_unit"]].campaign == "shader-build-authority"
    ]
    semantic_shared_rows = [row for row in shared_rows
                            if row["source_path"] in translated_sources]
    shared_pairs = {
        (row["source_unit"], row["dependency_unit"])
        for row in shared_rows
    }
    semantic_shared_pairs = {
        (row["source_unit"], row["dependency_unit"])
        for row in semantic_shared_rows
    }
    bridge_rows = [
        row for row in dependencies
        if row["source_unit"] == "webgpu:renderer:render_context_webgpu_impl"
        and row["dependency_unit"] == "webgl2:renderer:load_store_actions_ext"
    ]
    bridge_pairs = {
        (row["source_unit"], row["dependency_unit"])
        for row in bridge_rows
    }
    semantic_bridge_pairs = {
        pair for pair in bridge_pairs
        if any(row["source_path"] in translated_sources
               and (row["source_unit"], row["dependency_unit"]) == pair
               for row in bridge_rows)
    }
    generated_rows = [row for row in dependencies
                      if row["resolution_kind"] == "generated-from-owned-source"]
    semantic_generated_rows = [row for row in generated_rows
                               if row["source_path"] in translated_sources]
    generated_pairs = {(row["source_unit"], row["dependency_unit"])
                       for row in generated_rows}
    semantic_generated_pairs = {(row["source_unit"], row["dependency_unit"])
                                for row in semantic_generated_rows}
    webgpu_abi_rows = [
        row for row in dependencies
        if row["source_unit"] in units
        and row["dependency_unit"] in units
        and units[row["source_unit"]].campaign == "webgpu"
        and units[row["dependency_unit"]].campaign == "webgpu"
        and (":wagyu:" in row["dependency_unit"]
             or row["dependency_unit"] == "webgpu:renderer:webgpu_compat")
    ]
    webgpu_abi_pairs = {(row["source_unit"], row["dependency_unit"])
                       for row in webgpu_abi_rows}
    def is_ore_authority(path: str) -> bool:
        return (path == "renderer/include/rive/renderer/gpu_resource.hpp"
                or path.startswith("renderer/include/rive/renderer/ore/"))

    ore_rows = [row for row in dependencies
                if row["resolution_kind"] == "pinned-source-external"
                and is_ore_authority(row["resolved_path"])]
    ore_seams = {(row["source_unit"], row["resolved_path"]) for row in ore_rows}
    renderer_rows = [
        row for row in dependencies
        if row["resolution_kind"] == "pinned-source-external"
        and not is_ore_authority(row["resolved_path"])
    ]
    renderer_seams = {(row["source_unit"], row["resolved_path"])
                      for row in renderer_rows}
    vma_include_rows = [row for row in dependencies if row["dependency_token"] == "vk_mem_alloc.h"]
    vma_probes: set[str] = set()
    vma_units: set[str] = set()
    vma_pattern = re.compile(r"\b(vma[A-Z]\w*)\s*\(")
    for owner in owners.values():
        if owner.campaign != "vulkan":
            continue
        source = upstream_path(upstream, owner.source_path, "VMA probe source")
        for line_number, line in enumerate(source.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
            for match in vma_pattern.finditer(line):
                vma_probes.add(f"vma-probe:{owner.source_path}:{line_number}:{match.group(1)}")
                vma_units.add(owner.unit)
    require((len(shared_rows), len(semantic_shared_rows), len(shared_pairs),
             len(semantic_shared_pairs)) == (198, 194, 107, 103),
            "shared-authority overlay raw/pair denominator drift")
    require(len(bridge_rows) == len(bridge_pairs) == 1,
            "WebGPU/WebGL2 bridge denominator drift")
    require((len(generated_rows), len(semantic_generated_rows), len(generated_pairs),
             len(semantic_generated_pairs)) == (440, 438, 351, 349),
            "generated-authority overlay raw/pair denominator drift")
    require((len(webgpu_abi_rows), len(webgpu_abi_pairs)) == (20, 19),
            "WebGPU ABI overlay raw/pair denominator drift")
    require((len(ore_rows), len(ore_seams)) == (35, 35),
            "shared ORE overlay denominator drift")
    require((len(renderer_rows), len(renderer_seams)) == (75, 72),
            "shared renderer overlay denominator drift")
    require((len(vma_include_rows), len(vma_probes)) == (6, 18),
            "VMA overlay include/probe denominator drift")

    def components_for_units(unit_ids: Iterable[str]) -> set[str]:
        return {unit_component[unit_id] for unit_id in unit_ids if unit_id in unit_component}

    def matching_artifacts(overlay_id: str) -> set[str]:
        if overlay_id == "generated-authority":
            return {path for path in file_artifacts
                    if not path.startswith("vendor/vk-mem-0.5.0/")}
        if overlay_id == "webgpu-abi":
            return {path for path in file_artifacts
                    if ("webgpu" in path.lower() or "wagyu" in path.lower())
                    and "/source/generated_glsl/" not in path}
        if overlay_id == "vulkan-vma-adaptation":
            return {path for path in file_artifacts
                    if path.startswith("vendor/vk-mem-0.5.0/")}
        return set()

    def matching_trees(overlay_id: str) -> set[str]:
        if overlay_id == "generated-authority":
            return {path for path in tree_artifacts if "Vulkan-Headers" not in path}
        if overlay_id == "vulkan-vma-adaptation":
            return {path for path in tree_artifacts if "Vulkan-Headers" in path}
        return set()

    plan_overlays = plan["overlay"]
    require(isinstance(plan_overlays, list), "source-review overlays must be an array")
    require([item.get("id") for item in plan_overlays] == EXPECTED_OVERLAY_IDS,
            "source-review overlay order drift")
    if validate_plan:
        require(plan["overlay_denominator"] == len(EXPECTED_OVERLAY_IDS),
                "source-review overlay denominator drift")
    expectations: dict[str, OverlayExpectation] = {}
    for index, table in enumerate(plan_overlays):
        require(isinstance(table, dict), f"overlay {index} must be a table")
        overlay_id = EXPECTED_OVERLAY_IDS[index]
        if validate_plan:
            require_exact_keys(table, OVERLAY_PLAN_KEYS, f"overlay {overlay_id}")
        require(table["id"] == overlay_id, f"overlay ID drift: {overlay_id}")
        require(isinstance(table["rule"], str) and table["rule"].strip(),
                f"overlay {overlay_id} has no rule")

        dependency_tokens: set[str] = set()
        semantic_dependency_count = 0
        configuration_tokens: set[str] = set()
        build_predicate_tokens: set[str] = set()
        generated_tokens: set[str] = set()
        bridge_tokens: set[str] = set()
        excluded_tokens: set[str] = set()
        if overlay_id == "shared-authority-consumers":
            dependency_tokens = {
                *(canonical_row("dependency-raw", row) for row in shared_rows),
                *(f"dependency-pair:{source}->{dependency}"
                  for source, dependency in shared_pairs),
            }
            semantic_dependency_count = len(semantic_shared_rows) + len(semantic_shared_pairs)
            component_ids = components_for_units(unit for pair in shared_pairs for unit in pair)
        elif overlay_id == "webgpu-to-webgl2-load-store":
            dependency_tokens = {
                *(canonical_row("dependency-raw", row) for row in bridge_rows),
                *(f"dependency-pair:{source}->{dependency}"
                  for source, dependency in bridge_pairs),
            }
            semantic_dependency_count = len(bridge_rows) + len(semantic_bridge_pairs)
            component_ids = components_for_units(unit for pair in bridge_pairs for unit in pair)
        elif overlay_id == "generated-authority":
            dependency_tokens = {
                *(canonical_row("dependency-raw", row) for row in generated_rows),
                *(f"dependency-pair:{source}->{dependency}"
                  for source, dependency in generated_pairs),
            }
            semantic_dependency_count = (len(semantic_generated_rows)
                                         + len(semantic_generated_pairs))
            generated_tokens = {canonical_row("generated", row) for row in generated}
            component_ids = components_for_units(
                unit for row in generated_rows
                for unit in (row["source_unit"], row["dependency_unit"])
            )
        elif overlay_id == "webgpu-abi":
            component_ids = {unit.component_id for unit in units.values()
                             if unit.campaign == "webgpu"}
            dependency_tokens = {
                *(canonical_row("dependency-raw", row) for row in webgpu_abi_rows),
                *(f"dependency-pair:{source}->{dependency}"
                  for source, dependency in webgpu_abi_pairs),
            }
            semantic_dependency_count = len(dependency_tokens)
        elif overlay_id == "shared-ore-contracts":
            component_ids = components_for_units(row["source_unit"] for row in ore_rows)
            dependency_tokens = {
                *(canonical_row("dependency-raw", row) for row in ore_rows),
                *(f"dependency-seam:{source}->{resolved}"
                  for source, resolved in ore_seams),
            }
            semantic_dependency_count = len(dependency_tokens)
        elif overlay_id == "shared-renderer-contracts":
            component_ids = components_for_units(row["source_unit"] for row in renderer_rows)
            dependency_tokens = {
                *(canonical_row("dependency-raw", row) for row in renderer_rows),
                *(f"dependency-seam:{source}->{resolved}"
                  for source, resolved in renderer_seams),
            }
            semantic_dependency_count = len(dependency_tokens)
        elif overlay_id == "vulkan-vma-adaptation":
            component_ids = components_for_units(
                {*vma_units, *(row["source_unit"] for row in vma_include_rows)}
            )
            dependency_tokens = {
                *(canonical_row("dependency-raw", row) for row in vma_include_rows),
                *vma_probes,
            }
            semantic_dependency_count = len(dependency_tokens)
        elif overlay_id == "backend-identity-and-browser-bridges":
            component_ids = set(browser_components)
            bridge_tokens = set(browser_bridge_tokens)
        else:
            excluded_owners = [owner for owner in owners.values() if not owner.translated]
            component_ids = components_for_units(owner.unit for owner in excluded_owners)
            excluded_tokens = {f"excluded:{owner.source_path}:{owner.disposition}"
                               for owner in excluded_owners}
            build_excluded = {
                owner.source_path for owner in excluded_owners
                if owner.disposition == "source-exclusion-non-webgl2-build"
            }
            require(build_excluded == set(CLASSIFICATION_BUILD_PREDICATES),
                    "classification overlay build-exclusion membership drift")
            component_ids.add(unit_component["build:pls_renderer"])
            component_ids.add(unit_component["webgl2:renderer:pls_impl_webgl"])
            for source_path in CLASSIFICATION_BUILD_SELECTIONS:
                require(source_path in owners and owners[source_path].translated,
                        f"classification overlay selected source drift: {source_path}")
            rive_webgl_rows = [
                row for row in configurations
                if row["source_path"] == "renderer/premake5_pls_renderer.lua"
                and row["ownership_unit"] == "build:pls_renderer"
                and row["token"] == "RIVE_WEBGL"
                and row["line"] == "152"
            ]
            require(len(rive_webgl_rows) == 1,
                    "classification overlay RIVE_WEBGL authority drift")
            configuration_tokens.add(
                canonical_row("configuration-raw", rive_webgl_rows[0])
            )
            for source_path, (first, last, predicate) in \
                    CLASSIFICATION_BUILD_PREDICATES.items():
                build_predicate_tokens.add(
                    "build-exclusion:"
                    f"{source_path}:renderer/premake5_pls_renderer.lua:"
                    f"{first}-{last}:{predicate}"
                )
            for source_path, (first, last, predicate) in \
                    CLASSIFICATION_BUILD_SELECTIONS.items():
                build_predicate_tokens.add(
                    "build-selection:"
                    f"{source_path}:renderer/premake5_pls_renderer.lua:"
                    f"{first}-{last}:{predicate}"
                )

        support_paths = {artifact.path for artifact in support.values()
                         if artifact.review_overlay == overlay_id}
        if overlay_id == "backend-identity-and-browser-bridges":
            require(support_paths == EXPECTED_BROWSER_SUPPORT_PATHS,
                    "browser bridge support membership drift")
        artifact_paths = matching_artifacts(overlay_id)
        tree_paths = matching_trees(overlay_id)
        external_paths: set[str] = set()
        if overlay_id == "shared-ore-contracts":
            external_paths = {row["resolved_path"] for row in ore_rows}
        elif overlay_id == "shared-renderer-contracts":
            external_paths = {row["resolved_path"] for row in renderer_rows}
        require(external_paths <= set(external_authorities),
                f"overlay {overlay_id} has unknown external authority")
        generated_paths = (set(generated_outputs)
                           if overlay_id == "generated-authority" else set())
        records = {
            *(f"component:{component_id}" for component_id in component_ids),
            *(f"support:{path}" for path in support_paths),
            *dependency_tokens,
            *generated_tokens,
            *bridge_tokens,
            *(f"artifact:{path}:{file_artifacts[path].sha256}" for path in artifact_paths),
            *(f"tree:{path}:{tree_artifacts[path].tree_sha256}" for path in tree_paths),
            *(f"external:{path}:{external_authorities[path].sha256}"
              for path in external_paths),
            *(f"generated-output:{path}:{generated_outputs[path].sha256}"
              for path in generated_paths),
            *excluded_tokens,
            *configuration_tokens,
            *build_predicate_tokens,
        }
        record_count, record_sha = canonical_digest(records)
        expectation = OverlayExpectation(
            id=overlay_id,
            component_ids=tuple(sorted(component_ids)),
            support_paths=tuple(sorted(support_paths)),
            dependency_record_count=len(dependency_tokens),
            semantic_dependency_record_count=semantic_dependency_count,
            configuration_record_count=len(configuration_tokens),
            build_predicate_record_count=len(build_predicate_tokens),
            generated_record_count=len(generated_tokens),
            browser_bridge_record_count=len(bridge_tokens),
            generated_paths=tuple(sorted(generated_paths)),
            external_paths=tuple(sorted(external_paths)),
            artifact_paths=tuple(sorted(artifact_paths)),
            tree_bindings=tuple(sorted((path, tree_artifacts[path].tree_sha256)
                                       for path in tree_paths)),
            excluded_source_count=len(excluded_tokens),
            authority_keys=tuple(sorted(records)),
            authority_record_count=record_count,
            authority_sha256=record_sha,
        )
        expected_plan = {
            "component_count": len(expectation.component_ids),
            "support_count": len(expectation.support_paths),
            "dependency_record_count": expectation.dependency_record_count,
            "semantic_dependency_record_count": expectation.semantic_dependency_record_count,
            "configuration_record_count": expectation.configuration_record_count,
            "build_predicate_record_count": expectation.build_predicate_record_count,
            "generated_record_count": expectation.generated_record_count,
            "browser_bridge_record_count": expectation.browser_bridge_record_count,
            "physical_generated_record_count": len(expectation.generated_paths),
            "external_record_count": len(expectation.external_paths),
            "artifact_record_count": len(expectation.artifact_paths),
            "tree_count": len(expectation.tree_bindings),
            "excluded_source_count": expectation.excluded_source_count,
            "authority_record_count": expectation.authority_record_count,
            "authority_sha256": expectation.authority_sha256,
        }
        if validate_plan:
            for key, value in expected_plan.items():
                require(table[key] == value,
                        f"overlay {overlay_id} {key} drift: expected {value}")
        expectations[overlay_id] = expectation
    return expectations


def validate_changed_byte_closure(
    repo: Path,
    plan: dict[str, Any],
    translations: dict[str, Translation],
    file_artifacts: dict[str, FileArtifact],
    tree_artifacts: dict[str, TreeArtifact],
    support: dict[str, SupportArtifact],
) -> int:
    closure = plan["changed_byte_closure"]
    require(isinstance(closure, dict), "changed-byte closure must be a table")
    require_exact_keys(closure, CLOSURE_KEYS, "changed-byte closure")
    base_ref = str(closure["base_ref"])
    head_ref = str(closure["head_ref"])
    require(base_ref == EXPECTED_ADMISSION_BASE_REF,
            "changed-byte base_ref is not the pinned campaign admission revision")
    require_revision(repo, base_ref, "changed-byte base_ref")
    require_ancestor(repo, head_ref, "changed-byte head_ref")
    require(run_git(repo, "merge-base", "--is-ancestor", base_ref, head_ref).returncode == 0,
            "changed-byte base_ref is not an ancestor of head_ref")
    require(head_ref == plan["workspace_base_ref"],
            "changed-byte head_ref is not the review workspace base")
    require(closure["diff_filter"] == "ACDMRT", "changed-byte diff filter drift")
    require(closure["category_order"] == EXPECTED_CATEGORY_ORDER,
            "changed-byte category order drift")
    lists = {
        "campaign_tooling_paths": closure["campaign_tooling_paths"],
        "ownership_only_paths": closure["ownership_only_paths"],
        "explicit_deleted_paths": closure["explicit_deleted_paths"],
    }
    for name, paths in lists.items():
        require(isinstance(paths, list) and all(isinstance(path, str) for path in paths),
                f"invalid {name}")
        require(len(paths) == len(set(paths)), f"duplicate path in {name}")

    result = run_git(repo, "diff", "--name-only", "--diff-filter=ACDMRT", base_ref, head_ref)
    require(result.returncode == 0, "cannot derive changed-byte closure")
    changed = {line for line in result.stdout.splitlines() if line}
    documentation = {path for path in changed
                     if path.startswith("docs/") or path == "NUXIE_PATCH.md"
                     or path.endswith("/NUXIE_PATCH.md")}
    authorities = {
        "translated_target": {item.target_path for item in translations.values()},
        "source_snapshot": {item.snapshot_path for item in translations.values()},
        "dependency_tree_member": {member for tree in tree_artifacts.values()
                                   for member in tree.members},
        "dependency_file": set(file_artifacts),
        "source_review_support": set(support),
        "campaign_documentation": documentation,
        "campaign_tooling": set(lists["campaign_tooling_paths"]),
        "ownership_only_evidence": set(lists["ownership_only_paths"]),
        "explicit_deletion": set(lists["explicit_deleted_paths"]),
    }
    remaining = set(changed)
    counts: dict[str, int] = {}
    for category in EXPECTED_CATEGORY_ORDER:
        assigned = remaining & authorities[category]
        remaining -= assigned
        counts[category] = len(assigned)
    require(not remaining,
            f"changed-byte closure has unclassified paths: {', '.join(sorted(remaining))}")
    require(len(changed) == closure["changed_path_denominator"],
            "changed-byte path denominator drift")
    for category, count in counts.items():
        require(closure[category] == count,
                f"changed-byte category drift for {category}: expected {count}")
    return len(changed)


def validate_frozen_review_bytes(
    repo: Path,
    manifest: dict[str, Any],
    plan: dict[str, Any],
    translations: dict[str, Translation],
    file_artifacts: dict[str, FileArtifact],
    tree_artifacts: dict[str, TreeArtifact],
    support: dict[str, SupportArtifact],
) -> None:
    scopes = {str(manifest[key]) for key in FROZEN_CAMPAIGN_PATH_KEYS}
    scopes.add(str(manifest["translation_receipt_directory"]))
    scopes.update(item.target_path for item in translations.values())
    scopes.update(item.snapshot_path for item in translations.values())
    scopes.update(file_artifacts)
    scopes.update(tree_artifacts)
    scopes.update(support)
    require_frozen_scopes(repo, str(plan["workspace_base_ref"]), scopes)


def load_authority(repo: Path, upstream: Path, manifest_path: Path) -> Authority:
    require(manifest_path == (repo / "docs/backend-port-campaign.toml").resolve(),
            "source-review campaign manifest path drift")
    manifest = load_toml(manifest_path, "campaign manifest")
    ownership_status = manifest.get("ownership_review_status")
    manifest_keys = set(EXPECTED_MANIFEST_KEYS)
    if ownership_status == "complete":
        manifest_keys.update(OWNERSHIP_COMPLETION_PIN_KEYS)
    require_exact_keys(manifest, manifest_keys, "campaign manifest")
    require(manifest.get("schema_version") == 1, "invalid campaign manifest schema")
    require(git_tracked(repo, manifest_path), "campaign manifest is not tracked")
    require(manifest.get("upstream_ref") == EXPECTED_UPSTREAM_REF,
            "campaign is not pinned to the expected upstream revision")
    validate_queue(manifest)
    require(manifest.get("ignored_skills") == ["implement", "tdd"],
            "ignored-skill contract drift")
    require(manifest.get("source_review_status") in {"active", "complete"},
            "source-review status is not active or complete")
    if manifest["active_queue"] != "source-review":
        require(manifest["source_review_status"] == "complete",
                "campaign advanced past an incomplete source review")
    for key, expected in EXPECTED_SOURCE_REVIEW_PATHS.items():
        require(manifest.get(key) == expected,
                f"source-review authority path drift: {key}")
    for key, expected in EXPECTED_OWNERSHIP_REVIEW_PATHS.items():
        require(manifest.get(key) == expected,
                f"ownership-review authority path drift: {key}")
    require(ownership_status in {"active", "complete"},
            "ownership-review status is not active or complete")
    launch_ref = manifest.get("ownership_review_launch_ref")
    require(launch_ref == "pending" or (
        isinstance(launch_ref, str)
        and re.fullmatch(r"[0-9a-f]{40}", launch_ref) is not None
    ), "ownership-review launch ref is not pending or a full SHA")
    if ownership_status == "complete":
        require(launch_ref != "pending",
                "complete ownership review has a pending launch ref")
    schema_path = repo_path(repo, str(manifest["source_review_schema"]),
                            "source-review schema")
    require(schema_path.is_file(), "source-review schema is missing")
    require(git_tracked(repo, schema_path), "source-review schema is not tracked")
    require(sha256(schema_path) == EXPECTED_SCHEMA_SHA256,
            "source-review schema bytes drifted from launch authority")
    require(manifest.get("cutover_contract") == EXPECTED_CUTOVER_CONTRACT,
            "renderer cutover contract drift")
    require(git_head(upstream) == manifest["upstream_ref"], "upstream revision drift")
    backends = manifest.get("backend", [])
    require([backend.get("id") for backend in backends] == ["vulkan", "webgpu", "webgl2"],
            "exact renderer backend set drift")
    for backend in backends:
        require(backend.get("translation_status") == "complete",
                f"backend translation is incomplete: {backend.get('id')}")
    for source_set in manifest.get("shared_source_set", []):
        require(source_set.get("translation_status") == "complete",
                f"shared translation is incomplete: {source_set.get('id')}")

    plan_path = repo_path(repo, str(manifest["source_review_plan"]), "source-review plan")
    plan = load_toml(plan_path, "source-review plan")
    require(git_tracked(repo, plan_path), "source-review plan is not tracked")
    require(sha256(plan_path) == EXPECTED_PLAN_SHA256,
            "source-review plan bytes drifted from launch authority")
    validate_plan_shape(plan, manifest, repo)
    frozen_manifest = load_toml_at_revision(
        repo, EXPECTED_WORKSPACE_BASE_REF, "docs/backend-port-campaign.toml",
        "frozen campaign manifest",
    )
    for key in FROZEN_MANIFEST_KEYS:
        require(manifest.get(key) == frozen_manifest.get(key),
                f"campaign field drifted from workspace_base_ref: {key}")

    ownership_rows = read_tsv(repo_path(repo, str(manifest["ownership_inventory"]),
                                        "ownership inventory"))
    order_rows = read_tsv(repo_path(repo, str(manifest["ownership_unit_order"]),
                                    "ownership order"))
    require(len(ownership_rows) == manifest["denominator"]["ownership_rows"],
            "ownership row denominator drift")
    require(len(ownership_rows) == manifest["denominator"]["sources"],
            "source denominator drift")
    require(len(order_rows) == manifest["denominator"]["ownership_units"],
            "ownership unit denominator drift")

    owners: dict[str, Owner] = {}
    for row in ownership_rows:
        owner = Owner(row["campaign"], row["source_path"], row["source_sha256"],
                      row["ownership_unit"], row["port_disposition"], row["target_path"])
        require(owner.source_path not in owners,
                f"duplicate frozen source owner: {owner.source_path}")
        source = upstream_path(upstream, owner.source_path, "owned source")
        require(source.is_file(), f"owned source is missing: {owner.source_path}")
        require(sha256(source) == owner.source_sha256,
                f"owned source hash drift: {owner.source_path}")
        owners[owner.source_path] = owner

    units: dict[str, Unit] = {}
    component_units: dict[str, list[str]] = {}
    component_order: dict[str, int] = {}
    for row in order_rows:
        unit = Unit(
            row["ownership_unit"], row["campaign"], int(row["order_group"]),
            row["component_id"], int(row["source_count"]),
            tuple(value for value in row["dependency_units"].split(";") if value),
        )
        require(unit.unit not in units, f"duplicate ownership unit: {unit.unit}")
        require(0 <= unit.order_group <= 6, f"invalid order_group for {unit.unit}")
        require(bool(re.fullmatch(r"component-\d{3}", unit.component_id)),
                f"invalid component id: {unit.component_id}")
        require(unit.component_id not in component_order
                or component_order[unit.component_id] == unit.order_group,
                f"SCC component is split across review waves: {unit.component_id}")
        component_order[unit.component_id] = unit.order_group
        component_units.setdefault(unit.component_id, []).append(unit.unit)
        units[unit.unit] = unit
    components = {
        component_id: Component(component_id, component_order[component_id], tuple(unit_ids))
        for component_id, unit_ids in component_units.items()
    }
    owners_by_component: dict[str, list[Owner]] = {
        component_id: [] for component_id in components
    }
    unit_owner_counts = {unit_id: 0 for unit_id in units}
    for owner in owners.values():
        require(owner.unit in units, f"source owner has unknown unit: {owner.source_path}")
        unit = units[owner.unit]
        require(owner.campaign == unit.campaign,
                f"source/unit campaign drift: {owner.source_path}")
        owners_by_component[unit.component_id].append(owner)
        unit_owner_counts[owner.unit] += 1
    for unit_id, count in unit_owner_counts.items():
        require(count == units[unit_id].source_count,
                f"ownership unit source count drift: {unit_id}")

    translated_sources = {owner.source_path for owner in owners.values() if owner.translated}
    excluded_sources = set(owners) - translated_sources
    translated_units = {owner.unit for owner in owners.values() if owner.translated}
    translated_components = {units[unit_id].component_id for unit_id in translated_units}
    source_lines = {path: logical_lines(upstream_path(upstream, path, "owned source"))
                    for path in owners}
    source_bytes = {path: upstream_path(upstream, path, "owned source").stat().st_size
                    for path in owners}
    derived_global = {
        "source_denominator": len(owners),
        "translated_source_denominator": len(translated_sources),
        "excluded_source_denominator": len(excluded_sources),
        "logical_source_line_denominator": sum(source_lines.values()),
        "translated_logical_source_line_denominator": sum(source_lines[path]
                                                           for path in translated_sources),
        "excluded_logical_source_line_denominator": sum(source_lines[path]
                                                         for path in excluded_sources),
        "source_byte_denominator": sum(source_bytes.values()),
        "translated_source_byte_denominator": sum(source_bytes[path]
                                                  for path in translated_sources),
        "excluded_source_byte_denominator": sum(source_bytes[path]
                                                for path in excluded_sources),
        "unit_denominator": len(units),
        "translated_unit_denominator": len(translated_units),
        "excluded_only_unit_denominator": len(units) - len(translated_units),
        "component_denominator": len(components),
        "translated_component_denominator": len(translated_components),
        "excluded_only_component_denominator": len(components) - len(translated_components),
        "component_receipt_denominator": len(components),
    }
    require(derived_global["source_denominator"] == 200,
            "source-review must cover exactly 200 sources")
    require(derived_global["translated_source_denominator"] == 188,
            "source-review must cover exactly 188 translated sources")
    require(derived_global["excluded_source_denominator"] == 12,
            "source-review must cover exactly 12 nontranslated sources")
    require(derived_global["unit_denominator"] == 135,
            "source-review must cover exactly 135 units")
    require(derived_global["component_denominator"] == 115,
            "source-review must cover exactly 115 SCC components")
    for key, value in derived_global.items():
        require(plan[key] == value, f"source-review plan {key} drift: expected {value}")
    validate_waves(plan, owners_by_component, units, components, upstream)

    dependencies = read_tsv(repo_path(repo, str(manifest["dependency_inventory"]),
                                      "dependency inventory"))
    configurations = read_tsv(repo_path(repo, str(manifest["configuration_inventory"]),
                                        "configuration inventory"))
    generated = read_tsv(repo_path(repo, str(manifest["generated_artifact_inventory"]),
                                   "generated inventory"))
    require(len(dependencies) == manifest["denominator"]["dependency_edges"],
            "full dependency row denominator drift")
    require(len(configurations) == manifest["denominator"]["configuration_rows"],
            "full configuration row denominator drift")
    require(len(generated) == manifest["denominator"]["generated_artifacts"],
            "full generated artifact denominator drift")
    require(plan["semantic_dependency_rows"]
            == sum(row["source_path"] in translated_sources for row in dependencies),
            "semantic dependency denominator drift")
    require(plan["semantic_configuration_rows"]
            == sum(row["source_path"] in translated_sources for row in configurations),
            "semantic configuration denominator drift")
    semantic_generated = sum(
        row["source_path"] in translated_sources
        and row["resolution_kind"] == "generated-from-owned-source"
        for row in dependencies
    )
    require(semantic_generated == plan["semantic_generated_owner_edges"],
            "semantic generated-owner edge denominator drift")

    translations, file_artifacts, tree_artifacts = load_translation_closure(
        repo, upstream, manifest, plan, owners, units
    )
    support = load_support_artifacts(
        repo, manifest, plan, {item.target_path for item in translations.values()}
    )
    support_inventory_path = repo_path(
        repo, str(manifest["source_review_support_inventory"]),
        "source-review support inventory",
    )
    require(sha256(support_inventory_path) == EXPECTED_SUPPORT_INVENTORY_SHA256,
            "source-review support inventory bytes drifted from launch authority")
    external_authorities = load_dependency_authorities(
        upstream, dependencies, owners, plan
    )
    generated_outputs = load_generated_outputs(
        repo, upstream, manifest, generated, plan
    )
    overlays = overlay_expectations(plan, repo, upstream, owners, units, dependencies,
                                    configurations, generated, generated_outputs,
                                    external_authorities, file_artifacts, tree_artifacts, support)
    validate_frozen_review_bytes(
        repo, manifest, plan, translations, file_artifacts, tree_artifacts, support
    )
    changed_path_count = validate_changed_byte_closure(
        repo, plan, translations, file_artifacts, tree_artifacts, support
    )
    receipt_directory = repo_path(repo, str(manifest["source_review_receipt_directory"]),
                                  "source-review receipt directory")
    return Authority(
        repo, upstream, manifest, plan, owners, units, components, owners_by_component,
        translations, file_artifacts, tree_artifacts, support, generated_outputs,
        external_authorities, overlays, receipt_directory, load_role(repo, manifest),
        changed_path_count,
    )


def parse_scoped_citation(citation: str, label: str) -> tuple[str, str, int, int]:
    match = re.fullmatch(
        r"(source|target|support|artifact|tree|external|generated):(.+):(\d+)-(\d+)",
        citation,
    )
    require(match is not None, f"invalid {label} citation: {citation}")
    kind, path, start, end = match.groups()
    first, last = int(start), int(end)
    require(first >= 1 and last >= first, f"invalid {label} citation range: {citation}")
    return kind, path, first, last


def full_citation(kind: str, path: str, lines: int) -> str:
    require(lines > 0, f"cannot cite empty {kind} file: {path}")
    return f"{kind}:{path}:1-{lines}"


def valid_review_run_id(value: Any) -> bool:
    return (isinstance(value, str)
            and bool(re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._:-]{7,}", value))
            and "placeholder" not in value.lower())


def canonical_component_path(authority: Authority, component_id: str) -> Path:
    return authority.receipt_directory / f"{component_id}.source-review.toml"


def validate_findings(findings: Any, open_findings: Any,
                      allowed_lines: dict[tuple[str, str], int], id_pattern: re.Pattern[str],
                      label: str) -> tuple[str, ...]:
    require(isinstance(findings, list), f"{label} findings must be an array")
    finding_ids: list[str] = []
    for index, finding in enumerate(findings):
        require(isinstance(finding, dict), f"{label} finding {index} must be a table")
        require_exact_keys(finding, FINDING_KEYS, f"{label} finding {index}")
        finding_id = finding["id"]
        require(isinstance(finding_id, str) and id_pattern.fullmatch(finding_id),
                f"{label} has unstable finding ID: {finding_id}")
        require(finding_id not in finding_ids,
                f"{label} duplicates finding ID: {finding_id}")
        require(finding["severity"] in EXPECTED_SEVERITIES,
                f"{label} finding severity drift: {finding_id}")
        require(isinstance(finding["summary"], str) and finding["summary"].strip(),
                f"{label} finding has no summary: {finding_id}")
        citations = finding["citations"]
        require(isinstance(citations, list) and citations,
                f"{label} finding has no citations: {finding_id}")
        require(len(citations) == len(set(citations)),
                f"{label} finding duplicates citations: {finding_id}")
        for citation in citations:
            require(isinstance(citation, str), f"{label} finding citation must be a string")
            kind, citation_path, _, last = parse_scoped_citation(citation, f"{label} finding")
            require((kind, citation_path) in allowed_lines,
                    f"{label} finding cites an unowned file: {citation}")
            require(last <= allowed_lines[(kind, citation_path)],
                    f"{label} finding citation exceeds file: {citation}")
        finding_ids.append(finding_id)
    require(type(open_findings) is int, f"{label} open_findings must be an integer")
    require(open_findings == len(findings), f"{label} open_findings count drift")
    return tuple(finding_ids)


def validate_component_receipt(path: Path, authority: Authority) -> ComponentReceiptResult:
    component_id = path.name.removesuffix(".source-review.toml")
    require(component_id in authority.components,
            f"unknown component receipt filename: {path.name}")
    require(path == canonical_component_path(authority, component_id),
            f"noncanonical component receipt filename: {path.name}")
    require(git_tracked(authority.repo, path), f"component receipt is not tracked: {path}")
    receipt = load_toml(path, "source-review component receipt")
    require_exact_keys(receipt, COMPONENT_RECEIPT_KEYS, f"receipt {path.name}")
    component = authority.components[component_id]
    required_scalars = {
        "schema_version": 1,
        "component_id": component_id,
        "units": list(component.units),
        "receipt_kind": "source-review-component",
        "upstream_ref": authority.manifest["upstream_ref"],
        "workspace_base_ref": authority.plan["workspace_base_ref"],
        "role": authority.source_reviewer_role,
        "review_wave": component.review_wave,
        "coverage": EXPECTED_COVERAGE,
    }
    for key, expected in required_scalars.items():
        require(receipt[key] == expected, f"receipt {path.name} {key} drift")
    review_run_id = receipt["review_run_id"]
    require(valid_review_run_id(review_run_id),
            f"receipt {path.name} has invalid review_run_id")

    expected_owners = authority.owners_by_component[component_id]
    source_records = receipt["sources"]
    require(isinstance(source_records, list), f"receipt {path.name} sources must be an array")
    source_by_path: dict[str, dict[str, Any]] = {}
    for index, record in enumerate(source_records):
        require(isinstance(record, dict), f"receipt {path.name} source {index} must be a table")
        require_exact_keys(record, SOURCE_RECORD_KEYS, f"receipt {path.name} source {index}")
        source_path = record["path"]
        require(isinstance(source_path, str) and source_path not in source_by_path,
                f"receipt {path.name} has duplicate source: {source_path}")
        source_by_path[source_path] = record
    require(set(source_by_path) == {owner.source_path for owner in expected_owners},
            f"receipt {path.name} source membership drift")
    for owner in expected_owners:
        source = upstream_path(authority.upstream, owner.source_path, "reviewed source")
        expected = {
            "path": owner.source_path,
            "sha256": owner.source_sha256,
            "citation": full_citation("source", owner.source_path, logical_lines(source)),
            "disposition": owner.disposition,
        }
        require(source_by_path[owner.source_path] == expected,
                f"receipt {path.name} source evidence drift: {owner.source_path}")

    expected_translations = [authority.translations[owner.source_path]
                             for owner in expected_owners if owner.translated]
    target_records = receipt["targets"]
    require(isinstance(target_records, list), f"receipt {path.name} targets must be an array")
    target_by_path: dict[str, dict[str, Any]] = {}
    for index, record in enumerate(target_records):
        require(isinstance(record, dict), f"receipt {path.name} target {index} must be a table")
        require_exact_keys(record, TARGET_RECORD_KEYS, f"receipt {path.name} target {index}")
        target_path = record["path"]
        require(isinstance(target_path, str) and target_path not in target_by_path,
                f"receipt {path.name} has duplicate target: {target_path}")
        target_by_path[target_path] = record
    require(set(target_by_path) == {item.target_path for item in expected_translations},
            f"receipt {path.name} target membership drift")
    for item in expected_translations:
        target = repo_path(authority.repo, item.target_path, "reviewed target")
        expected = {
            "path": item.target_path,
            "sha256": item.target_sha256,
            "citation": full_citation("target", item.target_path, logical_lines(target)),
        }
        require(target_by_path[item.target_path] == expected,
                f"receipt {path.name} target evidence drift: {item.target_path}")

    allowed_lines = {
        ("source", owner.source_path): logical_lines(
            upstream_path(authority.upstream, owner.source_path, "finding source")
        ) for owner in expected_owners
    }
    allowed_lines.update({
        ("target", item.target_path): logical_lines(
            repo_path(authority.repo, item.target_path, "finding target")
        ) for item in expected_translations
    })
    component_number = component_id.removeprefix("component-")
    finding_ids = validate_findings(
        receipt["findings"], receipt["open_findings"], allowed_lines,
        re.compile(rf"SR-C{re.escape(component_number)}-(?:0[1-9]|[1-9][0-9])"),
        f"receipt {path.name}",
    )
    return ComponentReceiptResult(
        component_id, component.review_wave, review_run_id, component.units,
        frozenset(source_by_path), frozenset(target_by_path), finding_ids,
        len(receipt["findings"]),
    )


def validate_support_receipt(path: Path, authority: Authority) -> SupportReceiptResult:
    require(path == authority.receipt_directory / "support.source-review.toml",
            "support receipt filename must be support.source-review.toml")
    require(git_tracked(authority.repo, path), f"support receipt is not tracked: {path}")
    receipt = load_toml(path, "source-review support receipt")
    require_exact_keys(receipt, SUPPORT_RECEIPT_KEYS, "support receipt")
    required_scalars = {
        "schema_version": 1,
        "receipt_kind": "source-review-support",
        "upstream_ref": authority.manifest["upstream_ref"],
        "workspace_base_ref": authority.plan["workspace_base_ref"],
        "role": authority.source_reviewer_role,
        "review_wave": "support",
        "coverage": EXPECTED_COVERAGE,
    }
    for key, expected in required_scalars.items():
        require(receipt[key] == expected, f"support receipt {key} drift")
    review_run_id = receipt["review_run_id"]
    require(valid_review_run_id(review_run_id), "support receipt has invalid review_run_id")
    records = receipt["artifacts"]
    require(isinstance(records, list), "support receipt artifacts must be an array")
    record_by_path: dict[str, dict[str, Any]] = {}
    for index, record in enumerate(records):
        require(isinstance(record, dict), f"support receipt artifact {index} must be a table")
        require_exact_keys(record, SUPPORT_RECORD_KEYS, f"support receipt artifact {index}")
        artifact_path = record["path"]
        require(isinstance(artifact_path, str) and artifact_path not in record_by_path,
                f"support receipt has duplicate artifact: {artifact_path}")
        record_by_path[artifact_path] = record
    require(set(record_by_path) == set(authority.support_artifacts),
            "support receipt artifact membership drift")
    for artifact in authority.support_artifacts.values():
        expected = {
            "path": artifact.path,
            "sha256": artifact.sha256,
            "logical_lines": artifact.logical_lines,
            "citation": full_citation("support", artifact.path, artifact.logical_lines),
            "artifact_role": artifact.artifact_role,
            "review_overlay": artifact.review_overlay,
            "source_authority": artifact.source_authority,
            "disposition": artifact.disposition,
        }
        require(record_by_path[artifact.path] == expected,
                f"support receipt evidence drift: {artifact.path}")
    allowed_lines = {("support", artifact.path): artifact.logical_lines
                     for artifact in authority.support_artifacts.values()}
    finding_ids = validate_findings(
        receipt["findings"], receipt["open_findings"], allowed_lines,
        re.compile(r"SR-SUP-(?:0[1-9]|[1-9][0-9])"), "support receipt",
    )
    return SupportReceiptResult(review_run_id, frozenset(record_by_path), finding_ids,
                                len(receipt["findings"]))


def validate_overlay_receipt(path: Path, authority: Authority) -> OverlayReceiptResult:
    require(path == authority.receipt_directory / "overlays.source-review.toml",
            "overlay receipt filename must be overlays.source-review.toml")
    require(git_tracked(authority.repo, path), f"overlay receipt is not tracked: {path}")
    receipt = load_toml(path, "source-review overlay receipt")
    require_exact_keys(receipt, OVERLAY_RECEIPT_KEYS, "overlay receipt")
    required_scalars = {
        "schema_version": 1,
        "receipt_kind": "source-review-overlays",
        "upstream_ref": authority.manifest["upstream_ref"],
        "workspace_base_ref": authority.plan["workspace_base_ref"],
        "role": authority.source_reviewer_role,
        "review_wave": "overlays",
        "coverage": EXPECTED_OVERLAY_COVERAGE,
    }
    for key, expected in required_scalars.items():
        require(receipt[key] == expected, f"overlay receipt {key} drift")
    review_run_id = receipt["review_run_id"]
    require(valid_review_run_id(review_run_id), "overlay receipt has invalid review_run_id")
    records = receipt["overlays"]
    require(isinstance(records, list), "overlay receipt overlays must be an array")
    require([record.get("id") for record in records if isinstance(record, dict)]
            == EXPECTED_OVERLAY_IDS, "overlay receipt order or membership drift")
    for index, record in enumerate(records):
        require(isinstance(record, dict), f"overlay receipt record {index} must be a table")
        require_exact_keys(record, OVERLAY_RECORD_KEYS, f"overlay receipt record {index}")
        overlay_id = EXPECTED_OVERLAY_IDS[index]
        expectation = authority.overlays[overlay_id]
        bindings = record["tree_bindings"]
        require(isinstance(bindings, list),
                f"overlay {overlay_id} tree_bindings must be an array")
        for binding_index, binding in enumerate(bindings):
            require(isinstance(binding, dict),
                    f"overlay {overlay_id} tree binding {binding_index} must be a table")
            require_exact_keys(binding, TREE_BINDING_KEYS,
                               f"overlay {overlay_id} tree binding {binding_index}")
        external_bindings = record["external_bindings"]
        generated_bindings = record["generated_bindings"]
        require(isinstance(external_bindings, list),
                f"overlay {overlay_id} external_bindings must be an array")
        require(isinstance(generated_bindings, list),
                f"overlay {overlay_id} generated_bindings must be an array")
        for binding_index, binding in enumerate([*external_bindings, *generated_bindings]):
            require(isinstance(binding, dict),
                    f"overlay {overlay_id} upstream binding {binding_index} must be a table")
            require_exact_keys(binding, UPSTREAM_FILE_BINDING_KEYS,
                               f"overlay {overlay_id} upstream binding {binding_index}")
        component_receipts = record["component_receipts"]
        support_receipts = record["support_receipts"]
        require(isinstance(component_receipts, list),
                f"overlay {overlay_id} component_receipts must be an array")
        require(isinstance(support_receipts, list),
                f"overlay {overlay_id} support_receipts must be an array")
        for binding_index, binding in enumerate([*component_receipts, *support_receipts]):
            require(isinstance(binding, dict),
                    f"overlay {overlay_id} receipt binding {binding_index} must be a table")
            require_exact_keys(binding, RECEIPT_BINDING_KEYS,
                               f"overlay {overlay_id} receipt binding {binding_index}")
        expected_component_receipts = []
        for component_id in expectation.component_ids:
            component_path = canonical_component_path(authority, component_id)
            expected_component_receipts.append({
                "id": component_id,
                "path": component_path.relative_to(authority.repo).as_posix(),
                "sha256": sha256(component_path),
            })
        expected_support_receipts = []
        if expectation.support_paths:
            support_path = authority.receipt_directory / "support.source-review.toml"
            expected_support_receipts.append({
                "id": "support",
                "path": support_path.relative_to(authority.repo).as_posix(),
                "sha256": sha256(support_path),
            })
        expected = {
            "id": overlay_id,
            "authority_record_count": expectation.authority_record_count,
            "authority_sha256": expectation.authority_sha256,
            "component_ids": list(expectation.component_ids),
            "support_paths": list(expectation.support_paths),
            "tree_bindings": [
                {"path": tree_path, "tree_sha256": tree_sha}
                for tree_path, tree_sha in expectation.tree_bindings
            ],
            "external_bindings": [
                {
                    "path": external_path,
                    "sha256": authority.external_authorities[external_path].sha256,
                    "logical_lines": authority.external_authorities[external_path].logical_lines,
                }
                for external_path in expectation.external_paths
            ],
            "generated_bindings": [
                {
                    "path": generated_path,
                    "sha256": authority.generated_outputs[generated_path].sha256,
                    "logical_lines": authority.generated_outputs[generated_path].logical_lines,
                }
                for generated_path in expectation.generated_paths
            ],
            "authority_keys": list(expectation.authority_keys),
            "component_receipts": expected_component_receipts,
            "support_receipts": expected_support_receipts,
            "attestation": "reviewed-complete-derived-overlay-authority",
        }
        require(record == expected, f"overlay receipt evidence drift: {overlay_id}")

    allowed_by_overlay: dict[str, dict[tuple[str, str], int]] = {}
    for overlay_id, expectation in authority.overlays.items():
        allowed: dict[tuple[str, str], int] = {}
        for component_id in expectation.component_ids:
            for owner in authority.owners_by_component[component_id]:
                allowed[("source", owner.source_path)] = logical_lines(
                    upstream_path(authority.upstream, owner.source_path,
                                  "overlay finding source")
                )
                if owner.translated:
                    item = authority.translations[owner.source_path]
                    allowed[("target", item.target_path)] = logical_lines(
                        repo_path(authority.repo, item.target_path, "overlay finding target")
                    )
        for support_path in expectation.support_paths:
            allowed[("support", support_path)] = \
                authority.support_artifacts[support_path].logical_lines
        for artifact_path in expectation.artifact_paths:
            allowed[("artifact", artifact_path)] = logical_lines(
                repo_path(authority.repo, artifact_path, "overlay finding artifact")
            )
        for tree_path, _ in expectation.tree_bindings:
            for member in authority.tree_artifacts[tree_path].members:
                allowed[("tree", member)] = logical_lines(
                    repo_path(authority.repo, member, "overlay finding tree member")
                )
        for external_path in expectation.external_paths:
            allowed[("external", external_path)] = \
                authority.external_authorities[external_path].logical_lines
        for generated_path in expectation.generated_paths:
            allowed[("generated", generated_path)] = \
                authority.generated_outputs[generated_path].logical_lines
        allowed_by_overlay[overlay_id] = allowed

    findings = receipt["findings"]
    require(isinstance(findings, list), "overlay receipt findings must be an array")
    finding_ids: list[str] = []
    for index, finding in enumerate(findings):
        require(isinstance(finding, dict), f"overlay finding {index} must be a table")
        require_exact_keys(finding, OVERLAY_FINDING_KEYS, f"overlay finding {index}")
        overlay_id = finding["overlay_id"]
        require(overlay_id in authority.overlays,
                f"overlay finding has unknown overlay: {overlay_id}")
        overlay_number = EXPECTED_OVERLAY_IDS.index(overlay_id) + 1
        pattern = re.compile(rf"SR-OVL-{overlay_number:02}-(?:0[1-9]|[1-9][0-9])")
        finding_id = finding["id"]
        require(isinstance(finding_id, str) and pattern.fullmatch(finding_id),
                f"overlay receipt has unstable finding ID: {finding_id}")
        require(finding_id not in finding_ids,
                f"overlay receipt duplicates finding ID: {finding_id}")
        require(finding["severity"] in EXPECTED_SEVERITIES,
                f"overlay finding severity drift: {finding_id}")
        require(isinstance(finding["summary"], str) and finding["summary"].strip(),
                f"overlay finding has no summary: {finding_id}")
        citations = finding["citations"]
        require(isinstance(citations, list) and citations,
                f"overlay finding has no citations: {finding_id}")
        require(len(citations) == len(set(citations)),
                f"overlay finding duplicates citations: {finding_id}")
        allowed = allowed_by_overlay[overlay_id]
        for citation in citations:
            require(isinstance(citation, str), "overlay finding citation must be a string")
            kind, citation_path, _, last = parse_scoped_citation(citation, "overlay finding")
            require((kind, citation_path) in allowed,
                    f"overlay finding cites outside its authority: {citation}")
            require(last <= allowed[(kind, citation_path)],
                    f"overlay finding citation exceeds file: {citation}")
        finding_ids.append(finding_id)
    require(type(receipt["open_findings"]) is int,
            "overlay receipt open_findings must be an integer")
    require(receipt["open_findings"] == len(findings),
            "overlay receipt open_findings count drift")
    return OverlayReceiptResult(review_run_id, tuple(EXPECTED_OVERLAY_IDS),
                                tuple(finding_ids), len(findings))


def validate_component_set(authority: Authority,
                           component_ids: Iterable[str]) -> list[ComponentReceiptResult]:
    return [validate_component_receipt(canonical_component_path(authority, component_id),
                                       authority)
            for component_id in component_ids]


def global_check(authority: Authority) -> None:
    require(authority.receipt_directory.is_dir(),
            f"missing source-review receipt directory: {authority.receipt_directory}")
    component_paths = {canonical_component_path(authority, component_id)
                       for component_id in authority.components}
    support_path = authority.receipt_directory / "support.source-review.toml"
    overlay_path = authority.receipt_directory / "overlays.source-review.toml"
    expected_paths = {*component_paths, support_path, overlay_path}
    actual_paths = {
        path for path in authority.receipt_directory.rglob("*") if path.is_file()
    }
    require(actual_paths == expected_paths,
            f"source-review receipt set drift: {len(actual_paths)}/{len(expected_paths)}")
    results = validate_component_set(authority, sorted(authority.components))
    support = validate_support_receipt(support_path, authority)
    overlays = validate_overlay_receipt(overlay_path, authority)
    unit_ids = [unit for result in results for unit in result.unit_ids]
    source_paths = [path for result in results for path in result.source_paths]
    target_paths = [path for result in results for path in result.target_paths]
    finding_ids = [finding for result in results for finding in result.finding_ids]
    all_finding_ids = [*finding_ids, *support.finding_ids, *overlays.finding_ids]
    require(len(unit_ids) == len(set(unit_ids)) == len(authority.units),
            "global source-review unit overlap or omission")
    require(len(source_paths) == len(set(source_paths)) == len(authority.owners),
            "global source-review source overlap or omission")
    require(len(target_paths) == len(set(target_paths)) == len(authority.translations),
            "global source-review target overlap or omission")
    require(len(all_finding_ids) == len(set(all_finding_ids)),
            "global source-review finding IDs are not unique")
    open_findings = (sum(result.open_findings for result in results)
                     + support.open_findings + overlays.open_findings)
    audit = "red" if open_findings else "green"
    print(
        "backend source-review evidence complete: structure=complete, "
        f"audit={audit}, components={len(results)}/{len(authority.components)}, "
        f"units={len(unit_ids)}/{len(authority.units)}, "
        f"sources={len(source_paths)}/{len(authority.owners)}, "
        f"targets={len(target_paths)}/{len(authority.translations)}, "
        f"support={len(support.artifact_paths)}/{len(authority.support_artifacts)}, "
        f"overlays={len(overlays.overlay_ids)}/{len(authority.overlays)}, "
        f"open_findings={open_findings}, queue={authority.manifest['active_queue']}"
    )


def partial_check(path: Path, authority: Authority) -> None:
    if not path.is_absolute():
        path = authority.repo / path
    path = path.resolve()
    require(path.parent == authority.receipt_directory,
            f"source-review receipt is outside the receipt directory: {path}")
    if path.name == "support.source-review.toml":
        validate_component_set(authority, sorted(authority.components))
        result = validate_support_receipt(path, authority)
        print("backend source-review support receipt complete: "
              f"artifacts={len(result.artifact_paths)}, open_findings={result.open_findings}")
        return
    if path.name == "overlays.source-review.toml":
        validate_component_set(authority, sorted(authority.components))
        validate_support_receipt(authority.receipt_directory / "support.source-review.toml",
                                 authority)
        result = validate_overlay_receipt(path, authority)
        print("backend source-review overlay receipt complete: "
              f"overlays={len(result.overlay_ids)}, open_findings={result.open_findings}")
        return
    component_id = path.name.removesuffix(".source-review.toml")
    require(component_id in authority.components, f"unknown component receipt: {path.name}")
    candidate = authority.components[component_id]
    prior_components = sorted(component.component_id for component in authority.components.values()
                              if component.order_group < candidate.order_group)
    validate_component_set(authority, prior_components)
    result = validate_component_receipt(path, authority)
    print("backend source-review component receipt complete: "
          f"component={result.component_id}, wave={result.review_wave}, "
          f"units={len(result.unit_ids)}, sources={len(result.source_paths)}, "
          f"targets={len(result.target_paths)}, open_findings={result.open_findings}")


def admission_check(authority: Authority) -> None:
    require(authority.manifest["active_queue"] == "source-review",
            "source-review admission is only valid in the source-review queue")
    require(authority.manifest["source_review_status"] == "active",
            "source-review admission requires active source-review status")
    tree_files = {member for tree in authority.tree_artifacts.values() for member in tree.members}
    print("backend source-review admission clean: "
          f"components={len(authority.components)}, units={len(authority.units)}, "
          f"sources={len(authority.owners)}, targets={len(authority.translations)}, "
          f"support={len(authority.support_artifacts)}, overlays={len(authority.overlays)}, "
          f"external={len(authority.external_authorities)}, "
          f"generated_outputs={len(authority.generated_outputs)}, "
          f"dependency_files={len(authority.file_artifacts)}, "
          f"dependency_trees={len(authority.tree_artifacts)}, tree_files={len(tree_files)}, "
          f"changed_paths={authority.changed_path_count}")


def main() -> int:
    args = parse_args()
    repo = args.repo_root.resolve()
    upstream = args.upstream_root.resolve()
    manifest_path = args.manifest if args.manifest.is_absolute() else repo / args.manifest
    authority = load_authority(repo, upstream, manifest_path.resolve())
    if args.admission:
        admission_check(authority)
    elif args.receipt is not None:
        partial_check(args.receipt, authority)
    else:
        global_check(authority)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, KeyError, TypeError, ValueError, csv.Error,
            tomllib.TOMLDecodeError) as error:
        print(f"backend source-review failure: {error}", file=sys.stderr)
        raise SystemExit(1)
