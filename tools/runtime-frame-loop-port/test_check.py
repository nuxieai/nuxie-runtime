#!/usr/bin/env python3

from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess
import sys
import tempfile
import textwrap
import tomllib
import unittest


TOOL = pathlib.Path(__file__).with_name("check.py")
TOOL_DIR = pathlib.Path(__file__).resolve().parent
PRODUCTION_ROOT = TOOL_DIR.parents[1]
PRODUCTION_GAPS = PRODUCTION_ROOT / "docs/runtime-frame-loop-gaps.toml"
if str(TOOL_DIR) not in sys.path:
    sys.path.insert(0, str(TOOL_DIR))

from source_fingerprint import (
    candidate_source_fingerprint,
    rust_runner_provenance,
)
from check import (
    FL_B_FROZEN_SCOPE_FILES,
    FL_B_FROZEN_SCOPE_REF,
    validate_frozen_wave_scopes,
)


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
        subprocess.run(["git", "init", "-q"], cwd=self.repo, check=True)
        (self.repo / "docs/PORTING.md").write_text(
            "- **AF-1 Test adaptation.** Fixture.\n"
            "- **FLR-3 Frame-loop binding adaptation.** Fixture.\n"
        )
        (self.repo / "docs/owner-family-closure.md").write_text(
            "`src/animation/linear_animation.cpp`\n"
            "- [x] Zero values: covered by the fixture differential.\n"
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
                    "schema": "nuxie-runtime-frame-loop-trace/v2",
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
                    "construction_landmarks": {},
                    "mechanism_landmarks": {},
                    "mechanism_construction_landmarks": {},
                    "steady_landmarks": {},
                    "mechanism_corpus": [],
                    "steady_corpus": [],
                    "mechanism_fixture_sha256": {},
                    "mechanism_input_sha256": {},
                    "golden_stream_operations": {"cpp": {}, "rust": {}},
                    "mechanism_golden_stream_operations": {
                        "cpp": {},
                        "rust": {},
                    },
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
                [active_owner_family]
                id = "fixture-animation"
                checklist = "docs/owner-family-closure.md"
                cpp_files = ["src/animation/linear_animation.cpp"]
                required_adversarial = ["Zero values"]
                [expected_trace_landmarks]
                frame = []
                construction = []
                mechanism_frame = []
                mechanism_construction = []
                steady = []
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

    def refresh_source_fingerprint(self) -> None:
        trace_path = self.repo / "docs/trace.json"
        trace = json.loads(trace_path.read_text())
        trace["rust_candidate_source"] = candidate_source_fingerprint(
            self.repo, evidence_path=trace_path
        )
        trace["rust_runner_provenance"] = rust_runner_provenance(
            trace["rust_candidate_source"]
        )
        trace_path.write_text(json.dumps(trace))

    def run_check(
        self, *, closed: bool = False, refresh_fingerprint: bool = True
    ) -> subprocess.CompletedProcess[str]:
        if refresh_fingerprint:
            self.refresh_source_fingerprint()
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

    def install_production_ratchet(self, ratchet_id: str) -> dict[str, object]:
        with PRODUCTION_GAPS.open("rb") as source:
            production = tomllib.load(source)
        rows = [
            row
            for row in production.get("ratchet", [])
            if row.get("id") == ratchet_id
        ]
        self.assertEqual(len(rows), 1, ratchet_id)
        row = rows[0]
        gaps = self.gaps.read_text()
        self.assertIn("ratchet = []", gaps)
        block = textwrap.dedent(
            f"""
            [[ratchet]]
            id = {json.dumps(row["id"])}
            globs = {json.dumps(row["globs"])}
            pattern = {json.dumps(row["pattern"])}
            {
                f"content_begin = {json.dumps(row['content_begin'])}\n"
                f"content_end = {json.dumps(row['content_end'])}\n"
                f"content_sha256 = {json.dumps(row['content_sha256'])}"
                if "content_sha256" in row
                else ""
            }
            min_occurrences = {row.get("min_occurrences", 0)}
            max_occurrences = {row["max_occurrences"]}
            """
        ).strip()
        self.gaps.write_text(gaps.replace("ratchet = []", block))
        return row

    def assert_production_ratchet_case(
        self,
        ratchet_id: str,
        relative_source: str,
        forbidden_source: str,
        safe_source: str,
    ) -> None:
        base_gaps = self.gaps.read_text()
        row = self.install_production_ratchet(ratchet_id)
        relative_path = pathlib.PurePosixPath(relative_source)
        self.assertTrue(
            any(
                relative_source == glob
                or relative_path.match(glob)
                or (
                    "/**/" in glob
                    and relative_path.match(glob.replace("/**/", "/"))
                )
                for glob in row["globs"]
            ),
            f"{relative_source} is outside {row['globs']}",
        )
        source = self.repo / relative_source
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text(forbidden_source)
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            f"ratchet {ratchet_id} increased to 1 > 0",
            result.stderr,
        )

        self.gaps.write_text(base_gaps)
        self.install_production_ratchet(ratchet_id)
        source.write_text(safe_source)
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.gaps.write_text(base_gaps)

    def assert_required_production_ratchet_case(
        self,
        ratchet_id: str,
        relative_source: str,
        required_source: str,
        missing_source: str,
    ) -> None:
        base_gaps = self.gaps.read_text()
        try:
            row = self.install_production_ratchet(ratchet_id)
            minimum = row.get("min_occurrences", 0)
            self.assertGreater(minimum, 0, ratchet_id)
            source = self.repo / relative_source
            source.parent.mkdir(parents=True, exist_ok=True)
            source.write_text(required_source)
            result = self.run_check()
            self.assertEqual(result.returncode, 0, result.stderr)

            source.write_text(missing_source)
            result = self.run_check()
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(
                f"ratchet {ratchet_id} decreased",
                result.stderr,
            )
        finally:
            self.gaps.write_text(base_gaps)

    def test_required_ratchet_rejects_missing_structural_proof(self) -> None:
        gaps = self.gaps.read_text()
        self.gaps.write_text(
            gaps.replace(
                "ratchet = []",
                textwrap.dedent(
                    """
                    [[ratchet]]
                    id = "required_shape"
                    globs = ["crates/runtime/src/animation.rs"]
                    pattern = "struct RuntimeRequiredShape"
                    min_occurrences = 1
                    max_occurrences = 1
                    """
                ).strip(),
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "ratchet required_shape decreased to 0 < 1",
            result.stderr,
        )
        (self.repo / "crates/runtime/src/animation.rs").write_text(
            "struct RuntimeAnimation;\nstruct RuntimeRequiredShape;\n"
        )
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_open_atlas_passes_and_reports_counts(self) -> None:
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("files=1", result.stdout)
        self.assertIn("members=1", result.stdout)

    def test_owner_family_checklist_missing_cpp_file_fails(self) -> None:
        checklist = self.repo / "docs/owner-family-closure.md"
        checklist.write_text("- [x] Zero values: covered.\n")
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "active owner-family checklist omits C++ file", result.stderr
        )

    def test_owner_family_missing_cpp_source_fails(self) -> None:
        (self.upstream / "src/animation/linear_animation.cpp").unlink()
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("cites missing C++ file", result.stderr)

    def test_owner_family_checklist_missing_adversarial_row_fails(self) -> None:
        checklist = self.repo / "docs/owner-family-closure.md"
        checklist.write_text("`src/animation/linear_animation.cpp`\n")
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "active owner-family checklist omits completed adversarial row",
            result.stderr,
        )

    def test_planning_owner_family_accepts_open_adversarial_rows(self) -> None:
        self.ledger.write_text(
            self.ledger.read_text().replace(
                'required_adversarial = ["Zero values"]',
                'checklist_state = "planning"\n'
                'required_adversarial = ["Zero values"]',
            )
        )
        checklist = self.repo / "docs/owner-family-closure.md"
        checklist.write_text(
            "`src/animation/linear_animation.cpp`\n"
            "- [ ] Zero values: differential required before publication.\n"
        )
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_candidate_owner_family_rejects_open_adversarial_rows(self) -> None:
        checklist = self.repo / "docs/owner-family-closure.md"
        checklist.write_text(
            "`src/animation/linear_animation.cpp`\n"
            "- [ ] Zero values: differential required before publication.\n"
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "active owner-family checklist omits completed adversarial row",
            result.stderr,
        )

    def test_stale_untracked_candidate_source_fails(self) -> None:
        self.refresh_source_fingerprint()
        (self.repo / "crates/runtime/src/new_owner.rs").write_text(
            "struct NewOwner;\n"
        )
        result = self.run_check(refresh_fingerprint=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "Rust candidate source fingerprint is stale", result.stderr
        )

    def test_stale_rust_runner_provenance_fails(self) -> None:
        self.refresh_source_fingerprint()
        trace_path = self.repo / "docs/trace.json"
        trace = json.loads(trace_path.read_text())
        trace["rust_runner_provenance"]["candidate_source"]["sha256"] = (
            "0" * 64
        )
        trace_path.write_text(json.dumps(trace))

        result = self.run_check(refresh_fingerprint=False)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "Rust runner provenance is missing or stale", result.stderr
        )

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

    def test_frozen_wave_scope_rejects_membership_drift(self) -> None:
        ledger = self.ledger.read_text()
        ledger += textwrap.dedent(
            """

            [[frozen_wave_scope]]
            wave = "FL-B"
            expected_file_count = 1
            files = [
              "src/animation/linear_animation.cpp",
            ]
            """
        )
        self.ledger.write_text(ledger)
        self.assertEqual(self.run_check().returncode, 0)

        self.ledger.write_text(
            ledger.replace(
                'files = [\n  "src/animation/linear_animation.cpp",\n]',
                'files = [\n  "src/animation/missing_owner.cpp",\n]',
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "frozen wave FL-B membership differs from expanded source scope",
            result.stderr,
        )

    def test_pinned_fl_b_scope_rejects_removal(self) -> None:
        errors: list[str] = []
        validate_frozen_wave_scopes(
            rows=[],
            assignments={},
            source_set_waves={},
            wave_ids={"FL-B"},
            upstream_ref=FL_B_FROZEN_SCOPE_REF,
            errors=errors,
        )
        self.assertIn(
            f"missing pinned frozen wave scope for FL-B at {FL_B_FROZEN_SCOPE_REF}",
            errors,
        )

    def test_pinned_fl_b_scope_rejects_coordinated_membership_drift(self) -> None:
        drift_file = "src/animation/linear_animation.cpp"
        errors: list[str] = []
        validate_frozen_wave_scopes(
            rows=[
                {
                    "wave": "FL-B",
                    "expected_file_count": 1,
                    "files": [drift_file],
                }
            ],
            assignments={drift_file: "animation"},
            source_set_waves={"animation": "FL-B"},
            wave_ids={"FL-B"},
            upstream_ref=FL_B_FROZEN_SCOPE_REF,
            errors=errors,
        )
        self.assertTrue(
            any(
                error.startswith(
                    "pinned frozen wave FL-B literal membership differs from "
                )
                for error in errors
            ),
            errors,
        )
        self.assertEqual(len(FL_B_FROZEN_SCOPE_FILES), 45)

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

    def test_frame_loop_adaptation_accepts_existing_flr_rule(self) -> None:
        self.write_files(file_status="adapted")
        content = self.ledger.read_text().replace('rule = "AF-1"', 'rule = "FLR-3"')
        self.ledger.write_text(content)
        content = self.manifest.read_text().replace(
            'status = "pending"\nverification = "pending-verification"',
            'status = "faithful"\nverification = "orchestrator-verified"',
        )
        self.manifest.write_text(content)
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_frame_loop_adaptation_rejects_missing_flr_rule(self) -> None:
        self.write_files(file_status="adapted")
        content = self.ledger.read_text().replace('rule = "AF-1"', 'rule = "FLR-999"')
        self.ledger.write_text(content)
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("cites missing PORTING.md rule FLR-999", result.stderr)

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

    def test_missing_required_trace_counter_fails(self) -> None:
        content = self.ledger.read_text().replace(
            "frame = []", 'frame = ["component_add_dirt"]', 1
        )
        self.ledger.write_text(content)
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "trace landmarks set differs; missing=['component_add_dirt']",
            result.stderr,
        )

    def test_nonzero_steady_derived_rebuild_fails(self) -> None:
        content = self.ledger.read_text().replace(
            "steady = []", 'steady = ["skin_buffer_rebuilds"]', 1
        )
        self.ledger.write_text(content)
        trace = json.loads((self.repo / "docs/trace.json").read_text())
        trace["steady_landmarks"] = {
            "skin_buffer_rebuilds": {"cpp": 0, "rust": 1}
        }
        (self.repo / "docs/trace.json").write_text(json.dumps(trace))
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "steady trace skin_buffer_rebuilds.rust must be zero", result.stderr
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

    def test_fl_b1_negative_ratchets_reject_displaced_keyframe_shapes(self) -> None:
        cases = [
            (
                "keyframe_read_time_seconds",
                r"fn\s+seconds\s*\(\s*&self\s*,\s*fps",
                "fn seconds(&self, fps: u64) -> f32 { 0.0 }\n",
            ),
            (
                "keyed_property_parallel_frame_vectors",
                r"\b(?:color_key_frames|bool_key_frames|uint_key_frames|string_key_frames|callback_key_frames)\b",
                "struct RuntimeKeyedProperty { color_key_frames: Vec<u8> }\n",
            ),
            (
                "keyed_property_family_sidecars",
                r"\b(?:double_source_value|color_source_value|bool_source_value|double_property|color_property|bool_property|uint_property|string_property|callback_event)\s*:",
                "struct RuntimeKeyedProperty { double_source_value: f32 }\n",
            ),
        ]
        base_gaps = self.gaps.read_text()
        source = self.repo / "crates/runtime/src/animation.rs"

        for ratchet_id, pattern, forbidden_source in cases:
            with self.subTest(ratchet=ratchet_id):
                self.gaps.write_text(
                    base_gaps.replace(
                        "ratchet = []",
                        textwrap.dedent(
                        f"""
                        [[ratchet]]
                        id = "{ratchet_id}"
                        globs = ["crates/runtime/src/animation.rs"]
                        pattern = {json.dumps(pattern)}
                        max_occurrences = 0
                        """
                        ).strip(),
                    )
                )
                source.write_text(forbidden_source)
                result = self.run_check()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    f"ratchet {ratchet_id} increased to 1 > 0",
                    result.stderr,
                )

    def test_fl_b2_negative_ratchets_reject_displaced_occurrence_shapes(self) -> None:
        cases = [
            (
                "linear_animation_occurrence_descriptor",
                r"\banimation\s*:\s*RuntimeLinearAnimation\b",
                "struct Instance { animation: RuntimeLinearAnimation }\n",
            ),
            (
                "linear_animation_option_loop_override",
                r"\bloop_value\s*:\s*Option\s*<\s*u64\s*>",
                "struct Instance { loop_value: Option<u64> }\n",
            ),
            (
                "linear_animation_local_zero_guard",
                r"if\s+(?:self\.fps\s*==\s*0|fps\s*==\s*0\.0|duration\s*==\s*0\.0|range\s*!=\s*0\.0)",
                "fn advance(fps: f32) { if fps == 0.0 {} }\n",
            ),
            (
                "linear_animation_definition_vec_owner",
                r"\blinear_animations\s*:\s*Vec\s*<\s*RuntimeLinearAnimation\s*>",
                "struct Artboard { linear_animations: Vec<RuntimeLinearAnimation> }\n",
            ),
        ]
        base_gaps = self.gaps.read_text()
        source = self.repo / "crates/runtime/src/animation.rs"

        for ratchet_id, pattern, forbidden_source in cases:
            with self.subTest(ratchet=ratchet_id):
                self.gaps.write_text(
                    base_gaps.replace(
                        "ratchet = []",
                        textwrap.dedent(
                        f"""
                        [[ratchet]]
                        id = "{ratchet_id}"
                        globs = ["crates/runtime/src/animation.rs"]
                        pattern = {json.dumps(pattern)}
                        max_occurrences = 0
                        """
                        ).strip(),
                    )
                )
                source.write_text(forbidden_source)
                result = self.run_check()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    f"ratchet {ratchet_id} increased to 1 > 0",
                    result.stderr,
                )

    def test_fl_b3_negative_ratchets_reject_displaced_reset_shapes(self) -> None:
        cases = [
            (
                "animation_reset_flat_entries_owner",
                r"struct\s+AnimationReset\s*\{\s*entries\s*:\s*Vec",
                "struct AnimationReset { entries: Vec<Entry> }\n",
            ),
            (
                "animation_reset_global_seen_scan",
                r"let\s+mut\s+seen\s*=\s*Vec",
                "fn build() { let mut seen = Vec::new(); }\n",
            ),
            (
                "animation_reset_empty_owner_elision",
                r"if\s+entries\.is_empty\(\)\s*\{\s*None",
                "fn build(entries: Vec<u8>) { if entries.is_empty() { None } }\n",
            ),
        ]
        base_gaps = self.gaps.read_text()
        source = self.repo / "crates/runtime/src/animation.rs"

        for ratchet_id, pattern, forbidden_source in cases:
            with self.subTest(ratchet=ratchet_id):
                self.gaps.write_text(
                    base_gaps.replace(
                        "ratchet = []",
                        textwrap.dedent(
                        f"""
                        [[ratchet]]
                        id = "{ratchet_id}"
                        globs = ["crates/runtime/src/animation.rs"]
                        pattern = {json.dumps(pattern)}
                        max_occurrences = 0
                        """
                        ).strip(),
                    )
                )
                source.write_text(forbidden_source)
                result = self.run_check()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    f"ratchet {ratchet_id} increased to 1 > 0",
                    result.stderr,
                )

    def test_fl_b4_negative_ratchets_reject_displaced_blend_shapes(self) -> None:
        cases = [
            (
                "blend_occurrence_1d_value_copy",
                r"value\s*:\s*animation\.value",
                "fn build() { let _ = Occurrence { value: animation.value }; }\n",
            ),
            (
                "blend_occurrence_direct_source_copy",
                r"source\s*:\s*animation\.source\.clone",
                "fn build() { let _ = Occurrence { source: animation.source.clone() }; }\n",
            ),
            (
                "blend_occurrence_state_source_copy",
                r"source\s*:\s*blend_state\.source\.clone",
                "fn build() { let _ = Occurrence { source: blend_state.source.clone() }; }\n",
            ),
            (
                "blend_invalid_definition_elision",
                r"\(!animations\.is_empty\(\)\)\.then_some",
                "fn build() { (!animations.is_empty()).then_some(State { animations }); }\n",
            ),
            (
                "blend_from_to_index_rediscovery",
                r"let\s+from_index\s*=\s*to_index\.checked_sub",
                "fn advance() { let from_index = to_index.checked_sub(1); }\n",
            ),
        ]
        base_gaps = self.gaps.read_text()
        source = self.repo / "crates/runtime/src/state_machine.rs"

        for ratchet_id, pattern, forbidden_source in cases:
            with self.subTest(ratchet=ratchet_id):
                self.gaps.write_text(
                    base_gaps.replace(
                        "ratchet = []",
                        textwrap.dedent(
                        f"""
                        [[ratchet]]
                        id = "{ratchet_id}"
                        globs = ["crates/runtime/src/state_machine.rs"]
                        pattern = {json.dumps(pattern)}
                        max_occurrences = 0
                        """
                        ).strip(),
                    )
                )
                source.write_text(forbidden_source)
                result = self.run_check()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    f"ratchet {ratchet_id} increased to 1 > 0",
                    result.stderr,
                )

    def test_fl_c_negative_ratchets_reject_displaced_state_machine_shapes(self) -> None:
        cases = [
            (
                "state_machine_per_advance_collection_rebuild",
                (
                    r"fn\s+advance[^\{]*\{"
                    r"(?:(?!\n\s*(?:pub(?:\([^)]*\))?\s+)?fn\s)[\s\S])"
                    r"{0,8000}?let\s+(?:mut\s+)?"
                    r"(?:inputs|layers|transitions|conditions|listeners|actions)"
                    r"\s*=\s*[^;]*\.collect"
                ),
                "fn advance() { let transitions = source.iter().collect(); }\n",
                0,
            ),
            (
                "state_machine_occurrence_definition_copy",
                r"(?:input|layer|transition|condition|listener|action)_definition\s*:\s*[^,\n]*\.clone\(\)",
                "fn instance() { let _ = Slot { transition_definition: source.clone() }; }\n",
                0,
            ),
            (
                "state_machine_replacement_candidate_container",
                r"(?:let\s+(?:mut\s+)?|pub(?:\([^)]*\))?\s+)?(?:transition|condition|listener|action)_candidates\s*(?::\s*(?:Vec|BTreeMap|HashMap)|=\s*(?:Vec|BTreeMap|HashMap)::new)",
                "fn search() { let mut transition_candidates = Vec::new(); }\n",
                0,
            ),
            (
                "state_machine_event_listener_rescan",
                r"(?:events|listeners)\.iter\(\)\s*\.(?:find|find_map|position|any)",
                (
                    "fn deliver() { let _ = listeners.iter().find(|listener| listener.ready); }\n"
                    "fn redeliver() { let _ = events.iter().any(|event| event.ready); }\n"
                ),
                1,
            ),
            (
                "state_machine_transition_search_early_exit",
                r"if\s+(?:transition|conditions?)\.[^\n]*\{\s*return\s+(?:Ok\(false\)|None|false)",
                "fn allow() -> bool { if conditions.is_empty() { return false } true }\n",
                0,
            ),
            (
                "state_machine_invented_transition_return_guard",
                r"(?:condition_count\s*==\s*self\.conditions\.len\(\)|duration\s*!=\s*0\.0)",
                "fn allow(duration: f32) -> bool { duration != 0.0 }\n",
                0,
            ),
            (
                "state_machine_dropped_viewmodel_condition",
                r"RuntimeTransitionViewModelCondition::from_object\([^;]{0,240}\.map\(Self::ViewModel\)",
                (
                    "fn import() { let condition = "
                    "RuntimeTransitionViewModelCondition::from_object(file, graph, object)"
                    ".map(Self::ViewModel); }\n"
                ),
                0,
            ),
            (
                "state_machine_dropped_focus_condition",
                r"RuntimeTransitionFocusCondition::from_object\([^;]{0,240}\.map\(Self::Focus\)",
                (
                    "fn import() { let condition = "
                    "RuntimeTransitionFocusCondition::from_object(file, object)"
                    ".map(Self::Focus); }\n"
                ),
                0,
            ),
        ]
        base_gaps = self.gaps.read_text()
        source = self.repo / "crates/runtime/src/state_machine/instance.rs"
        source.parent.mkdir(parents=True, exist_ok=True)

        for ratchet_id, pattern, forbidden_source, maximum in cases:
            with self.subTest(ratchet=ratchet_id):
                self.gaps.write_text(
                    base_gaps.replace(
                        "ratchet = []",
                        textwrap.dedent(
                        f"""
                        [[ratchet]]
                        id = "{ratchet_id}"
                        globs = ["crates/runtime/src/state_machine/instance.rs"]
                        pattern = {json.dumps(pattern)}
                        max_occurrences = {maximum}
                        """
                        ).strip(),
                    )
                )
                source.write_text(forbidden_source)
                result = self.run_check()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    f"ratchet {ratchet_id} increased to {maximum + 1} > {maximum}",
                    result.stderr,
                )

    def test_duplicate_ratchet_ids_are_rejected(self) -> None:
        base_gaps = self.gaps.read_text()
        duplicate_rows = textwrap.dedent(
            """
            [[ratchet]]
            id = "duplicate"
            globs = ["crates/runtime/src/animation.rs"]
            pattern = "first"
            max_occurrences = 0

            [[ratchet]]
            id = "duplicate"
            globs = ["crates/runtime/src/animation.rs"]
            pattern = "second"
            max_occurrences = 0
            """
        ).strip()
        self.gaps.write_text(base_gaps.replace("ratchet = []", duplicate_rows))

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("duplicate ratchet ids: duplicate", result.stderr)

    def test_fl_c3_negative_ratchets_reject_displaced_layer_occurrence_shapes(self) -> None:
        cases = [
            (
                "state_machine_empty_any_shortcut",
                r"any_state[^\n]{0,160}transitions\.is_empty\(\)",
                "fn update(any_state: State) { if any_state.transitions.is_empty() {} }\n",
            ),
            (
                "state_machine_parallel_occurrence_indices",
                r"(?:current|source)_state_index\s*:",
                "struct Layer { current_state_index: usize }\n",
            ),
            (
                "state_machine_copied_active_transition_payload",
                r"active_transition_(?:actions|duration|interpolator|listener)",
                "struct Layer { active_transition_duration: f32 }\n",
            ),
            (
                "state_machine_prebuilt_state_occurrences",
                r"state_occurrences\s*:\s*(?:Vec|BTreeMap|HashMap)",
                "struct Layer { state_occurrences: Vec<State> }\n",
            ),
            (
                "state_machine_nested_owner_scan",
                (
                    r"fn\s+from_imported[\s\S]{0,3000}"
                    r"nested_artboards[^;]{0,400}find_map[^;]{0,400}"
                    r"RuntimeNestedAnimationInstance::StateMachine"
                ),
                (
                    "fn from_imported() { nested_artboards.iter().find_map(|nested| "
                    "matches!(nested, RuntimeNestedAnimationInstance::StateMachine(_)).then_some(nested)); }\n"
                ),
            ),
            (
                "state_machine_shared_layer_occurrence_alias",
                r"(?:any_state|current_state|state_from)\s*:\s*Option<(?:Rc|Arc)<",
                "struct Layer { state_from: Option<Rc<State>> }\n",
            ),
            (
                "state_machine_advance_source_before_mix",
                r"advance_transition_source_animation[\s\S]{0,1200}update_transition_mix",
                (
                    "fn advance() { self.advance_transition_source_animation(); "
                    "self.update_transition_mix(); }\n"
                ),
            ),
            (
                "nested_state_machine_public_clone_copies_live_occurrence",
                (
                    r"impl Clone for RuntimeNestedArtboardInstance"
                    r"[\s\S]{0,2400}animations:\s*self\.animations\.clone\(\)"
                ),
                (
                    "impl Clone for RuntimeNestedArtboardInstance { fn clone(&self) -> Self { "
                    "Self { animations: self.animations.clone() } } }\n"
                ),
            ),
            (
                "state_machine_transition_random_constant",
                (
                    r"(?:(?:random_weight|random_value)\s*=\s*"
                    r"0(?:\.0)?(?:_f(?:32|64))?"
                    r"|fn\s+(?:random_transition_value|random_value)"
                    r"\s*\([^)]*\)\s*->\s*f(?:32|64)\s*"
                    r"\{\s*0(?:\.0)?(?:_f(?:32|64))?\s*\})"
                ),
                "fn random_transition_value() -> f64 { 0.0 }\n",
            ),
            (
                "state_machine_transition_weight_widened_or_saturating",
                r"(?:total_weight|evaluated_random_weight)[^\n]{0,120}(?:u64|saturating_add)",
                "fn pick() { let total_weight: u64 = 0; }\n",
            ),
            (
                "state_machine_transition_candidate_reorder",
                (
                    r"weighted_candidates\."
                    r"(?:sort|sort_by|sort_by_key|sort_unstable|sort_unstable_by|sort_unstable_by_key)"
                ),
                "fn pick(weighted_candidates: &mut Vec<u32>) { weighted_candidates.sort(); }\n",
            ),
            (
                "state_machine_occurrence_owned_random_provider",
                r"random_provider\s*:\s*RuntimeRandomProvider",
                "struct StateMachineInstance { random_provider: RuntimeRandomProvider }\n",
            ),
            (
                "state_machine_random_time_only_seed",
                (
                    r"fn\s+initialize_layer\(\)"
                    r"[\s\S]{0,240}let\s+seed\s*=\s*SystemTime"
                ),
                (
                    "fn initialize_layer() { "
                    "let seed = SystemTime::now().elapsed().unwrap().as_nanos(); }\n"
                ),
            ),
            (
                "state_machine_random_unconditional_libc",
                r"libc::(?:rand|srand|RAND_MAX)",
                "fn draw() -> f32 { unsafe { libc::rand() as f32 } }\n",
            ),
            (
                "state_machine_random_native_rust_wall_clock_api",
                r"(?:SystemTime|UNIX_EPOCH)",
                "fn seed() { let _ = SystemTime::now(); }\n",
            ),
            (
                "state_machine_random_linux_wrong_high_resolution_clock",
                (
                    r"LINUX_HIGH_RESOLUTION_CLOCK_ID"
                    r"[^\n]{0,120}=\s*libc::CLOCK_MONOTONIC"
                ),
                (
                    "const LINUX_HIGH_RESOLUTION_CLOCK_ID: libc::clockid_t "
                    "= libc::CLOCK_MONOTONIC;\n"
                ),
            ),
            (
                "state_machine_random_apple_wrong_high_resolution_clock",
                (
                    r"APPLE_HIGH_RESOLUTION_CLOCK_ID"
                    r"[^\n]{0,120}=\s*libc::"
                    r"(?:CLOCK_REALTIME|CLOCK_MONOTONIC)(?:[^_]|$)"
                ),
                (
                    "const APPLE_HIGH_RESOLUTION_CLOCK_ID: libc::clockid_t "
                    "= libc::CLOCK_REALTIME;\n"
                ),
            ),
            (
                "state_machine_random_other_unix_wrong_high_resolution_clock",
                (
                    r"OTHER_UNIX_HIGH_RESOLUTION_CLOCK_ID"
                    r"[^\n]{0,120}=\s*libc::"
                    r"(?:CLOCK_REALTIME|CLOCK_MONOTONIC_RAW)"
                ),
                (
                    "const OTHER_UNIX_HIGH_RESOLUTION_CLOCK_ID: libc::clockid_t "
                    "= libc::CLOCK_MONOTONIC_RAW;\n"
                ),
            ),
            (
                "state_machine_random_windows_wrong_high_resolution_clock",
                (
                    r"(?:GetSystemTime|GetSystemTimePreciseAsFileTime"
                    r"|GetTickCount|timeGetTime)"
                ),
                "fn seed() { let _ = GetTickCount(); }\n",
            ),
            (
                "state_machine_random_native_panic",
                r"(?:panic!|unwrap\(|expect\()",
                "fn seed() { panic!(\"clock failed\"); }\n",
            ),
            (
                "nested_state_machine_optional_owner_construction",
                (
                    r"(?:fn\s+(?:nested_state_machine_instance|from_imported)"
                    r"[\s\S]{0,360}->\s*Option<(?:RuntimeNestedAnimationInstance|Self)>"
                    r"|let\s+Some\([^)]*\)\s*=\s*nested_state_machine_instance)"
                ),
                (
                    "fn nested_state_machine_instance() "
                    "-> Option<RuntimeNestedAnimationInstance> { None }\n"
                ),
            ),
            (
                "nested_state_machine_missing_input_name_drop",
                (
                    r"fn\s+input_id_named[\s\S]{0,700}"
                    r"(?:\.and_then\(\|input\|\s*input\.name\(\)\)"
                    r"[\s\S]{0,80}==\s*Some\(name\)"
                    r"|input\.name\.as_deref\(\)\s*==\s*Some\(name\))"
                ),
                (
                    "fn input_id_named(name: &str) { "
                    "input.name.as_deref() == Some(name); }\n"
                ),
            ),
            (
                "state_machine_all_layers_before_entry_actions",
                (
                    r"(?:let\s+layers\s*=\s*state_machine[\s\S]{0,700}\.collect\(\)"
                    r"|initialize_layer_entry_actions)"
                ),
                "fn initialize_layer_entry_actions() {}\n",
            ),
            (
                "state_machine_layer_init_error_drops_later_layers",
                (
                    r"fn\s+initialize_layers_in_authored_order"
                    r"[\s\S]{0,3200}(?:if\s+let\s+Err\([^)]*\)"
                    r"[\s\S]{0,120}\bbreak\s*;"
                    r"|if\s+self\.script_error\.is_some\(\)\s*"
                    r"\{\s*(?:continue|break)\s*;)"
                ),
                (
                    "fn initialize_layers_in_authored_order() { "
                    "if self.script_error.is_some() { continue; } }\n"
                ),
            ),
            (
                "state_machine_random_waiting_for_exit_overwrite",
                (
                    r"fn\s+find_random_transition[\s\S]{0,3600}"
                    r"self\.waiting_for_exit\s*=\s*(?:false|waiting_for_exit)"
                ),
                (
                    "fn find_random_transition() { "
                    "self.waiting_for_exit = waiting_for_exit; }\n"
                ),
            ),
            (
                "state_machine_random_selected_wait_latch_not_cleared",
                (
                    r"if\s+state\.uses_random_transition_selection\(\)"
                    r"(?:(?!\n\s*for\s*\()[\s\S]){0,5000}"
                    r"self\.change_state\("
                    r"[\s\S]{0,1200}\)\?;\s*return\s+Ok\(true\)"
                ),
                (
                    "fn update(state: State) { "
                    "if state.uses_random_transition_selection() { "
                    "self.change_state()?; return Ok(true); } }\n"
                ),
            ),
            (
                "state_machine_entry_databinds_available_too_early",
                (
                    r"fn\s+initialize_layers_in_authored_order"
                    r"[\s\S]{0,4200}data_bind_facilities_ready:\s*true"
                    r"[\s\S]{0,1000}perform_initial_entry_actions"
                ),
                (
                    "fn initialize_layers_in_authored_order() { "
                    "let executor = RuntimeStateMachineListenerActionExecutor { "
                    "data_bind_facilities_ready: true }; "
                    "perform_initial_entry_actions(executor); }\n"
                ),
            ),
            (
                "state_machine_focus_tree_built_before_layer_entries",
                (
                    r"pub\(crate\)\s+fn\s+new\(\s*state_machine_index"
                    r"(?:(?!self\s*\.initialize_layers_in_authored_order)"
                    r"[\s\S]){0,16000}"
                    r"(?:RuntimeFocusTree::from_artboard"
                    r"|focus\.sync\(artboard\)"
                    r"|install_nested_external_focus_domain\([^)]*\))"
                    r"[\s\S]{0,6000}"
                    r"self\s*\.initialize_layers_in_authored_order"
                ),
                (
                    "pub(crate) fn new(state_machine_index: usize, "
                    "artboard: &mut ArtboardInstance) { "
                    "let focus = RuntimeFocusTree::from_artboard(artboard); "
                    "self.initialize_layers_in_authored_order(); }\n"
                ),
            ),
        ]
        base_gaps = self.gaps.read_text()

        for ratchet_id, pattern, forbidden_source in cases:
            with self.subTest(ratchet=ratchet_id):
                self.gaps.write_text(
                    base_gaps.replace(
                        "ratchet = []",
                        textwrap.dedent(
                            f"""
                            [[ratchet]]
                            id = "{ratchet_id}"
                            globs = ["crates/runtime/src/state_machine/instance.rs"]
                            pattern = {json.dumps(pattern)}
                            max_occurrences = 0
                            """
                        ).strip(),
                    )
                )
                source = self.repo / "crates/runtime/src/state_machine/instance.rs"
                source.parent.mkdir(parents=True, exist_ok=True)
                source.write_text(forbidden_source)
                result = self.run_check()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    f"ratchet {ratchet_id} increased to 1 > 0",
                    result.stderr,
                )

    def test_fl_c4_live_ratchets_reject_forbidden_and_stop_at_function_boundaries(
        self,
    ) -> None:
        cases = [
            (
                "listener_input_direct_before_nested",
                "crates/nuxie-runtime/src/state_machine/listener_bool_change.rs",
                "impl X {\n    pub(crate) fn perform(&self) {\n        let _ = target.direct_input_index;\n        let _ = target.nested_input_local_id;\n    }\n    fn next(&self) {}\n}\n",
                "impl X {\n    pub(crate) fn perform(&self) {\n        let _ = target.nested_input_local_id;\n        let _ = target.direct_input_index;\n    }\n    pub(crate) fn next(&self) {\n        let _ = target.nested_input_local_id;\n    }\n}\n",
            ),
            (
                "script_input_artboard_live_source_overwrites_generated_id",
                "crates/nuxie-runtime/src/scripted_object.rs",
                "impl X {\n    pub(crate) fn apply_artboard_source(&mut self, id: u64) {\n        self.value = Some(id);\n    }\n    pub(crate) fn next(&self) {}\n}\n",
                "impl X {\n    pub(crate) fn apply_artboard_source(&mut self) {\n        self.artboard = None;\n    }\n    pub(crate) fn update_generated(&mut self) {\n        self.value = None;\n    }\n}\n",
            ),
            (
                "script_input_artboard_projection_reads_generated_id",
                "crates/nuxie-runtime/src/scripted_object.rs",
                "impl X {\n    pub(crate) fn projection_value(&self, kind: ScriptListenerInputKind) {\n        if kind == ScriptListenerInputKind::Artboard {\n            return self.value.clone();\n        }\n        None\n    }\n    pub(crate) fn next(&self) {}\n}\n",
                "impl X {\n    pub(crate) fn projection_value(&self, kind: ScriptListenerInputKind) {\n        if kind == ScriptListenerInputKind::Artboard {\n            return self.artboard.value();\n        }\n        self.value.clone()\n    }\n    pub(crate) fn next(&self) {}\n}\n",
            ),
            (
                "script_input_artboard_clone_blindly_copies_reference_state",
                "crates/nuxie-runtime/src/scripted_object.rs",
                "impl X {\n    pub(crate) fn clone_for_scripted_object(&self) -> Self {\n        Self { artboard: self.artboard.clone() }\n    }\n    pub(crate) fn next(&self) {}\n}\n",
                "impl X {\n    pub(crate) fn clone_for_scripted_object(&self) -> Self {\n        Self { artboard: self.artboard.clone_for_scripted_object() }\n    }\n    pub(crate) fn next(&self) {\n        let _ = self.artboard.clone();\n    }\n}\n",
            ),
            (
                "script_input_artboard_fresh_clone_uses_derived_state",
                "crates/nuxie-runtime/src/state_machine/scripted_listener_action.rs",
                "impl X {\n    fn from_definition(definition: &D) -> Self {\n        Self { properties: definition.properties.clone() }\n    }\n    fn next(&self) {}\n}\n",
                "impl X {\n    fn from_definition(definition: &D) -> Self {\n        Self { properties: definition.properties.clone_for_scripted_object() }\n    }\n    fn next(input: &D) {\n        let _ = input.properties.clone();\n    }\n}\n",
            ),
            (
                "data_converter_group_forward_output_type_walk",
                "crates/nuxie-runtime/src/data_bind_graph.rs",
                "impl C {\n    pub(crate) fn cpp_output_data_type(&self) -> T {\n        match self {\n            Self::Group(converters) => converters.iter().find(ok).unwrap(),\n            _ => T::None,\n        }\n    }\n    fn next(&self) {}\n}\n",
                "impl C {\n    pub(crate) fn cpp_output_data_type(&self) -> T {\n        match self {\n            Self::Group(converters) => converters.iter().rev().find(ok).unwrap(),\n            _ => T::None,\n        }\n    }\n    fn next(&self) {\n        if let Self::Group(converters) = self { let _ = converters.iter(); }\n    }\n}\n",
            ),
            (
                "data_converter_group_output_capability_union",
                "crates/nuxie-runtime/src/data_bind_graph.rs",
                "impl C {\n    pub(crate) fn can_change_output_kind(&self) -> bool {\n        match self {\n            Self::Group(converters) => converters.iter().any(Self::can_change_output_kind),\n            _ => false,\n        }\n    }\n    fn next(&self) {}\n}\n",
                "impl C {\n    pub(crate) fn can_change_output_kind(&self) -> bool {\n        self.cpp_output_kind() == Kind::Deferred\n    }\n    pub(crate) fn next(&self) {\n        if let Self::Group(converters) = self { let _ = converters.iter().any(ok); }\n    }\n}\n",
            ),
            (
                "scripted_object_partial_hydration_during_validation",
                "crates/nuxie/src/lib.rs",
                "fn prepare_script_listener_hydrations(definitions: &[D], instance: &mut I) {\n    for definition in definitions {\n        instance.set_input_core(name, value);\n    }\n}\nfn next() {}\n",
                "fn prepare_script_listener_hydrations(definitions: &[D]) {\n    let _ = definitions.iter().collect::<Vec<_>>();\n}\nfn hydrate(instance: &mut I) {\n    instance.set_input_core(name, value);\n}\n",
            ),
            (
                "scripted_object_split_hydration_entrypoint",
                "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs",
                "impl X {\n    pub fn hydrate_scripted_listener_action_instance(&mut self) {}\n    pub fn next(&self) {}\n}\n",
                "impl X {\n    pub fn hydrate_and_initialize_scripted_listener_action_instance(&mut self) {}\n    pub fn next(&self) {}\n}\n",
            ),
            (
                "scripted_object_batch_hydration_collection",
                "crates/nuxie/src/lib.rs",
                "fn prepare_script_listener_data_converter_hydrations() -> Vec<Hydration> {\n    Vec::new()\n}\nfn next() {}\n",
                "fn prepare_script_listener_data_converter_hydration() -> Hydration {\n    Hydration\n}\nfn next() {}\n",
            ),
            (
                "scripted_object_mutable_hydration_preflight",
                "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs",
                "impl X {\n    pub fn hydrate_and_initialize_scripted_listener_action_instance<F>(&mut self, prepare: F)\n    where\n        F: FnOnce(&mut Self),\n    {\n        prepare(self);\n    }\n    pub fn next(&self) {}\n}\n",
                "impl X {\n    pub fn hydrate_and_initialize_scripted_listener_action_instance<F>(&mut self, prepare: F)\n    where\n        F: FnOnce(&Self),\n    {\n        prepare(self);\n    }\n    pub fn next(&self) {}\n}\n",
            ),
            (
                "scripted_object_live_context_barrier_runs_init",
                "crates/nuxie/src/lib.rs",
                "fn install_live_scripted_object_contexts() {\n    machine.install_scripted_object_data_context(id, &context);\n    machine.hydrate_and_initialize_scripted_object_instance(id);\n}\nfn next() {}\n",
                "fn install_live_scripted_object_contexts() {\n    machine.install_scripted_object_data_context(id, &context);\n}\nfn next() {\n    machine.hydrate_and_initialize_scripted_object_instance(id);\n}\n",
            ),
            (
                "scripted_object_unbound_constructor_enters_live_context",
                "crates/nuxie/src/lib.rs",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
            ),
            (
                "scripted_object_unbound_constructor_enters_live_context",
                "crates/nuxie/src/lib.rs",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    // if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if !machine.has_scripted_listener_data_context() {\n        return Ok(());\n    }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
            ),
            (
                "scripted_object_unbound_constructor_enters_live_context",
                "crates/nuxie/src/lib.rs",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    /*\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    */\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
            ),
            (
                "scripted_object_unbound_constructor_enters_live_context",
                "crates/nuxie/src/lib.rs",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if feature_enabled {\n        if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
            ),
            (
                "scripted_object_unbound_constructor_enters_live_context",
                "crates/nuxie/src/lib.rs",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    while false {\n        if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
            ),
            (
                "scripted_object_unbound_constructor_enters_live_context",
                "crates/nuxie/src/lib.rs",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    if feature_enabled {\n        retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    }\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
            ),
            (
                "scripted_object_unbound_constructor_enters_live_context",
                "crates/nuxie/src/lib.rs",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if feature_enabled {\n        instantiate_state_machine_data_converters(file, machine);\n    }\n}\nfn next() {}\n",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
            ),
            (
                "scripted_object_unbound_constructor_enters_live_context",
                "crates/nuxie/src/lib.rs",
                "unsafe fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
                "unsafe fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
            ),
            (
                "scripted_object_unbound_constructor_enters_live_context",
                "crates/nuxie/src/lib.rs",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    ignore_tokens!(if !machine.has_scripted_listener_data_context() { return Ok(()); });\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
            ),
            (
                "scripted_object_unbound_constructor_enters_live_context",
                "crates/nuxie/src/lib.rs",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    #[cfg(any())]\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
            ),
            (
                "scripted_object_unbound_constructor_enters_live_context",
                "crates/nuxie/src/lib.rs",
                "#[cfg(feature = \"a\")]\nfn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\n#[cfg(not(feature = \"a\"))]\nfn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    instantiate_state_machine_data_converters(file, machine);\n}\n",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
            ),
            (
                "scripted_object_unbound_constructor_enters_live_context",
                "crates/nuxie/src/lib.rs",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    let _ = machine.has_scripted_listener_data_context();\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\npub fn next() {\n    instantiate_state_machine_data_converters(file, machine);\n}\n",
            ),
            (
                "scripted_object_unbound_constructor_enters_live_context",
                "crates/nuxie/src/lib.rs",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
                "fn instantiate_script_listener_actions_with_optional_factory() {\n    retry_cold_scripted_objects_during_constructor(file, machine, definitions, factory);\n    if !machine.has_scripted_listener_data_context() { return Ok(()); }\n    instantiate_state_machine_data_converters(file, machine);\n}\nfn next() {}\n",
            ),
            (
                "scripted_object_instance_map_drops_before_cloned_binds",
                "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs",
                "pub struct StateMachineInstance {\n    scripted_instances_by_global: BTreeMap<u32, Handle>,\n    focus: Focus,\n    scripted_object_bindings: Vec<Binding>,\n}\n",
                "pub struct StateMachineInstance {\n    scripted_object_bindings: Vec<Binding>,\n    scripted_instances_by_global: BTreeMap<u32, Handle>,\n}\n",
            ),
            (
                "scripted_converter_unstable_authored_order",
                "crates/nuxie-runtime/src/scripted_data_converter.rs",
                "impl X {\n    pub(crate) fn from_definition(values: &mut [Value]) {\n        values.sort_by_key(Value::authored_order);\n    }\n    pub(crate) fn next() {}\n}\n",
                "impl X {\n    pub(crate) fn from_definition(values: &[Value]) {\n        for value in values { value.retain(); }\n    }\n    pub(crate) fn next() {}\n}\n",
            ),
        ]
        for ratchet_id, source, forbidden, safe in cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_production_ratchet_case(
                    ratchet_id,
                    source,
                    forbidden,
                    safe,
                )

    def test_fl_c4_data_bind_converter_live_ratchets_reject_forbidden_and_stop_at_function_boundaries(
        self,
    ) -> None:
        cases = [
            (
                "state_machine_data_bind_template_drop_or_reorder",
                "crates/nuxie-runtime/src/state_machine/data_bind_template.rs",
                (
                    "pub(super) fn runtime_state_machine_data_bind_templates() {\n"
                    "    state_machine.data_binds.iter().filter_map(build).collect();\n"
                    "}\n"
                    "fn next() {}\n"
                ),
                (
                    "pub(super) fn runtime_state_machine_data_bind_templates() {\n"
                    "    state_machine.data_binds.iter().map(build).collect();\n"
                    "}\n"
                    "fn next() { let _ = values.iter().filter_map(build); }\n"
                ),
            ),
            (
                "state_machine_data_bind_container_enrollment_drop_or_reorder",
                "crates/nuxie-runtime/src/data_bind_graph.rs",
                (
                    "impl X {\n"
                    "    pub(crate) fn add_data_binds_to_container(&mut self) {\n"
                    "        self.bindings.iter().filter_map(enroll).collect();\n"
                    "    }\n"
                    "    fn next(&self) {}\n"
                    "}\n"
                ),
                (
                    "impl X {\n"
                    "    pub(crate) fn add_data_binds_to_container(&mut self) {\n"
                    "        self.bindings.iter().map(enroll).collect();\n"
                    "    }\n"
                    "    pub(crate) fn next(&self) {\n"
                    "        let _ = self.bindings.iter().filter_map(enroll);\n"
                    "    }\n"
                    "}\n"
                ),
            ),
            (
                "state_machine_base_data_bind_requires_context_path",
                "crates/nuxie-runtime/src/state_machine/data_bind_template.rs",
                (
                    "pub(super) fn runtime_state_machine_data_bind_templates() {\n"
                    "    let Some(path) = "
                    "file.data_bind_context_source_path_ids_for_object(data_bind) "
                    "else { return None; };\n"
                    "}\n"
                    "fn next() {}\n"
                ),
                (
                    "pub(super) fn runtime_state_machine_data_bind_templates() {\n"
                    "    let path = "
                    "file.data_bind_context_source_path_ids_for_object(data_bind)"
                    ".unwrap_or_default();\n"
                    "}\n"
                    "fn next() {\n"
                    "    let Some(path) = "
                    "file.data_bind_context_source_path_ids_for_object(data_bind) "
                    "else { return; };\n"
                    "}\n"
                ),
            ),
            (
                "data_bind_per_type_target_to_source_path",
                "crates/nuxie-runtime/src/data_bind_graph.rs",
                (
                    "fn apply_default_view_model_number_targets_to_sources() {}\n"
                ),
                "fn apply_default_view_model_targets_to_sources() {}\n",
            ),
            (
                "data_bind_target_dirt_default_context_only",
                "crates/nuxie-runtime/src/data_bind_graph.rs",
                (
                    "impl X {\n"
                    "    pub(crate) fn mark_target_dirty_for_data_bind(&mut self) {\n"
                    "        if self.default_view_model_source_context_bound() {\n"
                    "            self.mark_target();\n"
                    "        }\n"
                    "    }\n"
                    "    fn next(&self) {}\n"
                    "}\n"
                ),
                (
                    "impl X {\n"
                    "    pub(crate) fn mark_target_dirty_for_data_bind(&mut self) {\n"
                    "        self.nodes.mark_target();\n"
                    "    }\n"
                    "    pub(crate) fn next(&self) {\n"
                    "        let _ = self.default_view_model_source_context_bound();\n"
                    "    }\n"
                    "}\n"
                ),
            ),
            (
                "data_bind_occurrence_target_apply_before_dirt_clear",
                "crates/nuxie-runtime/src/data_bind_graph.rs",
                (
                    "impl X {\n"
                    "    pub(crate) fn update_default_view_model_binding(&mut self) {\n"
                    "        self.targets.apply_default_view_model_binding();\n"
                    "        self.clear_retained_data_bind_occurrence_dirt();\n"
                    "    }\n"
                    "    fn next(&self) {}\n"
                    "}\n"
                ),
                (
                    "impl X {\n"
                    "    pub(crate) fn update_default_view_model_binding(&mut self) {\n"
                    "        self.clear_retained_data_bind_occurrence_dirt();\n"
                    "        self.targets.apply_default_view_model_binding();\n"
                    "    }\n"
                    "    pub(crate) fn next(&self) {}\n"
                    "}\n"
                ),
            ),
            (
                "data_bind_occurrence_dirt_clear_incomplete",
                "crates/nuxie-runtime/src/data_bind_graph.rs",
                (
                    "impl X {\n"
                    "    fn clear_retained_data_bind_occurrence_dirt(&mut self) {\n"
                    "        self.take_target_dirt();\n"
                    "    }\n"
                    "    fn next(&self) {}\n"
                    "}\n"
                ),
                (
                    "impl X {\n"
                    "    fn clear_retained_data_bind_occurrence_dirt(&mut self) {\n"
                    "        self.take_target_dirt();\n"
                    "        self.take_pending_source_dirt();\n"
                    "    }\n"
                    "    fn next(&self) {}\n"
                    "}\n"
                ),
            ),
            (
                "scripted_converter_optional_lookup_split",
                "crates/nuxie-runtime/src/scripted_data_converter.rs",
                (
                    "impl X {\n"
                    "    pub(crate) fn apply_conversion(&mut self) {\n"
                    "        if self.has_data_converter_method() {\n"
                    "            self.call_data_converter_if_present();\n"
                    "        }\n"
                    "    }\n"
                    "    fn next(&self) {}\n"
                    "}\n"
                ),
                (
                    "impl X {\n"
                    "    pub(crate) fn apply_conversion(&mut self) {\n"
                    "        self.call_optional_data_converter_once();\n"
                    "    }\n"
                    "    pub(crate) fn next(&self) {\n"
                    "        let _ = self.has_data_converter_method();\n"
                    "    }\n"
                    "}\n"
                ),
            ),
            (
                "scripted_converter_optional_lookup_split",
                "crates/nuxie-scripting/src/vm.rs",
                (
                    "impl X {\n"
                    "    fn call_optional_data_converter_once(&self, method: Method) {\n"
                    "        let _ = self.table.get(method.as_str());\n"
                    "        let _ = self.table.get(method.as_str());\n"
                    "    }\n"
                    "    fn next(&self) {}\n"
                    "}\n"
                ),
                (
                    "impl X {\n"
                    "    fn call_optional_data_converter_once(&self, method: Method) {\n"
                    "        let value = self.table.get(method.as_str());\n"
                    "        self.invoke(value);\n"
                    "    }\n"
                    "    pub(crate) fn next(&self) {}\n"
                    "}\n"
                ),
            ),
        ]
        for ratchet_id, source, forbidden, safe in cases:
            with self.subTest(ratchet=ratchet_id, source=source):
                self.assert_production_ratchet_case(
                    ratchet_id,
                    source,
                    forbidden,
                    safe,
                )

    def test_fl_c5_definition_collection_live_ratchets_reject_forbidden_shapes(
        self,
    ) -> None:
        cases = [
            (
                "state_machine_listener_slot_compaction",
                "crates/nuxie-runtime/src/state_machine/state_machine.rs",
                """
                fn build(state_machine: &Definition) {
                    let listeners = state_machine
                        .listeners
                        .iter()
                        .filter_map(build_listener);
                }
                """,
                """
                fn build(state_machine: &Definition) {
                    let listeners = state_machine.listeners.iter().map(build_listener);
                }
                """,
            ),
            (
                "state_machine_input_slot_flattening",
                "crates/nuxie-runtime/src/state_machine/state_machine.rs",
                """
                fn build(state_machine: &Definition) {
                    let inputs = state_machine
                        .inputs
                        .iter()
                        .flatten();
                }
                """,
                """
                fn build(state_machine: &Definition) {
                    let inputs = state_machine.inputs.iter().map(|input| input.as_ref());
                }
                """,
            ),
            (
                "state_machine_authored_collection_map_or_set",
                "crates/nuxie-runtime/src/state_machine/state_machine.rs",
                """
                struct RuntimeStateMachine {
                    layers: HashSet<u32>,
                }
                """,
                """
                struct Definition {
                    layers: Vec<u32>,
                }
                """,
            ),
            (
                "state_machine_authored_collection_map_or_set",
                "crates/nuxie-runtime/src/state_machine/state_machine.rs",
                """
                struct RuntimeStateMachine {
                    inputs: Arc<HashMap<String, Input>>,
                }
                """,
                """
                struct Definition {
                    inputs: Arc<Vec<Option<Input>>>,
                }
                """,
            ),
            (
                "state_machine_other_authored_collection_drop_or_reorder",
                "crates/nuxie-runtime/src/state_machine/state_machine.rs",
                """
                fn build(state_machine: &Definition) {
                    let layers = state_machine
                        .layers
                        .iter()
                        .enumerate()
                        .filter_map(build_layer);
                }
                """,
                """
                fn build(state_machine: &Definition) {
                    let layers = state_machine.layers.iter().map(build_layer);
                }
                """,
            ),
            (
                "state_machine_other_authored_collection_drop_or_reorder",
                "crates/nuxie-runtime/src/state_machine/state_machine.rs",
                """
                fn build(state_machine: &Definition) {
                    let scripts = state_machine
                        .scripted_objects
                        .iter()
                        .cloned()
                        .filter_map(build_script);
                }
                """,
                """
                fn build(state_machine: &Definition) {
                    let scripts = state_machine.scripted_objects.iter().map(build_script);
                }
                """,
            ),
            (
                "state_machine_definition_owner_in_root",
                "crates/nuxie-runtime/src/state_machine.rs",
                """
                fn unrelated_displaced_helper() {}
                """,
                """
                pub use state_machine::RuntimeStateMachine;
                """,
            ),
        ]
        for ratchet_id, source, forbidden, safe in cases:
            with self.subTest(ratchet=ratchet_id, source=source):
                self.assert_production_ratchet_case(
                    ratchet_id,
                    source,
                    textwrap.dedent(forbidden),
                    textwrap.dedent(safe),
                )

    def test_fl_c5_entrypoint_and_public_api_negative_controls(self) -> None:
        self.assert_production_ratchet_case(
            "state_machine_instance_owner_in_compat_entry",
            "crates/nuxie-runtime/src/state_machine/instance.rs",
            "fn unrelated_displaced_helper() {}\n",
            (
                "pub use super::state_machine_instance::"
                "{FocusState, StateMachineInstance};\n"
            ),
        )
        required_cases = [
            (
                "state_machine_definition_reexport_required",
                "crates/nuxie-runtime/src/state_machine.rs",
                "pub use state_machine::RuntimeStateMachine;\n",
                "use state_machine::RuntimeStateMachine;\n",
            ),
            (
                "state_machine_instance_reexport_required",
                "crates/nuxie-runtime/src/state_machine/instance.rs",
                (
                    "pub use super::state_machine_instance::"
                    "{FocusState, StateMachineInstance};\n"
                ),
                (
                    "pub(super) use super::state_machine_instance::"
                    "{FocusState, StateMachineInstance};\n"
                ),
            ),
            (
                "state_machine_public_export_hub_required",
                "crates/nuxie-runtime/src/lib.rs",
                (
                    "pub use state_machine::{FocusState, RuntimeStateMachine, "
                    "StateMachineInstance, StateMachineReportedEvent};\n"
                ),
                (
                    "pub(crate) use state_machine::{FocusState, RuntimeStateMachine, "
                    "StateMachineInstance, StateMachineReportedEvent};\n"
                ),
            ),
        ]
        for ratchet_id, source, required, missing in required_cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_required_production_ratchet_case(
                    ratchet_id,
                    source,
                    required,
                    missing,
                )

        production_inventory_path = (
            PRODUCTION_ROOT
            / "crates/nuxie-runtime/tests/public_api_fl_c5.rs"
        )
        production_inventory = production_inventory_path.read_text()
        relative_inventory = (
            "crates/nuxie-runtime/tests/public_api_fl_c5.rs"
        )
        for ratchet_id, missing_inventory in [
            (
                "state_machine_public_api_inventory_required",
                production_inventory.replace(
                    "let _: $signature = $method;",
                    "let _ = $method;",
                    1,
                ),
            ),
            (
                "state_machine_public_api_exact_signature_count_required",
                production_inventory.replace(
                    "    exact_public_signature!(StateMachineInstance::"
                    "set_owned_view_model_context_view_model_source_for_data_bind",
                    "    removed_signature!(StateMachineInstance::"
                    "set_owned_view_model_context_view_model_source_for_data_bind",
                    1,
                ),
            ),
        ]:
            with self.subTest(ratchet=ratchet_id):
                self.assert_required_production_ratchet_case(
                    ratchet_id,
                    relative_inventory,
                    production_inventory,
                    missing_inventory,
                )

        base_gaps = self.gaps.read_text()
        try:
            self.install_production_ratchet(
                "state_machine_public_api_inventory_required"
            )
            source = self.repo / relative_inventory
            source.parent.mkdir(parents=True, exist_ok=True)
            source.write_text(production_inventory)
            result = self.run_check()
            self.assertEqual(result.returncode, 0, result.stderr)

            for label, count_preserving_substitution in [
                (
                    "central inventory signature",
                    production_inventory.replace(
                        "    exact_public_signature!(StateMachineInstance::"
                        "set_owned_view_model_context_asset_source_for_data_bind"
                        " => fn(&mut StateMachineInstance, &mut "
                        "RuntimeOwnedViewModelInstance, usize, u64) -> bool);",
                        "    exact_public_signature!(StateMachineInstance::"
                        "set_owned_view_model_context_enum_source_for_data_bind"
                        " => fn(&mut StateMachineInstance, &mut "
                        "RuntimeOwnedViewModelInstance, usize, u64) -> bool);",
                        1,
                    ),
                ),
                (
                    "generic hydration signature",
                    production_inventory.replace(
                        "StateMachineInstance::"
                        "hydrate_and_initialize_scripted_listener_action_instance"
                        "::<HydrationFactory>",
                        "StateMachineInstance::"
                        "hydrate_and_initialize_scripted_object_instance"
                        "::<HydrationFactory>",
                        1,
                    ),
                ),
            ]:
                with self.subTest(count_preserving_substitution=label):
                    self.assertNotEqual(
                        count_preserving_substitution,
                        production_inventory,
                        "the negative must replace one real exact signature",
                    )
                    source.write_text(count_preserving_substitution)
                    result = self.run_check()
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn(
                        "ratchet state_machine_public_api_inventory_required "
                        "content digest changed",
                        result.stderr,
                    )
        finally:
            self.gaps.write_text(base_gaps)

        for ratchet_id, source_path in [
            (
                "state_machine_definition_owner_in_root",
                "crates/nuxie-runtime/src/state_machine.rs",
            ),
            (
                "state_machine_instance_owner_in_compat_entry",
                "crates/nuxie-runtime/src/state_machine/instance.rs",
            ),
        ]:
            for qualifier in [
                "pub const",
                "pub unsafe",
                'pub extern "C"',
            ]:
                with self.subTest(ratchet=ratchet_id, qualifier=qualifier):
                    self.assert_production_ratchet_case(
                        ratchet_id,
                        source_path,
                        f"{qualifier} fn unrelated_displaced_helper() {{}}\n",
                        "pub use owner::Item;\n",
                    )

    def test_fl_c5_instance_lifecycle_live_ratchets_reject_forbidden_shapes(
        self,
    ) -> None:
        source = "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs"
        cases = [
            (
                "state_machine_instance_constructor_phase_reorder",
                """
                fn new(&mut self) {
                    self.initialize_ordinary_data_bind_container();
                    self.initialize_layers_in_authored_order(machine);
                }
                """,
                """
                fn new(&mut self) {
                    self.initialize_layers_in_authored_order(machine);
                    self.initialize_ordinary_data_bind_container();
                }
                """,
            ),
            (
                "state_machine_queued_event_default_sentinel",
                """
                #[derive(Debug, Default)]
                struct RuntimeQueuedFocusEvent {
                    listener_index: usize,
                    is_focus: bool,
                }
                """,
                """
                #[derive(Debug)]
                struct RuntimeQueuedFocusEvent {
                    listener_index: usize,
                    is_focus: bool,
                }
                """,
            ),
            (
                "state_machine_lifecycle_shallow_clone",
                """
                fn clone(&self) {
                    let queue = Rc::clone(&self.queued_focus_events);
                }
                """,
                """
                fn clone(&self) {
                    let queue = self.queued_focus_events.clone();
                }
                """,
            ),
            (
                "state_machine_dispose_missing_nested_detach",
                """
                impl StateMachineInstance {
                    pub fn dispose(&mut self) {
                        self.disposed = true;
                    }
                }
                """,
                """
                impl StateMachineInstance {
                    pub fn dispose(&mut self) {
                        self.detach_nested_event_registrations();
                        self.disposed = true;
                    }
                }
                """,
            ),
            (
                "state_machine_teardown_bind_layer_script_reorder",
                """
                impl Drop for StateMachineInstance {
                    fn drop(&mut self) {
                        self.teardown_layers();
                        self.teardown_bind_occurrences();
                        self.teardown_script_occurrences();
                    }
                }
                """,
                """
                impl Drop for StateMachineInstance {
                    fn drop(&mut self) {
                        self.teardown_bind_occurrences();
                        self.teardown_layers();
                        self.teardown_script_occurrences();
                    }
                }
                """,
            ),
            (
                "state_machine_teardown_bind_layer_script_reorder",
                """
                impl Drop for StateMachineInstance {
                    fn drop(&mut self) {
                        self.teardown_bind_occurrences();
                        self.teardown_script_occurrences();
                        self.teardown_layers();
                    }
                }
                """,
                """
                impl Drop for StateMachineInstance {
                    fn drop(&mut self) {
                        self.teardown_bind_occurrences();
                        self.teardown_layers();
                        self.teardown_script_occurrences();
                    }
                }
                """,
            ),
            (
                "state_machine_teardown_bind_layer_script_reorder",
                """
                impl Drop for StateMachineInstance {
                    fn drop(&mut self) {
                        self.teardown_script_occurrences();
                        self.teardown_bind_occurrences();
                        self.teardown_layers();
                    }
                }
                """,
                """
                impl Drop for StateMachineInstance {
                    fn drop(&mut self) {
                        self.teardown_bind_occurrences();
                        self.teardown_layers();
                        self.teardown_script_occurrences();
                    }
                }
                """,
            ),
        ]
        for ratchet_id, forbidden, safe in cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_production_ratchet_case(
                    ratchet_id,
                    source,
                    textwrap.dedent(forbidden),
                    textwrap.dedent(safe),
                )

    def test_fl_c5_layer_state_live_ratchets_require_occurrence_queries(
        self,
    ) -> None:
        layer_source = (
            "crates/nuxie-runtime/src/state_machine/"
            "state_machine_layer_instance.rs"
        )
        instance_source = (
            "crates/nuxie-runtime/src/state_machine/"
            "state_machine_instance.rs"
        )
        required_cases = [
            (
                "state_machine_layer_changed_flag_required",
                layer_source,
                "struct Layer { state_changed_on_advance: bool }\n",
                "struct Layer { changed_count: usize }\n",
            ),
            (
                "state_machine_layer_new_frame_clear_required",
                layer_source,
                (
                    "fn begin_new_frame(&mut self) {\n"
                    "    self.state_changed_on_advance = false;\n"
                    "}\n"
                ),
                "fn begin_new_frame(&mut self) {}\n",
            ),
            (
                "state_machine_layer_current_state_access_required",
                layer_source,
                (
                    "fn current_state<'a>(&self, layer: &'a RuntimeLayerState) "
                    "-> Option<&'a RuntimeLayerState> { Some(layer) }\n"
                ),
                "fn current_state_index(&self) -> Option<usize> { None }\n",
            ),
            (
                "state_machine_changed_count_scans_layer_flags_required",
                instance_source,
                (
                    "pub fn changed_state_count(&self) -> usize {\n"
                    "    self.layers.iter().filter(|layer| "
                    "layer.state_changed_on_advance()).count()\n"
                    "}\n"
                ),
                (
                    "pub fn changed_state_count(&self) -> usize {\n"
                    "    self.changed_state_count\n"
                    "}\n"
                ),
            ),
            (
                "state_machine_changed_state_query_required",
                instance_source,
                (
                    "pub fn changed_state(&self, index: usize) "
                    "-> Option<&RuntimeLayerState> {\n"
                    "    for layer in self.layers.iter() {\n"
                    "        if layer.state_changed_on_advance() {\n"
                    "            return layer.current_state(definition);\n"
                    "        }\n"
                    "    }\n"
                    "    None\n"
                    "}\n"
                ),
                "pub fn current_state(&self) -> Option<&RuntimeLayerState> { None }\n",
            ),
            (
                "state_machine_layer_random_scratch_required",
                layer_source,
                "struct Layer { evaluated_random_weights: Vec<u32> }\n",
                "struct Layer;\n",
            ),
            (
                "state_machine_layer_trigger_identity_required",
                layer_source,
                "struct Layer { view_model_trigger_layer_id: u64 }\n",
                "struct Layer;\n",
            ),
        ]
        for ratchet_id, source, required, missing in required_cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_required_production_ratchet_case(
                    ratchet_id,
                    source,
                    required,
                    missing,
                )

        self.assert_required_production_ratchet_case(
            "state_machine_artboard_single_nested_delegation_required",
            "crates/nuxie-runtime/src/artboard.rs",
            textwrap.dedent(
                """
                pub fn advance_nested_artboards_with_state_machine() {
                    StateMachineInstance::dispatch_collected_nested_events_with(
                        |artboard, events| {
                            artboard.advance_nested_artboards_collect_events(events);
                        },
                    );
                }
                pub fn advance_frame_components_with_state_machine() {
                    StateMachineInstance::dispatch_collected_nested_events_with(
                        |artboard, events| {
                            artboard.advance_frame_components_collect_events_with_mode(events);
                        },
                    );
                }
                """
            ),
            textwrap.dedent(
                """
                pub fn advance_nested_artboards_with_state_machine() {
                    advance_nested_artboards_collect_events();
                }
                pub fn advance_frame_components_with_state_machine() {
                    advance_frame_components_collect_events_with_mode();
                }
                """
            ),
        )

        forbidden_cases = [
            (
                "state_machine_cached_changed_count",
                "struct Machine {\n    changed_state_count: usize,\n}\n",
                "struct Machine { layers: Vec<Layer> }\n",
            ),
            (
                "state_machine_stale_transition_query_alias",
                "impl Machine { fn random_value(&self) {} }\n",
                "impl Machine { fn changed_state(&self) {} }\n",
            ),
            (
                "state_machine_stale_transition_query_alias",
                "impl Machine { fn find_random_transition(&self) {} }\n",
                "impl Machine { fn changed_state(&self) {} }\n",
            ),
            (
                "state_machine_stale_transition_query_alias",
                "impl Machine { fn find_allowed_transition(&self) {} }\n",
                "impl Machine { fn changed_state(&self) {} }\n",
            ),
        ]
        for ratchet_id, forbidden, safe in forbidden_cases:
            with self.subTest(ratchet=ratchet_id, source=instance_source):
                self.assert_production_ratchet_case(
                    ratchet_id,
                    instance_source,
                    forbidden,
                    safe,
                )

    def test_fl_c5_hit_live_ratchets_require_complete_hierarchy_and_routing(
        self,
    ) -> None:
        source = "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs"
        cases = [
            (
                "state_machine_hit_trait_required",
                "trait HitComponent {}\n",
                "trait BooleanHit {}\n",
            ),
            (
                "state_machine_hit_concrete_types_required",
                """
                struct HitDrawable;
                struct HitExpandable;
                struct HitTextRun;
                struct HitLayout;
                struct HitNestedArtboard;
                struct HitComponentList;
                """,
                """
                struct HitDrawable;
                struct HitExpandable;
                struct HitTextRun;
                struct HitLayout;
                struct HitNestedArtboard;
                """,
            ),
            (
                "state_machine_hit_result_tristate_required",
                "enum HitResult { None, Hit, HitOpaque }\n",
                "type HitResult = bool;\n",
            ),
            (
                "state_machine_hit_three_pass_order_required",
                """
                fn update() {
                    for group in &mut groups { group.reset(owner, pointer_id); }
                    for hit in &mut hit_components {
                        hit.prepare_event(owner, groups);
                    }
                    let mut result = HitResult::None;
                    for hit in &mut hit_components { hit.process_event(owner); }
                }
                """,
                """
                fn update() {
                    for group in &mut groups { group.reset(owner, pointer_id); }
                    let mut result = HitResult::None;
                    for hit in &mut hit_components {
                        hit.prepare_event(owner, groups);
                        hit.process_event(owner);
                    }
                }
                """,
            ),
            (
                "state_machine_hit_can_hit_propagation_required",
                """
                fn update() {
                    hit.process_event(owner, result != HitResult::HitOpaque);
                    hit.process_event(owner, drag_start_result != HitResult::HitOpaque);
                }
                """,
                """
                fn update() {
                    hit.process_event(owner, result != HitResult::HitOpaque);
                    hit.process_event(owner, true);
                }
                """,
            ),
            (
                "state_machine_hit_component_list_reverse_required",
                """
                fn pointer() {
                    for item_index in order.into_iter().rev() {}
                    for item_index in order.into_iter().rev() {}
                }
                """,
                """
                fn pointer() {
                    for item_index in order.into_iter() {}
                    for item_index in order.into_iter().rev() {}
                }
                """,
            ),
            (
                "state_machine_hit_draw_order_counter_required",
                """
                fn advance() {
                    if self.draw_order_change_counter != draw_order_change_counter {
                        self.sort_hit_components(artboard);
                    }
                }
                """,
                "fn advance() { self.sort_hit_components(artboard); }\n",
            ),
            (
                "state_machine_hit_exit_release_required",
                """
                fn update() {
                    if hit_type == RuntimeListenerType::Exit {
                        self.release_pointer_input(pointer_id);
                    }
                }
                """,
                "fn update() { self.release_pointer_input(pointer_id); }\n",
            ),
            (
                "state_machine_hit_enable_disable_walks_required",
                """
                fn enable_pointer_events(&mut self, pointer_id: i32) {
                    for hit in &mut self.hit_components { hit.enable(pointer_id); }
                }
                fn disable_pointer_events(&mut self, pointer_id: i32) {
                    for hit in &mut self.hit_components { hit.disable(pointer_id); }
                }
                """,
                """
                fn enable_pointer_events(&mut self, pointer_id: i32) {}
                fn disable_pointer_events(&mut self, pointer_id: i32) {
                    for hit in &mut self.hit_components { hit.disable(pointer_id); }
                }
                """,
            ),
        ]
        for ratchet_id, required, missing in cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_required_production_ratchet_case(
                    ratchet_id,
                    source,
                    textwrap.dedent(required),
                    textwrap.dedent(missing),
                )

        forbidden_cases = [
            (
                "state_machine_hit_break_after_opaque",
                """
                fn update_listeners() {
                    if result == HitResult::HitOpaque { break; }
                }
                """,
                "fn update_listeners() { continue_processing(false); }\n",
            ),
            (
                "state_machine_displaced_pointer_listener_traversal",
                """
                fn update_pointer_listeners() {
                    for listener in self.listener_definitions.iter() {
                        listener.dispatch();
                    }
                }
                """,
                "fn update_pointer_listeners() { self.update_listeners(); }\n",
            ),
        ]
        for ratchet_id, forbidden, safe in forbidden_cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_production_ratchet_case(
                    ratchet_id,
                    source,
                    textwrap.dedent(forbidden),
                    textwrap.dedent(safe),
                )

    def test_fl_c5_bind_live_ratchets_preserve_null_order_and_delegation(
        self,
    ) -> None:
        source = "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs"
        required_cases = [
            (
                "state_machine_bind_primary_family_required",
                """
                fn set_view_model_instance() {}
                fn set_global_view_model_instance() {}
                fn complete_view_model_instances() {}
                fn bind() {}
                fn bind_view_model_instance() {}
                fn bind_data_context() {}
                fn inherit_data_context() {}
                fn set_data_context() {}
                fn data_context() {}
                fn global_view_model_instance() {}
                fn rebind() {}
                fn clear_data_context() {}
                fn relink_data_context() {}
                fn rebuild_data_bind() {}
                fn unbind() {}
                fn internal_data_context() {}
                """,
                """
                fn set_view_model_instance() {}
                fn set_global_view_model_instance() {}
                fn complete_view_model_instances() {}
                fn bind() {}
                fn bind_view_model_instance() {}
                fn bind_data_context() {}
                fn inherit_data_context() {}
                fn set_data_context() {}
                fn data_context() {}
                fn global_view_model_instance() {}
                fn rebind() {}
                fn clear_data_context() {}
                fn relink_data_context() {}
                fn rebuild_data_bind() {}
                fn unbind() {}
                """,
            ),
            (
                "state_machine_bind_null_branches_distinct_required",
                """
                fn set_view_model_instance(view_model_instance: Option<Handle>) {
                    let Some(view_model_instance) = view_model_instance else {
                        return false;
                    };
                    stage(view_model_instance)
                }
                fn bind_view_model_instance(view_model_instance: Option<Handle>) {
                    let Some(view_model_instance) = view_model_instance else {
                        self.clear_data_context();
                        artboard.unbind_for_state_machine_view_model_clear(file);
                        return Ok(true);
                    };
                    bind(view_model_instance)
                }
                """,
                """
                fn set_view_model_instance(view_model_instance: Option<Handle>) {
                    let Some(view_model_instance) = view_model_instance else {
                        self.clear_data_context();
                        return false;
                    };
                    stage(view_model_instance)
                }
                fn bind_view_model_instance(view_model_instance: Option<Handle>) {
                    self.set_view_model_instance(view_model_instance)
                }
                """,
            ),
            (
                "state_machine_bind_data_context_null_and_order_required",
                """
                fn bind_data_context(data_context: Option<&Context>) {
                    let data_context = data_context.ok_or(RuntimeDataContextBindError::NullDataContext)?;
                    self.clear_data_context();
                    data_context.add_rebind_dependent(&sink);
                    artboard.clear_data_context_for_state_machine_bind();
                    artboard.bind_owned_view_model_artboard_data_context(file, data_context);
                    self.internal_data_context(Some(data_context))
                }
                """,
                """
                fn bind_data_context(data_context: Option<&Context>) {
                    let Some(data_context) = data_context else { return Ok(false); };
                    self.clear_data_context();
                    data_context.add_rebind_dependent(&sink);
                    self.internal_data_context(Some(data_context))
                }
                """,
            ),
            (
                "state_machine_internal_context_listener_before_script_required",
                """
                fn internal_data_context() {
                    self.bind_view_model_listener_cells_for_data_context(data_context);
                    self.record_bind_phase("script-context-pass");
                    self.record_bind_phase("script-init-pass");
                }
                """,
                """
                fn internal_data_context() {
                    self.record_bind_phase("script-context-pass");
                    self.record_bind_phase("script-init-pass");
                    self.bind_view_model_listener_cells_for_data_context(data_context);
                }
                """,
            ),
            (
                "state_machine_typed_context_primary_delegation_required",
                """
                fn bind_empty_data_context() {
                    self.bind_typed_context_adaptation(bind)
                }
                fn bind_default_view_model_context() {
                    self.bind_typed_context_adaptation(bind)
                }
                fn bind_view_model_instance_context() {
                    self.bind_typed_context_adaptation(bind)
                }
                fn bind_imported_view_model_context() {
                    self.bind_typed_context_adaptation(bind)
                }
                fn bind_owned_view_model_context() {
                    self.bind_typed_context_adaptation(bind)
                }
                fn bind_owned_view_model_handle() {
                    self.bind_owned_view_model_context_handle(context)
                }
                fn bind_owned_view_model_context_handle() {
                    self.bind_owned_view_model_data_context(data_context)
                }
                fn bind_owned_view_model_context_mut() {
                    self.bind_typed_context_adaptation(bind)
                }
                fn bind_owned_view_model_contexts() {
                    self.bind_owned_view_model_data_context(data_context)
                }
                fn bind_script_artboard_data_context() {
                    self.bind_owned_view_model_data_context(data_context)
                }
                fn bind_owned_view_model_context_chain() {
                    self.bind_typed_context_adaptation(bind)
                }
                """,
                """
                fn bind_empty_data_context() {
                    self.data_bind_graph.bind_empty_data_context()
                }
                fn bind_default_view_model_context() {
                    self.bind_typed_context_adaptation(bind)
                }
                fn bind_view_model_instance_context() {
                    self.bind_typed_context_adaptation(bind)
                }
                fn bind_imported_view_model_context() {
                    self.bind_typed_context_adaptation(bind)
                }
                fn bind_owned_view_model_data_context(data_context: &Context) {
                    self.bind_data_context_to_machine(data_context)
                }
                fn bind_owned_view_model_context() {
                    self.bind_typed_context_adaptation(bind)
                }
                fn bind_owned_view_model_handle() {
                    self.bind_owned_view_model_context_handle(context)
                }
                fn bind_owned_view_model_context_handle() {
                    self.bind_owned_view_model_data_context(data_context)
                }
                fn bind_owned_view_model_context_mut() {
                    self.bind_typed_context_adaptation(bind)
                }
                fn bind_owned_view_model_contexts() {
                    self.bind_owned_view_model_data_context(data_context)
                }
                fn bind_script_artboard_data_context() {
                    self.bind_owned_view_model_data_context(data_context)
                }
                fn bind_owned_view_model_context_chain() {
                    self.bind_typed_context_adaptation(bind)
                }
                """,
            ),
        ]
        for ratchet_id, required, missing in required_cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_required_production_ratchet_case(
                    ratchet_id,
                    source,
                    textwrap.dedent(required),
                    textwrap.dedent(missing),
                )

        forbidden_cases = [
            (
                "state_machine_inherit_context_prior_clear",
                """
                fn inherit_data_context(data_context: Option<&Context>) {
                    self.clear_data_context();
                    data_context.add_rebind_dependent(&sink);
                    self.internal_data_context(data_context)
                }
                """,
                """
                fn inherit_data_context(data_context: Option<&Context>) {
                    data_context.add_rebind_dependent(&sink);
                    self.internal_data_context(data_context)
                }
                """,
            ),
            (
                "state_machine_bind_machine_before_artboard",
                """
                fn bind() {
                    self.internal_data_context(data_context);
                    artboard.bind_owned_view_model_artboard_data_context(file, data_context);
                }
                """,
                """
                fn bind() {
                    artboard.bind_owned_view_model_artboard_data_context(file, data_context);
                    self.internal_data_context(data_context);
                }
                """,
            ),
        ]
        for ratchet_id, forbidden, safe in forbidden_cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_production_ratchet_case(
                    ratchet_id,
                    source,
                    textwrap.dedent(forbidden),
                    textwrap.dedent(safe),
                )

    def test_fl_c5_event_live_ratchets_preserve_batches_visibility_and_seams(
        self,
    ) -> None:
        source = "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs"
        required_cases = [
            (
                "state_machine_event_apply_order_required",
                """
                fn apply_local_event_listeners() {
                    self.update_data_binds_false(artboard);
                    let events = std::mem::take(&mut self.reporting_events);
                    self.reported_listener_view_models.swap_into(&mut listeners);
                    self.notify_events_with_context(artboard, None, &events, None);
                    for &listener_index in &listener_indices { notify(listener_index); }
                }
                """,
                """
                fn apply_local_event_listeners() {
                    self.update_data_binds_false(artboard);
                    let events = std::mem::take(&mut self.reporting_events);
                    self.reported_listener_view_models.swap_into(&mut listeners);
                    for &listener_index in &listener_indices { notify(listener_index); }
                    self.notify_events_with_context(artboard, None, &events, None);
                }
                """,
            ),
            (
                "state_machine_event_exact_100_batches_required",
                """
                const MAX_EVENT_ITERATIONS: usize = 100;
                for _ in 0..MAX_EVENT_ITERATIONS { apply_batch(); }
                """,
                """
                const MAX_EVENT_ITERATIONS: usize = 99;
                for _ in 0..MAX_EVENT_ITERATIONS { apply_batch(); }
                """,
            ),
            (
                "state_machine_event_pending_cursor_required",
                """
                fn reported_event_count() {
                    len.saturating_sub(self.next_unapplied_reported_event_index())
                }
                fn reported_event() {
                    let index = self.next_unapplied_reported_event_index() + index;
                }
                """,
                """
                fn reported_event_count() { len }
                fn reported_event() { events.get(index) }
                """,
            ),
            (
                "state_machine_event_local_bubble_audio_order_required",
                """
                fn notify_events_with_context_and_script_host() {
                    self.record_event_dispatch_phase("local-dispatch");
                    dispatch();
                    self.bubble_events_to_owner_seam(events);
                    self.reach_recorded_audio_event_seam(events)
                }
                fn reach_recorded_audio_event_seam() {
                    for event in events.iter().filter(|event| event.is_audio_event()) {
                        self.audio_event_seam.selected(
                            occurrence,
                            &mut self.audio_event_selection_count,
                            &mut self.audio_event_last_occurrence,
                        );
                    }
                }
                trait AudioEventSeam {
                    fn selected();
                }
                impl AudioEventSeam for RecordingAudioEventSeam {
                    fn selected() {
                        *selection_count = selection_count.saturating_add(1);
                        *last_occurrence = Some(occurrence);
                    }
                }
                audio_event_selection_count: usize,
                audio_event_last_occurrence: Option<AudioEventOccurrence>,
                audio_event_seam: Rc::new(RecordingAudioEventSeam),
                """,
                """
                fn notify_events_with_context_and_script_host() {
                    self.record_event_dispatch_phase("local-dispatch");
                    dispatch();
                    self.bubble_events_to_owner_seam(events);
                    self.reach_recorded_audio_event_seam(events)
                }
                fn reach_recorded_audio_event_seam() {
                    for event in events {
                        record(event);
                    }
                }
                """,
            ),
            (
                "state_machine_event_listener_trigger_guard_sink_required",
                """
                RuntimeCellDirtSink::reporting_listener(queue, listener_index)
                """,
                """
                RuntimeCellDirtSink::reporting_data_bind(queue, listener_index)
                """,
            ),
            (
                "state_machine_event_bubble_fifo_required",
                """
                fn bubble_events_to_owner_seam(events: &[Event]) {
                    self.bubbled_event_reports.extend_from_slice(events);
                }
                """,
                """
                fn bubble_events_to_owner_seam(events: &[Event]) {
                    self.record_event_dispatch_phase("bubble-to-owner");
                }
                """,
            ),
        ]
        for ratchet_id, required, missing in required_cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_required_production_ratchet_case(
                    ratchet_id,
                    source,
                    textwrap.dedent(required),
                    textwrap.dedent(missing),
                )

        forbidden_cases = [
            (
                "state_machine_event_current_batch_exposed",
                """
                fn reported_event_count() {
                    self.reporting_events.len()
                }
                """,
                """
                fn reported_event_count() {
                    self.reported_events.len()
                }
                """,
            ),
            (
                "state_machine_event_local_audio_execution",
                """
                fn reach_recorded_audio_event_seam() {
                    self.play_audio_event(event);
                }
                """,
                """
                fn reach_recorded_audio_event_seam() {
                    self.record_event_dispatch_phase("recorded-audio-seam");
                }
                """,
            ),
        ]
        for ratchet_id, forbidden, safe in forbidden_cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_production_ratchet_case(
                    ratchet_id,
                    source,
                    textwrap.dedent(forbidden),
                    textwrap.dedent(safe),
                )

    def test_fl_c5_event_trigger_zero_guard_negative_control(self) -> None:
        self.assert_required_production_ratchet_case(
            "state_machine_event_listener_trigger_zero_guard_required",
            "crates/nuxie-runtime/src/view_model_cell.rs",
            textwrap.dedent(
                """
                notification.suppress_trigger_zero
                    && matches!(self.value, RuntimeViewModelCellValue::Trigger(0))
                """
            ),
            textwrap.dedent(
                """
                notification.suppress_trigger_zero
                    && matches!(self.value, RuntimeViewModelCellValue::Trigger(1))
                """
            ),
        )

    def test_fl_c5_listener_firing_boundary_negative_control(self) -> None:
        required = textwrap.dedent(
            """
            fn finish_listener_view_model_firing_boundary() {
                reported_listener_view_models.report_data_bind(listener_index);
            }
            fn owned_context_listener_report_waits_for_nested_relative_relink() {
                binding.cell.ptr_eq(changed_cell);
                RuntimeListenerViewModelPath::Relative;
                resolved_name_ids.len() > 1;
                manifest.resolve_name(name_id);
            }
            fn write_owned_view_model_context_with_listener_boundary() {
                post_apply_listener_view_models.push(listener_index);
            }
            """
        )
        missing_cases = [
            required.replace("binding.cell.ptr_eq(changed_cell);", ""),
            required.replace("RuntimeListenerViewModelPath::Relative;", ""),
            required.replace("resolved_name_ids.len() > 1;", ""),
            required.replace("manifest.resolve_name(name_id);", ""),
        ]
        for missing in missing_cases:
            self.assert_required_production_ratchet_case(
                "state_machine_vm_listener_firing_boundary_required",
                "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs",
                required,
                missing,
            )

    def test_fl_c5_event_bubble_owner_wiring_negative_control(self) -> None:
        self.assert_required_production_ratchet_case(
            "state_machine_event_bubble_owner_wiring_required",
            "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs",
            textwrap.dedent(
                """
                fn new() {
                    event_bubble_owner_attached: !artboard.frame_origin();
                }
                fn reported_event_count() { bubbled_event_reports.len(); }
                fn reported_event() { bubbled_event_reports.get(index); }
                """
            ),
            textwrap.dedent(
                """
                fn unrelated_setup() { attach_event_bubble_owner(); }
                fn reported_event_count() { reported_events.len(); }
                fn reported_event() { reported_events.get(index); }
                """
            ),
        )

    def test_fl_c5_advance_live_ratchets_and_negative_controls(self) -> None:
        instance_source = (
            "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs"
        )
        required_cases = [
            (
                "state_machine_advance_instance_owner_required",
                """
                fn advance_on_artboard() {}
                fn advance(artboard: &mut ArtboardInstance) {}
                fn advance_artboard_frame_components() { nested_events; }
                pub fn advance_and_apply() {}
                pub fn advance_and_apply_with_view_models() {}
                fn advance_and_apply_state_machines_with_view_models() {
                    if advance_view_models {
                        advance_detached_view_models();
                    }
                }
                fn settle_artboard_update_passes() {}
                """,
                """
                fn advance_on_artboard() {}
                fn advance(artboard: &mut ArtboardInstance) {}
                pub fn advance_and_apply() {}
                pub fn advance_and_apply_with_view_models() {}
                """,
            ),
            (
                "state_machine_advance_raw_order_required",
                """
                fn advance_with_report_mode() {
                    self.record_advance_phase("draw-sort-check");
                    self.apply_local_event_listeners();
                    self.record_advance_phase("clear-latch");
                    self.record_advance_phase("pre-layer-binds");
                    self.record_advance_phase("authored-layers");
                    self.record_advance_phase("converter-advance");
                    self.record_advance_phase("inputs-advanced");
                }
                """,
                """
                fn advance_with_report_mode() {
                    self.record_advance_phase("draw-sort-check");
                    self.apply_local_event_listeners();
                    self.record_advance_phase("clear-latch");
                    self.record_advance_phase("authored-layers");
                    self.record_advance_phase("converter-advance");
                    self.record_advance_phase("inputs-advanced");
                }
                """,
            ),
            (
                "state_machine_advance_return_terms_required",
                """
                fn raw_return() {
                    let advanced = keep_going
                        || self.reported_event_count() != 0
                        || self.has_pending_listener_view_model_reports();
                }
                fn advance_and_apply_return() {
                    changed
                        || elapsed_seconds == 0.0
                        || instance.reported_event_count() != 0
                        || instance.has_pending_listener_view_model_reports()
                }
                """,
                """
                fn raw_return() {
                    let advanced = keep_going
                        || self.reported_event_count() != 0
                        || self.has_pending_listener_view_model_reports();
                }
                fn advance_and_apply_return() {
                    changed
                        || elapsed_seconds == 0.0
                        || instance.has_pending_listener_view_model_reports()
                }
                """,
            ),
            (
                "state_machine_advance_return_terms_required",
                """
                fn raw_return() {
                    let advanced = keep_going
                        || self.reported_event_count() != 0
                        || self.has_pending_listener_view_model_reports();
                }
                fn advance_and_apply_return() {
                    changed
                        || elapsed_seconds == 0.0
                        || instance.reported_event_count() != 0
                        || instance.has_pending_listener_view_model_reports()
                }
                """,
                """
                fn raw_return() {
                    let advanced = keep_going
                        || self.reported_event_count() != 0
                        || self.has_pending_listener_view_model_reports();
                }
                fn advance_and_apply_return() {
                    changed
                        || elapsed_seconds == 0.0
                        || instance.reported_event_count() != 0
                }
                """,
            ),
            (
                "state_machine_advance_return_terms_required",
                """
                fn raw_return() {
                    let advanced = keep_going
                        || self.reported_event_count() != 0
                        || self.has_pending_listener_view_model_reports();
                }
                fn advance_and_apply_return() {
                    changed
                        || elapsed_seconds == 0.0
                        || instance.reported_event_count() != 0
                        || instance.has_pending_listener_view_model_reports()
                }
                """,
                """
                fn raw_return() {
                    let advanced = keep_going
                        || self.reported_event_count() != 0
                        || self.has_pending_listener_view_model_reports();
                }
                fn advance_and_apply_return() {
                    changed
                        || instance.reported_event_count() != 0
                        || instance.has_pending_listener_view_model_reports()
                }
                """,
            ),
            (
                "state_machine_advance_unconditional_settlement_required",
                """
                fn settle_artboard_update_passes() {
                    const MAX_SETTLEMENT_PASSES: usize = 5;
                    for _ in 0..MAX_SETTLEMENT_PASSES {
                        for state_machine in state_machines.iter_mut() {
                            if artboard.try_change_state_machine_instance(state_machine) {
                                artboard.advance_state_machine_instance_after_state_probe(
                                    state_machine,
                                    0.0,
                                );
                            }
                        }
                    }
                }
                """,
                """
                fn settle_artboard_update_passes() {
                    const MAX_SETTLEMENT_PASSES: usize = 5;
                    for _ in 0..MAX_SETTLEMENT_PASSES {
                        for state_machine in state_machines.iter_mut() {
                            if state_machine.requires_post_update_state_probe()
                                && artboard.try_change_state_machine_instance(state_machine)
                            {
                                artboard.advance_state_machine_instance_after_state_probe(
                                    state_machine,
                                    0.0,
                                );
                            }
                        }
                    }
                }
                """,
            ),
            (
                "state_machine_advance_persistent_dirt_real_facade_required",
                """
                fn fl_c5_advance_and_apply_persistent_dirt_component_stops_after_five_passes() {
                    artboard.install_persistent_dirt_component_fixture();
                    let advanced = machine
                        .advance_and_apply(&mut artboard, 0.25);
                    let receipt =
                        artboard.persistent_dirt_component_fixture_receipt();
                    assert_eq!(
                        (advanced, receipt.0, receipt.1, receipt.2),
                        (true, 6, 5, true),
                    );
                }
                """,
                """
                fn fl_c5_advance_and_apply_persistent_dirt_component_stops_after_five_passes() {
                    artboard.install_persistent_dirt_component_fixture();
                    let advanced =
                        StateMachineInstance::settle_artboard_update_passes();
                    let receipt =
                        artboard.persistent_dirt_component_fixture_receipt();
                    assert_eq!(
                        (advanced, receipt.0, receipt.1, receipt.2),
                        (true, 6, 5, true),
                    );
                }
                """,
            ),
        ]
        for ratchet_id, required, missing in required_cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_required_production_ratchet_case(
                    ratchet_id,
                    instance_source,
                    textwrap.dedent(required),
                    textwrap.dedent(missing),
                )

        self.assert_required_production_ratchet_case(
            "state_machine_nested_event_dispatch_retained_scratch_required",
            instance_source,
            textwrap.dedent(
                """
                nested_event_dispatch_scratch:
                    Vec<(usize, Vec<StateMachineReportedEvent>)>,
                fn dispatch_collected_nested_events_with() {
                    let mut nested_events = std::mem::take(
                        &mut state_machine.nested_event_dispatch_scratch,
                    );
                    for (host, events) in nested_events.drain(..) {
                        notify(host, events);
                    }
                    state_machine.nested_event_dispatch_scratch = nested_events;
                }
                """
            ),
            textwrap.dedent(
                """
                fn dispatch_collected_nested_events_with() {
                    let mut nested_events = Vec::new();
                    for (host, events) in nested_events {
                        notify(host, events);
                    }
                }
                """
            ),
        )

        self.assert_required_production_ratchet_case(
            "state_machine_persistent_dirt_component_schedule_required",
            "crates/nuxie-runtime/src/artboard.rs",
            textwrap.dedent(
                """
                struct PersistentDirtComponentFixture {
                    local_id: usize,
                }
                fn install_persistent_dirt_component_fixture() {
                    dependency_order.push(root);
                    advancing_components.push(RuntimeAdvancingComponent {
                        kind: AdvancingComponentKind::Artboard,
                    });
                }
                fn advance_components() {
                    match kind {
                        AdvancingComponentKind::Artboard => {
                            self.advance_persistent_dirt_component_fixture(
                                entry.local_id,
                            )
                        }
                    }
                }
                fn update_components_with_hook_recording() {
                    self.update_persistent_dirt_component_fixture(local_id);
                }
                """
            ),
            textwrap.dedent(
                """
                struct PersistentDirtComponentFixture {
                    local_id: usize,
                }
                fn install_persistent_dirt_component_fixture() {}
                fn advance_components() {
                    self.advance_persistent_dirt_component_fixture(0);
                }
                fn update_pass() {
                    self.update_persistent_dirt_component_fixture(0);
                }
                """
            ),
        )

        self.assert_required_production_ratchet_case(
            "state_machine_facade_constructor_retains_preparation_error_required",
            "crates/nuxie/src/lib.rs",
            textwrap.dedent(
                """
                pub fn state_machine_instance(&mut self, index: usize) -> Option<StateMachineInstance> {
                    let mut machine = self.raw.state_machine_instance(index)?;
                    let _ =
                        try_prepare_state_machine_scripted_data_context_without_factory(
                            file,
                            artboard,
                            &mut machine,
                        );
                    Some(machine)
                }
                pub fn state_machine_instance(&mut self, index: usize) -> Option<StateMachineInstance> {
                    let mut machine = self.raw.state_machine_instance(index)?;
                    let _ =
                        try_prepare_state_machine_scripted_data_context_without_factory(
                            file,
                            artboard,
                            &mut machine,
                        );
                    Some(machine)
                }
                """
            ),
            textwrap.dedent(
                """
                pub fn state_machine_instance(&mut self, index: usize) -> Option<StateMachineInstance> {
                    let mut machine = self.raw.state_machine_instance(index)?;
                    if try_prepare_state_machine_scripted_data_context_without_factory(
                        file,
                        artboard,
                        &mut machine,
                    ).is_err() {
                        return None;
                    }
                    Some(machine)
                }
                """
            ),
        )

        forbidden_cases = [
            (
                "state_machine_nested_event_dispatch_fresh_vec",
                """
                fn dispatch_collected_nested_events_with() {
                    let mut nested_events = Vec::new();
                    collect(&mut nested_events);
                }
                """,
                """
                fn dispatch_collected_nested_events_with() {
                    let mut nested_events = std::mem::take(
                        &mut self.nested_event_dispatch_scratch,
                    );
                    collect(&mut nested_events);
                }
                """,
                instance_source,
            ),
            (
                "state_machine_facade_constructor_drops_preparation_error",
                """
                pub fn state_machine_instance() {
                    if try_prepare_state_machine_scripted_data_context_without_factory(
                        file,
                        artboard,
                        machine,
                    ).is_err() {
                        return None;
                    }
                }
                """,
                """
                pub fn state_machine_instance() {
                    let _ =
                        try_prepare_state_machine_scripted_data_context_without_factory(
                            file,
                            artboard,
                            machine,
                        );
                    Some(machine)
                }
                """,
                "crates/nuxie/src/lib.rs",
            ),
            (
                "state_machine_advance_public_persistent_dirt_probe",
                """
                pub fn runtime_persistent_dirt_settlement_probe() {
                    settle_artboard_update_passes();
                }
                """,
                """
                #[cfg(test)]
                fn persistent_dirt_component_fixture_receipt() {}
                """,
                instance_source,
            ),
            (
                "state_machine_advance_clean_zero_fast_path",
                """
                fn advance(elapsed_seconds: f32) {
                    if elapsed_seconds == 0.0 {
                        return false;
                    }
                }
                """,
                "fn advance(elapsed_seconds: f32) { forward(elapsed_seconds); }\n",
                instance_source,
            ),
            (
                "state_machine_advance_capability_gated_probe",
                """
                fn settle_artboard_update_passes() {
                    if state_machine.transition_probe_enabled()
                        && artboard.try_change_state_machine_instance(state_machine)
                    {}
                }
                """,
                """
                fn settle_artboard_update_passes() {
                    if artboard.try_change_state_machine_instance(state_machine) {}
                }
                """,
                instance_source,
            ),
            (
                "state_machine_advance_nonfinite_rejection",
                """
                fn advance(elapsed_seconds: f32) {
                    if !elapsed_seconds.is_finite() {
                        return;
                    }
                }
                """,
                "fn advance(elapsed_seconds: f32) { forward(elapsed_seconds); }\n",
                instance_source,
            ),
            (
                "state_machine_advance_artboard_settlement_implementation",
                """
                fn settle() {
                    for pass in 0..5 {
                        update_pass(pass);
                        try_change_state_machine_instance(state_machine);
                    }
                }
                """,
                "fn settle() { StateMachineInstance::settle(); }\n",
                "crates/nuxie-runtime/src/artboard.rs",
            ),
            (
                "state_machine_advance_artboard_settlement_implementation",
                """
                fn renamed_settlement_owner() {
                    for pass in 0..5 {
                        update_pass(pass);
                        advance_state_machine_instance_after_state_probe(
                            state_machine,
                        );
                    }
                }
                """,
                """
                fn renamed_settlement_owner() {
                    StateMachineInstance::settle_artboard_update_passes();
                }
                """,
                "crates/nuxie-runtime/src/artboard.rs",
            ),
            (
                "state_machine_artboard_nested_collect_dispatch_orchestration",
                """
                fn renamed_duplicate_orchestration_helper() {
                    let mut forwarded_events = Vec::new();
                    advance_frame_components_collect_events_with_mode(
                        elapsed,
                        mode,
                        Some(&mut forwarded_events),
                    );
                    for (host, reports) in forwarded_events {
                        machine.notify_events(self, Some(host), &reports);
                    }
                }
                """,
                """
                fn renamed_duplicate_orchestration_helper() {
                    StateMachineInstance::dispatch_collected_nested_events_with(
                        self,
                        machine,
                        |artboard, forwarded_events| {
                            artboard.advance_frame_components_collect_events_with_mode(
                                elapsed,
                                mode,
                                Some(forwarded_events),
                            )
                        },
                    );
                }
                """,
                "crates/nuxie-runtime/src/artboard.rs",
            ),
        ]
        for ratchet_id, forbidden, safe, source in forbidden_cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_production_ratchet_case(
                    ratchet_id,
                    source,
                    textwrap.dedent(forbidden),
                    textwrap.dedent(safe),
                )

    def test_fl_c5_keyframe_live_ratchets_and_negative_controls(self) -> None:
        required_cases = [
            (
                "state_machine_keyframe_first_source_bind_required",
                "crates/nuxie-runtime/src/artboard_data_bind.rs",
                """
                fn build_key_frame_data_bind_templates() {
                    let mut claimed_targets = BTreeSet::new();
                    for target in targets {
                        if !claimed_targets.insert(target.id) {
                            continue;
                        }
                    }
                }
                """,
                """
                fn build_key_frame_data_bind_templates() {
                    for target in targets {
                        templates.insert(target.id, target);
                    }
                }
                """,
            ),
            (
                "state_machine_keyframe_holder_set_required",
                "crates/nuxie-runtime/src/data_bind_graph.rs",
                """
                enum RuntimeKeyFrameDataBindTarget {
                    Number,
                    Color,
                    Boolean,
                    String,
                }
                """,
                """
                enum RuntimeKeyFrameDataBindTarget {
                    Number,
                    Color,
                    Boolean,
                    String,
                    Integer,
                }
                """,
            ),
            (
                "state_machine_keyframe_build_order_required",
                "crates/nuxie-runtime/src/animation.rs",
                """
                fn build_key_frame_data_binds() {
                    self.add_key_frame_value_holder(global_id, value);
                    self.key_frame_data_bind_graphs
                        .push(prototype.clone_for_key_frame_instance());
                    graph.take_key_frame_binding_updates(phase);
                }
                """,
                """
                fn build_key_frame_data_binds() {
                    self.key_frame_data_bind_graphs
                        .push(prototype.clone_for_key_frame_instance());
                    self.add_key_frame_value_holder(global_id, value);
                    graph.take_key_frame_binding_updates(phase);
                }
                """,
            ),
            (
                "state_machine_keyframe_animation_traversal_order_required",
                "crates/nuxie-runtime/src/animation.rs",
                """
                fn key_frame_data_bind_templates_in_animation_order() {
                    animation.keyed_objects
                        .flat_map(|object| object.keyed_properties)
                        .flat_map(|property| property.key_frames)
                        .filter_map(RuntimeKeyFrame::bindable_global_id)
                        .filter_map(|id| template_by_key_frame.get(&id));
                }
                """,
                """
                fn key_frame_data_bind_templates_in_animation_order() {
                    templates
                        .filter(|template| key_frame_ids.contains(&template.id));
                }
                """,
            ),
            (
                "state_machine_keyframe_container_phase_order_required",
                "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs",
                """
                fn advance_with_report_mode() {
                    self.prepare_key_frame_data_bind_enrollment(Enrollment::Initial);
                    self.update_data_binds_false();
                    self.prepare_key_frame_data_bind_enrollment(Enrollment::Late);
                    self.record_advance_phase("authored-layers");
                    advance_layers();
                    self.record_advance_phase("converter-advance");
                    self.advance_key_frame_data_bind_enrollment(Enrollment::Initial);
                    for occurrence in self.data_bind_occurrences.clone() {
                        advance(occurrence);
                    }
                    self.advance_key_frame_data_bind_enrollment(Enrollment::Late);
                }
                """,
                """
                fn advance_with_report_mode() {
                    self.update_data_binds_false();
                    self.prepare_key_frame_data_bind_enrollment(Enrollment::Initial);
                    self.prepare_key_frame_data_bind_enrollment(Enrollment::Late);
                    self.record_advance_phase("authored-layers");
                    advance_layers();
                    self.record_advance_phase("converter-advance");
                    for occurrence in self.data_bind_occurrences.clone() {
                        advance(occurrence);
                    }
                }
                """,
            ),
            (
                "state_machine_keyframe_remove_before_holder_required",
                "crates/nuxie-runtime/src/animation.rs",
                """
                fn remove_key_frame_data_binds() {
                    for ((id, enrollment), graph) in self
                        .key_frame_data_bind_occurrences
                        .drain(..)
                        .zip(self.key_frame_data_bind_graphs.drain(..))
                    {
                        drop(graph);
                    }
                    self.key_frame_value_holders = None;
                }
                """,
                """
                fn remove_key_frame_data_binds() {
                    self.key_frame_value_holders = None;
                    for ((id, enrollment), graph) in self
                        .key_frame_data_bind_occurrences
                        .drain(..)
                        .zip(self.key_frame_data_bind_graphs.drain(..))
                    {
                        drop(graph);
                    }
                }
                """,
            ),
            (
                "state_machine_keyframe_snapshot_rebuild_and_owner_ids_required",
                "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs",
                """
                fn key_frame_data_bind_occurrence_ids() {
                    layer.ensure_key_frame_data_binds(graphs);
                    layer.enroll_unassigned_key_frame_data_binds(next_id);
                    layer.collect_key_frame_data_bind_occurrence_ids(enrollment, &mut ids);
                    ids.sort_unstable();
                }
                """,
                """
                fn key_frame_data_bind_occurrence_ids() {
                    layer.collect_key_frame_data_bind_occurrence_ids(enrollment, &mut ids);
                    ids.sort_unstable();
                }
                """,
            ),
            (
                "state_machine_keyframe_snapshot_ensure_is_construction_only",
                "crates/nuxie-runtime/src/animation.rs",
                """
                fn ensure_key_frame_data_binds() {
                    if self.key_frame_data_bind_graphs.is_empty() {
                        self.build_key_frame_data_binds_internal(
                            prototype,
                            enrollment,
                            false,
                        );
                    }
                }
                """,
                """
                fn ensure_key_frame_data_binds() {
                    self.prepare_key_frame_data_binds(prototype);
                }
                """,
            ),
            (
                "state_machine_keyframe_machine_teardown_before_layers_required",
                "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs",
                """
                fn teardown_bind_occurrences() {
                    layer.remove_key_frame_data_binds();
                    self.key_frame_data_bind_graphs.clear();
                }
                fn teardown_layers() {
                    self.layers.clear();
                }
                """,
                """
                fn teardown_bind_occurrences() {
                    self.key_frame_data_bind_graphs.clear();
                }
                fn teardown_layers() {
                    self.layers.clear();
                }
                """,
            ),
            (
                "state_machine_keyframe_cpp_initialize_converter_order_required",
                "crates/nuxie-runtime/tests/cpp_probe.rs",
                """
                fn fl_c5_keyframe_initialize_converter_order() {
                    assert!(holder < clone);
                    assert!(clone < file);
                    assert!(file < target);
                    assert!(target < property);
                    assert!(property < initialize);
                    assert!(initialize < converter);
                    assert!(converter < enrollment);
                    assert!(enrollment < tracking);
                }
                """,
                """
                fn fl_c5_keyframe_initialize_converter_order() {
                    assert!(holder < clone);
                    assert!(clone < file);
                    assert!(file < target);
                    assert!(target < property);
                    assert!(property < converter);
                    assert!(converter < initialize);
                    assert!(initialize < enrollment);
                    assert!(enrollment < tracking);
                }
                """,
            ),
            (
                "state_machine_keyframe_state_build_sites_required",
                "crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs",
                """
                fn new() {
                    any_state.build_key_frame_data_binds(
                        key_frame_data_bind_graphs,
                        crate::animation::RuntimeKeyFrameDataBindEnrollment::Initial,
                    );
                    current_state.build_key_frame_data_binds(
                        key_frame_data_bind_graphs,
                        crate::animation::RuntimeKeyFrameDataBindEnrollment::Initial,
                    );
                }
                fn reset_state() {
                    current_state.build_key_frame_data_binds(
                        key_frame_data_bind_graphs,
                        crate::animation::RuntimeKeyFrameDataBindEnrollment::Late,
                    );
                }
                fn change_state() {
                    current_state.build_key_frame_data_binds(
                        key_frame_data_bind_graphs,
                        crate::animation::RuntimeKeyFrameDataBindEnrollment::Late,
                    );
                }
                """,
                """
                fn new() {
                    any_state.build_key_frame_data_binds(
                        key_frame_data_bind_graphs,
                        crate::animation::RuntimeKeyFrameDataBindEnrollment::Initial,
                    );
                    current_state.build_key_frame_data_binds(
                        key_frame_data_bind_graphs,
                        crate::animation::RuntimeKeyFrameDataBindEnrollment::Initial,
                    );
                }
                fn reset_state() {
                    current_state.build_key_frame_data_binds(
                        key_frame_data_bind_graphs,
                        crate::animation::RuntimeKeyFrameDataBindEnrollment::Late,
                    );
                }
                fn change_state() {
                    drop(current_state);
                }
                """,
            ),
            (
                "state_machine_keyframe_layer_occurrence_order_required",
                "crates/nuxie-runtime/src/state_machine/state_machine_layer_instance.rs",
                """
                fn collect_key_frame_data_bind_occurrence_ids() {
                    any_state.collect_key_frame_data_bind_occurrence_ids(enrollment, ids);
                    state_from.collect_key_frame_data_bind_occurrence_ids(enrollment, ids);
                    current_state.collect_key_frame_data_bind_occurrence_ids(enrollment, ids);
                }
                fn remove_key_frame_data_binds() {
                    any_state.remove_key_frame_data_binds();
                    state_from.remove_key_frame_data_binds();
                    current_state.remove_key_frame_data_binds();
                }
                """,
                """
                fn collect_key_frame_data_bind_occurrence_ids() {
                    any_state.collect_key_frame_data_bind_occurrence_ids(enrollment, ids);
                    current_state.collect_key_frame_data_bind_occurrence_ids(enrollment, ids);
                    state_from.collect_key_frame_data_bind_occurrence_ids(enrollment, ids);
                }
                fn remove_key_frame_data_binds() {
                    any_state.remove_key_frame_data_binds();
                    current_state.remove_key_frame_data_binds();
                    state_from.remove_key_frame_data_binds();
                }
                """,
            ),
        ]
        for ratchet_id, source, required, missing in required_cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_required_production_ratchet_case(
                    ratchet_id,
                    source,
                    textwrap.dedent(required),
                    textwrap.dedent(missing),
                )

        self.assert_production_ratchet_case(
            "state_machine_keyframe_last_source_bind_selection",
            "crates/nuxie-runtime/src/artboard_data_bind.rs",
            textwrap.dedent(
                """
                fn build_key_frame_data_bind_templates() {
                    for data_bind in artboard_data_binds(index).iter().rev() {
                        select(data_bind);
                    }
                }
                """
            ),
            textwrap.dedent(
                """
                fn build_key_frame_data_bind_templates() {
                    for data_bind in artboard_data_binds(index) {
                        select(data_bind);
                    }
                }
                """
            ),
        )
        self.assert_production_ratchet_case(
            "state_machine_keyframe_binding_reorder_by_data_bind_index",
            "crates/nuxie-runtime/src/data_bind_graph.rs",
            textwrap.dedent(
                """
                fn new_key_frame_bindings() {
                    default_view_model_bindings.sort_by_key(|binding| binding.data_bind_index);
                }
                """
            ),
            textwrap.dedent(
                """
                fn new_key_frame_bindings() {
                    retain_key_frame_traversal_order(default_view_model_bindings);
                }
                """
            ),
        )
        self.assert_production_ratchet_case(
            "state_machine_keyframe_snapshot_ensure_prepares_existing",
            "crates/nuxie-runtime/src/animation.rs",
            textwrap.dedent(
                """
                fn ensure_key_frame_data_binds() {
                    self.prepare_key_frame_data_binds(prototype);
                }
                """
            ),
            textwrap.dedent(
                """
                fn ensure_key_frame_data_binds() {
                    build_empty_snapshot_occurrence();
                }
                """
            ),
        )

    def test_fl_c5_focus_semantic_live_ratchets_and_negative_controls(self) -> None:
        instance_source = (
            "crates/nuxie-runtime/src/state_machine/state_machine_instance.rs"
        )
        required_cases = [
            (
                "state_machine_focus_semantic_typed_queues_required",
                """
                struct RuntimeQueuedFocusEvent {
                    listener_index: usize,
                    is_focus: bool,
                }
                struct RuntimeQueuedSemanticEvent {
                    listener_index: Option<usize>,
                    action_type: u32,
                }
                struct StateMachineInstance {
                    queued_focus_events: Vec<RuntimeQueuedFocusEvent>,
                    queued_semantic_events: Vec<RuntimeQueuedSemanticEvent>,
                }
                """,
                """
                struct StateMachineInstance {
                    queued_focus_events: Vec<ScriptListenerInvocation>,
                    queued_semantic_events: Vec<ScriptListenerInvocation>,
                }
                """,
            ),
            (
                "state_machine_focus_snapshot_clear_required",
                """
                fn process_focus_events() {
                    let focus_events =
                        std::mem::take(&mut self.queued_focus_events);
                    for event in focus_events {
                        perform(event);
                    }
                }
                """,
                """
                fn process_focus_events() {
                    for event in self.queued_focus_events.drain(..) {
                        perform(event);
                    }
                }
                """,
            ),
            (
                "state_machine_semantic_snapshot_clear_required",
                """
                fn process_semantic_events() {
                    let semantic_events =
                        std::mem::take(&mut self.queued_semantic_events);
                    for event in semantic_events {
                        perform(event);
                    }
                }
                """,
                """
                fn process_semantic_events() {
                    while let Some(event) = self.queued_semantic_events.pop() {
                        perform(event);
                    }
                }
                """,
            ),
            (
                "state_machine_focus_then_semantic_phase_required",
                """
                fn process_deferred_listener_group_events() {
                    self.process_focus_events();
                    if self.script_error.is_some() {
                        return;
                    }
                    self.process_semantic_events();
                }
                """,
                """
                fn process_deferred_listener_group_events() {
                    self.process_semantic_events();
                    self.process_focus_events();
                }
                """,
            ),
            (
                "state_machine_focus_state_owner_safe_required",
                """
                // RECORDED `src/input/focus_manager.cpp`, row B6-0238
                pub struct FocusState {
                    pub has_focus: bool,
                    pub expects_keyboard_input: bool,
                }
                pub fn internal_focus_manager(&self) -> bool {
                    true
                }
                pub fn focus_state(&self) -> FocusState {
                    FocusState {
                        has_focus,
                        expects_keyboard_input,
                    }
                }
                """,
                """
                pub fn focus_state(&self) -> *const FocusNode {
                    self.primary_focus
                }
                """,
            ),
            (
                "state_machine_focus_manager_switch_fallback_required",
                """
                fn install_external_focus(parent_focus: &RuntimeFocusTree) {
                    if self.external_focus_manager_selected
                        && self.focus.shares_manager(parent_focus)
                    {
                        return;
                    }
                    self.clean_selected_focus_before_manager_switch();
                    self.record("clean-tree-recorded-seam");
                    self.internal_focus =
                        Some(std::mem::take(&mut self.focus));
                    self.external_focus_manager_selected = true;
                    self.record("assign-external");
                    self.record("rebuild-tree-recorded-seam");
                }
                fn clear_external_focus_manager() {
                    self.clean_selected_focus_before_manager_switch();
                    self.record("clean-tree-recorded-seam");
                    self.focus = self.internal_focus.take().unwrap();
                    self.external_focus_manager_selected = false;
                    self.record("assign-internal");
                    self.record("rebuild-tree-recorded-seam");
                }
                """,
                """
                fn install_external_focus(parent_focus: RuntimeFocusTree) {
                    self.focus = parent_focus;
                }
                fn clear_external_focus_manager() {}
                """,
            ),
            (
                "state_machine_semantic_resolver_seam_required",
                """
                // RECORDED absent row B6-0329 `src/semantic/semantic_manager.cpp`
                trait SemanticNodeResolver {
                    fn semantic_data_local_id(
                        &self,
                        semantic_node_id: u32,
                    ) -> Option<usize>;
                }
                semantic_node_resolver: Option<Rc<dyn SemanticNodeResolver>>,
                semantic_node_resolver: None,
                pub fn fire_semantic_action(
                    semantic_node_id: u32,
                    action_type: u32,
                ) {
                    let Some(resolver) = self.semantic_node_resolver.clone() else {
                        return false;
                    };
                    record("node-by-id-recorded-seam");
                    let semantic_data_local_id =
                        resolver.semantic_data_local_id(semantic_node_id);
                    let phase = match action_type {
                        0 => "tap",
                        1 => "increase",
                        2 => "decrease",
                        _ => return false,
                    };
                    self.semantic_action_for_target(target, action_type);
                }
                """,
                """
                pub fn fire_semantic_action(
                    semantic_node_id: u32,
                    action_type: u32,
                ) {
                    let target = semantic_node_id as usize;
                    self.semantic_action_for_target(target, action_type);
                }
                """,
            ),
        ]
        for ratchet_id, required, missing in required_cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_required_production_ratchet_case(
                    ratchet_id,
                    instance_source,
                    textwrap.dedent(required),
                    textwrap.dedent(missing),
                )

        self.assert_production_ratchet_case(
            "state_machine_semantic_ordinal_projection",
            instance_source,
            textwrap.dedent(
                """
                fn internal_semantic_node_id() {}
                """
            ),
            textwrap.dedent(
                """
                trait SemanticNodeResolver {
                    fn semantic_data_local_id(&self, id: u32) -> Option<usize>;
                }
                """
            ),
        )

        self.assert_required_production_ratchet_case(
            "state_machine_focus_owner_safe_identity_projection_required",
            "crates/nuxie-runtime/src/focus.rs",
            textwrap.dedent(
                """
                fn shares_manager(&self, other: &Self) -> bool {
                    Rc::ptr_eq(&self.domain, &other.domain)
                }
                fn has_focus_target(&self, target_local: usize) -> bool {
                    self.domain
                        .borrow()
                        .targets
                        .contains_key(&(self.owner_identity, target_local))
                }
                fn has_primary_focus(&self) -> bool {
                    self.domain.borrow().manager.primary_focus().is_some()
                }
                """
            ),
            textwrap.dedent(
                """
                fn has_focus_target(&self, target_local: usize) -> bool {
                    self.focused_target == Some(target_local)
                }
                fn has_primary_focus(&self) -> bool {
                    !self.focused_listener_chain().is_empty()
                }
                """
            ),
        )

        forbidden_cases = [
            (
                "state_machine_focus_semantic_queue_default_sentinel",
                """
                #[derive(Default)]
                struct RuntimeQueuedFocusEvent {
                    listener_index: usize,
                    is_focus: bool,
                }
                """,
                """
                struct RuntimeQueuedFocusEvent {
                    listener_index: usize,
                    is_focus: bool,
                }
                """,
            ),
            (
                "state_machine_semantic_manager_internal_implementation",
                "enum SemanticTree { Node(SemanticNode) }\n",
                "enum RuntimeSemanticManagerSelection { None, InternalRecorded }\n",
            ),
            (
                "state_machine_focus_manager_internal_implementation",
                "struct RuntimeFocusManager { nodes: Vec<RuntimeFocusNode> }\n",
                "pub struct FocusState { has_focus: bool }\n",
            ),
        ]
        for ratchet_id, forbidden, safe in forbidden_cases:
            with self.subTest(ratchet=ratchet_id):
                self.assert_production_ratchet_case(
                    ratchet_id,
                    instance_source,
                    textwrap.dedent(forbidden),
                    textwrap.dedent(safe),
                )

    def test_fl_c4_negative_ratchets_reject_displaced_listener_action_shapes(self) -> None:
        cases = [
            (
                "listener_action_occurrence_filter_map",
                (
                    r"(?:listener\.actions|state\.fire_actions|transition\.fire_actions|focus_listener_groups|"
                    r"keyboard_listener_groups|gamepad_listener_groups|"
                    r"semantic_listener_groups)[^;]{0,500}filter_map"
                ),
                "fn import() { listener.actions.iter().filter_map(decode); }\n",
            ),
            (
                "listener_action_occurrence_reorder",
                (
                    r"(?:listener_actions|fire_actions|focus_listener_groups|"
                    r"keyboard_listener_groups|gamepad_listener_groups|"
                    r"semantic_listener_groups)\.(?:sort|sort_by|sort_by_key|"
                    r"sort_unstable|sort_unstable_by|sort_unstable_by_key|dedup)"
                ),
                "fn import() { listener_actions.sort_by_key(|action| action.id); }\n",
            ),
            (
                "listener_fire_event_import_snapshot",
                r"(?:event_template|from_runtime_event\([^)]*\))",
                "fn import() { let event_template = from_runtime_event(event); }\n",
            ),
            (
                "listener_fire_event_deferred_host_report",
                (
                    r"RuntimeScheduledListenerAction::FireEvent"
                    r"[\s\S]{0,1200}pending_listener_events\.push"
                ),
                (
                    "fn perform(action: RuntimeScheduledListenerAction) { "
                    "if let RuntimeScheduledListenerAction::FireEvent(event) = action { "
                    "pending_listener_events.push(event); } }\n"
                ),
            ),
            (
                "listener_reported_event_payload_frozen_at_fire",
                r"reported_events\[start\.\.\]\.to_vec\(\)",
                (
                    "fn take_reported_events(&mut self) { "
                    "let events = self.reported_events[start..].to_vec(); }\n"
                ),
            ),
            (
                "listener_reported_event_observed_without_live_artboard",
                (
                    r"pub\s+fn\s+reported_event\(\s*&self"
                    r"[\s\S]{0,240}reported_events\.get\("
                ),
                (
                    "pub fn reported_event(&self, index: usize) { "
                    "self.reported_events.get(index); }\n"
                ),
            ),
            (
                "listener_viewmodel_consumed_null_occurrence_drop",
                (
                    r'"ListenerViewModelChange"[\s\S]{0,240}'
                    r"bindable_property\.is_some\(\)"
                ),
                (
                    'fn validate() { match kind { "ListenerViewModelChange" => '
                    "action.bindable_property.is_some(), _ => true } }\n"
                ),
            ),
            (
                "listener_viewmodel_perform_reads_imported_default",
                (
                    r"RuntimeScheduledListenerAction::ViewModelChange\(action\)"
                    r"[\s\S]{0,1800}action\.value\.as_ref\(\)"
                ),
                (
                    "fn perform(action: RuntimeScheduledListenerAction) { "
                    "if let RuntimeScheduledListenerAction::ViewModelChange(action) = action { "
                    "let value = action.value.as_ref(); } }\n"
                ),
            ),
            (
                "focus_action_target_import_graph_lookup",
                r"\.runtime_graph\(\)",
                (
                    "fn perform(artboard: &ArtboardInstance) { "
                    "let graph = artboard.runtime_graph(); }\n"
                ),
            ),
            (
                "listener_group_import_graph_child_lookup",
                (
                    r"fn\s+listener_target_direct_child"
                    r"[\s\S]{0,1400}\.runtime_graph\(\)"
                ),
                (
                    "fn listener_target_direct_child(artboard: &ArtboardInstance) { "
                    "let graph = artboard.runtime_graph(); }\n"
                ),
            ),
            (
                "listener_pointer_mark_depends_on_action_change",
                (
                    r"if\s+self\.perform_listener_actions_with_event_context\("
                    r"[\s\S]{0,700}\)\?\s*\{\s*self\.needs_advance\s*=\s*true"
                ),
                (
                    "fn pointer() { if self.perform_listener_actions_with_event_context()? "
                    "{ self.needs_advance = true; } }\n"
                ),
            ),
            (
                "listener_align_target_shared_fused_inverse",
                r"parent_world\.(?:determinant|invert|invert_or_identity)\(",
                (
                    "fn perform(parent_world: Mat2D) { "
                    "let inverse = parent_world.invert_or_identity(); }\n"
                ),
            ),
            (
                "listener_scripted_action_double_dispatch",
                (
                    r"fn\s+perform_listener_actions_with_event_context"
                    r"[\s\S]{0,6500}perform_script_object_listener_action"
                ),
                (
                    "fn perform_listener_actions_with_event_context() { "
                    "perform_script_object_listener_action(); }\n"
                ),
            ),
            (
                "listener_scripted_resource_error_erased",
                (
                    r"(?:(?:fn\s+perform_listener_actions_with_event_context"
                    r"[\s\S]{0,6500}perform_scripted_listener_action"
                    r"|fn\s+perform_scheduled_listener_actions"
                    r"[\s\S]{0,2600}perform_instance_action"
                    r"|call_scripted_drawable_input|perform_listener_actions)"
                    r"\([^;]{0,300}unwrap_or\(false\)"
                    r"|let\s+_\s*=\s*[^;]{0,300}"
                    r"(?:call_scripted_drawable_input|perform_listener_actions)\()"
                ),
                (
                    "fn perform_listener_actions_with_event_context() { "
                    "let changed = perform_scripted_listener_action()"
                    ".unwrap_or(false); }\n"
                ),
            ),
            (
                "listener_ordinary_script_error_truncates_fifo",
                r"perform_instance_action\([^;]{0,300}\)\?",
                (
                    "fn perform_scheduled_listener_actions() -> Result<bool, Error> { "
                    "let changed = executor.perform_instance_action("
                    "artboard, action, targets)?; Ok(changed) }\n"
                ),
            ),
            (
                "focused_dispatch_flat_group_scan",
                (
                    r"pub\s+fn\s+(?:key_input|text_input|gamepad_dispatch)"
                    r"\([^\{]*\{[\s\S]{0,1800}"
                    r"(?:keyboard_listener_groups|gamepad_listener_groups)\.iter\(\)"
                ),
                (
                    "pub fn key_input() { "
                    "for group in self.keyboard_listener_groups.iter() {} }\n"
                ),
            ),
            (
                "gamepad_focused_identity_global_only",
                r"already_dispatched\s*:\s*Option<u32>",
                "fn broadcast(already_dispatched: Option<u32>) {}\n",
            ),
            (
                "gamepad_scripted_parent_listener_fallback",
                (
                    r'component\(group\.target_local_id\)[\s\S]{0,420}'
                    r'type_name\s*==\s*"ScriptedDrawable"[\s\S]{0,220}'
                    r"&&\s*let\s+Some\(script\)"
                ),
                (
                    'fn dispatch() { if let Some(id) = component(group.target_local_id) '
                    '.filter(|component| component.type_name == "ScriptedDrawable") '
                    ".map(|component| component.global_id) && let Some(script) = "
                    "artboard.script_instance_for_global(id) { call(script); } "
                    "perform_listener_actions(); }\n"
                ),
            ),
            (
                "scripted_drawable_subtype_dispatch_drop",
                r'type_name\s*(?:==|!=)\s*"ScriptedDrawable"',
                'fn register(component: Component) { if component.type_name == "ScriptedDrawable" {} }\n',
            ),
            (
                "mixed_report_listener_falls_through",
                (
                    r"for\s+\([^)]*listener[^)]*\)\s+in\s+listener_definitions"
                    r"[^\{]*\{(?:(?!listener_uses_report_queue)[\s\S]){0,700}"
                    r"(?:focus_listener_groups|keyboard_listener_groups|"
                    r"gamepad_listener_groups|semantic_listener_groups)\.push"
                ),
                (
                    "fn init() { for (index, listener) in listener_definitions.iter() { "
                    "self.focus_listener_groups.push(listener); } }\n"
                ),
            ),
            (
                "mixed_report_listener_hit_test_rediscovery",
                r"fn\s+hit_test[\s\S]{0,900}state_machine\.listeners",
                (
                    "fn hit_test(&self, artboard: &Artboard) { "
                    "let state_machine = artboard.state_machine(0).unwrap(); "
                    "for listener in state_machine.listeners.iter() { listener.hit_test(); } }\n"
                ),
            ),
            (
                "listener_missing_pointer_path_drops_other_channels",
                (
                    r"runtime_listener_hit_paths\([^;]{0,300}"
                    r"(?:\?|\.is_empty\(\)[\s\S]{0,160}"
                    r"(?:return\s+None|continue))"
                ),
                (
                    "fn import(graph: &Graph) -> Option<Listener> { "
                    "let hit_paths = runtime_listener_hit_paths(graph, 1)?; "
                    "Some(Listener { hit_paths }) }\n"
                ),
            ),
            (
                "scripted_input_groups_not_refreshed_at_dispatch",
                (
                    r"pub\s+fn\s+(?:key_input|text_input|gamepad_dispatch)"
                    r"\([^\{]*\{(?:(?!ensure_scripted_input_groups_current)"
                    r"[\s\S]){0,500}(?:focus\.sync|focused_listener_chain)"
                ),
                (
                    "pub fn key_input(&mut self, artboard: &Artboard) { "
                    "self.focus.sync(artboard); "
                    "for node in self.focus.focused_listener_chain() {} }\n"
                ),
            ),
            (
                "scripted_listener_last_data_bind_first_match",
                (
                    r"fn\s+runtime_scripted_listener_action_binding_definition"
                    r"[\s\S]{0,6500}\.find(?:_map)?\("
                ),
                (
                    "fn runtime_scripted_listener_action_binding_definition() { "
                    "let binding = file.data_binds().find(|binding| binding.matches()); }\n"
                ),
            ),
            (
                "scripted_listener_fresh_clone_aliases_live_state",
                (
                    r"fn\s+fresh_clone[\s\S]{0,2600}"
                    r"(?:inputs:\s*self\.inputs\.clone\(\)"
                    r"|retained_bind:\s*binding\.retained_bind\.clone\(\)"
                    r"|converter_state:\s*binding\.converter_state\.clone\(\)"
                    r"|formula_random_source:\s*binding\.formula_random_source\.clone\(\))"
                ),
                (
                    "fn fresh_clone(&self) -> Self { Self { "
                    "inputs: self.inputs.clone(), retained_bind: "
                    "binding.retained_bind.clone() } }\n"
                ),
            ),
            (
                "scripted_converter_occurrence_dedup_or_reorder",
                (
                    r"fn\s+(?:scripted_converter_occurrences"
                    r"|collect_scripted_converter_occurrences"
                    r"|scripted_converter_occurrence_snapshots"
                    r"|collect_scripted_converter_occurrence_snapshots)"
                    r"[\s\S]{0,3200}(?:BTreeMap|HashMap|\.sort|\.dedup)"
                ),
                (
                    "fn scripted_converter_occurrence_snapshots() { "
                    "let mut occurrences = Vec::new(); "
                    "occurrences.sort_by_key(Occurrence::path); }\n"
                ),
            ),
            (
                "scripted_input_binding_flags_zeroed_on_clone",
                (
                    r"(?:fn\s+from_definition\(\s*definition:"
                    r"|pub\(crate\)\s+fn\s+fresh_clone\(&self\))"
                    r"[\s\S]{0,2600}"
                    r"(?:flags:\s*0|RuntimeRetainedDataBind::new\(\s*0)"
                ),
                (
                    "fn from_definition(definition: &Binding) -> Occurrence { "
                    "Occurrence { flags: 0, retained_bind: "
                    "RuntimeRetainedDataBind::new(0, false) } }\n"
                ),
            ),
            (
                "scripted_converter_unbind_clears_authored_collection",
                (
                    r"fn\s+unbind_sources[\s\S]{0,900}"
                    r"(?:inputs\.clear\(\)|data_binds\.clear\(\)|self\.inputs\s*=)"
                ),
                "fn unbind_sources(&mut self) { self.inputs.clear(); }\n",
            ),
            (
                "script_input_artboard_id_equality_short_circuit",
                (
                    r"self\.value\s*==\s*Some\("
                    r"RuntimeDataBindGraphValue::Artboard"
                ),
                (
                    "fn apply_artboard(&mut self, value: u64) { "
                    "if self.value == Some(RuntimeDataBindGraphValue::Artboard(value)) "
                    "{ return; } }\n"
                ),
            ),
            (
                "script_input_trigger_collapsed_to_boolean",
                (
                    r"RuntimeDataBindGraphValue::Trigger\([^)]*"
                    r"(?:!=\s*0|\.min\(1\)|\.clamp\(0,\s*1\))"
                ),
                (
                    "fn trigger(value: u64) { "
                    "let value = RuntimeDataBindGraphValue::Trigger(value.min(1)); }\n"
                ),
            ),
            (
                "state_machine_fire_trigger_resolved_at_import",
                (
                    r"struct\s+RuntimeStateMachineFireTriggerPath\s*\{"
                    r"(?:(?!\n\})[\s\S]){0,700}RuntimeViewModelCell"
                ),
                (
                    "struct RuntimeStateMachineFireTriggerPath { "
                    "trigger_cell: RuntimeViewModelCell }\n"
                ),
            ),
            (
                "state_machine_fire_trigger_drops_file_context",
                (
                    r"fn\s+fire_view_model_trigger[\s\S]{0,900}owned_data_context"
                    r"[\s\S]{0,450}\n\s*false\s*\n\s*\}"
                ),
                (
                    "fn fire_view_model_trigger() {\n"
                    "    if self.owned_data_context.fire() { return true; }\n"
                    "    false\n"
                    "}\n"
                ),
            ),
        ]
        base_gaps = self.gaps.read_text()
        source = self.repo / "crates/runtime/src/state_machine/instance.rs"
        source.parent.mkdir(parents=True, exist_ok=True)

        for ratchet_id, pattern, forbidden_source in cases:
            with self.subTest(ratchet=ratchet_id):
                self.gaps.write_text(
                    base_gaps.replace(
                        "ratchet = []",
                        textwrap.dedent(
                            f"""
                            [[ratchet]]
                            id = "{ratchet_id}"
                            globs = ["crates/runtime/src/state_machine/instance.rs"]
                            pattern = {json.dumps(pattern)}
                            max_occurrences = 0
                            """
                        ).strip(),
                    )
                )
                source.write_text(forbidden_source)
                result = self.run_check()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    f"ratchet {ratchet_id} increased to ",
                    result.stderr,
                )

    def test_ratchets_scan_complete_files_not_individual_lines(self) -> None:
        gaps = self.gaps.read_text()
        self.gaps.write_text(
            gaps.replace(
                "ratchet = []",
                textwrap.dedent(
                    r"""
                    [[ratchet]]
                    id = "multiline_regression"
                    globs = ["crates/runtime/src/state_machine/instance.rs"]
                    pattern = "fn\\s+advance[\\s\\S]{0,200}\\.collect"
                    max_occurrences = 0
                    """
                ).strip(),
            )
        )
        source = self.repo / "crates/runtime/src/state_machine/instance.rs"
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text(
            textwrap.dedent(
                """
                fn advance() {
                    let transitions = source
                        .iter()
                        .collect::<Vec<_>>();
                }
                """
            )
        )

        result = self.run_check()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "ratchet multiline_regression increased to 1 > 0",
            result.stderr,
        )

    def test_mechanism_input_hash_is_fail_closed(self) -> None:
        fixture = self.upstream / "tests/assets/scroll.riv"
        fixture.parent.mkdir(parents=True)
        fixture.write_bytes(b"scroll fixture")
        input_script = self.repo / "tools/trace/scroll.txt"
        input_script.parent.mkdir(parents=True)
        input_script.write_text("0.1 pointerDown 1 2\n")
        fixture_hash = hashlib.sha256(fixture.read_bytes()).hexdigest()
        input_hash = hashlib.sha256(input_script.read_bytes()).hexdigest()
        with self.ledger.open("a") as ledger:
            ledger.write(
                textwrap.dedent(
                    f"""

                    [[trace_mechanism_fixture]]
                    id = "scroll"
                    path = "tests/assets/scroll.riv"
                    sha256 = "{fixture_hash}"
                    samples = [0.0, 0.1]
                    input_script = "tools/trace/scroll.txt"
                    input_sha256 = "{input_hash}"
                    steady = false
                    """
                )
            )
        trace_path = self.repo / "docs/trace.json"
        trace = json.loads(trace_path.read_text())
        trace["mechanism_corpus"] = ["scroll"]
        trace["steady_corpus"] = []
        trace["mechanism_fixture_sha256"] = {"scroll": fixture_hash}
        trace["mechanism_input_sha256"] = {"scroll": input_hash}
        trace_path.write_text(json.dumps(trace))
        self.assertEqual(self.run_check().returncode, 0)

        input_script.write_text("0.1 pointerDown 3 4\n")
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("trace mechanism input scroll hash is", result.stderr)


if __name__ == "__main__":
    unittest.main()
