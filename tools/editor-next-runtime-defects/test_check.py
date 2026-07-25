#!/usr/bin/env python3

from __future__ import annotations

import hashlib
import importlib.util
import pathlib
import subprocess
import tempfile
import textwrap
import unittest


TOOL = pathlib.Path(__file__).with_name("check.py")
CHECKER_SPEC = importlib.util.spec_from_file_location("editor_defect_check", TOOL)
assert CHECKER_SPEC is not None and CHECKER_SPEC.loader is not None
CHECKER = importlib.util.module_from_spec(CHECKER_SPEC)
CHECKER_SPEC.loader.exec_module(CHECKER)
PIN = "d788e8ec6e8b598526607d6a1e8818e8b637b60c"
RUNTIME_IDS = [f"RT-ED-{value:03d}" for value in range(1, 8)]
LOCAL_IDS = [
    *(f"LOC-{value:03d}" for value in range(1, 10)),
    *(f"LOC-{value:03d}" for value in range(11, 20)),
]
EXPECTED_IDS = RUNTIME_IDS + LOCAL_IDS
LEASE_RESERVED = [
    "crates/nuxie-graph/src/lib.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/artboard_data_bind.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-runtime/src/focus.rs",
    "crates/nuxie-runtime/src/lib.rs",
    "crates/nuxie-runtime/src/objects.rs",
    "crates/nuxie-runtime/src/retained_data_bind.rs",
    "crates/nuxie-runtime/src/text.rs",
    "docs/runtime-frame-loop-gaps.toml",
]
LEASE_FUTURE = [
    "crates/nuxie-runtime/src/animation.rs",
    "crates/nuxie-runtime/src/state_machine.rs",
    "crates/nuxie-runtime/src/state_machine/**",
]
LEASE_LEDGERS = [
    "docs/runtime-frame-loop-ownership.toml",
    "docs/runtime-frame-loop-status.md",
    "file-correspondence-manifest.toml",
]
CHILDREN = {
    "RT-ED-003": (["P04-C01", "P19-C03"], [], []),
    "RT-ED-004": (
        ["P04-C01", "P05-C01", "P10-C01", "P12-C01", "P15-C01"],
        [],
        [],
    ),
    "RT-ED-005": (["P09-C01"], [], []),
    "RT-ED-007": (["P19-C09"], [], []),
    "LOC-001": ([], ["P13-C07"], []),
    "LOC-002": ([], ["P04-C11", "P09-C01", "P09-C03", "P09-C06"], []),
    "LOC-005": ([], ["P09-C05"], []),
    "LOC-006": ([], ["P09-C04"], []),
    "LOC-007": ([], ["P11-C12"], []),
    "LOC-008": ([], ["P08-C06"], []),
    "LOC-009": ([], ["P14-C01"], []),
    "LOC-011": ([], ["P08-C06"], []),
    "LOC-012": ([], ["P19-C08"], []),
    "LOC-013": ([], ["P08-C08"], []),
    "LOC-014": ([], ["P08-C09"], []),
    "LOC-015": ([], ["P18-C01", "P18-C04", "P18-C05", "P18-C07"], []),
    "LOC-016": ([], ["P18-C01", "P18-C04"], []),
    "LOC-017": ([], ["P18-C07"], []),
    "LOC-018": ([], ["P04-C12", "P07-C04"], []),
    "LOC-019": ([], ["P14-C06"], []),
}


class EditorNextRuntimeDefectCheckTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        root = pathlib.Path(self.temp.name)
        self.repo = root / "repo"
        self.source = root / "source"
        self.upstream = root / "rive-runtime"
        (self.repo / "docs").mkdir(parents=True)
        self.source.mkdir()
        self.upstream.mkdir()
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
        (self.upstream / "README.md").write_text("fixture\n")
        subprocess.run(["git", "add", "."], cwd=self.upstream, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "fixture"], cwd=self.upstream, check=True
        )
        self.atlas = self.repo / "docs/atlas.toml"
        self.corrections = self.repo / "docs/corrections.toml"
        self.fixtures = self.repo / "docs/fixtures.toml"
        self.write_source_artifacts()
        self.write_corrections()
        self.write_fixtures()
        self.write_atlas()

    def write_source_artifacts(self) -> None:
        for name in ("proposal.md", "defects.md", "ledger.json"):
            (self.source / name).write_text(f"{name}\n")

    def artifact_row(self, name: str) -> str:
        digest = hashlib.sha256((self.source / name).read_bytes()).hexdigest()
        artifact_id = {
            "proposal.md": "cutover-proposal",
            "defects.md": "runtime-defects",
            "ledger.json": "parity-ledger",
        }[name]
        return textwrap.dedent(
            f"""
            [[artifact]]
            id = "{artifact_id}"
            path = "{name}"
            sha256 = "{digest}"
            """
        )

    def write_corrections(self) -> None:
        rows = "\n".join(
            textwrap.dedent(
                f"""
                [[correction]]
                id = "COR-{value:02d}"
                status = "open"
                description = "Correction {value}"
                resolution = "Resolution {value}"
                """
            ).strip()
            for value in range(1, 13)
        )
        self.corrections.write_text(
            textwrap.dedent(
                f"""
                schema = "nuxie.editor-next.runtime-defect-corrections/v1"
                version = 1
                source_pin = "{PIN}"
                expected_corrections = 12

                {rows}
                """
            ).lstrip()
        )

    def write_fixtures(self) -> None:
        rows = "\n".join(
            textwrap.dedent(
                f"""
                [[fixture]]
                id = "fixture.{defect_id.lower()}"
                defect_id = "{defect_id}"
                kind = "three-layer"
                status = "registered"
                driver = "standalone"
                """
            ).strip()
            for defect_id in EXPECTED_IDS
        )
        self.fixtures.write_text(
            textwrap.dedent(
                f"""
                schema = "nuxie.editor-next.runtime-defect-fixtures/v1"
                version = 1
                upstream_ref = "{PIN}"
                expected_fixtures = 25

                {rows}
                """
            ).lstrip()
        )

    def fixture_digest(
        self,
        defect_id: str,
        *,
        status: str = "registered",
        driver: str = "standalone",
    ) -> str:
        canonical = "\0".join(
            (
                f"fixture.{defect_id.lower()}",
                defect_id,
                "three-layer",
                status,
                driver,
            )
        )
        return hashlib.sha256(canonical.encode()).hexdigest()

    def defect_row(self, defect_id: str) -> str:
        formal, candidate, disputed = CHILDREN.get(defect_id, ([], [], []))
        hashes = {
            name: hashlib.sha256((self.source / name).read_bytes()).hexdigest()
            for name in ("proposal.md", "defects.md", "ledger.json")
        }
        return textwrap.dedent(
            f"""
            [[defect]]
            id = "{defect_id}"
            title = "Fixture {defect_id}"
            source_class = "test-observation"
            state = "reported"
            owner_class = "runtime"
            classification = "unqualified"
            ticket = "F-ED-00"
            fixture_id = "fixture.{defect_id.lower()}"
            preliminary_disposition = "Pending: isolated test row."
            reproduction_sha256 = "{self.fixture_digest(defect_id)}"
            rust_stimulus = "pending: isolated Rust stimulus"
            cpp_stimulus = "pending: isolated C++ stimulus"
            source_files = ["pending: exact file closure"]
            source_members = ["pending: exact member closure"]
            lifecycle_phases = ["pending: exact lifecycle closure"]
            rust_owner = "pending: exact Rust owner"
            displaced_mechanism = "pending: exact displaced mechanism"
            dependencies = []
            target_tests = []
            required_floors = ["runtime_tests"]
            owning_ledger = "test-ledger"
            adaptation_rule = "pending: exact adaptation rule"
            decision_row = "pending: exact decision row"
            touch = []
            dont_touch = ["@active-fl-lease"]
            formal_children = {formal!r}
            candidate_children = {candidate!r}
            disputed_children = {disputed!r}

            [defect.artifact_hashes]
            proposal = "{hashes['proposal.md']}"
            runtime_defects = "{hashes['defects.md']}"
            parity_ledger = "{hashes['ledger.json']}"

            [defect.revisions]
            original_localization_rust_sha = {{ status = "pending", reason = "Not localized." }}
            editor_last_consumed_runtime_sha = {{ status = "pending", reason = "Not consumed." }}
            investigation_head_sha = {{ status = "pending", reason = "Not investigated." }}
            merged_repair_sha = {{ status = "pending", reason = "No repair." }}
            consumed_runtime_sha = {{ status = "pending", reason = "No runtime consumption." }}
            consumed_superproject_sha = {{ status = "pending", reason = "No superproject consumption." }}

            [defect.renderer_provenance]
            status = "pending"
            reason = "Renderer provenance not yet captured."

            [defect.executor_verification]
            status = "pending"
            reason = "Executor has not run."

            [defect.orchestrator_verification]
            status = "pending"
            reason = "Orchestrator has not run."

            [[defect.history]]
            state = "reported"
            actor = "editor-cutover"
            evidence = "source-artifact"

            [defect.cpp_result]
            status = "pending"
            reason = "Fixture not run."

            [defect.rust_result]
            status = "pending"
            reason = "Fixture not run."

            [defect.editor_result]
            status = "fail"
            command = "fixture-command"
            evidence = "source-artifact"
            """
        )

    def write_atlas(self) -> None:
        rows = "\n".join(self.defect_row(defect_id) for defect_id in EXPECTED_IDS)
        artifacts = "\n".join(
            self.artifact_row(name)
            for name in ("proposal.md", "defects.md", "ledger.json")
        )
        self.atlas.write_text(
            textwrap.dedent(
                f"""
                schema = "nuxie.editor-next.runtime-defect-atlas/v1"
                version = 1
                upstream_ref = "{PIN}"
                editor_consumed_runtime_ref = "13aedd6d92de0991eed8dc3fda085db2dff18d48"
                investigation_base_ref = "efb6ad128d6aac7b81ed57d4a8b76eb9259ec833"
                source_snapshot_status = "landed"
                source_snapshot_ref = "13aedd6d92de0991eed8dc3fda085db2dff18d48"
                corrections_file = "docs/corrections.toml"
                fixtures_file = "docs/fixtures.toml"
                expected_defects = 25
                expected_formal_children = 8
                expected_candidate_children = 20
                expected_union_children = 27
                expected_overlap_children = ["P09-C01"]
                reserved_ids = ["LOC-010"]

                [floors]
                runtime_tests = 414
                nuxie_tests = 140
                cpp_probe_tests = 721
                golden_entries = 317
                golden_segments = 647
                scripted_entries = 317
                scripted_segments = 647
                renderer_pixels = 1468
                maximum_sdk_bytes = 9437184

                [lease]
                refreshed = "2026-07-24"
                active_wave = "FL-A"
                branch = "levi/fl-a"
                reserved_files = {LEASE_RESERVED!r}
                future_files = {LEASE_FUTURE!r}
                shared_ledgers = {LEASE_LEDGERS!r}

                {artifacts}

                {rows}
                """
            ).lstrip()
        )

    def run_check(
        self,
        *,
        source: bool = True,
        closed: bool = False,
        cpp_probe: pathlib.Path | None = None,
    ) -> subprocess.CompletedProcess[str]:
        command = [
            "python3",
            str(TOOL),
            "--repo-root",
            str(self.repo),
            "--atlas",
            str(self.atlas),
            "--corrections",
            str(self.corrections),
            "--fixtures",
            str(self.fixtures),
            "--expected-upstream-ref",
            PIN,
            "--test-mode",
        ]
        if source:
            command.extend(["--source-root", str(self.source)])
        if cpp_probe is not None:
            command.extend(["--cpp-probe", str(cpp_probe)])
        if closed:
            command.append("--require-closed")
        return subprocess.run(command, text=True, capture_output=True, check=False)

    def test_complete_reported_atlas_passes(self) -> None:
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("defects=25", result.stdout)
        self.assertIn("corrections=12", result.stdout)

    def test_production_cli_requires_every_provenance_input(self) -> None:
        command = [
            "python3",
            str(TOOL),
            "--repo-root",
            str(self.repo),
            "--atlas",
            str(self.atlas),
            "--corrections",
            str(self.corrections),
            "--fixtures",
            str(self.fixtures),
            "--expected-upstream-ref",
            PIN,
        ]
        result = subprocess.run(
            command, text=True, capture_output=True, check=False
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn("--source-root", result.stderr)
        self.assertIn("--rive-runtime-dir", result.stderr)
        self.assertIn("--cpp-probe", result.stderr)

    def test_test_mode_cannot_validate_repository_atlas(self) -> None:
        canonical = self.repo / "docs/editor-next-runtime-defect-atlas.toml"
        canonical.write_text(self.atlas.read_text())
        command = [
            "python3",
            str(TOOL),
            "--repo-root",
            str(self.repo),
            "--atlas",
            str(canonical),
            "--corrections",
            str(self.corrections),
            "--fixtures",
            str(self.fixtures),
            "--expected-upstream-ref",
            PIN,
            "--test-mode",
        ]
        result = subprocess.run(
            command, text=True, capture_output=True, check=False
        )
        self.assertEqual(result.returncode, 2)
        self.assertIn(
            "--test-mode cannot validate the repository atlas", result.stderr
        )

    def test_correction_ids_are_pinned_not_just_counted(self) -> None:
        self.corrections.write_text(
            self.corrections.read_text().replace('id = "COR-12"', 'id = "COR-13"')
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("correction ids must be exactly COR-01..COR-12", result.stderr)

    def test_correction_resolution_is_required(self) -> None:
        self.corrections.write_text(
            self.corrections.read_text().replace(
                'resolution = "Resolution 1"\n',
                "",
                1,
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("COR-01 has no resolution", result.stderr)

    def test_implemented_fixture_rejects_pending_driver(self) -> None:
        self.fixtures.write_text(
            self.fixtures.read_text()
            .replace('status = "registered"', 'status = "implemented"', 1)
            .replace('driver = "standalone"', 'driver = "pending:F-ED-01"', 1)
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("uses non-executable driver", result.stderr)
        self.assertIn("implemented but atlas row RT-ED-001 is reported", result.stderr)

    def test_cpp_probe_registry_must_exactly_match_cpp_driven_fixtures(self) -> None:
        self.fixtures.write_text(
            self.fixtures.read_text().replace(
                'driver = "standalone"',
                'driver = "cpp_probe/registry.cpp"',
                1,
            )
        )
        probe = pathlib.Path(self.temp.name) / "probe"
        probe.write_text("#!/bin/sh\nprintf '%s\\n' fixture.wrong\n")
        probe.chmod(0o755)
        result = self.run_check(cpp_probe=probe)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("C++ probe registry must exactly match", result.stderr)

    def test_cpp_probe_provenance_stamp_is_required(self) -> None:
        probe = pathlib.Path(self.temp.name) / "probe-without-stamp"
        probe.write_text("#!/bin/sh\nexit 0\n")
        probe.chmod(0o755)
        errors: list[str] = []
        CHECKER.validate_cpp_probe_provenance(
            probe,
            self.repo,
            PIN,
            errors,
        )
        self.assertEqual(
            errors,
            [f"C++ probe provenance stamp is missing at {probe}.provenance"],
        )

    def test_qualified_renderer_cannot_skip_complete_provenance(self) -> None:
        row = {
            "id": "RT-ED-004",
            "state": "qualified",
            "owner_class": "renderer",
            "required_floors": ["renderer_pixels"],
            "renderer_provenance": {
                "status": "not-applicable",
                "reason": "incorrectly omitted",
            },
        }
        errors: list[str] = []
        CHECKER.validate_closure_schema(
            row,
            {"kind": "browser-renderer"},
            {},
            errors,
        )
        self.assertIn(
            "RT-ED-004 is qualified renderer work but renderer provenance "
            "is not complete",
            errors,
        )

    def test_per_defect_child_mapping_is_substitution_proof(self) -> None:
        content = self.atlas.read_text()
        content = content.replace(
            "formal_children = ['P04-C01', 'P19-C03']",
            "formal_children = ['P04-C01']",
            1,
        )
        content = content.replace(
            "formal_children = ['P04-C01', 'P05-C01', 'P10-C01', 'P12-C01', 'P15-C01']",
            "formal_children = ['P04-C01', 'P05-C01', 'P10-C01', 'P12-C01', 'P15-C01', 'P19-C03']",
            1,
        )
        self.atlas.write_text(content)
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "RT-ED-003 child mapping differs from the pinned exact map",
            result.stderr,
        )

    def test_closure_schema_rejects_missing_owner(self) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace(
                'rust_owner = "pending: exact Rust owner"\n',
                "",
                1,
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("RT-ED-001 has no closure field rust_owner", result.stderr)

    def test_reproduction_hash_binds_atlas_to_fixture_row(self) -> None:
        self.fixtures.write_text(
            self.fixtures.read_text().replace(
                'driver = "standalone"',
                'driver = "different-executable"',
                1,
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("RT-ED-001 reproduction_sha256 is", result.stderr)

    def test_qualified_fixture_requires_hashed_stimulus_files(self) -> None:
        self.fixtures.write_text(
            self.fixtures.read_text().replace(
                'status = "registered"',
                'status = "qualified"',
                1,
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "is qualified but has no hashed stimulus files",
            result.stderr,
        )

    def test_active_lease_cannot_shrink_itself(self) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace(
                "'crates/nuxie-runtime/src/artboard.rs',",
                "",
                1,
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "lease reserved_files differs from the pinned coordination contract",
            result.stderr,
        )

    def test_atlas_must_name_the_checked_corrections_file(self) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace(
                'corrections_file = "docs/corrections.toml"',
                'corrections_file = "docs/other.toml"',
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("atlas corrections_file resolves to", result.stderr)

    def test_pending_editor_snapshot_cannot_claim_a_landed_commit(self) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace(
                'source_snapshot_status = "landed"',
                'source_snapshot_status = "pending-editor-commit"',
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "pending-editor-commit snapshot must have an empty source_snapshot_ref",
            result.stderr,
        )

    def test_every_atlas_fixture_must_have_one_registry_entry(self) -> None:
        fixture = textwrap.dedent(
            """
            [[fixture]]
            id = "fixture.rt-ed-007"
            defect_id = "RT-ED-007"
            kind = "three-layer"
            status = "registered"
            driver = "standalone"
            """
        ).strip()
        self.fixtures.write_text(self.fixtures.read_text().replace(fixture, ""))
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "atlas fixture ids missing from registry: fixture.rt-ed-007",
            result.stderr,
        )

    def test_child_overlap_ratchet_is_exact(self) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace(
                'expected_overlap_children = ["P09-C01"]',
                'expected_overlap_children = ["P04-C12"]',
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "child-overlap ratchet names P04-C12, actual overlap is P09-C01",
            result.stderr,
        )

    def test_regression_floor_cannot_be_lowered(self) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace(
                "cpp_probe_tests = 721",
                "cpp_probe_tests = 720",
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "floor cpp_probe_tests is 720; minimum is 721",
            result.stderr,
        )

    def test_source_artifact_hash_drift_fails(self) -> None:
        (self.source / "defects.md").write_text("changed\n")
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("artifact runtime-defects hash is", result.stderr)

    def test_missing_defect_id_fails(self) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace('id = "RT-ED-007"', 'id = "LOC-010"')
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("atlas is missing defect ids: RT-ED-007", result.stderr)
        self.assertIn("atlas has unexpected defect ids: LOC-010", result.stderr)

    def test_illegal_state_transition_fails(self) -> None:
        original = textwrap.dedent(
            """
            state = "reported"
            owner_class = "runtime"
            """
        ).strip()
        replacement = textwrap.dedent(
            """
            state = "mapped"
            owner_class = "runtime"
            """
        ).strip()
        content = self.atlas.read_text().replace(original, replacement, 1)
        original_history = textwrap.dedent(
            """
            [[defect.history]]
            state = "reported"
            actor = "editor-cutover"
            evidence = "source-artifact"
            """
        ).strip()
        replacement_history = (
            original_history
            + "\n\n"
            + textwrap.dedent(
                """
                [[defect.history]]
                state = "mapped"
                actor = "executor"
                evidence = "invalid-skip"
                """
            ).strip()
        )
        self.atlas.write_text(
            content.replace(original_history, replacement_history, 1)
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("illegal state transition reported -> mapped", result.stderr)

    def test_qualified_row_cannot_keep_pending_direct_result(self) -> None:
        content = self.atlas.read_text().replace(
            'state = "reported"\nowner_class = "runtime"',
            'state = "qualified"\nowner_class = "runtime"',
            1,
        )
        history = textwrap.dedent(
            """
            [[defect.history]]
            state = "reported"
            actor = "editor-cutover"
            evidence = "source-artifact"
            """
        ).strip()
        qualified_history = (
            history
            + "\n\n"
            + textwrap.dedent(
                """
                [[defect.history]]
                state = "reproduced"
                actor = "executor"
                evidence = "direct-fixture"

                [[defect.history]]
                state = "qualified"
                actor = "executor"
                evidence = "three-layer-differential"
                """
            ).strip()
        )
        self.atlas.write_text(content.replace(history, qualified_history, 1))
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "RT-ED-001 is qualified but cpp_result is still pending",
            result.stderr,
        )
        self.assertIn(
            "RT-ED-001 is qualified but closure field rust_owner is pending",
            result.stderr,
        )

    def test_verification_can_remain_pending_until_its_state_transition(self) -> None:
        pending = {"status": "pending", "reason": "Independent run has not completed."}

        errors: list[str] = []
        CHECKER.validate_verification(
            "LOC-019",
            "executor_verification",
            pending,
            "mapped",
            errors,
        )
        self.assertEqual(errors, [])

        errors = []
        CHECKER.validate_verification(
            "LOC-019",
            "executor_verification",
            pending,
            "executor-green",
            errors,
        )
        self.assertIn(
            "LOC-019 is executor-green but closure field "
            "executor_verification is pending",
            errors,
        )

        errors = []
        CHECKER.validate_verification(
            "LOC-019",
            "orchestrator_verification",
            pending,
            "executor-green",
            errors,
        )
        self.assertEqual(errors, [])

        errors = []
        CHECKER.validate_verification(
            "LOC-019",
            "orchestrator_verification",
            pending,
            "orchestrator-verified",
            errors,
        )
        self.assertIn(
            "LOC-019 is orchestrator-verified but closure field "
            "orchestrator_verification is pending",
            errors,
        )

    def test_every_row_must_carry_the_complete_active_lease(self) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace(
                'dont_touch = ["@active-fl-lease"]',
                'dont_touch = ["crates/nuxie-runtime/src/artboard.rs"]',
                1,
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "RT-ED-001 omits active lease locks:",
            result.stderr,
        )
        self.assertIn("crates/nuxie-runtime/src/animation.rs", result.stderr)

    def test_active_lease_sentinel_expands_to_the_exact_lock_set(self) -> None:
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_closed_mode_rejects_every_open_row(self) -> None:
        result = self.run_check(closed=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("rows remain open: LOC-001", result.stderr)
        self.assertIn("RT-ED-007", result.stderr)


if __name__ == "__main__":
    unittest.main()
