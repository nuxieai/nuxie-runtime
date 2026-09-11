"""Negative controls for the immutable-source gate, using disposable repos."""
import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("target_gate", Path(__file__).with_name("check-target-runtime.py"))
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


class ImmutableTargetTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.git("init", "-q")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "Test")
        for path in ["crates/runtime/src/lib.rs", "vendor/layout/src/lib.rs", "Cargo.toml", "Cargo.lock"]:
            self.write(path, "baseline\n")
        self.git("add", ".")
        self.git("commit", "-qm", "baseline")
        self.base = self.git("rev-parse", "HEAD").strip()

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, text=True)

    def write(self, path, text):
        target = self.root / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text)

    def result(self):
        return gate.audit(self.root, self.base)

    def test_clean_and_compiler_changes_are_allowed(self):
        self.assertEqual(self.result()["status"], "pass")
        self.write("tools/html-to-riv/src/lib.rs", "compiler\n")
        self.assertEqual(self.result()["status"], "pass")
        self.git("add", ".")
        self.git("commit", "-qm", "compiler")
        self.write("tools/html-to-riv/src/lib.rs", "updated\n")
        self.assertEqual(self.result()["status"], "pass")

    def test_runtime_vendor_and_resolution_changes_reject(self):
        for path in ["crates/runtime/src/lib.rs", "vendor/layout/src/lib.rs", "Cargo.toml", "Cargo.lock"]:
            with self.subTest(path=path):
                self.write(path, "mutated\n")
                self.assertIn(path, self.result()["changesOutsideCompiler"]["worktree"])
                self.write(path, "baseline\n")

    def test_staged_change_cannot_hide_behind_worktree_undo(self):
        path = "crates/runtime/src/lib.rs"
        self.write(path, "mutated\n")
        self.git("add", path)
        self.write(path, "baseline\n")
        result = self.result()
        self.assertEqual(result["status"], "fail")
        self.assertIn(path, result["changesOutsideCompiler"]["index"])
        self.assertEqual(result["changesOutsideCompiler"]["worktree"], [])

    def test_committed_runtime_change_rejects(self):
        self.write("crates/runtime/src/lib.rs", "mutation\n")
        self.git("add", ".")
        self.git("commit", "-qm", "runtime mutation")
        self.assertEqual(self.result()["status"], "fail")

    def test_new_protected_source_rejects(self):
        self.write("crates/runtime/src/css.rs", "new behavior\n")
        self.assertIn("crates/runtime/src/css.rs", self.result()["changesOutsideCompiler"]["untracked"])

    def test_deletion_rejects(self):
        (self.root / "crates/runtime/src/lib.rs").unlink()
        self.assertEqual(self.result()["status"], "fail")

    def test_module_prefix_cannot_hide_sibling(self):
        self.write("tools/html-to-riv-other/inject.rs", "new behavior\n")
        self.assertEqual(self.result()["status"], "fail")

    def test_mode_change_rejects(self):
        (self.root / "crates/runtime/src/lib.rs").chmod(0o755)
        self.assertEqual(self.result()["status"], "fail")


if __name__ == "__main__":
    unittest.main()
