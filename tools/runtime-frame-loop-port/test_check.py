#!/usr/bin/env python3

from __future__ import annotations

import pathlib
import json
import subprocess
import tempfile
import textwrap
import unittest


TOOL = pathlib.Path(__file__).with_name("check.py")


class RuntimeFrameLoopPortCheckTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        root = pathlib.Path(self.temp.name)
        self.repo = root / "repo"
        self.upstream = root / "rive-runtime"
        (self.repo / "docs").mkdir(parents=True)
        (self.repo / "crates/runtime/src").mkdir(parents=True)
        (self.upstream / "src/animation").mkdir(parents=True)
        (self.repo / "docs/PORTING.md").write_text(
            "- **AF-1 Test adaptation.** Fixture.\n"
        )
        (self.repo / "crates/runtime/src/animation.rs").write_text(
            "struct RuntimeAnimation;\n"
        )
        (self.upstream / "src/animation/linear_animation.cpp").write_text(
            "\n".join(f"// line {value}" for value in range(1, 20)) + "\n"
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
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        self.ledger = self.repo / "docs/ownership.toml"
        self.gaps = self.repo / "docs/gaps.toml"
        self.manifest = self.repo / "file-correspondence-manifest.toml"
        self.write_files()

    def write_files(self, *, file_status: str = "pending") -> None:
        rule = '\nrule = "AF-1"' if file_status == "adapted" else ""
        self.manifest.write_text(
            textwrap.dedent(
                f"""
                upstream_ref = "{self.ref}"
                [[file]]
                upstream = "src/animation/linear_animation.cpp"
                status = "pending"
                verification = "pending-verification"
                rust_module = "crates/runtime/src/animation.rs"
                """
            ).lstrip()
        )
        self.gaps.write_text(
            textwrap.dedent(
                f"""
                version = 1
                upstream_ref = "{self.ref}"
                decision = []
                ratchet = []
                """
            ).lstrip()
        )
        (self.repo / "docs/trace.json").write_text(
            json.dumps(
                {
                    "schema": "nuxie-runtime-frame-loop-trace/v1",
                    "upstream_ref": self.ref,
                    "corpus": [
                        "advance_blend_mode",
                        "ai_assitant",
                        "align_target",
                        "animated_clipping",
                        "animation_reset_cases",
                        "spotify_kids_demo",
                    ],
                    "scope": {"static_cpp_files": 1},
                    "landmarks": {},
                    "golden_stream_operations": {"cpp": {}, "rust": {}},
                    "functions": {
                        "cpp": {"src/animation/linear_animation.cpp": []},
                        "rust": {"crates/runtime/src/animation.rs": []},
                    },
                }
            )
        )
        pending = 1 if file_status == "pending" else 0
        adapted = 1 if file_status == "adapted" else 0
        self.ledger.write_text(
            textwrap.dedent(
                f"""
                version = 1
                upstream_ref = "{self.ref}"
                porting_rules_file = "docs/PORTING.md"
                trace_evidence_file = "docs/trace.json"
                import_ledger = []
                [expected_file_status_counts]
                faithful = 0
                adapted = {adapted}
                divergent-by-decision = 0
                pending = {pending}
                compensation = 0
                [expected_member_status_counts]
                faithful = 0
                adapted = 0
                divergent-by-decision = 0
                pending = 1
                compensation = 0
                [[wave]]
                id = "FL-B"
                sequence = 1
                depends_on = []
                [[source_set]]
                id = "animation"
                wave = "FL-B"
                include = ["src/animation/*.cpp"]
                exclude = []
                rust_modules = ["crates/runtime/src/animation.rs"]
                static_closure = "Animation definitions are reached by virtual dispatch."
                [[file]]
                upstream = "src/animation/linear_animation.cpp"
                source_set = "animation"
                wave = "FL-B"
                rust_modules = ["crates/runtime/src/animation.rs"]
                dynamically_reached = true
                status = "{file_status}"{rule}
                [[member]]
                id = "animation.owner"
                wave = "FL-B"
                cpp_files = ["src/animation/linear_animation.cpp"]
                rust_file = "crates/runtime/src/animation.rs"
                rust_anchor = "RuntimeAnimation"
                status = "pending"
                """
            ).lstrip()
        )

    def run_check(self, *, closed: bool = False) -> subprocess.CompletedProcess[str]:
        command = [
            "python3",
            str(TOOL),
            "--repo-root",
            str(self.repo),
            "--rive-runtime-dir",
            str(self.upstream),
            "--ledger",
            str(self.ledger),
            "--gaps",
            str(self.gaps),
            "--file-manifest",
            str(self.manifest),
        ]
        if closed:
            command.append("--require-closed")
        return subprocess.run(command, text=True, capture_output=True, check=False)

    def test_open_atlas_passes_and_reports_counts(self) -> None:
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("files=1", result.stdout)
        self.assertIn("members=1", result.stdout)

    def test_closed_mode_rejects_pending_file_and_member(self) -> None:
        result = self.run_check(closed=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "file src/animation/linear_animation.cpp is pending", result.stderr
        )
        self.assertIn("member animation.owner is pending", result.stderr)

    def test_new_cpp_file_fails_expected_count_ratchet(self) -> None:
        (self.upstream / "src/animation/new_owner.cpp").write_text("// new\n")
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "expanded frame-loop files missing classification rows: "
            "src/animation/new_owner.cpp",
            result.stderr,
        )

    def test_overlap_fails(self) -> None:
        content = self.ledger.read_text()
        content += textwrap.dedent(
            """
            [[source_set]]
            id = "duplicate"
            wave = "FL-B"
            include = ["src/animation/linear_animation.cpp"]
            exclude = []
            rust_modules = ["crates/runtime/src/animation.rs"]
            static_closure = "Duplicate fixture."
            """
        )
        self.ledger.write_text(content)
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("assigned by both animation and duplicate", result.stderr)

    def test_adaptation_requires_binding_rule(self) -> None:
        self.write_files(file_status="adapted")
        content = self.ledger.read_text().replace('rule = "AF-1"', 'rule = "AF-999"')
        self.ledger.write_text(content)
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("cites missing PORTING.md rule AF-999", result.stderr)

    def test_dynamic_reachability_marker_must_match_trace(self) -> None:
        trace = json.loads((self.repo / "docs/trace.json").read_text())
        trace["functions"]["cpp"] = {"src/animation/other.cpp": []}
        (self.repo / "docs/trace.json").write_text(json.dumps(trace))
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "dynamically_reached=True, but trace evidence says False",
            result.stderr,
        )

    def test_closed_file_requires_orchestrator_verified_manifest(self) -> None:
        self.write_files(file_status="adapted")
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "before file correspondence is orchestrator-verified", result.stderr
        )

    def test_untracked_trace_counter_mismatch_fails(self) -> None:
        trace = json.loads((self.repo / "docs/trace.json").read_text())
        trace["landmarks"] = {"component_add_dirt": {"cpp": 1, "rust": 2}}
        (self.repo / "docs/trace.json").write_text(json.dumps(trace))
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "trace landmark mismatches have no gap rows: component_add_dirt",
            result.stderr,
        )

    def test_stream_work_mismatch_fails(self) -> None:
        trace = json.loads((self.repo / "docs/trace.json").read_text())
        trace["golden_stream_operations"] = {
            "cpp": {"drawPath": 1},
            "rust": {"drawPath": 2},
        }
        (self.repo / "docs/trace.json").write_text(json.dumps(trace))
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("golden-stream work counts differ", result.stderr)


if __name__ == "__main__":
    unittest.main()
