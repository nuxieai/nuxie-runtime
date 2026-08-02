#!/usr/bin/env python3

import pathlib
import subprocess
import sys
import tempfile
import textwrap
import unittest


TOOL = pathlib.Path(__file__).with_name("rust_attribution.py")
ADDITIONS_HEADER = textwrap.dedent(
    """\
    schema = "nuxie-rust-additions/v1"
    schema_version = 1
    category_values = ["scene-api", "flowsession-abi", "retained-render", "codegen", "test-infra"]
    """
)


class RustAttributionCliTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        source = self.root / "crates/nuxie-runtime/src/lib.rs"
        source.parent.mkdir(parents=True)
        source.write_text("// runtime\n")
        self.manifest = self.root / "file-correspondence-manifest.toml"
        self.additions = self.root / "rust-additions.toml"

    def run_check(self) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable,
                str(TOOL),
                "--repo-root",
                str(self.root),
                "--manifest",
                str(self.manifest),
                "--additions",
                str(self.additions),
            ],
            text=True,
            capture_output=True,
            check=False,
        )

    def test_unknown_rust_source_fails(self) -> None:
        self.manifest.write_text("schema_version = 1\n")
        self.additions.write_text(ADDITIONS_HEADER)

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "unclassified Rust files: crates/nuxie-runtime/src/lib.rs",
            result.stderr,
        )

    def test_manifest_rust_module_list_classifies_source(self) -> None:
        self.manifest.write_text(
            textwrap.dedent(
                """
                schema_version = 1

                [[file]]
                upstream = "src/runtime.cpp"
                rust_module = "crates/elsewhere/src/owner.rs; crates/nuxie-runtime/src/lib.rs"
                """
            ).lstrip()
        )
        self.additions.write_text(ADDITIONS_HEADER)

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("every in-scope Rust source is classified", result.stdout)

    def test_test_only_rust_source_is_out_of_scope(self) -> None:
        test_source = self.root / "crates/nuxie-runtime/src/runtime_tests.rs"
        test_source.write_text("#[test]\nfn exercises_runtime() {}\n")
        self.manifest.write_text(
            textwrap.dedent(
                """
                schema_version = 1

                [[file]]
                rust_module = "crates/nuxie-runtime/src/lib.rs"
                """
            ).lstrip()
        )
        self.additions.write_text(ADDITIONS_HEADER)

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_tracked_addition_classifies_source(self) -> None:
        self.manifest.write_text("schema_version = 1\n")
        self.additions.write_text(
            ADDITIONS_HEADER
            + textwrap.dedent(
                """
                [[addition]]
                path = "crates/nuxie-runtime/src/lib.rs"
                category = "codegen"
                """
            ).lstrip()
        )

        result = self.run_check()

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_tracked_addition_rejects_unknown_category(self) -> None:
        self.manifest.write_text("schema_version = 1\n")
        self.additions.write_text(
            ADDITIONS_HEADER
            + textwrap.dedent(
                """
                [[addition]]
                path = "crates/nuxie-runtime/src/lib.rs"
                category = "misc"
                """
            ).lstrip()
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "invalid category for crates/nuxie-runtime/src/lib.rs: misc",
            result.stderr,
        )

    def test_tracked_addition_rejects_missing_source(self) -> None:
        self.manifest.write_text(
            textwrap.dedent(
                """
                schema_version = 1

                [[file]]
                rust_module = "crates/nuxie-runtime/src/lib.rs"
                """
            ).lstrip()
        )
        self.additions.write_text(
            ADDITIONS_HEADER
            + textwrap.dedent(
                """
                [[addition]]
                path = "crates/nuxie/src/missing.rs"
                category = "scene-api"
                """
            ).lstrip()
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "classified Rust source does not exist: crates/nuxie/src/missing.rs",
            result.stderr,
        )

    def test_tracked_addition_rejects_duplicate_path(self) -> None:
        self.manifest.write_text("schema_version = 1\n")
        self.additions.write_text(
            ADDITIONS_HEADER
            + textwrap.dedent(
                """
                [[addition]]
                path = "crates/nuxie-runtime/src/lib.rs"
                category = "codegen"

                [[addition]]
                path = "crates/nuxie-runtime/src/lib.rs"
                category = "test-infra"
                """
            ).lstrip()
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "duplicate classified Rust paths: crates/nuxie-runtime/src/lib.rs",
            result.stderr,
        )

    def test_tracked_addition_rejects_manifest_overlap(self) -> None:
        self.manifest.write_text(
            textwrap.dedent(
                """
                schema_version = 1

                [[file]]
                rust_module = "crates/nuxie-runtime/src/lib.rs"
                """
            ).lstrip()
        )
        self.additions.write_text(
            ADDITIONS_HEADER
            + textwrap.dedent(
                """
                [[addition]]
                path = "crates/nuxie-runtime/src/lib.rs"
                category = "codegen"
                """
            ).lstrip()
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "Rust paths are both attributed and classified as additions: crates/nuxie-runtime/src/lib.rs",
            result.stderr,
        )

    def test_rejects_additions_schema_vocabulary_drift(self) -> None:
        self.manifest.write_text("schema_version = 1\n")
        self.additions.write_text(
            ADDITIONS_HEADER.replace('"test-infra"', '"misc"')
            + textwrap.dedent(
                """
                [[addition]]
                path = "crates/nuxie-runtime/src/lib.rs"
                category = "codegen"
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invalid rust-additions category_values", result.stderr)


if __name__ == "__main__":
    unittest.main()

