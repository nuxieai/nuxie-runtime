import json
import tempfile
import unittest
from pathlib import Path

import wasm_perf


class CorpusSelectionTests(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "assets").mkdir()
        (self.root / "assets" / "large.riv").write_bytes(b"L" * 100)
        (self.root / "assets" / "small.riv").write_bytes(b"S" * 10)
        (self.root / "assets" / "scripted.riv").write_bytes(b"X" * 90)
        self.corpus = self.root / "corpus.toml"
        self.corpus.write_text(
            """
[[file]]
id = "large"
path = "assets/large.riv"
samples = [0.0, 0.5]

[[file]]
id = "scripted"
path = "assets/scripted.riv"
input_script = "inputs/scripted.txt"
samples = [0.0]

[[file]]
id = "small"
path = "assets/small.riv"
samples = [0.25]
""",
            encoding="utf-8",
        )
        self.perf = self.root / "perf.toml"
        self.perf.write_text(
            """
schema = "nuxie-perf-corpus-v1"
source = "corpus.toml"

[[file]]
id = "large"
bytes = 100
categories = ["largest"]

[[file]]
id = "scripted"
bytes = 90
categories = ["largest", "scripted"]

[[file]]
id = "small"
bytes = 10
categories = ["largest"]
""",
            encoding="utf-8",
        )

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_selects_largest_supported_and_uses_first_sample(self):
        fixtures = wasm_perf.select_fixtures(
            self.perf, self.corpus, self.root, limit=2, requested_ids=[]
        )

        self.assertEqual([fixture["id"] for fixture in fixtures], ["large", "small"])
        self.assertEqual(fixtures[0]["sample_seconds"], 0.0)
        self.assertEqual(fixtures[1]["sample_seconds"], 0.25)

    def test_explicit_unsupported_fixture_fails_closed(self):
        with self.assertRaisesRegex(wasm_perf.ContractError, "scripted semantics"):
            wasm_perf.select_fixtures(
                self.perf, self.corpus, self.root, limit=1, requested_ids=["scripted"]
            )

    def test_explicit_image_fixture_fails_closed_without_production_decoder(self):
        corpus = self.corpus.read_text(encoding="utf-8").replace(
            'id = "large"\npath = "assets/large.riv"',
            'id = "large"\npath = "assets/large.riv"\nfeatures = ["type-key:105:ImageAsset"]',
        )
        self.corpus.write_text(corpus, encoding="utf-8")

        with self.assertRaisesRegex(wasm_perf.ContractError, "image decode semantics"):
            wasm_perf.select_fixtures(
                self.perf, self.corpus, self.root, limit=1, requested_ids=["large"]
            )

    def test_missing_fixture_fails_closed(self):
        (self.root / "assets" / "large.riv").unlink()

        with self.assertRaisesRegex(wasm_perf.ContractError, "missing fixture"):
            wasm_perf.select_fixtures(
                self.perf, self.corpus, self.root, limit=1, requested_ids=["large"]
            )

    def test_unknown_requested_id_fails_closed(self):
        with self.assertRaisesRegex(wasm_perf.ContractError, "unknown perf fixture"):
            wasm_perf.select_fixtures(
                self.perf, self.corpus, self.root, limit=1, requested_ids=["absent"]
            )


class ReportContractTests(unittest.TestCase):
    def test_audits_production_only_feature_tree_and_import(self):
        wasm_perf.audit_production_boundary(
            'browser-renderer-smoke\n└── nuxie feature "default"',
            "pub struct WasmPerfRunner; impl WasmPerfRunner { File::import(bytes); }",
        )

    def test_rejects_test_support_in_measured_feature_tree(self):
        with self.assertRaisesRegex(wasm_perf.ContractError, "test-support"):
            wasm_perf.audit_production_boundary(
                'nuxie feature "test-support"',
                "pub struct WasmPerfRunner; File::import(bytes);",
            )

    def test_rejects_unsigned_import_in_measured_runner(self):
        with self.assertRaisesRegex(wasm_perf.ContractError, "production File::import"):
            wasm_perf.audit_production_boundary(
                'nuxie feature "default"',
                "pub struct WasmPerfRunner; File::import_with_unsigned_scripts(bytes);",
            )

    def test_parses_native_report_contract(self):
        parsed = wasm_perf.parse_native_report(
            """rive-golden-benchmark-v1
elapsed_ms=12.5
total_ms=12.5
advance_ms=3.0
input_ms=0
prepare_ms=0
draw_ms=8.0
bookkeeping_ms=0.5
segments=10
"""
        )

        self.assertEqual(parsed["schema"], "rive-golden-benchmark-v1")
        self.assertEqual(parsed["segments"], 10)
        self.assertEqual(parsed["accounted_ms"], 11.0)

    def test_rejects_incomplete_browser_report(self):
        report = {
            "schema": "rive-golden-benchmark-v1",
            "elapsed_ms": 1.0,
            "advance_ms": 0.5,
            "draw_ms": 0.4,
            "segments": 1,
        }

        with self.assertRaisesRegex(wasm_perf.ContractError, "missing report field"):
            wasm_perf.validate_timing_report(report)

    def test_builds_report_only_comparison_with_variance(self):
        fixture = {
            "id": "large",
            "bytes": 100,
            "relative_path": "assets/large.riv",
            "sample_seconds": 0.0,
        }
        wasm_runs = [
            timing(12.0, 3.0, 8.0, 10),
            timing(14.0, 4.0, 9.0, 10),
            timing(13.0, 3.5, 8.5, 10),
        ]
        native_runs = [
            timing(6.0, 1.0, 4.0, 10),
            timing(7.0, 1.5, 4.5, 10),
            timing(6.5, 1.25, 4.25, 10),
        ]

        report = wasm_perf.build_comparison_report(
            [fixture],
            {"large": wasm_runs},
            {"large": native_runs},
            identity={
                "git_sha": "abc123",
                "rive_runtime_sha": "def456",
                "browser": "chrome",
                "build_profile": "release",
            },
            repeat=10,
        )

        row = report["fixtures"][0]
        self.assertEqual(report["schema"], "nuxie-wasm-perf-v1")
        self.assertEqual(report["conclusion"], "report-only")
        self.assertEqual(row["wasm"]["run_count"], 3)
        self.assertAlmostEqual(row["ratio"]["elapsed"], 2.0)
        self.assertGreater(row["wasm"]["elapsed_ms"]["coefficient_of_variation"], 0)

    def test_round_trips_machine_readable_json(self):
        payload = {"schema": "nuxie-wasm-perf-v1", "conclusion": "report-only"}
        self.assertEqual(json.loads(wasm_perf.canonical_json(payload)), payload)


def timing(elapsed, advance, draw, segments):
    accounted = advance + draw
    return {
        "schema": "rive-golden-benchmark-v1",
        "elapsed_ms": elapsed,
        "total_ms": elapsed,
        "advance_ms": advance,
        "input_ms": 0.0,
        "prepare_ms": 0.0,
        "draw_ms": draw,
        "accounted_ms": accounted,
        "bookkeeping_ms": max(elapsed - accounted, 0.0),
        "segments": segments,
    }


if __name__ == "__main__":
    unittest.main()
