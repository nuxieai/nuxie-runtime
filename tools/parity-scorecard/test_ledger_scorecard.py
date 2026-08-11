import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

from ledger_scorecard import aggregate_ledger_scorecard, render_ledger_scorecard


TOOL = Path(__file__).with_name("parity_scorecard.py")
REPO_ROOT = TOOL.parents[2]


class LedgerScorecardTests(unittest.TestCase):
    def test_aggregates_every_requested_ledger_without_reclassifying_rows(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            (repo / "docs").mkdir()
            self.write(
                repo / "file-correspondence-manifest.toml",
                """
                [[file]]
                upstream = "src/z.cpp"
                status = "faithful"
                rust_module = "crates/b.rs; crates/a.rs"

                [[file]]
                upstream = "src/audio/b.cpp"
                status = "pending"
                b6_cluster = "audio"

                [[file]]
                upstream = "src/audio/a.cpp"
                status = "pending"
                b6_cluster = "audio"

                [[file]]
                upstream = "src/layout.cpp"
                status = "divergent-by-decision"
                rust_module = "crates/c.rs"
                """,
            )
            self.write(
                repo / "rust-additions.toml",
                """
                [[addition]]
                path = "crates/e.rs"
                category = "scene-api"

                [[addition]]
                path = "crates/d.rs"
                category = "codegen"
                """,
            )
            self.write(
                repo / "test-correspondence-manifest.toml",
                """
                [[file]]
                upstream = "tests/b.cpp"
                test_case_count = 3
                status = "pending"

                [[file]]
                upstream = "tests/a.cpp"
                test_case_count = 2
                status = "ported-direct"
                """,
            )
            self.write(
                repo / "silver-corpus.toml",
                """
                [corpus]
                min_cpp_rust_exact = 1

                [[case]]
                id = "exact"
                status = "exact"
                [[case]]
                id = "divergent"
                status = "diverges"
                [[case]]
                id = "unsupported"
                status = "unsupported-feature"
                [[case]]
                id = "scripted"
                status = "pending-scripted"
                [[case]]
                id = "unknown"
                status = "provenance-unknown"
                """,
            )
            self.write(
                repo / "corpus.toml",
                """
                [[file]]
                id = "b"
                [[file]]
                id = "a"
                """,
            )
            self.write(
                repo / "docs/runtime-frame-loop-ownership.toml",
                """
                [[file]]
                upstream = "src/b.cpp"
                status = "faithful"
                [[file]]
                upstream = "src/a.cpp"
                status = "divergent-by-decision"

                [[member]]
                id = "b"
                status = "faithful"
                [[member]]
                id = "a"
                status = "adapted"
                """,
            )
            self.write(
                repo / "docs/runtime-frame-loop-gaps.toml",
                """
                [[gap]]
                id = "b"
                status = "closed"
                [[gap]]
                id = "a"
                status = "open"
                """,
            )
            self.write(
                repo / "docs/parity-gap-register.md",
                """
                ## D — Deliberate-divergence register (declare, don't fix)

                10. Later row summary. More detail is not part of the summary.
                2. Earlier row summary.
                11. **[SUPERSEDED] Old decision.** Historical detail.

                ## Additive host-extension register

                - **X2 — Later extension.** Later extension summary. More detail.
                - **X1 — Earlier extension.** Earlier extension summary.

                ## H — Housekeeping
                """,
            )

            scorecard = aggregate_ledger_scorecard(repo)

        self.assertEqual(
            scorecard["cpp_to_rust"],
            {
                "status_counts": {
                    "divergent-by-decision": 1,
                    "faithful": 1,
                    "pending": 2,
                },
                "pending_by_family": {
                    "audio": ["src/audio/a.cpp", "src/audio/b.cpp"]
                },
                "total": 4,
            },
        )
        self.assertEqual(
            scorecard["rust_to_cpp"],
            {
                "addition_category_counts": {"codegen": 1, "scene-api": 1},
                "attributed": 3,
                "classified": 2,
                "total": 5,
            },
        )
        self.assertEqual(
            scorecard["tests"],
            {
                "file_status_counts": {"pending": 1, "ported-direct": 1},
                "files": 2,
                "test_cases": 5,
            },
        )
        self.assertEqual(
            scorecard["silver"],
            {
                "min_exact": 1,
                "ratchet_met": True,
                "status_counts": {
                    "divergent": 1,
                    "exact": 1,
                    "pending-scripted": 1,
                    "provenance-unknown": 1,
                    "unsupported": 1,
                },
                "total": 5,
            },
        )
        self.assertEqual(scorecard["golden"], {"entries": 2})
        self.assertEqual(
            scorecard["frame_loop"],
            {
                "file_status_counts": {
                    "divergent-by-decision": 1,
                    "faithful": 1,
                },
                "files": 2,
                "gap_status_counts": {"closed": 1, "open": 1},
                "gaps": 2,
                "member_status_counts": {"adapted": 1, "faithful": 1},
                "members": 2,
            },
        )
        self.assertEqual(
            scorecard["d_rows"],
            [
                {"id": "D2", "summary": "Earlier row summary."},
                {"id": "D10", "summary": "Later row summary."},
            ],
        )
        self.assertEqual(
            scorecard["x_rows"],
            [
                {
                    "id": "X1",
                    "name": "Earlier extension",
                    "summary": "Earlier extension summary.",
                },
                {
                    "id": "X2",
                    "name": "Later extension",
                    "summary": "Later extension summary.",
                },
            ],
        )

    def test_render_is_terminal_friendly_sorted_and_has_no_generated_timestamp(self):
        scorecard = {
            "cpp_to_rust": {
                "status_counts": {"pending": 2, "faithful": 1},
                "pending_by_family": {
                    "z-family": ["src/z.cpp"],
                    "a-family": ["src/b.cpp", "src/a.cpp"],
                },
                "total": 3,
            },
            "rust_to_cpp": {
                "addition_category_counts": {"scene-api": 1, "codegen": 2},
                "attributed": 3,
                "classified": 3,
                "total": 6,
            },
            "tests": {
                "file_status_counts": {"pending": 1, "ported-direct": 1},
                "files": 2,
                "test_cases": 5,
            },
            "silver": {
                "min_exact": 1,
                "ratchet_met": True,
                "status_counts": {"unsupported": 1, "exact": 1},
                "total": 2,
            },
            "golden": {"entries": 2},
            "frame_loop": {
                "file_status_counts": {"faithful": 2},
                "files": 2,
                "gap_status_counts": {"open": 1, "closed": 1},
                "gaps": 2,
                "member_status_counts": {"faithful": 1, "adapted": 1},
                "members": 2,
            },
            "d_rows": [
                {"id": "D2", "summary": "Earlier."},
                {"id": "D10", "summary": "Later."},
            ],
            "x_rows": [
                {"id": "X2", "name": "Later extension", "summary": "Later."},
                {"id": "X1", "name": "Earlier extension", "summary": "Earlier."},
            ],
        }

        rendered = render_ledger_scorecard(scorecard)

        self.assertEqual(rendered, render_ledger_scorecard(scorecard))
        self.assertNotIn("generated", rendered.lower())
        self.assertNotRegex(rendered, r"\d{4}-\d{2}-\d{2}T")
        self.assertLess(rendered.index("`faithful`: 1"), rendered.index("`pending`: 2"))
        self.assertLess(rendered.index("### a-family"), rendered.index("### z-family"))
        self.assertLess(rendered.index("`src/a.cpp`"), rendered.index("`src/b.cpp`"))
        self.assertIn("Exact ratchet: 1/1 (met)", rendered)
        self.assertIn("Gaps: 2 (`closed`: 1; `open`: 1)", rendered)
        self.assertLess(rendered.index("- D2 — Earlier."), rendered.index("- D10 — Later."))
        self.assertLess(
            rendered.index("- X1 — **Earlier extension.** Earlier."),
            rendered.index("- X2 — **Later extension.** Later."),
        )

    def test_snapshot_command_prints_and_writes_the_same_document(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "parity-scorecard.md"
            completed = subprocess.run(
                [
                    sys.executable,
                    str(TOOL),
                    "snapshot",
                    "--repo-root",
                    str(REPO_ROOT),
                    "--output",
                    str(output),
                ],
                text=True,
                capture_output=True,
            )

            self.assertEqual(completed.returncode, 0, completed.stderr)
            self.assertEqual(completed.stdout, output.read_text())
            self.assertIn("## C++ → Rust file correspondence", completed.stdout)
            self.assertIn("Gaps: 10 (`closed`: 10; `open`: 0)", completed.stdout)
            self.assertIn("## D-row register", completed.stdout)
            self.assertIn("## Additive host-extension register", completed.stdout)
            self.assertIn("- X1 — **semantic-geometry-cache-authority.**", completed.stdout)
            self.assertNotIn("- D12", completed.stdout)

    @staticmethod
    def write(path: Path, contents: str) -> None:
        path.write_text(textwrap.dedent(contents).lstrip())


if __name__ == "__main__":
    unittest.main()
