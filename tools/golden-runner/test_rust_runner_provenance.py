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
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

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


@unittest.skipUnless(shutil.which("cargo"), "requires a cargo toolchain")
class EnsureRunnerIntegrationTest(unittest.TestCase):
    """End-to-end acceptance for the poisoned-cache scenario."""

    def setUp(self):
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

    def runner_output(self):
        binary = self.root / "target" / "debug" / "rust-golden-runner"
        return subprocess.run(
            [binary], check=True, stdout=subprocess.PIPE, text=True
        ).stdout.strip()

    def stamp(self):
        with open(self.root / "target/golden-gate/ordinary-debug.json") as handle:
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

    def test_clobbered_uplift_is_restored_from_verified_copy(self):
        self.ensure()
        uplift = self.root / "target" / "debug" / "rust-golden-runner"
        uplift.write_bytes(b"not the verified runner")
        self.ensure()
        self.assertEqual(self.runner_output(), "1")


if __name__ == "__main__":
    unittest.main()
