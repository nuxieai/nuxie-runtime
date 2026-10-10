"""Exercise fixture cache reuse offline without touching the real fixture tree."""

import hashlib
import os
from pathlib import Path
import re
import subprocess
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("fetch-test-assets.sh")
FIXTURE = b"offline pinned fixture\n"


class FetchTestAssetsTest(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / "tools").mkdir()
        self.script = self.root / "tools/fetch-test-assets.sh"
        # Use small synthetic pinned bytes for every manifest entry; exercise
        # the complete production fetch script with its normal fixture paths.
        digest = hashlib.sha256(FIXTURE).hexdigest()
        self.script.write_text(re.sub(r"[0-9a-f]{64}", digest, SCRIPT.read_text()))
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.runtime = self.root / "runtime"
        for relative in (
            "skia/dependencies/skia/resources/fonts/sbix.ttf",
            "tests/unit_tests/silvers/data_bind_blob_test.sriv",
        ):
            destination = self.runtime / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(FIXTURE)
        self.env = {
            **os.environ,
            "PATH": str(self.bin) + os.pathsep + os.environ["PATH"],
            "RIVE_RUNTIME_DIR": str(self.runtime),
        }
        self.allow_fetches()

    def stub(self, name, body):
        path = self.bin / name
        path.write_text("#!/usr/bin/env bash\nset -eu\n" + body)
        path.chmod(0o755)

    def allow_fetches(self, content=FIXTURE):
        # cat-file finds the recorded refs, while show returns pinned bytes.
        quoted = content.decode().replace("'", "'\"'\"'")
        self.stub("git", "if [[ $3 == cat-file ]]; then exit 0; fi\nprintf '%s' '" + quoted + "'\n")
        self.stub("curl", "while [[ $# -gt 0 ]]; do\n  if [[ $1 == --output ]]; then printf '%s' '" + quoted + "' > \"$2\"; exit 0; fi\n  shift\ndone\nexit 1\n")

    def run_script(self):
        return subprocess.run(
            ["bash", str(self.script)], env=self.env,
            capture_output=True, text=True, timeout=30,
        )

    def assert_ready(self):
        result = self.run_script()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_valid_cached_files_keep_mtimes_without_git_or_network(self):
        self.assert_ready()
        files = [path for directory in ("fixtures", "fuzz/seeds") for path in (self.root / directory).rglob("*") if path.is_file()]
        self.assertGreater(len(files), 100)
        for path in files:
            os.utime(path, ns=(1_000_000_000, 1_000_000_000))
        before = {path: path.stat().st_mtime_ns for path in files}
        self.stub("git", "echo 'cached fixture consulted git' >&2\nexit 1\n")
        self.stub("curl", "echo 'cached fixture consulted network' >&2\nexit 1\n")
        self.assert_ready()
        self.assertEqual({path: path.stat().st_mtime_ns for path in files}, before)

    def test_corrupt_fixture_and_fuzz_seed_are_repaired(self):
        self.assert_ready()
        paths = (
            self.root / "fixtures/animation/smi_test.riv",
            self.root / "fuzz/seeds/fuzz_import/smi_test.riv",
        )
        for path in paths:
            path.write_bytes(b"corrupt fixture\n")
        self.assert_ready()
        for path in paths:
            self.assertEqual(path.read_bytes(), FIXTURE)

    def test_fetched_bytes_still_require_the_pinned_checksum(self):
        self.allow_fetches(b"incorrect upstream bytes\n")
        result = self.run_script()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("fixture checksum mismatch:", result.stderr)


if __name__ == "__main__":
    unittest.main()
