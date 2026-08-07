#!/usr/bin/env python3
"""Focused controls for the pure-runtime boundary guard; no C++ counterpart."""

from __future__ import annotations

import importlib.util
import pathlib
import subprocess
import sys
import tempfile
import textwrap
import unittest


TOOL = pathlib.Path(__file__).with_name("check.py")
REPO_ROOT = TOOL.parents[2]
TOOL_SPEC = importlib.util.spec_from_file_location("pure_runtime_boundary_tool", TOOL)
assert TOOL_SPEC is not None and TOOL_SPEC.loader is not None
BOUNDARY_TOOL = importlib.util.module_from_spec(TOOL_SPEC)
TOOL_SPEC.loader.exec_module(BOUNDARY_TOOL)


class PureRuntimeBoundaryCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.members = ["crates/nuxie-runtime"]
        self.package = self.create_package(
            "crates/nuxie-runtime", "nuxie-runtime", ""
        )
        (self.package / "src/lib.rs").write_text("// parity baseline\n")

    def write_workspace(self, workspace_dependencies: str = "") -> None:
        members = ",\n".join(f'    "{member}"' for member in self.members)
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                f"""\
                [workspace]
                members = [
                {members}
                ]
                resolver = "3"

                {workspace_dependencies}
                """
            )
        )

    def create_package(
        self, relative: str, name: str, manifest_body: str
    ) -> pathlib.Path:
        package = self.root / relative
        (package / "src").mkdir(parents=True)
        (package / "src/lib.rs").write_text("// parity baseline\n")
        (package / "Cargo.toml").write_text(
            textwrap.dedent(
                f"""\
                [package]
                name = "{name}"
                version = "0.0.0"

                {manifest_body}
                """
            )
        )
        if relative not in self.members:
            self.members.append(relative)
        self.write_workspace()
        return package

    def create_portable_abi_facade(self) -> None:
        renderer = self.create_package(
            "crates/nuxie-renderer",
            "nuxie-renderer",
            "",
        )
        (renderer / "src/lib.rs").write_text("// portable renderer facade\n")
        facade = self.create_package(
            "crates/nuxie",
            "nuxie",
            """
            [features]
            renderer = ["dep:nuxie-renderer"]

            [dependencies]
            nuxie-renderer = { path = "../nuxie-renderer", optional = true }
            """,
        )
        (facade / "src/lib.rs").write_text(
            "#[cfg(test)]\nmod tests { pub struct SyntheticFixture; }\n"
        )

    def write_manifest(self, body: str) -> None:
        (self.package / "Cargo.toml").write_text(
            textwrap.dedent(
                f"""\
                [package]
                name = "nuxie-runtime"
                version = "0.0.0"

                {body}
                """
            )
        )

    def run_check(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(TOOL), "--repo-root", str(self.root)],
            text=True,
            capture_output=True,
            check=False,
        )

    def test_allows_baseline_dependency(self) -> None:
        self.write_manifest(
            """
            [dependencies]
            nuxie-schema = { path = "../nuxie-schema" }
            """
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("pure-runtime boundary check passed", result.stdout)

    def test_rejects_portable_abi_facade_dependency_from_runtime(self) -> None:
        self.write_manifest(
            """
            [dependencies]
            nuxie = { path = "../nuxie" }
            """
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("imports product dependency 'nuxie'", result.stderr)

    def test_rejects_renamed_product_dependency(self) -> None:
        self.write_manifest(
            """
            [dependencies]
            authoring_adapter = { package = "nuxie-authoring", path = "../authoring" }
            """
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("package 'nuxie-authoring'", result.stderr)

    def test_rejects_target_specific_product_dependency(self) -> None:
        self.write_manifest(
            """
            [target.'cfg(target_os = "macos")'.dependencies]
            nux-container = { path = "../nux-container" }
            """
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("target.cfg(target_os = \"macos\").dependencies", result.stderr)

    def test_new_workspace_package_is_protected_by_default(self) -> None:
        self.create_package(
            "crates/new-baseline",
            "new-baseline",
            """
            [dependencies]
            nuxie-product = { path = "../product" }
            """,
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("crates/new-baseline/Cargo.toml", result.stderr)
        self.assertIn("nuxie-product", result.stderr)

    def test_implicit_path_workspace_member_is_protected(self) -> None:
        helper = self.root / "crates/helper"
        (helper / "src").mkdir(parents=True)
        (helper / "src/lib.rs").write_text("// implicit workspace member\n")
        (helper / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "helper"
                version = "0.0.0"

                [dependencies]
                nuxie-product = { path = "../product" }
                """
            )
        )
        self.write_manifest(
            """
            [dependencies]
            helper = { path = "../helper" }
            """
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("crates/helper/Cargo.toml", result.stderr)
        self.assertIn("nuxie-product", result.stderr)

    def test_workspace_inherited_path_is_resolved_from_workspace_root(self) -> None:
        helper = self.root / "crates/helper"
        (helper / "src").mkdir(parents=True)
        (helper / "src/lib.rs").write_text("// implicit workspace member\n")
        (helper / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "helper"
                version = "0.0.0"

                [dependencies]
                nuxie-product = "1"
                """
            )
        )
        self.write_manifest(
            """
            [dependencies]
            helper.workspace = true
            """
        )
        self.write_workspace(
            """
            [workspace.dependencies]
            helper = { path = "crates/helper" }
            """
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("crates/helper/Cargo.toml", result.stderr)
        self.assertIn("nuxie-product", result.stderr)

    def test_non_virtual_workspace_root_package_is_protected(self) -> None:
        (self.root / "src").mkdir()
        (self.root / "src/lib.rs").write_text("// root package\n")
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime"]

                [dependencies]
                nuxie-product = "1"
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("./Cargo.toml", result.stderr)
        self.assertIn("nuxie-product", result.stderr)

    def test_root_package_scan_does_not_claim_nested_product_sources(self) -> None:
        product = self.root / "crates/nuxie-authoring"
        (product / "src").mkdir(parents=True)
        (product / "src/lib.rs").write_text("pub struct ProjectDataSecret;\n")
        (product / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "nuxie-authoring"
                version = "0.0.0"
                """
            )
        )
        (self.root / "src").mkdir()
        (self.root / "src/lib.rs").write_text("// protected root package\n")
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime", "crates/nuxie-authoring"]
                """
            )
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_virtual_manifest_does_not_hide_protected_root_source(self) -> None:
        hidden = self.root / "src/hidden"
        hidden.mkdir(parents=True)
        (hidden / "Cargo.toml").write_text("[workspace]\nmembers = []\n")
        (hidden / "mod.rs").write_text("pub struct ProjectDataSecret;\n")
        (self.root / "src/lib.rs").write_text("mod hidden;\n")
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime"]
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("src/hidden/mod.rs", result.stderr)
        self.assertIn("project-data boundary debt", result.stderr)

    def test_unlisted_nested_package_does_not_hide_compiled_root_module(self) -> None:
        hidden = self.root / "src/hidden"
        hidden.mkdir(parents=True)
        (hidden / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "misleading-owner"
                version = "0.0.0"
                """
            )
        )
        (hidden / "mod.rs").write_text("pub struct ProjectDataSecret;\n")
        (self.root / "src/lib.rs").write_text("mod hidden;\n")
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime"]
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("src/hidden/mod.rs", result.stderr)
        self.assertIn("project-data boundary debt", result.stderr)

    def test_excluded_standalone_package_is_not_claimed_by_root(self) -> None:
        standalone = self.root / "vendor/standalone"
        (standalone / "src").mkdir(parents=True)
        (standalone / "src/lib.rs").write_text("pub struct ProjectDataSecret;\n")
        (standalone / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "standalone-product"
                version = "0.0.0"

                [workspace]
                """
            )
        )
        (self.root / "src").mkdir()
        (self.root / "src/lib.rs").write_text("// protected root package\n")
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime"]
                exclude = ["vendor/standalone"]
                """
            )
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_excluded_package_inside_source_tree_cannot_hide_module(self) -> None:
        hidden = self.root / "src/hidden"
        hidden.mkdir(parents=True)
        (hidden / "Cargo.toml").write_text(
            "[package]\nname = \"hidden\"\nversion = \"0.0.0\"\n"
        )
        (hidden / "mod.rs").write_text("pub struct ProjectDataSecret;\n")
        (self.root / "src/lib.rs").write_text("mod hidden;\n")
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime"]
                exclude = ["src/hidden"]
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("src/hidden/mod.rs", result.stderr)
        self.assertIn("project-data boundary debt", result.stderr)

    def test_root_package_cannot_path_import_nested_product_source(self) -> None:
        product = self.root / "crates/nuxie-authoring"
        (product / "src").mkdir(parents=True)
        (product / "src/secret.rs").write_text("pub struct ProjectDataSecret;\n")
        (product / "src/lib.rs").write_text("mod secret;\n")
        (product / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "nuxie-authoring"
                version = "0.0.0"
                """
            )
        )
        (self.root / "src").mkdir()
        (self.root / "src/lib.rs").write_text(
            '#[path = "../crates/nuxie-authoring/src/secret.rs"]\nmod secret;\n'
        )
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime", "crates/nuxie-authoring"]
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("path attribute crosses a package boundary", result.stderr)

    def test_root_package_cannot_cfg_attr_product_source(self) -> None:
        product = self.root / "crates/nuxie-authoring"
        product.mkdir(parents=True)
        (product / "secret.rs").write_text("pub struct ProjectDataSecret;\n")
        (product / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "nuxie-authoring"
                version = "0.0.0"
                """
            )
        )
        (self.root / "src").mkdir()
        (self.root / "src/lib.rs").write_text(
            '#[cfg_attr(target_os = "ios", '
            'path = "../crates/nuxie-authoring/secret.rs")]\n'
            "mod secret;\n"
        )
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime", "crates/nuxie-authoring"]
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("path attribute crosses a package boundary", result.stderr)

    def test_inline_module_path_attribute_fails_closed(self) -> None:
        product = self.root / "src/inline/nuxie-authoring"
        product.mkdir(parents=True)
        (product / "secret.rs").write_text("pub struct ProjectDataSecret;\n")
        (product / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "nuxie-authoring"
                version = "0.0.0"

                [lib]
                path = "secret.rs"
                """
            )
        )
        (self.root / "src/lib.rs").write_text(
            "mod inline {\n"
            '    #[path = "nuxie-authoring/secret.rs"]\n'
            "    mod secret;\n"
            "}\n"
        )
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = [
                    "crates/nuxie-runtime",
                    "src/inline/nuxie-authoring",
                ]
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("inline/block context could not be verified", result.stderr)

    def test_root_package_cannot_include_nested_product_source(self) -> None:
        product = self.root / "crates/nuxie-authoring"
        product.mkdir(parents=True)
        (product / "secret.rs").write_text("pub struct ProjectDataSecret;\n")
        (product / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "nuxie-authoring"
                version = "0.0.0"
                """
            )
        )
        (self.root / "src").mkdir()
        (self.root / "src/lib.rs").write_text(
            'include!("../crates/nuxie-authoring/secret.rs");\n'
        )
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime", "crates/nuxie-authoring"]
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("include! crosses a package boundary", result.stderr)

    def test_package_cannot_include_bytes_from_another_package(self) -> None:
        product = self.create_package(
            "crates/nuxie-product", "nuxie-product", ""
        )
        fixture = product / "tests/fixtures/product.bin"
        fixture.parent.mkdir(parents=True)
        fixture.write_bytes(b"product-owned fixture")
        (self.package / "src/lib.rs").write_text(
            'const FIXTURE: &[u8] = include_bytes!('
            '"../../nuxie-product/tests/fixtures/product.bin");\n'
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("include_bytes! crosses a package boundary", result.stderr)

    def test_package_cannot_hide_cross_package_bytes_in_concat(self) -> None:
        product = self.create_package(
            "crates/nuxie-product", "nuxie-product", ""
        )
        fixture = product / "tests/fixtures/product.bin"
        fixture.parent.mkdir(parents=True)
        fixture.write_bytes(b"product-owned fixture")
        (self.package / "src/lib.rs").write_text(
            'const FIXTURE: &[u8] = include_bytes!(concat!('
            'env!("CARGO_MANIFEST_DIR"), '
            '"/../nuxie-product/tests/fixtures/product.bin"));\n'
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("include_bytes! crosses a package boundary", result.stderr)

    def test_package_rejects_unverified_data_include_forms(self) -> None:
        (self.package / "src/lib.rs").write_text(
            'const FIXTURE: &[u8] = include_bytes!(concat!("../../../", "secret"));\n'
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("data include form could not be verified", result.stderr)

    def test_package_cannot_include_str_from_another_package(self) -> None:
        product = self.create_package(
            "crates/nuxie-product", "nuxie-product", ""
        )
        fixture = product / "tests/fixtures/product.txt"
        fixture.parent.mkdir(parents=True)
        fixture.write_text("product-owned fixture\n")
        (self.package / "src/lib.rs").write_text(
            'const FIXTURE: &str = include_str!('
            'r#"../../nuxie-product/tests/fixtures/product.txt"#);\n'
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("include_str! crosses a package boundary", result.stderr)

    def test_package_can_include_bytes_from_neutral_repository_fixtures(self) -> None:
        fixture = self.root / "fixtures/neutral.bin"
        fixture.parent.mkdir(parents=True)
        fixture.write_bytes(b"neutral fixture")
        (self.package / "src/lib.rs").write_text(
            'const FIXTURE: &[u8] = include_bytes!('
            '"../../../fixtures/neutral.bin");\n'
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_root_package_cannot_raw_include_nested_product_source(self) -> None:
        product = self.root / "crates/nuxie-authoring"
        product.mkdir(parents=True)
        (product / "secret.rs").write_text("pub struct ProjectDataSecret;\n")
        (product / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "nuxie-authoring"
                version = "0.0.0"
                """
            )
        )
        (self.root / "src").mkdir()
        (self.root / "src/lib.rs").write_text(
            'include!(r#"../crates/nuxie-authoring/secret.rs"#);\n'
        )
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime", "crates/nuxie-authoring"]
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("include! crosses a package boundary", result.stderr)

    def test_root_package_cannot_high_hash_raw_include_product_source(self) -> None:
        product = self.root / "crates/nuxie-authoring"
        product.mkdir(parents=True)
        (product / "secret.rs").write_text("pub struct ProjectDataSecret;\n")
        (product / "Cargo.toml").write_text(
            "[package]\nname = \"nuxie-authoring\"\nversion = \"0.0.0\"\n"
        )
        (self.root / "src").mkdir()
        hashes = "#" * 17
        (self.root / "src/lib.rs").write_text(
            f'include!(r{hashes}"../crates/nuxie-authoring/secret.rs"{hashes});\n'
        )
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime", "crates/nuxie-authoring"]
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("include! crosses a package boundary", result.stderr)

    def test_root_package_rejects_composed_include_path(self) -> None:
        (self.root / "src").mkdir()
        (self.root / "src/lib.rs").write_text(
            'include!(concat!("../crates/", "nuxie-authoring/secret.rs"));\n'
        )
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime"]
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("include! form could not be verified", result.stderr)

    def test_root_package_rejects_commented_include_form(self) -> None:
        (self.root / "src").mkdir()
        (self.root / "src/lib.rs").write_text(
            'include!/* boundary obscurer */("inside.rs");\n'
        )
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime"]
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("include! form could not be verified", result.stderr)

    def test_root_package_rejects_alternate_delimiter_include(self) -> None:
        (self.root / "src").mkdir()
        (self.root / "src/lib.rs").write_text('include! { "inside.rs" }\n')
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime"]
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("include! form could not be verified", result.stderr)

    def test_audited_runtime_codegen_include_is_allowed(self) -> None:
        (self.package / "src/objects.rs").write_text(
            'include!(concat!(env!("OUT_DIR"), "/runtime_objects.rs"));\n'
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_workspace_dependencies_are_not_root_package_edges(self) -> None:
        (self.root / "src").mkdir()
        (self.root / "src/lib.rs").write_text("// root package\n")
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "root-runtime"
                version = "0.0.0"

                [workspace]
                members = ["crates/nuxie-runtime"]

                [workspace.dependencies]
                nuxie-product = "1"
                """
            )
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_workspace_exclude_filters_member_glob_matches(self) -> None:
        excluded = self.root / "crates/excluded"
        (excluded / "src").mkdir(parents=True)
        (excluded / "src/lib.rs").write_text("// separate workspace\n")
        (excluded / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "excluded"
                version = "0.0.0"

                [workspace]

                [dependencies]
                nuxie-product = "1"
                """
            )
        )
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [workspace]
                members = ["crates/*"]
                exclude = ["crates/excluded"]
                """
            )
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_workspace_exclude_does_not_expand_wildcards(self) -> None:
        included = self.root / "crates/included"
        (included / "src").mkdir(parents=True)
        (included / "src/lib.rs").write_text("// Cargo still includes this member\n")
        (included / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "included"
                version = "0.0.0"

                [dependencies]
                nuxie-product = "1"
                """
            )
        )
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [workspace]
                members = ["crates/*"]
                exclude = ["crates/*"]
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("crates/included/Cargo.toml", result.stderr)

    def test_rejects_dependency_on_browser_adapter(self) -> None:
        self.write_manifest(
            """
            [dependencies]
            nuxie-browser-adapter = { path = "../browser-adapter" }
            """
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("nuxie-browser-adapter", result.stderr)

    def test_rejects_browser_adapter_workspace_ownership(self) -> None:
        self.create_package(
            "crates/nuxie-browser-adapter", "nuxie-browser-adapter", ""
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("belongs to its external product/platform owner", result.stderr)

    def test_rejects_product_dependency_from_portable_abi(self) -> None:
        self.create_package(
            "crates/nux-capi",
            "nux-capi",
            """
            [target.'cfg(any(target_os = "ios", target_os = "macos"))'.dependencies]
            nuxie-product = { path = "../product", optional = true }
            """,
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("crates/nux-capi/Cargo.toml", result.stderr)
        self.assertIn("nuxie-product", result.stderr)

    def test_rejects_apple_dependency_from_portable_abi(self) -> None:
        self.create_package(
            "crates/nux-capi",
            "nux-capi",
            """
            [dependencies]
            platform = { package = "nuxie-apple-adapter", path = "../apple" }
            """,
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("crates/nux-capi/Cargo.toml", result.stderr)
        self.assertIn("nuxie-apple-adapter", result.stderr)

    def test_rejects_apple_adapter_as_a_runtime_workspace_member(self) -> None:
        self.create_package(
            "crates/nuxie-apple-adapter",
            "nuxie-apple-adapter",
            "",
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("is owned by nuxie-ios", result.stderr)

    def test_rejects_product_vocabulary_in_portable_abi_header(self) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        include = package / "include"
        include.mkdir()
        (include / "nux_capi.h").write_text(
            "typedef struct NuxExperienceSession NuxExperienceSession;\n"
            "typedef struct NuxProductSession NuxProductSession;\n"
            "typedef struct NuxExperience NuxExperience;\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("portable ABI contains product/Apple vocabulary", result.stderr)
        self.assertIn("NuxExperienceSession", result.stderr)
        self.assertIn("NuxProductSession", result.stderr)
        self.assertIn("NuxExperience", result.stderr)

    def test_rejects_snake_case_product_vocabulary_in_portable_abi(self) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        include = package / "include"
        include.mkdir()
        (include / "nux_capi.h").write_text(
            "void nux_experience_session_create(void);\n"
            "void nux_experience_create(void);\n"
            "void nux_product_session_create(void);\n"
            "void nux_flow_session_advance(void);\n"
            "void nux_package_open(void);\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("nux_experience_session_create", result.stderr)
        self.assertIn("nux_experience_create", result.stderr)
        self.assertIn("nux_product_session_create", result.stderr)
        self.assertIn("nux_flow_session_advance", result.stderr)
        self.assertIn("nux_package_open", result.stderr)

    def test_rejects_prefixed_product_vocabulary_in_portable_abi(self) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        (package / "src/lib.rs").write_text(
            "fn create_product_session() {}\n"
            "struct PortableProductSession;\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("create_product_session", result.stderr)
        self.assertIn("PortableProductSession", result.stderr)

    def test_rejects_product_vocabulary_in_included_header_fragment(self) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        include = package / "include"
        include.mkdir()
        (include / "nux_capi.h").write_text('#include "portable_api.inc"\n')
        (include / "portable_api.inc").write_text(
            "typedef struct NuxProductSession NuxProductSession;\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("portable_api.inc", result.stderr)
        self.assertIn("NuxProductSession", result.stderr)

    def test_rejects_nux_artifact_vocabulary_in_portable_abi(self) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        (package / "src/lib.rs").write_text(
            "struct NuxArtifact;\n"
            "fn nux_artifact_open() {}\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("NuxArtifact", result.stderr)
        self.assertIn("nux_artifact_open", result.stderr)

    def test_allows_mathematical_product_vocabulary_in_portable_abi(self) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        (package / "src/lib.rs").write_text(
            "// Return the product of the values.\n"
            "fn dot_product() -> usize { [1, 2].iter().product() }\n"
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_scans_nested_directory_named_target_in_portable_abi(self) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        include = package / "include/target"
        include.mkdir(parents=True)
        (include / "nux_capi.h").write_text(
            "typedef struct NuxExperienceSession NuxExperienceSession;\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("include/target/nux_capi.h", result.stderr)
        self.assertIn("NuxExperienceSession", result.stderr)

    def test_rejects_apple_vocabulary_in_portable_abi_comment(self) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        (package / "src/lib.rs").write_text(
            "// The Apple renderer consumes this callback.\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("portable ABI contains product/Apple vocabulary", result.stderr)
        self.assertIn("'Apple'", result.stderr)

    def test_allows_exact_portable_abi_facade_edge_without_debt_report(self) -> None:
        self.create_portable_abi_facade()
        self.create_package(
            "crates/nux-capi",
            "nux-capi",
            """
            [dependencies]
            nuxie = { path = "../nuxie", default-features = false }
            """,
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn("portable-c-abi-mixed-facade", result.stdout)
        self.assertNotIn("mixed-facade", result.stdout)

    def test_rejects_direct_product_dependency_from_nuxie(self) -> None:
        self.create_portable_abi_facade()
        self.create_package("crates/nuxie-product", "nuxie-product", "")
        (self.root / "crates/nuxie/Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "nuxie"
                version = "0.0.0"

                [features]
                renderer = ["dep:nuxie-renderer"]

                [dependencies]
                nuxie-renderer = { path = "../nuxie-renderer", optional = true }
                nuxie-product = { path = "../nuxie-product" }
                """
            )
        )
        self.create_package(
            "crates/nux-capi",
            "nux-capi",
            """
            [dependencies]
            nuxie = { path = "../nuxie", default-features = false }
            """,
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("crates/nuxie/Cargo.toml", result.stderr)
        self.assertIn("nuxie-product", result.stderr)

    def test_rejects_transitive_product_dependency_from_nuxie(self) -> None:
        self.create_package(
            "crates/nuxie",
            "nuxie",
            """
            [dependencies]
            helper = { path = "../helper" }
            """,
        )
        self.create_package(
            "crates/helper",
            "helper",
            """
            [dependencies]
            nuxie-product = { path = "../nuxie-product" }
            """,
        )
        self.create_package("crates/nuxie-product", "nuxie-product", "")

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("crates/helper/Cargo.toml", result.stderr)
        self.assertIn("nuxie-product", result.stderr)

    def test_rejects_product_source_from_nuxie(self) -> None:
        package = self.create_package("crates/nuxie", "nuxie", "")
        (package / "src/lib.rs").write_text(
            "use nuxie_product::flow_session::FlowSession;\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("crates/nuxie/src/lib.rs", result.stderr)
        self.assertIn("product/authoring module", result.stderr)

    def test_rejects_target_workspace_aliased_product_dependency_from_nuxie(
        self,
    ) -> None:
        self.create_package(
            "crates/nuxie",
            "nuxie",
            """
            [target.'cfg(target_os = "ios")'.dependencies]
            bridge.workspace = true
            """,
        )
        self.write_workspace(
            """
            [workspace.dependencies]
            bridge = { package = "nuxie-product", path = "crates/nuxie-product" }
            """
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("crates/nuxie/Cargo.toml", result.stderr)
        self.assertIn("package 'nuxie-product'", result.stderr)
        self.assertIn("target.cfg(target_os = \"ios\").dependencies", result.stderr)

    def test_allows_exact_nuxie_self_test_support_dependency(self) -> None:
        facade = self.create_package(
            "crates/nuxie",
            "nuxie",
            """
            [features]
            test-support = []

            [dev-dependencies]
            nuxie = { path = ".", default-features = false, features = ["test-support"] }
            """,
        )
        (facade / "src/lib.rs").write_text(
            "#[cfg(test)]\nmod tests { pub struct SyntheticFixture; }\n"
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_rejects_production_authoring_marker_in_nuxie_lib(self) -> None:
        facade = self.create_package("crates/nuxie", "nuxie", "")
        (facade / "src/lib.rs").write_text("pub struct AuthoringRecord;\n")

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("binary-authoring test-only boundary debt", result.stderr)
        self.assertIn("escaped its test-only cfg module", result.stderr)

    def test_rejects_authoring_marker_in_compound_cfg_test_module(self) -> None:
        facade = self.create_package("crates/nuxie", "nuxie", "")
        (facade / "src/lib.rs").write_text(
            '#[cfg(all(feature = "scripting", test))]\n'
            "#[allow(dead_code)]\n"
            "mod tests { pub struct AuthoringRecord; }\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("binary-authoring boundary debt spread", result.stderr)

    def test_rejects_authoring_marker_in_cfg_that_can_build_without_test(self) -> None:
        facade = self.create_package("crates/nuxie", "nuxie", "")
        (facade / "src/lib.rs").write_text(
            '#[cfg(any(test, feature = "scripting"))]\n'
            "mod tests { pub struct AuthoringRecord; }\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("escaped its test-only cfg module", result.stderr)

    def test_rejects_expansion_of_nuxie_self_test_support_dependency(self) -> None:
        self.create_package(
            "crates/nuxie",
            "nuxie",
            """
            [features]
            renderer = []
            test-support = []

            [dev-dependencies]
            nuxie = { path = ".", default-features = false, features = ["test-support", "renderer"] }
            """,
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("nuxie self edge", result.stderr)
        self.assertIn("only the test-support feature", result.stderr)

    def test_rejects_product_dependency_through_excluded_in_repo_helper(
        self,
    ) -> None:
        self.create_package(
            "crates/nuxie",
            "nuxie",
            """
            [dependencies]
            helper = { path = "../helper" }
            """,
        )
        helper = self.root / "crates/helper"
        (helper / "src").mkdir(parents=True)
        (helper / "src/lib.rs").write_text("// excluded helper\n")
        (helper / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "helper"
                version = "0.0.0"

                [dependencies]
                nuxie-product = { path = "../nuxie-product" }
                """
            )
        )
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [workspace]
                members = ["crates/nuxie-runtime", "crates/nuxie"]
                exclude = ["crates/helper"]
                resolver = "3"
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("crates/nuxie/Cargo.toml", result.stderr)
        self.assertIn("in-repo path dependency", result.stderr)
        self.assertIn("outside the protected workspace scan", result.stderr)

    def test_rejects_unapproved_vendor_helper_outside_workspace_scan(self) -> None:
        self.create_package(
            "crates/nuxie",
            "nuxie",
            """
            [dependencies]
            helper = { path = "../../vendor/helper" }
            """,
        )
        self.create_package("crates/nuxie-product", "nuxie-product", "")
        helper = self.root / "vendor/helper"
        (helper / "src").mkdir(parents=True)
        (helper / "src/lib.rs").write_text("// hidden first-party helper\n")
        (helper / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "helper"
                version = "0.0.0"

                [dependencies]
                nuxie-product = { path = "../../crates/nuxie-product" }
                """
            )
        )
        members = ",\n".join(f'    "{member}"' for member in self.members)
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                f"""
                [workspace]
                members = [
                {members}
                ]
                exclude = ["vendor/helper"]
                resolver = "3"
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("vendor/helper", result.stderr)
        self.assertIn("outside the protected workspace scan", result.stderr)

    def test_rejects_excluded_local_provider_hidden_behind_cargo_patch(self) -> None:
        self.create_package(
            "crates/nuxie",
            "nuxie",
            """
            [dependencies]
            helper = "1"
            """,
        )
        self.create_package("crates/nuxie-product", "nuxie-product", "")
        helper = self.root / "vendor/helper"
        (helper / "src").mkdir(parents=True)
        (helper / "src/lib.rs").write_text("// patched first-party helper\n")
        (helper / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "helper"
                version = "1.0.0"

                [dependencies]
                nuxie-product = { path = "../../crates/nuxie-product" }
                """
            )
        )
        members = ",\n".join(f'    "{member}"' for member in self.members)
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                f"""
                [workspace]
                members = [
                {members}
                ]
                exclude = ["vendor/helper"]
                resolver = "3"

                [patch.crates-io]
                helper = {{ path = "vendor/helper" }}
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("path patch 'helper'", result.stderr)
        self.assertIn("outside the protected workspace scan", result.stderr)

    def test_rejects_dependency_provider_overrides_in_cargo_config(self) -> None:
        config_directory = self.root / ".cargo"
        config_directory.mkdir()
        override_cases = {
            "patch": textwrap.dedent(
                """
                [patch.crates-io]
                helper = { path = "../vendor/helper" }
                """
            ),
            "paths": 'paths = ["../vendor/helper"]\n',
            "source": textwrap.dedent(
                """
                [source.crates-io]
                replace-with = "vendored-sources"

                [source.vendored-sources]
                directory = "../vendor"
                """
            ),
        }

        for config_name in ("config.toml", "config"):
            config_path = config_directory / config_name
            for override_name, config_body in override_cases.items():
                with self.subTest(config=config_name, override=override_name):
                    config_path.write_text(config_body)

                    result = self.run_check()

                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn(
                        f".cargo/{config_name}: dependency provider override "
                        f"[{override_name}] is not allowed",
                        result.stderr,
                    )
            config_path.unlink()

    def test_scans_nonexcluded_local_cargo_patch_provider(self) -> None:
        self.create_package(
            "crates/nuxie",
            "nuxie",
            """
            [dependencies]
            helper = "1"
            """,
        )
        self.create_package("crates/nuxie-product", "nuxie-product", "")
        helper = self.root / "helpers/helper"
        (helper / "src").mkdir(parents=True)
        (helper / "src/lib.rs").write_text("// patched helper\n")
        (helper / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "helper"
                version = "1.0.0"

                [dependencies]
                nuxie-product = { path = "../../crates/nuxie-product" }
                """
            )
        )
        self.write_workspace(
            """
            [patch.crates-io]
            helper = { path = "helpers/helper" }
            """
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("helpers/helper/Cargo.toml", result.stderr)
        self.assertIn("nuxie-product", result.stderr)

    def test_allows_the_exact_runtime_self_patch_for_external_product_types(self) -> None:
        self.create_package("crates/nuxie", "nuxie", "")
        self.create_package("crates/nuxie-scripting", "nuxie-scripting", "")
        self.write_workspace(
            f'''
            [patch."{BOUNDARY_TOOL.RUNTIME_REPOSITORY}"]
            nuxie = {{ path = "crates/nuxie" }}
            nuxie-scripting = {{ path = "crates/nuxie-scripting" }}
            '''
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_rejects_runtime_self_patch_shape_drift(self) -> None:
        self.create_package("crates/nuxie", "nuxie", "")
        self.write_workspace(
            f'''
            [patch."{BOUNDARY_TOOL.RUNTIME_REPOSITORY}"]
            nuxie = {{ path = "crates/nuxie" }}
            '''
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must contain exactly", result.stderr)

    def test_live_contract_rejects_returning_extracted_product_source(self) -> None:
        contract = self.root / "docs/product-crate-seams.md"
        contract.parent.mkdir(parents=True)
        contract.write_text("# Product crate seams\n")
        extracted = self.root / "crates/nux-container"
        extracted.mkdir()

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("extracted product source must remain owned", result.stderr)

    def test_live_contract_requires_an_exact_external_product_provider(self) -> None:
        contract = self.root / "docs/product-crate-seams.md"
        contract.parent.mkdir(parents=True)
        contract.write_text("# Product crate seams\n")
        self.create_package("crates/nuxie", "nuxie", "")
        self.create_package("crates/nuxie-scripting", "nuxie-scripting", "")
        self.create_package(
            "crates/nuxie-product",
            "nuxie-product",
            '''
            [dependencies]
            nuxie-product-scripting = { path = "../nuxie-product-scripting", optional = true }
            ''',
        )
        self.write_workspace(
            f'''
            [patch."{BOUNDARY_TOOL.RUNTIME_REPOSITORY}"]
            nuxie = {{ path = "crates/nuxie" }}
            nuxie-scripting = {{ path = "crates/nuxie-scripting" }}
            '''
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("optional exact-revision dependency", result.stderr)

    def test_rejects_deprecated_cargo_replace_override(self) -> None:
        self.create_package(
            "crates/nuxie",
            "nuxie",
            """
            [dependencies]
            helper = "=1.0.0"
            """,
        )
        helper = self.root / "vendor/helper"
        (helper / "src").mkdir(parents=True)
        (helper / "src/lib.rs").write_text("// replacement helper\n")
        (helper / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "helper"
                version = "1.0.0"
                """
            )
        )
        members = ",\n".join(f'    "{member}"' for member in self.members)
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                f"""
                [workspace]
                members = [
                {members}
                ]
                exclude = ["vendor/helper"]
                resolver = "3"

                [replace]
                "helper:1.0.0" = {{ path = "vendor/helper" }}
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("deprecated [replace] override 'helper:1.0.0'", result.stderr)

    def test_rejects_expansion_of_portable_abi_facade_edge(self) -> None:
        self.create_package(
            "crates/nux-capi",
            "nux-capi",
            """
            [dependencies]
            nuxie = { path = "../nuxie", default-features = false, features = ["scripting"] }
            """,
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("portable ABI facade edge", result.stderr)

    def test_rejects_duplicate_alias_of_portable_abi_facade(self) -> None:
        self.create_package(
            "crates/nux-capi",
            "nux-capi",
            """
            [dependencies]
            nuxie = { path = "../nuxie", default-features = false }
            duplicate = { package = "nuxie", path = "../nuxie", default-features = false }
            """,
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("portable ABI facade edge", result.stderr)
        self.assertIn("dependency key 'nuxie'", result.stderr)

    def test_rejects_product_feature_forwarding_over_portable_abi_facade(
        self,
    ) -> None:
        self.create_package(
            "crates/nux-capi",
            "nux-capi",
            """
            [features]
            product = ["nuxie/scripting"]

            [dependencies]
            nuxie = { path = "../nuxie", default-features = false }
            """,
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "forwards forbidden portable ABI facade feature 'scripting'",
            result.stderr,
        )

    def test_rejects_indirect_product_feature_forwarding(self) -> None:
        self.create_package(
            "crates/nux-capi",
            "nux-capi",
            """
            [features]
            product = ["helper"]
            helper = ["nuxie/scripting"]

            [dependencies]
            nuxie = { path = "../nuxie", default-features = false }
            """,
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("feature 'helper' forwards forbidden", result.stderr)

    def test_allows_renderer_feature_forwarding(self) -> None:
        self.create_portable_abi_facade()
        self.create_package(
            "crates/nux-capi",
            "nux-capi",
            """
            [features]
            renderer = ["nuxie/renderer"]

            [dependencies]
            nuxie = { path = "../nuxie", default-features = false }
            """,
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_rejects_workspace_inherited_aliased_product_dependency(self) -> None:
        self.write_manifest(
            """
            [dependencies]
            bridge.workspace = true
            """
        )
        self.write_workspace(
            """
            [workspace.dependencies]
            bridge = { package = "nuxie-product", path = "crates/product" }
            """
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("package 'nuxie-product'", result.stderr)

    def test_rejects_features_added_to_workspace_inherited_portable_abi_facade(
        self,
    ) -> None:
        self.create_package(
            "crates/nux-capi",
            "nux-capi",
            """
            [dependencies]
            nuxie = { workspace = true, features = ["scripting"] }
            """,
        )
        self.write_workspace(
            """
            [workspace.dependencies]
            nuxie = { path = "crates/nuxie", default-features = false }
            """
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("cannot enable dependency features", result.stderr)

    def test_rejects_nonlocal_portable_abi_facade(self) -> None:
        self.create_package(
            "crates/nux-capi",
            "nux-capi",
            """
            [dependencies]
            nuxie = { git = "https://example.invalid/nuxie", default-features = false }
            """,
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must resolve to local crates/nuxie", result.stderr)

    def test_rejects_excluded_local_portable_abi_facade(self) -> None:
        self.create_package(
            "crates/nux-capi",
            "nux-capi",
            """
            [dependencies]
            nuxie = { path = "../nuxie", default-features = false }
            """,
        )
        facade = self.root / "crates/nuxie"
        (facade / "src").mkdir(parents=True)
        (facade / "src/lib.rs").write_text("// excluded portable ABI facade\n")
        (facade / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [package]
                name = "nuxie"
                version = "0.0.0"
                """
            )
        )
        (self.root / "Cargo.toml").write_text(
            textwrap.dedent(
                """
                [workspace]
                members = ["crates/nuxie-runtime", "crates/nux-capi"]
                exclude = ["crates/nuxie"]
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("in-repo path dependency", result.stderr)
        self.assertIn("outside the protected workspace scan", result.stderr)

    def test_workspace_inheritance_cannot_disable_facade_default_features(self) -> None:
        self.create_package(
            "crates/nux-capi",
            "nux-capi",
            """
            [dependencies]
            nuxie = { workspace = true, default-features = false }
            """,
        )
        self.write_workspace(
            """
            [workspace.dependencies]
            nuxie = { path = "crates/nuxie" }
            """
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must disable default features", result.stderr)

    def test_rejects_explicit_product_module_path(self) -> None:
        self.write_manifest("")
        (self.package / "src/lib.rs").write_text(
            "fn leak() { nuxie::flow_session::FlowSessionConfig::default(); }\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("product/authoring module", result.stderr)

    def test_rejects_product_root_reexports(self) -> None:
        product = self.create_package("crates/nuxie-product", "nuxie-product", "")
        sources = [
            "pub mod flow_session {}\npub use crate::flow_session::*;\n",
            "pub mod flow_session { pub struct FlowSession; }\n"
            "pub use flow_session::FlowSession;\n",
            "pub mod flow_session { pub struct FlowSession; pub struct FlowOperation; }\n"
            "pub use flow_session::{FlowOperation, FlowSession};\n",
            "pub mod flow_session { pub struct FlowSession; }\n"
            "pub use self::flow_session::FlowSession as Session;\n",
            "mod compatibility { pub use crate::flow_session::*; }\n"
            "pub mod flow_session { pub struct FlowSession; }\n"
            "pub use compatibility::*;\n",
        ]
        for source in sources:
            with self.subTest(source=source):
                (product / "src/lib.rs").write_text(source)
                result = self.run_check()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    "crate-root compatibility exports cannot return", result.stderr
                )

    def test_allows_namespaced_product_reexport(self) -> None:
        product = self.create_package("crates/nuxie-product", "nuxie-product", "")
        (product / "src/lib.rs").write_text(
            "pub mod project_data { pub use nuxie_project_data::*; }\n"
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_allows_approved_portable_abi_facade_symbols_and_file_import(
        self,
    ) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        (package / "src/approved.rs").write_text(
            "use nuxie::{File, StateMachineInstance};\n"
            "fn import(bytes: &[u8], _: &StateMachineInstance) {\n"
            "    let _ = File::import(bytes);\n"
            "}\n"
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_rejects_root_scene_reexport_from_portable_abi(self) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        (package / "src/new_scene.rs").write_text("use nuxie::Scene;\n")

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "portable ABI facade symbol 'Scene' is not approved", result.stderr
        )

    def test_rejects_product_method_on_approved_portable_abi_facade_type(
        self,
    ) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        (package / "src/new_flow.rs").write_text(
            "fn leak(file: &nuxie::File) { file.prepare_flow_player(); }\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("portable ABI facade product method", result.stderr)

    def test_rejects_product_trust_constructor_on_approved_file_type(self) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        (package / "src/new_trust.rs").write_text(
            "fn leak(bytes: &[u8]) { let _ = nuxie::File::import_with_unsigned_scripts(bytes); }\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("approved baseline facade surface", result.stderr)

    def test_rejects_every_current_product_trust_constructor_name(self) -> None:
        methods = (
            "import_with_trusted_scripts",
            "import_with_trusted_scripts_and_limits",
            "import_with_script_capability",
            "import_with_unsigned_scripts",
        )

        for method in methods:
            with self.subTest(method=method):
                errors = BOUNDARY_TOOL.portable_abi_facade_source_errors(
                    "crates/nux-capi/src/trust.rs", f"Self::{method}(bytes);"
                )
                self.assertTrue(errors, method)

    def test_rejects_portable_abi_facade_crate_alias(self) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        (package / "src/new_alias.rs").write_text(
            "use nuxie as facade;\nfn leak(_: facade::Scene) {}\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("portable ABI facade use tree", result.stderr)

    def test_rejects_portable_abi_facade_wildcard_import(self) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        (package / "src/new_glob.rs").write_text(
            "use nuxie::*;\nfn leak(_: Scene) {}\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("portable ABI facade use tree", result.stderr)

    def test_rejects_other_portable_abi_facade_alias_forms(self) -> None:
        sources = (
            "extern crate nuxie as facade;",
            "use nuxie::File as FacadeFile;",
            "use nuxie::{File as FacadeFile};",
            "type FacadeFile = nuxie::File;",
            "use {nuxie as facade};",
            "use {nuxie::{self as facade}};",
            "use {nuxie::*};",
            "use nuxie::{*};",
            "use nuxie::File as r#Facade;",
        )

        for source in sources:
            with self.subTest(source=source):
                errors = BOUNDARY_TOOL.portable_abi_facade_source_errors(
                    "crates/nux-capi/src/alias.rs", source
                )
                self.assertTrue(errors, source)

    def test_rejects_self_alias_through_impl_for_file(self) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        (package / "src/new_impl.rs").write_text(
            """
            use nuxie::File;
            trait Local { fn f(bytes: &[u8]); }
            impl Local for File {
                fn f(bytes: &[u8]) {
                    let _ = Self::import_with_unsigned_scripts(bytes);
                }
            }
            """
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("impls targeting portable ABI facade symbols", result.stderr)

    def test_rejects_local_authoring_module(self) -> None:
        self.write_manifest("")
        (self.package / "src/lib.rs").write_text("mod authoring;\n")

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("product/authoring module", result.stderr)

    def test_rejects_product_module_from_build_script(self) -> None:
        self.write_manifest("")
        (self.package / "build.rs").write_text("mod authoring;\n")

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("build.rs", result.stderr)
        self.assertIn("product/authoring module", result.stderr)

    def test_rejects_product_module_from_custom_cargo_target_path(self) -> None:
        self.write_manifest(
            """
            [lib]
            path = "runtime.rs"
            """
        )
        (self.package / "runtime.rs").write_text("mod authoring;\n")

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("runtime.rs", result.stderr)
        self.assertIn("product/authoring module", result.stderr)

    def test_rejects_custom_target_sibling_module_debt(self) -> None:
        self.write_manifest(
            """
            [lib]
            path = "runtime.rs"
            """
        )
        (self.package / "runtime.rs").write_text("mod authoring_support;\n")
        (self.package / "authoring_support.rs").write_text(
            "pub struct ProjectDataSecret;\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("authoring_support.rs", result.stderr)
        self.assertIn("project-data boundary debt spread", result.stderr)

    def test_source_directory_named_target_is_scanned(self) -> None:
        self.write_manifest("")
        target_module = self.package / "src/target"
        target_module.mkdir()
        (target_module / "mod.rs").write_text("pub struct ProjectDataSecret;\n")

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("src/target/mod.rs", result.stderr)

    def test_package_metadata_dependencies_are_not_cargo_edges(self) -> None:
        self.write_manifest(
            """
            [package.metadata.tool.dependencies]
            nuxie-product = "documentation-only"
            """
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_all_declared_custom_cargo_target_paths_are_discovered(self) -> None:
        manifest = {
            "package": {"build": "cargo/custom-build.rs"},
            "lib": {"path": "cargo/custom-lib.rs"},
            "bin": [{"path": "cargo/custom-bin.rs"}],
            "example": [{"path": "cargo/custom-example.rs"}],
            "test": [{"path": "cargo/custom-test.rs"}],
            "bench": [{"path": "cargo/custom-bench.rs"}],
        }

        sources = {
            path.relative_to(self.package.resolve()).as_posix()
            for path in BOUNDARY_TOOL.package_rust_sources(self.package, manifest)
        }

        self.assertTrue(
            {
                "cargo/custom-build.rs",
                "cargo/custom-lib.rs",
                "cargo/custom-bin.rs",
                "cargo/custom-example.rs",
                "cargo/custom-test.rs",
                "cargo/custom-bench.rs",
            }.issubset(sources)
        )

    def test_rejects_project_data_debt_in_new_file(self) -> None:
        self.write_manifest("")
        (self.package / "src/new_bridge.rs").write_text(
            "use crate::ProjectDataConverterProgram;\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("project-data boundary debt spread", result.stderr)

    def test_project_data_debt_has_no_grandfathered_files(self) -> None:
        self.write_manifest("")
        (self.package / "src/lib.rs").write_text("mod project_data_converter;\n")

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("project-data boundary debt spread", result.stderr)

    def test_rejects_glob_imported_product_host_limit_in_new_file(self) -> None:
        package = self.create_package(
            "crates/nuxie-scripting", "nuxie-scripting", ""
        )
        (package / "src/new_limits.rs").write_text(
            "use crate::vm::resource_limits::ScriptResourceLimit::*;\n"
            "fn trip() { let _ = (HostValueBytes, Commands); }\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("product-host-commands boundary debt spread", result.stderr)

    def test_rejects_product_host_cycle_method_in_new_file(self) -> None:
        package = self.create_package(
            "crates/nuxie-scripting", "nuxie-scripting", ""
        )
        (package / "src/new_host_cycle.rs").write_text(
            "fn cycle(vm: &Vm) { vm.begin_host_cycle(); }\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("product-host-commands boundary debt spread", result.stderr)

    def test_rejects_product_host_command_drain_method_in_new_file(self) -> None:
        package = self.create_package(
            "crates/nuxie-scripting", "nuxie-scripting", ""
        )
        (package / "src/new_host_commands.rs").write_text(
            "fn drain(vm: &Vm) { vm.drain_host_commands(); }\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("product-host-commands boundary debt spread", result.stderr)

    def test_product_host_resource_limits_are_no_longer_an_exception(self) -> None:
        package = self.create_package(
            "crates/nuxie-scripting", "nuxie-scripting", ""
        )
        resource_limits = package / "src/vm/resource_limits.rs"
        resource_limits.parent.mkdir(parents=True)
        resource_limits.write_text("enum Limit { HostValueBytes, CommandContent }\n")

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("product-host-commands boundary debt spread", result.stderr)

    def test_deleted_internal_debt_exception_fails_in_repository(self) -> None:
        (self.root / ".git").write_text("gitdir: fixture\n")
        missing = "crates/example/src/debt.rs"

        errors = BOUNDARY_TOOL.missing_debt_exception_errors(
            self.root,
            {"example": set()},
            {"example": {missing}},
        )

        self.assertEqual(
            errors,
            [f"{missing}: missing example boundary debt exception file; remove the allowlist entry"],
        )

    def test_rejects_browser_presentation_debt_in_new_file(self) -> None:
        package = self.create_package(
            "crates/nuxie-renderer", "nuxie-renderer", ""
        )
        (package / "src/new_browser_host.rs").write_text(
            "pub fn create() -> BrowserFactory { todo!() }\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("browser-presentation boundary debt spread", result.stderr)

    def test_browser_presentation_has_no_debt_exceptions(self) -> None:
        self.assertEqual(
            BOUNDARY_TOOL.INTERNAL_DEBT_FILES["browser-presentation"], set()
        )

    def test_rejects_apple_measurement_debt_in_new_file(self) -> None:
        package = self.create_package("crates/nux-capi", "nux-capi", "")
        (package / "src/new_size_root.rs").write_text(
            "fn root(surface: AppleSurface) {}\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("apple-presentation boundary debt spread", result.stderr)

    def test_apple_policy_has_no_debt_exceptions(self) -> None:
        self.assertEqual(
            BOUNDARY_TOOL.INTERNAL_DEBT_FILES["apple-image-admission"], set()
        )
        self.assertEqual(
            BOUNDARY_TOOL.INTERNAL_DEBT_FILES["apple-presentation"], set()
        )

    def test_rejects_binary_authoring_debt_in_new_file(self) -> None:
        package = self.create_package(
            "crates/nuxie-binary", "nuxie-binary", ""
        )
        (package / "src/new_builder.rs").write_text(
            "fn build(records: Vec<AuthoringRecord>) {}\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("binary-authoring boundary debt spread", result.stderr)

    def test_binary_authoring_is_reduced_to_the_zero_shipping_compatibility_seam(self) -> None:
        self.assertEqual(
            BOUNDARY_TOOL.INTERNAL_DEBT_FILES["binary-authoring"],
            {"crates/nuxie-binary/src/legacy_test_support.rs"},
        )

    def test_binary_authoring_compatibility_requires_the_test_support_gate(self) -> None:
        package = self.create_package(
            "crates/nuxie-binary",
            "nuxie-binary",
            """
            [features]
            default = []
            test-support = []
            """,
        )
        (package / "src/lib.rs").write_text("mod legacy_test_support;\n")
        (package / "src/legacy_test_support.rs").write_text(
            "pub struct AuthoringRecord;\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must remain gated", result.stderr)

    def test_commented_and_string_fake_gates_do_not_hide_an_unguarded_module(self) -> None:
        package = self.create_package(
            "crates/nuxie-binary",
            "nuxie-binary",
            """
            [features]
            default = []
            test-support = []
            """,
        )
        (package / "src/lib.rs").write_text(
            '// #[cfg(feature = "test-support")]\n'
            '// mod legacy_test_support;\n'
            'const FAKE: &str = r#"#[cfg(feature = "test-support")]\n'
            'mod legacy_test_support;"#;\n'
            "mod legacy_test_support;\n"
        )
        (package / "src/legacy_test_support.rs").write_text(
            "pub struct AuthoringRecord;\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must remain gated", result.stderr)

    def test_live_test_support_gate_allows_the_compatibility_module(self) -> None:
        package = self.create_package(
            "crates/nuxie-binary",
            "nuxie-binary",
            """
            [features]
            default = []
            test-support = []
            """,
        )
        (package / "src/lib.rs").write_text(
            '#[cfg(feature = "test-support")]\nmod legacy_test_support;\n'
        )
        (package / "src/legacy_test_support.rs").write_text(
            "pub struct AuthoringRecord;\n"
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_binary_authoring_compatibility_cannot_be_a_default_feature(self) -> None:
        package = self.create_package(
            "crates/nuxie-binary",
            "nuxie-binary",
            """
            [features]
            default = ["test-support"]
            test-support = []
            """,
        )
        (package / "src/lib.rs").write_text(
            '#[cfg(feature = "test-support")]\nmod legacy_test_support;\n'
        )
        (package / "src/legacy_test_support.rs").write_text(
            "pub struct AuthoringRecord;\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("outside the default shipping feature set", result.stderr)

    def test_binary_authoring_compatibility_cannot_hide_behind_default_alias(self) -> None:
        package = self.create_package(
            "crates/nuxie-binary",
            "nuxie-binary",
            """
            [features]
            default = ["compatibility"]
            compatibility = ["test-support"]
            test-support = []
            """,
        )
        (package / "src/lib.rs").write_text(
            '#[cfg(feature = "test-support")]\nmod legacy_test_support;\n'
        )
        (package / "src/legacy_test_support.rs").write_text(
            "pub struct AuthoringRecord;\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("outside the default shipping feature set", result.stderr)

    def test_rejects_editor_gpu_and_source_tooling_in_baseline_interfaces(self) -> None:
        package = self.create_package(
            "crates/nuxie-scripting", "nuxie-scripting", ""
        )
        (package / "src/editor_tools.rs").write_text(
            "pub struct GpuCanvasProgram;\n"
            "impl Vm { pub fn register_source_module(&self) {} }\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("editor-gpu-tooling boundary debt spread", result.stderr)

    def test_rejects_multiline_source_loading_api_in_baseline_interfaces(self) -> None:
        package = self.create_package(
            "crates/nuxie-scripting", "nuxie-scripting", ""
        )
        (package / "src/editor_tools.rs").write_text(
            "impl Vm {\n"
            "    pub fn load(\n"
            "        &self,\n"
            "        name: &str,\n"
            "        source: &str,\n"
            "    ) {}\n"
            "}\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("editor-gpu-tooling boundary debt spread", result.stderr)

    def test_bytecode_gpu_canvas_baseline_interface_remains_allowed(self) -> None:
        package = self.create_package(
            "crates/nuxie-scripting", "nuxie-scripting", ""
        )
        (package / "src/gpu_canvas.rs").write_text(
            "pub struct GpuCanvasBytecodeProgram;\n"
            "impl Vm { pub fn load_bytecode(&self) {} }\n"
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_comments_do_not_create_false_imports(self) -> None:
        self.write_manifest("")
        (self.package / "src/lib.rs").write_text(
            "pub struct NeutralBaseline;\n"
            "// do not import nuxie::scene or ProjectDataConverterProgram\n"
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_block_comments_and_raw_strings_do_not_create_false_debt(self) -> None:
        self.write_manifest("")
        (self.package / "src/notes.rs").write_text(
            "/* nested /* ProjectDataConverterProgram */ comment */\n"
            'const NOTE: &str = r#"AuthoringRecord and HostCommand"#;\n'
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_character_literals_do_not_confuse_string_stripping(self) -> None:
        source = "let quote = '\"';\nlet marker = \"host_commands\";\n"

        stripped = BOUNDARY_TOOL.strip_rust_non_code(source)

        self.assertIn("let quote", stripped)
        self.assertNotIn("host_commands", stripped)


class PureRuntimeBoundaryWiringTest(unittest.TestCase):
    def test_makefile_exposes_independent_test_check_and_gate_targets(self) -> None:
        makefile = (REPO_ROOT / "Makefile").read_text()

        self.assertIn("pure-runtime-boundary-test:", makefile)
        self.assertIn("pure-runtime-boundary-check:", makefile)
        self.assertIn("pure-runtime-boundary-gate:", makefile)
        self.assertIn('tools/report-all.sh "pure-runtime-boundary"', makefile)

    def test_ci_and_landing_run_the_pure_runtime_boundary_gate(self) -> None:
        workflow = (REPO_ROOT / ".github/workflows/ci.yml").read_text()
        landing = (REPO_ROOT / "tools/land.sh").read_text()

        self.assertIn("run: make pure-runtime-boundary-gate", workflow)
        self.assertIn("pure-runtime-boundary-gate", landing)


if __name__ == "__main__":
    unittest.main()
