import json
import os
from pathlib import Path, PureWindowsPath
from dataclasses import replace
import tempfile
import unittest
from types import SimpleNamespace
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

    def test_windows_paths_emit_valid_bazel_labels_and_source_roots(self):
        # Keep real temporary files, but expose Windows spelling from every
        # relative path, including paths discovered by glob in the emitter.
        class WindowsSpellingPath(type(Path())):
            def relative_to(self, *args, **kwargs):
                return PureWindowsPath(super().relative_to(*args, **kwargs))

        manifest = self.package("core")
        (manifest.parent / "src/bin").mkdir()
        (manifest.parent / "src/bin/probe.rs").write_text("fn main() {}\n")
        packages = {
            name: replace(package, directory=WindowsSpellingPath(package.directory))
            for name, package in collect_packages([manifest]).items()
        }
        with patch.object(runtime, "ROOT", WindowsSpellingPath(self.root)):
            labels = runtime.materialize(runtime.command_options(["build", "-p", "core"]), packages)
        self.assertEqual({label.split(":")[1] for label in labels}, {"core", "probe"})
        self.assertTrue(all("\\" not in label for label in labels))
        build_path = self.root / labels[0][2:].split(":")[0] / "BUILD.bazel"
        build = build_path.read_text()
        self.assertIn('crate_root = "//crates/core:src/lib.rs"', build)
        self.assertIn('crate_root = "//crates/core:src/bin/probe.rs"', build)

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

    def test_native_tools_do_not_widen_shipping_registry(self):
        audio = self.package("nuxie-audio", '[features]\naudio-device = ["dep:cpal"]\n[dependencies]\ncpal = { version = "0.16", optional = true }\n')
        capi = self.package("nux-capi", '[dependencies]\nnuxie-audio = { path = "../nuxie-audio" }\n')
        replay = self.package("renderer-replay", '[features]\nnative-metal = ["dep:objc2"]\nnative-ore-metal = ["native-metal"]\n[dependencies]\nobjc2 = { version = "0.6", optional = true }\n')
        packages = collect_packages([audio, capi, replay])
        for package, features, expected_registry in (
            ("nux-capi", [], "@runtime_crates"),
            ("nuxie-audio", [], "@runtime_crates"),
            ("nuxie-audio", ["audio-device"], "@runtime_native_tools_crates"),
            ("renderer-replay", ["native-metal"], "@runtime_native_tools_crates"),
            ("renderer-replay", ["native-ore-metal"], "@runtime_native_tools_crates"),
        ):
            options = runtime.command_options(["check", "-p", package] + (["--features", ",".join(features)] if features else []))
            labels = runtime.materialize(options, packages)
            build = (self.root / labels[0][2:].replace(":" + package, "/BUILD.bazel")).read_text()
            registry_load = next(line for line in build.splitlines() if '"all_crate_deps"' in line)
            self.assertEqual(registry_load.split('"')[1], expected_registry + "//:defs.bzl")
            if package == "nux-capi":
                package_directory = self.root / labels[0][2:].split(":")[0]
                audio_build = (package_directory.parent / "nuxie-audio/BUILD.bazel").read_text()
                self.assertNotIn('crate_deps(["cpal"]', audio_build)

    def test_bazel_uses_absolute_cache_location_and_preserves_spaces(self):
        executable = self.root / "bazel with spaces"
        executable.write_text("#!/bin/sh\nexit 0\n")
        executable.chmod(0o755)
        with patch.dict(os.environ, {"NUXIE_BAZEL_BIN": str(executable), "NUXIE_BAZEL_OUTPUT_USER_ROOT": str(self.root / "cache with spaces"), "CI": "1"}):
            self.assertEqual(runtime.bazel_command(), [str(executable), "--nosystem_rc", "--nohome_rc", "--output_user_root=" + str(self.root / "cache with spaces"), "--batch"])
        with patch.dict(os.environ, {"NUXIE_BAZEL_BIN": str(executable), "NUXIE_BAZEL_OUTPUT_USER_ROOT": "relative"}):
            with self.assertRaisesRegex(ValueError, "must be absolute"):
                runtime.bazel_command()

    def test_windows_uses_native_bazelisk_binary_instead_of_npm_shim(self):
        package = self.root / "target/bazel-tools/node_modules/@bazel/bazelisk"
        package.mkdir(parents=True)
        native = package / "bazelisk-windows_amd64.exe"
        native.touch()
        shim = self.root / "target/bazel-tools/node_modules/.bin/bazelisk"
        shim.parent.mkdir(parents=True)
        shim.write_text("#!/bin/sh\n")
        with patch.dict(os.environ, {}, clear=True), patch.object(runtime.sys, "platform", "win32"), \
             patch.object(runtime.host_platform, "machine", return_value="AMD64"), \
             patch.object(runtime.shutil, "which", return_value=None):
            self.assertEqual(runtime.bazel_command()[0], str(native))

    def test_windows_binary_publication_keeps_exe_and_ignores_debug_sidecars(self):
        executable = self.root / "bazel-out/corpus-r.exe"
        executable.parent.mkdir()
        executable.write_bytes(b"compiled Windows binary")
        sidecar = executable.with_suffix(".pdb")
        sidecar.write_bytes(b"debug symbols")
        output = "\n".join(path.relative_to(self.root).as_posix() for path in (executable, sidecar))
        with patch.object(runtime.sys, "platform", "win32"), \
             patch.object(runtime, "packages_from_workspace", return_value={}), \
             patch.object(runtime, "materialize", return_value=["//runner:corpus-r"]), \
             patch.object(runtime, "bazel_command", return_value=["bazel"]), \
             patch.object(runtime.subprocess, "check_output", return_value=output), \
             patch.object(runtime.subprocess, "run", return_value=SimpleNamespace(returncode=0)) as run, \
             patch("sys.argv", ["runtime.py", "run", "-p", "pixel-compare", "--bin", "corpus-r", "--", "--manifest", "corpus-r.toml"]):
            self.assertEqual(runtime.main(), 0)
        published = self.root / "target/debug/corpus-r.exe"
        self.assertEqual(published.read_bytes(), executable.read_bytes())
        self.assertEqual(run.call_args.args[0], [str(published), "--manifest", "corpus-r.toml"])

    def test_shared_cache_override_preserves_default_output_isolation(self):
        executable = self.root / "bazel"
        executable.touch()
        with patch.dict(os.environ, {"NUXIE_BAZEL_BIN": str(executable),
                                     "NUXIE_BAZEL_CACHE_DIR": str(self.root / "shared cache")}, clear=True):
            command = runtime.bazel_command()
        self.assertEqual(command, [str(executable), "--nosystem_rc", "--nohome_rc",
                                   "--bazelrc=" + str(self.root / ".bazel-cache.local.bazelrc")])
        self.assertIn(str(self.root / "shared cache/actions"),
                      (self.root / ".bazel-cache.local.bazelrc").read_text())

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

    def test_coverage_build_preserves_instrumentation_and_isolated_products(self):
        manifest = self.package("rust-golden-runner", '[features]\nscripting = []\ncoverage-trace = []\n', library=False)
        executable = self.root / "bazel-out/rust-golden-runner"
        executable.parent.mkdir()
        executable.write_bytes(b"instrumented runner")
        isolated = self.root / "target/callcount/rust"
        with patch.dict(os.environ, {"RUSTFLAGS": "-Cinstrument-coverage", "CARGO_TARGET_DIR": str(isolated)}, clear=True), \
             patch.object(runtime, "packages_from_workspace", return_value=collect_packages([manifest])), \
             patch.object(runtime, "bazel_command", return_value=["bazel"]), \
             patch.object(runtime.subprocess, "check_output", return_value="bazel-out/rust-golden-runner"), \
             patch.object(runtime.subprocess, "run", return_value=SimpleNamespace(returncode=0)) as run, \
             patch("sys.argv", ["runtime.py", "build", "--release", "--quiet", "--manifest-path", str(self.root / "Cargo.toml"),
                                "-p", "rust-golden-runner", "--features", "scripting,coverage-trace"]):
            self.assertEqual(runtime.main(), 0)
        arguments = run.call_args.args[0]
        self.assertIn("--config=release", arguments)
        self.assertIn("--@rules_rust//rust/settings:extra_rustc_flag=-Cinstrument-coverage", arguments)
        label = next(argument for argument in arguments if argument.startswith("//build/bazel-config/"))
        build = (self.root / label[2:].split(":")[0] / "BUILD.bazel").read_text()
        features = json.loads(next(line.split(" = ", 1)[1].rstrip(",") for line in build.splitlines() if "crate_features = " in line))
        self.assertTrue({"coverage-trace", "scripting"}.issubset(features))
        self.assertEqual((isolated / "release/rust-golden-runner").read_bytes(), executable.read_bytes())
        self.assertFalse((self.root / "target/release/rust-golden-runner").exists())

    def test_frontend_scopes_provenance_without_changing_other_environment_inputs(self):
        with patch.dict(os.environ, {"NUX_RUNTIME_SOURCE_REVISION": "revision with spaces=one",
                                     "NUX_RUNTIME_RIVE_ORACLE": "fixture", "RIVE_RUNTIME_DIR": "rive"}, clear=True), \
             patch.object(runtime, "packages_from_workspace", return_value={}), \
             patch.object(runtime, "materialize", return_value=["//core:core"]), \
             patch.object(runtime, "bazel_command", return_value=["bazel"]), \
             patch.object(runtime.subprocess, "run", return_value=SimpleNamespace(returncode=0)) as run, \
             patch("sys.argv", ["runtime.py", "test", "-p", "core"]):
            self.assertEqual(runtime.main(), 0)
        arguments = run.call_args.args[0]
        self.assertIn("--define=NUX_RUNTIME_SOURCE_REVISION=revision with spaces=one", arguments)
        self.assertNotIn("--action_env=NUX_RUNTIME_SOURCE_REVISION", arguments)
        self.assertIn("--action_env=NUX_RUNTIME_RIVE_ORACLE", arguments)
        self.assertIn("--action_env=RIVE_RUNTIME_DIR", arguments)

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
