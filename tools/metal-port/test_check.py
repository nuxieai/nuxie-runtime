from __future__ import annotations

import collections
import copy
import csv
import importlib.util
import os
import pathlib
import subprocess
import tempfile
import tomllib
import unittest
from unittest import mock


MODULE_PATH = pathlib.Path(__file__).with_name("check.py")
SPEC = importlib.util.spec_from_file_location("metal_port_check", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


def configured_upstream() -> pathlib.Path:
    configured = os.environ.get("RIVE_RUNTIME_DIR")
    if not configured:
        raise unittest.SkipTest("set RIVE_RUNTIME_DIR to run pinned upstream checks")
    return pathlib.Path(configured).resolve()


def translation_manifest_fixture() -> dict[str, object]:
    upstream_ref = "a" * 40
    manifest = {
        "upstream_ref": upstream_ref,
        "source": [
            {
                "upstream": "renderer/include/rive/renderer/ore/ore_types.hpp",
                "lane": "ore-metal",
                "status": "pending",
                "rust_modules": ["crates/nuxie-ore-metal/src/mechanical_port/source/renderer/include/rive/renderer/ore/ore_types_hpp.rs"],
            },
            {
                "upstream": "renderer/include/rive/renderer/ore/ore_rstb_entry_container.hpp",
                "lane": "ore-metal",
                "status": "pending",
                "rust_modules": ["crates/nuxie-ore-metal/src/mechanical_port/source/renderer/include/rive/renderer/ore/ore_rstb_entry_container_hpp.rs"],
            },
            {
                "upstream": "renderer/include/rive/renderer/ore/ore_binding_map.hpp",
                "lane": "ore-metal",
                "status": "pending",
                "rust_modules": ["crates/nuxie-ore-metal/src/mechanical_port/source/renderer/include/rive/renderer/ore/ore_binding_map_hpp.rs"],
            },
            {
                "upstream": "renderer/src/ore/ore_binding_map.cpp",
                "lane": "ore-metal",
                "status": "pending",
                "rust_modules": ["crates/nuxie-ore-metal/src/mechanical_port/source/renderer/src/ore/ore_binding_map_cpp.rs"],
            },
            {
                "upstream": "renderer/src/ore/metal/ore_context_metal.mm",
                "lane": "ore-metal",
                "status": "pending",
                "rust_modules": ["crates/nuxie-ore-metal/src/metal/context.rs"],
            },
        ],
        "translation_unit": [
            {
                "id": "ore-types",
                "phase": "trial",
                "sources": ["renderer/include/rive/renderer/ore/ore_types.hpp"],
                "source_dependencies": [],
                "dispatch_prerequisites": [],
                "rust_targets": ["crates/nuxie-ore-metal/src/mechanical_port/source/renderer/include/rive/renderer/ore/ore_types_hpp.rs"],
                "worker_role": "luna-extra-high",
                "worker_claim": "trial-ore-types",
                "source_reviewer_role": "sol-high",
                "ownership_reviewer_role": "sol-high",
                "fixer_role": "sol-high",
                "base_ref": upstream_ref,
                "status": "ready",
                "requires_lifetime_rows": True,
            },
            {
                "id": "ore-rstb-container",
                "phase": "trial",
                "sources": [
                    "renderer/include/rive/renderer/ore/ore_rstb_entry_container.hpp"
                ],
                "source_dependencies": [],
                "dispatch_prerequisites": [],
                "rust_targets": [
                    "crates/nuxie-ore-metal/src/mechanical_port/source/renderer/include/rive/renderer/ore/ore_rstb_entry_container_hpp.rs"
                ],
                "worker_role": "luna-extra-high",
                "worker_claim": "trial-ore-rstb-container",
                "source_reviewer_role": "sol-high",
                "ownership_reviewer_role": "sol-high",
                "fixer_role": "sol-high",
                "base_ref": upstream_ref,
                "status": "pending",
                "requires_lifetime_rows": True,
            },
            {
                "id": "ore-binding-map",
                "phase": "trial",
                "sources": [
                    "renderer/include/rive/renderer/ore/ore_binding_map.hpp",
                    "renderer/src/ore/ore_binding_map.cpp",
                ],
                "source_dependencies": [],
                "dispatch_prerequisites": [],
                "rust_targets": [
                    "crates/nuxie-ore-metal/src/mechanical_port/source/renderer/include/rive/renderer/ore/ore_binding_map_hpp.rs",
                    "crates/nuxie-ore-metal/src/mechanical_port/source/renderer/src/ore/ore_binding_map_cpp.rs",
                ],
                "worker_role": "luna-extra-high",
                "worker_claim": "trial-ore-binding-map",
                "source_reviewer_role": "sol-high",
                "ownership_reviewer_role": "sol-high",
                "fixer_role": "sol-high",
                "base_ref": upstream_ref,
                "status": "pending",
                "requires_lifetime_rows": True,
            },
            {
                "id": "ore-context",
                "phase": "bulk",
                "sources": ["renderer/src/ore/metal/ore_context_metal.mm"],
                "source_dependencies": ["ore-types"],
                "dispatch_prerequisites": ["ore-types"],
                "rust_targets": ["crates/nuxie-ore-metal/src/metal/context.rs"],
                "worker_role": "sol-high",
                "worker_claim": "unclaimed",
                "source_reviewer_role": "sol-high",
                "ownership_reviewer_role": "sol-high",
                "fixer_role": "sol-high",
                "base_ref": upstream_ref,
                "status": "pending",
                "requires_lifetime_rows": True,
            },
        ],
    }
    for unit in manifest["translation_unit"]:
        unit["lifetime_authority"] = "ore-port-lifetimes"
        unit["translation_receipt"] = "pending"
        unit["source_review_receipt"] = "pending"
        unit["ownership_review_receipt"] = "pending"
        unit["fix_receipt"] = "pending"
        unit["compile_receipt"] = "pending"
        unit["verification_receipt"] = "pending"
        unit["open_findings"] = 0
    return manifest


class MetalPortCheckTests(unittest.TestCase):
    def test_source_card_must_name_its_canonical_translation_owner(self) -> None:
        source = "renderer/src/metal/example.mm"
        target = "crates/nuxie-renderer/src/mechanical_port/source/example_mm.rs"
        manifest = {
            "source_globs": ["renderer/src/metal/example.mm"],
            "source_excludes": [],
            "source": [
                {
                    "upstream": source,
                    "lane": "renderer-platform",
                    "status": "in-progress",
                    "issue": "UNIV-1",
                    "rust_modules": ["crates/nuxie-renderer/src/legacy.rs"],
                }
            ],
            "translation_unit": [
                {"sources": [source], "rust_targets": [target]}
            ],
        }
        errors: list[str] = []
        with mock.patch.object(CHECK, "expand_source_scope", return_value={source}):
            CHECK.validate_source_rows(
                manifest, pathlib.Path("."), pathlib.Path("."), errors
            )
        self.assertIn("do not overlap its translation owner", "\n".join(errors))

        manifest["source"][0]["rust_modules"].append(target)
        errors.clear()
        with mock.patch.object(CHECK, "expand_source_scope", return_value={source}):
            CHECK.validate_source_rows(
                manifest, pathlib.Path("."), pathlib.Path("."), errors
            )
        self.assertEqual(errors, [])

    def test_green_closeout_and_complete_phases_require_sealed_campaign_evidence(self) -> None:
        manifest = {
            "source": [{"upstream": "source.mm", "status": "in-progress"}],
            "translation_unit": [{
                "id": "metal",
                "status": "fixed",
                "translation_receipt": "docs/metal-port-receipts/metal.translation.toml",
                "source_review_receipt": "pending",
                "ownership_review_receipt": "pending",
                "fix_receipt": "pending",
                "compile_receipt": "pending",
                "verification_receipt": "pending",
            }],
        }
        ownership = {"owner": [{"id": "renderer.context", "status": "in-progress"}]}
        progress = {
            "phase": [
                {"id": "translation", "status": "complete"},
                {"id": "ownership-review", "status": "complete"},
                {"id": "compiler", "status": "complete"},
            ],
            "suite": [
                {"id": "V1", "status": "green"},
                {"id": "V9", "status": "green"},
            ],
        }

        errors: list[str] = []
        CHECK.validate_progress_promotion_claims(manifest, ownership, progress, errors)
        joined = "\n".join(errors)
        self.assertIn("ownership-review phase cannot be complete", joined)
        self.assertIn("compiler phase cannot be complete", joined)
        self.assertIn("V1 cannot be green", joined)
        self.assertIn("V9 cannot be green", joined)
        self.assertIn("source.mm", joined)
        self.assertIn("renderer.context", joined)

        unit = manifest["translation_unit"][0]
        unit["status"] = "verified"
        manifest["source"][0]["status"] = "verified"
        ownership["owner"][0]["status"] = "verified"
        for field in CHECK.TRANSLATION_RECEIPT_FIELDS:
            unit[field] = CHECK.canonical_receipt_path("metal", field)

        errors.clear()
        CHECK.validate_progress_promotion_claims(manifest, ownership, progress, errors)
        self.assertEqual(errors, [])

    def test_divergence_ledger_records_two_reviews_and_correction_before_resolution(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        path = repo_root / "docs/metal-port-divergences.tsv"
        with path.open(encoding="utf-8", newline="") as source:
            rows = list(csv.DictReader(source, delimiter="\t"))

        self.assertEqual({row["id"] for row in rows}, CHECK.DIVERGENCE_IDS)
        self.assertTrue(all(row["status"] == "resolved" for row in rows))
        for row in rows:
            unit = f"divergence-{row['id']}"
            self.assertEqual(
                row["source_review_receipt"],
                CHECK.canonical_receipt_path(unit, "source_review_receipt"),
            )
            self.assertEqual(
                row["ownership_review_receipt"],
                CHECK.canonical_receipt_path(unit, "ownership_review_receipt"),
            )
            self.assertEqual(
                row["correction_receipt"],
                CHECK.canonical_receipt_path(unit, "fix_receipt"),
            )

    def test_unresolved_divergences_block_unit_source_and_owner_promotion(self) -> None:
        source = "renderer/src/metal/render_context_metal_impl.mm"
        manifest = {
            "translation_unit": [{"id": "metal", "sources": [source], "status": "fixed"}],
            "source": [{"upstream": source, "status": "ported"}],
        }
        ownership = {
            "owner": [{
                "id": "renderer.owner",
                "status": "verified",
                "citations": [f"cpp:{source}:1"],
            }]
        }
        rows = [
            {"id": f"divergence-{index}", "upstream_source": source, "status": "review-needed"}
            for index in range(6)
        ]
        errors: list[str] = []
        CHECK.validate_divergence_promotions(manifest, ownership, rows, errors)
        joined = "\n".join(errors)
        self.assertIn("translation unit metal cannot promote", joined)
        self.assertIn(f"source {source} cannot promote", joined)
        self.assertIn("ownership row renderer.owner cannot promote", joined)

        for index, row in enumerate(rows):
            row["status"] = "accepted" if index % 2 == 0 else "resolved"
        errors.clear()
        CHECK.validate_divergence_promotions(manifest, ownership, rows, errors)
        self.assertEqual(errors, [])

    def test_shader_source_batch_is_exact_pinned_make_order_and_role_loop(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        with (repo_root / "docs/metal-port-manifest.toml").open("rb") as source:
            manifest = tomllib.load(source)
        with (repo_root / "docs/metal-shader-source-inventory.tsv").open(
            encoding="utf-8", newline=""
        ) as source:
            inventory = list(csv.DictReader(source, delimiter="\t"))

        expected = [row["source"] for row in inventory]
        self.assertEqual(len(expected), 40)
        self.assertEqual(expected[0], "renderer/src/shaders/Makefile")
        self.assertEqual(expected[1], "renderer/src/shaders/minify.py")
        self.assertEqual(expected[2:22], sorted(expected[2:22]))
        self.assertEqual(expected[22:25], sorted(expected[22:25]))
        self.assertEqual(expected[25:36], sorted(expected[25:36]))
        self.assertEqual(
            expected[36],
            "renderer/src/shaders/metal/generate_draw_combinations.py",
        )
        self.assertEqual(
            expected[37:40],
            [
                "renderer/src/shaders/metal/color_ramp.metal",
                "renderer/src/shaders/metal/draw.metal",
                "renderer/src/shaders/metal/tessellate.metal",
            ],
        )
        errors: list[str] = []
        CHECK.validate_metal_shader_translation_unit(manifest, expected, errors)
        self.assertEqual(errors, [])

        bad_manifest = copy.deepcopy(manifest)
        unit = next(
            row
            for row in bad_manifest["translation_unit"]
            if row["id"] == "metal-shader-source-batch"
        )
        unit["sources"] = list(reversed(unit["sources"]))
        unit["dispatch_ordinal"] = 3
        unit["worker_role"] = "sol-high"
        unit["source_review_receipt"] = "premature"
        unit["artifact_targets"] = unit["artifact_targets"][:-1]
        errors.clear()
        CHECK.validate_metal_shader_translation_unit(
            bad_manifest, expected, errors
        )
        joined = "\n".join(errors)
        self.assertIn("sources must exactly match inventory order", joined)
        self.assertIn("dispatch ordinal must be 32", joined)
        self.assertIn("must use luna-extra-high", joined)
        self.assertIn("source_review_receipt must be canonical", joined)
        self.assertIn("artifact targets must match", joined)

        advanced_manifest = copy.deepcopy(manifest)
        advanced_unit = next(
            row
            for row in advanced_manifest["translation_unit"]
            if row["id"] == "metal-shader-source-batch"
        )
        advanced_unit["status"] = "reviewed"
        advanced_unit["source_review_receipt"] = "pending"
        advanced_unit["ownership_review_receipt"] = "pending"
        errors.clear()
        CHECK.validate_metal_shader_translation_unit(
            advanced_manifest, expected, errors
        )
        joined = "\n".join(errors)
        self.assertIn("source_review_receipt must be canonical", joined)
        self.assertIn("ownership_review_receipt must be canonical", joined)

    def test_field_ledger_separates_mapping_from_translation(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        path = repo_root / "docs/render-context-metal-fields.tsv"
        with path.open(encoding="utf-8", newline="") as source:
            rows = list(csv.DictReader(source, delimiter="\t"))

        self.assertEqual(len(rows), 455)
        self.assertTrue(all(row["mapping_status"] == "prepared" for row in rows))
        self.assertEqual(
            sum(row["translation_status"] == "verified" for row in rows),
            455,
        )
        pending = [row for row in rows if row["translation_status"] == "pending"]
        self.assertEqual(len(pending), 0)
        by_field = {(row["cpp_type"], row["cpp_field"]): row for row in rows}
        expected_types = {
            ("PaintAuxData", "m_inverseFwidth"): "volatile Vec2D",
            ("RenderContext", "m_polarSegmentCountsAllocator"): "TrivialArrayAllocator<uint32_t, alignof(float4)>",
            ("RenderContext", "m_logicalFlushes"): "std::vector<std::unique_ptr<LogicalFlush>>",
            ("RenderContext::LogicalFlush", "m_pendingComplexGradDraws"): "std::vector<const Gradient *>",
            ("RenderContext::LogicalFlush", "m_pendingFeatherAtlasDraws"): "std::vector<PathDraw *>",
            ("RenderContext::LogicalFlush", "m_draws"): "std::vector<DrawUniquePtr>",
        }
        for key, cpp_type in expected_types.items():
            self.assertEqual(by_field[key]["cpp_declared_type"], cpp_type)
        self.assertNotIn("NonNull", by_field[("PaintAuxData", "m_inverseFwidth")]["rust_field"])
        self.assertIn("arena owner", by_field[("RenderContext", "m_polarSegmentCountsAllocator")]["rust_field"])
        self.assertIn("Vec<Box<", by_field[("RenderContext", "m_logicalFlushes")]["rust_field"])
        self.assertIn("Vec<NonNull", by_field[("RenderContext::LogicalFlush", "m_pendingComplexGradDraws")]["rust_field"])
        self.assertIn("Vec<Box<", by_field[("RenderContext::LogicalFlush", "m_draws")]["rust_field"])
        self.assertIn("&'ctx RenderContext", by_field[("RenderContext::LogicalFlush", "m_ctx")]["rust_field"])
        self.assertIn("&'flush mut LogicalFlush", by_field[("RenderContext::TessellationWriter", "m_flush")]["rust_field"])
        self.assertIn("ArenaLinkedList<'flush", by_field[("RenderContext::LogicalFlush", "m_drawList")]["rust_field"])
        self.assertIn("Option<NonNull<DrawBatch>>", by_field[("RenderContext::LogicalFlush", "m_firstDstBlendBarrier")]["rust_field"])
        self.assertIn("[u8; 152]", by_field[("FlushUniforms", "m_padTo256Bytes")]["rust_field"])

    def test_prepared_field_mapping_rejects_placeholder_lifetime_prose(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        source_path = repo_root / "docs/render-context-metal-fields.tsv"
        with source_path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = list(reader.fieldnames or ())
            rows = list(reader)
        rows[92]["construction_and_publication"] = (
            "This is declared source state; preserve the source mutation sites."
        )
        with tempfile.TemporaryDirectory() as temporary:
            temporary_root = pathlib.Path(temporary)
            temporary_map = temporary_root / "fields.tsv"
            with temporary_map.open("w", encoding="utf-8", newline="") as output:
                writer = csv.DictWriter(output, fieldnames=fieldnames, delimiter="\t")
                writer.writeheader()
                writer.writerows(rows)
            errors: list[str] = []
            CHECK.validate_render_context_field_map(
                {
                    "render_context_field_map": "fields.tsv",
                    "upstream_ref": "4ac7b32798da0482e441ef09304dc3b480ed3ee5",
                },
                temporary_root,
                configured_upstream(),
                errors,
            )
        self.assertIn("marks placeholder prose prepared", "\n".join(errors))

    def test_prepared_field_mapping_rejects_ownership_alternatives(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        source_path = repo_root / "docs/render-context-metal-fields.tsv"
        with source_path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = list(reader.fieldnames or ())
            rows = list(reader)
        row = next(item for item in rows if item["cpp_field"] == "m_tessSpanData" and item["cpp_type"].endswith("TessellationWriter"))
        row["safe_rust_adaptation"] = "Use a lifetime-bound borrow or NonNull handle."
        with tempfile.TemporaryDirectory() as temporary:
            temporary_root = pathlib.Path(temporary)
            temporary_map = temporary_root / "fields.tsv"
            with temporary_map.open("w", encoding="utf-8", newline="") as output:
                writer = csv.DictWriter(output, fieldnames=fieldnames, delimiter="\t")
                writer.writeheader()
                writer.writerows(rows)
            errors: list[str] = []
            CHECK.validate_render_context_field_map(
                {"render_context_field_map": "fields.tsv", "upstream_ref": "4ac7b32798da0482e441ef09304dc3b480ed3ee5"},
                temporary_root, configured_upstream(), errors,
            )
        self.assertIn("marks placeholder prose prepared", "\n".join(errors))

    def test_configuration_ledger_separates_mapping_from_translation(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        path = repo_root / "docs/render-context-metal-configurations.tsv"
        with path.open(encoding="utf-8", newline="") as source:
            rows = list(csv.DictReader(source, delimiter="\t"))

        self.assertEqual(len(rows), 85)
        self.assertTrue(all(row["mapping_status"] == "prepared" for row in rows))
        self.assertTrue(
            all(row["translation_disposition"] == "required" for row in rows)
        )
        self.assertTrue(all(row["exclusion_reason"] == "-" for row in rows))
        self.assertEqual(
            sum(row["translation_status"] == "verified" for row in rows),
            85,
        )
        self.assertEqual(
            sum(row["translation_status"] == "pending" for row in rows),
            0,
        )

    def test_configuration_translation_cannot_outrun_owning_unit_receipts(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        source_path = repo_root / "docs/render-context-metal-configurations.tsv"
        with source_path.open(encoding="utf-8", newline="") as source:
            reader = csv.DictReader(source, delimiter="\t")
            fieldnames = list(reader.fieldnames or ())
            rows = list(reader)
        rows[0]["translation_status"] = "translated"
        with tempfile.TemporaryDirectory() as temporary:
            temporary_root = pathlib.Path(temporary)
            target = temporary_root / "configurations.tsv"
            with target.open("w", encoding="utf-8", newline="") as output:
                writer = csv.DictWriter(output, fieldnames=fieldnames, delimiter="\t")
                writer.writeheader()
                writer.writerows(rows)
            sources = sorted({row["upstream_file"] for row in rows})
            manifest = {
                "render_context_configuration_map": "configurations.tsv",
                "upstream_ref": "4ac7b32798da0482e441ef09304dc3b480ed3ee5",
                "translation_unit": [
                    {"id": f"unit-{index}", "status": "pending", "sources": [name]}
                    for index, name in enumerate(sources)
                ],
            }
            errors: list[str] = []
            CHECK.validate_render_context_configuration_map(
                manifest, temporary_root, configured_upstream(), errors
            )
        self.assertIn("must be pending for its receipt-gated owning unit", "\n".join(errors))

    def test_row_coverage_status_is_bidirectional_with_unit_receipts(self) -> None:
        self.assertEqual(CHECK.derived_coverage_status({"status": "pending"}), "pending")
        self.assertEqual(CHECK.derived_coverage_status({"status": "translated"}), "translated")
        self.assertEqual(CHECK.derived_coverage_status({"status": "compiled"}), "translated")
        self.assertEqual(CHECK.derived_coverage_status({"status": "verified"}), "verified")

    def test_source_and_file_map_cannot_lag_compiled_unit(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = pathlib.Path(temporary)
            relative = "renderer/src/metal/render_context_metal_impl.mm"
            file_map = root / "files.tsv"
            file_map.write_text(
                "version\tupstream_sha\tupstream_file\tlines\tsymbol\tstatus\trust_owner\tremaining\n"
                f"1\t{'a' * 40}\t{relative}\t1-1\tbody\tpartial\tout.rs\tremaining\n"
            )
            manifest = {
                "render_context_file_map": "files.tsv",
                "source": [{"upstream": relative, "status": "pending"}],
                "translation_unit": [{
                    "id": "metal", "status": "compiled", "sources": [relative]
                }],
            }
            errors: list[str] = []
            CHECK.validate_source_unit_promotion(manifest, root, errors)
            joined = "\n".join(errors)
            self.assertIn("status must be ported", joined)
            self.assertIn("file-map rows", joined)

            manifest["source"][0]["status"] = "ported"
            file_map.write_text(file_map.read_text().replace("\tpartial\t", "\tported\t"))
            errors.clear()
            CHECK.validate_source_unit_promotion(manifest, root, errors)
            self.assertEqual(errors, [])

    def test_global_compiler_and_behavior_phase_barriers(self) -> None:
        manifest = translation_manifest_fixture()
        units = manifest["translation_unit"]
        units[0]["status"] = "compiled"
        errors: list[str] = []
        CHECK.validate_translation_units(manifest, errors)
        self.assertIn("compiler queue cannot start until all 41 units", "\n".join(errors))

        for unit in units:
            unit["status"] = "fixed"
        units[0]["status"] = "compiled"
        errors.clear()
        CHECK.validate_translation_units(manifest, errors)
        self.assertNotIn("compiler queue cannot start until all 41 units", "\n".join(errors))

        for unit in units:
            unit["status"] = "compiled"
        units[0]["status"] = "verified"
        errors.clear()
        CHECK.validate_translation_units(manifest, errors)
        self.assertNotIn("behavior verification cannot start", "\n".join(errors))

        units[-1]["status"] = "fixed"
        errors.clear()
        CHECK.validate_translation_units(manifest, errors)
        self.assertIn("behavior verification cannot start until all 41 units", "\n".join(errors))

    def test_fixed_cannot_jump_to_verified_with_only_translation_loop_receipts(self) -> None:
        unit = {
            "id": "unit", "status": "verified", "base_ref": "a" * 40,
            "open_findings": 0,
            "compile_receipt": "pending", "verification_receipt": "pending",
        }
        for field in (
            "translation_receipt", "source_review_receipt",
            "ownership_review_receipt", "fix_receipt",
        ):
            unit[field] = CHECK.canonical_receipt_path("unit", field)
        errors: list[str] = []
        CHECK.validate_translation_receipts(unit, errors)
        joined = "\n".join(errors)
        self.assertIn("compile_receipt must be canonical", joined)
        self.assertIn("verification_receipt must be canonical", joined)

    def test_generic_dependency_and_include_authorities_are_exhaustive(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        with (repo_root / "docs/render-context-metal-dependencies.tsv").open(
            encoding="utf-8", newline=""
        ) as source:
            dependencies = list(csv.DictReader(source, delimiter="\t"))
        with (repo_root / "docs/render-context-metal-includes.tsv").open(
            encoding="utf-8", newline=""
        ) as source:
            includes = list(csv.DictReader(source, delimiter="\t"))

        self.assertEqual(
            [row["upstream_file"] for row in dependencies],
            list(CHECK.RENDER_CONTEXT_DEPENDENCY_SOURCES),
        )
        self.assertEqual(len(dependencies), 35)
        self.assertTrue(all(row["mapping_status"] == "prepared" for row in dependencies))
        self.assertTrue(all(row["translation_status"] == "verified" for row in dependencies))
        self.assertEqual(len(includes), 174)
        self.assertEqual(len({row["include_token"] for row in includes}), 76)
        self.assertTrue(all(row["mapping_status"] == "prepared" for row in includes))

    def test_manifest_dispatch_queue_is_exact_and_covers_every_source_once(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        with (repo_root / "docs/metal-port-manifest.toml").open("rb") as source:
            manifest = tomllib.load(source)
        units = manifest["translation_unit"]
        dispatched = {
            unit["dispatch_ordinal"]: unit["id"]
            for unit in units
            if "dispatch_ordinal" in unit
        }
        self.assertEqual(set(dispatched), set(range(1, 42)))
        self.assertEqual(
            tuple(dispatched[ordinal] for ordinal in range(1, 42)),
            CHECK.MECHANICAL_DISPATCH_ORDER,
        )
        assigned = [source for unit in units for source in unit["sources"]]
        manifest_sources = [row["upstream"] for row in manifest["source"]]
        self.assertEqual(len(assigned), len(set(assigned)))
        self.assertEqual(set(assigned), set(manifest_sources))

        by_id = {unit["id"]: unit for unit in units}
        for unit_id in CHECK.ORE_TRANSLATION_UNIT_ORDER:
            unit = by_id[unit_id]
            self.assertEqual(
                unit["dispatch_ordinal"],
                CHECK.MECHANICAL_DISPATCH_ORDINALS[unit_id],
            )
            self.assertEqual(unit["worker_role"], "luna-extra-high")
            self.assertNotEqual(unit["worker_claim"], "unclaimed")
            self.assertIn(
                unit["status"],
                {"translated", "fixed", "compiled", "verified"},
            )
            self.assertEqual(
                unit["translation_receipt"],
                CHECK.canonical_receipt_path(unit_id, "translation_receipt"),
            )
            self.assertTrue(
                all(
                    target == CHECK.mechanical_ore_target(source)
                    for source, target in zip(unit["sources"], unit["rust_targets"])
                )
            )
        for unit in units:
            for prerequisite in unit["dispatch_prerequisites"]:
                self.assertLess(
                    by_id[prerequisite]["dispatch_ordinal"],
                    unit["dispatch_ordinal"],
                )

    def test_imperative_plan_matches_zero_baseline_and_dispatch_ordinals(self) -> None:
        plan = (MODULE_PATH.parents[2] / "docs/METAL_RENDERER_PORT_PLAN.md").read_text()
        self.assertIn("dispatch ordinal 32", plan)
        self.assertIn("ordinals 33 through 35", plan)
        self.assertIn("ordinals 36 through 38", plan)
        self.assertIn("ordinals 39 through 41", plan)
        self.assertIn(
            "All 111 primary manifest rows and all 41 translation units are `in-progress`",
            plan,
        )
        self.assertIn("exactly 634 blocks and 845 branch entries", plan)
        self.assertNotIn("dispatch ordinal 20", plan)

    def test_translation_units_reject_non_topological_dispatch_ordinal(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        with (repo_root / "docs/metal-port-manifest.toml").open("rb") as source:
            manifest = tomllib.load(source)
        unit = next(
            row for row in manifest["translation_unit"] if row["id"] == "gpu-resource"
        )
        unit["dispatch_ordinal"] = 4
        errors: list[str] = []
        CHECK.validate_translation_units(manifest, errors)
        self.assertIn(
            "translation unit gpu-resource dispatch prerequisite generic-refcnt "
            "ordinal 5 must precede consumer ordinal 4",
            errors,
        )

    def test_dispatch_claims_are_strictly_sequenced_but_ore_can_progress(self) -> None:
        manifest = translation_manifest_fixture()
        first = manifest["translation_unit"][0]
        first["dispatch_ordinal"] = 1
        first["status"] = "in-progress"
        manifest["source"][0]["status"] = "in-progress"
        errors: list[str] = []
        CHECK.validate_translation_units(manifest, errors)
        self.assertEqual(errors, [])

        second = manifest["translation_unit"][1]
        second["dispatch_ordinal"] = 2
        second["status"] = "in-progress"
        second["worker_claim"] = "luna-second"
        manifest["source"][1]["status"] = "in-progress"
        errors.clear()
        CHECK.validate_translation_units(manifest, errors)
        self.assertEqual(errors, [])

        first["status"] = "pending"
        first["worker_claim"] = "unclaimed"
        manifest["source"][0]["status"] = "pending"
        errors.clear()
        CHECK.validate_translation_units(manifest, errors)
        self.assertIn("advances before ordinal 1 ore-types is claimed", "\n".join(errors))

    def test_translation_receipts_are_canonical_and_state_gated(self) -> None:
        unit = {
            "id": "ore-types",
            "status": "reviewed",
            "translation_receipt": "arbitrary",
            "source_review_receipt": "pending",
            "ownership_review_receipt": "unrecorded",
            "fix_receipt": "pending",
            "open_findings": 0,
        }
        errors: list[str] = []
        CHECK.validate_translation_receipts(unit, errors)
        joined = "\n".join(errors)
        self.assertIn("translation_receipt must be canonical", joined)
        self.assertIn("source_review_receipt must be canonical", joined)
        self.assertIn("ownership_review_receipt must be canonical", joined)

        # Findings discovered by the two Sol reviews remain visible while a
        # translated unit is being corrected. They block only fixed-or-later
        # promotion, not the review/fix loop itself.
        unit.update(
            status="translated",
            translation_receipt=CHECK.canonical_receipt_path(
                "ore-types", "translation_receipt"
            ),
            source_review_receipt="pending",
            ownership_review_receipt="pending",
            fix_receipt="pending",
            compile_receipt="pending",
            verification_receipt="pending",
            open_findings=2,
        )
        errors.clear()
        CHECK.validate_translation_receipts(unit, errors)
        self.assertNotIn("must have zero open findings", "\n".join(errors))

        unit["id"] = "missing-unit"
        unit["status"] = "fixed"
        for field in (
            "translation_receipt",
            "source_review_receipt",
            "ownership_review_receipt",
            "fix_receipt",
        ):
            unit[field] = CHECK.canonical_receipt_path("missing-unit", field)
        unit["compile_receipt"] = "pending"
        unit["verification_receipt"] = "pending"
        unit["open_findings"] = 1
        errors.clear()
        CHECK.validate_translation_receipts(unit, errors, MODULE_PATH.parents[2])
        joined = "\n".join(errors)
        self.assertIn("receipt does not exist as a tracked file", joined)
        self.assertIn("must have zero open findings", joined)

    def test_tracked_receipt_must_parse_and_report_zero_findings(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            relative = CHECK.canonical_receipt_path("ore-types", "translation_receipt")
            receipt = root / relative
            receipt.parent.mkdir(parents=True)
            receipt.write_text('arbitrary = "filename-only evidence"\n')
            subprocess.run(["git", "-C", str(root), "add", relative], check=True)
            unit = {
                "id": "ore-types",
                "base_ref": "a" * 40,
                "status": "translated",
                "translation_receipt": relative,
                "source_review_receipt": "pending",
                "ownership_review_receipt": "pending",
                "fix_receipt": "pending",
                "open_findings": 0,
            }
            errors: list[str] = []
            CHECK.validate_translation_receipts(unit, errors, root)
            joined = "\n".join(errors)
            self.assertIn("schema_version must be 1", joined)
            self.assertIn("requires nonempty artifact_digests", joined)

            receipt.write_text(
                receipt.read_text()
                + "open_findings = 7\n"
            )
            errors.clear()
            CHECK.validate_translation_receipts(unit, errors, root)
            self.assertIn("open_findings must be zero", "\n".join(errors))

    def test_nonempty_fabricated_receipt_cannot_advance(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory) / "repo"
            upstream = pathlib.Path(directory) / "upstream"
            root.mkdir()
            upstream.mkdir()
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "config", "user.email", "test@example.com"], check=True)
            subprocess.run(["git", "-C", str(root), "config", "user.name", "Test"], check=True)
            (root / "out.rs").write_text("real artifact\n")
            (upstream / "source.mm").write_text("pinned source\n")
            subprocess.run(["git", "-C", str(root), "add", "out.rs"], check=True)
            subprocess.run(["git", "-C", str(root), "commit", "-qm", "base"], check=True)
            workspace_ref = subprocess.check_output(
                ["git", "-C", str(root), "rev-parse", "HEAD"], text=True
            ).strip()
            relative = CHECK.canonical_receipt_path("unit", "translation_receipt")
            receipt = root / relative
            receipt.parent.mkdir(parents=True)
            receipt.write_text(
                "\n".join([
                    "schema_version = 1",
                    'unit = "unit"',
                    'receipt_kind = "translation"',
                    f'upstream_ref = {"\"" + "a" * 40 + "\""}',
                    f'workspace_base_ref = "{workspace_ref}"',
                    'role = "luna-extra-high"',
                    "open_findings = 0",
                    "omitted_lines = 0",
                    "omitted_declarations = 0",
                    "omitted_conditionals = 0",
                    "omitted_include_owners = 0",
                    'commands = ["false :: exit=9 :: count=0"]',
                    'evidence = ["missing.txt"]',
                    f'artifact_digests = {{ "out.rs" = "{"0" * 64}" }}',
                    f'source_digests = {{ "source.mm" = "{"0" * 64}" }}',
                    "",
                ])
            )
            subprocess.run(["git", "-C", str(root), "add", relative], check=True)
            unit = {
                "id": "unit",
                "base_ref": "a" * 40,
                "sources": ["source.mm", "missing-source.mm"],
                "rust_targets": ["out.rs", "missing-out.rs"],
                "artifact_targets": [],
                "status": "translated",
                "translation_receipt": relative,
                "source_review_receipt": "pending",
                "ownership_review_receipt": "pending",
                "fix_receipt": "pending",
                "open_findings": 0,
            }
            errors: list[str] = []
            CHECK.validate_translation_receipts(unit, errors, root, upstream)
            joined = "\n".join(errors)
            self.assertIn("commands must truthfully claim success", joined)
            self.assertIn("evidence path is missing or untracked", joined)
            self.assertIn("artifact_digests must exactly cover unit outputs", joined)
            self.assertIn("artifact digest mismatches bytes", joined)
            self.assertIn("source_digests must exactly cover unit sources", joined)
            self.assertIn("source digest mismatches pinned bytes", joined)

            (root / "missing-out.rs").write_text("second artifact\n")
            (root / "evidence.txt").write_text("43 checks passed\n")
            (upstream / "missing-source.mm").write_text("second pinned source\n")
            subprocess.run(
                ["git", "-C", str(root), "add", "missing-out.rs", "evidence.txt"],
                check=True,
            )
            artifact_digests = {
                name: CHECK.hashlib.sha256((root / name).read_bytes()).hexdigest()
                for name in ("out.rs", "missing-out.rs")
            }
            source_digests = {
                name: CHECK.hashlib.sha256((upstream / name).read_bytes()).hexdigest()
                for name in ("source.mm", "missing-source.mm")
            }
            receipt.write_text(
                "\n".join([
                    "schema_version = 1",
                    'unit = "unit"',
                    'receipt_kind = "translation"',
                    f'upstream_ref = {"\"" + "a" * 40 + "\""}',
                    f'workspace_base_ref = "{workspace_ref}"',
                    'role = "luna-extra-high"',
                    "open_findings = 0",
                    "omitted_lines = 0",
                    "omitted_declarations = 0",
                    "omitted_conditionals = 0",
                    "omitted_include_owners = 0",
                    'commands = ["python3 -m unittest :: exit=0 :: count=43"]',
                    'evidence = ["evidence.txt"]',
                    "artifact_digests = { " + ", ".join(
                        f'"{name}" = "{digest}"' for name, digest in artifact_digests.items()
                    ) + " }",
                    "source_digests = { " + ", ".join(
                        f'"{name}" = "{digest}"' for name, digest in source_digests.items()
                    ) + " }",
                    "",
                ])
            )
            subprocess.run(["git", "-C", str(root), "add", relative], check=True)
            errors.clear()
            CHECK.validate_translation_receipts(unit, errors, root, upstream)
            self.assertEqual(errors, [])

    def test_review_receipts_bind_owned_sources_artifacts_context_and_baseline(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory) / "repo"
            upstream = pathlib.Path(directory) / "upstream"
            root.mkdir()
            upstream.mkdir()
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "config", "user.email", "test@example.com"], check=True)
            subprocess.run(["git", "-C", str(root), "config", "user.name", "Test"], check=True)
            artifact = root / "out.rs"
            artifact.write_text("fn reviewed() {}\n")
            (upstream / "source.mm").write_text("source behavior\n")
            (upstream / "unrelated.mm").write_text("unrelated\n")
            subprocess.run(["git", "-C", str(root), "add", "out.rs"], check=True)
            subprocess.run(["git", "-C", str(root), "commit", "-qm", "base"], check=True)
            base = subprocess.check_output(
                ["git", "-C", str(root), "rev-parse", "HEAD"], text=True
            ).strip()
            digest = CHECK.hashlib.sha256(artifact.read_bytes()).hexdigest()
            receipt = root / "review.toml"

            def write_review(kind: str, run_id: str, cpp: str, coverage: list[str]) -> None:
                receipt.write_text(
                    "\n".join([
                        "schema_version = 1",
                        'unit = "unit"',
                        f'receipt_kind = "{kind}"',
                        f'upstream_ref = {"\"" + "a" * 40 + "\""}',
                        f'workspace_base_ref = "{base}"',
                        'role = "sol-high"',
                        "open_findings = 0",
                        'commands = ["review-check :: exit=0 :: count=2"]',
                        f'evidence = ["{cpp}", "rust:out.rs:1"]',
                        f'artifact_digests = {{ "out.rs" = "{digest}" }}',
                        "findings = []",
                        f'review_run_id = "{run_id}"',
                        "coverage = [" + ", ".join(f'"{value}"' for value in coverage) + "]",
                        f'citations = ["{cpp}", "rust:out.rs:1"]',
                        "",
                    ])
                )

            source_coverage = [
                "owned-source-lines", "declarations", "conditionals",
                "include-owners", "source-semantics",
            ]
            write_review("source-review", "source-run-0001", "cpp:source.mm:1", source_coverage)
            errors: list[str] = []
            CHECK.validate_receipt_contents(
                receipt, "unit", "source_review_receipt", "a" * 40, errors,
                repo_root=root, upstream_root=upstream,
                expected_sources=["source.mm"], expected_artifacts=["out.rs"],
            )
            self.assertEqual(errors, [])

            artifact.write_text("fn reviewed() {}\nfn appended() {}\n")
            appended_digest = CHECK.hashlib.sha256(artifact.read_bytes()).hexdigest()
            receipt.write_text(receipt.read_text().replace(digest, appended_digest))
            errors.clear()
            CHECK.validate_receipt_contents(
                receipt, "unit", "source_review_receipt", "a" * 40, errors,
                repo_root=root, upstream_root=upstream,
                expected_sources=["source.mm"], expected_artifacts=["out.rs"],
            )
            self.assertIn(
                "do not cover every current Rust artifact line", "\n".join(errors)
            )
            receipt.write_text(receipt.read_text().replace("rust:out.rs:1", "rust:out.rs:1-2"))
            errors.clear()
            CHECK.validate_receipt_contents(
                receipt, "unit", "source_review_receipt", "a" * 40, errors,
                repo_root=root, upstream_root=upstream,
                expected_sources=["source.mm"], expected_artifacts=["out.rs"],
            )
            self.assertEqual(errors, [])

            (upstream / "source.mm").write_text("source behavior\nappended source\n")
            errors.clear()
            CHECK.validate_receipt_contents(
                receipt, "unit", "source_review_receipt", "a" * 40, errors,
                repo_root=root, upstream_root=upstream,
                expected_sources=["source.mm"], expected_artifacts=["out.rs"],
            )
            self.assertIn(
                "do not cover every current source line", "\n".join(errors)
            )
            receipt.write_text(receipt.read_text().replace("cpp:source.mm:1", "cpp:source.mm:1-2"))
            errors.clear()
            CHECK.validate_receipt_contents(
                receipt, "unit", "source_review_receipt", "a" * 40, errors,
                repo_root=root, upstream_root=upstream,
                expected_sources=["source.mm"], expected_artifacts=["out.rs"],
            )
            self.assertEqual(errors, [])

            # Restore the one-line fixture used by the remaining review-shape probes.
            artifact.write_text("fn reviewed() {}\n")
            (upstream / "source.mm").write_text("source behavior\n")
            digest = CHECK.hashlib.sha256(artifact.read_bytes()).hexdigest()

            write_review("source-review", "source-run-0001", "cpp:unrelated.mm:1", source_coverage)
            errors.clear()
            CHECK.validate_receipt_contents(
                receipt, "unit", "source_review_receipt", "a" * 40, errors,
                repo_root=root, upstream_root=upstream,
                expected_sources=["source.mm"], expected_artifacts=["out.rs"],
            )
            self.assertIn("exactly cover owned unit sources", "\n".join(errors))

            ownership_coverage = [
                "fields", "lifetimes", "threads", "retain-release", "drop-order",
                "unsafe-invariants", "divergences",
            ]
            write_review("ownership-review", "ownership-run-0002", "cpp:source.mm:1", ownership_coverage)
            errors.clear()
            CHECK.validate_receipt_contents(
                receipt, "unit", "ownership_review_receipt", "a" * 40, errors,
                repo_root=root, upstream_root=upstream,
                expected_sources=["source.mm"], expected_artifacts=["out.rs"],
            )
            self.assertEqual(errors, [])

            write_review("ownership-review", "ownership-run-0003", "cpp:unrelated.mm:1", ownership_coverage)
            errors.clear()
            CHECK.validate_receipt_contents(
                receipt, "unit", "ownership_review_receipt", "a" * 40, errors,
                repo_root=root, upstream_root=upstream,
                expected_sources=["source.mm"], expected_artifacts=["out.rs"],
            )
            self.assertIn("exactly cover owned unit sources", "\n".join(errors))

            write_review("ownership-review", "ownership-run-0002", "cpp:source.mm:1", ownership_coverage)

            errors.clear()
            CHECK.validate_receipt_contents(
                receipt, "unit", "ownership_review_receipt", "a" * 40, errors,
                repo_root=root, upstream_root=upstream,
                expected_sources=["source.mm"], expected_artifacts=["out.rs"],
                required_cpp_ranges=["cpp:source.mm:1"],
                required_rust_owner="out.rs", require_scoped_evidence=True,
            )
            self.assertEqual(errors, [])
            receipt.write_text(receipt.read_text().replace("cpp:source.mm:1", "cpp:unrelated.mm:1"))
            errors.clear()
            CHECK.validate_receipt_contents(
                receipt, "unit", "ownership_review_receipt", "a" * 40, errors,
                repo_root=root, upstream_root=upstream,
                expected_sources=["source.mm"], expected_artifacts=["out.rs"],
                required_cpp_ranges=["cpp:source.mm:1"],
                required_rust_owner="out.rs", require_scoped_evidence=True,
            )
            self.assertIn("exact divergence C++ range", "\n".join(errors))

            write_review("source-review", "source-run-0001", "cpp:source.mm:1", source_coverage)
            copied = receipt.read_text().replace('receipt_kind = "source-review"', 'receipt_kind = "ownership-review"')
            receipt.write_text(copied)
            errors.clear()
            CHECK.validate_receipt_contents(
                receipt, "unit", "ownership_review_receipt", "a" * 40, errors,
                repo_root=root, upstream_root=upstream,
                expected_sources=["source.mm"], expected_artifacts=["out.rs"],
            )
            self.assertIn("ownership_review review contract", "\n".join(errors))

            tree = subprocess.check_output(
                ["git", "-C", str(root), "write-tree"], text=True
            ).strip()
            orphan = subprocess.check_output(
                ["git", "-C", str(root), "commit-tree", tree, "-m", "orphan"], text=True
            ).strip()
            receipt.write_text(receipt.read_text().replace(base, orphan))
            errors.clear()
            CHECK.validate_receipt_contents(
                receipt, "unit", "ownership_review_receipt", "a" * 40, errors,
                repo_root=root, upstream_root=upstream,
                expected_sources=["source.mm"], expected_artifacts=["out.rs"],
            )
            self.assertIn("must be an ancestor of current HEAD", "\n".join(errors))

    def test_receipt_command_replay_checks_exit_and_declared_count(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory) / "repo"
            upstream = pathlib.Path(directory) / "upstream"
            root.mkdir()
            upstream.mkdir()
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            subprocess.run(["git", "-C", str(root), "config", "user.email", "test@example.com"], check=True)
            subprocess.run(["git", "-C", str(root), "config", "user.name", "Test"], check=True)
            artifact = root / "out.rs"
            source = upstream / "source.mm"
            artifact.write_text("fn translated() {}\n")
            source.write_text("source\n")
            subprocess.run(["git", "-C", str(root), "add", "out.rs"], check=True)
            subprocess.run(["git", "-C", str(root), "commit", "-qm", "base"], check=True)
            base = subprocess.check_output(
                ["git", "-C", str(root), "rev-parse", "HEAD"], text=True
            ).strip()
            receipt = root / "translation.toml"
            receipt.write_text("\n".join([
                "schema_version = 1",
                'unit = "unit"',
                'receipt_kind = "translation"',
                f'upstream_ref = {"\"" + "a" * 40 + "\""}',
                f'workspace_base_ref = "{base}"',
                'role = "luna-extra-high"',
                "open_findings = 0",
                "omitted_lines = 0",
                "omitted_declarations = 0",
                "omitted_conditionals = 0",
                "omitted_include_owners = 0",
                'commands = ["printf 7 :: exit=0 :: count=7"]',
                'evidence = ["rust:out.rs:1"]',
                f'artifact_digests = {{ "out.rs" = "{CHECK.hashlib.sha256(artifact.read_bytes()).hexdigest()}" }}',
                f'source_digests = {{ "source.mm" = "{CHECK.hashlib.sha256(source.read_bytes()).hexdigest()}" }}',
                "",
            ]))
            old_replay = CHECK.REPLAY_RECEIPT_COMMANDS
            CHECK.REPLAY_RECEIPT_COMMANDS = True
            CHECK._RECEIPT_COMMAND_CACHE.clear()
            try:
                errors: list[str] = []
                CHECK.validate_receipt_contents(
                    receipt, "unit", "translation_receipt", "a" * 40, errors,
                    repo_root=root, upstream_root=upstream,
                    expected_sources=["source.mm"], expected_artifacts=["out.rs"],
                )
                self.assertEqual(errors, [])

                receipt.write_text(receipt.read_text().replace("count=7", "count=8"))
                errors.clear()
                CHECK.validate_receipt_contents(
                    receipt, "unit", "translation_receipt", "a" * 40, errors,
                    repo_root=root, upstream_root=upstream,
                    expected_sources=["source.mm"], expected_artifacts=["out.rs"],
                )
                self.assertIn("command replay count is 7, claimed 8", "\n".join(errors))

                receipt.write_text(receipt.read_text().replace("printf 7", "false"))
                errors.clear()
                CHECK.validate_receipt_contents(
                    receipt, "unit", "translation_receipt", "a" * 40, errors,
                    repo_root=root, upstream_root=upstream,
                    expected_sources=["source.mm"], expected_artifacts=["out.rs"],
                )
                self.assertIn("command replay exited 1", "\n".join(errors))
            finally:
                CHECK.REPLAY_RECEIPT_COMMANDS = old_replay
                CHECK._RECEIPT_COMMAND_CACHE.clear()

    def test_review_receipts_reject_reused_review_context(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            unit = {
                "id": "unit", "status": "reviewed", "base_ref": "a" * 40,
                "translation_receipt": CHECK.canonical_receipt_path("unit", "translation_receipt"),
                "source_review_receipt": CHECK.canonical_receipt_path("unit", "source_review_receipt"),
                "ownership_review_receipt": CHECK.canonical_receipt_path("unit", "ownership_review_receipt"),
                "fix_receipt": "pending", "open_findings": 0,
            }
            for field in ("source_review_receipt", "ownership_review_receipt"):
                path = root / unit[field]
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text('review_run_id = "same-review-context"\n')
            errors: list[str] = []
            CHECK.validate_translation_receipts(unit, errors, root)
            self.assertIn("must use distinct review_run_id values", "\n".join(errors))

    def test_fix_receipt_preserves_clean_or_finding_history(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            receipt = pathlib.Path(directory) / "fix.toml"
            receipt.write_text(
                "\n".join([
                    "schema_version = 1",
                    'unit = "unit"',
                    'receipt_kind = "fix"',
                    f'upstream_ref = {"\"" + "a" * 40 + "\""}',
                    f'workspace_base_ref = {"\"" + "b" * 40 + "\""}',
                    'role = "sol-high"',
                    "open_findings = 0",
                    'commands = ["final-audit :: exit=0 :: count=1"]',
                    'evidence = ["audit.txt"]',
                    f'artifact_digests = {{ "audit.txt" = "{"0" * 64}" }}',
                    'resolutions = ["NO_FINDINGS: final clean audit"]',
                    "",
                ])
            )
            errors: list[str] = []
            CHECK.validate_receipt_contents(
                receipt, "unit", "fix_receipt", "a" * 40, errors
            )
            self.assertEqual(errors, [])

            receipt.write_text(receipt.read_text().replace(
                '"NO_FINDINGS: final clean audit"', '"clean"'
            ))
            errors.clear()
            CHECK.validate_receipt_contents(
                receipt, "unit", "fix_receipt", "a" * 40, errors
            )
            self.assertIn("stable finding IDs", "\n".join(errors))

    def test_mechanical_translation_workflow_locks_bun_roles_and_queue_order(self) -> None:
        workflow = dict(CHECK.MECHANICAL_TRANSLATION_WORKFLOW)
        manifest = {"mechanical_translation_workflow": workflow}
        errors: list[str] = []
        CHECK.validate_mechanical_translation_workflow(manifest, errors)
        self.assertEqual(errors, [])

        workflow["translator_role"] = "sol-high"
        workflow["feature_or_fixture_work_items_forbidden"] = False
        workflow.pop("cleanup_only_after_full_parity")
        workflow["test_first_feature_queue"] = True
        CHECK.validate_mechanical_translation_workflow(manifest, errors)
        joined = "\n".join(errors)
        self.assertIn("translator_role must be 'luna-extra-high'", joined)
        self.assertIn("feature_or_fixture_work_items_forbidden must be True", joined)
        self.assertIn("missing keys: cleanup_only_after_full_parity", joined)
        self.assertIn("invented keys: test_first_feature_queue", joined)

    def test_translation_units_cover_pending_ore_sources_once(self) -> None:
        manifest = translation_manifest_fixture()

        errors: list[str] = []
        CHECK.validate_translation_units(manifest, errors)
        self.assertEqual(errors, [])

        manifest["translation_unit"][1]["sources"] = [
            "renderer/include/rive/renderer/ore/ore_types.hpp"
        ]
        errors.clear()
        CHECK.validate_translation_units(manifest, errors)
        joined = "\n".join(errors)
        self.assertIn("missing active manifest sources", joined)
        self.assertIn("overlapping translation-unit sources", joined)

    def test_manifest_targets_must_be_rooted_in_compiled_module_tree(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            crate = root / "crates" / "demo" / "src"
            target = crate / "mechanical_port" / "source" / "renderer" / "owner_hpp.rs"
            target.parent.mkdir(parents=True)
            target.write_text("pub struct Owner;\n")
            (crate / "lib.rs").write_text("pub mod public_api;\n")
            manifest = {
                "translation_unit": [{
                    "id": "owner",
                    "rust_targets": [
                        "crates/demo/src/mechanical_port/source/renderer/owner_hpp.rs"
                    ],
                }]
            }
            errors: list[str] = []
            CHECK.validate_manifest_targets_are_compiled_modules(manifest, root, errors)
            self.assertIn("compiler-inert", "\n".join(errors))

            (crate / "lib.rs").write_text("mod mechanical_port;\n")
            (crate / "mechanical_port.rs").write_text("pub mod source {}\n")
            errors.clear()
            CHECK.validate_manifest_targets_are_compiled_modules(manifest, root, errors)
            self.assertIn("lack the compiled target_inventory module", "\n".join(errors))

            (crate / "mechanical_port.rs").write_text(
                "pub mod source {}\nmod target_inventory;\n"
            )
            inventory = crate / "mechanical_port" / "target_inventory.rs"
            inventory.parent.mkdir(exist_ok=True)
            inventory.write_text("stale\n")
            errors.clear()
            CHECK.validate_manifest_targets_are_compiled_modules(manifest, root, errors)
            self.assertIn("compiled manifest target inventory drifted", "\n".join(errors))

            inventory.write_text("\n".join([
                "//! @generated by the Metal campaign authority; do not edit by hand.",
                "#![allow(unused_imports)]",
                "",
                "use crate::mechanical_port::source::renderer::owner_hpp as _;",
                "",
                "pub(crate) const MANIFEST_TARGET_COUNT: usize = 1;",
                "const _: [(); MANIFEST_TARGET_COUNT] = [(); MANIFEST_TARGET_COUNT];",
                "",
            ]))
            errors.clear()
            CHECK.validate_manifest_targets_are_compiled_modules(manifest, root, errors)
            self.assertEqual(errors, [])

    def test_translation_units_reject_bad_roles_refs_targets_and_cycles(self) -> None:
        manifest = translation_manifest_fixture()
        units = manifest["translation_unit"]
        assert isinstance(units, list)
        first = units[0]
        second = units[1]
        first["dispatch_prerequisites"] = ["ore-context"]
        first["dependencies"] = []
        first["rust_targets"] = ["crates/nuxie-ore-metal/src/./types.rs"]
        first["worker_role"] = "sol-high"
        first["worker_claim"] = "unclaimed"
        second["rust_targets"] = ["crates/nuxie-ore-metal/src/types.rs"]
        second["worker_role"] = "unassigned"
        second["worker_claim"] = "trial-ore-binding-map"
        second["base_ref"] = "b" * 40
        second["requires_lifetime_rows"] = False
        units[3]["rust_targets"] = ["crates/nuxie-renderer/src/ore"]

        errors: list[str] = []
        CHECK.validate_translation_units(manifest, errors)
        joined = "\n".join(errors)
        self.assertIn("dispatch prerequisite cycle", joined)
        self.assertIn("uses ambiguous dependencies", joined)
        self.assertIn("owned by multiple translation units", joined)
        self.assertIn("invalid worker role", joined)
        self.assertIn("base_ref does not match", joined)
        self.assertIn("outside an allowed mechanical source namespace", joined)
        self.assertIn("must be a canonical .rs file", joined)
        self.assertIn("foundation trial unit ore-types must use luna-extra-high", joined)
        self.assertIn("foundation trial unit ore-types has drifted Rust targets", joined)
        self.assertIn("must require lifetime rows", joined)
        self.assertIn("is ready without a worker claim", joined)
        self.assertIn("duplicate worker claims", joined)

    def test_lifetime_ledger_is_pinned_tracked_and_gates_stateful_units(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            repo = root / "repo"
            upstream = root / "upstream"
            subprocess.run(["git", "init", "-q", str(repo)], check=True)
            types = upstream / "renderer/include/rive/renderer/ore/ore_types.hpp"
            rstb = upstream / "renderer/include/rive/renderer/ore/ore_rstb_entry_container.hpp"
            binding_header = upstream / "renderer/include/rive/renderer/ore/ore_binding_map.hpp"
            binding_source = upstream / "renderer/src/ore/ore_binding_map.cpp"
            context = upstream / "renderer/src/ore/metal/ore_context_metal.mm"
            types.parent.mkdir(parents=True)
            binding_source.parent.mkdir(parents=True)
            context.parent.mkdir(parents=True)
            types.write_text("types\n")
            rstb.write_text("rstb\n")
            binding_header.write_text("binding header\n")
            binding_source.write_text("binding source\n")
            context.write_text("context\n")
            ledger = repo / "docs/ore-port-lifetimes.tsv"
            ledger.parent.mkdir(parents=True)
            upstream_ref = "a" * 40
            header = "\t".join(CHECK.LIFETIME_COLUMNS)
            rows = [
                [
                    "1",
                    upstream_ref,
                    "ore-types",
                    "renderer/include/rive/renderer/ore/ore_types.hpp",
                    "TextureDataDesc.data",
                    "borrowed pointer for call",
                    "borrowed slice",
                    "recording thread",
                    "none",
                    "borrow ends when call returns",
                    "invalid span fails before native call",
                    "prepared",
                    "cpp:renderer/include/rive/renderer/ore/ore_types.hpp:1,1",
                ],
                [
                    "1",
                    upstream_ref,
                    "ore-rstb-container",
                    "renderer/include/rive/renderer/ore/ore_rstb_entry_container.hpp",
                    "RstbEntry.source",
                    "owned vector",
                    "Vec<u8>",
                    "construction thread",
                    "none",
                    "released with entry",
                    "malformed container fails closed",
                    "review-needed",
                    "",
                ],
                [
                    "1",
                    upstream_ref,
                    "ore-binding-map",
                    "renderer/src/ore/ore_binding_map.cpp",
                    "fromBlob/toBlob",
                    "behavior-only source",
                    "methods on BindingMap",
                    "construction then immutable",
                    "none",
                    "map owns parsed entries",
                    "malformed map fails closed",
                    "review-needed",
                    "",
                ],
                [
                    "1",
                    upstream_ref,
                    "ore-binding-map",
                    "renderer/include/rive/renderer/ore/ore_binding_map.hpp",
                    "m_entries",
                    "owned vector",
                    "Vec<Entry>",
                    "construction then immutable",
                    "none",
                    "released with map",
                    "malformed map fails closed",
                    "review-needed",
                    "",
                ],
                [
                    "1",
                    upstream_ref,
                    "ore-context",
                    "renderer/src/ore/metal/ore_context_metal.mm",
                    "m_mtlCommandBuffer",
                    "ARC strong",
                    "Option<Retained<MTLCommandBuffer>>",
                    "recording thread",
                    "concrete Metal context owns handle",
                    "released on completion or abandonment",
                    "missing command buffer fails closed",
                    "review-needed",
                    "",
                ],
            ]
            ledger.write_text(
                header + "\n" + "\n".join("\t".join(row) for row in rows) + "\n"
            )
            manifest = translation_manifest_fixture()
            manifest["lifetime_ledger"] = "docs/ore-port-lifetimes.tsv"

            errors: list[str] = []
            CHECK.validate_lifetime_ledger(manifest, repo, upstream, errors)
            self.assertIn("untracked lifetime ledger", "\n".join(errors))

            subprocess.run(
                ["git", "-C", str(repo), "add", "docs/ore-port-lifetimes.tsv"],
                check=True,
            )
            errors.clear()
            CHECK.validate_lifetime_ledger(manifest, repo, upstream, errors)
            self.assertEqual(errors, [])

            proof = repo / "proof.rs"
            proof.write_text("proof\n")
            original_evidence = rows[0][12]
            rows[0][12] = "rust:proof.rs:1"
            ledger.write_text(
                header + "\n" + "\n".join("\t".join(row) for row in rows) + "\n"
            )
            errors.clear()
            CHECK.validate_lifetime_ledger(manifest, repo, upstream, errors)
            self.assertIn("cites untracked Rust evidence proof.rs", "\n".join(errors))
            rows[0][12] = original_evidence

            ledger.write_text(header + "\n" + "\t".join(rows[0]) + "\n")
            errors.clear()
            CHECK.validate_lifetime_ledger(manifest, repo, upstream, errors)
            self.assertIn(
                "translation unit ore-context has no lifetime rows",
                "\n".join(errors),
            )

            rows[0][0] = "2"
            rows[0][1] = "b" * 40
            rows[0][3] = "renderer/src/ore/metal/ore_context_metal.mm"
            rows[0][12] = ""
            rows[0].append("surplus")
            rows[1][11] = "claimed"
            ledger.write_text(
                header + "\n" + "\n".join("\t".join(row) for row in rows) + "\n"
            )
            errors.clear()
            CHECK.validate_lifetime_ledger(manifest, repo, upstream, errors)
            joined = "\n".join(errors)
            self.assertIn("invalid schema version", joined)
            self.assertIn("pin does not match", joined)
            self.assertIn("source is not owned by unit ore-types", joined)
            self.assertIn("prepared without evidence", joined)
            self.assertIn("invalid status `claimed`", joined)
            self.assertIn("has surplus columns", joined)

    def test_reference_provenance_is_bound_to_manifest_and_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            stream = root / "fixtures/scene.rive-stream"
            reference = root / "fixtures/scene.png"
            provenance = root / "fixtures/scene.provenance"
            stream.parent.mkdir(parents=True)
            stream.write_bytes(b"stream")
            reference.write_bytes(b"png")
            runtime_revision = "a" * 40
            input_manifest_sha256 = "b" * 64
            replay_sha256 = "c" * 64
            provenance.write_text(
                "\n".join(
                    [
                        "provenance_schema=1",
                        "renderer_implementation=cpp-native-metal",
                        "capture_tool=renderer-replay-ffi-metal",
                        "backend=metal",
                        "adapter_device=Test Metal Device",
                        "case_id=scene",
                        f"stream_sha256={CHECK.sha256_file(stream)}",
                        f"runtime_revision={runtime_revision}",
                        f"reference_input_manifest_sha256={input_manifest_sha256}",
                        f"replay_sha256={replay_sha256}",
                        f"png_sha256={CHECK.sha256_file(reference)}",
                        "frame=0",
                        "frame_width=64",
                        "frame_height=64",
                        "mode=clockwise-atomic",
                        "sample_count=1",
                    ]
                )
                + "\n"
            )
            subprocess.run(["git", "-C", str(root), "add", "fixtures"], check=True)
            manifest = {
                "upstream_ref": runtime_revision,
                "reference_provenance": [
                    {
                        "id": "scene",
                        "path": "fixtures/scene.provenance",
                        "stream": "fixtures/scene.rive-stream",
                        "reference": "fixtures/scene.png",
                        "renderer_implementation": "cpp-native-metal",
                        "capture_tool": "renderer-replay-ffi-metal",
                        "backend": "metal",
                        "adapter_device": "Test Metal Device",
                        "replay_sha256": replay_sha256,
                        "reference_input_manifest_sha256": input_manifest_sha256,
                        "frame": 0,
                        "frame_width": 64,
                        "frame_height": 64,
                        "mode": "clockwise-atomic",
                        "sample_count": 1,
                    }
                ],
            }

            errors: list[str] = []
            CHECK.validate_reference_provenance(manifest, root, errors)
            self.assertEqual(errors, [])

            stream.write_bytes(b"drifted stream")
            errors.clear()
            CHECK.validate_reference_provenance(manifest, root, errors)
            self.assertIn("stream_sha256", "\n".join(errors))

            stream.write_bytes(b"stream")
            provenance.write_text(
                provenance.read_text().replace(
                    f"replay_sha256={replay_sha256}", f"replay_sha256={'0' * 64}"
                )
            )
            errors.clear()
            CHECK.validate_reference_provenance(manifest, root, errors)
            self.assertIn("replay_sha256", "\n".join(errors))

    def test_scope_expansion_is_exhaustive_and_honors_exclusions(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "renderer/src/metal"
            source.mkdir(parents=True)
            (source / "a.mm").write_text("a")
            (source / "b.h").write_text("b")
            self.assertEqual(
                CHECK.expand_source_scope(
                    root,
                    ["renderer/src/metal/*"],
                    ["renderer/src/metal/b.h"],
                ),
                {"renderer/src/metal/a.mm"},
            )

    def test_render_context_file_map_is_pinned_contiguous_and_nonoverlapping(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            repo = root / "repo"
            upstream = root / "upstream"
            subprocess.run(["git", "init", "-q", str(repo)], check=True)
            source = upstream / "renderer/src/metal/render_context_metal_impl.mm"
            header = (
                upstream
                / "renderer/include/rive/renderer/metal/render_context_metal_impl.h"
            )
            source.parent.mkdir(parents=True)
            header.parent.mkdir(parents=True)
            source.write_text("one\ntwo\nthree\n")
            header.write_text("one\ntwo\n")
            rust_owner = repo / "owner.rs"
            rust_owner.write_text("owner\n")
            file_map = repo / "docs/render-context-metal-file-map.tsv"
            file_map.parent.mkdir(parents=True)
            columns = "\t".join(CHECK.RENDER_CONTEXT_FILE_MAP_COLUMNS)
            upstream_ref = "a" * 40
            rows = [
                [
                    "1",
                    upstream_ref,
                    "renderer/src/metal/render_context_metal_impl.mm",
                    "1-2",
                    "first",
                    "partial",
                    "owner.rs",
                    "-",
                ],
                [
                    "1",
                    upstream_ref,
                    "renderer/src/metal/render_context_metal_impl.mm",
                    "3-3",
                    "second",
                    "missing",
                    "-",
                    "not ported",
                ],
                [
                    "1",
                    upstream_ref,
                    "renderer/include/rive/renderer/metal/render_context_metal_impl.h",
                    "1-2",
                    "header",
                    "partial",
                    "owner.rs",
                    "remaining",
                ],
            ]

            def write_map() -> None:
                file_map.write_text(
                    columns + "\n" + "\n".join("\t".join(row) for row in rows) + "\n"
                )

            write_map()
            subprocess.run(
                ["git", "-C", str(repo), "add", "owner.rs", "docs"], check=True
            )
            manifest = {
                "upstream_ref": upstream_ref,
                "render_context_file_map": "docs/render-context-metal-file-map.tsv",
            }
            errors: list[str] = []
            CHECK.validate_render_context_file_map(manifest, repo, upstream, errors)
            self.assertEqual(errors, [])

            rows[1][3] = "2-3"
            write_map()
            errors.clear()
            CHECK.validate_render_context_file_map(manifest, repo, upstream, errors)
            self.assertIn("expected 3", "\n".join(errors))

            rows[1][3] = "4-4"
            write_map()
            errors.clear()
            CHECK.validate_render_context_file_map(manifest, repo, upstream, errors)
            joined = "\n".join(errors)
            self.assertIn("expected 3", joined)
            self.assertIn("ends outside", joined)

    def test_render_context_field_rows_must_exactly_cover_extracted_declarations(self) -> None:
        declarations = {
            ("renderer/header.h", "Owner", "m_device"): ("Device *", 10, "all"),
            ("renderer/header.h", "Owner", "m_queue"): ("Queue", 11, "WITH_TOOLS"),
        }
        rows = [
            {
                "upstream_file": "renderer/header.h",
                "cpp_type": "Owner",
                "cpp_field": "m_device",
                "cpp_declared_type": "Device *",
                "declaration_line": "10",
                "configuration": "all",
            },
            {
                "upstream_file": "renderer/header.h",
                "cpp_type": "Owner",
                "cpp_field": "m_queue",
                "cpp_declared_type": "Queue",
                "declaration_line": "11",
                "configuration": "WITH_TOOLS",
            },
        ]
        errors: list[str] = []
        CHECK.compare_render_context_field_rows(rows, declarations, errors)
        self.assertEqual(errors, [])

        rows.pop()
        errors.clear()
        CHECK.compare_render_context_field_rows(rows, declarations, errors)
        self.assertIn("omits declarations", "\n".join(errors))

        rows.append(
            {
                "upstream_file": "renderer/header.h",
                "cpp_type": "Owner",
                "cpp_field": "m_invented",
                "cpp_declared_type": "int",
                "declaration_line": "12",
                "configuration": "all",
            }
        )
        errors.clear()
        CHECK.compare_render_context_field_rows(rows, declarations, errors)
        joined = "\n".join(errors)
        self.assertIn("omits declarations", joined)
        self.assertIn("invents declarations", joined)

        rows[0]["declaration_line"] = "99"
        errors.clear()
        CHECK.compare_render_context_field_rows(
            rows[:1], {next(iter(declarations)): ("Device *", 10, "all")}, errors
        )
        self.assertIn("expected 10", "\n".join(errors))

        rows[0]["declaration_line"] = "10"
        rows[0]["configuration"] = "DEBUG"
        errors.clear()
        CHECK.compare_render_context_field_rows(
            rows[:1], {next(iter(declarations)): ("Device *", 10, "all")}, errors
        )
        self.assertIn("expected 'all'", "\n".join(errors))

    def test_render_context_field_extractor_covers_dependency_owner_families(self) -> None:
        upstream = configured_upstream()
        errors: list[str] = []
        declarations = CHECK.extract_render_context_field_declarations(upstream, errors)
        self.assertEqual(errors, [])
        self.assertEqual(len(declarations), 455)
        source_counts = collections.Counter(key[0] for key in declarations)
        self.assertEqual(source_counts["renderer/include/rive/renderer/gpu.hpp"], 208)
        self.assertEqual(source_counts["renderer/include/rive/renderer/render_context.hpp"], 142)
        self.assertEqual(
            declarations[("include/rive/renderer.hpp", "RenderBuffer", "m_mapCount")][2],
            "DEBUG",
        )

    def test_render_context_field_extractor_rejects_source_line_drift(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        extractor = repo_root / "tools/metal-port/extract_field_authority.py"
        owners = repo_root / CHECK.RENDER_CONTEXT_FIELD_OWNER_INPUT
        sources = repo_root / CHECK.RENDER_CONTEXT_FIELD_SOURCE_INPUT
        with tempfile.TemporaryDirectory() as temporary:
            bad_sources = pathlib.Path(temporary) / "sources.tsv"
            text = sources.read_text(encoding="utf-8")
            source_hash = text.splitlines()[1].split("\t")[-1]
            bad_sources.write_text(text.replace(source_hash, "0" * 64, 1), encoding="utf-8")
            process = subprocess.run(
                ["python3", str(extractor), "--upstream-root", str(configured_upstream()), "--owners", str(owners), "--sources", str(bad_sources)],
                text=True,
                capture_output=True,
            )
        self.assertNotEqual(process.returncode, 0)
        self.assertIn("field-owner input hash drifted", process.stderr)

    def test_render_context_field_extractor_is_pristine_checkout_hermetic(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        upstream_repo = configured_upstream()
        with tempfile.TemporaryDirectory() as temporary:
            pristine = pathlib.Path(temporary) / "rive-runtime"
            pristine.mkdir()
            archive = subprocess.Popen(
                ["git", "-C", str(upstream_repo), "archive", CHECK.read_toml(repo_root / "docs/metal-port-manifest.toml")["upstream_ref"]],
                stdout=subprocess.PIPE,
            )
            assert archive.stdout is not None
            unpack = subprocess.run(
                ["tar", "-x", "-C", str(pristine)], stdin=archive.stdout, check=True
            )
            archive.stdout.close()
            self.assertEqual(archive.wait(), 0)
            self.assertEqual(unpack.returncode, 0)
            errors: list[str] = []
            declarations = CHECK.extract_render_context_field_declarations(
                pristine, errors
            )
        self.assertEqual(errors, [])
        self.assertEqual(len(declarations), 455)
        self.assertIn(
            (
                "renderer/include/rive/renderer/metal/render_context_metal_impl.h",
                "RenderContextMetalImpl::MetalFeatures",
                "atomicBarrierType",
            ),
            declarations,
        )

    def test_render_context_configuration_rows_cover_blocks_and_branches(self) -> None:
        blocks = {
            ("renderer/source.mm", 3, 9): (3, 5, 7),
            ("renderer/source.mm", 12, 14): (12,),
        }
        rows = [
            {
                "upstream_file": "renderer/source.mm",
                "lines": "3-9",
                "branch_lines": "3,5,7",
            },
            {
                "upstream_file": "renderer/source.mm",
                "lines": "12-14",
                "branch_lines": "12",
            },
        ]
        errors: list[str] = []
        CHECK.compare_render_context_configuration_rows(rows, blocks, errors)
        self.assertEqual(errors, [])

        rows.pop()
        errors.clear()
        CHECK.compare_render_context_configuration_rows(rows, blocks, errors)
        self.assertIn("omits blocks", "\n".join(errors))

        rows[0]["branch_lines"] = "3,7"
        errors.clear()
        CHECK.compare_render_context_configuration_rows(rows, {next(iter(blocks)): (3, 5, 7)}, errors)
        self.assertIn("expected (3, 5, 7)", "\n".join(errors))

        rows[0]["lines"] = "bad"
        errors.clear()
        CHECK.compare_render_context_configuration_rows(rows, blocks, errors)
        self.assertIn("invalid range", "\n".join(errors))

    def test_exhaustive_authority_ledgers_fail_closed_on_every_mutation(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        upstream = configured_upstream()
        with (repo_root / "docs/metal-port-manifest.toml").open("rb") as source:
            manifest = tomllib.load(source)
        expected = CHECK.load_authority_builder().build(repo_root, upstream)

        with tempfile.TemporaryDirectory() as temporary:
            temporary_root = pathlib.Path(temporary)
            for relative, content in expected.items():
                path = temporary_root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding="utf-8")

            errors: list[str] = []
            CHECK.compare_exhaustive_authority_ledgers(
                manifest,
                temporary_root,
                expected,
                errors,
                require_tracked=False,
            )
            self.assertEqual(errors, [])

            for relative, content in expected.items():
                with self.subTest(relative=relative):
                    path = temporary_root / relative
                    lines = content.splitlines(keepends=True)
                    path.write_text("".join(lines[:-1]), encoding="utf-8")
                    errors.clear()
                    CHECK.compare_exhaustive_authority_ledgers(
                        manifest,
                        temporary_root,
                        expected,
                        errors,
                        require_tracked=False,
                    )
                    self.assertIn(
                        f"exhaustive authority ledger drifted from pinned sources: {relative}",
                        errors,
                    )
                    path.write_text(content, encoding="utf-8")

            include_relative = CHECK.EXHAUSTIVE_AUTHORITY_LEDGERS[
                "direct_include_authority"
            ]
            include_path = temporary_root / include_relative
            include_content = expected[include_relative]
            include_lines = include_content.splitlines(keepends=True)
            import_index = next(
                index
                for index, line in enumerate(include_lines)
                if "\timport\t" in line
            )
            for mutated in (
                "".join(include_lines[:import_index] + include_lines[import_index + 1 :]),
                include_content + include_lines[import_index],
            ):
                include_path.write_text(mutated, encoding="utf-8")
                errors.clear()
                CHECK.compare_exhaustive_authority_ledgers(
                    manifest,
                    temporary_root,
                    expected,
                    errors,
                    require_tracked=False,
                )
                self.assertIn(
                    "exhaustive authority ledger drifted from pinned sources: "
                    f"{include_relative}",
                    errors,
                )
            include_path.write_text(include_content, encoding="utf-8")

            bad_manifest = dict(manifest)
            bad_manifest.pop("source_dependency_authority")
            errors.clear()
            CHECK.compare_exhaustive_authority_ledgers(
                bad_manifest,
                temporary_root,
                expected,
                errors,
                require_tracked=False,
            )
            self.assertTrue(
                any("source_dependency_authority must be" in error for error in errors)
            )

    def test_include_authority_covers_imports_and_exact_global_correspondence(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        with (repo_root / "docs/metal-port-include-authority.tsv").open(
            encoding="utf-8", newline=""
        ) as source:
            includes = list(csv.DictReader(source, delimiter="\t"))
        with (repo_root / "docs/metal-port-source-dependencies.tsv").open(
            encoding="utf-8", newline=""
        ) as source:
            dependencies = list(csv.DictReader(source, delimiter="\t"))

        self.assertEqual(len(includes), 366)
        self.assertEqual(len({row["include_token"] for row in includes}), 142)
        self.assertEqual(len({row["upstream_file"] for row in includes}), 74)
        self.assertEqual(
            collections.Counter(row["directive"] for row in includes),
            {"include": 351, "import": 15},
        )
        global_rows = [
            row
            for row in includes
            if row["resolution_kind"] == "upstream-global-source"
        ]
        self.assertEqual(len(global_rows), 58)
        self.assertEqual(len({row["resolved_source"] for row in global_rows}), 31)
        for row in global_rows:
            self.assertEqual(row["mapping_status"], "existing-complete")
            self.assertTrue(row["dependency_unit"].startswith("existing-rust:"))
            self.assertTrue((repo_root / row["correspondence_owner"]).is_file())
            self.assertTrue(row["correspondence_evidence"].startswith("rust:"))
        self.assertEqual(len(dependencies), 359)
        global_edges = [
            row
            for row in dependencies
            if row["unit_edge_status"] == "existing-rust-correspondence"
        ]
        self.assertEqual(len(global_edges), 58)
        self.assertFalse(
            any(row["unit_edge_status"] == "global-source-boundary" for row in dependencies)
        )

    def test_authority_builders_extractors_and_inputs_must_be_tracked(self) -> None:
        repo_root = MODULE_PATH.parents[2]
        upstream = configured_upstream()
        with (repo_root / "docs/metal-port-manifest.toml").open("rb") as source:
            manifest = tomllib.load(source)

        exhaustive_paths = [
            "tools/metal-port/build_authority_ledgers.py",
            *(path.as_posix() for path in CHECK.EXHAUSTIVE_AUTHORITY_LEDGERS.values()),
        ]
        for untracked in exhaustive_paths:
            with self.subTest(untracked=untracked):
                errors: list[str] = []
                with mock.patch.object(
                    CHECK,
                    "git_tracked_file",
                    side_effect=lambda _root, path: path != untracked,
                ):
                    CHECK.validate_exhaustive_authority_ledgers(
                        manifest, repo_root, upstream, errors
                    )
                self.assertTrue(
                    any(untracked in error and "untracked" in error for error in errors),
                    errors,
                )

        for untracked in (
            "docs/render-context-metal-field-owners.tsv",
            "tools/metal-port/extract_field_authority.py",
        ):
            with self.subTest(untracked=untracked):
                errors = []
                with mock.patch.object(
                    CHECK,
                    "git_tracked_file",
                    side_effect=lambda _root, path: path != untracked,
                ):
                    CHECK.validate_render_context_field_map(
                        manifest, repo_root, upstream, errors
                    )
                self.assertIn(
                    f"untracked render-context field authority {untracked}",
                    errors,
                )

    def test_translation_convention_ids_are_exhaustive_and_unique(self) -> None:
        ids = sorted(CHECK.TRANSLATION_CONVENTION_IDS)
        errors: list[str] = []
        CHECK.compare_translation_convention_ids(ids, errors)
        self.assertEqual(errors, [])

        CHECK.compare_translation_convention_ids(ids[:-1], errors)
        self.assertIn("omit", "\n".join(errors))

        errors.clear()
        CHECK.compare_translation_convention_ids(ids + [ids[0], "invented"], errors)
        joined = "\n".join(errors)
        self.assertIn("duplicate", joined)
        self.assertIn("invent", joined)

    def test_missing_upstream_source_and_unproved_port_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            upstream = root / "upstream"
            repo = root / "repo"
            (upstream / "renderer/src/metal").mkdir(parents=True)
            repo.mkdir()
            (upstream / "renderer/src/metal/a.mm").write_text("a")
            manifest = {
                "source_globs": ["renderer/src/metal/*"],
                "source_excludes": [],
                "source": [
                    {
                        "upstream": "renderer/src/metal/extra.mm",
                        "status": "ported",
                        "issue": "UNIV-2086",
                        "lane": "renderer-platform",
                        "rust_modules": [],
                        "evidence": [],
                    }
                ],
            }
            errors: list[str] = []
            CHECK.validate_source_rows(manifest, repo, upstream, errors)
            joined = "\n".join(errors)
            self.assertIn("untracked upstream Metal sources", joined)
            self.assertIn("out-of-scope source rows", joined)
            self.assertIn("without a Rust module", joined)
            self.assertIn("without verification evidence", joined)

    def test_citations_are_line_checked(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "source.mm"
            source.write_text("one\ntwo\n")
            errors: list[str] = []
            CHECK.validate_citation("cpp:source.mm:1-2", root, root, errors)
            self.assertEqual(errors, [])
            CHECK.validate_citation("cpp:source.mm:3", root, root, errors)
            self.assertIn("citation line is outside", errors[-1])

    def test_ownership_promotion_requires_existing_evidence_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "source.mm"
            source.write_text("source\n")
            ownership = {
                "owner": [
                    {
                        "id": "renderer.device",
                        "issue": "UNIV-2086",
                        "status": "ported",
                        "required_tests": ["device lifetime"],
                        "citations": ["cpp:source.mm:1"],
                        "evidence_paths": ["tests/missing.rs"],
                    }
                ]
            }
            errors: list[str] = []
            CHECK.validate_owner_rows(ownership, root, root, errors)
            self.assertIn("names missing evidence path", "\n".join(errors))

            ownership["owner"][0]["evidence_paths"] = []
            errors.clear()
            CHECK.validate_owner_rows(ownership, root, root, errors)
            self.assertIn("without concrete evidence paths", "\n".join(errors))

    def test_ownership_promotion_rejects_untracked_evidence_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            source = root / "source.mm"
            source.write_text("source\n")
            evidence = root / "tests/evidence.rs"
            evidence.parent.mkdir()
            evidence.write_text("evidence\n")
            ownership = {
                "owner": [
                    {
                        "id": "renderer.device",
                        "issue": "UNIV-2086",
                        "status": "verified",
                        "required_tests": ["device lifetime"],
                        "citations": ["cpp:source.mm:1"],
                        "evidence_paths": ["tests/evidence.rs"],
                    }
                ]
            }
            errors: list[str] = []
            CHECK.validate_owner_rows(ownership, root, root, errors)
            self.assertIn("names untracked evidence path", "\n".join(errors))

            subprocess.run(
                ["git", "-C", str(root), "add", "tests/evidence.rs"], check=True
            )
            errors.clear()
            CHECK.validate_owner_rows(ownership, root, root, errors)
            self.assertNotIn("evidence path", "\n".join(errors))

    def test_ownership_promotion_cannot_outrun_cited_unit_receipts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            subprocess.run(["git", "init", "-q", str(root)], check=True)
            source = root / "source.mm"
            evidence = root / "evidence.rs"
            source.write_text("source\n")
            evidence.write_text("evidence\n")
            subprocess.run(["git", "-C", str(root), "add", "evidence.rs"], check=True)
            ownership = {
                "owner": [{
                    "id": "renderer.device",
                    "issue": "UNIV-2086",
                    "status": "ported",
                    "required_tests": ["device lifetime"],
                    "citations": ["cpp:source.mm:1"],
                    "evidence_paths": ["evidence.rs"],
                }]
            }
            manifest = {
                "translation_unit": [{
                    "id": "source-unit",
                    "sources": ["source.mm"],
                    "status": "pending",
                }]
            }
            errors: list[str] = []
            CHECK.validate_owner_rows(ownership, root, root, errors, manifest)
            self.assertIn("promotion outruns unit receipts", "\n".join(errors))

    def test_source_promotion_requires_tracked_modules_and_distinct_parity_evidence(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            upstream = root / "upstream"
            repo = root / "repo"
            source = upstream / "renderer/src/metal/a.mm"
            source.parent.mkdir(parents=True)
            source.write_text("source\n")
            subprocess.run(["git", "init", "-q", str(repo)], check=True)
            module = repo / "src/metal.rs"
            module.parent.mkdir(parents=True)
            module.write_text("module\n")
            evidence = repo / "tests/metal.rs"
            evidence.parent.mkdir()
            evidence.write_text("evidence\n")
            parity = repo / "docs/evidence/UNIV-2086.md"
            parity.parent.mkdir(parents=True)
            parity.write_text("parity\n")
            manifest = {
                "source_globs": ["renderer/src/metal/*"],
                "source_excludes": [],
                "source": [
                    {
                        "upstream": "renderer/src/metal/a.mm",
                        "status": "verified",
                        "issue": "UNIV-2086",
                        "lane": "renderer-platform",
                        "rust_modules": ["src/metal.rs"],
                        "evidence": ["tests/metal.rs"],
                        "parity_evidence": [],
                    }
                ],
            }
            errors: list[str] = []
            CHECK.validate_source_rows(manifest, repo, upstream, errors)
            joined = "\n".join(errors)
            self.assertIn("names untracked Rust module", joined)
            self.assertIn("names untracked evidence path", joined)
            self.assertIn("verified without parity evidence", joined)

            subprocess.run(
                [
                    "git",
                    "-C",
                    str(repo),
                    "add",
                    "src/metal.rs",
                    "tests/metal.rs",
                    "docs/evidence/UNIV-2086.md",
                ],
                check=True,
            )
            manifest["source"][0]["parity_evidence"] = [
                "docs/evidence/UNIV-2086.md"
            ]
            errors.clear()
            CHECK.validate_source_rows(manifest, repo, upstream, errors)
            self.assertEqual(errors, [])


if __name__ == "__main__":
    unittest.main()
