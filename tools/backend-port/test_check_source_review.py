#!/usr/bin/env python3
"""End-to-end and mutation tests for check_source_review.py.

The fixture is intentionally a real pair of Git repositories.  It exercises the
same hard denominators and derived overlay authorities as the production
campaign while keeping every file tiny enough for a focused checker test.
"""

from __future__ import annotations

import hashlib
import importlib.util
import json
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any


HERE = Path(__file__).resolve().parent
CHECKER = HERE / "check_source_review.py"
FIXTURE_BROWSER_COMPONENT_IDS = {"component-000", "component-001"}
FIXTURE_BROWSER_TOKENS = {
    "browser-fixture:explicit-webgpu-selection",
    "browser-fixture:explicit-webgl2-selection",
    "browser-fixture:no-automatic-fallback",
}
SPEC = importlib.util.spec_from_file_location("check_source_review_under_test", CHECKER)
assert SPEC is not None and SPEC.loader is not None
checker = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = checker
SPEC.loader.exec_module(checker)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def logical_lines(path: Path) -> int:
    return len(path.read_bytes().splitlines())


def toml_value(value: Any) -> str:
    if isinstance(value, str):
        return json.dumps(value)
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, list) or isinstance(value, tuple):
        return "[" + ", ".join(toml_value(item) for item in value) + "]"
    if isinstance(value, dict):
        return "{ " + ", ".join(
            f"{key} = {toml_value(item)}" for key, item in value.items()
        ) + " }"
    raise TypeError(f"unsupported TOML value: {value!r}")


def write_flat_toml(path: Path, values: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        "".join(f"{key} = {toml_value(value)}\n" for key, value in values.items()),
        encoding="utf-8",
    )


def write_tsv(path: Path, header: list[str], rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    body = ["\t".join(header)]
    body.extend("\t".join(str(row.get(column, "")) for column in header) for row in rows)
    path.write_text("\n".join(body) + "\n", encoding="utf-8")


def git(root: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(root), *args], text=True, capture_output=True, check=False
    )
    if result.returncode:
        raise AssertionError(
            f"git {' '.join(args)} failed in {root}:\n{result.stdout}{result.stderr}"
        )
    return result.stdout.strip()


class SyntheticCampaign:
    """Build the smallest authority graph that satisfies every hard contract."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.repo = root / "repo"
        self.upstream = root / "upstream"
        self.repo.mkdir(parents=True)
        self.upstream.mkdir(parents=True)
        # macOS exposes /var through /private/var; checker containment compares
        # resolved children against the supplied root, so keep both canonical.
        self.repo = self.repo.resolve()
        self.upstream = self.upstream.resolve()
        self._init_git(self.repo)
        self._init_git(self.upstream)
        self._build_upstream()
        self._build_units_and_owners()
        self._build_dependency_authority()
        self._build_authority_checkpoint()
        self._build_review_authority()
        git(self.repo, "add", ".")
        git(self.repo, "commit", "-qm", "add synthetic review authority and receipts")
        self.plan_sha256 = digest(self.repo / "docs/backend-port-source-review-plan.toml")
        self.support_inventory_sha256 = digest(
            self.repo / "docs/backend-port-source-review-support.tsv"
        )
        self.schema_sha256 = digest(self.repo / "docs/backend-port-source-review-schema.md")

    @staticmethod
    def _init_git(root: Path) -> None:
        git(root, "init", "-q")
        git(root, "config", "user.name", "Source Review Test")
        git(root, "config", "user.email", "source-review@example.invalid")

    def _build_upstream(self) -> None:
        self.source_paths = [f"sources/source{index:03}.cpp" for index in range(200)]
        self.source_paths[104] = "renderer/src/gl/pls_impl_webgl.cpp"
        selection_paths = sorted(
            path for path in checker.CLASSIFICATION_BUILD_SELECTIONS
            if path != "renderer/src/gl/pls_impl_webgl.cpp"
        )
        for index, source_path in zip(range(106, 113), selection_paths, strict=True):
            self.source_paths[index] = source_path
        self.source_paths[105] = "renderer/premake5_pls_renderer.lua"
        for index, source_path in zip(
            range(190, 200), sorted(checker.CLASSIFICATION_BUILD_PREDICATES), strict=True
        ):
            self.source_paths[index] = source_path
        for index in range(200):
            path = self.upstream / self.source_paths[index]
            path.parent.mkdir(parents=True, exist_ok=True)
            if index == 45:
                path.write_text(
                    "".join(f"vmaCreateResource{probe:02}();\n" for probe in range(18)),
                    encoding="utf-8",
                )
            else:
                path.write_text(f"source-{index:03}\n", encoding="utf-8")
        self.generated_output_paths = [
            f"renderer/src/shaders/generated/output{index:03}.hpp"
            for index in range(520)
        ]
        for index, relative in enumerate(self.generated_output_paths):
            path = self.upstream / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(f"generated-output-{index:03}\n", encoding="utf-8")
        self.ore_external_paths = [
            f"renderer/include/rive/renderer/ore/external{index:02}.hpp"
            for index in range(12)
        ]
        self.renderer_external_paths = [
            f"renderer/include/rive/renderer/shared/external{index:02}.hpp"
            for index in range(22)
        ]
        self.vma_external_path = "vendor/vk_mem_alloc.h"
        for index, relative in enumerate([
            *self.ore_external_paths,
            *self.renderer_external_paths,
            self.vma_external_path,
        ]):
            path = self.upstream / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(f"external-authority-{index:02}\n", encoding="utf-8")
        git(self.upstream, "add", ".")
        git(self.upstream, "commit", "-qm", "pin synthetic upstream")
        self.upstream_ref = git(self.upstream, "rev-parse", "HEAD")

    def _build_units_and_owners(self) -> None:
        ids: list[str] = []
        for index in range(135):
            if index == 0:
                unit_id = "webgpu:renderer:render_context_webgpu_impl"
            elif index == 1:
                unit_id = "webgl2:renderer:load_store_actions_ext"
            elif 2 <= index <= 20:
                unit_id = f"webgpu:wagyu:unit{index:03}"
            elif index < 45:
                unit_id = f"webgpu:renderer:unit{index:03}"
            elif index < 80:
                unit_id = f"vulkan:renderer:unit{index:03}"
            elif index < 105:
                unit_id = (
                    "webgl2:renderer:pls_impl_webgl"
                    if index == 104 else f"webgl2:renderer:unit{index:03}"
                )
            elif index == 105:
                unit_id = "build:pls_renderer"
            else:
                unit_id = f"shader:authority:unit{index:03}"
            ids.append(unit_id)
        self.unit_ids = ids

        def campaign(index: int) -> str:
            if index < 45:
                return "webgpu"
            if index < 80:
                return "vulkan"
            if index < 105:
                return "webgl2"
            return "shader-build-authority"

        self.units: dict[str, Any] = {}
        component_units: dict[str, list[str]] = {}
        for index, unit_id in enumerate(ids):
            component = f"component-{(index if index < 115 else index - 115):03}"
            order_group = int(component[-3:]) % 7
            unit = checker.Unit(unit_id, campaign(index), order_group, component, 0, ())
            self.units[unit_id] = unit
            component_units.setdefault(component, []).append(unit_id)

        unit_source_counts = {unit_id: 0 for unit_id in ids}
        self.owners: dict[str, Any] = {}
        for index in range(200):
            unit_index = index if index < 135 else 0
            unit_id = ids[unit_index]
            source_path = self.source_paths[index]
            source = self.upstream / source_path
            translated = index < 188
            owner = checker.Owner(
                campaign(unit_index),
                source_path,
                digest(source),
                unit_id,
                (
                    "translate" if translated
                    else "source-exclusion-non-webgl2-build" if index >= 190
                    else "excluded-by-pinned-build"
                ),
                f"target/target{index:03}.rs" if translated else "",
            )
            self.owners[source_path] = owner
            unit_source_counts[unit_id] += 1

        self.units = {
            unit_id: checker.Unit(
                unit.unit,
                unit.campaign,
                unit.order_group,
                unit.component_id,
                unit_source_counts[unit_id],
                unit.dependency_units,
            )
            for unit_id, unit in self.units.items()
        }
        self.components = {
            component_id: checker.Component(
                component_id,
                int(component_id[-3:]) % 7,
                tuple(unit_ids),
            )
            for component_id, unit_ids in component_units.items()
        }
        self.owners_by_component = {component_id: [] for component_id in self.components}
        for owner in self.owners.values():
            component_id = self.units[owner.unit].component_id
            self.owners_by_component[component_id].append(owner)

    def _build_authority_checkpoint(self) -> None:
        git(self.repo, "commit", "--allow-empty", "-qm", "empty translation base")
        self.closure_base = git(self.repo, "rev-parse", "HEAD")
        self.translations: dict[str, Any] = {}
        for index in range(188):
            source_path = self.source_paths[index]
            source = self.upstream / source_path
            target_path = f"target/target{index:03}.rs"
            snapshot_path = f"snapshots/source{index:03}.cpp"
            target = self.repo / target_path
            snapshot = self.repo / snapshot_path
            target.parent.mkdir(parents=True, exist_ok=True)
            snapshot.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"target-{index:03}\n", encoding="utf-8")
            shutil.copyfile(source, snapshot)
            owner = self.owners[source_path]
            self.translations[source_path] = checker.Translation(
                source_path,
                owner.unit,
                target_path,
                owner.source_sha256,
                digest(target),
                snapshot_path,
                digest(snapshot),
            )

        self.support: dict[str, Any] = {}
        for index, overlay_id in enumerate(checker.EXPECTED_OVERLAY_IDS):
            relative = f"support/support{index:02}.txt"
            path = self.repo / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(f"support-{overlay_id}\n", encoding="utf-8")
            self.support[relative] = checker.SupportArtifact(
                relative,
                digest(path),
                logical_lines(path),
                "synthetic-source-semantics-support",
                overlay_id,
                f"synthetic-authority:{overlay_id}",
                "review-full-source-semantics",
            )

        artifact_specs = {
            "vendor/generated/tool.hpp": "generated dependency file\n",
            "vendor/webgpu/abi.hpp": "webgpu ABI dependency file\n",
            "vendor/vk-mem-0.5.0/vk_mem_alloc.h": "VMA dependency file\n",
        }
        self.file_artifacts: dict[str, Any] = {}
        for relative, contents in artifact_specs.items():
            path = self.repo / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(contents, encoding="utf-8")
            self.file_artifacts[relative] = checker.FileArtifact(relative, digest(path))

        tree_specs = {
            "vendor/generated-tree": ("member.txt", "generated tree member\n"),
            "vendor/Vulkan-Headers/include": ("vulkan.h", "Vulkan header tree member\n"),
        }
        self.tree_artifacts: dict[str, Any] = {}
        for relative, (member_name, contents) in tree_specs.items():
            root = self.repo / relative
            root.mkdir(parents=True, exist_ok=True)
            (root / member_name).write_text(contents, encoding="utf-8")
            members = tuple(
                sorted(path.relative_to(self.repo).as_posix() for path in root.rglob("*") if path.is_file())
            )
            self.tree_artifacts[relative] = checker.TreeArtifact(
                relative, checker.tree_digest(root), members
            )

        tool = self.repo / "tools/backend-port/check_source_review.py"
        tool.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(CHECKER, tool)
        self._write_frozen_authorities()
        git(self.repo, "add", ".")
        git(self.repo, "commit", "-qm", "freeze translated bytes")
        self.workspace_ref = git(self.repo, "rev-parse", "HEAD")

    def _dependency_row(
        self,
        source_path: str,
        source_unit: str,
        dependency_unit: str,
        token: str,
        resolved: str,
        resolution: str,
    ) -> dict[str, str]:
        if resolution in {"owned-source", "generated-from-owned-source"}:
            resolved = source_path
            resolved_sha256 = self.owners[source_path].source_sha256
        elif resolution == "pinned-source-external":
            resolved_sha256 = digest(self.upstream / resolved)
        else:
            resolved = "-"
            resolved_sha256 = "-"
        return {
            "source_path": source_path,
            "source_unit": source_unit,
            "dependency_unit": dependency_unit,
            "dependency_token": token,
            "resolved_path": resolved,
            "resolved_sha256": resolved_sha256,
            "resolution_kind": resolution,
        }

    def _build_dependency_authority(self) -> None:
        translated = self.source_paths[:188]
        excluded = self.source_paths[188:]
        backend = self.unit_ids[:105]
        shader = self.unit_ids[105:]
        rows: list[dict[str, str]] = []

        shared_pairs: list[tuple[str, str, str]] = []
        for index in range(107):
            source_unit = backend[index % len(backend)]
            dependency_unit = shader[index // len(backend)]
            source_path = translated[index] if index < 103 else excluded[index - 103]
            shared_pairs.append((source_unit, dependency_unit, source_path))
            rows.append(self._dependency_row(
                source_path, source_unit, dependency_unit, f"shared-{index:03}",
                f"generated/shared{index:03}.hpp", "owned-source",
            ))
        for index in range(91):
            source_unit, dependency_unit, source_path = shared_pairs[index % 103]
            rows.append(self._dependency_row(
                source_path, source_unit, dependency_unit, f"shared-extra-{index:03}",
                f"generated/shared-extra{index:03}.hpp", "owned-source",
            ))

        rows.append(self._dependency_row(
            translated[0], self.unit_ids[0], self.unit_ids[1], "load-store-actions-ext",
            "renderer/load_store_actions_ext.hpp", "owned-source",
        ))

        generated_pairs: list[tuple[str, str, str]] = []
        for index in range(351):
            source_unit = shader[index % len(shader)]
            dependency_unit = backend[index // len(shader)]
            source_path = translated[index % len(translated)] if index < 349 else excluded[4 + index - 349]
            generated_pairs.append((source_unit, dependency_unit, source_path))
            rows.append(self._dependency_row(
                source_path, source_unit, dependency_unit, f"generated-{index:03}",
                f"generated/artifact{index:03}.hpp", "generated-from-owned-source",
            ))
        for index in range(89):
            source_unit, dependency_unit, source_path = generated_pairs[index]
            rows.append(self._dependency_row(
                source_path, source_unit, dependency_unit, f"generated-extra-{index:03}",
                f"generated/extra{index:03}.hpp", "generated-from-owned-source",
            ))

        webgpu_sources = [self.unit_ids[0], *self.unit_ids[21:40]]
        wagyu = self.unit_ids[2:21]
        abi_pairs: list[tuple[str, str]] = []
        for index in range(19):
            pair = (webgpu_sources[index], wagyu[index])
            abi_pairs.append(pair)
            rows.append(self._dependency_row(
                translated[20 + index], pair[0], pair[1], f"abi-{index:02}",
                f"webgpu/abi{index:02}.hpp", "owned-source",
            ))
        rows.append(self._dependency_row(
            translated[39], abi_pairs[0][0], abi_pairs[0][1], "abi-extra",
            "webgpu/abi-extra.hpp", "owned-source",
        ))

        for index in range(35):
            rows.append(self._dependency_row(
                translated[40 + index], backend[index], "external:ore",
                f"ore-{index:02}",
                self.ore_external_paths[index % len(self.ore_external_paths)],
                "pinned-source-external",
            ))

        renderer_paths = self.renderer_external_paths
        renderer_pairs: list[tuple[str, str]] = []
        for index in range(66):
            pair = (backend[index // len(renderer_paths)], renderer_paths[index % len(renderer_paths)])
            renderer_pairs.append(pair)
            rows.append(self._dependency_row(
                translated[75 + index], pair[0], "external:renderer",
                f"renderer-{index:02}", pair[1], "pinned-source-external",
            ))
        for index in range(3):
            pair = renderer_pairs[index]
            rows.append(self._dependency_row(
                translated[124 + index], pair[0], "external:renderer",
                f"renderer-extra-{index:02}", pair[1], "pinned-source-external",
            ))

        for index in range(6):
            rows.append(self._dependency_row(
                translated[45 + index], self.unit_ids[45 + index], "external:vma",
                "vk_mem_alloc.h", self.vma_external_path, "pinned-source-external",
            ))

        self.dependencies = rows
        self.generated = [
            {
                "stage": f"synthetic-stage-{index % 7}",
                "artifact_path": f"generated/output{index:03}.hpp",
                "artifact_sha256": digest(
                    self.upstream / f"renderer/src/shaders/generated/output{index:03}.hpp"
                ),
                "retention": "retained",
                "direct_include_count": "0",
            }
            for index in range(520)
        ]
        self.configurations = [{
            "source_path": "renderer/premake5_pls_renderer.lua",
            "ownership_unit": "build:pls_renderer",
            "token": "RIVE_WEBGL",
            "line": "152",
        }]
        self.generated_outputs = {
            relative: checker.UpstreamFileAuthority(
                relative,
                digest(self.upstream / relative),
                logical_lines(self.upstream / relative),
                (self.upstream / relative).stat().st_size,
            )
            for relative in self.generated_output_paths
        }
        external_paths = {
            row["resolved_path"] for row in self.dependencies
            if row["resolution_kind"] == "pinned-source-external"
        }
        self.external_authorities = {
            relative: checker.UpstreamFileAuthority(
                relative,
                digest(self.upstream / relative),
                logical_lines(self.upstream / relative),
                (self.upstream / relative).stat().st_size,
            )
            for relative in external_paths
        }

    def _write_translation_receipts(self) -> None:
        receipt_root = self.repo / "docs/translations"
        for index in range(188):
            source_path = self.source_paths[index]
            source = self.upstream / source_path
            item = self.translations[source_path]
            values = {
                "schema_version": 1,
                "campaign": self.owners[source_path].campaign,
                "ownership_unit": self.owners[source_path].unit,
                "translation_kind": "complete-source-owner",
                "source_path": source_path,
                "source_sha256": digest(source),
                "source_lines": logical_lines(source),
                "source_bytes": source.stat().st_size,
                "target_path": item.target_path,
                "target_sha256": item.target_sha256,
                "source_snapshot_path": item.snapshot_path,
                "source_snapshot_sha256": item.snapshot_sha256,
                "dependency_units": [],
                "dependency_artifacts": (
                    [
                        *[
                            {"path": item.path, "sha256": item.sha256}
                            for item in self.file_artifacts.values()
                        ],
                        *[
                            {"path": item.path, "tree_sha256": item.tree_sha256}
                            for item in self.tree_artifacts.values()
                        ],
                    ]
                    if index == 0 else []
                ),
            }
            write_flat_toml(receipt_root / f"source{index:03}.translation.toml", values)

    def _plan_waves(self) -> list[dict[str, Any]]:
        waves: list[dict[str, Any]] = []
        for order_group in range(7):
            components = [item for item in self.components.values() if item.order_group == order_group]
            units = [self.units[unit_id] for component in components for unit_id in component.units]
            owners = [owner for component in components for owner in self.owners_by_component[component.component_id]]
            translated = [owner for owner in owners if owner.translated]
            excluded = [owner for owner in owners if not owner.translated]
            translated_units = {owner.unit for owner in translated}
            translated_components = {
                self.units[unit_id].component_id for unit_id in translated_units
            }
            waves.append({
                "id": f"g{order_group}",
                "order_group": order_group,
                "source_count": len(owners),
                "translated_source_count": len(translated),
                "excluded_source_count": len(excluded),
                "logical_source_lines": sum(logical_lines(self.upstream / owner.source_path) for owner in owners),
                "translated_logical_source_lines": sum(logical_lines(self.upstream / owner.source_path) for owner in translated),
                "excluded_logical_source_lines": sum(logical_lines(self.upstream / owner.source_path) for owner in excluded),
                "unit_count": len(units),
                "translated_unit_count": sum(unit.unit in translated_units for unit in units),
                "excluded_only_unit_count": sum(unit.unit not in translated_units for unit in units),
                "component_count": len(components),
                "translated_component_count": sum(component.component_id in translated_components for component in components),
                "excluded_only_component_count": sum(component.component_id not in translated_components for component in components),
            })
        return waves

    def _derive_overlays(self) -> dict[str, Any]:
        overlay_shell = [
            {"id": overlay_id, "rule": f"synthetic exact rule for {overlay_id}"}
            for overlay_id in checker.EXPECTED_OVERLAY_IDS
        ]
        original = checker.derive_browser_bridge_authority
        original_support = checker.EXPECTED_BROWSER_SUPPORT_PATHS
        checker.derive_browser_bridge_authority = lambda *_: (
            set(FIXTURE_BROWSER_COMPONENT_IDS), set(FIXTURE_BROWSER_TOKENS)
        )
        checker.EXPECTED_BROWSER_SUPPORT_PATHS = {"support/support07.txt"}
        try:
            return checker.overlay_expectations(
                {"overlay": overlay_shell, "overlay_denominator": 9},
                self.repo,
                self.upstream,
                self.owners,
                self.units,
                self.dependencies,
                self.configurations,
                self.generated,
                self.generated_outputs,
                self.external_authorities,
                self.file_artifacts,
                self.tree_artifacts,
                self.support,
                validate_plan=False,
            )
        finally:
            checker.derive_browser_bridge_authority = original
            checker.EXPECTED_BROWSER_SUPPORT_PATHS = original_support

    def _write_plan(self, overlays: dict[str, Any]) -> None:
        translated_paths = {path for path, owner in self.owners.items() if owner.translated}
        excluded_paths = set(self.owners) - translated_paths
        translated_units = {owner.unit for owner in self.owners.values() if owner.translated}
        translated_components = {self.units[unit].component_id for unit in translated_units}
        source_lines = {path: logical_lines(self.upstream / path) for path in self.owners}
        source_bytes = {(path): (self.upstream / path).stat().st_size for path in self.owners}
        top: dict[str, Any] = {
            "schema_version": 1,
            "upstream_ref": self.upstream_ref,
            "workspace_base_ref": self.workspace_ref,
            "review_kind": "global-source-semantics",
            "review_mode": "independent-read-only-scc-waves",
            "receipt_directory": "docs/backend-port-source-reviews",
            "source_denominator": 200,
            "translated_source_denominator": 188,
            "excluded_source_denominator": 12,
            "logical_source_line_denominator": sum(source_lines.values()),
            "translated_logical_source_line_denominator": sum(source_lines[path] for path in translated_paths),
            "excluded_logical_source_line_denominator": sum(source_lines[path] for path in excluded_paths),
            "source_byte_denominator": sum(source_bytes.values()),
            "translated_source_byte_denominator": sum(source_bytes[path] for path in translated_paths),
            "excluded_source_byte_denominator": sum(source_bytes[path] for path in excluded_paths),
            "unit_denominator": 135,
            "translated_unit_denominator": len(translated_units),
            "excluded_only_unit_denominator": 135 - len(translated_units),
            "component_denominator": 115,
            "translated_component_denominator": len(translated_components),
            "excluded_only_component_denominator": 115 - len(translated_components),
            "component_receipt_denominator": 115,
            "semantic_dependency_rows": sum(row["source_path"] in translated_paths for row in self.dependencies),
            "semantic_configuration_rows": 1,
            "generated_artifacts": 520,
            "retained_generated_artifacts": len(self.generated_outputs),
            "ephemeral_generated_artifacts": 0,
            "retained_generated_logical_lines": sum(
                item.logical_lines for item in self.generated_outputs.values()
            ),
            "retained_generated_bytes": sum(
                item.byte_count for item in self.generated_outputs.values()
            ),
            "pinned_external_dependency_files": len(self.external_authorities),
            "pinned_external_dependency_logical_lines": sum(
                item.logical_lines for item in self.external_authorities.values()
            ),
            "pinned_external_dependency_bytes": sum(
                item.byte_count for item in self.external_authorities.values()
            ),
            "semantic_generated_owner_edges": sum(
                row["source_path"] in translated_paths
                and row["resolution_kind"] == "generated-from-owned-source"
                for row in self.dependencies
            ),
            "translation_snapshot_denominator": 188,
            "translation_dependency_file_denominator": len(self.file_artifacts),
            "translation_dependency_tree_denominator": len(self.tree_artifacts),
            "translation_dependency_tree_file_denominator": len({
                member for tree in self.tree_artifacts.values() for member in tree.members
            }),
            "support_artifact_denominator": 9,
            "support_artifact_logical_line_denominator": 9,
            "overlay_denominator": 9,
            "coverage": checker.EXPECTED_COVERAGE,
            "severity_order": checker.EXPECTED_SEVERITIES,
            "finding_id_rule": "SR-C<component number>-<two-digit nonzero ordinal>",
        }
        result = git(self.repo, "diff", "--name-only", "--diff-filter=ACDMRT", self.closure_base, self.workspace_ref)
        changed = {line for line in result.splitlines() if line}
        categories = {
            "translated_target": {item.target_path for item in self.translations.values()},
            "source_snapshot": {item.snapshot_path for item in self.translations.values()},
            "dependency_tree_member": {
                member for tree in self.tree_artifacts.values() for member in tree.members
            },
            "dependency_file": set(self.file_artifacts),
            "source_review_support": set(self.support),
            "campaign_documentation": {
                path for path in changed if path.startswith("docs/")
            },
            "campaign_tooling": {"tools/backend-port/check_source_review.py"},
            "ownership_only_evidence": set(),
            "explicit_deletion": set(),
        }
        remaining = set(changed)
        category_counts: dict[str, int] = {}
        for category in checker.EXPECTED_CATEGORY_ORDER:
            assigned = remaining & categories[category]
            remaining -= assigned
            category_counts[category] = len(assigned)
        if remaining:
            raise AssertionError(f"unclassified synthetic closure paths: {sorted(remaining)}")
        closure = {
            "base_ref": self.closure_base,
            "head_ref": self.workspace_ref,
            "diff_filter": "ACDMRT",
            "category_order": checker.EXPECTED_CATEGORY_ORDER,
            "campaign_tooling_paths": ["tools/backend-port/check_source_review.py"],
            "ownership_only_paths": [],
            "explicit_deleted_paths": [],
            "changed_path_denominator": len(changed),
            **category_counts,
        }
        path = self.repo / "docs/backend-port-source-review-plan.toml"
        path.parent.mkdir(parents=True, exist_ok=True)
        chunks = ["".join(f"{key} = {toml_value(value)}\n" for key, value in top.items())]
        chunks.append("\n[rules]\n")
        chunks.append("".join(f"{key} = {toml_value(value)}\n" for key, value in checker.EXPECTED_RULES.items()))
        chunks.append("\n[changed_byte_closure]\n")
        chunks.append("".join(f"{key} = {toml_value(value)}\n" for key, value in closure.items()))
        for wave in self._plan_waves():
            chunks.append("\n[[wave]]\n")
            chunks.append("".join(f"{key} = {toml_value(value)}\n" for key, value in wave.items()))
        for overlay_id in checker.EXPECTED_OVERLAY_IDS:
            expectation = overlays[overlay_id]
            table = {
                "id": overlay_id,
                "rule": f"synthetic exact rule for {overlay_id}",
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
            chunks.append("\n[[overlay]]\n")
            chunks.append("".join(f"{key} = {toml_value(value)}\n" for key, value in table.items()))
        path.write_text("".join(chunks), encoding="utf-8")

    def _write_manifest(self) -> None:
        path = self.repo / "docs/backend-port-campaign.toml"
        top = {
            "schema_version": 1,
            "upstream_ref": self.upstream_ref,
            "active_queue": "source-review",
            "preparation_status": "green",
            "ignored_skills": ["implement", "tdd"],
            "translation_receipt_directory": "docs/translations",
            "source_review_plan": "docs/backend-port-source-review-plan.toml",
            "source_review_schema": "docs/backend-port-source-review-schema.md",
            "source_review_receipt_directory": "docs/backend-port-source-reviews",
            "source_review_support_inventory": "docs/backend-port-source-review-support.tsv",
            "source_review_status": "active",
            "ownership_review_plan": "docs/backend-port-ownership-review-plan.toml",
            "ownership_review_schema": "docs/backend-port-ownership-review-schema.md",
            "ownership_review_receipt_directory": "docs/backend-port-ownership-reviews",
            "ownership_review_launch_ref": "pending",
            "ownership_review_status": "active",
            "queue_order": ["audit", "translation-admission", "source-review", "ownership-review"],
            "shared_source_set": [],
            "shared_generic_authority": "docs/shared.toml",
            "shared_ownership_authority": "docs/shared-ownership.toml",
            "source_inventory": "docs/source-inventory.tsv",
            "ownership_inventory": "docs/ownership.tsv",
            "dependency_inventory": "docs/dependencies.tsv",
            "ownership_unit_order": "docs/order.tsv",
            "toolchain_authority": "docs/toolchain.toml",
            "generated_artifact_inventory": "docs/generated.tsv",
            "configuration_inventory": "docs/configurations.tsv",
            "field_profiles": "docs/field-profiles.toml",
            "field_inventory": "docs/fields.tsv",
            "lifecycle_inventory": "docs/lifecycle.tsv",
            "oracle_contract": "docs/oracle.toml",
            "legacy_wgpu_inventory": "docs/legacy-wgpu.tsv",
            "owner_contracts": "docs/owner-contracts.toml",
            "repeatability_inventory": "docs/repeatability.tsv",
        }
        chunks = ["".join(f"{key} = {toml_value(value)}\n" for key, value in top.items())]
        chunks.append("\n[cutover_contract]\n")
        chunks.append("".join(
            f"{key} = {toml_value(value)}\n"
            for key, value in checker.EXPECTED_CUTOVER_CONTRACT.items()
        ))
        chunks.append("\n[denominator]\n")
        chunks.append(
            f"sources = 200\n"
            f"ownership_rows = 200\n"
            f"ownership_units = 135\n"
            f"dependency_edges = {len(self.dependencies)}\n"
            f"configuration_rows = {len(self.configurations)}\n"
            f"generated_artifacts = {len(self.generated)}\n"
        )
        for backend in ("vulkan", "webgpu", "webgl2"):
            chunks.append("\n[[backend]]\n")
            chunks.append(f"id = {toml_value(backend)}\ntranslation_status = \"complete\"\n")
        path.write_text("".join(chunks), encoding="utf-8")

    def _write_frozen_authorities(self) -> None:
        """Write every byte that the launch checkpoint promises to freeze."""
        self._write_translation_receipts()
        ownership_rows = [
            {
                "campaign": owner.campaign,
                "source_path": owner.source_path,
                "source_sha256": owner.source_sha256,
                "ownership_unit": owner.unit,
                "port_disposition": owner.disposition,
                "target_path": owner.target_path,
            }
            for owner in self.owners.values()
        ]
        write_tsv(
            self.repo / "docs/ownership.tsv",
            ["campaign", "source_path", "source_sha256", "ownership_unit", "port_disposition", "target_path"],
            ownership_rows,
        )
        order_rows = [
            {
                "ownership_unit": unit.unit,
                "campaign": unit.campaign,
                "order_group": unit.order_group,
                "component_id": unit.component_id,
                "source_count": unit.source_count,
                "dependency_units": "",
            }
            for unit in self.units.values()
        ]
        write_tsv(
            self.repo / "docs/order.tsv",
            ["ownership_unit", "campaign", "order_group", "component_id", "source_count", "dependency_units"],
            order_rows,
        )
        write_tsv(
            self.repo / "docs/dependencies.tsv",
            ["source_path", "source_unit", "dependency_unit", "dependency_token", "resolved_path", "resolved_sha256", "resolution_kind"],
            self.dependencies,
        )
        write_tsv(
            self.repo / "docs/generated.tsv",
            ["stage", "artifact_path", "artifact_sha256", "retention", "direct_include_count"],
            self.generated,
        )
        write_tsv(
            self.repo / "docs/configurations.tsv",
            ["source_path", "ownership_unit", "token", "line"],
            self.configurations,
        )
        write_tsv(
            self.repo / "docs/backend-port-source-review-support.tsv",
            ["artifact_path", "artifact_sha256", "logical_lines", "artifact_role", "review_overlay", "source_authority", "disposition"],
            [
                {
                    "artifact_path": item.path,
                    "artifact_sha256": item.sha256,
                    "logical_lines": item.logical_lines,
                    "artifact_role": item.artifact_role,
                    "review_overlay": item.review_overlay,
                    "source_authority": item.source_authority,
                    "disposition": item.disposition,
                }
                for item in self.support.values()
            ],
        )
        (self.repo / "docs/shared.toml").write_text(
            "[mechanical_translation_workflow]\n"
            "source_reviewer_role = \"independent-source-reviewer\"\n",
            encoding="utf-8",
        )
        dummy_files = {
            "docs/shared-ownership.toml": "schema_version = 1\n",
            "docs/source-inventory.tsv": "source_path\n",
            "docs/toolchain.toml": (
                "schema_version = 1\n"
                "shader_directory = \"renderer/src/shaders\"\n"
            ),
            "docs/field-profiles.toml": "schema_version = 1\n",
            "docs/fields.tsv": "field\n",
            "docs/lifecycle.tsv": "event\n",
            "docs/oracle.toml": "schema_version = 1\n",
            "docs/legacy-wgpu.tsv": "path\n",
            "docs/owner-contracts.toml": "schema_version = 1\n",
            "docs/repeatability.tsv": "entry\n",
            "docs/backend-port-source-review-schema.md": "# Synthetic source-review schema\n",
        }
        for relative, contents in dummy_files.items():
            path = self.repo / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(contents, encoding="utf-8")
        self._write_manifest()

    def _write_component_receipts(self) -> None:
        root = self.repo / "docs/backend-port-source-reviews"
        for component_id, component in self.components.items():
            owners = self.owners_by_component[component_id]
            sources = [
                {
                    "path": owner.source_path,
                    "sha256": owner.source_sha256,
                    "citation": f"source:{owner.source_path}:1-{logical_lines(self.upstream / owner.source_path)}",
                    "disposition": owner.disposition,
                }
                for owner in owners
            ]
            targets = [
                {
                    "path": self.translations[owner.source_path].target_path,
                    "sha256": self.translations[owner.source_path].target_sha256,
                    "citation": (
                        f"target:{self.translations[owner.source_path].target_path}:1-"
                        f"{logical_lines(self.repo / self.translations[owner.source_path].target_path)}"
                    ),
                }
                for owner in owners if owner.translated
            ]
            write_flat_toml(root / f"{component_id}.source-review.toml", {
                "schema_version": 1,
                "component_id": component_id,
                "units": list(component.units),
                "receipt_kind": "source-review-component",
                "upstream_ref": self.upstream_ref,
                "workspace_base_ref": self.workspace_ref,
                "role": "independent-source-reviewer",
                "review_run_id": f"synthetic-run-{component_id}",
                "review_wave": component.review_wave,
                "coverage": checker.EXPECTED_COVERAGE,
                "sources": sources,
                "targets": targets,
                "findings": [],
                "open_findings": 0,
            })

    def _write_support_receipt(self) -> Path:
        path = self.repo / "docs/backend-port-source-reviews/support.source-review.toml"
        artifacts = [
            {
                "path": artifact.path,
                "sha256": artifact.sha256,
                "logical_lines": artifact.logical_lines,
                "citation": f"support:{artifact.path}:1-{artifact.logical_lines}",
                "artifact_role": artifact.artifact_role,
                "review_overlay": artifact.review_overlay,
                "source_authority": artifact.source_authority,
                "disposition": artifact.disposition,
            }
            for artifact in self.support.values()
        ]
        write_flat_toml(path, {
            "schema_version": 1,
            "receipt_kind": "source-review-support",
            "upstream_ref": self.upstream_ref,
            "workspace_base_ref": self.workspace_ref,
            "role": "independent-source-reviewer",
            "review_run_id": "synthetic-support-run",
            "review_wave": "support",
            "coverage": checker.EXPECTED_COVERAGE,
            "artifacts": artifacts,
            "findings": [],
            "open_findings": 0,
        })
        return path

    def _write_overlay_receipt(self, overlays: dict[str, Any], support_receipt: Path) -> None:
        records: list[dict[str, Any]] = []
        for overlay_id in checker.EXPECTED_OVERLAY_IDS:
            expectation = overlays[overlay_id]
            component_receipts = []
            for component_id in expectation.component_ids:
                relative = f"docs/backend-port-source-reviews/{component_id}.source-review.toml"
                component_receipts.append({
                    "id": component_id,
                    "path": relative,
                    "sha256": digest(self.repo / relative),
                })
            support_receipts = []
            if expectation.support_paths:
                support_receipts.append({
                    "id": "support",
                    "path": "docs/backend-port-source-reviews/support.source-review.toml",
                    "sha256": digest(support_receipt),
                })
            records.append({
                "id": overlay_id,
                "authority_record_count": expectation.authority_record_count,
                "authority_sha256": expectation.authority_sha256,
                "component_ids": list(expectation.component_ids),
                "support_paths": list(expectation.support_paths),
                "tree_bindings": [
                    {"path": path, "tree_sha256": tree_sha}
                    for path, tree_sha in expectation.tree_bindings
                ],
                "external_bindings": [
                    {
                        "path": path,
                        "sha256": self.external_authorities[path].sha256,
                        "logical_lines": self.external_authorities[path].logical_lines,
                    }
                    for path in expectation.external_paths
                ],
                "generated_bindings": [
                    {
                        "path": path,
                        "sha256": self.generated_outputs[path].sha256,
                        "logical_lines": self.generated_outputs[path].logical_lines,
                    }
                    for path in expectation.generated_paths
                ],
                "authority_keys": list(expectation.authority_keys),
                "component_receipts": component_receipts,
                "support_receipts": support_receipts,
                "attestation": "reviewed-complete-derived-overlay-authority",
            })
        write_flat_toml(self.repo / "docs/backend-port-source-reviews/overlays.source-review.toml", {
            "schema_version": 1,
            "receipt_kind": "source-review-overlays",
            "upstream_ref": self.upstream_ref,
            "workspace_base_ref": self.workspace_ref,
            "role": "independent-source-reviewer",
            "review_run_id": "synthetic-overlay-run",
            "review_wave": "overlays",
            "coverage": checker.EXPECTED_OVERLAY_COVERAGE,
            "overlays": records,
            "findings": [],
            "open_findings": 0,
        })

    def _build_review_authority(self) -> None:
        self._write_translation_receipts()
        ownership_rows = [
            {
                "campaign": owner.campaign,
                "source_path": owner.source_path,
                "source_sha256": owner.source_sha256,
                "ownership_unit": owner.unit,
                "port_disposition": owner.disposition,
                "target_path": owner.target_path,
            }
            for owner in self.owners.values()
        ]
        write_tsv(
            self.repo / "docs/ownership.tsv",
            ["campaign", "source_path", "source_sha256", "ownership_unit", "port_disposition", "target_path"],
            ownership_rows,
        )
        order_rows = [
            {
                "ownership_unit": unit.unit,
                "campaign": unit.campaign,
                "order_group": unit.order_group,
                "component_id": unit.component_id,
                "source_count": unit.source_count,
                "dependency_units": "",
            }
            for unit in self.units.values()
        ]
        write_tsv(
            self.repo / "docs/order.tsv",
            ["ownership_unit", "campaign", "order_group", "component_id", "source_count", "dependency_units"],
            order_rows,
        )
        write_tsv(
            self.repo / "docs/dependencies.tsv",
            ["source_path", "source_unit", "dependency_unit", "dependency_token", "resolved_path", "resolved_sha256", "resolution_kind"],
            self.dependencies,
        )
        write_tsv(
            self.repo / "docs/generated.tsv",
            ["stage", "artifact_path", "artifact_sha256", "retention", "direct_include_count"],
            self.generated,
        )
        write_tsv(
            self.repo / "docs/configurations.tsv",
            ["source_path", "ownership_unit", "token", "line"],
            self.configurations,
        )
        write_tsv(
            self.repo / "docs/backend-port-source-review-support.tsv",
            ["artifact_path", "artifact_sha256", "logical_lines", "artifact_role", "review_overlay", "source_authority", "disposition"],
            [
                {
                    "artifact_path": item.path,
                    "artifact_sha256": item.sha256,
                    "logical_lines": item.logical_lines,
                    "artifact_role": item.artifact_role,
                    "review_overlay": item.review_overlay,
                    "source_authority": item.source_authority,
                    "disposition": item.disposition,
                }
                for item in self.support.values()
            ],
        )
        write_flat_toml(self.repo / "docs/shared.toml", {
            "mechanical_translation_workflow": {
                "source_reviewer_role": "independent-source-reviewer"
            }
        })
        # The shared authority uses a real table, not an inline top-level value.
        (self.repo / "docs/shared.toml").write_text(
            "[mechanical_translation_workflow]\n"
            "source_reviewer_role = \"independent-source-reviewer\"\n",
            encoding="utf-8",
        )
        overlays = self._derive_overlays()
        self._write_plan(overlays)
        self._write_manifest()
        self._write_component_receipts()
        support_receipt = self._write_support_receipt()
        self._write_overlay_receipt(overlays, support_receipt)


class GeneratedOutputAuthorityTests(unittest.TestCase):
    def make_roots(self) -> tuple[tempfile.TemporaryDirectory[str], Path, Path]:
        temporary = tempfile.TemporaryDirectory(prefix="generated-authority-")
        root = Path(temporary.name).resolve()
        repo = root / "repo"
        upstream = root / "upstream"
        repo.mkdir()
        upstream.mkdir()
        toolchain = repo / "docs/toolchain.toml"
        toolchain.parent.mkdir()
        toolchain.write_text(
            "schema_version = 1\n"
            "shader_directory = \"renderer/src/shaders\"\n",
            encoding="utf-8",
        )
        return temporary, repo, upstream

    @staticmethod
    def retained_row(header: Path) -> dict[str, str]:
        return {
            "stage": "synthetic-final-header",
            "artifact_path": "intermediate.hpp",
            "artifact_sha256": digest(header),
            "retention": "retained",
            "direct_include_count": "1",
        }

    @staticmethod
    def ephemeral_row() -> dict[str, str]:
        return {
            "stage": "synthetic-intermediate",
            "artifact_path": "intermediate.tmp",
            "artifact_sha256": "-",
            "retention": "ephemeral-final-header-retained",
            "direct_include_count": "0",
        }

    @staticmethod
    def plan(header: Path) -> dict[str, int]:
        return {
            "generated_artifacts": 2,
            "retained_generated_artifacts": 1,
            "ephemeral_generated_artifacts": 1,
            "retained_generated_logical_lines": logical_lines(header),
            "retained_generated_bytes": header.stat().st_size,
        }

    def test_valid_retained_header_with_absent_ephemeral_intermediate(self) -> None:
        temporary, repo, upstream = self.make_roots()
        self.addCleanup(temporary.cleanup)
        shader_root = upstream / "renderer/src/shaders"
        shader_root.mkdir(parents=True)
        header = shader_root / "intermediate.hpp"
        header.write_text("retained final header\n", encoding="utf-8")
        generated = [self.retained_row(header), self.ephemeral_row()]
        outputs = checker.load_generated_outputs(
            repo,
            upstream,
            {"toolchain_authority": "docs/toolchain.toml"},
            generated,
            self.plan(header),
        )
        self.assertEqual(set(outputs), {"renderer/src/shaders/intermediate.hpp"})

    def test_present_ephemeral_intermediate_is_rejected(self) -> None:
        temporary, repo, upstream = self.make_roots()
        self.addCleanup(temporary.cleanup)
        shader_root = upstream / "renderer/src/shaders"
        shader_root.mkdir(parents=True)
        header = shader_root / "intermediate.hpp"
        header.write_text("retained final header\n", encoding="utf-8")
        (shader_root / "intermediate.tmp").write_text(
            "unexpected intermediate\n", encoding="utf-8"
        )
        with self.assertRaisesRegex(
            checker.ReviewError,
            "ephemeral generated output unexpectedly exists",
        ):
            checker.load_generated_outputs(
                repo,
                upstream,
                {"toolchain_authority": "docs/toolchain.toml"},
                [self.retained_row(header), self.ephemeral_row()],
                self.plan(header),
            )

    def test_ephemeral_intermediate_without_retained_header_row_is_rejected(self) -> None:
        temporary, repo, upstream = self.make_roots()
        self.addCleanup(temporary.cleanup)
        (upstream / "renderer/src/shaders").mkdir(parents=True)
        with self.assertRaisesRegex(
            checker.ReviewError,
            "ephemeral generated output has no retained header",
        ):
            checker.load_generated_outputs(
                repo,
                upstream,
                {"toolchain_authority": "docs/toolchain.toml"},
                [self.ephemeral_row()],
                {
                    "generated_artifacts": 1,
                    "retained_generated_artifacts": 0,
                    "ephemeral_generated_artifacts": 1,
                    "retained_generated_logical_lines": 0,
                    "retained_generated_bytes": 0,
                },
            )


class SourceReviewCheckerTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls._base_tmp = tempfile.TemporaryDirectory(prefix="source-review-base-")
        cls.base = SyntheticCampaign(Path(cls._base_tmp.name))

    @classmethod
    def tearDownClass(cls) -> None:
        cls._base_tmp.cleanup()

    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory(prefix="source-review-case-")
        case_root = Path(self._tmp.name)
        self.repo = case_root / "repo"
        self.upstream = case_root / "upstream"
        shutil.copytree(self.base.repo, self.repo)
        shutil.copytree(self.base.upstream, self.upstream)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def run_checker(
        self,
        *extra: str,
        plan_sha256: str | None = None,
        support_inventory_sha256: str | None = None,
        schema_sha256: str | None = None,
    ) -> subprocess.CompletedProcess[str]:
        # The production checker deliberately pins literal production commits
        # and authority hashes.  A synthetic Git repository cannot manufacture
        # those SHA-1 commit IDs, so the test process patches only those literal
        # anchors.  All validation code and all derived contracts remain the
        # production module loaded directly from CHECKER.
        runner = r'''
import importlib.util
import sys
spec = importlib.util.spec_from_file_location("synthetic_source_review_checker", sys.argv[1])
module = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = module
spec.loader.exec_module(module)
module.EXPECTED_UPSTREAM_REF = sys.argv[2]
module.EXPECTED_WORKSPACE_BASE_REF = sys.argv[3]
module.EXPECTED_ADMISSION_BASE_REF = sys.argv[4]
module.EXPECTED_PLAN_SHA256 = sys.argv[5]
module.EXPECTED_SUPPORT_INVENTORY_SHA256 = sys.argv[6]
module.EXPECTED_SCHEMA_SHA256 = sys.argv[7]
module.derive_browser_bridge_authority = lambda *_: (
    {"component-000", "component-001"},
    {
        "browser-fixture:explicit-webgpu-selection",
        "browser-fixture:explicit-webgl2-selection",
        "browser-fixture:no-automatic-fallback",
    },
)
module.EXPECTED_BROWSER_SUPPORT_PATHS = {"support/support07.txt"}
sys.argv = [sys.argv[1], *sys.argv[8:]]
try:
    raise SystemExit(module.main())
except (OSError, KeyError, TypeError, ValueError, module.csv.Error,
        module.tomllib.TOMLDecodeError) as error:
    print(f"backend source-review failure: {error}", file=sys.stderr)
    raise SystemExit(1)
'''
        return subprocess.run(
            [
                sys.executable,
                "-c", runner,
                str(CHECKER),
                self.base.upstream_ref,
                self.base.workspace_ref,
                self.base.closure_base,
                plan_sha256 or self.base.plan_sha256,
                support_inventory_sha256 or self.base.support_inventory_sha256,
                schema_sha256 or self.base.schema_sha256,
                "--repo-root", str(self.repo),
                "--upstream-root", str(self.upstream),
                "--manifest", "docs/backend-port-campaign.toml",
                *extra,
            ],
            text=True,
            capture_output=True,
            check=False,
        )

    def assert_success(self, result: subprocess.CompletedProcess[str], fragment: str) -> None:
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn(fragment, result.stdout)

    def assert_failure(self, result: subprocess.CompletedProcess[str], fragment: str) -> None:
        self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn(fragment, result.stderr)

    def receipt(self, component: int) -> Path:
        return self.repo / f"docs/backend-port-source-reviews/component-{component:03}.source-review.toml"

    def add_overlay_finding(
        self,
        overlay_id: str,
        finding_id: str,
        citations: list[str],
    ) -> None:
        path = self.repo / "docs/backend-port-source-reviews/overlays.source-review.toml"
        finding = toml_value([{
            "id": finding_id,
            "overlay_id": overlay_id,
            "severity": "P1",
            "summary": "synthetic overlay finding",
            "citations": citations,
        }])
        self.replace(
            path,
            "findings = []\nopen_findings = 0",
            f"findings = {finding}\nopen_findings = 1",
        )

    @staticmethod
    def replace(path: Path, old: str, new: str, count: int = 1) -> None:
        text = path.read_text(encoding="utf-8")
        if old not in text:
            raise AssertionError(f"mutation anchor not found in {path}: {old[:80]!r}")
        path.write_text(text.replace(old, new, count), encoding="utf-8")

    def test_admission_replays_the_complete_frozen_authority(self) -> None:
        result = self.run_checker("--admission")
        self.assert_success(result, "components=115, units=135, sources=200, targets=188")
        self.assertIn("support=9, overlays=9", result.stdout)
        self.assertIn("external=35, generated_outputs=520", result.stdout)

    def test_global_green_accepts_all_canonical_receipts(self) -> None:
        result = self.run_checker()
        self.assert_success(result, "structure=complete, audit=green")
        self.assertIn("components=115/115", result.stdout)
        self.assertIn("open_findings=0", result.stdout)

    def test_partial_component_does_not_require_a_same_wave_peer(self) -> None:
        self.receipt(0).unlink()
        result = self.run_checker("--receipt", "docs/backend-port-source-reviews/component-007.source-review.toml")
        self.assert_success(result, "component=component-007, wave=g0")

    def test_partial_component_requires_every_prior_wave(self) -> None:
        self.receipt(0).unlink()
        result = self.run_checker("--receipt", "docs/backend-port-source-reviews/component-001.source-review.toml")
        self.assert_failure(result, "missing source-review component receipt")

    def test_partial_support_requires_all_components(self) -> None:
        self.receipt(114).unlink()
        result = self.run_checker("--receipt", "docs/backend-port-source-reviews/support.source-review.toml")
        self.assert_failure(result, "missing source-review component receipt")

    def test_partial_overlay_accepts_complete_prerequisites(self) -> None:
        result = self.run_checker("--receipt", "docs/backend-port-source-reviews/overlays.source-review.toml")
        self.assert_success(result, "overlay receipt complete: overlays=9")

    def test_recursive_extra_receipt_file_is_rejected(self) -> None:
        extra = self.repo / "docs/backend-port-source-reviews/nested/unaccounted.txt"
        extra.parent.mkdir()
        extra.write_text("not a receipt\n", encoding="utf-8")
        self.assert_failure(self.run_checker(), "source-review receipt set drift: 118/117")

    def test_later_queue_requires_source_review_complete(self) -> None:
        manifest = self.repo / "docs/backend-port-campaign.toml"
        self.replace(manifest, 'active_queue = "source-review"', 'active_queue = "ownership-review"')
        self.assert_failure(
            self.run_checker(),
            "campaign advanced past an incomplete source review",
        )

    def test_later_queue_accepts_structurally_complete_source_review(self) -> None:
        manifest = self.repo / "docs/backend-port-campaign.toml"
        self.replace(manifest, 'active_queue = "source-review"', 'active_queue = "ownership-review"')
        self.replace(manifest, 'source_review_status = "active"', 'source_review_status = "complete"')
        result = self.run_checker()
        self.assert_success(result, "structure=complete, audit=green")
        self.assertIn("queue=ownership-review", result.stdout)

    def test_structural_red_is_a_successful_complete_audit(self) -> None:
        component = self.receipt(0)
        old_hash = digest(component)
        finding = toml_value([{
            "id": "SR-C000-01",
            "severity": "P1",
            "summary": "synthetic semantic mismatch",
            "citations": ["source:sources/source000.cpp:1-1"],
        }])
        self.replace(component, "findings = []\nopen_findings = 0", f"findings = {finding}\nopen_findings = 1")
        new_hash = digest(component)
        overlay = self.repo / "docs/backend-port-source-reviews/overlays.source-review.toml"
        text = overlay.read_text(encoding="utf-8")
        self.assertIn(old_hash, text)
        overlay.write_text(text.replace(old_hash, new_hash), encoding="utf-8")
        result = self.run_checker()
        self.assert_success(result, "structure=complete, audit=red")
        self.assertIn("open_findings=1", result.stdout)

    def test_overlay_artifact_and_tree_citations_are_valid_in_scope(self) -> None:
        self.add_overlay_finding(
            "generated-authority",
            "SR-OVL-03-01",
            [
                "artifact:vendor/generated/tool.hpp:1-1",
                "tree:vendor/generated-tree/member.txt:1-1",
            ],
        )
        result = self.run_checker()
        self.assert_success(result, "structure=complete, audit=red")
        self.assertIn("open_findings=1", result.stdout)

    def test_overlay_artifact_citation_outside_authority_is_rejected(self) -> None:
        self.add_overlay_finding(
            "generated-authority",
            "SR-OVL-03-01",
            ["artifact:vendor/vk-mem-0.5.0/vk_mem_alloc.h:1-1"],
        )
        self.assert_failure(self.run_checker(), "overlay finding cites outside its authority")

    def test_overlay_tree_citation_outside_authority_is_rejected(self) -> None:
        self.add_overlay_finding(
            "generated-authority",
            "SR-OVL-03-01",
            ["tree:vendor/Vulkan-Headers/include/vulkan.h:1-1"],
        )
        self.assert_failure(self.run_checker(), "overlay finding cites outside its authority")

    def test_overlay_external_citation_is_valid_in_scope(self) -> None:
        self.add_overlay_finding(
            "shared-ore-contracts",
            "SR-OVL-05-01",
            ["external:renderer/include/rive/renderer/ore/external00.hpp:1-1"],
        )
        result = self.run_checker()
        self.assert_success(result, "structure=complete, audit=red")

    def test_overlay_external_citation_outside_authority_is_rejected(self) -> None:
        self.add_overlay_finding(
            "shared-ore-contracts",
            "SR-OVL-05-01",
            ["external:renderer/include/rive/renderer/shared/external00.hpp:1-1"],
        )
        self.assert_failure(self.run_checker(), "overlay finding cites outside its authority")

    def test_upstream_external_authority_byte_mutation_is_rejected(self) -> None:
        path = self.upstream / "renderer/include/rive/renderer/ore/external00.hpp"
        path.write_text("mutated external authority\n", encoding="utf-8")
        self.assert_failure(
            self.run_checker("--admission"),
            "pinned external dependency hash drift",
        )

    def test_overlay_generated_output_citation_is_valid_in_scope(self) -> None:
        self.add_overlay_finding(
            "generated-authority",
            "SR-OVL-03-01",
            ["generated:renderer/src/shaders/generated/output000.hpp:1-1"],
        )
        result = self.run_checker()
        self.assert_success(result, "structure=complete, audit=red")

    def test_overlay_generated_output_citation_outside_authority_is_rejected(self) -> None:
        self.add_overlay_finding(
            "shared-ore-contracts",
            "SR-OVL-05-01",
            ["generated:renderer/src/shaders/generated/output000.hpp:1-1"],
        )
        self.assert_failure(self.run_checker(), "overlay finding cites outside its authority")

    def test_upstream_generated_output_byte_mutation_is_rejected(self) -> None:
        path = self.upstream / "renderer/src/shaders/generated/output000.hpp"
        path.write_text("mutated generated output\n", encoding="utf-8")
        self.assert_failure(
            self.run_checker("--admission"),
            "retained generated output hash drift",
        )

    def test_mutated_source_hash_is_rejected(self) -> None:
        path = self.receipt(100)
        correct = digest(self.upstream / "sources/source100.cpp")
        self.replace(path, correct, "0" * 64)
        self.assert_failure(self.run_checker(), "source evidence drift")

    def test_mutated_source_path_is_rejected(self) -> None:
        path = self.receipt(101)
        self.replace(path, "sources/source101.cpp", "sources/renamed101.cpp")
        self.assert_failure(self.run_checker(), "source membership drift")

    def test_missing_component_member_is_rejected(self) -> None:
        path = self.receipt(100)
        text = path.read_text(encoding="utf-8")
        mutated, replacements = re.subn(r"(?m)^sources = \[.*\]$", "sources = []", text, count=1)
        self.assertEqual(replacements, 1)
        path.write_text(mutated, encoding="utf-8")
        self.assert_failure(self.run_checker(), "source membership drift")

    def test_mutated_plan_denominator_is_rejected(self) -> None:
        path = self.repo / "docs/backend-port-source-review-plan.toml"
        self.replace(path, "source_denominator = 200", "source_denominator = 201")
        self.assert_failure(
            self.run_checker("--admission", plan_sha256=digest(path)),
            "source-review plan source_denominator drift",
        )

    def test_plan_byte_mutation_is_rejected_by_launch_hash(self) -> None:
        path = self.repo / "docs/backend-port-source-review-plan.toml"
        path.write_text(path.read_text(encoding="utf-8") + "\n", encoding="utf-8")
        self.assert_failure(
            self.run_checker("--admission"),
            "source-review plan bytes drifted from launch authority",
        )

    def test_support_inventory_byte_mutation_is_rejected_by_launch_hash(self) -> None:
        path = self.repo / "docs/backend-port-source-review-support.tsv"
        path.write_text(path.read_text(encoding="utf-8") + "\n", encoding="utf-8")
        self.assert_failure(
            self.run_checker("--admission"),
            "source-review support inventory bytes drifted from launch authority",
        )

    def test_paired_target_and_translation_receipt_drift_is_rejected_by_base(self) -> None:
        target = self.repo / "target/target000.rs"
        old_target_hash = digest(target)
        target.write_text("coordinated-but-post-base-target-change\n", encoding="utf-8")
        receipt = self.repo / "docs/translations/source000.translation.toml"
        self.replace(receipt, old_target_hash, digest(target))
        self.assert_failure(
            self.run_checker("--admission"),
            "review bytes drifted from workspace_base_ref",
        )

    def test_untracked_replacement_of_frozen_target_is_rejected(self) -> None:
        git(self.repo, "rm", "--cached", "target/target001.rs")
        self.assert_failure(
            self.run_checker("--admission"),
            "frozen review-byte scope is not tracked: target/target001.rs",
        )

    def test_coordinated_support_path_swap_is_rejected(self) -> None:
        support = self.repo / "support/support00.txt"
        swapped = self.repo / "support/swapped00.txt"
        support.rename(swapped)
        support.symlink_to(swapped.name)
        self.assert_failure(
            self.run_checker("--admission"),
            "support artifact is not tracked: support/support00.txt",
        )

    def test_mutated_cutover_contract_is_rejected(self) -> None:
        path = self.repo / "docs/backend-port-campaign.toml"
        self.replace(
            path,
            'editor_selection = "explicit-user-selected-no-automatic-fallback"',
            'editor_selection = "automatic-fallback"',
        )
        self.assert_failure(self.run_checker("--admission"), "renderer cutover contract drift")

    def test_mutated_ignored_skill_contract_is_rejected(self) -> None:
        path = self.repo / "docs/backend-port-campaign.toml"
        self.replace(path, 'ignored_skills = ["implement", "tdd"]', 'ignored_skills = ["implement"]')
        self.assert_failure(self.run_checker("--admission"), "ignored-skill contract drift")

    def test_invented_top_level_manifest_key_is_rejected(self) -> None:
        path = self.repo / "docs/backend-port-campaign.toml"
        self.replace(
            path,
            "schema_version = 1\n",
            "schema_version = 1\ninvented_authority = \"not-allowed\"\n",
        )
        self.assert_failure(self.run_checker("--admission"), "campaign manifest invents keys")

    def test_source_review_schema_path_swap_is_rejected(self) -> None:
        path = self.repo / "docs/backend-port-campaign.toml"
        self.replace(
            path,
            'source_review_schema = "docs/backend-port-source-review-schema.md"',
            'source_review_schema = "docs/swapped-source-review-schema.md"',
        )
        self.assert_failure(
            self.run_checker("--admission"),
            "source-review authority path drift: source_review_schema",
        )

    def test_source_review_schema_byte_mutation_is_rejected(self) -> None:
        path = self.repo / "docs/backend-port-source-review-schema.md"
        path.write_text(path.read_text(encoding="utf-8") + "mutated schema\n", encoding="utf-8")
        self.assert_failure(
            self.run_checker("--admission"),
            "source-review schema bytes drifted from launch authority",
        )

    def test_mutated_overlay_authority_key_is_rejected(self) -> None:
        path = self.repo / "docs/backend-port-source-reviews/overlays.source-review.toml"
        self.replace(path, "component:component-000", "component:component-999")
        self.assert_failure(self.run_checker(), "overlay receipt evidence drift")

    def test_mutated_component_prerequisite_hash_is_rejected(self) -> None:
        component = self.receipt(0)
        component.write_text(component.read_text(encoding="utf-8") + "\n", encoding="utf-8")
        self.assert_failure(
            self.run_checker("--receipt", "docs/backend-port-source-reviews/overlays.source-review.toml"),
            "overlay receipt evidence drift",
        )

    def test_mutated_support_prerequisite_hash_is_rejected(self) -> None:
        support = self.repo / "docs/backend-port-source-reviews/support.source-review.toml"
        old_hash = digest(support)
        overlay = self.repo / "docs/backend-port-source-reviews/overlays.source-review.toml"
        self.assertIn(old_hash, overlay.read_text(encoding="utf-8"))
        self.replace(overlay, old_hash, "f" * 64)
        self.assert_failure(self.run_checker(), "overlay receipt evidence drift")

    def test_boolean_open_findings_is_rejected(self) -> None:
        path = self.receipt(100)
        self.replace(path, "open_findings = 0", "open_findings = true")
        self.assert_failure(
            self.run_checker("--receipt", "docs/backend-port-source-reviews/component-100.source-review.toml"),
            "open_findings must be an integer",
        )

    def test_zero_ordinal_finding_id_is_rejected(self) -> None:
        path = self.receipt(0)
        finding = toml_value([{
            "id": "SR-C000-00",
            "severity": "P2",
            "summary": "invalid zero ordinal",
            "citations": ["source:sources/source000.cpp:1-1"],
        }])
        self.replace(path, "findings = []\nopen_findings = 0", f"findings = {finding}\nopen_findings = 1")
        self.assert_failure(
            self.run_checker("--receipt", "docs/backend-port-source-reviews/component-000.source-review.toml"),
            "unstable finding ID",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
