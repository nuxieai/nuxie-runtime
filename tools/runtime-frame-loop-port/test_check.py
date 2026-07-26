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

    def test_owner_family_checklist_missing_adversarial_row_fails(self) -> None:
        checklist = self.repo / "docs/owner-family-closure.md"
        checklist.write_text("`src/animation/linear_animation.cpp`\n")
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
                r"let\s+(?:mut\s+)?(?:inputs|layers|transitions|conditions|listeners|actions)\s*=\s*[^;]*\.collect",
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
