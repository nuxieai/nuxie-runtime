import json
import subprocess
import sys
import shutil
import tempfile
import textwrap
import tomllib
import unittest
from pathlib import Path

from ledger_scorecard import (
    aggregate_ledger_scorecard,
    render_ledger_scorecard,
    resolve_evidence_record,
)


TOOL = Path(__file__).with_name("parity_scorecard.py")
REPO_ROOT = TOOL.parents[2]


class LedgerScorecardTests(unittest.TestCase):
    def test_pending_verification_is_not_reported_as_behaviorally_proven(self):
        scorecard = aggregate_ledger_scorecard(REPO_ROOT)

        proof = scorecard["owner_proofs"]["by_upstream"][
            "src/assets/audio_asset.cpp"
        ]
        self.assertEqual(proof["mapping"], "mapped")
        self.assertEqual(proof["behavioral"], "unverified")
        self.assertEqual(proof["effective_state"], "stale")

    def test_structural_divergence_overrides_verified_file_correspondence(self):
        scorecard = aggregate_ledger_scorecard(REPO_ROOT)

        proof = scorecard["owner_proofs"]["by_upstream"]["src/artboard.cpp"]
        self.assertEqual(proof["mapping"], "mapped")
        self.assertEqual(proof["structural"], "divergent")
        self.assertEqual(proof["behavioral"], "unverified")
        self.assertEqual(proof["effective_state"], "stale")

    def test_verified_classification_is_not_behavioral_proof(self):
        scorecard = aggregate_ledger_scorecard(REPO_ROOT)

        proof = scorecard["owner_proofs"]["by_upstream"][
            "src/advancing_component.cpp"
        ]
        self.assertEqual(proof["verification"], "orchestrator-verified")
        self.assertEqual(proof["behavioral"], "unverified")

    def test_decisions_and_extensions_are_independent_owner_dimensions(self):
        scorecard = aggregate_ledger_scorecard(REPO_ROOT)

        proof = scorecard["owner_proofs"]["by_upstream"][
            "src/lua/renderer/lua_gpu.cpp"
        ]
        self.assertEqual(proof["decisions"], ["D18"])
        self.assertEqual(proof["extensions"], ["X3"])
        self.assertEqual(proof["exception"], "intentional-extension")
        self.assertEqual(proof["behavioral"], "unverified")

    def test_owner_freshness_honors_current_post_audit_rows(self):
        scorecard = aggregate_ledger_scorecard(REPO_ROOT)

        self.assertEqual(
            scorecard["owner_proofs"]["freshness_counts"],
            {"current": 10, "stale": 446},
        )
        current = scorecard["owner_proofs"]["by_upstream"][
            "src/animation/keyframe_int.cpp"
        ]
        self.assertEqual(current["freshness"], "current")
        self.assertEqual(
            current["freshness_basis"], "row-audit-pin-and-rust-source"
        )
        self.assertEqual(
            current["reviewed_rust_source_sha256"],
            "1a7d9862c077ed42f765aa6b2bef7c8a5a80a405134531f6c2ae7ab8424956f5",
        )
        self.assertTrue(
            all(
                proof["audit_record"]
                for proof in scorecard["owner_proofs"]["by_upstream"].values()
            )
        )

    def test_each_non_proven_dimension_has_an_actionable_owner_list(self):
        scorecard = aggregate_ledger_scorecard(REPO_ROOT)

        lists = scorecard["owner_proofs"]["non_proven_by_dimension"]
        self.assertEqual(len(lists["stale"]), 446)
        self.assertEqual(len(lists["behaviorally-unverified"]), 456)
        self.assertEqual(len(lists["known-divergent"]), 186)
        self.assertEqual(len(lists["incomplete-mapping"]), 5)
        self.assertIn(
            "src/constraints/scrolling/elastic_scroll_physics.cpp",
            lists["incomplete-mapping"],
        )

    def test_missing_owner_verification_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            self.copy_current_inputs(repo)
            manifest = repo / "file-correspondence-manifest.toml"
            manifest.write_text(
                manifest.read_text().replace(
                    'verification = "orchestrator-verified"\n', "", 1
                )
            )

            with self.assertRaisesRegex(
                ValueError, "src/advancing_component.cpp.*missing verification"
            ):
                aggregate_ledger_scorecard(repo)

    def test_faithful_owner_requires_a_rust_mapping(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            self.copy_current_inputs(repo)
            manifest = repo / "file-correspondence-manifest.toml"
            manifest.write_text(
                manifest.read_text().replace(
                    'rust_module = "crates/nuxie-runtime/src/lib.rs; crates/nuxie-runtime/src/artboard.rs; crates/nuxie-runtime/src/artboard/advancing_component.rs"',
                    'rust_module = ""',
                    1,
                )
            )

            with self.assertRaisesRegex(
                ValueError, "src/advancing_component.cpp.*missing Rust owner mapping"
            ):
                aggregate_ledger_scorecard(repo)

    def test_duplicate_upstream_owner_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            self.copy_current_inputs(repo)
            manifest = repo / "file-correspondence-manifest.toml"
            manifest.write_text(
                manifest.read_text().replace(
                    'upstream = "src/animation/animation_reset.cpp"',
                    'upstream = "src/advancing_component.cpp"',
                    1,
                )
            )

            with self.assertRaisesRegex(
                ValueError, "duplicate upstream owner src/advancing_component.cpp"
            ):
                aggregate_ledger_scorecard(repo)

    def test_invalid_structural_verdict_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            self.copy_current_inputs(repo)
            manifest = repo / "file-correspondence-manifest.toml"
            manifest.write_text(
                manifest.read_text().replace(
                    'b6_verdict = "ADAPTED"', 'b6_verdict = "HAND-WAVED"', 1
                )
            )

            with self.assertRaisesRegex(
                ValueError, "src/advancing_component.cpp.*invalid structural verdict"
            ):
                aggregate_ledger_scorecard(repo)

    def test_missing_structural_evidence_record_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            self.copy_current_inputs(repo)
            manifest = repo / "file-correspondence-manifest.toml"
            manifest.write_text(
                manifest.read_text().replace(
                    'audit_record = "docs/b6-audit/results/misc-core.md"',
                    'audit_record = "docs/b6-audit/results/missing.md"',
                    1,
                )
            )

            with self.assertRaisesRegex(
                ValueError, "src/advancing_component.cpp.*evidence record does not exist"
            ):
                aggregate_ledger_scorecard(repo)

    def test_structural_evidence_must_name_its_b6_row(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            self.copy_current_inputs(repo)
            manifest = repo / "file-correspondence-manifest.toml"
            manifest.write_text(
                manifest.read_text().replace(
                    'audit_record = "docs/b6-audit/results/misc-core.md"',
                    'audit_record = "docs/b6-audit/results/unavailable.md"',
                    1,
                )
            )

            with self.assertRaisesRegex(
                ValueError, "src/advancing_component.cpp.*does not contain B6-0001"
            ):
                aggregate_ledger_scorecard(repo)

    def test_structural_evidence_verdict_must_match_the_claim(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            self.copy_current_inputs(repo)
            evidence = repo / "docs/b6-audit/results/post-b6-port-reviews.md"
            evidence.write_text(
                evidence.read_text().replace(
                    'row_id: B6-0146; cpp_files: ["src/core.cpp"]; verdict: ADAPTED',
                    'row_id: B6-0146; cpp_files: ["src/core.cpp"]; verdict: DIVERGENT',
                )
            )

            with self.assertRaisesRegex(
                ValueError, "src/core.cpp.*does not substantiate ADAPTED"
            ):
                aggregate_ledger_scorecard(repo)

    def test_deleted_review_report_resolves_to_git_history(self):
        contents, locator, local = resolve_evidence_record(REPO_ROOT, "P3B-report.md")

        self.assertFalse(local)
        self.assertIsNotNone(contents)
        self.assertRegex(locator, r"^git:[0-9a-f]{40}:P3B-report\.md$")

    def test_current_audit_row_must_cite_the_live_upstream_pin(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            self.copy_current_inputs(repo)
            manifest = repo / "file-correspondence-manifest.toml"
            manifest.write_text(
                manifest.read_text().replace(
                    'current_audit_rows = [',
                    'current_audit_rows = ["B6-0001", ',
                    1,
                )
            )

            with self.assertRaisesRegex(
                ValueError, "src/advancing_component.cpp.*current audit record does not cite 4ac7b327"
            ):
                aggregate_ledger_scorecard(repo)

    def test_current_audit_row_must_match_reviewed_rust_sources(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            self.copy_current_inputs(repo)
            reviewed_source = repo / "crates/nuxie-runtime/src/animation/keyframe_int.rs"
            reviewed_source.write_text(reviewed_source.read_text() + "\n// drift\n")

            with self.assertRaisesRegex(
                ValueError,
                "src/animation/keyframe_int.cpp.*reviewed Rust source fingerprint mismatch",
            ):
                aggregate_ledger_scorecard(repo)

    def test_second_pass_verdict_must_match_the_claim(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            self.copy_current_inputs(repo)
            evidence = repo / "docs/b6-audit/SECOND_PASS.md"
            evidence.write_text(
                evidence.read_text().replace(
                    "| B6-0027 | `focus_listener_group.cpp` | TRACKED-GAP |",
                    "| B6-0027 | `focus_listener_group.cpp` | ADAPTED |",
                )
            )

            with self.assertRaisesRegex(
                ValueError,
                "src/animation/focus_listener_group.cpp.*does not substantiate TRACKED-GAP",
            ):
                aggregate_ledger_scorecard(repo)

    def test_unknown_decision_reference_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            self.copy_current_inputs(repo)
            manifest = repo / "file-correspondence-manifest.toml"
            manifest.write_text(
                manifest.read_text().replace("registered D18:", "registered D999:")
            )

            with self.assertRaisesRegex(
                ValueError, "src/lua/renderer/lua_gpu.cpp.*unknown decision D999"
            ):
                aggregate_ledger_scorecard(repo)

    def test_declared_owner_count_must_match_manifest_rows(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            self.copy_current_inputs(repo)
            manifest = repo / "file-correspondence-manifest.toml"
            manifest.write_text(manifest.read_text().replace("row_count = 456", "row_count = 455"))

            with self.assertRaisesRegex(
                ValueError, "declares 455 owner rows but contains 456"
            ):
                aggregate_ledger_scorecard(repo)

    def test_unknown_correspondence_status_is_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            self.copy_current_inputs(repo)
            manifest = repo / "file-correspondence-manifest.toml"
            manifest.write_text(
                manifest.read_text().replace(
                    'status = "faithful"', 'status = "claimed-faithful"', 1
                )
            )

            with self.assertRaisesRegex(
                ValueError, "src/advancing_component.cpp.*invalid correspondence status"
            ):
                aggregate_ledger_scorecard(repo)

    def test_aggregates_every_requested_ledger_without_reclassifying_rows(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            (repo / "docs").mkdir()
            self.write(
                repo / "file-correspondence-manifest.toml",
                """
                schema = "nuxie-file-correspondence/v1"
                upstream_ref = "test-upstream"
                audit_upstream_ref = "test-upstream"
                row_count = 4
                status_values = ["faithful", "divergent-by-decision", "pending"]
                verification_values = ["pending-verification", "orchestrator-verified"]
                audit_verdict_values = ["ISOMORPHIC", "ADAPTED", "DIVERGENT", "TRACKED-GAP", "N/A"]

                [[file]]
                upstream = "src/z.cpp"
                status = "faithful"
                verification = "orchestrator-verified"
                rust_module = "crates/b.rs; crates/a.rs"
                b6_row_id = "B6-0001"
                b6_verdict = "ISOMORPHIC"
                b6_cluster = "test"
                audit_record = "docs/audit.md"
                note = "Direct mapped owner."

                [[file]]
                upstream = "src/audio/b.cpp"
                status = "pending"
                verification = "pending-verification"
                rust_module = ""
                b6_row_id = "B6-0002"
                b6_verdict = "N/A"
                b6_cluster = "audio"
                audit_record = "docs/audit.md"
                note = "Pending owner."

                [[file]]
                upstream = "src/audio/a.cpp"
                status = "pending"
                verification = "pending-verification"
                rust_module = ""
                b6_row_id = "B6-0003"
                b6_verdict = "N/A"
                b6_cluster = "audio"
                audit_record = "docs/audit.md"
                note = "Pending owner."

                [[file]]
                upstream = "src/layout.cpp"
                status = "divergent-by-decision"
                verification = "orchestrator-verified"
                rust_module = "crates/c.rs"
                b6_row_id = "B6-0004"
                b6_verdict = "DIVERGENT"
                b6_cluster = "test"
                audit_record = "docs/audit.md"
                note = "Covered by D2."
                """,
            )
            self.write(
                repo / "docs/audit.md",
                """
                - row_id: B6-0001; cpp_files: ["src/z.cpp"]; verdict: ISOMORPHIC
                - row_id: B6-0002; cpp_files: ["src/audio/b.cpp"]; verdict: N/A
                - row_id: B6-0003; cpp_files: ["src/audio/a.cpp"]; verdict: N/A
                - row_id: B6-0004; cpp_files: ["src/layout.cpp"]; verdict: DIVERGENT
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
                status = "exact"
                [[file]]
                id = "a"
                status = "exact"
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
                "covered_test_cases": 2,
                "uncovered_test_cases": 3,
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
        self.assertEqual(
            scorecard["golden"], {"entries": 2, "status_counts": {"exact": 2}}
        )
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
                "covered_test_cases": 2,
                "uncovered_test_cases": 3,
            },
            "silver": {
                "min_exact": 1,
                "ratchet_met": True,
                "status_counts": {"unsupported": 1, "exact": 1},
                "total": 2,
            },
            "golden": {"entries": 2, "status_counts": {"exact": 2}},
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
            json_output = Path(temporary) / "parity-owner-proofs.json"
            completed = subprocess.run(
                [
                    sys.executable,
                    str(TOOL),
                    "snapshot",
                    "--repo-root",
                    str(REPO_ROOT),
                    "--output",
                    str(output),
                    "--json",
                    str(json_output),
                ],
                text=True,
                capture_output=True,
            )

            self.assertEqual(completed.returncode, 0, completed.stderr)
            self.assertEqual(completed.stdout, output.read_text())
            self.assertIn(
                "## C++ → Rust correspondence inputs (non-authoritative)",
                completed.stdout,
            )
            self.assertIn("## C++ → Rust owner proof", completed.stdout)
            self.assertIn("Effective proof states:", completed.stdout)
            self.assertIn("Structural states:", completed.stdout)
            self.assertIn("Behavioral states:", completed.stdout)
            self.assertIn("Exception states:", completed.stdout)
            self.assertIn("### stale owners (446)", completed.stdout)
            self.assertIn("### behaviorally-unverified owners (456)", completed.stdout)
            self.assertIn("### known-divergent owners (186)", completed.stdout)
            self.assertIn("### incomplete-mapping owners (5)", completed.stdout)
            self.assertIn("`src/artboard.cpp`", completed.stdout)
            self.assertIn("`src/lua/renderer/lua_gpu.cpp`", completed.stdout)
            self.assertIn("Behavioral states: `unverified`: 456", completed.stdout)
            self.assertIn("Covered test cases: 655/1404", completed.stdout)
            self.assertIn("Uncovered test cases: 749", completed.stdout)
            self.assertIn(
                "Status counts: `diverges`: 10; `exact`: 349; `not-yet`: 5",
                completed.stdout,
            )
            self.assertIn("Gaps: 10 (`closed`: 10; `open`: 0)", completed.stdout)
            self.assertIn("## D-row register", completed.stdout)
            self.assertIn("## Additive host-extension register", completed.stdout)
            self.assertIn("- X1 — **semantic-geometry-cache-authority.**", completed.stdout)
            self.assertNotIn("- D12", completed.stdout)
            proof_document = json.loads(json_output.read_text())
            self.assertEqual(proof_document["schema"], "nuxie-owner-parity-proof/v1")
            self.assertEqual(
                proof_document["upstream_ref"],
                "4ac7b32798da0482e441ef09304dc3b480ed3ee5",
            )
            self.assertEqual(len(proof_document["owners"]), 456)
            self.assertEqual(
                proof_document["owners"],
                sorted(proof_document["owners"], key=lambda row: row["upstream"]),
            )
            self.assertEqual(
                proof_document["summary"]["effective_state_counts"],
                {"known-divergent": 3, "stale": 446, "unverified": 7},
            )
            self.assertEqual(
                proof_document["evidence_dimensions"]["tests"][
                    "uncovered_test_cases"
                ],
                749,
            )

    def test_make_snapshot_persists_markdown_and_machine_owner_proof(self):
        makefile = (REPO_ROOT / "Makefile").read_text()

        self.assertIn(
            "PARITY_OWNER_PROOF_DOC ?= $(CURDIR)/docs/parity-owner-proofs.json",
            makefile,
        )
        self.assertIn(
            '--output "$(PARITY_SCORECARD_DOC)" --json "$(PARITY_OWNER_PROOF_DOC)"',
            makefile,
        )

    def test_make_gate_rejects_stale_human_or_machine_snapshots(self):
        makefile = (REPO_ROOT / "Makefile").read_text()

        self.assertIn("parity-scorecard-check:", makefile)
        self.assertIn(
            'cmp "$(PARITY_SCORECARD_GENERATED_DOC)" "$(PARITY_SCORECARD_DOC)"',
            makefile,
        )
        self.assertIn(
            'cmp "$(PARITY_OWNER_PROOF_GENERATED_DOC)" "$(PARITY_OWNER_PROOF_DOC)"',
            makefile,
        )
        self.assertIn(
            '"parity scorecard snapshot freshness" "$(MAKE) --no-print-directory parity-scorecard-check"',
            makefile,
        )

    def test_ci_runs_proof_aware_parity_scorecard_gate(self):
        workflow = (REPO_ROOT / ".github" / "workflows" / "ci.yml").read_text()

        self.assertIn("- name: Proof-aware parity scorecard gate", workflow)
        self.assertIn("run: make parity-scorecard", workflow)

    @staticmethod
    def write(path: Path, contents: str) -> None:
        path.write_text(textwrap.dedent(contents).lstrip())

    @staticmethod
    def copy_current_inputs(destination: Path) -> None:
        destination.mkdir(exist_ok=True)
        for name in (
            "file-correspondence-manifest.toml",
            "rust-additions.toml",
            "test-correspondence-manifest.toml",
            "silver-corpus.toml",
            "corpus.toml",
        ):
            shutil.copy2(REPO_ROOT / name, destination / name)
        (destination / "docs").mkdir()
        for name in (
            "runtime-frame-loop-ownership.toml",
            "runtime-frame-loop-gaps.toml",
            "parity-gap-register.md",
        ):
            shutil.copy2(REPO_ROOT / "docs" / name, destination / "docs" / name)
        shutil.copytree(
            REPO_ROOT / "docs" / "b6-audit", destination / "docs" / "b6-audit"
        )
        manifest = tomllib.loads(
            (REPO_ROOT / "file-correspondence-manifest.toml").read_text()
        )
        current_rows = set(manifest["current_audit_rows"])
        for row in manifest["file"]:
            if row.get("b6_row_id") not in current_rows:
                continue
            for source in row["rust_module"].split(";"):
                relative = Path(source.strip())
                target = destination / relative
                target.parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(REPO_ROOT / relative, target)
        for report in (
            "LUABIND-report.md",
            "P1G-report.md",
            "P2A-report.md",
            "P2B-report.md",
            "P3B-report.md",
        ):
            contents, _, _ = resolve_evidence_record(REPO_ROOT, report)
            assert contents is not None
            (destination / report).write_text(contents)


if __name__ == "__main__":
    unittest.main()
