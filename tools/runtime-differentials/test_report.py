import json
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import report


class RuntimeDifferentialReportTest(unittest.TestCase):
    def setUp(self):
        self.raw = tempfile.TemporaryDirectory()
        self.addCleanup(self.raw.cleanup)
        self.root = Path(self.raw.name)
        self.runtime = self.root / "runtime"
        (self.runtime / "tests/unit_tests/assets").mkdir(parents=True)
        (self.runtime / "tests/unit_tests/silvers").mkdir(parents=True)
        (self.root / "fixtures").mkdir()

    def test_golden_report_fingerprints_effective_scripted_classifications(self):
        (self.runtime / "tests/unit_tests/assets/exact.riv").write_bytes(b"exact")
        (self.root / "fixtures/scripted.riv").write_bytes(b"scripted")
        (self.root / "fixtures/input.txt").write_bytes(b"input")
        manifest = self.root / "corpus.toml"
        manifest.write_text(
            """
[[file]]
id = "exact"
path = "tests/unit_tests/assets/exact.riv"
status = "exact"

[[file]]
id = "scripted"
path = "fixtures/scripted.riv"
input_script = "fixtures/input.txt"
status = "exact"
scripted_divergence_signature = "line 4: rust `a` vs c++ `b`"
features = ["scripted-status:diverges"]
"""
        )
        runner = self.root / "runner"
        runner.write_bytes(b"runner")

        payload = report.build_golden_report(
            manifest,
            self.runtime,
            self.root,
            "scripted",
            "c" * 40,
            "d" * 40,
            [("rust", runner)],
        )

        self.assertEqual(payload["schema"], report.SCHEMA)
        self.assertEqual(payload["summary"], {"divergent": 1, "exact": 1})
        self.assertEqual(payload["cases"][1]["outcome"], "divergent")
        self.assertEqual(
            payload["cases"][1]["signature"],
            "line 4: rust `a` vs c++ `b`",
        )
        self.assertEqual(payload["runners"][0]["sha256"], report.sha256(runner))
        self.assertEqual(
            payload["cases"][1]["input_script"],
            report.fixture_record("fixtures/input.txt", self.root / "fixtures/input.txt"),
        )

    def test_ordinary_golden_report_does_not_claim_scripted_cases_ran(self):
        fixture = self.runtime / "tests/unit_tests/assets/scripted.riv"
        fixture.write_bytes(b"scripted")
        manifest = self.root / "corpus.toml"
        manifest.write_text(
            """
[[file]]
id = "scripted"
path = "tests/unit_tests/assets/scripted.riv"
rust_execute_scripts = true
status = "exact"
features = []
"""
        )

        ordinary = report.build_golden_report(
            manifest, self.runtime, self.root, "ordinary", "c" * 40, "d" * 40, []
        )
        scripted = report.build_golden_report(
            manifest, self.runtime, self.root, "scripted", "c" * 40, "d" * 40, []
        )

        self.assertFalse(ordinary["cases"][0]["executed"])
        self.assertTrue(scripted["cases"][0]["executed"])

    def test_golden_report_rejects_a_malformed_divergence_signature(self):
        fixture = self.runtime / "tests/unit_tests/assets/gap.riv"
        fixture.write_bytes(b"gap")
        manifest = self.root / "corpus.toml"
        manifest.write_text(
            """
[[file]]
id = "gap"
path = "tests/unit_tests/assets/gap.riv"
status = "diverges"
divergence_signature = "known mismatch"
"""
        )

        with self.assertRaisesRegex(report.ReportError, "malformed divergence signature"):
            report.build_golden_report(
                manifest, self.runtime, self.root, "ordinary", "c" * 40, "d" * 40, []
            )

    def test_report_rejects_a_checkout_at_a_different_commit(self):
        completed = mock.Mock(returncode=0, stdout="a" * 40 + "\n", stderr="")
        with mock.patch.object(report.subprocess, "run", return_value=completed):
            with self.assertRaisesRegex(report.ReportError, "checkout is"):
                report.validate_git_ref(self.root, "b" * 40, "runtime")

    def test_silver_report_preserves_all_outcome_classes_and_baseline_identity(self):
        fixture = self.runtime / "tests/unit_tests/assets/source.riv"
        fixture.write_bytes(b"fixture")
        baseline = self.runtime / "tests/unit_tests/silvers/source.sriv"
        baseline.write_bytes(b"baseline")
        manifest = self.root / "silver-corpus.toml"
        rows = []
        for status in ("exact", "diverges", "unsupported-feature", "pending-scripted"):
            note = (
                "Genuine gap; first difference: frame 0, op 1: values differ."
                if status == "diverges"
                else "classified"
            )
            rows.append(
                f'''[[case]]\nid = "{status}"\nexpected = "tests/unit_tests/silvers/source.sriv"\nsource = "source.riv"\nlane = "runtime"\nverification = "sriv-v1-epsilon"\nstatus = "{status}"\nnote = "{note}"\n'''
            )
        manifest.write_text(
            '[corpus]\nupstream_ref = "' + "c" * 40 + '"\n\n' + "\n".join(rows)
        )

        payload = report.build_silver_report(
            manifest, self.runtime, "d" * 40, []
        )

        self.assertEqual(
            payload["summary"],
            {"divergent": 1, "exact": 1, "pending": 1, "unsupported": 1},
        )
        self.assertEqual(payload["cases"][1]["baseline"]["sha256"], report.sha256(baseline))
        self.assertFalse(payload["cases"][3]["executed"])

    def test_silver_report_fingerprints_declared_and_action_loaded_inputs(self):
        assets = self.runtime / "tests/unit_tests/assets"
        (assets / "source.riv").write_bytes(b"fixture")
        (assets / "child.riv").write_bytes(b"dependency")
        (assets / "font.ttf").write_bytes(b"font")
        baseline = self.runtime / "tests/unit_tests/silvers/source.sriv"
        baseline.write_bytes(b"baseline")
        manifest = self.root / "silver-corpus.toml"
        manifest.write_text(
            '[corpus]\nupstream_ref = "'
            + "c" * 40
            + '''"

[[case]]
id = "with-inputs"
expected = "tests/unit_tests/silvers/source.sriv"
source = "source.riv"
dependencies = ["child.riv"]
lane = "runtime"
verification = "sriv-v1-epsilon"
status = "exact"
actions = [{ kind = "set-view-model-font-bytes", property = "font", source = "font.ttf" }]
note = "Provenanced."
'''
        )

        payload = report.build_silver_report(manifest, self.runtime, "d" * 40, [])
        case = payload["cases"][0]

        self.assertEqual(case["dependencies"], [report.fixture_record("child.riv", assets / "child.riv")])
        self.assertEqual(
            case["action_fixtures"],
            [report.fixture_record("font.ttf", assets / "font.ttf")],
        )

    def test_atomic_json_round_trips(self):
        output = self.root / "reports/report.json"
        report.write_json(output, {"schema": report.SCHEMA})
        self.assertEqual(json.loads(output.read_text()), {"schema": report.SCHEMA})

    def test_failed_gate_can_record_an_unavailable_runner(self):
        missing = self.root / "validator-that-did-not-build"

        self.assertEqual(
            report.runner_records([("validator", missing)], allow_missing=True),
            [
                {
                    "role": "validator",
                    "path": str(missing),
                    "sha256": None,
                    "missing": True,
                }
            ],
        )
        with self.assertRaisesRegex(report.ReportError, "cannot fingerprint"):
            report.runner_records([("validator", missing)])

    def test_failure_diagnostics_promote_newly_exact_to_a_machine_outcome(self):
        payload = {
            "summary": {"divergent": 2},
            "cases": [
                {"id": "golden-gap", "outcome": "divergent", "executed": True},
                {"id": "silver-gap", "outcome": "divergent", "executed": True},
            ],
        }

        report.apply_diagnostics(
            payload,
            "failure: golden-gap now compares exact; promote it explicitly\n"
            "silver-corpus error: silver-gap is classified diverges but now compares exact; promote it\n",
            gate_rc=1,
        )

        self.assertEqual(payload["summary"], {"newly-exact": 2})
        self.assertEqual(payload["gate_status"], "failed")
        self.assertEqual(payload["cases"][0]["outcome"], "newly-exact")
        self.assertIn("promote it", payload["cases"][1]["diagnostic"])

    def test_failure_diagnostics_reject_an_unknown_case(self):
        payload = {"summary": {}, "cases": []}
        with self.assertRaisesRegex(report.ReportError, "unknown case missing"):
            report.apply_diagnostics(
                payload,
                "failure: missing now compares exact; promote it explicitly",
                gate_rc=1,
            )

    def test_gate_rc_and_changed_divergence_are_attributed_to_the_case(self):
        payload = {
            "lane": "silver",
            "summary": {"divergent": 2, "exact": 1},
            "cases": [
                {"id": "verified", "outcome": "divergent", "executed": True},
                {"id": "changed", "outcome": "divergent", "executed": True},
                {"id": "later", "outcome": "exact", "executed": True},
            ],
        }

        report.apply_diagnostics(
            payload,
            "[divergent] verified: frame 0, op 1: stable\n"
            "[diverges] changed: c++ stream ok (12 bytes)\n"
            "silver-corpus error: changed divergence changed: recorded frame 0, op 1: old; "
            "actual frame 0, op 1: new\n",
            gate_rc=101,
        )

        self.assertEqual(payload["gate_status"], "failed")
        self.assertTrue(payload["cases"][0]["executed"])
        self.assertEqual(payload["cases"][0]["divergence_check"], "verified")
        self.assertTrue(payload["cases"][1]["executed"])
        self.assertEqual(payload["cases"][1]["divergence_check"], "changed")
        self.assertIn("actual frame", payload["cases"][1]["diagnostic"])
        self.assertFalse(payload["cases"][2]["executed"])

    def test_failed_golden_report_only_marks_terminal_rust_results_executed(self):
        payload = {
            "lane": "golden-ordinary",
            "summary": {"exact": 3},
            "cases": [
                {"id": "skipped", "outcome": "exact", "executed": True},
                {"id": "cpp-only", "outcome": "exact", "executed": True},
                {"id": "verified", "outcome": "exact", "executed": True},
            ],
        }

        report.apply_diagnostics(
            payload,
            "[exact] skipped: skipped (requires scripted runners)\n"
            "[exact] cpp-only: c++ stream ok (12 bytes)\n"
            "[exact] verified: c++ stream ok (12 bytes)\n"
            "[exact] verified: rust comparison verified\n",
            gate_rc=1,
        )

        self.assertFalse(payload["cases"][0]["executed"])
        self.assertFalse(payload["cases"][1]["executed"])
        self.assertTrue(payload["cases"][2]["executed"])

    def test_exact_regressions_are_attributed_to_the_executed_case(self):
        payload = {
            "lane": "golden-scripted",
            "summary": {"exact": 2},
            "cases": [
                {"id": "golden-regression", "outcome": "exact", "executed": True},
                {"id": "silver-regression", "outcome": "exact", "executed": True},
            ],
        }

        report.apply_diagnostics(
            payload,
            "failure: golden-regression: stream differs from C++ under byte verification: "
            "line 2: rust `a` vs c++ `b`\n"
            "silver-corpus error: silver-regression exact entry diverged: "
            "frame 0, op 1: expected fill, got stroke\n",
            gate_rc=1,
        )

        self.assertEqual(payload["gate_status"], "failed")
        self.assertEqual(payload["summary"], {"regressed": 2})
        for case in payload["cases"]:
            self.assertTrue(case["executed"])
            self.assertEqual(case["outcome"], "regressed")
            self.assertRegex(case["diagnostic"], r"differ|diverge")

    def test_nonzero_gate_rc_cannot_be_misreported_as_passed(self):
        payload = {"lane": "golden-scripted", "summary": {}, "cases": []}
        report.apply_diagnostics(payload, "cargo failed before execution", gate_rc=137)
        self.assertEqual(payload["gate_status"], "failed")

    def test_unknown_silver_provenance_retains_a_missing_fixture_identity(self):
        baseline = self.runtime / "tests/unit_tests/silvers/unknown.sriv"
        baseline.write_bytes(b"baseline")
        manifest = self.root / "silver-corpus.toml"
        manifest.write_text(
            '[corpus]\nupstream_ref = "'
            + "c" * 40
            + '''"

[[case]]
id = "unknown"
expected = "tests/unit_tests/silvers/unknown.sriv"
source = "provenance-unknown"
lane = "unknown"
verification = "sriv-v1-epsilon"
status = "provenance-unknown"
note = "No producer exists."
'''
        )

        payload = report.build_silver_report(manifest, self.runtime, "d" * 40, [])

        self.assertEqual(
            payload["cases"][0]["fixture"],
            {"path": "provenance-unknown", "sha256": None, "missing": True},
        )

    def test_provenanced_silver_case_requires_its_fixture(self):
        baseline = self.runtime / "tests/unit_tests/silvers/missing.sriv"
        baseline.write_bytes(b"baseline")
        manifest = self.root / "silver-corpus.toml"
        manifest.write_text(
            '[corpus]\nupstream_ref = "'
            + "c" * 40
            + '''"

[[case]]
id = "missing"
expected = "tests/unit_tests/silvers/missing.sriv"
source = "missing.riv"
lane = "runtime"
verification = "sriv-v1-epsilon"
status = "exact"
note = "Provenanced."
'''
        )

        with self.assertRaisesRegex(report.ReportError, "cannot fingerprint"):
            report.build_silver_report(manifest, self.runtime, "d" * 40, [])

    def test_scripted_inline_source_is_an_explicit_virtual_fixture(self):
        baseline = self.runtime / "tests/unit_tests/silvers/scripted.sriv"
        baseline.write_bytes(b"baseline")
        manifest = self.root / "silver-corpus.toml"
        manifest.write_text(
            '[corpus]\nupstream_ref = "'
            + "c" * 40
            + '''"

[[case]]
id = "scripted"
expected = "tests/unit_tests/silvers/scripted.sriv"
source = "inline-script"
lane = "scripted"
verification = "sriv-v1-epsilon"
status = "pending-scripted"
note = "The fixture is produced by an inline script."
'''
        )

        payload = report.build_silver_report(manifest, self.runtime, "d" * 40, [])
        self.assertEqual(
            payload["cases"][0]["fixture"],
            {"path": "inline-script", "sha256": None, "virtual": True},
        )


if __name__ == "__main__":
    unittest.main()
