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
