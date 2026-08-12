import collections
import hashlib
import json
import re
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

from drift_queue import recent_touch_counts, tracked_gap_candidates


TOOL = Path(__file__).with_name("drift_queue.py")
REPO_ROOT = TOOL.parents[2]


class RuntimeDriftQueueTests(unittest.TestCase):
    def run_build(self, *extra_args: str) -> dict:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "queue.json"
            subprocess.run(
                [
                    sys.executable,
                    str(TOOL),
                    "build",
                    "--repo-root",
                    str(REPO_ROOT),
                    "--output",
                    str(output),
                    *extra_args,
                ],
                check=True,
            )
            return json.loads(output.read_text())

    def differential_report(
        self, lane: str, cases: list[dict], rust_commit: str
    ) -> dict:
        manifest = "silver-corpus.toml" if lane == "silver" else "corpus.toml"
        manifest_path = REPO_ROOT / manifest
        runner_roles = ["validator"] if lane == "silver" else ["cpp", "rust"]
        overrides = {case["id"]: case for case in cases}
        with manifest_path.open("rb") as handle:
            parsed = tomllib.load(handle)
        source_rows = parsed["case" if lane == "silver" else "file"]
        normalized_cases = []
        for source_row in source_rows:
            case_id = source_row["id"]
            declared_status = source_row["status"]
            if lane == "golden-scripted":
                features = source_row.get("features", [])
                if "scripted-status:exact" in features:
                    declared_status = "exact"
                elif "scripted-status:diverges" in features:
                    declared_status = "diverges"
            default_outcome = {
                "exact": "exact",
                "diverges": "divergent",
                "unsupported-feature": "unsupported",
                "provenance-unknown": "unsupported",
                "not-yet": "pending",
                "pending": "pending",
                "pending-scripted": "pending",
            }[declared_status]
            case = overrides.get(
                case_id,
                {
                    "id": case_id,
                    "outcome": default_outcome,
                    "executed": default_outcome in {"exact", "divergent"},
                },
            )
            record = {
                "declared_status": declared_status,
                "executed": False,
                "fixture": {
                    "path": source_row.get("source", source_row.get("path")),
                    "sha256": "1" * 64,
                },
                "id": case["id"],
                "outcome": case["outcome"],
                "verification": "exact",
                **case,
            }
            for script_field in ("input_script", "view_model_script"):
                if script_field in source_row:
                    record[script_field] = {
                        "path": source_row[script_field],
                        "sha256": "4" * 64,
                    }
            if lane == "silver":
                action_fixtures = [
                    action["source"]
                    for action in source_row.get("actions", [])
                    if isinstance(action, dict)
                    and action.get("kind") == "set-view-model-font-bytes"
                ]
                record.update(
                    {
                        "baseline": {
                            "path": source_row["expected"],
                            "sha256": "2" * 64,
                        },
                        "dependencies": [
                            {"path": dependency, "sha256": "5" * 64}
                            for dependency in source_row.get("dependencies", [])
                        ],
                        "action_fixtures": [
                            {"path": fixture, "sha256": "6" * 64}
                            for fixture in action_fixtures
                        ],
                        "lane": source_row["lane"],
                    }
                )
            normalized_cases.append(record)
        report = {
            "schema": "nuxie-runtime-differentials/v1",
            "lane": lane,
            "cpp_ref": json.loads(
                (REPO_ROOT / "docs/parity-owner-proofs.json").read_text()
            )["upstream_ref"],
            "rust_commit": rust_commit,
            "gate_status": "failed",
            "manifest": {
                "path": manifest,
                "sha256": hashlib.sha256(manifest_path.read_bytes()).hexdigest(),
            },
            "runners": [
                {"role": role, "path": f"/unavailable/{role}", "sha256": "3" * 64}
                for role in runner_roles
            ],
            "summary": dict(
                sorted(collections.Counter(case["outcome"] for case in normalized_cases).items())
            ),
            "cases": normalized_cases,
        }
        return report

    def test_every_owner_proof_row_is_partitioned_once(self):
        report = self.run_build()

        owner_accounting = report["accounting"]["owner-proofs"]
        self.assertEqual(owner_accounting["rows"], 456)
        self.assertEqual(
            owner_accounting["candidates"] + owner_accounting["proven"],
            owner_accounting["rows"],
        )
        owner_candidates = [
            candidate
            for candidate in report["candidates"]
            if candidate["source_kind"] == "owner-proof"
        ]
        self.assertEqual(len(owner_candidates), owner_accounting["candidates"])
        self.assertEqual(
            len(owner_accounting["proven_rows"]), owner_accounting["proven"]
        )
        self.assertFalse(
            {candidate["source_row"] for candidate in owner_candidates}
            & set(owner_accounting["proven_rows"])
        )
        self.assertEqual(
            len({candidate["id"] for candidate in owner_candidates}),
            len(owner_candidates),
        )
        self.assertIn(
            "owner:src/artboard.cpp",
            {candidate["id"] for candidate in owner_candidates},
        )
        by_id = {candidate["id"]: candidate for candidate in owner_candidates}
        animation = by_id["owner:src/animation/state_machine.cpp"]
        self.assertEqual(animation["owner_family"], "animation")
        self.assertIn("behavioral proof is unverified", animation["missing_proofs"])
        self.assertIn("freshness is stale", animation["missing_proofs"])
        self.assertIn("behavioral proof is unverified", animation["first_signal"])
        current = by_id["owner:src/layout/grid_item_placement.cpp"]
        self.assertIn("structural proof is divergent", current["missing_proofs"])
        self.assertIn("behavioral proof is unverified", current["missing_proofs"])
        self.assertNotIn("fingerprint match", current["first_signal"])
        self.assertIn(current["churn_freshness"]["churn"], {"low", "medium", "high"})
        self.assertIsInstance(current["churn_freshness"]["recent_touch_count"], int)
        source_rows = {
            owner["upstream"]
            for owner in json.loads(
                (REPO_ROOT / "docs/parity-owner-proofs.json").read_text()
            )["owners"]
        }
        self.assertEqual(
            source_rows,
            {candidate["source_row"] for candidate in owner_candidates}
            | set(owner_accounting["proven_rows"]),
        )

    def test_every_test_and_differential_manifest_row_is_partitioned_once(self):
        report = self.run_build()

        expected_rows = {
            "upstream-tests": 157,
            "golden": 364,
            "silver": 252,
        }
        source_files = {
            "upstream-tests": ("test-correspondence-manifest.toml", "file", "upstream"),
            "golden": ("corpus.toml", "file", "id"),
            "silver": ("silver-corpus.toml", "case", "id"),
        }
        for source_kind, row_count in expected_rows.items():
            accounting = report["accounting"][source_kind]
            self.assertEqual(accounting["rows"], row_count)
            self.assertEqual(
                accounting["candidates"] + accounting["proven"],
                accounting["rows"],
            )
            candidates = [
                candidate
                for candidate in report["candidates"]
                if candidate["source_kind"] == source_kind
            ]
            self.assertEqual(len(candidates), accounting["candidates"])
            self.assertEqual(len(accounting["proven_rows"]), accounting["proven"])
            self.assertFalse(
                {candidate["source_row"] for candidate in candidates}
                & set(accounting["proven_rows"])
            )
            self.assertEqual(
                len({candidate["source_row"] for candidate in candidates}),
                len(candidates),
            )
            manifest, table, id_field = source_files[source_kind]
            with (REPO_ROOT / manifest).open("rb") as handle:
                source_rows = {
                    row[id_field] for row in tomllib.load(handle)[table]
                }
            self.assertEqual(
                source_rows,
                {candidate["source_row"] for candidate in candidates}
                | set(accounting["proven_rows"]),
            )

    def test_candidates_are_clustered_and_prioritized_without_collapsing_dispositions(self):
        report = self.run_build()

        required = {
            "id",
            "source_kind",
            "source_row",
            "upstream_owner",
            "owner_state",
            "evidence_links",
            "first_signal",
            "confidence",
            "product_reach",
            "churn_freshness",
            "disposition",
            "evidence_state",
            "semantic_boundary",
            "owner_family",
            "subsystem",
            "discovery_value",
            "cluster_id",
        }
        for candidate in report["candidates"]:
            self.assertTrue(required <= candidate.keys(), candidate["id"])
            self.assertTrue(candidate["evidence_links"], candidate["id"])
            self.assertTrue(candidate["first_signal"], candidate["id"])

        self.assertTrue(
            {
                "known-divergence",
                "intentional-decision",
                "extension",
                "unsupported",
                "stale-proof",
                "unknown",
                "pending-proof",
            }
            <= {candidate["disposition"] for candidate in report["candidates"]}
        )
        self.assertEqual(
            report["candidates"],
            sorted(
                report["candidates"],
                key=lambda candidate: (-candidate["discovery_value"], candidate["id"]),
            ),
        )
        cluster_members = [
            candidate_id
            for cluster in report["clusters"]
            for candidate_id in cluster["candidate_ids"]
        ]
        self.assertCountEqual(
            cluster_members,
            [candidate["id"] for candidate in report["candidates"]],
        )
        self.assertEqual(len(cluster_members), len(set(cluster_members)))
        for filter_name, field in {
            "owner_families": "owner_family",
            "subsystems": "subsystem",
            "evidence_states": "evidence_state",
            "dispositions": "disposition",
        }.items():
            self.assertEqual(
                report["filters"][filter_name],
                sorted({candidate[field] for candidate in report["candidates"]}),
            )
        dimensions = json.loads(
            (REPO_ROOT / "docs/parity-owner-proofs.json").read_text()
        )["evidence_dimensions"]
        expected_intentional = {
            f"decision:{row['id']}" for row in dimensions["decisions"]
        } | {f"extension:{row['id']}" for row in dimensions["extensions"]}
        self.assertEqual(
            expected_intentional,
            {
                candidate["id"]
                for candidate in report["candidates"]
                if candidate["source_kind"] in {"decision", "extension"}
            },
        )

    def test_every_tracked_gap_row_is_partitioned_and_open_rows_stay_visible(self):
        report = self.run_build()

        accounting = report["accounting"]["tracked-gaps"]
        self.assertEqual(accounting["rows"], 79)
        self.assertEqual(
            accounting["candidates"] + accounting["proven"], accounting["rows"]
        )
        gap_candidates = {
            candidate["source_row"]: candidate
            for candidate in report["candidates"]
            if candidate["source_kind"] == "tracked-gap"
        }
        self.assertIn("V1", gap_candidates)
        self.assertIn("F4", gap_candidates)
        self.assertIn("RB-2", gap_candidates)
        for partial in ("V2", "V16", "V24", "V26", "V31"):
            self.assertIn(partial, gap_candidates)
        self.assertNotIn("V5", gap_candidates)
        self.assertNotIn("RB-3", gap_candidates)
        self.assertEqual(gap_candidates["F10"]["disposition"], "unknown")
        self.assertEqual(len(accounting["proven_rows"]), accounting["proven"])
        self.assertFalse(set(gap_candidates) & set(accounting["proven_rows"]))
        source_rows = {
            match.group(1)
            for line in (REPO_ROOT / "docs/parity-gap-register.md").read_text().splitlines()
            if (match := re.match(r"^\|\s*((?:V|F|A|C|H|W)\d+|RB-\d+)\s*\|", line))
        }
        self.assertEqual(
            source_rows, set(gap_candidates) | set(accounting["proven_rows"])
        )

    def test_tracked_gap_status_uses_family_specific_cells(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            docs = repo / "docs"
            docs.mkdir()
            (docs / "parity-gap-register.md").write_text(
                "\n".join(
                    (
                        "| F1 | partial feature | 10 | PARTIAL | absent dependency remains |",
                        "| F2 | deferred feature | 10 | DEFERRED (approved) | later |",
                        "| F3 | unknown feature | 10 | UNKNOWN | investigate |",
                        "| C1 | CLOSED — coverage complete | keep green |",
                        "| W1 | watch item | RESOLVED | upstream fixed |",
                        "| H1 | history mentions deferred work |",
                    )
                )
                + "\n"
            )
            candidates, accounting = tracked_gap_candidates(repo)

        by_id = {row["id"]: row for row in candidates}
        self.assertEqual(by_id["gap:F1"]["evidence_state"], "tracked-gap")
        self.assertEqual(by_id["gap:F2"]["evidence_state"], "unsupported-feature")
        self.assertEqual(by_id["gap:F3"]["evidence_state"], "unknown")
        self.assertEqual(by_id["gap:H1"]["evidence_state"], "tracked-gap")
        self.assertEqual(accounting["proven_rows"], ["C1", "W1"])

    def test_fresh_differential_artifacts_enrich_existing_rows_and_add_regressions(self):
        with tempfile.TemporaryDirectory() as temporary:
            artifacts = Path(temporary)
            rust_commit = subprocess.run(
                ["git", "-C", str(REPO_ROOT), "rev-parse", "HEAD"],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            (artifacts / "golden-ordinary.json").write_text(
                json.dumps(
                    self.differential_report(
                        "golden-ordinary",
                        [
                            {
                                "id": "echo_show_demo",
                                "outcome": "divergent",
                                "executed": True,
                                "signature": "fresh first difference",
                                "divergence_check": "verified",
                            },
                            {
                                "id": "advance_blend_mode",
                                "outcome": "regressed",
                                "executed": True,
                                "diagnostic": "advance_blend_mode: fresh regression",
                            },
                            {
                                "id": "editor_scripted_vector_v7",
                                "outcome": "regressed",
                                "executed": True,
                                "diagnostic": "ordinary lane regression",
                            },
                        ],
                        rust_commit,
                    )
                )
            )
            scripted = self.differential_report(
                "golden-scripted",
                [
                    {
                        "id": "editor_scripted_vector_v7",
                        "outcome": "divergent",
                        "executed": True,
                        "signature": "known scripted divergence",
                    }
                ],
                rust_commit,
            )
            next(
                case
                for case in scripted["cases"]
                if case["id"] == "editor_scripted_vector_v7"
            )["declared_status"] = "diverges"
            scripted["summary"] = dict(
                sorted(
                    collections.Counter(
                        case["outcome"] for case in scripted["cases"]
                    ).items()
                )
            )
            (artifacts / "golden-scripted.json").write_text(json.dumps(scripted))
            (artifacts / "silver.json").write_text(
                json.dumps(
                    self.differential_report(
                        "silver",
                        [
                            {
                                "id": "advance_blend_mode-inputs",
                                "outcome": "regressed",
                                "executed": True,
                                "diagnostic": "silver fresh regression",
                            }
                        ],
                        rust_commit,
                    )
                )
            )

            report = self.run_build("--differential-dir", str(artifacts))

        by_id = {candidate["id"]: candidate for candidate in report["candidates"]}
        self.assertEqual(by_id["golden:echo_show_demo"]["confidence"], "high")
        self.assertEqual(
            by_id["golden:echo_show_demo"]["first_signal"],
            "fresh first difference",
        )
        self.assertEqual(
            by_id["golden:echo_show_demo"]["churn_freshness"]["state"],
            "runtime-artifact-current",
        )
        self.assertEqual(
            by_id["golden:advance_blend_mode"]["disposition"],
            "known-divergence",
        )
        silver_regression = by_id["silver:advance_blend_mode-inputs"]
        self.assertEqual(silver_regression["confidence"], "high")
        self.assertEqual(silver_regression["owner_state"], "resolved")
        self.assertEqual(
            silver_regression["upstream_owner"],
            "tests/unit_tests/runtime/serialized_rendering_test.cpp",
        )
        self.assertEqual(silver_regression["owner_family"], "runtime-tests")
        self.assertIn("silver-corpus.toml", silver_regression["evidence_links"])
        self.assertIn(
            "tests/unit_tests/silvers/advance_blend_mode-inputs.sriv",
            silver_regression["evidence_links"],
        )
        cross_lane = by_id["golden:editor_scripted_vector_v7"]
        self.assertEqual(cross_lane["evidence_state"], "regressed")
        self.assertEqual(cross_lane["first_signal"], "ordinary lane regression")
        self.assertEqual(
            {item["lane"] for item in cross_lane["differential_observations"]},
            {"golden-ordinary", "golden-scripted"},
        )
        self.assertEqual(
            report["accounting"]["golden"],
            {
                "rows": 364,
                "candidates": 12,
                "proven": 352,
                "proven_rows": report["accounting"]["golden"]["proven_rows"],
            },
        )
        self.assertNotIn(
            "advance_blend_mode",
            report["accounting"]["golden"]["proven_rows"],
        )
        self.assertEqual(report["differential_artifacts"][0]["status"], "accepted")

    def test_unexecuted_artifact_rows_do_not_claim_current_runtime_observation(self):
        with tempfile.TemporaryDirectory() as temporary:
            artifacts = Path(temporary)
            rust_commit = subprocess.run(
                ["git", "-C", str(REPO_ROOT), "rev-parse", "HEAD"],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            (artifacts / "silver.json").write_text(
                json.dumps(
                    self.differential_report(
                        "silver",
                        [
                            {
                                "id": "listener_action_inputs",
                                "outcome": "pending",
                                "executed": False,
                            }
                        ],
                        rust_commit,
                    )
                )
            )
            report = self.run_build("--differential-dir", str(artifacts))

        row = next(
            candidate
            for candidate in report["candidates"]
            if candidate["id"] == "silver:listener_action_inputs"
        )
        self.assertEqual(row["evidence_state"], "pending-scripted")
        self.assertEqual(row["churn_freshness"]["state"], "manifest-current")
        self.assertIn("Scripted producer provenance", row["first_signal"])

    def test_sparse_or_unknown_differential_artifacts_fail_closed(self):
        with tempfile.TemporaryDirectory() as temporary:
            artifacts = Path(temporary)
            sparse = {
                "schema": "nuxie-runtime-differentials/v1",
                "lane": "golden-ordinary",
                "cpp_ref": "0" * 40,
                "rust_commit": "0" * 40,
                "cases": [],
            }
            (artifacts / "sparse.json").write_text(json.dumps(sparse))
            failed = subprocess.run(
                [
                    sys.executable,
                    str(TOOL),
                    "build",
                    "--repo-root",
                    str(REPO_ROOT),
                    "--output",
                    str(artifacts / "queue.json"),
                    "--differential-dir",
                    str(artifacts),
                ],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(failed.returncode, 0)
            self.assertIn("malformed manifest provenance", failed.stderr)

            for path in artifacts.glob("*.json"):
                path.unlink()
            rust_commit = subprocess.run(
                ["git", "-C", str(REPO_ROOT), "rev-parse", "HEAD"],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            unknown = self.differential_report("golden-ordinary", [], rust_commit)
            unknown["lane"] = "mystery"
            (artifacts / "unknown.json").write_text(json.dumps(unknown))
            failed = subprocess.run(
                [
                    sys.executable,
                    str(TOOL),
                    "build",
                    "--repo-root",
                    str(REPO_ROOT),
                    "--output",
                    str(artifacts / "queue.json"),
                    "--differential-dir",
                    str(artifacts),
                ],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(failed.returncode, 0)
            self.assertIn("unsupported differential lane mystery", failed.stderr)

    def test_stale_artifact_keeps_its_identity_without_matching_current_manifest(self):
        with tempfile.TemporaryDirectory() as temporary:
            artifacts = Path(temporary)
            stale = self.differential_report("golden-ordinary", [], "0" * 40)
            stale["manifest"]["sha256"] = "a" * 64
            stale["cases"] = stale["cases"][:2]
            stale["summary"] = dict(
                sorted(
                    collections.Counter(
                        case["outcome"] for case in stale["cases"]
                    ).items()
                )
            )
            (artifacts / "stale.json").write_text(json.dumps(stale))
            report = self.run_build("--differential-dir", str(artifacts))

        self.assertEqual(len(report["differential_artifacts"]), 1)
        artifact = report["differential_artifacts"][0]
        self.assertEqual(artifact["cpp_ref"], stale["cpp_ref"])
        self.assertEqual(artifact["gate_status"], "failed")
        self.assertEqual(artifact["lane"], "golden-ordinary")
        self.assertEqual(artifact["path"], "runtime-differential:stale.json")
        self.assertEqual(artifact["rust_commit"], "0" * 40)
        self.assertEqual(artifact["status"], "stale")

    def test_artifact_identity_is_directory_relative_and_subsystem_suffix_is_exact(self):
        rust_commit = subprocess.run(
            ["git", "-C", str(REPO_ROOT), "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        reports = []
        for _ in range(2):
            with tempfile.TemporaryDirectory() as temporary:
                artifacts = Path(temporary)
                report = self.differential_report(
                    "silver",
                    [
                        {
                            "id": "advance_blend_mode-inputs",
                            "outcome": "regressed",
                            "executed": True,
                            "diagnostic": "stable artifact identity",
                        }
                    ],
                    rust_commit,
                )
                (artifacts / "silver.json").write_text(json.dumps(report))
                reports.append(self.run_build("--differential-dir", str(artifacts)))

        for report in reports:
            self.assertEqual(
                report["differential_artifacts"][0]["path"],
                "runtime-differential:silver.json",
            )
            candidate = next(
                row
                for row in report["candidates"]
                if row["id"] == "silver:advance_blend_mode-inputs"
            )
            self.assertIn(
                "runtime-differential:silver.json", candidate["evidence_links"]
            )
            self.assertEqual(
                candidate["differential_observations"][0]["artifact"],
                "runtime-differential:silver.json",
            )
            rive_testing = next(
                row
                for row in report["candidates"]
                if row["id"] == "test:tests/unit_tests/runtime/rive_testing.cpp"
            )
            self.assertEqual(rive_testing["subsystem"], "rive_testing")

    def test_stale_artifact_still_requires_lane_provenance_shape(self):
        with tempfile.TemporaryDirectory() as temporary:
            artifacts = Path(temporary)
            for lane, removed_field, diagnostic in (
                ("golden-ordinary", "fixture", "malformed golden case provenance"),
                ("silver", "baseline", "malformed silver case provenance"),
                ("silver", "dependencies", "malformed silver case provenance"),
                ("silver", "action_fixtures", "malformed silver case provenance"),
            ):
                with self.subTest(lane=lane, removed_field=removed_field):
                    report = self.differential_report(lane, [], "0" * 40)
                    report["manifest"]["sha256"] = "a" * 64
                    report["cases"][0].pop(removed_field)
                    artifact_path = artifacts / "stale.json"
                    artifact_path.write_text(json.dumps(report))
                    failed = subprocess.run(
                        [
                            sys.executable,
                            str(TOOL),
                            "build",
                            "--repo-root",
                            str(REPO_ROOT),
                            "--output",
                            str(artifacts / "queue.json"),
                            "--differential-dir",
                            str(artifacts),
                        ],
                        capture_output=True,
                        text=True,
                    )
                    self.assertNotEqual(failed.returncode, 0)
                    self.assertIn(diagnostic, failed.stderr)
                    artifact_path.unlink()

    def test_stale_artifact_rejects_file_tags_outside_their_v1_source_cases(self):
        with tempfile.TemporaryDirectory() as temporary:
            artifacts = Path(temporary)
            mutations = (
                ("golden-ordinary", "fixture", "missing", "malformed golden case provenance"),
                ("golden-ordinary", "fixture", "virtual", "malformed golden case provenance"),
                ("silver", "baseline", "missing", "malformed silver case provenance"),
                ("silver", "baseline", "virtual", "malformed silver case provenance"),
            )
            for lane, field, tag, diagnostic in mutations:
                with self.subTest(lane=lane, field=field, tag=tag):
                    report = self.differential_report(lane, [], "0" * 40)
                    report["manifest"]["sha256"] = "a" * 64
                    record = report["cases"][0][field]
                    record["sha256"] = None
                    record[tag] = True
                    artifact_path = artifacts / "stale.json"
                    artifact_path.write_text(json.dumps(report))
                    failed = subprocess.run(
                        [
                            sys.executable,
                            str(TOOL),
                            "build",
                            "--repo-root",
                            str(REPO_ROOT),
                            "--output",
                            str(artifacts / "queue.json"),
                            "--differential-dir",
                            str(artifacts),
                        ],
                        capture_output=True,
                        text=True,
                    )
                    self.assertNotEqual(failed.returncode, 0)
                    self.assertIn(diagnostic, failed.stderr)
                    artifact_path.unlink()

    def test_pending_relevant_change_matches_the_next_committed_churn_window(self):
        with tempfile.TemporaryDirectory() as temporary:
            repo = Path(temporary)
            subprocess.run(["git", "init", "-q", str(repo)], check=True)
            subprocess.run(
                ["git", "-C", str(repo), "config", "user.name", "Drift Test"],
                check=True,
            )
            subprocess.run(
                ["git", "-C", str(repo), "config", "user.email", "drift@test"],
                check=True,
            )
            corpus = repo / "corpus.toml"
            corpus.write_text("version = 1\n")
            subprocess.run(["git", "-C", str(repo), "add", "corpus.toml"], check=True)
            subprocess.run(
                ["git", "-C", str(repo), "commit", "-qm", "initial"], check=True
            )
            filler = repo / "filler.txt"
            for revision in range(99):
                filler.write_text(f"{revision}\n")
                subprocess.run(
                    ["git", "-C", str(repo), "add", "filler.txt"], check=True
                )
                subprocess.run(
                    ["git", "-C", str(repo), "commit", "-qm", f"filler {revision}"],
                    check=True,
                )
            clean = recent_touch_counts(repo, {"corpus.toml"})["corpus.toml"]
            (repo / "unrelated.txt").write_text("pending\n")
            pending_unrelated = recent_touch_counts(repo, {"corpus.toml"})[
                "corpus.toml"
            ]
            (repo / "unrelated.txt").unlink()
            corpus.write_text("version = 2\n")
            pending = recent_touch_counts(repo, {"corpus.toml"})["corpus.toml"]
            subprocess.run(["git", "-C", str(repo), "add", "corpus.toml"], check=True)
            subprocess.run(
                ["git", "-C", str(repo), "commit", "-qm", "update"], check=True
            )
            committed = recent_touch_counts(repo, {"corpus.toml"})["corpus.toml"]

        self.assertEqual(clean, 1)
        self.assertEqual(pending_unrelated, clean)
        self.assertEqual(pending, clean)
        self.assertEqual(committed, pending)

    def test_passed_artifact_cannot_hide_a_missing_runner(self):
        with tempfile.TemporaryDirectory() as temporary:
            artifacts = Path(temporary)
            rust_commit = subprocess.run(
                ["git", "-C", str(REPO_ROOT), "rev-parse", "HEAD"],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            report = self.differential_report("golden-ordinary", [], rust_commit)
            report["gate_status"] = "passed"
            report["runners"][0] = {
                "role": "cpp",
                "path": "/unavailable/cpp",
                "sha256": None,
                "missing": True,
            }
            (artifacts / "golden.json").write_text(json.dumps(report))
            failed = subprocess.run(
                [
                    sys.executable,
                    str(TOOL),
                    "build",
                    "--repo-root",
                    str(REPO_ROOT),
                    "--output",
                    str(artifacts / "queue-output.json"),
                    "--differential-dir",
                    str(artifacts),
                ],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(failed.returncode, 0)
            self.assertIn("malformed runner provenance", failed.stderr)

    def test_fresh_artifact_rejects_impossible_passed_regression_and_duplicate_runner(self):
        rust_commit = subprocess.run(
            ["git", "-C", str(REPO_ROOT), "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        for mutation, diagnostic in (
            ("passed-regression", "impossible execution state"),
            ("duplicate-runner", "malformed runner provenance"),
        ):
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as temporary:
                artifacts = Path(temporary)
                report = self.differential_report("golden-ordinary", [], rust_commit)
                if mutation == "passed-regression":
                    case = next(
                        case for case in report["cases"] if case["id"] == "advance_blend_mode"
                    )
                    case.update(
                        outcome="regressed",
                        executed=True,
                        diagnostic="fabricated regression",
                    )
                    report["gate_status"] = "passed"
                    report["summary"] = dict(
                        sorted(
                            collections.Counter(
                                case["outcome"] for case in report["cases"]
                            ).items()
                        )
                    )
                else:
                    report["runners"].append(dict(report["runners"][0]))
                (artifacts / "golden.json").write_text(json.dumps(report))
                failed = subprocess.run(
                    [
                        sys.executable,
                        str(TOOL),
                        "build",
                        "--repo-root",
                        str(REPO_ROOT),
                        "--output",
                        str(artifacts / "queue.json"),
                        "--differential-dir",
                        str(artifacts),
                    ],
                    capture_output=True,
                    text=True,
                )
                self.assertNotEqual(failed.returncode, 0)
                self.assertIn(diagnostic, failed.stderr)

    def test_artifact_cannot_spoof_manifest_lane_or_status_for_fixture_exception(self):
        with tempfile.TemporaryDirectory() as temporary:
            artifacts = Path(temporary)
            rust_commit = subprocess.run(
                ["git", "-C", str(REPO_ROOT), "rev-parse", "HEAD"],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            report = self.differential_report("silver", [], rust_commit)
            row = next(
                case for case in report["cases"] if case["id"] == "advance_blend_mode-inputs"
            )
            row["lane"] = "scripted"
            row["declared_status"] = "provenance-unknown"
            row["fixture"] = {
                "path": row["fixture"]["path"],
                "sha256": None,
                "virtual": True,
            }
            report["summary"] = dict(
                sorted(collections.Counter(case["outcome"] for case in report["cases"]).items())
            )
            (artifacts / "silver.json").write_text(json.dumps(report))
            failed = subprocess.run(
                [
                    sys.executable,
                    str(TOOL),
                    "build",
                    "--repo-root",
                    str(REPO_ROOT),
                    "--output",
                    str(artifacts / "queue-output.json"),
                    "--differential-dir",
                    str(artifacts),
                ],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(failed.returncode, 0)
            self.assertIn("malformed case provenance", failed.stderr)

    def test_artifact_cannot_omit_manifest_declared_script_provenance(self):
        with tempfile.TemporaryDirectory() as temporary:
            artifacts = Path(temporary)
            rust_commit = subprocess.run(
                ["git", "-C", str(REPO_ROOT), "rev-parse", "HEAD"],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            report = self.differential_report("golden-ordinary", [], rust_commit)
            scripted = next(case for case in report["cases"] if "input_script" in case)
            scripted.pop("input_script")
            (artifacts / "golden.json").write_text(json.dumps(report))
            failed = subprocess.run(
                [
                    sys.executable,
                    str(TOOL),
                    "build",
                    "--repo-root",
                    str(REPO_ROOT),
                    "--output",
                    str(artifacts / "queue-output.json"),
                    "--differential-dir",
                    str(artifacts),
                ],
                capture_output=True,
                text=True,
            )
            self.assertNotEqual(failed.returncode, 0)
            self.assertIn("malformed script provenance", failed.stderr)

    def test_artifact_cannot_omit_manifest_declared_silver_provenance(self):
        rust_commit = subprocess.run(
            ["git", "-C", str(REPO_ROOT), "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        for case_id, field in (
            ("bidirectional_binding_source", "dependencies"),
            ("data_bind_font_test", "action_fixtures"),
        ):
            with self.subTest(case_id=case_id, field=field), tempfile.TemporaryDirectory() as temporary:
                artifacts = Path(temporary)
                report = self.differential_report("silver", [], rust_commit)
                case = next(case for case in report["cases"] if case["id"] == case_id)
                self.assertTrue(case[field])
                case[field] = []
                (artifacts / "silver.json").write_text(json.dumps(report))
                failed = subprocess.run(
                    [
                        sys.executable,
                        str(TOOL),
                        "build",
                        "--repo-root",
                        str(REPO_ROOT),
                        "--output",
                        str(artifacts / "queue-output.json"),
                        "--differential-dir",
                        str(artifacts),
                    ],
                    capture_output=True,
                    text=True,
                )
                self.assertNotEqual(failed.returncode, 0)
                self.assertIn("malformed silver provenance", failed.stderr)

    def test_build_writes_a_deterministic_human_report_and_check_detects_drift(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "queue.json"
            markdown = Path(temporary) / "queue.md"
            build = [
                sys.executable,
                str(TOOL),
                "build",
                "--repo-root",
                str(REPO_ROOT),
                "--output",
                str(output),
                "--markdown-output",
                str(markdown),
            ]
            subprocess.run(build, check=True)
            first_json = output.read_bytes()
            first_markdown = markdown.read_bytes()
            subprocess.run(build, check=True)
            self.assertEqual(output.read_bytes(), first_json)
            self.assertEqual(markdown.read_bytes(), first_markdown)
            self.assertIn("# Runtime drift queue", markdown.read_text())
            self.assertIn("Filter fields", markdown.read_text())

            check = [
                sys.executable,
                str(TOOL),
                "check",
                "--repo-root",
                str(REPO_ROOT),
                "--json",
                str(output),
                "--markdown",
                str(markdown),
            ]
            subprocess.run(check, check=True)
            output.write_text("{}\n")
            stale = subprocess.run(check, capture_output=True, text=True)
            self.assertNotEqual(stale.returncode, 0)
            self.assertIn("runtime drift queue snapshot is stale", stale.stderr)


if __name__ == "__main__":
    unittest.main()
