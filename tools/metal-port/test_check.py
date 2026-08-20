from __future__ import annotations

import importlib.util
import pathlib
import subprocess
import tempfile
import unittest


MODULE_PATH = pathlib.Path(__file__).with_name("check.py")
SPEC = importlib.util.spec_from_file_location("metal_port_check", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)


def translation_manifest_fixture() -> dict[str, object]:
    upstream_ref = "a" * 40
    return {
        "upstream_ref": upstream_ref,
        "source": [
            {
                "upstream": "renderer/include/rive/renderer/ore/ore_types.hpp",
                "lane": "ore-metal",
                "status": "pending",
            },
            {
                "upstream": "renderer/include/rive/renderer/ore/ore_rstb_entry_container.hpp",
                "lane": "ore-metal",
                "status": "pending",
            },
            {
                "upstream": "renderer/include/rive/renderer/ore/ore_binding_map.hpp",
                "lane": "ore-metal",
                "status": "pending",
            },
            {
                "upstream": "renderer/src/ore/ore_binding_map.cpp",
                "lane": "ore-metal",
                "status": "pending",
            },
            {
                "upstream": "renderer/src/ore/metal/ore_context_metal.mm",
                "lane": "ore-metal",
                "status": "pending",
            },
        ],
        "translation_unit": [
            {
                "id": "ore-types",
                "phase": "trial",
                "sources": ["renderer/include/rive/renderer/ore/ore_types.hpp"],
                "dependencies": [],
                "rust_targets": ["crates/nuxie-ore-metal/src/types.rs"],
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
                "dependencies": [],
                "rust_targets": [
                    "crates/nuxie-ore-metal/src/rstb_entry_container.rs"
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
                "dependencies": [],
                "rust_targets": ["crates/nuxie-ore-metal/src/binding_map.rs"],
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
                "dependencies": ["ore-types"],
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


class MetalPortCheckTests(unittest.TestCase):
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
        self.assertIn("missing pending ORE sources", joined)
        self.assertIn("overlapping translation-unit sources", joined)

    def test_translation_units_reject_bad_roles_refs_targets_and_cycles(self) -> None:
        manifest = translation_manifest_fixture()
        units = manifest["translation_unit"]
        assert isinstance(units, list)
        first = units[0]
        second = units[1]
        first["dependencies"] = ["ore-context"]
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
        self.assertIn("dependency cycle", joined)
        self.assertIn("owned by multiple translation units", joined)
        self.assertIn("invalid worker role", joined)
        self.assertIn("base_ref does not match", joined)
        self.assertIn("outside crates/nuxie-ore-metal/src", joined)
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
                    "ported",
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
            ("renderer/header.h", "Owner", "m_device"): 10,
            ("renderer/header.h", "Owner", "m_queue"): 11,
        }
        rows = [
            {
                "upstream_file": "renderer/header.h",
                "cpp_type": "Owner",
                "cpp_field": "m_device",
                "declaration_line": "10",
            },
            {
                "upstream_file": "renderer/header.h",
                "cpp_type": "Owner",
                "cpp_field": "m_queue",
                "declaration_line": "11",
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
                "declaration_line": "12",
            }
        )
        errors.clear()
        CHECK.compare_render_context_field_rows(rows, declarations, errors)
        joined = "\n".join(errors)
        self.assertIn("omits declarations", joined)
        self.assertIn("invents declarations", joined)

        rows[0]["declaration_line"] = "99"
        errors.clear()
        CHECK.compare_render_context_field_rows(rows[:1], {next(iter(declarations)): 10}, errors)
        self.assertIn("expected 10", "\n".join(errors))

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
