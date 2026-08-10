import copy
import gzip
import hashlib
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path


TOOL = Path(__file__).with_name("perf_gate.py")
REPO_ROOT = TOOL.parents[2]
SPEC = importlib.util.spec_from_file_location("perf_gate", TOOL)
assert SPEC is not None and SPEC.loader is not None
PERF_GATE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = PERF_GATE
SPEC.loader.exec_module(PERF_GATE)


class PerfCorpusTests(unittest.TestCase):
    def setUp(self):
        self.manifest = PERF_GATE.load_manifest(REPO_ROOT / "perf-corpus.toml")
        self.corpus = PERF_GATE.load_toml(REPO_ROOT / "corpus.toml")

    def test_checked_in_manifest_is_broad_and_diverse(self):
        PERF_GATE.validate_manifest(
            self.manifest,
            self.corpus,
            corpus_path=REPO_ROOT / "corpus.toml",
            rive_runtime_dir=None,
        )
        self.assertGreaterEqual(len(self.manifest.files), 20)

    def test_data_viz_is_enrolled_with_the_measured_ratchet(self):
        data_viz = next(
            file for file in self.manifest.files if file.id == "data_viz_demo"
        )

        self.assertEqual(data_viz.file_bytes, 1_652_339)
        self.assertEqual(data_viz.baseline_ratio, 244.770162)
        self.assertEqual(data_viz.ceiling, 282)
        self.assertEqual(
            set(data_viz.categories),
            {
                "largest",
                "layout-heavy",
                "nested-artboards",
                "scripted",
                "text-heavy",
            },
        )

    def test_data_viz_profile_attribution_is_retained_and_derivable(self):
        evidence = (
            REPO_ROOT / "docs" / "evidence" / "univ-1687-data-viz-2026-08-10"
        )
        folded_path = evidence / "pre-fix-profile.folded.gz"
        summary_path = evidence / "pre-fix-profile-summary.json"
        source_patch_path = evidence / "pre-fix-source.patch"

        self.assertEqual(
            hashlib.sha256(folded_path.read_bytes()).hexdigest(),
            "f019028868e454195117590f68b462df0f27fc0895cb81449b60ed732d40dc62",
        )
        self.assertEqual(
            hashlib.sha256(source_patch_path.read_bytes()).hexdigest(),
            "c4c3ce44e72d0c748880cb871a0e8920829761bcd7b3b43a125a6f3096e90b8d",
        )
        self.assertEqual(
            hashlib.sha256(summary_path.read_bytes()).hexdigest(),
            "88d4743f1138ae8abb0b8c9248ace83cc8594f3f669c6881f657fc26c1432468",
        )

        folded = gzip.decompress(folded_path.read_bytes()).decode()
        stacks = []
        for line in folded.splitlines():
            stack, samples = line.rsplit(" ", 1)
            stacks.append((set(stack.split(";")), int(samples)))

        summary = json.loads(summary_path.read_text())
        self.assertEqual(summary["schema"], "nuxie-xctrace-folded/v1")
        self.assertEqual(summary["sample_period_ms"], 1)
        self.assertEqual(summary["sample_rows"], 5_780)
        self.assertEqual(summary["total_samples"], 5_780)
        self.assertEqual(summary["unique_stacks"], 1_859)
        self.assertEqual(sum(samples for _, samples in stacks), 5_780)

        inclusive = {
            frame: sum(samples for stack, samples in stacks if frame in stack)
            for frame in (
                "<nuxie_runtime::draw::TaffyRuntimeLayoutEngine>::compute_layout_with_root_hug",
                "nuxie_runtime::text::static_text_layout_measure_bounds",
                "<harfrust::hb::face::hb_font_t>::shape",
                "<nuxie_runtime::artboard::ArtboardInstance>::sync_style_changes",
                "<nuxie_runtime::draw::TaffyRuntimeLayoutEngine>::nested_artboard_layout_axis_hug_size",
                "<nuxie_runtime::artboard::ArtboardInstance>::apply_nested_artboard_layout_bounds",
                "<nuxie_runtime::artboard::ArtboardInstance>::refresh_layout_constraint_bounds",
            )
        }
        self.assertEqual(
            tuple(inclusive.values()),
            (5_382, 5_022, 3_884, 2_739, 2_568, 2_464, 2_408),
        )

    def test_blocking_gate_is_wired_into_make_landing_and_ci(self):
        makefile = (REPO_ROOT / "Makefile").read_text()
        land = (REPO_ROOT / "tools" / "land.sh").read_text()
        workflow = (REPO_ROOT / ".github" / "workflows" / "ci.yml").read_text()
        perf_job = workflow.split("\n  perf-gate:", 1)[1].split(
            "\n  parity-scorecard:", 1
        )[0]

        self.assertIn("perf-gate-measure: perf-runtime-ref-check", makefile)
        self.assertIn("perf-gate: perf-gate-measure", makefile)
        self.assertIn("perf-gate-tighten:", makefile)
        self.assertIn("timing_gates=(perf-gate-tighten)", land)
        self.assertIn('cat "$cache/$g.log"', land)
        self.assertIn('git diff --quiet -- perf-corpus.toml', land)
        self.assertIn("commit the tightened perf-corpus.toml", land)
        self.assertIn("make perf-gate PERF_JSON_META=", perf_job)
        self.assertNotIn("continue-on-error", perf_job)
        self.assertNotIn("make perf-hot-loop", workflow)
        self.assertIn("      - perf-gate", workflow)

    def test_manifest_rejects_a_parked_source_entry(self):
        changed = copy.deepcopy(self.corpus)
        selected = {file.id for file in self.manifest.files}
        entry = next(file for file in changed["file"] if file["id"] in selected)
        entry["status"] = "diverges"

        with self.assertRaisesRegex(ValueError, "must remain exact"):
            PERF_GATE.validate_manifest(
                self.manifest,
                changed,
                corpus_path=REPO_ROOT / "corpus.toml",
                rive_runtime_dir=None,
            )

    def test_manifest_rejects_missing_feature_diversity(self):
        files = tuple(
            PERF_GATE.PerfFile(
                file.id,
                file.file_bytes,
                ("largest",),
                file.note,
                file.baseline_ratio,
                file.ceiling,
            )
            for file in self.manifest.files
        )
        manifest = PERF_GATE.PerfManifest(
            self.manifest.source, self.manifest.minimum_files, files
        )

        with self.assertRaisesRegex(ValueError, "missing required diversity"):
            PERF_GATE.validate_manifest(
                manifest,
                self.corpus,
                corpus_path=REPO_ROOT / "corpus.toml",
                rive_runtime_dir=None,
            )

    def test_manifest_ceiling_cannot_be_loosened_without_a_baseline(self):
        contents = (REPO_ROOT / "perf-corpus.toml").read_text()
        contents = contents.replace("ceiling = 24", "ceiling = 25", 1)
        (REPO_ROOT / "target").mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=REPO_ROOT / "target") as directory:
            path = Path(directory) / "perf-corpus.toml"
            path.write_text(contents)
            with self.assertRaisesRegex(ValueError, r"must equal ceil"):
                PERF_GATE.load_manifest(path)

    def test_report_enforces_every_per_file_ceiling(self):
        report = make_report(self.manifest)
        rows = PERF_GATE.evaluate_report(
            self.manifest, report, Path("report.json")
        )
        self.assertTrue(all(row.passed for row in rows))

        first = self.manifest.files[0]
        report["files"][0]["runners"]["rust"]["phases"]["advance_draw"][
            "median_ms"
        ] = (first.ceiling + 0.001) * 100
        rows = PERF_GATE.evaluate_report(
            self.manifest, report, Path("report.json")
        )
        self.assertFalse(rows[0].passed)
        self.assertTrue(all(row.passed for row in rows[1:]))

    def test_ratchet_updates_only_improved_baselines(self):
        report = make_report(self.manifest)
        first, second = self.manifest.files[:2]
        report["files"][0]["runners"]["rust"]["phases"]["advance_draw"][
            "median_ms"
        ] = first.baseline_ratio * 0.5 * 100
        report["files"][1]["runners"]["rust"]["phases"]["advance_draw"][
            "median_ms"
        ] = second.baseline_ratio * 1.01 * 100
        rows = PERF_GATE.evaluate_report(
            self.manifest, report, Path("report.json")
        )

        updates = PERF_GATE.ratchet_updates(self.manifest, rows)

        self.assertIn(first.id, updates)
        self.assertLess(updates[first.id][1], first.ceiling)
        self.assertNotIn(second.id, updates)

    def test_tighten_uses_the_worst_ratio_from_three_sessions(self):
        sessions = []
        for multiplier in (0.80, 0.95, 0.85):
            report = make_report(self.manifest)
            report["files"][0]["runners"]["rust"]["phases"]["advance_draw"][
                "median_ms"
            ] *= multiplier
            sessions.append(
                PERF_GATE.evaluate_report(self.manifest, report, Path("report.json"))
            )

        rows = PERF_GATE.maximum_rows(tuple(sessions))
        updates = PERF_GATE.ratchet_updates(self.manifest, rows)

        self.assertAlmostEqual(
            updates[self.manifest.files[0].id][0],
            round(self.manifest.files[0].baseline_ratio * 0.95, 6),
        )

    def test_tightened_manifest_preserves_notes_and_revalidates(self):
        first = self.manifest.files[0]
        improved = round(first.baseline_ratio * 0.5, 6)
        updates = {first.id: (improved, PERF_GATE.math.ceil(improved * 1.15))}
        (REPO_ROOT / "target").mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=REPO_ROOT / "target") as directory:
            path = Path(directory) / "perf-corpus.toml"
            original = (REPO_ROOT / "perf-corpus.toml").read_text()
            path.write_text(original)

            PERF_GATE.write_tightened_manifest(path, updates)
            tightened = PERF_GATE.load_manifest(path)

        tightened_first = tightened.files[0]
        self.assertEqual(tightened_first.note, first.note)
        self.assertEqual(tightened_first.baseline_ratio, updates[first.id][0])
        self.assertEqual(tightened_first.ceiling, updates[first.id][1])


def make_report(manifest):
    files = []
    for file in manifest.files:
        files.append(
            {
                "id": file.id,
                "segments": 100,
                "runners": {
                    "cpp": {
                        "phases": {"advance_draw": {"median_ms": 100.0}}
                    },
                    "rust": {
                        "phases": {
                            "advance_draw": {"median_ms": file.baseline_ratio * 100}
                        }
                    },
                },
            }
        )
    return {
        "schema": "rive-perf-compare-json-v1",
        "metric": "runner_hot_loop_ms",
        "iterations": 5,
        "warmups": 0,
        "benchmark_repeat": 1,
        "benchmark_frames": 100,
        "benchmark_hz": 60.0,
        "rust_execute_scripts": True,
        "meta": {"build_profile": "release"},
        "files": files,
    }


if __name__ == "__main__":
    unittest.main()
