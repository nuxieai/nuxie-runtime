"""Tests for the Rust golden-runner content-provenance guard.

The integration tests drive the real script against a synthetic cargo
workspace and encode the acceptance criterion directly: a source rewritten
*without* a newer mtime (the state a regenerated schema.rs racing a
concurrent cargo build leaves behind) must still produce a runner built from
the current content.
"""

import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import rust_runner_provenance as guard


class ChangedMembersTest(unittest.TestCase):
    def state(self):
        return {
            "schema": guard.DIGEST_SCHEMA,
            "rustc": "rustc 1.0.0",
            "workspace": "w" * 64,
            "members": {"a": "1" * 64, "b": "2" * 64},
        }

    def test_missing_recorded_state_selects_every_member(self):
        self.assertEqual(guard.changed_members(self.state(), None), ["a", "b"])

    def test_toolchain_change_selects_every_member(self):
        recorded = self.state()
        recorded["rustc"] = "rustc 0.9.9"
        self.assertEqual(guard.changed_members(self.state(), recorded), ["a", "b"])

    def test_workspace_manifest_change_selects_every_member(self):
        recorded = self.state()
        recorded["workspace"] = "x" * 64
        self.assertEqual(guard.changed_members(self.state(), recorded), ["a", "b"])

    def test_single_member_drift_selects_only_that_member(self):
        recorded = self.state()
        recorded["members"]["b"] = "3" * 64
        self.assertEqual(guard.changed_members(self.state(), recorded), ["b"])

    def test_new_member_counts_as_changed(self):
        recorded = self.state()
        del recorded["members"]["b"]
        self.assertEqual(guard.changed_members(self.state(), recorded), ["b"])

    def test_identical_state_selects_nothing(self):
        self.assertEqual(guard.changed_members(self.state(), self.state()), [])


class MemberDigestTest(unittest.TestCase):
    def test_digest_sees_content_not_mtime(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / "crate").mkdir()
            source = root / "crate" / "lib.rs"
            source.write_text("one")
            before = guard.member_digest(root, Path("crate"))
            stamp = source.stat()
            source.write_text("two")
            os.utime(source, (stamp.st_atime, stamp.st_mtime))
            after = guard.member_digest(root, Path("crate"))
            self.assertNotEqual(before, after)

    def test_digest_ignores_hidden_files_and_target(self):
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / "crate").mkdir()
            (root / "crate" / "lib.rs").write_text("one")
            before = guard.member_digest(root, Path("crate"))
            (root / "crate" / ".hidden").write_text("junk")
            (root / "crate" / "target").mkdir()
            (root / "crate" / "target" / "artifact").write_text("junk")
            self.assertEqual(guard.member_digest(root, Path("crate")), before)


class ReadOnlyRunnerPublicationTest(unittest.TestCase):
    """Exercise publication and verified restoration with Bazel-style modes."""

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.target = self.root / "target"
        (self.target / "debug").mkdir(parents=True)
        self.uplift = self.target / "debug" / guard.RUNNER_PACKAGE
        self.artifact = self.target / "debug" / "rust-golden-runner-ordinary"
        self.stamp_path = self.target / "golden-gate/ordinary-debug.json"
        self.payload = b"first executable"
        state = {
            "schema": guard.DIGEST_SCHEMA,
            "rustc": "test toolchain",
            "workspace": "fixed",
            "members": {},
        }
        for name, value in (
            ("workspace_members", {}),
            ("cargo_target_directory", self.target),
            ("current_digest_state", state),
        ):
            patcher = mock.patch.object(guard, name, return_value=value)
            patcher.start()
            self.addCleanup(patcher.stop)

        def build(command, cwd, capture=False):
            self.assertIn("build", command)
            self.uplift.write_bytes(self.payload)
            self.uplift.chmod(0o555)

        patcher = mock.patch.object(guard, "run", side_effect=build)
        self.build = patcher.start()
        self.addCleanup(patcher.stop)

    def ensure(self):
        guard.ensure_runner(self.root, "ordinary", "debug")

    def assert_published(self):
        for path in (self.uplift, self.artifact):
            self.assertEqual(path.read_bytes(), self.payload)
            self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o555)
        self.assertEqual(
            guard.load_json(self.stamp_path)["binary_sha256"],
            guard.sha256_path(self.artifact),
        )
        self.assertEqual(
            sorted(path.name for path in self.uplift.parent.iterdir()),
            sorted([self.uplift.name, self.artifact.name]),
        )

    def test_rebuild_replaces_read_only_artifact(self):
        self.ensure()
        self.assert_published()
        self.payload = b"replacement executable"
        self.stamp_path.unlink()  # Missing provenance requires a fresh build.
        self.ensure()
        self.assertEqual(self.build.call_count, 2)
        self.assert_published()

    def test_verified_copy_restores_read_only_uplift(self):
        self.ensure()
        stamp = self.stamp_path.read_bytes()
        self.uplift.unlink()
        self.uplift.write_bytes(b"another variant")
        self.uplift.chmod(0o555)
        self.ensure()
        self.assertEqual(self.build.call_count, 1)
        self.assertEqual(self.stamp_path.read_bytes(), stamp)
        self.assert_published()


@unittest.skipUnless(shutil.which("cargo"), "requires a cargo toolchain")
class EnsureRunnerIntegrationTest(unittest.TestCase):
    """End-to-end acceptance for the poisoned-cache scenario."""

    def setUp(self):
        environment = mock.patch.dict(os.environ)
        environment.start()
        self.addCleanup(environment.stop)
        # The synthetic fixture retains the known Cargo mtime failure as the
        # independent oracle. Production uses the direct Bazel frontend, whose
        # actual C ABI/native compilation is qualified separately.
        compiler = mock.patch.object(guard, "runner_build_command", return_value=["cargo"])
        compiler.start()
        self.addCleanup(compiler.stop)
        os.environ.pop("CARGO_TARGET_DIR", None)
        self.raw = tempfile.TemporaryDirectory()
        self.addCleanup(self.raw.cleanup)
        self.root = Path(self.raw.name)
        (self.root / "Cargo.toml").write_text(
            '[workspace]\nresolver = "2"\n'
            'members = ["probe-lib", "rust-golden-runner"]\n'
        )
        (self.root / "probe-lib" / "src").mkdir(parents=True)
        (self.root / "probe-lib" / "Cargo.toml").write_text(
            '[package]\nname = "probe-lib"\nversion = "0.1.0"\nedition = "2021"\n'
        )
        self.lib = self.root / "probe-lib" / "src" / "lib.rs"
        self.lib.write_text("pub fn value() -> u32 { 1 }\n")
        runner = self.root / "rust-golden-runner"
        (runner / "src").mkdir(parents=True)
        (runner / "Cargo.toml").write_text(
            '[package]\nname = "rust-golden-runner"\nversion = "0.1.0"\n'
            'edition = "2021"\n\n[features]\nscripting = []\n\n'
            '[dependencies]\nprobe-lib = { path = "../probe-lib" }\n'
        )
        (runner / "src" / "main.rs").write_text(
            'fn main() { println!("{}", probe_lib::value()); }\n'
        )
        subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=self.root,
            check=True,
            stdout=subprocess.DEVNULL,
        )

    def ensure(self):
        guard.ensure_runner(self.root, "ordinary", "debug")

    def runner_output(self, target_dir=None):
        target_dir = target_dir or self.root / "target"
        binary = target_dir / "debug" / "rust-golden-runner"
        return subprocess.run(
            [binary], check=True, stdout=subprocess.PIPE, text=True
        ).stdout.strip()

    def stamp(self, target_dir=None):
        target_dir = target_dir or self.root / "target"
        with open(target_dir / "golden-gate/ordinary-debug.json") as handle:
            return json.load(handle)

    def test_poisoned_cache_is_detected_and_rebuilt(self):
        self.ensure()
        self.assertEqual(self.runner_output(), "1")

        # Rewrite the dependency without advancing its mtime: the state a
        # regenerated source racing a concurrent cargo build leaves behind.
        stamp = self.lib.stat()
        self.lib.write_text("pub fn value() -> u32 { 2 }\n")
        os.utime(self.lib, (stamp.st_atime, stamp.st_mtime))

        # Plain cargo misses the change; that is the hole being guarded.
        subprocess.run(
            ["cargo", "build", "-p", "rust-golden-runner"],
            cwd=self.root,
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        self.assertEqual(self.runner_output(), "1")

        self.ensure()
        self.assertEqual(self.runner_output(), "2")

    def test_verified_runner_is_reused_without_rebuilding(self):
        self.ensure()
        first = self.stamp()
        artifact = self.root / "target" / "debug" / "rust-golden-runner-ordinary"
        before = artifact.stat().st_mtime_ns
        self.ensure()
        self.assertEqual(self.stamp(), first)
        self.assertEqual(artifact.stat().st_mtime_ns, before)

    def test_bazel_configuration_change_invalidates_the_runner_stamp(self):
        self.ensure()
        before = self.stamp()["digest_state"]
        (self.root / "MODULE.bazel").write_text('module(name = "probe", version = "0.1.0")\n')
        self.ensure()
        self.assertNotEqual(self.stamp()["digest_state"], before)
        self.assertEqual(self.runner_output(), "1")

    def test_clobbered_uplift_is_restored_from_verified_copy(self):
        self.ensure()
        uplift = self.root / "target" / "debug" / "rust-golden-runner"
        uplift.write_bytes(b"not the verified runner")
        self.ensure()
        self.assertEqual(self.runner_output(), "1")

    def test_environment_target_directories_do_not_touch_default_artifacts(self):
        self.ensure()
        default_stamp = self.stamp()
        default_runner = self.root / "target/debug/rust-golden-runner"
        default_mtime = default_runner.stat().st_mtime_ns
        for configured in ("target/isolated", str(self.root / "absolute-target")):
            with self.subTest(target_dir=configured):
                os.environ["CARGO_TARGET_DIR"] = configured
                target_dir = (self.root / configured).resolve()
                self.assertEqual(guard.cargo_target_directory(self.root), target_dir)
                self.ensure()
                self.assertEqual(self.runner_output(target_dir), "1")
                self.assertEqual(self.stamp(target_dir)["variant"], "ordinary")
                artifact = target_dir / "debug/rust-golden-runner-ordinary"
                before = artifact.stat().st_mtime_ns
                self.ensure()
                self.assertEqual(artifact.stat().st_mtime_ns, before)
                guard.ensure_sources(self.root)
                self.assertTrue((target_dir / "golden-gate/rust-sources.json").is_file())
                self.assertEqual(self.stamp(), default_stamp)
                self.assertEqual(default_runner.stat().st_mtime_ns, default_mtime)

    def test_cargo_configuration_target_directory_is_used(self):
        (self.root / ".cargo").mkdir()
        (self.root / ".cargo/config.toml").write_text(
            '[build]\ntarget-dir = "configured-target"\n'
        )
        target_dir = self.root / "configured-target"
        self.assertEqual(guard.cargo_target_directory(self.root), target_dir.resolve())
        guard.ensure_runner(self.root, "scripted", "debug")
        self.assertEqual(self.runner_output(target_dir), "1")
        self.assertTrue((target_dir / "debug/rust-golden-runner-scripted").is_file())
        self.assertTrue((target_dir / "golden-gate/scripted-debug.json").is_file())
        guard.ensure_sources(self.root)
        self.assertTrue((target_dir / "golden-gate/rust-sources.json").is_file())
        self.assertFalse((self.root / "target/golden-gate").exists())


if __name__ == "__main__":
    unittest.main()
