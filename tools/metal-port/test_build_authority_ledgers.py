#!/usr/bin/env python3

from __future__ import annotations

import csv
import importlib.util
import os
import pathlib
import subprocess
import sys
import tempfile
import tomllib
import unittest
from contextlib import contextmanager


REPO = pathlib.Path(__file__).resolve().parents[2]
MODULE_PATH = pathlib.Path(__file__).with_name("build_authority_ledgers.py")
SPEC = importlib.util.spec_from_file_location("build_authority_ledgers", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
AUTHORITY = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = AUTHORITY
SPEC.loader.exec_module(AUTHORITY)


@contextmanager
def pristine_upstream() -> pathlib.Path:
    configured = os.environ.get("RIVE_RUNTIME_DIR")
    if not configured:
        raise unittest.SkipTest("set RIVE_RUNTIME_DIR to run pinned upstream extraction")
    with tempfile.TemporaryDirectory() as directory:
        checkout = pathlib.Path(directory) / "rive-runtime"
        subprocess.run(
            ["git", "clone", "--shared", "--no-checkout", configured, str(checkout)],
            check=True,
            capture_output=True,
            text=True,
        )
        subprocess.run(
            ["git", "checkout", "--detach", AUTHORITY.PIN],
            cwd=checkout,
            check=True,
            capture_output=True,
            text=True,
        )
        yield checkout


def rows(relative: str) -> list[dict[str, str]]:
    with (REPO / relative).open(encoding="utf-8", newline="") as source:
        return list(csv.DictReader(source, delimiter="\t"))


class AuthorityLedgerTests(unittest.TestCase):
    def test_committed_ledgers_equal_machine_derived_authority(self) -> None:
        with pristine_upstream() as upstream:
            self.assertFalse((upstream / "renderer/src/shaders/out").exists())
            rendered = AUTHORITY.build(REPO, upstream)
            for relative, expected in rendered.items():
                with self.subTest(relative=relative):
                    self.assertEqual(
                        (REPO / relative).read_text(encoding="utf-8"), expected
                    )

    def test_preprocessor_denominator_and_special_lanes_are_exact(self) -> None:
        actual = rows("docs/metal-port-preprocessor-authority.tsv")
        self.assertEqual(len(actual), 845)
        self.assertEqual(len({row["block_id"] for row in actual}), 634)
        ore_files = {row["upstream_file"] for row in actual if "/ore/" in row["upstream_file"]}
        shader_files = {
            row["upstream_file"]
            for row in actual
            if pathlib.PurePosixPath(row["upstream_file"]).suffix
            in {".glsl", ".vert", ".frag"}
        }
        self.assertEqual(len(ore_files), 5)
        self.assertEqual(len(shader_files), 33)
        nested = {
            (
                row["upstream_file"],
                int(row["block_start"]),
                int(row["block_end"]),
                int(row["branch_line"]),
            )
            for row in actual
            if int(row["branch_line"]) in {136, 139, 604, 606, 607, 612}
        }
        self.assertTrue(
            {
                ("renderer/src/shaders/glsl.glsl", 136, 141, 136),
                ("renderer/src/shaders/glsl.glsl", 136, 141, 139),
                ("renderer/src/shaders/glsl.glsl", 604, 615, 604),
                ("renderer/src/shaders/glsl.glsl", 604, 615, 606),
                ("renderer/src/shaders/glsl.glsl", 607, 614, 607),
                ("renderer/src/shaders/glsl.glsl", 607, 614, 612),
            }
            <= nested
        )

    def test_preprocessor_directives_allow_hostile_whitespace_after_hash(self) -> None:
        source = """# if OUTER
#\t if INNER
#   else
#\t endif
# elif OTHER
# endif
"""
        blocks, _includes = AUTHORITY.parse_source_structure("hostile.glsl", source)
        self.assertEqual(len(blocks), 2)
        self.assertEqual(sum(len(block.branches) for block in blocks), 4)
        self.assertEqual({block.end for block in blocks}, {4, 6})

    def test_python_hash_comments_are_not_preprocessor_directives(self) -> None:
        blocks, _includes = AUTHORITY.parse_source_structure(
            "generator.py", "#         if not a Python conditional:\n"
        )
        self.assertEqual(blocks, [])

    def test_authority_translation_status_tracks_receipt_gated_unit_state(self) -> None:
        unit = {"status": "pending"}
        self.assertEqual(AUTHORITY.authority_translation_status(unit), "pending")
        unit["status"] = "translated"
        self.assertEqual(AUTHORITY.authority_translation_status(unit), "translated")
        unit["status"] = "compiled"
        self.assertEqual(AUTHORITY.authority_translation_status(unit), "translated")
        unit["status"] = "verified"
        self.assertEqual(AUTHORITY.authority_translation_status(unit), "verified")

    def test_include_and_dependency_denominators_are_exact(self) -> None:
        includes = rows("docs/metal-port-include-authority.tsv")
        dependencies = rows("docs/metal-port-source-dependencies.tsv")
        self.assertEqual(len(includes), 366)
        self.assertEqual(len({row["include_token"] for row in includes}), 142)
        self.assertEqual(len({row["upstream_file"] for row in includes}), 74)
        self.assertEqual(len(dependencies), 359)
        self.assertEqual(
            sum(row["directive"] == "import" for row in includes), 15
        )
        source_only = [
            row for row in dependencies if row["unit_edge_status"] == "source-only-dependency"
        ]
        self.assertEqual(sum(int(row["occurrence_count"]) for row in source_only), 36)
        self.assertEqual(
            len({(row["source_unit"], row["dependency_unit"]) for row in source_only}),
            29,
        )

    def test_import_removal_changes_directive_authority(self) -> None:
        source = '#include "local.hpp"\n#import <Metal/Metal.h>\n'
        _blocks, directives = AUTHORITY.parse_source_structure("owner.mm", source)
        self.assertEqual([row[1] for row in directives], ["include", "import"])
        _blocks, removed = AUTHORITY.parse_source_structure(
            "owner.mm", source.replace("#import <Metal/Metal.h>\n", "")
        )
        self.assertEqual(len(removed), len(directives) - 1)

    def test_import_addition_changes_directive_authority(self) -> None:
        source = '#include "local.hpp"\n'
        _blocks, directives = AUTHORITY.parse_source_structure("owner.mm", source)
        _blocks, added = AUTHORITY.parse_source_structure(
            "owner.mm", source + '# import "second.hpp"\n'
        )
        self.assertEqual(len(added), len(directives) + 1)
        self.assertEqual(added[-1][1], "import")

    def test_pinned_authority_rejects_import_removal_and_addition(self) -> None:
        manifest = tomllib.loads(
            (REPO / "docs/metal-port-manifest.toml").read_text(encoding="utf-8")
        )
        with pristine_upstream() as upstream:
            sources = AUTHORITY.source_scope(manifest, upstream)
            path = upstream / "renderer/include/rive/renderer/metal/render_context_metal_impl.h"
            original = path.read_text(encoding="utf-8")
            removed = original.replace("#import <Metal/Metal.h>", "", 1)
            self.assertNotEqual(removed, original)
            path.write_text(removed, encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "direct include count: expected 366, got 365"):
                AUTHORITY.collect_authority(manifest, upstream, sources)
            path.write_text(
                original + "\n#import <Foundation/Foundation.h>\n", encoding="utf-8"
            )
            with self.assertRaisesRegex(ValueError, "direct include count: expected 366, got 367"):
                AUTHORITY.collect_authority(manifest, upstream, sources)

    def test_upstream_global_edges_have_exact_existing_rust_correspondence(self) -> None:
        includes = rows("docs/metal-port-include-authority.tsv")
        global_rows = [
            row
            for row in includes
            if row["resolution_kind"] == "upstream-global-source"
        ]
        self.assertEqual(len(global_rows), 58)
        self.assertEqual(len({row["resolved_source"] for row in global_rows}), 31)
        for row in global_rows:
            self.assertEqual(row["mapping_status"], "existing-complete")
            self.assertEqual(
                row["translation_disposition"],
                "reuse-exact-existing-rust-correspondence",
            )
            self.assertTrue(row["dependency_unit"].startswith("existing-rust:"))
            owner = row["correspondence_owner"]
            self.assertTrue((REPO / owner).is_file())
            self.assertTrue(row["correspondence_evidence"].startswith(f"rust:{owner}:"))
        dependencies = rows("docs/metal-port-source-dependencies.tsv")
        global_edges = [
            row
            for row in dependencies
            if row["unit_edge_status"] == "existing-rust-correspondence"
        ]
        self.assertEqual(len(global_edges), 58)
        self.assertTrue(all(row["correspondence_owner"] != "-" for row in global_edges))
        self.assertTrue(
            all(row["correspondence_evidence"].startswith("rust:") for row in global_edges)
        )

    def test_source_scc_is_not_used_as_dispatch_ordering(self) -> None:
        dispatch = rows("docs/metal-port-dispatch-prerequisites.tsv")
        scc_members = {
            row["translation_unit"]
            for row in dispatch
            if row["source_dependency_scc"] != "-"
        }
        self.assertEqual(
            scc_members,
            {
                "generic-render-context-contract",
                "generic-render-context-implementation",
                "generic-render-context-helper",
                "generic-render-context-impl-contract",
                "generic-rive-render-factory",
                "generic-rive-renderer",
                "metal-render-context-api",
                "ore-bind-group",
                "ore-buffer",
                "ore-context-render-pass",
            },
        )
        for row in dispatch:
            self.assertEqual(row["ordering_contract"], "acyclic-dispatch-only")
        self.assertEqual(
            [row["translation_unit"] for row in dispatch],
            list(AUTHORITY.DISPATCH_ORDER),
        )
        by_id = {row["translation_unit"]: row for row in dispatch}
        for row in dispatch:
            consumer_ordinal = int(row["dispatch_ordinal"])
            prerequisites = (
                []
                if row["dispatch_prerequisites"] == "-"
                else row["dispatch_prerequisites"].split(";")
            )
            for prerequisite in prerequisites:
                self.assertLess(
                    int(by_id[prerequisite]["dispatch_ordinal"]), consumer_ordinal
                )

    def test_build_behavior_covers_make_and_python_branches(self) -> None:
        actual = rows("docs/metal-port-build-branch-authority.tsv")
        make = [row for row in actual if row["authority_kind"] == "make-rule-family"]
        minify = [row for row in actual if row["upstream_file"].endswith("/minify.py")]
        generator = [
            row
            for row in actual
            if row["upstream_file"].endswith("/generate_draw_combinations.py")
        ]
        self.assertEqual(len(make), 12)
        self.assertEqual(sum(row["entry_id"].startswith("apple-") for row in make), 7)
        self.assertEqual((sum(row["branch_kind"] == "If" for row in minify), sum(row["branch_kind"] == "IfExp" for row in minify)), (36, 6))
        self.assertEqual((sum(row["branch_kind"] == "If" for row in generator), sum(row["branch_kind"] == "IfExp" for row in generator)), (10, 1))
        self.assertEqual(
            {
                row["option_family"]
                for row in minify
                if row["option_family"] != "algorithmic"
            },
            {
                "human-readable-output",
                "lexer-provider-path",
                "msvc-header-output",
                "output-directory",
            },
        )


if __name__ == "__main__":
    unittest.main()
