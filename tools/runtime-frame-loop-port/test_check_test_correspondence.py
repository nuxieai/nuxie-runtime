#!/usr/bin/env python3

from __future__ import annotations

import pathlib
import subprocess
import sys
import tempfile
import textwrap
import unittest


TOOL_DIR = pathlib.Path(__file__).resolve().parent
if str(TOOL_DIR) not in sys.path:
    sys.path.insert(0, str(TOOL_DIR))

from check_test_correspondence import CheckFailure, check_manifest


class TestCorrespondenceCheckTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        root = pathlib.Path(self.temp.name)
        self.repo = root / "repo"
        self.upstream = root / "rive-runtime"
        self.repo.mkdir()
        runtime = self.upstream / "tests/unit_tests/runtime"
        (runtime / "scripting").mkdir(parents=True)
        (runtime / "alpha_test.cpp").write_text(
            'TEST_CASE("alpha works", "[runtime]") {}\n'
            '/* TEST_CASE("disabled", "[runtime]") {} */\n'
        )
        (runtime / "scripting/script_test.cpp").write_text(
            'TEST_CASE(\n    "script " "one", "[scripting]") {}\n'
            'TEST_CASE("script two", "[scripting]") {}\n'
        )
        subprocess.run(["git", "init", "-q"], cwd=self.upstream, check=True)
        subprocess.run(
            ["git", "config", "user.email", "test@example.com"],
            cwd=self.upstream,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "Test"],
            cwd=self.upstream,
            check=True,
        )
        subprocess.run(["git", "add", "."], cwd=self.upstream, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "fixture"], cwd=self.upstream, check=True
        )
        self.ref = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.upstream,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        self.manifest = self.repo / "test-correspondence-manifest.toml"
        self.write_manifest()
        subprocess.run(["git", "init", "-q"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "config", "user.email", "test@example.com"],
            cwd=self.repo,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "Test"], cwd=self.repo, check=True
        )
        subprocess.run(["git", "add", "."], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "fixture"], cwd=self.repo, check=True
        )

    def write_manifest(
        self,
        *,
        expected_pending: int = 0,
        max_pending: int = 0,
        script_status: str = "partial",
        covered_test_cases: str = '["script one"]',
    ) -> None:
        self.manifest.write_text(
            textwrap.dedent(
                f'''
                schema = "nuxie-test-correspondence/v1"
                schema_version = 1
                upstream_repository = "rive-app/rive-runtime"
                upstream_ref = "{self.ref}"
                source_globs = ["tests/unit_tests/runtime/*.cpp", "tests/unit_tests/runtime/scripting/*.cpp"]
                row_count = 2
                test_case_count = 3
                status_values = ["ported-differential", "ported-direct", "partial", "pending", "n-a"]

                [expected_status_counts]
                ported-differential = 0
                ported-direct = 1
                partial = {1 if script_status == "partial" else 0}
                pending = {expected_pending}
                n-a = 0

                [ratchet]
                max_pending = {max_pending}

                [[file]]
                upstream = "tests/unit_tests/runtime/alpha_test.cpp"
                test_case_count = 1
                status = "ported-direct"
                evidence = ["crates/runtime/src/lib.rs::alpha_works"]
                note = "The one upstream case is ported directly."

                [[file]]
                upstream = "tests/unit_tests/runtime/scripting/script_test.cpp"
                test_case_count = 2
                status = "{script_status}"
                evidence = {"[]" if script_status == "pending" else '["crates/scripting/tests/script.rs::script_one"]'}
                {f"covered_test_cases = {covered_test_cases}" if script_status == "partial" else ""}
                note = "Only the named upstream case is covered."
                '''
            ).lstrip()
        )

    def test_valid_manifest_recensuses_files_cases_and_statuses(self) -> None:
        summary = check_manifest(self.repo, self.upstream, self.manifest)
        self.assertEqual(summary.files, 2)
        self.assertEqual(summary.test_cases, 3)
        self.assertEqual(summary.status_counts["partial"], 1)

    def test_commented_out_test_case_is_not_counted(self) -> None:
        self.manifest.write_text(
            self.manifest.read_text().replace("test_case_count = 1", "test_case_count = 2", 1)
        )
        with self.assertRaisesRegex(CheckFailure, "alpha_test.cpp.*declares 2.*pin has 1"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_expected_status_counts_are_exact(self) -> None:
        self.manifest.write_text(
            self.manifest.read_text().replace("partial = 1", "partial = 0")
        )
        with self.assertRaisesRegex(CheckFailure, "expected_status_counts.partial=0.*actual=1"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_pending_count_cannot_exceed_ratchet(self) -> None:
        self.write_manifest(expected_pending=1, max_pending=0, script_status="pending")
        with self.assertRaisesRegex(CheckFailure, "pending count 1 exceeds ratchet 0"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_pending_cannot_regrow_after_a_tracked_shrink(self) -> None:
        self.write_manifest(expected_pending=1, max_pending=1, script_status="pending")
        with self.assertRaisesRegex(CheckFailure, "pending count 1 regressed from baseline 0"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_partial_rows_must_name_real_strict_subset_of_cases(self) -> None:
        self.write_manifest(covered_test_cases='["not upstream"]')
        with self.assertRaisesRegex(CheckFailure, "unknown covered_test_cases"):
            check_manifest(self.repo, self.upstream, self.manifest)


if __name__ == "__main__":
    unittest.main()
