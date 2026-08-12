from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

TOOL = Path(__file__).with_name("freshness.py")
sys.path.insert(0, str(TOOL.parent))

from bootstrap_registry import (  # noqa: E402
    anchors_from_audit,
    behavioral_trace_tree,
    structural_proofs,
)
from freshness import FreshnessError  # noqa: E402
from ledger_scorecard import load_structural_freshness  # noqa: E402


def sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


class FreshnessCliTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.repo = root / "repo"
        self.upstream = root / "rive-runtime"
        self.repo.mkdir()
        self.upstream.mkdir()

        (self.repo / "src").mkdir()
        (self.repo / "docs").mkdir()
        (self.upstream / "src").mkdir()
        (self.repo / "src" / "owner.rs").write_text(
            "pub fn update() -> u32 { 1 }\n", encoding="utf-8"
        )
        (self.repo / "src" / "unrelated.rs").write_text(
            "pub fn unrelated() {}\n", encoding="utf-8"
        )
        (self.repo / "src" / "legacy.rs").write_text(
            "pub fn legacy() {}\n", encoding="utf-8"
        )
        (self.upstream / "src" / "owner.cpp").write_text(
            "int Owner::update() { return 1; }\n", encoding="utf-8"
        )
        (self.upstream / "src" / "legacy.cpp").write_text(
            "void Legacy::update() {}\n", encoding="utf-8"
        )
        (self.repo / "docs" / "audit.md").write_text(
            "## B6-0001\n\nrow_id: B6-0001\nowner: src/owner.cpp\nverdict: ISOMORPHIC\n"
            "\n## B6-0002\n\nrow_id: B6-0002\nowner: src/legacy.cpp\nverdict: ADAPTED\n",
            encoding="utf-8",
        )
        (self.repo / "probe.py").write_text(
            "def capture(): return 1\n", encoding="utf-8"
        )
        (self.repo / "fixture.riv").write_bytes(b"fixture-v1")

        self._git_init(self.repo)
        self._git_init(self.upstream)
        self.repo_ref = self._commit_all(self.repo, "proof source")
        self.upstream_ref = self._commit_all(self.upstream, "proof source")
        self._write_manifests()
        manifest_ref = self._commit_all(self.repo, "proof registry inputs")
        registry_path = self.repo / "parity-evidence-proofs.json"
        registry = json.loads(registry_path.read_text())
        registry["captures"] = {"structural_rust_commit": manifest_ref}
        registry["correspondence_manifest"] = "file-correspondence-manifest.toml"
        registry_path.write_text(json.dumps(registry, indent=2) + "\n")
        self._commit_all(self.repo, "proof registry")

    def _git_init(self, root: Path) -> None:
        subprocess.run(["git", "init", "-q", str(root)], check=True)
        subprocess.run(
            ["git", "-C", str(root), "config", "user.email", "test@example.com"],
            check=True,
        )
        subprocess.run(
            ["git", "-C", str(root), "config", "user.name", "Freshness Test"],
            check=True,
        )

    def _commit_all(self, root: Path, message: str) -> str:
        subprocess.run(["git", "-C", str(root), "add", "."], check=True)
        subprocess.run(
            ["git", "-C", str(root), "commit", "-q", "-m", message], check=True
        )
        return subprocess.check_output(
            ["git", "-C", str(root), "rev-parse", "HEAD"], text=True
        ).strip()

    def _write_manifests(self) -> None:
        rust_payload = (self.repo / "src" / "owner.rs").read_bytes()
        cpp_payload = (self.upstream / "src" / "owner.cpp").read_bytes()
        audit_payload = (self.repo / "docs" / "audit.md").read_bytes()
        probe_payload = (self.repo / "probe.py").read_bytes()
        fixture_payload = (self.repo / "fixture.riv").read_bytes()
        (self.repo / "file-correspondence-manifest.toml").write_text(
            f"""schema = "nuxie-file-correspondence/v1"
upstream_ref = "{self.upstream_ref}"
audit_upstream_ref = "{self.upstream_ref}"
current_audit_rows = ["B6-0001"]

[[file]]
upstream = "src/owner.cpp"
rust_module = "src/owner.rs"
b6_row_id = "B6-0001"
b6_cluster = "runtime-core"
b6_verdict = "ISOMORPHIC"
audit_record = "docs/audit.md"

[[file]]
upstream = "src/legacy.cpp"
rust_module = "src/legacy.rs"
b6_row_id = "B6-0002"
b6_cluster = "legacy"
b6_verdict = "ADAPTED"
audit_record = "docs/audit.md"
""",
            encoding="utf-8",
        )
        registry = {
            "schema": "nuxie-parity-evidence-proofs/v1",
            "upstream_ref": self.upstream_ref,
            "proofs": [
                {
                    "id": "structural:B6-0001",
                    "kind": "structural",
                    "owner": "src/owner.cpp",
                    "owner_family": "runtime-core",
                    "product_reach": "high",
                    "rust_mapping_paths": ["src/owner.rs"],
                    "structural_claim": {
                        "row_id": "B6-0001",
                        "verdict": "ISOMORPHIC",
                        "audit_record": "docs/audit.md",
                    },
                    "captured_rust_commit": self.repo_ref,
                    "upstream_ref": self.upstream_ref,
                    "evidence": {
                        "path": "docs/audit.md",
                        "sha256": sha256(audit_payload),
                    },
                    "cpp_items": [
                        {
                            "id": "cpp:Owner::update",
                            "path": "src/owner.cpp",
                            "sha256": sha256(cpp_payload),
                        }
                    ],
                    "rust_items": [
                        {
                            "id": "rust:update",
                            "path": "src/owner.rs",
                            "sha256": sha256(rust_payload),
                            "selector": {"kind": "line-window", "start": 1, "end": 1},
                        }
                    ],
                    "probes": [
                        {
                            "id": "probe:capture",
                            "path": "probe.py",
                            "sha256": sha256(probe_payload),
                        }
                    ],
                    "fixtures": [
                        {
                            "id": "fixture:owner",
                            "path": "fixture.riv",
                            "root": "repo",
                            "sha256": sha256(fixture_payload),
                        }
                    ],
                }
            ],
        }
        (self.repo / "parity-evidence-proofs.json").write_text(
            json.dumps(registry, indent=2) + "\n", encoding="utf-8"
        )

    def run_report(self) -> tuple[subprocess.CompletedProcess[str], dict]:
        output = self.repo / "report.json"
        result = subprocess.run(
            [
                sys.executable,
                str(TOOL),
                "report",
                "--repo-root",
                str(self.repo),
                "--rive-runtime-dir",
                str(self.upstream),
                "--output",
                str(output),
            ],
            text=True,
            capture_output=True,
        )
        document = json.loads(output.read_text()) if output.exists() else {}
        return result, document

    def test_source_body_changes_stale_only_the_affected_owner(self) -> None:
        result, report = self.run_report()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["proofs"][0]["current_validity"], "current")

        (self.repo / "src" / "unrelated.rs").write_text(
            "pub fn unrelated() { let _changed = true; }\n", encoding="utf-8"
        )
        result, report = self.run_report()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["proofs"][0]["current_validity"], "current")

        (self.repo / "src" / "owner.rs").write_text(
            "pub fn update() -> u32 { 2 }\n", encoding="utf-8"
        )
        result, report = self.run_report()
        self.assertEqual(result.returncode, 0, result.stderr)
        proof = report["proofs"][0]
        self.assertEqual(proof["historical_validity"], "valid")
        self.assertEqual(proof["current_validity"], "stale")
        self.assertIn("rust-item-changed:rust:update", proof["stale_reasons"])

    def test_owner_remapping_does_not_inherit_historical_item_freshness(self) -> None:
        manifest = self.repo / "file-correspondence-manifest.toml"
        manifest.write_text(
            manifest.read_text().replace(
                'rust_module = "src/owner.rs"',
                'rust_module = "src/unrelated.rs"',
                1,
            ),
            encoding="utf-8",
        )

        result, report = self.run_report()

        self.assertEqual(result.returncode, 0, result.stderr)
        owner = next(
            proof for proof in report["proofs"] if proof["id"] == "structural:B6-0001"
        )
        self.assertEqual(owner["current_validity"], "stale")
        self.assertIn("rust-owner-mapping-changed", owner["stale_reasons"])

    def test_deleted_bound_rust_item_is_reported_stale_with_tombstone(self) -> None:
        (self.repo / "src" / "owner.rs").unlink()

        result, report = self.run_report()

        self.assertEqual(result.returncode, 0, result.stderr)
        proof = next(
            proof for proof in report["proofs"] if proof["owner"] == "src/owner.cpp"
        )
        self.assertEqual(proof["historical_validity"], "valid")
        self.assertEqual(proof["current_validity"], "stale")
        self.assertIn("rust-item-changed:rust:update", proof["stale_reasons"])
        source = next(
            record
            for record in report["repo_source_state"]
            if record["path"] == "src/owner.rs"
        )
        self.assertEqual(source, {"path": "src/owner.rs", "missing": True})

    def test_changed_structural_claim_requires_current_substantiation(self) -> None:
        manifest = self.repo / "file-correspondence-manifest.toml"
        manifest.write_text(
            manifest.read_text().replace(
                'b6_verdict = "ISOMORPHIC"', 'b6_verdict = "ADAPTED"', 1
            ),
            encoding="utf-8",
        )

        result, _ = self.run_report()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "current evidence record docs/audit.md does not substantiate B6-0001",
            result.stderr,
        )

    def test_rereviewed_verdict_preserves_historical_proof_as_stale(self) -> None:
        manifest = self.repo / "file-correspondence-manifest.toml"
        manifest.write_text(
            manifest.read_text().replace(
                'b6_verdict = "ISOMORPHIC"', 'b6_verdict = "ADAPTED"', 1
            ),
            encoding="utf-8",
        )
        audit = self.repo / "docs" / "audit.md"
        audit.write_text(
            audit.read_text().replace("verdict: ISOMORPHIC", "verdict: ADAPTED", 1),
            encoding="utf-8",
        )

        result, report = self.run_report()

        self.assertEqual(result.returncode, 0, result.stderr)
        owner = next(
            proof for proof in report["proofs"] if proof["owner"] == "src/owner.cpp"
        )
        self.assertEqual(owner["historical_validity"], "valid")
        self.assertEqual(owner["current_validity"], "stale")
        self.assertIn("structural-verdict-changed", owner["stale_reasons"])
        freshness = load_structural_freshness(
            self.repo,
            self.repo / "report.json",
            tomllib.loads(manifest.read_text()),
            self.upstream,
        )
        self.assertEqual(freshness["src/owner.cpp"]["current_validity"], "stale")

    def test_rereviewed_row_id_stays_one_stale_owner_downstream(self) -> None:
        manifest = self.repo / "file-correspondence-manifest.toml"
        manifest.write_text(
            manifest.read_text()
            .replace(
                'current_audit_rows = ["B6-0001"]', 'current_audit_rows = ["B6-0099"]'
            )
            .replace('b6_row_id = "B6-0001"', 'b6_row_id = "B6-0099"', 1),
            encoding="utf-8",
        )
        audit = self.repo / "docs" / "audit.md"
        audit.write_text(
            audit.read_text()
            .replace("## B6-0001", "## B6-0099", 1)
            .replace("row_id: B6-0001", "row_id: B6-0099", 1),
            encoding="utf-8",
        )

        result, report = self.run_report()

        self.assertEqual(result.returncode, 0, result.stderr)
        owners = [
            proof for proof in report["proofs"] if proof["owner"] == "src/owner.cpp"
        ]
        self.assertEqual(len(owners), 1)
        self.assertEqual(owners[0]["current_validity"], "stale")
        self.assertIn("structural-row-id-changed", owners[0]["stale_reasons"])
        freshness = load_structural_freshness(
            self.repo,
            self.repo / "report.json",
            tomllib.loads(manifest.read_text()),
            self.upstream,
        )
        self.assertEqual(set(freshness), {"src/legacy.cpp", "src/owner.cpp"})
        self.assertEqual(freshness["src/owner.cpp"]["current_validity"], "stale")

    def test_cpp_body_change_preserving_symbol_reports_changed_owner(self) -> None:
        (self.upstream / "src" / "owner.cpp").write_text(
            "int Owner::update() { return 2; }\n", encoding="utf-8"
        )

        result, report = self.run_report()

        self.assertEqual(result.returncode, 0, result.stderr)
        proof = next(
            proof for proof in report["proofs"] if proof["id"] == "structural:B6-0001"
        )
        self.assertEqual(proof["current_validity"], "stale")
        self.assertIn("cpp-item-changed:cpp:Owner::update", proof["stale_reasons"])
        self.assertEqual(report["upstream_owner_changes"]["changed"], ["src/owner.cpp"])

    def test_frozen_structural_audits_remain_historical_not_current(self) -> None:
        result, report = self.run_report()

        self.assertEqual(result.returncode, 0, result.stderr)
        legacy = next(
            proof for proof in report["proofs"] if proof["owner"] == "src/legacy.cpp"
        )
        self.assertEqual(legacy["historical_validity"], "valid")
        self.assertEqual(legacy["current_validity"], "stale")
        self.assertEqual(legacy["binding_completeness"], "legacy-unbound")
        self.assertIn("legacy-proof-missing-content-bindings", legacy["stale_reasons"])

    def test_probe_and_fixture_changes_invalidate_the_captured_proof(self) -> None:
        (self.repo / "probe.py").write_text(
            "def capture(): return 2\n", encoding="utf-8"
        )
        (self.repo / "fixture.riv").write_bytes(b"fixture-v2")

        result, report = self.run_report()

        self.assertEqual(result.returncode, 0, result.stderr)
        proof = report["proofs"][0]
        self.assertEqual(proof["current_validity"], "stale")
        self.assertIn("probe-changed:probe:capture", proof["stale_reasons"])
        self.assertIn("fixture-changed:fixture:owner", proof["stale_reasons"])

    def test_a_bound_item_can_move_without_becoming_stale(self) -> None:
        (self.repo / "src" / "owner.rs").write_text(
            "// unrelated leading comment\npub fn update() -> u32 { 1 }\n",
            encoding="utf-8",
        )

        result, report = self.run_report()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["proofs"][0]["current_validity"], "current")

    def test_ambiguous_duplicate_item_body_is_stale(self) -> None:
        (self.repo / "src" / "owner.rs").write_text(
            "pub fn update() -> u32 { 1 }\npub fn update() -> u32 { 1 }\n",
            encoding="utf-8",
        )

        result, report = self.run_report()

        self.assertEqual(result.returncode, 0, result.stderr)
        proof = report["proofs"][0]
        self.assertEqual(proof["current_validity"], "stale")
        self.assertIn("rust-item-changed:rust:update", proof["stale_reasons"])

    def test_missing_or_malformed_audit_records_are_rejected(self) -> None:
        (self.repo / "docs" / "audit.md").unlink()
        result, _ = self.run_report()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("evidence record does not exist", result.stderr)

        (self.repo / "docs" / "audit.md").write_text(
            "## B6-9999\n\nrow_id: B6-9999\nowner: src/other.cpp\nverdict: ISOMORPHIC\n",
            encoding="utf-8",
        )
        result, _ = self.run_report()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("does not substantiate B6-0001", result.stderr)

    def test_malformed_or_historically_false_hashes_are_rejected(self) -> None:
        registry_path = self.repo / "parity-evidence-proofs.json"
        registry = json.loads(registry_path.read_text())
        registry["proofs"][0]["rust_items"][0]["sha256"] = "not-a-hash"
        registry_path.write_text(json.dumps(registry), encoding="utf-8")
        result, _ = self.run_report()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must be a lowercase SHA-256 digest", result.stderr)

        registry = json.loads(registry_path.read_text())
        registry["proofs"][0]["rust_items"][0]["sha256"] = "0" * 64
        registry_path.write_text(json.dumps(registry), encoding="utf-8")
        result, _ = self.run_report()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("captured hashes do not match historical source", result.stderr)

        registry = json.loads(registry_path.read_text())
        item = registry["proofs"][0]["rust_items"][0]
        item["selector"] = {"kind": "line-window", "start": 999, "end": 999}
        item["sha256"] = sha256(b"")
        registry_path.write_text(json.dumps(registry), encoding="utf-8")
        result, _ = self.run_report()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("captured hashes do not match historical source", result.stderr)

    def test_stale_report_ranks_by_reach_and_changed_source_churn(self) -> None:
        (self.repo / "src" / "owner.rs").write_text(
            "pub fn update() -> u32 { 2 }\n", encoding="utf-8"
        )

        result, report = self.run_report()

        self.assertEqual(result.returncode, 0, result.stderr)
        stale = report["stale_owners"]
        self.assertEqual(
            [proof["subsystem"] for proof in stale],
            sorted(proof["subsystem"] for proof in stale),
        )
        owner = next(proof for proof in stale if proof["id"] == "structural:B6-0001")
        self.assertEqual(owner["product_reach"], "high")
        self.assertGreaterEqual(owner["source_churn"], 1)
        self.assertEqual(owner["subsystem"], "owner")
        self.assertEqual(report["summary"]["stale_by_owner_family"]["runtime-core"], 1)
        self.assertEqual(report["summary"]["stale_by_subsystem"]["owner"], 1)

    def test_new_and_removed_upstream_owners_are_reported(self) -> None:
        (self.upstream / "src" / "new_owner.cpp").write_text(
            "int NewOwner::update() { return 1; }\n", encoding="utf-8"
        )
        (self.upstream / "src" / "owner.cpp").unlink()

        result, report = self.run_report()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["upstream_owner_changes"]["new"], ["src/new_owner.cpp"])
        self.assertEqual(report["upstream_owner_changes"]["removed"], ["src/owner.cpp"])
        self.assertEqual(report["upstream_owner_changes"]["changed"], ["src/owner.cpp"])
        freshness = load_structural_freshness(
            self.repo,
            self.repo / "report.json",
            tomllib.loads(
                (self.repo / "file-correspondence-manifest.toml").read_text()
            ),
            self.upstream,
        )
        self.assertEqual(freshness["src/owner.cpp"]["current_validity"], "stale")

    def test_removed_legacy_owner_keeps_historical_proof_downstream(self) -> None:
        manifest = self.repo / "file-correspondence-manifest.toml"
        contents = manifest.read_text()
        legacy_start = contents.index('\n[[file]]\nupstream = "src/legacy.cpp"')
        manifest.write_text(contents[:legacy_start] + "\n", encoding="utf-8")
        (self.upstream / "src" / "legacy.cpp").unlink()

        result, report = self.run_report()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            report["upstream_owner_changes"]["removed"], ["src/legacy.cpp"]
        )
        legacy = next(
            proof for proof in report["proofs"] if proof["owner"] == "src/legacy.cpp"
        )
        self.assertEqual(legacy["historical_validity"], "valid")
        self.assertEqual(legacy["current_validity"], "stale")
        self.assertIn("structural-owner-removed", legacy["stale_reasons"])
        freshness = load_structural_freshness(
            self.repo,
            self.repo / "report.json",
            tomllib.loads(manifest.read_text()),
            self.upstream,
        )
        self.assertNotIn("src/legacy.cpp", freshness)
        self.assertIn("src/owner.cpp", freshness)

    def test_upstream_pin_can_advance_without_recapturing_unchanged_proofs(
        self,
    ) -> None:
        (self.upstream / "src" / "new_owner.cpp").write_text(
            "int NewOwner::update() { return 1; }\n", encoding="utf-8"
        )
        current_ref = self._commit_all(self.upstream, "advance upstream")
        manifest = self.repo / "file-correspondence-manifest.toml"
        manifest.write_text(
            manifest.read_text().replace(self.upstream_ref, current_ref),
            encoding="utf-8",
        )

        result, report = self.run_report()

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(report["captured_upstream_ref"], self.upstream_ref)
        self.assertEqual(report["current_upstream_ref"], current_ref)
        self.assertEqual(report["upstream_owner_changes"]["new"], ["src/new_owner.cpp"])
        owner = next(
            proof for proof in report["proofs"] if proof["id"] == "structural:B6-0001"
        )
        self.assertEqual(owner["current_validity"], "current")

    def test_cpp_audit_anchor_must_match_the_captured_upstream_ref(self) -> None:
        with self.assertRaisesRegex(FreshnessError, "does not match"):
            anchors_from_audit(
                section="cpp@00000000:src/owner.cpp:1",
                repo_root=self.repo,
                upstream_root=self.upstream,
                rust_revision=self.repo_ref,
                upstream_revision=self.upstream_ref,
            )

    def test_structural_capture_requires_owner_and_all_mapping_paths(self) -> None:
        manifest = tomllib.loads(
            (self.repo / "file-correspondence-manifest.toml").read_text()
        )
        row = manifest["file"][0]
        crates = self.repo / "crates" / "runtime" / "src"
        crates.mkdir(parents=True)
        (crates / "owner.rs").write_text("pub fn owner() {}\n", encoding="utf-8")
        (crates / "unrelated.rs").write_text(
            "pub fn unrelated() {}\n", encoding="utf-8"
        )
        row["rust_module"] = (
            "crates/runtime/src/owner.rs;crates/runtime/src/unrelated.rs"
        )
        audit = self.repo / "docs" / "audit.md"
        audit.write_text(
            "## B6-0001\n\n"
            "row_id: B6-0001\nowner: src/owner.cpp\nverdict: ISOMORPHIC\n"
            f"cpp@{self.upstream_ref[:8]}:src/legacy.cpp:1\n"
            "crates/runtime/src/owner.rs:1\n",
            encoding="utf-8",
        )
        revision = self._commit_all(self.repo, "incomplete audit")

        with self.assertRaisesRegex(FreshnessError, "omits upstream owner"):
            structural_proofs(
                repo_root=self.repo,
                upstream_root=self.upstream,
                manifest=manifest,
                rust_revision=revision,
                upstream_revision=self.upstream_ref,
            )

        audit.write_text(
            audit.read_text().replace("src/legacy.cpp", "src/owner.cpp"),
            encoding="utf-8",
        )
        revision = self._commit_all(self.repo, "owner anchor")
        with self.assertRaisesRegex(FreshnessError, "omits Rust mappings"):
            structural_proofs(
                repo_root=self.repo,
                upstream_root=self.upstream,
                manifest=manifest,
                rust_revision=revision,
                upstream_revision=self.upstream_ref,
            )

    def test_freshness_rejects_incomplete_captured_structural_bindings(self) -> None:
        registry_path = self.repo / "parity-evidence-proofs.json"
        registry = json.loads(registry_path.read_text())
        registry["proofs"][0]["cpp_items"][0]["path"] = "src/legacy.cpp"
        registry_path.write_text(json.dumps(registry), encoding="utf-8")

        result, _ = self.run_report()

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("captured bindings omit upstream owner", result.stderr)

    def test_unreachable_prelinearization_trace_rebinds_to_recorded_tree(self) -> None:
        tree = subprocess.check_output(
            ["git", "-C", str(self.repo), "rev-parse", f"{self.repo_ref}^{{tree}}"],
            text=True,
        ).strip()

        self.assertEqual(
            behavioral_trace_tree(
                repo_root=self.repo,
                rust_revision=self.repo_ref,
                trace_rust_ref="0" * 40,
                recorded_tree=tree,
            ),
            tree,
        )

        with self.assertRaisesRegex(FreshnessError, "different tree"):
            behavioral_trace_tree(
                repo_root=self.repo,
                rust_revision=self.repo_ref,
                trace_rust_ref="0" * 40,
                recorded_tree="1" * 40,
            )


if __name__ == "__main__":
    unittest.main()
