"""Cache override regressions; no compiler or external service is required."""

import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from bazel_cache import guarded_environment, startup_options


class CacheTest(unittest.TestCase):
    def test_v2_cache_configuration_uses_the_verified_installed_environment(self):
        with tempfile.TemporaryDirectory() as temporary, patch.dict(os.environ, {
                "NUXIE_CI_RUNNER_POLICY_VERSION": "2", "NUXIE_BAZEL_CACHE_DIR": "/untrusted/cache",
            }, clear=True), patch("bazel_cache.subprocess.check_output", return_value=(
                "export NUXIE_BAZEL_CACHE_DIR='" + temporary + "/installed cache'\n"
                "export NUXIE_BAZEL_JOBS=3\n"
            )) as command:
            startup_options(Path(temporary))
            self.assertIn(str(Path(temporary) / "installed cache/actions"),
                          (Path(temporary) / ".bazel-cache.local.bazelrc").read_text())
            self.assertEqual(os.environ["NUXIE_BAZEL_JOBS"], "3")
            self.assertEqual(command.call_args.args[0][:3], ["/bin/bash", "-eu", "-c"])
            self.assertIn("nuxie_runner_policy_verify", command.call_args.args[0][3])

    def test_invalid_installed_environment_cannot_change_process_settings(self):
        for output in ("export HOME=/other\n", "export NUXIE_BAZEL_JOBS=100; exit 0\n", "garbage\n"):
            with self.subTest(output=output), patch.dict(os.environ, {"NUXIE_CI_RUNNER_POLICY_VERSION": "2"}, clear=True), \
                    patch("bazel_cache.subprocess.check_output", return_value=output):
                with self.assertRaises(ValueError):
                    guarded_environment()
                self.assertEqual(dict(os.environ), {"NUXIE_CI_RUNNER_POLICY_VERSION": "2"})

    def test_default_keeps_bazel_output_base_and_repository_configuration(self):
        with tempfile.TemporaryDirectory() as directory, patch.dict(os.environ, {}, clear=True):
            self.assertEqual(startup_options(Path(directory)), [])
            self.assertEqual(list(Path(directory).iterdir()), [])

    def test_shared_cache_does_not_share_generated_configuration_or_outputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shared = root / "shared cache with 'quotes'"
            with patch.dict(os.environ, {"NUXIE_BAZEL_CACHE_DIR": str(shared)}, clear=True):
                for name in ("first", "second"):
                    checkout = root / name
                    checkout.mkdir()
                    self.assertEqual(startup_options(checkout), ["--bazelrc=" + str(checkout / ".bazel-cache.local.bazelrc")])
                first = root / "first/.bazel-cache.local.bazelrc"
                second = root / "second/.bazel-cache.local.bazelrc"
                self.assertEqual(first.read_bytes(), second.read_bytes())
                self.assertFalse(first.samefile(second))
                self.assertNotIn("output_base", first.read_text())
                self.assertNotIn("output_user_root", first.read_text())
                before = first.stat().st_mtime_ns
                startup_options(first.parent)
                self.assertEqual(first.stat().st_mtime_ns, before)

    def test_rejects_invalid_overrides_without_writing_configuration(self):
        for value in ("", "relative", "/tmp/cache\nline", "/tmp/cache\0line", "/tmp/cache\rline"):
            with self.subTest(value=value), tempfile.TemporaryDirectory() as directory:
                with patch("bazel_cache.os.environ", {"NUXIE_BAZEL_CACHE_DIR": value}):
                    with self.assertRaisesRegex(ValueError, "absolute path on one line"):
                        startup_options(Path(directory))
                self.assertEqual(list(Path(directory).iterdir()), [])


if __name__ == "__main__":
    unittest.main()
