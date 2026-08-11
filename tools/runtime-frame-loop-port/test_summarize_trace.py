#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import pathlib
import tempfile
import unittest


MODULE_PATH = pathlib.Path(__file__).with_name("summarize_trace.py")
SPEC = importlib.util.spec_from_file_location("summarize_trace", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
SUMMARIZER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SUMMARIZER)


class RuntimeFrameLoopTraceSummaryTest(unittest.TestCase):
    def test_layout_landmark_uses_the_component_list_request_boundary(self) -> None:
        self.assertEqual(
            SUMMARIZER.LANDMARKS["layout_compute"]["rust"],
            {
                "source": "crates/nuxie-runtime/src/draw.rs",
                "anchor": (
                    "if self.component_list_locals().into_iter().all(|local_id| {"
                ),
                "occurrence": 1,
            },
        )

    def test_scroll_virtualizer_landmark_uses_the_retained_layout_boundary(self) -> None:
        self.assertEqual(
            SUMMARIZER.MECHANISM_LANDMARKS["scroll_virtualizer_settlements"]["rust"],
            {
                "source": (
                    "crates/nuxie-runtime/src/constraints/scrolling/"
                    "scroll_virtualizer.rs"
                ),
                "anchor": "let layout_bounds = artboard.retained_layout_bounds();",
                "occurrence": 1,
            },
        )

    def test_state_machine_landmarks_follow_the_fl_c5_owner_split(self) -> None:
        expected_owner = (
            "<nuxie_runtime::state_machine::state_machine_instance::"
            "StateMachineInstance>"
        )
        self.assertEqual(
            SUMMARIZER.LANDMARKS["state_machine_advance"]["rust"],
            f"{expected_owner}::advance_with_report_mode",
        )
        self.assertEqual(
            SUMMARIZER.LANDMARKS["event_apply_batch"]["rust"],
            f"{expected_owner}::apply_local_event_listeners",
        )
        self.assertEqual(
            SUMMARIZER.CONSTRUCTION_LANDMARKS["state_machine_instance"][
                "rust"
            ],
            f"{expected_owner}::new",
        )

    def test_exact_function_count_requires_one_match(self) -> None:
        functions = {
            "src/a.cpp": [{"name": "Owner::advance", "count": 3}],
            "src/b.cpp": [{"name": "Owner::draw", "count": 5}],
        }
        self.assertEqual(
            SUMMARIZER.exact_function_count(
                functions, ["Owner::advance", "Owner::draw"]
            ),
            8,
        )
        with self.assertRaisesRegex(ValueError, "matched 0 functions"):
            SUMMARIZER.exact_function_count(functions, "Owner::missing")

    def test_source_scope_unions_includes_and_applies_excludes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            upstream = pathlib.Path(directory)
            (upstream / "src/animation").mkdir(parents=True)
            (upstream / "src/animation/a.cpp").write_text("// a\n")
            (upstream / "src/animation/a_state.cpp").write_text("// state\n")
            ledger = {
                "source_set": [
                    {
                        "id": "animation",
                        "include": ["src/animation/*.cpp"],
                        "exclude": ["src/animation/*state*.cpp"],
                    }
                ]
            }
            scope, assignments = SUMMARIZER.source_scope(ledger, upstream)
            self.assertEqual(scope, {"src/animation/a.cpp"})
            self.assertEqual(
                assignments, {"src/animation/a.cpp": "animation"}
            )

    def test_exact_source_line_count_requires_pinned_anchor_and_segment(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "src/owner.cpp"
            source.parent.mkdir()
            source.write_text("before();\nowner.advance();\nafter();\n")
            coverage = {
                "data": [
                    {
                        "files": [
                            {
                                "filename": str(source),
                                "segments": [
                                    [1, 1, 3, True, True, False],
                                    [2, 1, 7, True, True, False],
                                    [3, 1, 3, True, True, False],
                                ],
                            }
                        ]
                    }
                ]
            }
            self.assertEqual(
                SUMMARIZER.exact_source_line_count(
                    coverage,
                    source_root=root,
                    source="src/owner.cpp",
                    anchor="owner.advance();",
                ),
                7,
            )
            with self.assertRaisesRegex(ValueError, "matched 0 lines"):
                SUMMARIZER.exact_source_line_count(
                    coverage,
                    source_root=root,
                    source="src/owner.cpp",
                    anchor="missing();",
                )

    def test_landmark_count_sums_exact_source_anchors(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            source = root / "src/owner.rs"
            source.parent.mkdir()
            source.write_text("authored();\nembedded_a();\nembedded_b();\n")
            coverage = {
                "data": [
                    {
                        "files": [
                            {
                                "filename": str(source),
                                "segments": [
                                    [1, 1, 7, True, True, False],
                                    [2, 1, 3, True, True, False],
                                    [3, 1, 2, True, True, False],
                                ],
                            }
                        ]
                    }
                ]
            }

            self.assertEqual(
                SUMMARIZER.landmark_count(
                    functions={},
                    coverage=coverage,
                    pattern={
                        "sum": [
                            {
                                "source": "src/owner.rs",
                                "anchor": "authored();",
                            },
                            {
                                "source": "src/owner.rs",
                                "anchor": "embedded_a();",
                            },
                            {
                                "source": "src/owner.rs",
                                "anchor": "embedded_b();",
                            },
                        ]
                    },
                    source_root=root,
                ),
                12,
            )

    def test_stream_counts_ignore_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            streams = pathlib.Path(directory)
            (streams / "cpp-scene.txt").write_text(
                "rive-golden-v1\n"
                "source scene.riv\n"
                "frameSize 64 64\n"
                "sample 0\n"
                "drawPath 1 2\n"
                "save\n"
                "restore\n"
            )
            self.assertEqual(
                SUMMARIZER.stream_counts(streams, "cpp"),
                {"drawPath": 1, "restore": 1, "save": 1},
            )


if __name__ == "__main__":
    unittest.main()
