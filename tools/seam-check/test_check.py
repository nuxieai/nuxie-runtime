#!/usr/bin/env python3
"""Focused controls for the Nuxie-only seam guard; no C++ counterpart."""

from __future__ import annotations

import pathlib
import subprocess
import sys
import tempfile
import textwrap
import unittest


TOOL = pathlib.Path(__file__).with_name("check.py")


class SeamCheckCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.package = self.root / "crates/nuxie-runtime"
        (self.package / "src").mkdir(parents=True)
        (self.package / "src/lib.rs").write_text("// parity baseline\n")

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
        self.assertIn("seam check passed", result.stdout)

    def test_rejects_mixed_facade_dependency(self) -> None:
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

    def test_rejects_explicit_product_module_path(self) -> None:
        self.write_manifest("")
        (self.package / "src/lib.rs").write_text(
            "fn leak() { nuxie::flow_session::FlowSessionConfig::default(); }\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("product/authoring module", result.stderr)

    def test_rejects_local_authoring_module(self) -> None:
        self.write_manifest("")
        (self.package / "src/lib.rs").write_text("mod authoring;\n")

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("product/authoring module", result.stderr)

    def test_rejects_project_data_debt_in_new_file(self) -> None:
        self.write_manifest("")
        (self.package / "src/new_bridge.rs").write_text(
            "use crate::ProjectDataConverterProgram;\n"
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("project-data seam debt spread", result.stderr)

    def test_current_grandfathered_file_is_a_ratchet_exception(self) -> None:
        self.write_manifest("")
        (self.package / "src/lib.rs").write_text("mod project_data_converter;\n")

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("project-data=1 grandfathered file(s)", result.stdout)

    def test_comments_do_not_create_false_imports(self) -> None:
        self.write_manifest("")
        (self.package / "src/lib.rs").write_text(
            "// do not import nuxie::scene or ProjectDataConverterProgram\n"
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()

