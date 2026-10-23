import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from verify_inputs import verify_inputs


class VerifyInputsTest(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.source = self.root / "renderer/src/shaders/Makefile"
        self.source.parent.mkdir(parents=True)
        self.source.write_bytes(b"shader input\n")
        self.manifest = self.root / "source-manifest.json"
        self.manifest.write_text(json.dumps({"revision": "pinned", "files": {
            "renderer/src/shaders/Makefile": hashlib.sha256(b"shader input\n").hexdigest(),
        }}))

    def test_accepts_exact_bytes(self):
        self.assertEqual(verify_inputs(self.manifest, self.root, "pinned")["revision"], "pinned")

    def test_rejects_wrong_revision(self):
        with self.assertRaisesRegex(ValueError, "revision"):
            verify_inputs(self.manifest, self.root, "other")

    def test_rejects_changed_bytes(self):
        self.source.write_bytes(b"changed\n")
        with self.assertRaisesRegex(ValueError, "digest mismatch"):
            verify_inputs(self.manifest, self.root, "pinned")

    def test_rejects_extra_input(self):
        self.source.with_name("extra.glsl").write_text("extra")
        with self.assertRaisesRegex(ValueError, "file set"):
            verify_inputs(self.manifest, self.root, "pinned")

    def test_rejects_missing_input(self):
        self.source.unlink()
        with self.assertRaisesRegex(ValueError, "file set"):
            verify_inputs(self.manifest, self.root, "pinned")

    def test_rejects_escaping_path(self):
        self.manifest.write_text(json.dumps({"revision": "pinned", "files": {
            "renderer/src/shaders/../../escape": "0" * 64,
        }}))
        with self.assertRaisesRegex(ValueError, "invalid"):
            verify_inputs(self.manifest, self.root, "pinned")


if __name__ == "__main__":
    unittest.main()
