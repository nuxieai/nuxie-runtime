import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from cargo_graph import collect_packages
import runtime


class RuntimeBuildTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.root_patch = patch.object(runtime, "ROOT", self.root)
        self.root_patch.start()
        self.addCleanup(self.root_patch.stop)

    def package(self, name, extra="", library=True):
        directory = self.root / "crates" / name
        (directory / "src").mkdir(parents=True)
        (directory / "src" / ("lib.rs" if library else "main.rs")).write_text("fn main() {}\n")
        manifest = directory / "Cargo.toml"
        manifest.write_text(f'[package]\nname = "{name}"\nversion = "0.1.0"\nedition = "2024"\n' + extra)
        return manifest

    def test_combined_library_and_integration_test_keep_both_gates(self):
        manifest = self.package("core")
        (manifest.parent / "tests").mkdir()
        (manifest.parent / "tests/stroke.rs").write_text("#[test] fn strokes() {}\n")
        labels = runtime.materialize(runtime.command_options(["test", "-p", "core", "--lib", "--test", "stroke"]), collect_packages([manifest]))
        self.assertEqual({label.split(":")[1] for label in labels}, {"core__unit_test", "core__test_stroke"})

    def test_workspace_exclude_and_all_targets_select_binary_and_example(self):
        core = self.package("core")
        (core.parent / "examples").mkdir()
        (core.parent / "examples/recovery.rs").write_text("fn main() {}\n")
        (core.parent / "src/bin").mkdir()
        (core.parent / "src/bin/probe.rs").write_text("fn main() {}\n")
        other = self.package("other")
        options = runtime.command_options(["check", "--workspace", "--exclude", "other", "--all-targets"])
        labels = runtime.materialize(options, collect_packages([core, other]))
        self.assertEqual({label.split(":")[1] for label in labels}, {"core__test", "core__unit_test", "probe__test", "recovery__test"})

    def test_optional_features_are_separate_configuration_targets(self):
        core = self.package("core", '[features]\noptional = ["dep:dep"]\n[dependencies]\ndep = { path = "../dep", optional = true }\n')
        dep = self.package("dep")
        packages = collect_packages([core, dep])
        plain = runtime.materialize(runtime.command_options(["check", "-p", "core"]), packages)
        optional = runtime.materialize(runtime.command_options(["check", "-p", "core", "--features", "optional"]), packages)
        self.assertNotEqual(plain, optional)
        self.assertNotIn(':dep"', (self.root / plain[0][2:].replace(":core", "/BUILD.bazel")).read_text())
        self.assertIn(':dep"', (self.root / optional[0][2:].replace(":core", "/BUILD.bazel")).read_text())

    def test_bazel_uses_absolute_cache_location_and_preserves_spaces(self):
        executable = self.root / "bazel with spaces"
        executable.write_text("#!/bin/sh\nexit 0\n")
        executable.chmod(0o755)
        with patch.dict(os.environ, {"NUXIE_BAZEL_BIN": str(executable), "NUXIE_BAZEL_OUTPUT_USER_ROOT": str(self.root / "cache with spaces"), "CI": "1"}):
            self.assertEqual(runtime.bazel_command(), [str(executable), "--nosystem_rc", "--nohome_rc", "--output_user_root=" + str(self.root / "cache with spaces"), "--batch"])
        with patch.dict(os.environ, {"NUXIE_BAZEL_BIN": str(executable), "NUXIE_BAZEL_OUTPUT_USER_ROOT": "relative"}):
            with self.assertRaisesRegex(ValueError, "must be absolute"):
                runtime.bazel_command()

    def test_failed_build_keeps_previously_published_artifact(self):
        core = self.package("core", library=False)
        destination = self.root / "target/debug/core"
        destination.parent.mkdir(parents=True)
        destination.write_text("previous good binary")
        executable = self.root / "bazel"
        log = self.root / "calls.jsonl"
        executable.write_text("#!/usr/bin/env python3\nimport json, sys\nfrom pathlib import Path\nPath(" + repr(str(log)) + ").open('a').write(json.dumps(sys.argv[1:]) + '\\n')\nraise SystemExit(7)\n")
        executable.chmod(0o755)
        with patch.dict(os.environ, {"NUXIE_BAZEL_BIN": str(executable)}, clear=True), patch.object(runtime, "packages_from_workspace", return_value=collect_packages([core])), patch("sys.argv", ["runtime.py", "build", "-p", "core"]):
            self.assertEqual(runtime.main(), 2)
        self.assertEqual(destination.read_text(), "previous good binary")
        calls = [json.loads(line) for line in log.read_text().splitlines()]
        self.assertEqual(len(calls), 1)
        self.assertIn("build", calls[0])

    def test_publication_replaces_read_only_artifacts_and_reuses_identical_bytes(self):
        source = self.root / "bazel-product"
        destination = self.root / "published-product"
        source.write_bytes(b"new product")
        source.chmod(0o555)
        destination.write_bytes(b"old product")
        destination.chmod(0o444)
        runtime.publish_artifact(source, destination)
        self.assertEqual(destination.read_bytes(), b"new product")
        self.assertTrue(os.access(destination, os.X_OK))
        before = destination.stat().st_mtime_ns
        runtime.publish_artifact(source, destination)
        self.assertEqual(destination.stat().st_mtime_ns, before)


if __name__ == "__main__":
    unittest.main()
