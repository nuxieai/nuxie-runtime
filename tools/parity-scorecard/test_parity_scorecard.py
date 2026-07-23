import json
import os
import re
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


TOOL = Path(__file__).with_name("parity_scorecard.py")
REPO_ROOT = TOOL.parents[2]
RUNTIME_ENTRIES = 317
RUNTIME_SEGMENTS = 647
RENDERER_ENTRIES = 1468
GATE_COMMANDS = {
    "golden-compare": ["make", "golden-compare"],
    "scripted-golden-compare": ["make", "scripted-golden-compare"],
    "renderer-golden": ["make", "renderer-golden"],
    "cargo-test-workspace": ["make", "cpp-oracle-workspace-tests"],
    "capi-smoke": ["make", "capi-smoke"],
    "size-report": ["make", "size-report"],
}


def golden_summary(entries=RUNTIME_ENTRIES, segments=RUNTIME_SEGMENTS):
    return (
        f"golden-compare summary: entries={entries} exact={entries} "
        f"exact-segments={segments} diverges=0 unsupported-feature=0 not-yet=0\n"
    )


def renderer_summary(entries=RENDERER_ENTRIES):
    return (
        f"renderer-corpus exact={entries} byte-exact={entries} "
        f"diverges=0 gated=0 total={entries}\n"
    )


def size_summary(off=7534056, on=8335288, budget=8388608):
    return f"size-report summary: off-bytes={off} on-bytes={on} budget-bytes={budget}\n"


class ParityScorecardCliTests(unittest.TestCase):
    def test_workspace_floor_cannot_silently_skip_the_pinned_cpp_oracles(self):
        workflow = (REPO_ROOT / ".github" / "workflows" / "ci.yml").read_text()
        trusted_workflow = (
            REPO_ROOT / ".github" / "workflows" / "_trusted-macos.yml"
        ).read_text()
        makefile = (REPO_ROOT / "Makefile").read_text()

        self.assertEqual(
            GATE_COMMANDS["cargo-test-workspace"],
            ["make", "cpp-oracle-workspace-tests"],
        )
        self.assertRegex(
            makefile,
            re.compile(
                r"cpp-oracle-workspace-tests: fixtures golden-runner cpp-probe\s+"
                r'@test -x "\$\(GOLDEN_RUNNER\)"[\s\S]{0,300}'
                r'@test -x "\$\(CPP_PROBE\)"[\s\S]{0,300}'
                r'RIVE_GOLDEN_RUNNER="\$\(GOLDEN_RUNNER\)" '
                r'RIVE_CPP_PROBE="\$\(CPP_PROBE\)" cargo test --workspace'
            ),
        )
        self.assertIn("-- make cpp-oracle-workspace-tests", workflow)
        self.assertIn("run: make cpp-oracle-workspace-tests", trusted_workflow)

    def test_ci_records_static_renderer_floor_and_keeps_same_runner_separate(self):
        workflow = (REPO_ROOT / ".github" / "workflows" / "ci.yml").read_text()

        self.assertRegex(
            workflow,
            re.compile(
                r"--gate renderer-golden[\s\\]+"
                r"--output target/parity-scorecard/evidence/renderer-golden\.json[\s\\]+"
                r"-- make renderer-golden(?:\s|$)"
            ),
        )
        self.assertNotRegex(
            workflow,
            re.compile(
                r"--gate renderer-golden[\s\S]{0,240}"
                r"-- make renderer-golden-same-runner"
            ),
        )
        self.assertIn(
            "- name: Verify same-runner renderer pixels (non-scorecard)",
            workflow,
        )

    def test_same_runner_uses_current_runtime_without_relabeling_historical_oracle(self):
        workflow = (REPO_ROOT / ".github" / "workflows" / "ci.yml").read_text()
        makefile = (REPO_ROOT / "Makefile").read_text()
        live_bootstrap = (
            REPO_ROOT / "tools" / "renderer-dawn-live-reference-bootstrap.sh"
        ).read_text()
        live_cache_key = (
            REPO_ROOT / "tools" / "renderer-dawn-live-reference-cache-key.sh"
        ).read_text()

        self.assertIn(
            "RIVE_RUNTIME_REF: 7c778d13c5d903b3b74eec1dd6bb68a811dea5f2",
            workflow,
        )
        self.assertIn(
            "RIVE_SAME_RUNNER_RUNTIME_REF: "
            "d788e8ec6e8b598526607d6a1e8818e8b637b60c",
            workflow,
        )
        self.assertIn("make renderer-dawn-reference-bootstrap", workflow)
        self.assertIn("tools/renderer-dawn-live-reference-cache-key.sh", workflow)
        self.assertRegex(
            workflow,
            re.compile(
                r"Bootstrap current-runtime Dawn live reference replay[\s\S]{0,1200}"
                r"RIVE_RUNTIME_DIR: \$\{\{ github\.workspace \}\}/rive-runtime-shaders"
            ),
        )
        self.assertRegex(
            makefile,
            re.compile(
                r"renderer-golden-same-runner: "
                r"renderer-rust-replay-release renderer-dawn-live-reference-check"
            ),
        )
        self.assertIn(
            '--reference-replay "$(RENDERER_DAWN_LIVE_REFERENCE_REPLAY)"',
            makefile,
        )
        dependency_manifest = "renderer-dawn-live-reference-dependencies.txt"
        self.assertIn(dependency_manifest, live_bootstrap)
        self.assertIn(dependency_manifest, live_cache_key)

    def test_record_streams_gate_output_and_preserves_nonzero_exit_status(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary = Path(temporary_directory)
            evidence = temporary / "golden-compare.json"
            fake_bin = temporary / "bin"
            fake_bin.mkdir()
            fake_make = fake_bin / "make"
            fake_make.write_text("#!/bin/sh\nprintf 'gate output\\n'\nexit 7\n")
            fake_make.chmod(0o755)
            environment = os.environ.copy()
            environment["PATH"] = f"{fake_bin}{os.pathsep}{environment['PATH']}"

            completed = subprocess.run(
                [
                    sys.executable,
                    str(TOOL),
                    "record",
                    "--gate",
                    "golden-compare",
                    "--output",
                    str(evidence),
                    "--source-sha",
                    "test-sha",
                    "--",
                    "make",
                    "golden-compare",
                ],
                text=True,
                capture_output=True,
                env=environment,
            )

            self.assertEqual(completed.returncode, 7)
            self.assertEqual(completed.stdout, "gate output\n")
            record = json.loads(evidence.read_text())
            self.assertEqual(record["schema"], "nuxie-parity-gate-evidence-v1")
            self.assertEqual(record["gate"], "golden-compare")
            self.assertEqual(record["source_sha"], "test-sha")
            self.assertEqual(record["exit_code"], 7)
            self.assertEqual(record["command"], ["make", "golden-compare"])
            self.assertEqual(record["output"], "gate output\n")

    def test_record_rejects_a_command_that_does_not_match_the_gate(self):
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary = Path(temporary_directory)
            evidence = temporary / "capi-smoke.json"
            marker = temporary / "command-ran"

            completed = subprocess.run(
                [
                    sys.executable,
                    str(TOOL),
                    "record",
                    "--gate",
                    "capi-smoke",
                    "--output",
                    str(evidence),
                    "--source-sha",
                    "test-sha",
                    "--",
                    sys.executable,
                    "-c",
                    f"from pathlib import Path; Path({str(marker)!r}).touch()",
                ],
                text=True,
                capture_output=True,
            )

            self.assertEqual(completed.returncode, 2)
            self.assertIn("capi-smoke command mismatch", completed.stderr)
            self.assertFalse(marker.exists())
            self.assertFalse(evidence.exists())

    def test_check_rejects_failed_gate_even_when_its_summary_looks_green(self):
        repo, evidence = self.create_green_repo()
        self.write_evidence(
            evidence / "scripted-golden-compare.json",
            "scripted-golden-compare",
            golden_summary(),
            exit_code=1,
        )

        completed = self.run_check(repo)

        self.assertEqual(completed.returncode, 1)
        self.assertIn("scripted-golden-compare gate exited 1", completed.stderr)
        self.assertIn("scripted unavailable/red", completed.stdout)

    def test_check_rejects_unavailable_required_floor_evidence(self):
        repo, evidence = self.create_green_repo()
        (evidence / "renderer-golden.json").unlink()

        completed = self.run_check(repo)

        self.assertEqual(completed.returncode, 1)
        self.assertIn("required renderer-golden evidence is unavailable", completed.stderr)
        self.assertIn("pixel-exact unavailable/red", completed.stdout)

    def test_check_rejects_a_failed_workspace_floor_gate(self):
        repo, evidence = self.create_green_repo()
        self.write_evidence(
            evidence / "cargo-test-workspace.json",
            "cargo-test-workspace",
            "test result: FAILED\n",
            exit_code=101,
        )

        completed = self.run_check(repo)

        self.assertEqual(completed.returncode, 1)
        self.assertIn("cargo-test-workspace gate exited 101", completed.stderr)
        self.assertIn("cargo-test-workspace RED", completed.stdout)

    def test_check_rejects_a_manifest_that_regresses_below_the_committed_floor(self):
        repo, evidence = self.create_green_repo()
        self.write_runtime_manifest(
            repo / "corpus.toml", entries=316, segments=646
        )
        for gate in ("golden-compare", "scripted-golden-compare"):
            self.write_evidence(
                evidence / f"{gate}.json",
                gate,
                golden_summary(entries=316, segments=646),
            )

        completed = self.run_check(repo)

        self.assertEqual(completed.returncode, 1)
        self.assertIn("runtime corpus entry ratchet regressed: 316 < 317", completed.stderr)
        self.assertIn(
            "runtime exact-segments ratchet regressed: 646 < 647", completed.stderr
        )
        self.assertIn("exact-segments unavailable/red", completed.stdout)

    def test_check_rejects_a_summary_below_the_manifest_ratchet(self):
        repo, evidence = self.create_green_repo()
        self.write_evidence(
            evidence / "golden-compare.json",
            "golden-compare",
            golden_summary(segments=RUNTIME_SEGMENTS - 1),
        )

        completed = self.run_check(repo)

        self.assertEqual(completed.returncode, 1)
        self.assertIn("golden-compare ratchet mismatch", completed.stderr)
        self.assertIn("exact-segments unavailable/red", completed.stdout)

    def test_check_rejects_evidence_recorded_for_another_commit(self):
        repo, evidence = self.create_green_repo()
        document = json.loads((evidence / "renderer-golden.json").read_text())
        document["source_sha"] = "old-sha"
        (evidence / "renderer-golden.json").write_text(json.dumps(document) + "\n")

        completed = self.run_check(repo)

        self.assertEqual(completed.returncode, 1)
        self.assertIn(
            "renderer-golden evidence is stale: expected test-sha, got old-sha",
            completed.stderr,
        )

    def test_check_rejects_evidence_without_a_recorded_command(self):
        repo, evidence = self.create_green_repo()
        path = evidence / "cargo-test-workspace.json"
        document = json.loads(path.read_text())
        del document["command"]
        path.write_text(json.dumps(document) + "\n")

        completed = self.run_check(repo)

        self.assertEqual(completed.returncode, 1)
        self.assertIn(
            "cargo-test-workspace evidence has no command",
            completed.stderr,
        )

    def test_check_rejects_evidence_recorded_for_a_different_command(self):
        repo, evidence = self.create_green_repo()
        path = evidence / "cargo-test-workspace.json"
        document = json.loads(path.read_text())
        document["command"] = ["cargo", "test", "-p", "nux-capi"]
        path.write_text(json.dumps(document) + "\n")

        completed = self.run_check(repo)

        self.assertEqual(completed.returncode, 1)
        self.assertIn(
            "cargo-test-workspace evidence command mismatch",
            completed.stderr,
        )

    def test_check_labels_the_current_thin_perf_result_as_non_blocking(self):
        repo, _ = self.create_green_repo()
        (repo / "target" / "perf-compare.json").write_text(
            json.dumps(
                {
                    "schema": "rive-perf-compare-json-v1",
                    "meta": {"git_sha": "test-sha"},
                    "aggregate": {"entries": 6, "rust_over_cpp": 0.9},
                }
            )
            + "\n"
        )

        completed = self.run_check(repo)

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn(
            "runtime ratio 0.900 over 6/20 files (non-blocking; #OR-9)",
            completed.stdout,
        )
        self.assertIn("| 5 Performance & size | PARTIAL |", completed.stdout)

    def test_check_reports_recorded_size_evidence_within_budget_as_green(self):
        repo, evidence = self.create_green_repo()
        self.write_evidence(
            evidence / "size-report.json", "size-report", size_summary()
        )

        completed = self.run_check(repo)

        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn(
            "size OFF 7.19 MiB / ON 7.95 MiB vs budget 8.00 MiB (both variants block)",
            completed.stdout,
        )

    def test_check_rejects_size_evidence_over_budget(self):
        repo, evidence = self.create_green_repo()
        self.write_evidence(
            evidence / "size-report.json",
            "size-report",
            size_summary(on=8388609),
            exit_code=1,
        )

        completed = self.run_check(repo)

        self.assertEqual(completed.returncode, 1)
        self.assertIn("size-report gate exited 1", completed.stderr)
        self.assertIn("size-report exceeds the #B-3 budget", completed.stderr)

    def test_check_rejects_size_evidence_whose_budget_drifted(self):
        repo, evidence = self.create_green_repo()
        self.write_evidence(
            evidence / "size-report.json",
            "size-report",
            size_summary(on=9000000, budget=9437184),
        )

        completed = self.run_check(repo)

        self.assertEqual(completed.returncode, 1)
        self.assertIn(
            "size-report budget drifted from the #B-3 decision", completed.stderr
        )

    def test_check_rejects_an_sdk_denominator_that_omits_register_rows(self):
        repo, _ = self.create_green_repo()
        definition = (repo / "parity-scorecard.toml").read_text()
        (repo / "parity-scorecard.toml").write_text(
            definition.replace('rows = ["A1"]', "rows = []")
        )

        completed = self.run_check(repo)

        self.assertEqual(completed.returncode, 1)
        self.assertIn("sdk.rows must match the register A-row checklist", completed.stderr)

    def test_check_rejects_weakened_adapter_or_performance_requirements(self):
        repo, _ = self.create_green_repo()
        definition = (repo / "parity-scorecard.toml").read_text()
        definition = definition.replace("required_adapters = 2", "required_adapters = 1")
        definition = definition.replace("blocking_min_entries = 20", "blocking_min_entries = 1")
        definition = definition.replace("max_ratio = 1.0", "max_ratio = 1.5")
        definition = definition.replace("runtime_entries = 317", "runtime_entries = 1")
        definition = definition.replace(
            "runtime_exact_segments = 647", "runtime_exact_segments = 1"
        )
        definition = definition.replace("renderer_entries = 1468", "renderer_entries = 1")
        (repo / "parity-scorecard.toml").write_text(definition)

        completed = self.run_check(repo)

        self.assertEqual(completed.returncode, 1)
        self.assertIn("platform.required_adapters must be at least 2", completed.stderr)
        self.assertIn("performance.blocking_min_entries must be at least 20", completed.stderr)
        self.assertIn("performance.max_ratio must be at most 1.0", completed.stderr)
        self.assertIn("floor.runtime_entries must be at least 317", completed.stderr)
        self.assertIn("floor.runtime_exact_segments must be at least 647", completed.stderr)
        self.assertIn("floor.renderer_entries must be at least 1468", completed.stderr)

    def test_green_floor_evidence_prints_all_five_tiers_and_writes_json(self):
        repo, _ = self.create_green_repo(sdk_rows=("A1", "A2"))
        json_output = repo / "target" / "parity-scorecard.json"

        completed = subprocess.run(
            [
                sys.executable,
                str(TOOL),
                "check",
                "--repo-root",
                str(repo),
                "--source-sha",
                "test-sha",
                "--json",
                str(json_output),
            ],
            text=True,
            capture_output=True,
        )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        for tier_name in (
            "Frame parity",
            "Interaction parity",
            "SDK parity",
            "Platform parity",
            "Performance & size",
        ):
            self.assertIn(tier_name, completed.stdout)
        self.assertIn("tiers-green: 0/5", completed.stdout)
        self.assertIn("exact-segments 647/647", completed.stdout)
        self.assertIn("pixel-exact 1468/1468", completed.stdout)
        self.assertIn("cargo-test-workspace GREEN", completed.stdout)
        self.assertIn("capi-smoke GREEN", completed.stdout)
        self.assertIn(
            "r4-timing-gate per-commit scorecard evidence not built",
            completed.stdout,
        )
        for ticket in (
            "#OR-6",
            "#OR-1/#OR-2",
            "#OR-3",
            "#OR-4",
            "#OR-5",
            "#OR-7",
            "#HD-3",
            "#OR-9",
            "#B-3",
        ):
            self.assertIn(f"not built ({ticket}", completed.stdout)
        self.assertIn("A-rows closed 0/2 (open: A1,A2)", completed.stdout)

        report = json.loads(json_output.read_text())
        self.assertEqual(report["schema"], "nuxie-parity-scorecard-v1")
        self.assertEqual(report["source_sha"], "test-sha")
        self.assertEqual(report["tiers_green"], 0)
        self.assertTrue(report["evidence_valid"])
        self.assertEqual([tier["id"] for tier in report["tiers"]], [1, 2, 3, 4, 5])
        self.assertEqual(
            [gate["state"] for gate in report["regression_floor"]],
            ["GREEN"] * 5,
        )

    @staticmethod
    def write_evidence(path, gate, output, exit_code=0):
        path.write_text(
            json.dumps(
                {
                    "schema": "nuxie-parity-gate-evidence-v1",
                    "gate": gate,
                    "source_sha": "test-sha",
                    "exit_code": exit_code,
                    "command": GATE_COMMANDS[gate],
                    "output": output,
                }
            )
            + "\n"
        )

    def create_green_repo(self, sdk_rows=("A1",)):
        temporary_directory = tempfile.TemporaryDirectory()
        self.addCleanup(temporary_directory.cleanup)
        repo = Path(temporary_directory.name)
        evidence = repo / "target" / "parity-scorecard" / "evidence"
        evidence.mkdir(parents=True)
        (repo / "docs").mkdir()
        (repo / "docs" / "parity-gap-register.md").write_text(
            "## A — Embedder API surface gaps\n\n"
            "| id | gap | tier |\n|---|---|---|\n"
            + "".join(f"| {row} | test gap | 1 |\n" for row in sdk_rows)
            + "\n"
            "## C — Coverage holes\n"
        )
        (repo / "parity-scorecard.toml").write_text(
            textwrap.dedent(
                f"""
                schema_version = 1
                [floor]
                runtime_entries = 317
                runtime_exact_segments = 647
                renderer_entries = 1468
                [sdk]
                rows = {json.dumps(list(sdk_rows))}
                closed = []
                [platform]
                verified_adapters = ["test-adapter"]
                required_adapters = 2
                [performance]
                blocking_min_entries = 20
                max_ratio = 1.0
                [size]
                budget_bytes = 8388608
                """
            ).lstrip()
        )
        self.write_runtime_manifest(repo / "corpus.toml")
        self.write_renderer_manifest(repo / "corpus-r.toml")
        self.write_evidence(
            evidence / "golden-compare.json",
            "golden-compare",
            golden_summary(),
        )
        self.write_evidence(
            evidence / "scripted-golden-compare.json",
            "scripted-golden-compare",
            golden_summary(),
        )
        self.write_evidence(
            evidence / "renderer-golden.json",
            "renderer-golden",
            renderer_summary(),
        )
        self.write_evidence(
            evidence / "cargo-test-workspace.json",
            "cargo-test-workspace",
            "test result: ok\n",
        )
        self.write_evidence(
            evidence / "capi-smoke.json", "capi-smoke", "capi smoke: ok\n"
        )
        return repo, evidence

    @staticmethod
    def write_runtime_manifest(path, entries=RUNTIME_ENTRIES, segments=RUNTIME_SEGMENTS):
        assert entries > 0 and segments >= entries
        first_samples = segments - entries + 1
        rows = []
        for index in range(entries):
            sample_count = first_samples if index == 0 else 1
            samples = ", ".join(str(float(sample)) for sample in range(sample_count))
            rows.append(
                "[[file]]\n"
                f'id = "runtime-{index}"\n'
                f'path = "runtime-{index}.riv"\n'
                f"samples = [{samples}]\n"
                'status = "exact"\n'
            )
        path.write_text("\n".join(rows))

    @staticmethod
    def write_renderer_manifest(path, entries=RENDERER_ENTRIES):
        rows = [
            "[[entry]]\n" f'id = "pixel-{index}"\n' 'status = "exact"\n'
            for index in range(entries)
        ]
        path.write_text("\n".join(rows))

    @staticmethod
    def run_check(repo):
        return subprocess.run(
            [
                sys.executable,
                str(TOOL),
                "check",
                "--repo-root",
                str(repo),
                "--source-sha",
                "test-sha",
            ],
            text=True,
            capture_output=True,
        )


if __name__ == "__main__":
    unittest.main()
