#!/usr/bin/env python3

from __future__ import annotations

import json
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
            'TEST_CASE("script two", "[duplicate-name]") {}\n'
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
        self.case_ledger = self.repo / "test-correspondence-cases.json"
        self.write_manifest()
        self.write_case_ledger()
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
        max_pending: int = 0,
        alpha_status: str = "ported-direct",
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
                test_case_count = 4
                status_values = ["ported-differential", "ported-direct", "partial", "pending", "n-a"]
                case_ledger = "test-correspondence-cases.json"

                [ratchet]
                max_pending = {max_pending}

                [[file]]
                upstream = "tests/unit_tests/runtime/alpha_test.cpp"
                test_case_count = 1
                status = "{alpha_status}"
                evidence = {"[]" if alpha_status == "pending" else '["crates/runtime/src/lib.rs::alpha_works"]'}
                note = "The one upstream case is ported directly."

                [[file]]
                upstream = "tests/unit_tests/runtime/scripting/script_test.cpp"
                test_case_count = 3
                status = "{script_status}"
                evidence = {"[]" if script_status == "pending" else '["crates/scripting/tests/script.rs::script_one"]'}
                {f"covered_test_cases = {covered_test_cases}" if script_status == "partial" else ""}
                note = "Only the named upstream case is covered."
                '''
            ).lstrip()
        )

    def write_case_ledger(self, cases: list[dict[str, object]] | None = None, *, max_pending: int = 4) -> None:
        if cases is None:
            cases = [
                self.pending_case(
                    "tests/unit_tests/runtime/alpha_test.cpp", 1, 1, "alpha works"
                ),
                self.pending_case(
                    "tests/unit_tests/runtime/scripting/script_test.cpp",
                    1,
                    1,
                    "script one",
                ),
                self.pending_case(
                    "tests/unit_tests/runtime/scripting/script_test.cpp",
                    2,
                    3,
                    "script two",
                ),
                self.pending_case(
                    "tests/unit_tests/runtime/scripting/script_test.cpp",
                    3,
                    4,
                    "script two",
                ),
            ]
        self.case_ledger.write_text(
            json.dumps(
                {
                    "schema": "nuxie-test-case-correspondence/v1",
                    "schema_version": 1,
                    "upstream_ref": self.ref,
                    "source_globs": [
                        "tests/unit_tests/runtime/*.cpp",
                        "tests/unit_tests/runtime/scripting/*.cpp",
                    ],
                    "case_count": 4,
                    "status_values": ["pending", "direct", "differential", "adapted"],
                    "outcome_values": [
                        "unverified",
                        "pass",
                        "expected-red",
                        "not-applicable",
                    ],
                    "evidence_kinds": ["rust-test", "live-differential"],
                    "adaptation_kinds": [
                        "cxx-language-only",
                        "rust-safety",
                        "taffy",
                        "native-audio",
                        "native-scripting",
                    ],
                    "ratchet": {"max_pending": max_pending},
                    "cases": cases,
                },
                indent=2,
            )
            + "\n"
        )

    @staticmethod
    def pending_case(upstream: str, ordinal: int, line: int, name: str) -> dict[str, object]:
        return {
            "id": f"{upstream}#{ordinal}",
            "upstream": upstream,
            "ordinal": ordinal,
            "line": line,
            "name": name,
            "status": "pending",
            "outcome": "unverified",
            "evidence": [],
        }

    def cases(self) -> list[dict[str, object]]:
        return json.loads(self.case_ledger.read_text())["cases"]

    def test_valid_manifest_recensuses_files_cases_and_statuses(self) -> None:
        summary = check_manifest(self.repo, self.upstream, self.manifest)
        self.assertEqual(summary.files, 2)
        self.assertEqual(summary.test_cases, 4)
        self.assertEqual(summary.status_counts["partial"], 1)
        self.assertEqual(summary.case_status_counts["pending"], 4)

    def test_commented_out_test_case_is_not_counted(self) -> None:
        self.manifest.write_text(
            self.manifest.read_text().replace("test_case_count = 1", "test_case_count = 2", 1)
        )
        with self.assertRaisesRegex(CheckFailure, "alpha_test.cpp.*declares 2.*pin has 1"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_expected_status_counts_block_is_rejected(self) -> None:
        self.manifest.write_text(
            self.manifest.read_text().replace(
                "[ratchet]",
                "[expected_status_counts]\n"
                'ported-differential = 0\n'
                "ported-direct = 1\n"
                "partial = 1\n"
                "pending = 0\n"
                '"n-a" = 0\n'
                "\n[ratchet]",
            )
        )
        with self.assertRaisesRegex(CheckFailure, "expected_status_counts.*delete the block"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_pending_count_cannot_exceed_ratchet(self) -> None:
        self.write_manifest(max_pending=0, script_status="pending")
        with self.assertRaisesRegex(CheckFailure, "pending count 1 exceeds ratchet 0"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_pending_cannot_regrow_after_a_tracked_shrink(self) -> None:
        self.write_manifest(max_pending=1, script_status="pending")
        with self.assertRaisesRegex(
            CheckFailure, "script_test.cpp status pending regressed from historical partial"
        ):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_row_cannot_regress_after_a_tracked_promotion(self) -> None:
        self.write_manifest(script_status="ported-direct")
        subprocess.run(["git", "add", "."], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "promote script row"], cwd=self.repo, check=True
        )
        self.write_manifest(script_status="partial")
        with self.assertRaisesRegex(
            CheckFailure, "script_test.cpp status partial regressed from historical ported-direct"
        ):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_row_regression_is_caught_even_when_totals_balance(self) -> None:
        self.write_manifest(
            max_pending=1, alpha_status="pending", script_status="ported-direct"
        )
        with self.assertRaisesRegex(
            CheckFailure, "alpha_test.cpp status pending regressed from historical ported-direct"
        ):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_promotion_discarded_by_an_ours_merge_is_still_ratcheted(self) -> None:
        subprocess.run(
            ["git", "checkout", "-qb", "promote"], cwd=self.repo, check=True
        )
        self.write_manifest(script_status="ported-direct")
        subprocess.run(["git", "add", "."], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "promote script row"], cwd=self.repo, check=True
        )
        subprocess.run(["git", "checkout", "-q", "-"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "merge", "-q", "-s", "ours", "--no-edit", "promote"],
            cwd=self.repo,
            check=True,
        )
        self.write_manifest(script_status="partial")
        with self.assertRaisesRegex(
            CheckFailure, "script_test.cpp status partial regressed from historical ported-direct"
        ):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_unreadable_history_fails_closed(self) -> None:
        broken = pathlib.Path(self.temp.name) / "broken"
        broken.mkdir()
        manifest = broken / "test-correspondence-manifest.toml"
        manifest.write_text(self.manifest.read_text())
        with self.assertRaisesRegex(CheckFailure, "cannot read .* history"):
            check_manifest(broken, self.upstream, manifest)

    def test_shallow_clone_fails_closed(self) -> None:
        shallow = pathlib.Path(self.temp.name) / "shallow"
        subprocess.run(
            [
                "git",
                "clone",
                "-q",
                "--depth",
                "1",
                f"file://{self.repo}",
                str(shallow),
            ],
            check=True,
        )
        with self.assertRaisesRegex(CheckFailure, "clone is shallow"):
            check_manifest(
                shallow, self.upstream, shallow / "test-correspondence-manifest.toml"
            )

    def test_partial_rows_must_name_real_strict_subset_of_cases(self) -> None:
        self.write_manifest(covered_test_cases='["not upstream"]')
        with self.assertRaisesRegex(CheckFailure, "unknown covered_test_cases"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_nonzero_na_requires_explicit_cxx_language_adaptation(self) -> None:
        self.write_manifest(alpha_status="n-a")
        with self.assertRaisesRegex(CheckFailure, "nonzero n-a row requires adaptation"):
            check_manifest(self.repo, self.upstream, self.manifest)

        self.manifest.write_text(
            self.manifest.read_text().replace(
                'status = "n-a"\n',
                'status = "n-a"\nadaptation = "cxx-language-only"\n',
                1,
            )
        )
        summary = check_manifest(self.repo, self.upstream, self.manifest)
        self.assertEqual(summary.status_counts["n-a"], 1)

    def test_duplicate_upstream_names_remain_distinct_by_ordinal(self) -> None:
        summary = check_manifest(self.repo, self.upstream, self.manifest)
        duplicate_rows = [row for row in self.cases() if row["name"] == "script two"]
        self.assertEqual([row["ordinal"] for row in duplicate_rows], [2, 3])
        self.assertEqual(summary.case_status_counts["pending"], 4)

    def test_duplicate_case_identity_is_rejected(self) -> None:
        cases = self.cases()
        cases[1] = dict(cases[0])
        self.write_case_ledger(cases)
        with self.assertRaisesRegex(CheckFailure, "duplicate case identity"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_stale_case_line_is_rejected(self) -> None:
        cases = self.cases()
        cases[0]["line"] = 2
        self.write_case_ledger(cases)
        with self.assertRaisesRegex(CheckFailure, "stale line 2.*line is 1"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_stale_case_ordinal_is_rejected_as_a_census_mismatch(self) -> None:
        cases = self.cases()
        cases[2]["ordinal"] = 4
        cases[2]["id"] = "tests/unit_tests/runtime/scripting/script_test.cpp#4"
        cases.sort(key=lambda row: (str(row["upstream"]), int(row["ordinal"])))
        self.write_case_ledger(cases)
        with self.assertRaisesRegex(CheckFailure, "case census mismatch: missing=.*2.*extra=.*4"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_missing_case_row_is_rejected(self) -> None:
        cases = self.cases()
        cases.pop()
        self.write_case_ledger(cases)
        with self.assertRaisesRegex(CheckFailure, "declares 4 cases but contains 3"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_file_level_evidence_cannot_prove_a_case(self) -> None:
        report = self.repo / "docs/report.md"
        report.parent.mkdir()
        report.write_text("file-level report\n")
        cases = self.cases()
        cases[0].update(
            {
                "status": "direct",
                "outcome": "pass",
                "note": "Claims a direct port.",
                "evidence": [
                    {
                        "kind": "rust-test",
                        "path": "docs/report.md",
                        "line": 1,
                        "symbol": "alpha_port",
                    }
                ],
            }
        )
        self.write_case_ledger(cases, max_pending=3)
        with self.assertRaisesRegex(CheckFailure, "must point to a Rust source file"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_direct_case_requires_a_discovered_test_at_the_exact_locator(self) -> None:
        test_path = self.repo / "crates/runtime/tests/alpha.rs"
        test_path.parent.mkdir(parents=True)
        test_path.write_text("#[test]\nfn alpha_port() {}\n")
        cases = self.cases()
        cases[0].update(
            {
                "status": "direct",
                "outcome": "pass",
                "note": "Literal direct test.",
                "evidence": [
                    {
                        "kind": "rust-test",
                        "path": "crates/runtime/tests/alpha.rs",
                        "line": 2,
                        "symbol": "alpha_port",
                    }
                ],
            }
        )
        self.write_case_ledger(cases, max_pending=3)
        summary = check_manifest(self.repo, self.upstream, self.manifest)
        self.assertEqual(summary.case_status_counts["direct"], 1)

        cases[0]["evidence"][0]["line"] = 1
        self.write_case_ledger(cases, max_pending=3)
        with self.assertRaisesRegex(CheckFailure, "does not resolve alpha_port"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_expected_red_must_be_ignored_with_the_exact_missing_behavior_reason(self) -> None:
        test_path = self.repo / "crates/runtime/tests/alpha.rs"
        test_path.parent.mkdir(parents=True)
        reason = "expected-red: nested artboard frame propagation is missing"
        test_path.write_text(
            f'#[test]\n#[ignore = "{reason}"]\nfn alpha_port() {{}}\n'
        )
        cases = self.cases()
        cases[0].update(
            {
                "status": "direct",
                "outcome": "expected-red",
                "expected_red_reason": reason,
                "note": "Literal body is retained while production is missing.",
                "evidence": [
                    {
                        "kind": "rust-test",
                        "path": "crates/runtime/tests/alpha.rs",
                        "line": 3,
                        "symbol": "alpha_port",
                    }
                ],
            }
        )
        self.write_case_ledger(cases, max_pending=3)
        summary = check_manifest(self.repo, self.upstream, self.manifest)
        self.assertEqual(summary.case_outcome_counts["expected-red"], 1)

        cases[0]["expected_red_reason"] = "expected-red: a different reason"
        self.write_case_ledger(cases, max_pending=3)
        with self.assertRaisesRegex(CheckFailure, "does not match the Rust.*reason"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_generic_ignore_cannot_count_as_case_proof(self) -> None:
        test_path = self.repo / "crates/runtime/tests/alpha.rs"
        test_path.parent.mkdir(parents=True)
        test_path.write_text('#[test]\n#[ignore = "later"]\nfn alpha_port() {}\n')
        cases = self.cases()
        cases[0].update(
            {
                "status": "direct",
                "outcome": "pass",
                "note": "An ignored test must not count as passing.",
                "evidence": [
                    {
                        "kind": "rust-test",
                        "path": "crates/runtime/tests/alpha.rs",
                        "line": 3,
                        "symbol": "alpha_port",
                    }
                ],
            }
        )
        self.write_case_ledger(cases, max_pending=3)
        with self.assertRaisesRegex(CheckFailure, "pass evidence points to an ignored"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_live_differential_requires_an_explicit_executable_harness(self) -> None:
        harness = self.repo / "tools/differential.py"
        harness.parent.mkdir()
        harness.write_text("# live C++/Rust driver\n")
        cases = self.cases()
        cases[0].update(
            {
                "status": "differential",
                "outcome": "pass",
                "note": "The named live driver executes both sides.",
                "evidence": [
                    {
                        "kind": "live-differential",
                        "harness_path": "tools/differential.py",
                        "differential_id": "alpha",
                        "cpp_entry": "cpp_alpha",
                        "rust_entry": "rust_alpha",
                        "command": ["python3", "tools/differential.py", "alpha"],
                    }
                ],
            }
        )
        self.write_case_ledger(cases, max_pending=3)
        summary = check_manifest(self.repo, self.upstream, self.manifest)
        self.assertEqual(summary.case_status_counts["differential"], 1)

        cases[0]["evidence"][0]["harness_path"] = "docs/differential.md"
        self.write_case_ledger(cases, max_pending=3)
        with self.assertRaisesRegex(CheckFailure, "executable source harness"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_not_applicable_case_requires_explicit_cxx_language_adaptation(self) -> None:
        cases = self.cases()
        cases[0].update(
            {
                "status": "adapted",
                "outcome": "not-applicable",
                "note": "The observable is a C++ allocator-language contract.",
                "adaptation": {
                    "kind": "cxx-language-only",
                    "rationale": "Rust slices own the approved container boundary.",
                    "inapplicable_observable": "C++ allocator call count",
                },
            }
        )
        self.write_case_ledger(cases, max_pending=3)
        summary = check_manifest(self.repo, self.upstream, self.manifest)
        self.assertEqual(summary.case_outcome_counts["not-applicable"], 1)

        cases[0]["adaptation"]["kind"] = "taffy"
        self.write_case_ledger(cases, max_pending=3)
        with self.assertRaisesRegex(CheckFailure, "requires cxx-language-only adaptation"):
            check_manifest(self.repo, self.upstream, self.manifest)

    def test_case_ratchet_cannot_rise_or_forget_a_proven_case(self) -> None:
        test_path = self.repo / "crates/runtime/tests/cases.rs"
        test_path.parent.mkdir(parents=True)
        test_path.write_text("#[test]\nfn alpha_port() {}\n#[test]\nfn script_port() {}\n")
        cases = self.cases()
        cases[0].update(
            {
                "status": "direct",
                "outcome": "pass",
                "note": "Literal direct test.",
                "evidence": [
                    {
                        "kind": "rust-test",
                        "path": "crates/runtime/tests/cases.rs",
                        "line": 2,
                        "symbol": "alpha_port",
                    }
                ],
            }
        )
        self.write_case_ledger(cases, max_pending=3)
        subprocess.run(["git", "add", "."], cwd=self.repo, check=True)
        subprocess.run(["git", "commit", "-qm", "prove alpha"], cwd=self.repo, check=True)

        self.write_case_ledger(cases, max_pending=4)
        with self.assertRaisesRegex(CheckFailure, "max_pending 4 regressed"):
            check_manifest(self.repo, self.upstream, self.manifest)

        replacement = self.cases()
        replacement[0] = self.pending_case(
            "tests/unit_tests/runtime/alpha_test.cpp", 1, 1, "alpha works"
        )
        replacement[1].update(
            {
                "status": "direct",
                "outcome": "pass",
                "note": "Different case proof cannot replace alpha.",
                "evidence": [
                    {
                        "kind": "rust-test",
                        "path": "crates/runtime/tests/cases.rs",
                        "line": 4,
                        "symbol": "script_port",
                    }
                ],
            }
        )
        self.write_case_ledger(replacement, max_pending=3)
        with self.assertRaisesRegex(CheckFailure, "alpha_test.cpp#1 regressed"):
            check_manifest(self.repo, self.upstream, self.manifest)


if __name__ == "__main__":
    unittest.main()
