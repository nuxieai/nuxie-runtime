#!/usr/bin/env python3

from __future__ import annotations

import hashlib
import json
import pathlib
import subprocess
import sys
import tempfile
import textwrap
import unittest


TOOL = pathlib.Path(__file__).with_name("check.py")
TOOL_DIR = pathlib.Path(__file__).resolve().parent
if str(TOOL_DIR) not in sys.path:
    sys.path.insert(0, str(TOOL_DIR))

from source_fingerprint import (
    candidate_source_fingerprint,
    rust_runner_provenance,
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
