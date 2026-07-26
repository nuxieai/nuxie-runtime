#!/usr/bin/env python3

from __future__ import annotations

import hashlib
import importlib.util
import json
import pathlib
import shutil
import subprocess
import tempfile
import textwrap
import tomllib
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
PROPOSAL_NAME = "nuxie-editor-next-cutover-proposal.md"
DEFECTS_NAME = "nuxie-editor-next-runtime-defects.md"
LEDGER_NAME = "nuxie-editor-next-parity-ledger.json"
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
    "RT-ED-005": (["P09-C01"], [], []),
    "RT-ED-007": (["P19-C09"], [], []),
    "LOC-001": ([], ["P13-C07"], []),
    "LOC-002": (["P04-C11", "P09-C03", "P09-C06"], ["P09-C01"], []),
    "LOC-005": (["P09-C05"], [], []),
    "LOC-006": ([], ["P09-C04"], []),
    "LOC-007": (["P11-C12"], [], []),
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
        self.source = root / "plans"
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

    def init_git_repo(self, path: pathlib.Path) -> None:
        path.mkdir()
        subprocess.run(["git", "init", "-q"], cwd=path, check=True)
        subprocess.run(
            ["git", "config", "user.email", "test@example.com"],
            cwd=path,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "Test"],
            cwd=path,
            check=True,
        )

    def commit_file(
        self,
        repo: pathlib.Path,
        content: str,
        message: str,
    ) -> str:
        (repo / "record").write_text(content)
        subprocess.run(["git", "add", "record"], cwd=repo, check=True)
        subprocess.run(
            ["git", "commit", "-qm", message],
            cwd=repo,
            check=True,
        )
        return subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=repo,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()

    def revision_repositories(
        self,
        *,
        gitlink_revision: str | None = None,
    ) -> tuple[
        pathlib.Path,
        pathlib.Path,
        str,
        str,
        str,
        str,
        str,
    ]:
        runtime = pathlib.Path(self.temp.name) / "revision-runtime"
        editor = pathlib.Path(self.temp.name) / "revision-editor"
        self.init_git_repo(runtime)
        runtime_base = self.commit_file(runtime, "base\n", "runtime base")
        runtime_repair = self.commit_file(runtime, "repair\n", "runtime repair")
        runtime_current = self.commit_file(runtime, "current\n", "runtime current")

        self.init_git_repo(editor)
        editor_repair = self.commit_file(editor, "repair\n", "editor repair")
        (editor / "third_party").mkdir()
        subprocess.run(
            [
                "git",
                "update-index",
                "--add",
                "--cacheinfo",
                f"160000,{gitlink_revision or runtime_current},"
                "third_party/nuxie-runtime",
            ],
            cwd=editor,
            check=True,
        )
        subprocess.run(
            ["git", "commit", "-qm", "consume runtime"],
            cwd=editor,
            check=True,
        )
        superproject = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=editor,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        return (
            runtime,
            editor,
            runtime_base,
            runtime_repair,
            runtime_current,
            editor_repair,
            superproject,
        )

    def write_source_artifacts(self) -> None:
        (self.source / PROPOSAL_NAME).write_text(f"{PROPOSAL_NAME}\n")
        defect_sections = "\n\n".join(
            f"### {defect_id} — Fixture {defect_id}\n\nRecord {defect_id}."
            for defect_id in EXPECTED_IDS
        )
        (self.source / DEFECTS_NAME).write_text(
            f"# Runtime defects\n\n{defect_sections}\n"
        )

        formal_by_child: dict[str, list[str]] = {}
        candidates_by_child: dict[str, list[str]] = {}
        child_ids: set[str] = set()
        for defect_id, (formal, candidate, _) in CHILDREN.items():
            for child_id in formal:
                child_ids.add(child_id)
                formal_by_child.setdefault(child_id, []).append(defect_id)
            for child_id in candidate:
                child_ids.add(child_id)
                candidates_by_child.setdefault(child_id, []).append(defect_id)
        children = []
        for child_id in sorted(child_ids):
            mentions = " ".join(sorted(candidates_by_child.get(child_id, [])))
            children.append(
                {
                    "id": child_id,
                    "assertion": f"Fixture assertion. {mentions}".strip(),
                    "runtimeDependencies": [
                        {"id": defect_id}
                        for defect_id in sorted(formal_by_child.get(child_id, []))
                    ],
                }
            )
        (self.source / LEDGER_NAME).write_text(
            json.dumps(
                {
                    "rows": [
                        {
                            "id": "P00",
                            "children": children,
                        }
                    ]
                },
                indent=2,
            )
            + "\n"
        )

    def artifact_row(self, name: str) -> str:
        digest = hashlib.sha256((self.source / name).read_bytes()).hexdigest()
        artifact_id = {
            PROPOSAL_NAME: "cutover-proposal",
            DEFECTS_NAME: "runtime-defects",
            LEDGER_NAME: "parity-ledger",
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
            for name in (PROPOSAL_NAME, DEFECTS_NAME, LEDGER_NAME)
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
            proposal = "{hashes[PROPOSAL_NAME]}"
            runtime_defects = "{hashes[DEFECTS_NAME]}"
            parity_ledger = "{hashes[LEDGER_NAME]}"

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
            for name in (PROPOSAL_NAME, DEFECTS_NAME, LEDGER_NAME)
        )
        runtime_defects_sha256 = hashlib.sha256(
            (self.source / DEFECTS_NAME).read_bytes()
        ).hexdigest()
        parity_ledger_sha256 = hashlib.sha256(
            (self.source / LEDGER_NAME).read_bytes()
        ).hexdigest()
        self.atlas.write_text(
            textwrap.dedent(
                f"""
                schema = "nuxie.editor-next.runtime-defect-atlas/v2"
                version = 2
                upstream_ref = "{PIN}"
                editor_consumed_runtime_ref = "13aedd6d92de0991eed8dc3fda085db2dff18d48"
                investigation_base_ref = "efb6ad128d6aac7b81ed57d4a8b76eb9259ec833"
                source_snapshot_status = "landed"
                source_snapshot_ref = "13aedd6d92de0991eed8dc3fda085db2dff18d48"
                corrections_file = "docs/corrections.toml"
                fixtures_file = "docs/fixtures.toml"
                expected_defects = 25
                expected_formal_children = 9
                expected_candidate_children = 15
                expected_union_children = 23
                expected_overlap_children = ["P09-C01"]
                reserved_ids = ["LOC-010"]

                [program]
                formal_objective = "own-complete-editor-reported-runtime-defect-queue"
                state_source = "docs/editor-next-runtime-defect-atlas.toml"
                schedule_source = "docs/editor-next-runtime-defect-status.md"
                completion_source = "docs/editor-next-runtime-defect-goal.md"
                port_plan = "docs/editor-next-runtime-defect-port-map.md"
                porting_law = "docs/PORTING.md"
                collision_ledger = "docs/runtime-frame-loop-ownership.toml"
                coordinator_thread = "019f9c97-edcf-76d3-a786-11f443da22d3"
                editor_consumption_required = false
                editor_merge_blocks_program = false
                parallel_execution = true
                runtime_fix_assignment_requires_tracked_dependency = true
                terminal_state = "closed"
                intake_cycle = 1

                [inbox]
                canonical_branch = "origin/levi/editor-next-cutover-assembly"
                runtime_defects_path = "plans/nuxie-editor-next-runtime-defects.md"
                parity_ledger_path = "plans/nuxie-editor-next-parity-ledger.json"
                last_consumed_editor_ref = "13aedd6d92de0991eed8dc3fda085db2dff18d48"
                last_consumed_runtime_defects_sha256 = "{runtime_defects_sha256}"
                last_consumed_parity_ledger_sha256 = "{parity_ledger_sha256}"
                newest_available_editor_ref = "13aedd6d92de0991eed8dc3fda085db2dff18d48"
                newest_available_runtime_defects_sha256 = "{runtime_defects_sha256}"
                newest_available_parity_ledger_sha256 = "{parity_ledger_sha256}"
                unconsumed_records = 0
                imported_atlas_count = 25

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

    def append_defect(self, defect_id: str) -> None:
        self.atlas.write_text(
            self.atlas.read_text()
            .replace("expected_defects = 25", "expected_defects = 26")
            .replace("imported_atlas_count = 25", "imported_atlas_count = 26")
            + "\n"
            + self.defect_row(defect_id)
        )
        self.fixtures.write_text(
            self.fixtures.read_text().replace(
                "expected_fixtures = 25",
                "expected_fixtures = 26",
            )
            + "\n"
            + textwrap.dedent(
                f"""
                [[fixture]]
                id = "fixture.{defect_id.lower()}"
                defect_id = "{defect_id}"
                kind = "three-layer"
                status = "registered"
                driver = "standalone"
                """
            ).lstrip()
        )

    def refresh_source_bindings(self) -> None:
        atlas = tomllib.loads(self.atlas.read_text())
        old_hashes = {
            row["path"]: row["sha256"] for row in atlas["artifact"]
        }
        new_hashes = {
            name: hashlib.sha256((self.source / name).read_bytes()).hexdigest()
            for name in (PROPOSAL_NAME, DEFECTS_NAME, LEDGER_NAME)
        }
        content = self.atlas.read_text()
        for name, old_digest in old_hashes.items():
            content = content.replace(old_digest, new_hashes[name])
        self.atlas.write_text(content)

    def append_consumed_defect(
        self,
        defect_id: str,
        *,
        complete_source_record: bool = True,
    ) -> None:
        evidence = (
            f"- Editor SHA: `{'b' * 40}`\n"
            f"- Runtime SHA: `{'a' * 40}`\n"
            "- Command: `cargo test -p fixture loc_020`\n"
            "- Result: deterministic runtime failure.\n"
            if complete_source_record
            else f"Record {defect_id}.\n"
        )
        with (self.source / DEFECTS_NAME).open("a") as source:
            source.write(
                f"\n### {defect_id} — Fixture {defect_id}\n\n"
                f"{evidence}"
            )
        self.append_defect(defect_id)
        self.refresh_source_bindings()

    def make_future_row_incomplete(
        self,
        defect_id: str,
        *,
        state: str,
    ) -> None:
        marker = f'[[defect]]\nid = "{defect_id}"'
        before, separator, tail = self.atlas.read_text().rpartition(marker)
        self.assertTrue(separator)
        if state != "reported":
            tail = tail.replace(
                'state = "reported"',
                f'state = "{state}"',
                1,
            )
        tail = tail.replace(
            'status = "fail"\n'
            'command = "fixture-command"\n'
            'evidence = "source-artifact"',
            'status = "pending"\n'
            'reason = "Committed inbox record lacks an executable reproducer."',
            1,
        )
        if state == "intake-needs-evidence":
            history = textwrap.dedent(
                """
                [[defect.history]]
                state = "reported"
                actor = "editor-cutover"
                evidence = "source-artifact"
                """
            ).strip()
            intake = (
                history
                + "\n\n"
                + textwrap.dedent(
                    """
                    [[defect.history]]
                    state = "intake-needs-evidence"
                    actor = "f-ed-intake"
                    evidence = "committed inbox record lacks an executable reproducer"
                    """
                ).strip()
            )
            tail = tail.replace(history, intake, 1)
        self.atlas.write_text(before + separator + tail)

    def copy_source(self) -> pathlib.Path:
        newest_parent = pathlib.Path(
            tempfile.mkdtemp(
                prefix="newest-editor-",
                dir=self.temp.name,
            )
        )
        newest = newest_parent / "plans"
        shutil.copytree(self.source, newest)
        return newest

    def set_newest_checkpoint(
        self,
        newest: pathlib.Path,
        *,
        unconsumed: int,
        ref: str = "23aedd6d92de0991eed8dc3fda085db2dff18d48",
    ) -> None:
        atlas = tomllib.loads(self.atlas.read_text())
        inbox = atlas["inbox"]
        runtime_digest = hashlib.sha256(
            (newest / DEFECTS_NAME).read_bytes()
        ).hexdigest()
        ledger_digest = hashlib.sha256(
            (newest / LEDGER_NAME).read_bytes()
        ).hexdigest()
        content = self.atlas.read_text()
        content = content.replace(
            f'newest_available_editor_ref = '
            f'"{inbox["newest_available_editor_ref"]}"',
            f'newest_available_editor_ref = "{ref}"',
        )
        content = content.replace(
            f'newest_available_runtime_defects_sha256 = '
            f'"{inbox["newest_available_runtime_defects_sha256"]}"',
            f'newest_available_runtime_defects_sha256 = "{runtime_digest}"',
        )
        content = content.replace(
            f'newest_available_parity_ledger_sha256 = '
            f'"{inbox["newest_available_parity_ledger_sha256"]}"',
            f'newest_available_parity_ledger_sha256 = "{ledger_digest}"',
        )
        content = content.replace(
            f'unconsumed_records = {inbox["unconsumed_records"]}',
            f"unconsumed_records = {unconsumed}",
        )
        self.atlas.write_text(content)

    def run_check(
        self,
        *,
        source: bool = True,
        newest_source: pathlib.Path | None = None,
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
            command.extend(
                [
                    "--newest-source-root",
                    str(newest_source if newest_source is not None else self.source),
                ]
            )
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

    def test_inbox_contract_is_exact_and_bound_to_source_artifacts(self) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace(
                'canonical_branch = "origin/levi/editor-next-cutover-assembly"',
                'canonical_branch = "origin/main"',
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "inbox canonical_branch is 'origin/main'; expected "
            "'origin/levi/editor-next-cutover-assembly'",
            result.stderr,
        )

    def test_identical_inbox_checkpoints_require_zero_unconsumed_records(
        self,
    ) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace(
                "unconsumed_records = 0",
                "unconsumed_records = 1",
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "inbox checkpoints are identical but unconsumed_records is not 0",
            result.stderr,
        )

    def test_different_inbox_checkpoints_cannot_claim_zero_unconsumed_records(
        self,
    ) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace(
                'newest_available_editor_ref = '
                '"13aedd6d92de0991eed8dc3fda085db2dff18d48"',
                'newest_available_editor_ref = '
                '"23aedd6d92de0991eed8dc3fda085db2dff18d48"',
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "inbox checkpoints differ but unconsumed_records is 0",
            result.stderr,
        )

    def test_inbox_imported_count_must_match_atlas_rows(self) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace(
                "imported_atlas_count = 25",
                "imported_atlas_count = 24",
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "inbox imported_atlas_count is 24; atlas has 25 defect rows",
            result.stderr,
        )

    def test_inbox_consumed_hashes_must_match_source_artifacts(self) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace(
                "last_consumed_runtime_defects_sha256 = "
                f'"{hashlib.sha256((self.source / DEFECTS_NAME).read_bytes()).hexdigest()}"',
                "last_consumed_runtime_defects_sha256 = "
                f'"{"0" * 64}"',
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "inbox last_consumed_runtime_defects_sha256 does not match "
            "the pinned runtime_defects source artifact",
            result.stderr,
        )

    def test_inbox_consumed_ref_must_match_source_snapshot(self) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace(
                'last_consumed_editor_ref = '
                '"13aedd6d92de0991eed8dc3fda085db2dff18d48"',
                'last_consumed_editor_ref = '
                '"23aedd6d92de0991eed8dc3fda085db2dff18d48"',
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "inbox last_consumed_editor_ref does not match "
            "source_snapshot_ref",
            result.stderr,
        )

    def test_exact_new_or_changed_record_count_is_accepted(self) -> None:
        newest = self.copy_source()
        path = newest / DEFECTS_NAME
        path.write_text(
            path.read_text().replace(
                "Record LOC-001.",
                "Record LOC-001 changed.",
            )
        )
        self.set_newest_checkpoint(newest, unconsumed=1)
        result = self.run_check(newest_source=newest)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("unconsumed=1", result.stdout)

    def test_exact_new_or_changed_record_count_cannot_be_misstated(self) -> None:
        newest = self.copy_source()
        path = newest / DEFECTS_NAME
        path.write_text(
            path.read_text().replace(
                "Record LOC-001.",
                "Record LOC-001 changed.",
            )
        )
        self.set_newest_checkpoint(newest, unconsumed=2)
        result = self.run_check(newest_source=newest)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "exact new/changed canonical record count is 1",
            result.stderr,
        )

    def test_newest_inbox_accepts_rt_ed_record_changes(self) -> None:
        newest = self.copy_source()
        path = newest / DEFECTS_NAME
        path.write_text(
            path.read_text().replace(
                "Record RT-ED-001.",
                "Record RT-ED-001 changed.",
            )
        )
        self.set_newest_checkpoint(newest, unconsumed=1)
        result = self.run_check(newest_source=newest)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("unconsumed=1", result.stdout)

    def test_newest_inbox_rejects_record_deletion(self) -> None:
        newest = self.copy_source()
        path = newest / DEFECTS_NAME
        content = path.read_text()
        content = content[: content.index("### LOC-019")]
        path.write_text(content)
        self.set_newest_checkpoint(newest, unconsumed=1)
        result = self.run_check(newest_source=newest)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "newest Editor inbox deletes consumed records: LOC-019",
            result.stderr,
        )

    def test_newest_inbox_rejects_rt_ed_record_deletion(self) -> None:
        newest = self.copy_source()
        path = newest / DEFECTS_NAME
        content = path.read_text()
        errors: list[str] = []
        section = CHECKER.parse_defect_sections_text(
            content,
            "test source",
            errors,
        )["RT-ED-007"]
        self.assertEqual(errors, [])
        path.write_text(content.replace(section, "", 1))
        self.set_newest_checkpoint(newest, unconsumed=0)
        result = self.run_check(newest_source=newest)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "newest Editor inbox deletes consumed records: RT-ED-007",
            result.stderr,
        )

    def test_newest_source_hash_must_match_atlas(self) -> None:
        newest = self.copy_source()
        path = newest / DEFECTS_NAME
        path.write_text(path.read_text() + "\nchanged without checkpoint\n")
        result = self.run_check(newest_source=newest)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "newest_available_runtime_defects_sha256",
            result.stderr,
        )

    def test_newest_inbox_rejects_reserved_or_pre_future_loc_ids(self) -> None:
        for defect_id in ("LOC-000", "LOC-010"):
            with self.subTest(defect_id=defect_id):
                newest = self.copy_source()
                with (newest / DEFECTS_NAME).open("a") as source:
                    source.write(
                        f"\n### {defect_id} — Invalid intake\n\n"
                        f"Record {defect_id}.\n"
                    )
                self.set_newest_checkpoint(newest, unconsumed=1)
                result = self.run_check(newest_source=newest)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    f"newest Editor inbox has invalid future defect ids: "
                    f"{defect_id}",
                    result.stderr,
                )
                shutil.rmtree(newest)

    def test_program_contract_does_not_require_editor_consumption(self) -> None:
        self.atlas.write_text(
            self.atlas.read_text().replace(
                "editor_consumption_required = false",
                "editor_consumption_required = true",
            )
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "program editor_consumption_required is True; expected False",
            result.stderr,
        )

    def test_complete_enough_reported_new_loc_is_allowed_without_loosening_ids(
        self,
    ) -> None:
        self.append_consumed_defect("LOC-020")
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("defects=26", result.stdout)

    def test_future_source_completeness_does_not_trust_atlas_result_fields(
        self,
    ) -> None:
        self.append_consumed_defect("LOC-020")
        self.make_future_row_incomplete("LOC-020", state="reported")
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_incomplete_reported_new_loc_must_enter_needs_evidence(self) -> None:
        self.append_consumed_defect(
            "LOC-020",
            complete_source_record=False,
        )
        self.make_future_row_incomplete("LOC-020", state="reported")
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "LOC-020 committed inbox source record lacks a separately "
            "labeled full Editor SHA",
            result.stderr,
        )
        self.assertIn("state must be intake-needs-evidence", result.stderr)

    def test_incomplete_new_loc_is_valid_in_needs_evidence_state(self) -> None:
        self.append_consumed_defect(
            "LOC-020",
            complete_source_record=False,
        )
        self.make_future_row_incomplete(
            "LOC-020",
            state="intake-needs-evidence",
        )
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_future_source_record_requires_each_committed_evidence_field(
        self,
    ) -> None:
        complete = textwrap.dedent(
            f"""
            ### LOC-020 — Exact future row

            - Editor SHA: `{'b' * 40}`
            - Runtime SHA: `{'a' * 40}`
            - Command: `cargo test -p nuxie loc_020`
            - Observation: deterministic failure.
            """
        ).lstrip()
        cases = {
            "runtime-sha": (
                complete.replace("a" * 40, "short"),
                "a separately labeled full Runtime SHA",
            ),
            "editor-sha": (
                complete.replace("b" * 40, "short"),
                "a separately labeled full Editor SHA",
            ),
            "command": (
                complete.replace(
                    "- Command: `cargo test -p nuxie loc_020`",
                    "- Command: cargo test -p nuxie loc_020",
                ),
                "a labeled command bullet with nonempty inline code",
            ),
            "empty-command-code": (
                complete.replace(
                    "`cargo test -p nuxie loc_020`",
                    "`   `",
                ),
                "a labeled command bullet with nonempty inline code",
            ),
            "evidence": (
                complete.replace(
                    "- Observation: deterministic failure.",
                    "Deterministic failure.",
                ),
                "a labeled result/evidence/observation/failure/deficiency/"
                "classification bullet",
            ),
        }
        for name, (section, expected) in cases.items():
            with self.subTest(name=name):
                errors: list[str] = []
                CHECKER.validate_future_source_record(
                    "LOC-020",
                    "reported",
                    section,
                    errors,
                )
                self.assertTrue(errors)
                self.assertIn(expected, errors[0])

    def test_prior_v2_checkpoint_applies_evidence_to_changed_existing_loc(
        self,
    ) -> None:
        editor = pathlib.Path(self.temp.name) / "editor-intake-delta"
        self.init_git_repo(editor)
        plans = editor / "plans"
        plans.mkdir()
        old_source = textwrap.dedent(
            """
            # Runtime defects

            ### LOC-001 — Legacy record

            Legacy evidence syntax.

            ### RT-ED-001 — Stable runtime record

            Stable runtime evidence.
            """
        ).lstrip()
        (plans / DEFECTS_NAME).write_text(old_source)
        subprocess.run(["git", "add", "plans"], cwd=editor, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "prior v2 inbox"],
            cwd=editor,
            check=True,
        )
        prior_ref = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=editor,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        prior_atlas = {
            "schema": "nuxie.editor-next.runtime-defect-atlas/v2",
            "version": 2,
            "inbox": {
                "last_consumed_editor_ref": prior_ref,
                "runtime_defects_path": f"plans/{DEFECTS_NAME}",
            },
        }
        changed_source = old_source.replace(
            "Legacy evidence syntax.",
            "Changed existing LOC without canonical evidence.",
        )
        parse_errors: list[str] = []
        changed_sections = CHECKER.parse_defect_sections_text(
            changed_source,
            "current",
            parse_errors,
        )
        self.assertEqual(parse_errors, [])

        rows = [{"id": "LOC-001", "state": "reported"}]
        errors: list[str] = []
        CHECKER.validate_consumed_intake_delta(
            prior_atlas,
            rows,
            changed_sections,
            editor,
            errors,
        )
        self.assertTrue(errors)
        self.assertIn(
            "LOC-001 committed inbox source record lacks",
            errors[0],
        )

        rows[0]["state"] = "intake-needs-evidence"
        errors = []
        CHECKER.validate_consumed_intake_delta(
            prior_atlas,
            rows,
            changed_sections,
            editor,
            errors,
        )
        self.assertEqual(errors, [])

        complete_record = textwrap.dedent(
            f"""
            ### LOC-001 — Changed complete record

            - Editor SHA: `{prior_ref}`
            - Runtime SHA: `{'a' * 40}`
            - Exact command: `cargo test -p nuxie loc_001`
            - Result: deterministic failure.
            """
        ).lstrip()
        complete_source = old_source.replace(
            textwrap.dedent(
                """
                ### LOC-001 — Legacy record

                Legacy evidence syntax.
                """
            ).lstrip(),
            complete_record,
        )
        parse_errors = []
        complete_sections = CHECKER.parse_defect_sections_text(
            complete_source,
            "current",
            parse_errors,
        )
        self.assertEqual(parse_errors, [])
        rows[0]["state"] = "reported"
        errors = []
        CHECKER.validate_consumed_intake_delta(
            prior_atlas,
            rows,
            complete_sections,
            editor,
            errors,
        )
        self.assertEqual(errors, [])

        changed_runtime = old_source.replace(
            textwrap.dedent(
                """
                ### RT-ED-001 — Stable runtime record

                Stable runtime evidence.
                """
            ).lstrip(),
            textwrap.dedent(
                f"""
                ### RT-ED-001 — Changed complete runtime record

                - Editor SHA: `{prior_ref}`
                - Runtime SHA: `{'a' * 40}`
                - Exact command: `cargo test -p nuxie rt_ed_001`
                - Result: deterministic failure.
                """
            ).lstrip(),
        )
        runtime_sections = CHECKER.parse_defect_sections_text(
            changed_runtime,
            "current",
            [],
        )
        runtime_rows = rows + [{"id": "RT-ED-001", "state": "reported"}]
        errors = []
        CHECKER.validate_consumed_intake_delta(
            prior_atlas,
            runtime_rows,
            runtime_sections,
            editor,
            errors,
        )
        self.assertEqual(errors, [])

        incomplete_runtime = old_source.replace(
            "Stable runtime evidence.",
            "Changed RT-ED evidence without canonical provenance.",
        )
        runtime_sections = CHECKER.parse_defect_sections_text(
            incomplete_runtime,
            "current",
            [],
        )
        errors = []
        CHECKER.validate_consumed_intake_delta(
            prior_atlas,
            runtime_rows,
            runtime_sections,
            editor,
            errors,
        )
        self.assertTrue(
            any(
                "RT-ED-001 committed inbox source record lacks" in error
                for error in errors
            ),
            errors,
        )

        new_source = (
            old_source
            + "\n### LOC-020 — New incomplete record\n\n"
            + "Missing canonical evidence.\n"
        )
        new_sections = CHECKER.parse_defect_sections_text(
            new_source,
            "current",
            [],
        )
        errors = []
        CHECKER.validate_consumed_intake_delta(
            prior_atlas,
            [{"id": "LOC-020", "state": "reported"}],
            new_sections,
            editor,
            errors,
        )
        self.assertTrue(
            any(
                "LOC-020 committed inbox source record lacks" in error
                for error in errors
            ),
            errors,
        )

    def test_current_inbox_template_preserves_the_same_evidence_contract(
        self,
    ) -> None:
        section = textwrap.dedent(
            f"""
            ### RT-ED-008 — Current template

            - Exact Editor/runtime checkpoint: assembly
              `{'b' * 40}` with runtime pin `{'a' * 40}`.
            - Unchanged runtime reproducer:
              `cargo test -p nuxie rt_ed_008 -- --exact`.
            - Current classification: **confirmed runtime defect**.
            """
        ).lstrip()
        errors: list[str] = []
        CHECKER.validate_future_source_record(
            "RT-ED-008",
            "reported",
            section,
            errors,
        )
        self.assertEqual(errors, [])

    def test_combined_checkpoint_needs_distinct_editor_and_runtime_shas(
        self,
    ) -> None:
        section = textwrap.dedent(
            f"""
            ### RT-ED-008 — Incomplete combined checkpoint

            - Exact Editor/runtime checkpoint: `{'b' * 40}`.
            - Unchanged runtime reproducer: `cargo test rt_ed_008`.
            - Current classification: confirmed runtime defect.
            """
        ).lstrip()
        errors: list[str] = []
        CHECKER.validate_future_source_record(
            "RT-ED-008",
            "reported",
            section,
            errors,
        )
        self.assertTrue(errors)
        self.assertIn("full Runtime SHA", errors[0])

    def test_unrelated_continuation_sha_does_not_supply_runtime_role(
        self,
    ) -> None:
        section = textwrap.dedent(
            f"""
            ### RT-ED-008 — Unrelated SHA

            - Exact Editor checkpoint: `{'b' * 40}`.
              Unrelated artifact: `{'a' * 40}`.
            - Unchanged runtime reproducer: `cargo test rt_ed_008`.
            - Current classification: confirmed runtime defect.
            """
        ).lstrip()
        errors: list[str] = []
        CHECKER.validate_future_source_record(
            "RT-ED-008",
            "reported",
            section,
            errors,
        )
        self.assertTrue(errors)
        self.assertIn("full Runtime SHA", errors[0])

    def test_runtime_pin_cannot_supply_both_provenance_roles(self) -> None:
        section = textwrap.dedent(
            f"""
            ### RT-ED-008 — One provenance role

            - Exact Editor provenance: runtime pin `{'a' * 40}`.
            - Unchanged runtime reproducer: `cargo test rt_ed_008`.
            - Current classification: confirmed runtime defect.
            """
        ).lstrip()
        errors: list[str] = []
        CHECKER.validate_future_source_record(
            "RT-ED-008",
            "reported",
            section,
            errors,
        )
        self.assertTrue(errors)
        self.assertIn("full Editor SHA", errors[0])

        separated_marker = textwrap.dedent(
            f"""
            ### RT-ED-008 — Marker is not enough

            - Exact Editor provenance: assembly base notes;
              pinned C++ reference `{'c' * 40}`.
            - Runtime pin: `{'a' * 40}`.
            - Unchanged runtime reproducer: `cargo test rt_ed_008`.
            - Current classification: confirmed runtime defect.
            """
        ).lstrip()
        errors = []
        CHECKER.validate_future_source_record(
            "RT-ED-008",
            "reported",
            separated_marker,
            errors,
        )
        self.assertTrue(errors)
        self.assertIn("full Editor SHA", errors[0])

        cplusplus_laundering = textwrap.dedent(
            f"""
            ### RT-ED-008 — C++ is not Editor provenance

            - Exact Editor provenance: runtime pin `{'a' * 40}`;
              pinned C++ reference `{'c' * 40}`.
            - Runtime pin: `{'a' * 40}`.
            - Unchanged runtime reproducer: `cargo test rt_ed_008`.
            - Current classification: confirmed runtime defect.
            """
        ).lstrip()
        errors = []
        CHECKER.validate_future_source_record(
            "RT-ED-008",
            "reported",
            cplusplus_laundering,
            errors,
        )
        self.assertTrue(errors)
        self.assertIn("full Editor SHA", errors[0])

    def test_fixture_code_span_does_not_supply_command_role(self) -> None:
        section = textwrap.dedent(
            f"""
            ### RT-ED-008 — No command

            - Editor SHA: `{'b' * 40}`
            - Runtime SHA: `{'a' * 40}`
            - Minimal unchanged fixture: `fixtures/rt_ed_008.riv`.
            - Current classification: confirmed runtime defect.
            """
        ).lstrip()
        errors: list[str] = []
        CHECKER.validate_future_source_record(
            "RT-ED-008",
            "reported",
            section,
            errors,
        )
        self.assertTrue(errors)
        self.assertIn("command bullet", errors[0])

    def test_unlisted_labels_do_not_launder_command_or_evidence(self) -> None:
        section = textwrap.dedent(
            f"""
            ### RT-ED-008 — Misleading labels

            - Editor SHA: `{'b' * 40}`
            - Runtime SHA: `{'a' * 40}`
            - Not a command: `cargo test rt_ed_008`.
            - Unevidenced assertion: deterministic failure.
            """
        ).lstrip()
        errors: list[str] = []
        CHECKER.validate_future_source_record(
            "RT-ED-008",
            "reported",
            section,
            errors,
        )
        self.assertTrue(errors)
        self.assertIn("command bullet", errors[0])
        self.assertIn("classification bullet", errors[0])

    def test_record_comparison_ignores_only_canonical_defect_anchors(
        self,
    ) -> None:
        section = "### LOC-001 — Record\n\nEvidence.\n"
        anchored = section + '\n<a id="loc-002"></a>\n'
        self.assertEqual(
            CHECKER.normalized_source_record(section),
            CHECKER.normalized_source_record(anchored),
        )
        self.assertNotEqual(
            CHECKER.normalized_source_record(section),
            CHECKER.normalized_source_record(
                section + "\n<a id=\"unrelated\"></a>\n"
            ),
        )
        fenced = (
            section
            + "\n```html\n"
            + '<a id="loc-002"></a>\n'
            + "```\n"
        )
        changed_fenced = fenced.replace("loc-002", "loc-003")
        self.assertNotEqual(
            CHECKER.normalized_source_record(fenced),
            CHECKER.normalized_source_record(changed_fenced),
        )

    def test_markdown_parser_ignores_defect_headings_inside_fences(self) -> None:
        path = self.source / DEFECTS_NAME
        path.write_text(
            path.read_text()
            + textwrap.dedent(
                """

                ```markdown
                ### LOC-020 — Not an inbox row
                ```

                ~~~markdown
                ### LOC-021 — Also not an inbox row
                ~~~
                """
            )
        )
        self.refresh_source_bindings()
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_fence_markers_use_commonmark_indentation_columns(self) -> None:
        invalid_prefixes = ("\t", " \t", "  \t", "   \t")
        for prefix in invalid_prefixes:
            with self.subTest(kind="opener", prefix=repr(prefix)):
                errors: list[str] = []
                sections = CHECKER.parse_defect_sections_text(
                    f"{prefix}```markdown\n"
                    "### LOC-020 — Visible after indented code\n",
                    "fixture",
                    errors,
                )
                self.assertEqual(errors, [])
                self.assertIn("LOC-020", sections)

            with self.subTest(kind="closer", prefix=repr(prefix)):
                evidence = (
                    "### LOC-020 — Forged evidence\n\n"
                    "```text\n"
                    f"{prefix}```\n"
                    f"- Editor SHA: `{'b' * 40}`\n"
                    f"- Runtime SHA: `{'a' * 40}`\n"
                    "- Exact command: `cargo test forged`\n"
                    "- Result: forged after a pseudo-close.\n"
                    "```\n"
                )
                evidence_errors: list[str] = []
                CHECKER.validate_future_source_record(
                    "LOC-020",
                    "reported",
                    evidence,
                    evidence_errors,
                )
                self.assertTrue(evidence_errors)
                self.assertIn(
                    "a separately labeled full Editor SHA",
                    evidence_errors[0],
                )

        hidden = CHECKER.markdown_visible_lines(
            "   ```markdown\n### LOC-020 — Hidden\n   ```\n"
        )
        self.assertTrue(all(not visible for _, _, visible in hidden))

    def test_backtick_in_info_string_does_not_open_commonmark_fence(
        self,
    ) -> None:
        errors: list[str] = []
        sections = CHECKER.parse_defect_sections_text(
            "```bad`info\n"
            "### LOC-020 — Visible after invalid opener\n"
            f"- Editor SHA: `{'b' * 40}`\n"
            f"- Runtime SHA: `{'a' * 40}`\n"
            "- Exact command: `cargo test visible`\n"
            "- Result: visible after invalid opener.\n",
            "fixture",
            errors,
        )
        self.assertEqual(errors, [])
        self.assertIn("LOC-020", sections)
        evidence_errors: list[str] = []
        CHECKER.validate_future_source_record(
            "LOC-020",
            "reported",
            sections["LOC-020"],
            evidence_errors,
        )
        self.assertEqual(evidence_errors, [])

    def test_markdown_parser_rejects_suffixed_defect_heading_alias(
        self,
    ) -> None:
        path = self.source / DEFECTS_NAME
        path.write_text(
            path.read_text()
            + "\n### LOC-020-EXAMPLE — Must not truncate\n\nEvidence.\n"
        )
        self.refresh_source_bindings()
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "noncanonical defect-like heading "
            "'### LOC-020-EXAMPLE — Must not truncate'",
            result.stderr,
        )

    def test_defect_section_ends_at_every_h1_through_h3_boundary(self) -> None:
        for level in (1, 2, 3):
            with self.subTest(level=level):
                path = pathlib.Path(self.temp.name) / f"boundary-{level}.md"
                path.write_text(
                    "### LOC-020 — Incomplete record\n\n"
                    + "#" * level
                    + " Unrelated section\n\n"
                    + f"- Editor SHA: `{'b' * 40}`\n"
                    + f"- Runtime SHA: `{'a' * 40}`\n"
                    + "- Exact command: `cargo test hidden`\n"
                    + "- Result: hidden after boundary.\n"
                )
                errors: list[str] = []
                sections = CHECKER.parse_defect_sections(
                    path,
                    "fixture",
                    errors,
                )
                self.assertEqual(errors, [])
                evidence_errors: list[str] = []
                CHECKER.validate_future_source_record(
                    "LOC-020",
                    "reported",
                    sections["LOC-020"],
                    evidence_errors,
                )
                self.assertTrue(evidence_errors)
                self.assertIn(
                    "a separately labeled full Editor SHA",
                    evidence_errors[0],
                )

    def test_comments_and_fences_cannot_supply_intake_evidence(self) -> None:
        section = textwrap.dedent(
            f"""
            ### LOC-020 — Hidden evidence

            <!--
            - Editor SHA: `{'b' * 40}`
            - Runtime SHA: `{'a' * 40}`
            - Exact command: `cargo test commented`
            - Result: commented.
            -->

            ```text
            - Editor SHA: `{'c' * 40}`
            - Runtime SHA: `{'d' * 40}`
            - Exact command: `cargo test fenced`
            - Evidence: fenced.
            ```
            """
        ).lstrip()
        errors: list[str] = []
        CHECKER.validate_future_source_record(
            "LOC-020",
            "reported",
            section,
            errors,
        )
        self.assertTrue(errors)
        self.assertIn("a separately labeled full Editor SHA", errors[0])
        self.assertIn("a separately labeled full Runtime SHA", errors[0])
        self.assertIn("a labeled command bullet", errors[0])

    def test_block_comment_closing_line_suffix_remains_hidden(self) -> None:
        forged_heading = CHECKER.parse_defect_sections_text(
            "<!-- closed -->### LOC-020 — Forged heading\n"
            "### LOC-021 — Visible heading\n",
            "fixture",
            [],
        )
        self.assertEqual(set(forged_heading), {"LOC-021"})

        section = (
            "### LOC-020 — Hidden comment suffix\n\n"
            f"<!-- closed -->- Editor SHA: `{'b' * 40}`\n"
            f"<!-- closed -->- Runtime SHA: `{'a' * 40}`\n"
            "<!-- closed -->- Exact command: `cargo test forged`\n"
            "<!-- closed -->- Result: forged suffix.\n"
        )
        errors: list[str] = []
        CHECKER.validate_future_source_record(
            "LOC-020",
            "reported",
            section,
            errors,
        )
        self.assertTrue(errors)
        self.assertIn("a separately labeled full Editor SHA", errors[0])
        self.assertIn("a separately labeled full Runtime SHA", errors[0])
        self.assertIn("a labeled command bullet", errors[0])

    def test_indented_code_and_raw_pre_cannot_supply_intake_evidence(
        self,
    ) -> None:
        evidence = (
            f"- Editor SHA: `{'b' * 40}`\n"
            f"- Runtime SHA: `{'a' * 40}`\n"
            "- Exact command: `cargo test hidden`\n"
            "- Result: hidden evidence.\n"
        )
        sections = {
            "four-spaces": (
                "### LOC-020 — Hidden evidence\n\n"
                + textwrap.indent(evidence, "    ")
            ),
            "tab": (
                "### LOC-020 — Hidden evidence\n\n"
                + textwrap.indent(evidence, "\t")
            ),
            "one-space-tab": (
                "### LOC-020 — Hidden evidence\n\n"
                + textwrap.indent(evidence, " \t")
            ),
            "two-spaces-tab": (
                "### LOC-020 — Hidden evidence\n\n"
                + textwrap.indent(evidence, "  \t")
            ),
            "three-spaces-tab": (
                "### LOC-020 — Hidden evidence\n\n"
                + textwrap.indent(evidence, "   \t")
            ),
            "raw-pre": (
                "### LOC-020 — Hidden evidence\n\n"
                "<pre>\n"
                + evidence
                + "</pre>\n"
            ),
        }
        for name, section in sections.items():
            with self.subTest(name=name):
                errors: list[str] = []
                CHECKER.validate_future_source_record(
                    "LOC-020",
                    "reported",
                    section,
                    errors,
                )
                self.assertTrue(errors)
                self.assertIn(
                    "a separately labeled full Editor SHA",
                    errors[0],
                )
                self.assertIn(
                    "a separately labeled full Runtime SHA",
                    errors[0],
                )
                self.assertIn("a labeled command bullet", errors[0])

    def test_commonmark_html_block_types_cannot_expose_headings_or_evidence(
        self,
    ) -> None:
        hidden_heading = "### LOC-020 — Hidden HTML heading\n"
        evidence = (
            f"- Editor SHA: `{'b' * 40}`\n"
            f"- Runtime SHA: `{'a' * 40}`\n"
            "- Exact command: `cargo test hidden-html`\n"
            "- Result: hidden HTML evidence.\n"
        )

        def blocks(payload: str) -> dict[str, str]:
            return {
                "type-1": f"<ScRiPt>\n{payload}</sCrIpT>\n\n",
                "type-2": f"<!--\n{payload}-->\n\n",
                "type-3": f"<?processor\n{payload}?>\n\n",
                "type-4": f"<!DECLARATION\n{payload}>\n\n",
                "type-5": f"<![CDATA[\n{payload}]]>\n\n",
                "type-6": f"<DiV>\n{payload}</dIv>\n\n",
                "type-7": (
                    '<x-widget data-value="exact">\n'
                    f"{payload}</x-widget>\n\n"
                ),
            }

        for kind, block in blocks(hidden_heading).items():
            with self.subTest(kind=kind, surface="heading"):
                errors: list[str] = []
                sections = CHECKER.parse_defect_sections_text(
                    block + "### LOC-021 — Visible after HTML block\n",
                    "fixture",
                    errors,
                )
                self.assertEqual(errors, [])
                self.assertEqual(set(sections), {"LOC-021"})

        for kind, block in blocks(evidence).items():
            with self.subTest(kind=kind, surface="evidence"):
                errors: list[str] = []
                CHECKER.validate_future_source_record(
                    "LOC-020",
                    "reported",
                    "### LOC-020 — Hidden HTML evidence\n\n" + block,
                    errors,
                )
                self.assertTrue(errors)
                self.assertIn(
                    "a separately labeled full Editor SHA",
                    errors[0],
                )
                self.assertIn(
                    "a separately labeled full Runtime SHA",
                    errors[0],
                )
                self.assertIn("a labeled command bullet", errors[0])

    def test_non_ascii_whitespace_does_not_end_raw_html_block(self) -> None:
        section = (
            "### LOC-020 — Hidden after nonblank HTML line\n\n"
            "<div>\n"
            "\N{NO-BREAK SPACE}\n"
            f"- Editor SHA: `{'b' * 40}`\n"
            f"- Runtime SHA: `{'a' * 40}`\n"
            "- Exact command: `cargo test forged`\n"
            "- Result: forged after a nonblank HTML line.\n"
            "\n"
        )
        errors: list[str] = []
        CHECKER.validate_future_source_record(
            "LOC-020",
            "reported",
            section,
            errors,
        )
        self.assertTrue(errors)
        self.assertIn("a separately labeled full Editor SHA", errors[0])
        self.assertIn("a separately labeled full Runtime SHA", errors[0])
        self.assertIn("a labeled command bullet", errors[0])

    def test_type_seven_html_block_cannot_interrupt_paragraph(self) -> None:
        section = (
            "### LOC-020 — Visible paragraph continuation\n\n"
            "Paragraph text.\n"
            "<x-widget>\n"
            f"- Editor SHA: `{'b' * 40}`\n"
            f"- Runtime SHA: `{'a' * 40}`\n"
            "- Exact command: `cargo test visible`\n"
            "- Result: visible because type seven cannot interrupt.\n"
        )
        errors: list[str] = []
        CHECKER.validate_future_source_record(
            "LOC-020",
            "reported",
            section,
            errors,
        )
        self.assertEqual(errors, [])

        hidden_after_heading = (
            "### LOC-020 — HTML starts after heading\n"
            "<x-widget>\n"
            f"- Editor SHA: `{'b' * 40}`\n"
            f"- Runtime SHA: `{'a' * 40}`\n"
            "- Exact command: `cargo test hidden`\n"
            "- Result: hidden HTML block evidence.\n"
            "\n"
        )
        hidden_errors: list[str] = []
        CHECKER.validate_future_source_record(
            "LOC-020",
            "reported",
            hidden_after_heading,
            hidden_errors,
        )
        self.assertTrue(hidden_errors)
        self.assertIn(
            "a separately labeled full Editor SHA",
            hidden_errors[0],
        )

    def test_only_commonmark_line_endings_split_evidence_lines(self) -> None:
        separators = (
            "\v",
            "\f",
            "\x1c",
            "\x1d",
            "\x1e",
            "\x85",
            "\N{LINE SEPARATOR}",
            "\N{PARAGRAPH SEPARATOR}",
        )
        for separator in separators:
            with self.subTest(separator=repr(separator)):
                section = (
                    "### LOC-020 — Unicode is not a line ending\n\n"
                    f"prefix{separator}- Editor SHA: `{'b' * 40}`\n"
                    f"- Runtime SHA: `{'a' * 40}`\n"
                    "- Exact command: `cargo test forged`\n"
                    "- Result: only the Editor SHA is forged.\n"
                )
                errors: list[str] = []
                CHECKER.validate_future_source_record(
                    "LOC-020",
                    "reported",
                    section,
                    errors,
                )
                self.assertTrue(errors)
                self.assertIn(
                    "a separately labeled full Editor SHA",
                    errors[0],
                )

    def test_container_hidden_evidence_is_not_top_level_evidence(self) -> None:
        evidence = (
            f"  - Editor SHA: `{'b' * 40}`\n"
            f"  - Runtime SHA: `{'a' * 40}`\n"
            "  - Exact command: `cargo test hidden-container`\n"
            "  - Result: hidden container evidence.\n"
        )
        sections = {
            "fence": (
                "### LOC-020 — List-contained fence\n\n"
                "- ```text\n"
                + evidence
                + "  ```\n"
            ),
            "raw-html": (
                "### LOC-020 — List-contained HTML\n\n"
                "- <div>\n"
                + evidence
                + "  </div>\n"
            ),
        }
        for kind, section in sections.items():
            with self.subTest(kind=kind):
                errors: list[str] = []
                CHECKER.validate_future_source_record(
                    "LOC-020",
                    "reported",
                    section,
                    errors,
                )
                self.assertTrue(errors)
                self.assertIn(
                    "a separately labeled full Editor SHA",
                    errors[0],
                )
                self.assertIn(
                    "a separately labeled full Runtime SHA",
                    errors[0],
                )
                self.assertIn("a labeled command bullet", errors[0])

    def test_type_seven_html_starts_after_nonparagraph_blocks(self) -> None:
        prefixes = {
            "thematic-break": "***\n",
            "unordered-list": "- list item\n",
            "ordered-list": "1. list item\n",
            "zero-padded-one-list": "01. list item\n",
            "blockquote": "> quoted paragraph\n",
            "blockquote-without-space": ">quoted paragraph\n",
            "link-reference": "[label]: /destination\n",
            "escaped-link-reference": (
                "[Foo*bar\\]]:my_(url) 'title (with parens)'\n"
            ),
            "multiline-link-reference": "[\nfoo\n]: /url\n",
            "next-line-link-title": "[foo]: /url\n  \"title\"\n",
            "multiline-link-title": "[foo]: /url '\ntitle\n'\n",
        }
        evidence = (
            f"- Editor SHA: `{'b' * 40}`\n"
            f"- Runtime SHA: `{'a' * 40}`\n"
            "- Exact command: `cargo test hidden-html`\n"
            "- Result: hidden by type seven HTML.\n"
        )
        for kind, prefix in prefixes.items():
            with self.subTest(kind=kind):
                section = (
                    "### LOC-020 — HTML after a block\n\n"
                    + prefix
                    + "<x-widget>\n"
                    + evidence
                    + "\n"
                )
                errors: list[str] = []
                CHECKER.validate_future_source_record(
                    "LOC-020",
                    "reported",
                    section,
                    errors,
                )
                self.assertTrue(errors)
                self.assertIn(
                    "a separately labeled full Editor SHA",
                    errors[0],
                )

    def test_noninterrupting_markers_keep_paragraph_open_for_type_seven(
        self,
    ) -> None:
        continuations = {
            "link-reference": "[label]: /destination\n",
            "non-one-ordered-list": "2. paragraph text\n",
            "empty-bullet": "*\n",
            "empty-one-list": "1.\n",
        }
        evidence = (
            f"- Editor SHA: `{'b' * 40}`\n"
            f"- Runtime SHA: `{'a' * 40}`\n"
            "- Exact command: `cargo test visible-html`\n"
            "- Result: visible because the paragraph stays open.\n"
        )
        for kind, continuation in continuations.items():
            with self.subTest(kind=kind):
                section = (
                    "### LOC-020 — HTML remains in the paragraph\n\n"
                    "Paragraph text.\n"
                    + continuation
                    + "<x-widget>\n"
                    + evidence
                )
                errors: list[str] = []
                CHECKER.validate_future_source_record(
                    "LOC-020",
                    "reported",
                    section,
                    errors,
                )
                self.assertEqual(errors, [])

    def test_any_type_one_end_tag_closes_type_one_html(self) -> None:
        section = (
            "### LOC-020 — Mismatched type one close\n\n"
            "<script>\n"
            "</pre>\n"
            f"- Editor SHA: `{'b' * 40}`\n"
            f"- Runtime SHA: `{'a' * 40}`\n"
            "- Exact command: `cargo test visible-after-html`\n"
            "- Result: visible after any type one closing tag.\n"
        )
        errors: list[str] = []
        CHECKER.validate_future_source_record(
            "LOC-020",
            "reported",
            section,
            errors,
        )
        self.assertEqual(errors, [])

    def test_block_interruptions_cancel_multiline_link_candidates(
        self,
    ) -> None:
        evidence = (
            f"- Editor SHA: `{'b' * 40}`\n"
            f"- Runtime SHA: `{'a' * 40}`\n"
            "- Exact command: `cargo test forged-link-definition`\n"
            "- Result: forged inside a link definition.\n"
        )
        sections = {
            "multiline-label": (
                "### LOC-020 — Hidden link label\n\n"
                "[\n"
                + evidence
                + "]: /url\n"
            ),
            "inline-multiline-title": (
                "### LOC-020 — Hidden inline title\n\n"
                '[foo]: /url "\n'
                + evidence
                + '"\n'
            ),
            "next-line-multiline-title": (
                "### LOC-020 — Hidden next-line title\n\n"
                "[foo]: /url\n"
                '  "\n'
                + evidence
                + '"\n'
            ),
        }
        for kind, section in sections.items():
            with self.subTest(kind=kind):
                errors: list[str] = []
                CHECKER.validate_future_source_record(
                    "LOC-020",
                    "reported",
                    section,
                    errors,
                )
                self.assertEqual(errors, [])

    def test_defect_section_ends_at_setext_h1_and_h2_boundaries(self) -> None:
        for underline in ("=============", "-------------"):
            with self.subTest(underline=underline):
                content = (
                    "### LOC-020 — Incomplete record\n\n"
                    "Unrelated section\n"
                    f"{underline}\n\n"
                    f"- Editor SHA: `{'b' * 40}`\n"
                    f"- Runtime SHA: `{'a' * 40}`\n"
                    "- Exact command: `cargo test hidden`\n"
                    "- Result: hidden after boundary.\n"
                )
                errors: list[str] = []
                sections = CHECKER.parse_defect_sections_text(
                    content,
                    "fixture",
                    errors,
                )
                self.assertEqual(errors, [])
                evidence_errors: list[str] = []
                CHECKER.validate_future_source_record(
                    "LOC-020",
                    "reported",
                    sections["LOC-020"],
                    evidence_errors,
                )
                self.assertTrue(evidence_errors)
                self.assertIn(
                    "a separately labeled full Editor SHA",
                    evidence_errors[0],
                )

    def test_indented_and_wrong_level_defect_headings_are_rejected(self) -> None:
        headings = (
            "  ### LOC-020 — Indented",
            "# LOC-020 — H1",
            "## LOC-020 — H2",
            "#### LOC-020 — H4",
            "##### RT-ED-003 — H5",
        )
        for heading in headings:
            with self.subTest(heading=heading):
                path = pathlib.Path(self.temp.name) / (
                    "bad-heading-"
                    + hashlib.sha256(heading.encode()).hexdigest()
                    + ".md"
                )
                path.write_text(heading + "\n")
                errors: list[str] = []
                sections = CHECKER.parse_defect_sections(
                    path,
                    "fixture",
                    errors,
                )
                self.assertEqual(sections, {})
                self.assertTrue(
                    any("noncanonical defect-like heading" in error for error in errors),
                    errors,
                )

    def test_fabricated_atlas_only_loc_is_rejected(self) -> None:
        self.append_defect("LOC-020")
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "atlas has no exact consumed Editor inbox heading for: LOC-020",
            result.stderr,
        )

    def test_atlas_only_runtime_id_is_rejected_without_inbox_record(self) -> None:
        self.append_defect("RT-ED-008")
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "atlas has no exact consumed Editor inbox heading for: RT-ED-008",
            result.stderr,
        )

    def test_complete_consumed_future_runtime_record_is_accepted(self) -> None:
        self.append_consumed_defect("RT-ED-008")
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("defects=26", result.stdout)

    def test_arbitrary_tracking_record_type_is_rejected(self) -> None:
        headings = (
            "### BUG-1 — Not a canonical runtime defect\n",
            "### BUG_1 — Not a canonical runtime defect\n",
            "### BUG-1 —\n",
            "### BUG-1 - Not a canonical runtime defect\n",
            "### [BUG-1] — Not a canonical runtime defect\n",
            "### [LOC-020] — Alias is not canonical\n",
            "### [BUG-1](https://example.invalid) — Linked fake\n",
            "### <code>BUG-1</code> — HTML-wrapped fake\n",
            "### [BUG name] — Noncanonical record prefix\n",
            "BUG-1 — Not a canonical runtime defect\n"
            "========================================\n",
            "BUG_1 — Not a canonical runtime defect\n"
            "========================================\n",
            "[BUG-1] — Not a canonical runtime defect\n"
            "==========================================\n",
        )
        for heading in headings:
            with self.subTest(heading=heading):
                newest = self.copy_source()
                path = newest / DEFECTS_NAME
                path.write_text(
                    path.read_text()
                    + "\n"
                    + heading
                    + "\n- Editor SHA: `"
                    + "b" * 40
                    + "`\n- Runtime SHA: `"
                    + "a" * 40
                    + "`\n- Command: `cargo test bug_001`\n"
                    + "- Result: deterministic failure.\n"
                )
                self.set_newest_checkpoint(newest, unconsumed=0)
                result = self.run_check(newest_source=newest)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    "noncanonical defect-like heading",
                    result.stderr,
                )
                shutil.rmtree(newest)

    def test_consumed_inbox_rejects_wrapped_arbitrary_ids(self) -> None:
        headings = (
            "### __BUG-1__\n",
            "### <code>BUG-1</code>\n",
            "### [BUG-1][ref]\n",
            "### BUG-1—Title\n",
        )
        original_source = (self.source / DEFECTS_NAME).read_text()
        original_atlas = self.atlas.read_text()
        for heading in headings:
            with self.subTest(heading=heading):
                (self.source / DEFECTS_NAME).write_text(
                    original_source
                    + "\n"
                    + heading
                    + "\nArbitrary record body.\n"
                )
                self.atlas.write_text(original_atlas)
                self.refresh_source_bindings()
                result = self.run_check()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    "noncanonical defect-like heading",
                    result.stderr,
                )

    def test_loc_zero_and_reserved_loc_ten_are_rejected(self) -> None:
        for defect_id in ("LOC-000", "LOC-010"):
            with self.subTest(defect_id=defect_id):
                self.write_fixtures()
                self.write_atlas()
                self.append_defect(defect_id)
                result = self.run_check()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(
                    f"atlas has unexpected defect ids: {defect_id}",
                    result.stderr,
                )

    def test_formal_children_are_bound_to_ledger_runtime_dependencies(
        self,
    ) -> None:
        path = self.source / LEDGER_NAME
        ledger = json.loads(path.read_text())
        child = next(
            child
            for child in ledger["rows"][0]["children"]
            if child["id"] == "P04-C01"
        )
        child["runtimeDependencies"] = []
        path.write_text(json.dumps(ledger, indent=2) + "\n")
        self.refresh_source_bindings()
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "RT-ED-003 formal_children do not match parity-ledger "
            "runtimeDependencies",
            result.stderr,
        )

    def test_runtime_defects_are_structured_ledger_links(self) -> None:
        path = self.source / LEDGER_NAME
        ledger = json.loads(path.read_text())
        child = next(
            child
            for child in ledger["rows"][0]["children"]
            if child["id"] == "P04-C01"
        )
        child["runtimeDependencies"] = []
        child["runtimeDefects"] = [
            {
                "id": "RT-ED-003",
                "classification": "confirmed-runtime-defect",
            }
        ]
        path.write_text(json.dumps(ledger, indent=2) + "\n")
        self.refresh_source_bindings()
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_candidate_child_assertion_must_name_exact_loc_id(self) -> None:
        path = self.source / LEDGER_NAME
        ledger = json.loads(path.read_text())
        child = next(
            child
            for child in ledger["rows"][0]["children"]
            if child["id"] == "P13-C07"
        )
        child["assertion"] = child["assertion"].replace("LOC-001", "LOC one")
        path.write_text(json.dumps(ledger, indent=2) + "\n")
        self.refresh_source_bindings()
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "LOC-001 candidate child P13-C07 assertion does not contain "
            "the exact defect id LOC-001",
            result.stderr,
        )

    def test_source_artifact_paths_are_exact_and_cannot_escape(self) -> None:
        cases = {
            "absolute": (
                f'path = "{DEFECTS_NAME}"',
                f'path = "/tmp/{DEFECTS_NAME}"',
                "source artifact runtime-defects path must be a relative filename",
            ),
            "traversal": (
                f'path = "{DEFECTS_NAME}"',
                f'path = "../{DEFECTS_NAME}"',
                "source artifact runtime-defects path contains traversal",
            ),
            "wrong-filename": (
                f'path = "{DEFECTS_NAME}"',
                'path = "defects.md"',
                f"source artifact runtime-defects path must be {DEFECTS_NAME!r}",
            ),
            "alias": (
                f'path = "{DEFECTS_NAME}"',
                f'path = "{PROPOSAL_NAME}"',
                "duplicate source artifact paths",
            ),
        }
        for _, (old, new, expected) in cases.items():
            with self.subTest(case=_):
                self.write_atlas()
                self.atlas.write_text(
                    self.atlas.read_text().replace(old, new, 1)
                )
                result = self.run_check()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(expected, result.stderr)

    def test_source_artifact_cannot_resolve_through_a_symlink(self) -> None:
        artifact = self.source / DEFECTS_NAME
        outside = pathlib.Path(self.temp.name) / "outside-defects.md"
        outside.write_bytes(artifact.read_bytes())
        artifact.unlink()
        artifact.symlink_to(outside)
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "source artifact runtime-defects contains a symlink component",
            result.stderr,
        )

    def test_newest_inbox_source_cannot_resolve_through_a_symlink(
        self,
    ) -> None:
        newest = self.copy_source()
        path = newest / DEFECTS_NAME
        outside = pathlib.Path(self.temp.name) / "newest-outside-defects.md"
        outside.write_bytes(path.read_bytes())
        path.unlink()
        path.symlink_to(outside)
        self.set_newest_checkpoint(newest, unconsumed=0)
        result = self.run_check(newest_source=newest)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            f"newest Editor source {DEFECTS_NAME} contains a symlink component",
            result.stderr,
        )

    def test_cli_preserves_a_lexical_symlink_source_root(self) -> None:
        linked_source = pathlib.Path(self.temp.name) / "linked-plans"
        linked_source.symlink_to(self.source, target_is_directory=True)
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
            "--source-root",
            str(linked_source),
            "--newest-source-root",
            str(linked_source),
            "--expected-upstream-ref",
            PIN,
            "--test-mode",
        ]
        result = subprocess.run(
            command,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "source artifact cutover-proposal contains a symlink component",
            result.stderr,
        )

    def test_stimulus_files_reject_unsafe_and_symlink_paths(self) -> None:
        root = pathlib.Path(self.temp.name) / "stimulus-root"
        root.mkdir()
        outside = pathlib.Path(self.temp.name) / "outside.riv"
        outside.write_text("outside")
        (root / "linked.riv").symlink_to(outside)
        digest = hashlib.sha256(outside.read_bytes()).hexdigest()
        cases = {
            "absolute": (
                "/tmp/outside.riv",
                "has unsafe stimulus path '/tmp/outside.riv'",
            ),
            "traversal": (
                "../outside.riv",
                "has unsafe stimulus path '../outside.riv'",
            ),
            "symlink": (
                "linked.riv",
                "contains a symlink component",
            ),
        }
        for name, (relative, expected) in cases.items():
            with self.subTest(name=name):
                errors: list[str] = []
                CHECKER.validate_stimulus_files(
                    "fixture.loc-020",
                    "implemented",
                    [
                        {
                            "root": "repo",
                            "path": relative,
                            "sha256": digest,
                        }
                    ],
                    {"repo": root, "rive": None, "editor": None},
                    True,
                    errors,
                )
                self.assertTrue(any(expected in error for error in errors), errors)

    def test_stimuli_must_match_their_pinned_git_blobs(
        self,
    ) -> None:
        for root_name, pin_name in (
            ("repo", "investigation_base_ref"),
            ("editor", "source_snapshot_ref/last_consumed_editor_ref"),
            ("rive", "upstream_ref"),
        ):
            with self.subTest(root=root_name):
                root = pathlib.Path(self.temp.name) / f"{root_name}-stimulus-repo"
                self.init_git_repo(root)
                stimulus = root / "fixture.riv"
                stimulus.write_text("committed\n")
                subprocess.run(
                    ["git", "add", "fixture.riv"],
                    cwd=root,
                    check=True,
                )
                subprocess.run(
                    ["git", "commit", "-qm", "pinned stimulus"],
                    cwd=root,
                    check=True,
                )
                pin = subprocess.run(
                    ["git", "rev-parse", "HEAD"],
                    cwd=root,
                    text=True,
                    capture_output=True,
                    check=True,
                ).stdout.strip()

                stimulus.write_text("dirty\n")
                refreshed_hash = hashlib.sha256(
                    stimulus.read_bytes()
                ).hexdigest()
                errors: list[str] = []
                CHECKER.validate_stimulus_files(
                    "fixture.loc-020",
                    "implemented",
                    [
                        {
                            "root": root_name,
                            "path": "fixture.riv",
                            "sha256": refreshed_hash,
                        }
                    ],
                    {
                        "repo": root if root_name == "repo" else None,
                        "rive": root if root_name == "rive" else None,
                        "editor": root if root_name == "editor" else None,
                    },
                    True,
                    errors,
                    git_sources={
                        root_name: (root, pin, pin_name),
                    },
                )
                self.assertFalse(
                    any("registry records" in error for error in errors),
                    errors,
                )
                self.assertTrue(
                    any(
                        f"does not match pinned {pin_name} Git blob" in error
                        for error in errors
                    ),
                    errors,
                )

    def test_incomplete_intake_row_can_wait_for_evidence(self) -> None:
        content = self.atlas.read_text().replace(
            'state = "reported"\nowner_class = "runtime"',
            'state = "intake-needs-evidence"\nowner_class = "runtime"',
            1,
        )
        reported_history = textwrap.dedent(
            """
            [[defect.history]]
            state = "reported"
            actor = "editor-cutover"
            evidence = "source-artifact"
            """
        ).strip()
        intake_history = (
            reported_history
            + "\n\n"
            + textwrap.dedent(
                """
                [[defect.history]]
                state = "intake-needs-evidence"
                actor = "f-ed-intake"
                evidence = "committed inbox record lacks an executable reproducer"
                """
            ).strip()
        )
        self.atlas.write_text(
            content.replace(reported_history, intake_history, 1)
        )
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_intake_needs_evidence_cannot_skip_reproduction(self) -> None:
        errors: list[str] = []
        CHECKER.validate_history(
            "LOC-020",
            "qualified",
            [
                {
                    "state": "reported",
                    "actor": "editor-cutover",
                    "evidence": "source-artifact",
                },
                {
                    "state": "intake-needs-evidence",
                    "actor": "f-ed-intake",
                    "evidence": "missing command",
                },
                {
                    "state": "qualified",
                    "actor": "f-ed-executor",
                    "evidence": "invalid skip",
                },
            ],
            errors,
        )
        self.assertIn(
            "LOC-020 has illegal state transition "
            "intake-needs-evidence -> qualified",
            errors,
        )

    def test_runtime_owner_cannot_shortcut_qualified_to_handoff(self) -> None:
        errors: list[str] = []
        CHECKER.validate_history(
            "LOC-020",
            "handoff-ready",
            [
                {
                    "state": "reported",
                    "actor": "editor-cutover",
                    "evidence": "source-artifact",
                },
                {
                    "state": "reproduced",
                    "actor": "f-ed-executor",
                    "evidence": "reproducer",
                },
                {
                    "state": "qualified",
                    "actor": "f-ed-executor",
                    "evidence": "C++ oracle",
                },
                {
                    "state": "handoff-ready",
                    "actor": "f-ed-closeout",
                    "evidence": "invalid shortcut",
                },
            ],
            errors,
            owner_class="runtime",
        )
        self.assertIn(
            "LOC-020 has illegal state transition qualified -> handoff-ready",
            errors,
        )

    def test_editor_and_artifact_owners_may_handoff_after_qualification(
        self,
    ) -> None:
        for owner_class in ("editor", "artifact"):
            with self.subTest(owner_class=owner_class):
                errors: list[str] = []
                CHECKER.validate_history(
                    "LOC-020",
                    "handoff-ready",
                    [
                        {
                            "state": "reported",
                            "actor": "editor-cutover",
                            "evidence": "source-artifact",
                        },
                        {
                            "state": "reproduced",
                            "actor": "f-ed-executor",
                            "evidence": "reproducer",
                        },
                        {
                            "state": "qualified",
                            "actor": "f-ed-executor",
                            "evidence": "ownership proof",
                        },
                        {
                            "state": "handoff-ready",
                            "actor": "f-ed-closeout",
                            "evidence": "downstream disposition",
                        },
                    ],
                    errors,
                    owner_class=owner_class,
                )
                self.assertEqual(errors, [])

    def test_executor_green_history_requires_passing_executor_verification(
        self,
    ) -> None:
        errors: list[str] = []
        CHECKER.validate_history_verifications(
            "LOC-020",
            [{"state": "executor-green"}],
            {"status": "not-applicable", "reason": "Invalid shortcut."},
            {"status": "pending", "reason": "Not yet independently checked."},
            errors,
        )
        self.assertIn(
            "LOC-020 history contains executor-green but "
            "executor_verification is not pass",
            errors,
        )

    def test_orchestrator_history_requires_passing_independent_verification(
        self,
    ) -> None:
        errors: list[str] = []
        CHECKER.validate_history_verifications(
            "LOC-020",
            [
                {"state": "executor-green"},
                {"state": "orchestrator-verified"},
            ],
            {"status": "pass"},
            {"status": "not-applicable", "reason": "Invalid shortcut."},
            errors,
        )
        self.assertIn(
            "LOC-020 history contains orchestrator-verified but "
            "orchestrator_verification is not pass",
            errors,
        )

    def test_direct_handoff_and_nonrepair_history_do_not_require_gate_passes(
        self,
    ) -> None:
        errors: list[str] = []
        CHECKER.validate_history_verifications(
            "LOC-020",
            [
                {"state": "qualified"},
                {"state": "handoff-ready"},
                {"state": "closed"},
            ],
            {"status": "not-applicable", "reason": "Editor-owned."},
            {"status": "not-applicable", "reason": "Editor-owned."},
            errors,
        )
        CHECKER.validate_history_verifications(
            "LOC-021",
            [
                {"state": "reproduced"},
                {"state": "stale-oracle"},
                {"state": "closed"},
            ],
            {"status": "not-applicable", "reason": "No repair."},
            {"status": "not-applicable", "reason": "No repair."},
            errors,
        )
        self.assertEqual(errors, [])

    def test_independently_verified_repair_can_close_before_editor_consumes(
        self,
    ) -> None:
        errors: list[str] = []
        CHECKER.validate_history(
            "LOC-020",
            "closed",
            [
                {
                    "state": "reported",
                    "actor": "editor-cutover",
                    "evidence": "source-artifact",
                },
                {
                    "state": "reproduced",
                    "actor": "f-ed-executor",
                    "evidence": "exact reproducer",
                },
                {
                    "state": "qualified",
                    "actor": "f-ed-executor",
                    "evidence": "pinned C++ differential",
                },
                {
                    "state": "mapped",
                    "actor": "f-ed-executor",
                    "evidence": "owner-family map",
                },
                {
                    "state": "executor-green",
                    "actor": "f-ed-executor",
                    "evidence": "required gates",
                },
                {
                    "state": "orchestrator-verified",
                    "actor": "independent-orchestrator",
                    "evidence": "independent gates",
                },
                {
                    "state": "handoff-ready",
                    "actor": "f-ed-closeout",
                    "evidence": "downstream notification sent",
                },
                {
                    "state": "closed",
                    "actor": "f-ed-closeout",
                    "evidence": "merged repair and downstream notification",
                },
            ],
            errors,
        )
        self.assertEqual(errors, [])

    def test_executor_green_repair_cannot_close_without_independent_verification(
        self,
    ) -> None:
        errors: list[str] = []
        CHECKER.validate_history(
            "LOC-020",
            "closed",
            [
                {
                    "state": "reported",
                    "actor": "editor-cutover",
                    "evidence": "source-artifact",
                },
                {
                    "state": "reproduced",
                    "actor": "f-ed-executor",
                    "evidence": "exact reproducer",
                },
                {
                    "state": "qualified",
                    "actor": "f-ed-executor",
                    "evidence": "pinned C++ differential",
                },
                {
                    "state": "mapped",
                    "actor": "f-ed-executor",
                    "evidence": "owner-family map",
                },
                {
                    "state": "executor-green",
                    "actor": "f-ed-executor",
                    "evidence": "required gates",
                },
                {
                    "state": "closed",
                    "actor": "f-ed-closeout",
                    "evidence": "invalid self-close",
                },
            ],
            errors,
        )
        self.assertIn(
            "LOC-020 has illegal state transition executor-green -> closed",
            errors,
        )

    def test_executor_green_regression_reopens_fail_closed(self) -> None:
        errors: list[str] = []
        history = [
            {
                "state": "reported",
                "actor": "editor-cutover",
                "evidence": "source-artifact",
            },
            {
                "state": "reproduced",
                "actor": "f-ed-executor",
                "evidence": "exact reproducer",
            },
            {
                "state": "qualified",
                "actor": "f-ed-executor",
                "evidence": "pinned C++ differential",
            },
            {
                "state": "mapped",
                "actor": "f-ed-executor",
                "evidence": "owner-family map",
            },
            {
                "state": "executor-green",
                "actor": "f-ed-executor",
                "evidence": "historical required gates",
            },
            {
                "state": "regression-reopened",
                "actor": "independent-orchestrator",
                "evidence": "independent current-path regression",
            },
        ]
        CHECKER.validate_history(
            "LOC-020", "regression-reopened", history, errors
        )
        CHECKER.validate_history_verifications(
            "LOC-020",
            history,
            {"status": "pending"},
            {"status": "pending"},
            errors,
            {
                "status": "pass",
                "command": "historical executor command",
                "evidence": "historical executor evidence",
            },
        )
        self.assertEqual(errors, [])

    def test_reopened_cycle_rejects_historical_pass_as_current(self) -> None:
        errors: list[str] = []
        history = [
            {"state": "executor-green"},
            {"state": "regression-reopened"},
        ]
        CHECKER.validate_history_verifications(
            "LOC-020",
            history,
            {
                "status": "pass",
                "command": "old command",
                "evidence": "old evidence",
            },
            {"status": "pending"},
            errors,
            {
                "status": "pass",
                "command": "old command",
                "evidence": "old evidence",
            },
        )
        self.assertIn(
            "LOC-020 regression-reopened current repair cycle must keep "
            "executor_verification pending or fail until a fresh "
            "executor-green event",
            errors,
        )

    def test_reopened_cycle_cannot_skip_fresh_mapped_executor_gate(self) -> None:
        errors: list[str] = []
        history = [
            {
                "state": "reported",
                "actor": "editor-cutover",
                "evidence": "source-artifact",
            },
            {
                "state": "reproduced",
                "actor": "f-ed-executor",
                "evidence": "exact reproducer",
            },
            {
                "state": "qualified",
                "actor": "f-ed-executor",
                "evidence": "old qualification",
            },
            {
                "state": "mapped",
                "actor": "f-ed-executor",
                "evidence": "old map",
            },
            {
                "state": "executor-green",
                "actor": "f-ed-executor",
                "evidence": "old gates",
            },
            {
                "state": "regression-reopened",
                "actor": "independent-orchestrator",
                "evidence": "current regression",
            },
            {
                "state": "qualified",
                "actor": "f-ed-executor",
                "evidence": "fresh qualification",
            },
            {
                "state": "orchestrator-verified",
                "actor": "independent-orchestrator",
                "evidence": "invalid skipped gates",
            },
        ]
        CHECKER.validate_history(
            "LOC-020",
            "orchestrator-verified",
            history,
            errors,
        )
        CHECKER.validate_history_verifications(
            "LOC-020",
            history,
            {"status": "pending"},
            {
                "status": "pass",
                "actor": "independent-orchestrator",
                "command": "invalid command",
                "evidence": "invalid evidence",
            },
            errors,
            {
                "status": "pass",
                "command": "old command",
                "evidence": "old evidence",
            },
        )
        self.assertIn(
            "LOC-020 has illegal state transition qualified -> "
            "orchestrator-verified",
            errors,
        )
        self.assertIn(
            "LOC-020 current repair cycle cannot reach orchestrator-verified "
            "without a fresh executor-green event",
            errors,
        )

    def test_executor_green_cannot_move_backward_without_reopening(self) -> None:
        errors: list[str] = []
        CHECKER.validate_history(
            "LOC-020",
            "qualified",
            [
                {
                    "state": "reported",
                    "actor": "editor-cutover",
                    "evidence": "source-artifact",
                },
                {
                    "state": "reproduced",
                    "actor": "f-ed-executor",
                    "evidence": "exact reproducer",
                },
                {
                    "state": "qualified",
                    "actor": "f-ed-executor",
                    "evidence": "qualification",
                },
                {
                    "state": "mapped",
                    "actor": "f-ed-executor",
                    "evidence": "map",
                },
                {
                    "state": "executor-green",
                    "actor": "f-ed-executor",
                    "evidence": "gates",
                },
                {
                    "state": "qualified",
                    "actor": "f-ed-executor",
                    "evidence": "invalid backward transition",
                },
            ],
            errors,
        )
        self.assertIn(
            "LOC-020 has illegal state transition executor-green -> qualified",
            errors,
        )

    def test_closed_repair_can_retain_pending_editor_consumption_revisions(
        self,
    ) -> None:
        pending = {
            "status": "pending",
            "reason": "Editor may consume the merged repair later.",
        }
        for field in ("consumed_runtime_sha", "consumed_superproject_sha"):
            with self.subTest(field=field):
                errors: list[str] = []
                CHECKER.validate_revision(
                    "LOC-020",
                    field,
                    pending,
                    "closed",
                    False,
                    errors,
                    handoff_ready_path=True,
                )
                self.assertEqual(errors, [])

    def test_closed_repair_still_requires_a_merged_repair_sha(self) -> None:
        errors: list[str] = []
        CHECKER.validate_revision(
            "LOC-020",
            "merged_repair_sha",
            {
                "status": "pending",
                "reason": "Repair has not merged.",
            },
            "closed",
            False,
            errors,
        )
        self.assertIn(
            "LOC-020 is closed but closure field "
            "revisions.merged_repair_sha is pending",
            errors,
        )

    def test_closed_editor_consumed_path_still_requires_consumption_shas(
        self,
    ) -> None:
        errors: list[str] = []
        CHECKER.validate_revision(
            "LOC-020",
            "consumed_runtime_sha",
            {
                "status": "pending",
                "reason": "Missing despite editor-consumed history.",
            },
            "closed",
            False,
            errors,
            editor_consumed_path=True,
            handoff_ready_path=True,
        )
        self.assertIn(
            "LOC-020 is closed but closure field "
            "revisions.consumed_runtime_sha is pending",
            errors,
        )

    def test_no_repair_editor_consumed_path_still_requires_consumption_shas(
        self,
    ) -> None:
        errors: list[str] = []
        CHECKER.validate_revision(
            "LOC-020",
            "consumed_runtime_sha",
            {
                "status": "pending",
                "reason": "Missing despite editor-consumed history.",
            },
            "closed",
            True,
            errors,
            editor_consumed_path=True,
            handoff_ready_path=True,
        )
        self.assertIn(
            "LOC-020 is closed but closure field "
            "revisions.consumed_runtime_sha is pending",
            errors,
        )

    def test_closed_repair_without_handoff_still_requires_consumption_shas(
        self,
    ) -> None:
        errors: list[str] = []
        CHECKER.validate_revision(
            "LOC-020",
            "consumed_runtime_sha",
            {
                "status": "pending",
                "reason": "No downstream handoff was recorded.",
            },
            "closed",
            False,
            errors,
        )
        self.assertIn(
            "LOC-020 is closed but closure field "
            "revisions.consumed_runtime_sha is pending",
            errors,
        )

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
        self.assertIn("--newest-source-root", result.stderr)
        self.assertIn("--editor-repo-dir", result.stderr)
        self.assertIn("--rive-runtime-dir", result.stderr)
        self.assertIn("--cpp-probe", result.stderr)

    def test_editor_git_provenance_requires_ordered_commits_and_exact_heads(
        self,
    ) -> None:
        editor = pathlib.Path(self.temp.name) / "editor-repo"
        editor.mkdir()
        subprocess.run(["git", "init", "-q"], cwd=editor, check=True)
        subprocess.run(
            ["git", "config", "user.email", "test@example.com"],
            cwd=editor,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "Test"],
            cwd=editor,
            check=True,
        )
        (editor / "record").write_text("consumed\n")
        subprocess.run(["git", "add", "."], cwd=editor, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "consumed"],
            cwd=editor,
            check=True,
        )
        consumed = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=editor,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        (editor / "record").write_text("newest\n")
        subprocess.run(["git", "commit", "-qam", "newest"], cwd=editor, check=True)
        newest = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=editor,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        subprocess.run(
            [
                "git",
                "update-ref",
                "refs/remotes/origin/levi/editor-next-cutover-assembly",
                newest,
            ],
            cwd=editor,
            check=True,
        )
        consumed_checkout = pathlib.Path(self.temp.name) / "consumed-checkout"
        newest_checkout = pathlib.Path(self.temp.name) / "newest-checkout"
        subprocess.run(
            [
                "git",
                "worktree",
                "add",
                "--quiet",
                "--detach",
                str(consumed_checkout),
                consumed,
            ],
            cwd=editor,
            check=True,
        )
        subprocess.run(
            [
                "git",
                "worktree",
                "add",
                "--quiet",
                "--detach",
                str(newest_checkout),
                newest,
            ],
            cwd=editor,
            check=True,
        )
        inbox = {
            "last_consumed_editor_ref": consumed,
            "newest_available_editor_ref": newest,
            "canonical_branch": "origin/levi/editor-next-cutover-assembly",
        }
        errors: list[str] = []
        tip = CHECKER.validate_editor_git_provenance(
            inbox,
            consumed_checkout,
            newest_checkout,
            editor,
            errors,
        )
        self.assertEqual(errors, [])
        self.assertEqual(tip, newest)

        reversed_errors: list[str] = []
        CHECKER.validate_editor_git_provenance(
            {
                **inbox,
                "last_consumed_editor_ref": newest,
                "newest_available_editor_ref": consumed,
            },
            newest_checkout,
            consumed_checkout,
            editor,
            reversed_errors,
        )
        self.assertIn(
            "consumed Editor checkpoint is not an ancestor of the newest "
            "available checkpoint",
            reversed_errors,
        )

    def test_editor_source_bytes_must_match_the_pinned_commit_blobs(
        self,
    ) -> None:
        editor = pathlib.Path(self.temp.name) / "editor-blob-repo"
        self.init_git_repo(editor)
        plans = editor / "plans"
        plans.mkdir()
        (plans / PROPOSAL_NAME).write_text("proposal\n")
        (plans / DEFECTS_NAME).write_text("defects\n")
        (plans / LEDGER_NAME).write_text("{}\n")
        subprocess.run(["git", "add", "plans"], cwd=editor, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "pinned Editor source"],
            cwd=editor,
            check=True,
        )
        commit = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=editor,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        subprocess.run(
            [
                "git",
                "update-ref",
                "refs/remotes/origin/levi/editor-next-cutover-assembly",
                commit,
            ],
            cwd=editor,
            check=True,
        )
        inbox = {
            "last_consumed_editor_ref": commit,
            "newest_available_editor_ref": commit,
            "canonical_branch": "origin/levi/editor-next-cutover-assembly",
            "runtime_defects_path": f"plans/{DEFECTS_NAME}",
            "parity_ledger_path": f"plans/{LEDGER_NAME}",
        }
        errors: list[str] = []
        CHECKER.validate_editor_git_provenance(
            inbox,
            plans,
            plans,
            editor,
            errors,
        )
        self.assertEqual(errors, [])

        (plans / DEFECTS_NAME).write_text("dirty defects\n")
        dirty_errors: list[str] = []
        CHECKER.validate_editor_git_provenance(
            inbox,
            plans,
            plans,
            editor,
            dirty_errors,
        )
        self.assertTrue(
            any(
                "consumed Editor source runtime-defects bytes do not match "
                "the pinned Editor commit blob" in error
                for error in dirty_errors
            ),
            dirty_errors,
        )
        self.assertTrue(
            any(
                "newest Editor source runtime-defects bytes do not match "
                "the pinned Editor commit blob" in error
                for error in dirty_errors
            ),
            dirty_errors,
        )

    def test_literal_revision_provenance_resolves_and_matches_gitlink(
        self,
    ) -> None:
        (
            runtime,
            editor,
            runtime_base,
            runtime_repair,
            runtime_current,
            editor_repair,
            superproject,
        ) = self.revision_repositories()
        atlas = {
            "editor_consumed_runtime_ref": runtime_current,
            "investigation_base_ref": runtime_base,
            "source_snapshot_ref": superproject,
        }
        runtime_row = {
            "id": "LOC-020",
            "owner_class": "runtime",
            "revisions": {
                "original_localization_rust_sha": runtime_base,
                "editor_last_consumed_runtime_sha": runtime_base,
                "investigation_head_sha": runtime_current,
                "merged_repair_sha": runtime_repair,
                "consumed_runtime_sha": runtime_current,
                "consumed_superproject_sha": superproject,
            },
        }
        editor_row = {
            "id": "LOC-021",
            "owner_class": "editor",
            "revisions": {
                "merged_repair_sha": editor_repair,
                "consumed_runtime_sha": runtime_current,
                "consumed_superproject_sha": superproject,
            },
        }
        errors: list[str] = []
        CHECKER.validate_revision_provenance(
            atlas,
            [runtime_row, editor_row],
            runtime,
            editor,
            errors,
        )
        self.assertEqual(errors, [])

    def test_literal_revision_provenance_rejects_nonexistent_commits(
        self,
    ) -> None:
        (
            runtime,
            editor,
            runtime_base,
            runtime_repair,
            runtime_current,
            _,
            superproject,
        ) = self.revision_repositories()
        missing_runtime = "f" * 40
        missing_editor = "e" * 40
        row = {
            "id": "LOC-020",
            "owner_class": "runtime",
            "revisions": {
                "original_localization_rust_sha": runtime_base,
                "editor_last_consumed_runtime_sha": runtime_base,
                "investigation_head_sha": missing_runtime,
                "merged_repair_sha": runtime_repair,
                "consumed_runtime_sha": runtime_current,
                "consumed_superproject_sha": missing_editor,
            },
        }
        errors: list[str] = []
        CHECKER.validate_revision_provenance(
            {
                "editor_consumed_runtime_ref": runtime_current,
                "investigation_base_ref": runtime_base,
                "source_snapshot_ref": superproject,
            },
            [row],
            runtime,
            editor,
            errors,
        )
        self.assertTrue(
            any(
                "revisions.investigation_head_sha revision "
                f"'{missing_runtime}' does not resolve as a commit"
                in error
                for error in errors
            ),
            errors,
        )
        self.assertTrue(
            any(
                "revisions.consumed_superproject_sha revision "
                f"'{missing_editor}' does not resolve as a commit"
                in error
                for error in errors
            ),
            errors,
        )
        self.assertNotEqual(superproject, missing_editor)

    def test_revision_provenance_rejects_ancestry_and_gitlink_mismatch(
        self,
    ) -> None:
        (
            runtime,
            editor,
            runtime_base,
            _,
            runtime_current,
            _,
            _,
        ) = self.revision_repositories()
        tree = subprocess.run(
            ["git", "rev-parse", f"{runtime_base}^{{tree}}"],
            cwd=runtime,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        unrelated_repair = subprocess.run(
            ["git", "commit-tree", tree],
            cwd=runtime,
            input="unrelated repair\n",
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        subprocess.run(
            [
                "git",
                "update-index",
                "--add",
                "--cacheinfo",
                f"160000,{runtime_base},third_party/nuxie-runtime",
            ],
            cwd=editor,
            check=True,
        )
        subprocess.run(
            ["git", "commit", "-qm", "wrong runtime gitlink"],
            cwd=editor,
            check=True,
        )
        mismatched_superproject = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=editor,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        row = {
            "id": "LOC-020",
            "owner_class": "runtime",
            "revisions": {
                "editor_last_consumed_runtime_sha": unrelated_repair,
                "merged_repair_sha": unrelated_repair,
                "consumed_runtime_sha": runtime_current,
                "consumed_superproject_sha": mismatched_superproject,
            },
        }
        errors: list[str] = []
        CHECKER.validate_revision_provenance(
            {
                "editor_consumed_runtime_ref": runtime_current,
                "investigation_base_ref": runtime_base,
                "source_snapshot_ref": mismatched_superproject,
            },
            [row],
            runtime,
            editor,
            errors,
        )
        self.assertIn(
            "LOC-020 merged runtime repair is not an ancestor of its "
            "consumed runtime",
            errors,
        )
        self.assertIn(
            "LOC-020 revisions.editor_last_consumed_runtime_sha is not an "
            "ancestor of atlas.editor_consumed_runtime_ref",
            errors,
        )
        self.assertTrue(
            any(
                "atlas.editor_consumed_runtime_ref does not match "
                "source_snapshot_ref runtime gitlink" in error
                for error in errors
            ),
            errors,
        )
        self.assertTrue(
            any(
                "LOC-020 consumed superproject runtime gitlink is" in error
                for error in errors
            ),
            errors,
        )

    def test_landed_repair_identity_ratchet_is_exact(self) -> None:
        rows = [
            {
                "id": defect_id,
                "revisions": dict(expected),
            }
            for defect_id, expected in CHECKER.LANDED_REPAIR_PROVENANCE.items()
        ]
        errors: list[str] = []
        CHECKER.validate_landed_repair_provenance(rows, errors)
        self.assertEqual(errors, [])
        rows[0]["revisions"]["merged_repair_sha"] = {
            "status": "pending",
            "reason": "stale atlas",
        }
        CHECKER.validate_landed_repair_provenance(rows, errors)
        self.assertTrue(
            any(
                "RT-ED-003 revisions.merged_repair_sha" in error
                and "landed provenance ratchet requires" in error
                for error in errors
            ),
            errors,
        )

    def test_prior_origin_main_ids_cannot_be_deleted(self) -> None:
        runtime = pathlib.Path(self.temp.name) / "runtime-repo"
        (runtime / "docs").mkdir(parents=True)
        subprocess.run(["git", "init", "-q"], cwd=runtime, check=True)
        subprocess.run(
            ["git", "config", "user.email", "test@example.com"],
            cwd=runtime,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "Test"],
            cwd=runtime,
            check=True,
        )
        (runtime / "docs/editor-next-runtime-defect-atlas.toml").write_text(
            '[[defect]]\nid = "LOC-020"\n'
        )
        subprocess.run(["git", "add", "."], cwd=runtime, check=True)
        subprocess.run(["git", "commit", "-qm", "prior"], cwd=runtime, check=True)
        head = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=runtime,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        subprocess.run(
            ["git", "update-ref", "refs/remotes/origin/main", head],
            cwd=runtime,
            check=True,
        )
        errors: list[str] = []
        CHECKER.validate_prior_id_ratchet(runtime, [], errors)
        self.assertIn(
            "atlas deletes previously accepted origin/main defect ids: LOC-020",
            errors,
        )

    def test_prior_origin_main_history_is_an_immutable_prefix(self) -> None:
        runtime = pathlib.Path(self.temp.name) / "runtime-history-repo"
        self.init_git_repo(runtime)
        prior_history = [
            {
                "state": "reported",
                "actor": "editor-cutover",
                "evidence": "immutable intake",
            },
            {
                "state": "reproduced",
                "actor": "executor",
                "evidence": "immutable reproducer",
            },
        ]
        (runtime / "docs").mkdir()
        (runtime / "docs/editor-next-runtime-defect-atlas.toml").write_text(
            textwrap.dedent(
                """
                [[defect]]
                id = "LOC-020"
                state = "reproduced"

                [[defect.history]]
                state = "reported"
                actor = "editor-cutover"
                evidence = "immutable intake"

                [[defect.history]]
                state = "reproduced"
                actor = "executor"
                evidence = "immutable reproducer"
                """
            ).lstrip()
        )
        subprocess.run(["git", "add", "."], cwd=runtime, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "prior history"],
            cwd=runtime,
            check=True,
        )
        prior = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=runtime,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        subprocess.run(
            ["git", "update-ref", "refs/remotes/origin/main", prior],
            cwd=runtime,
            check=True,
        )

        advanced = {
            "id": "LOC-020",
            "state": "qualified",
            "history": [
                *prior_history,
                {
                    "state": "qualified",
                    "actor": "executor",
                    "evidence": "pinned differential",
                },
            ],
        }
        errors: list[str] = []
        CHECKER.validate_prior_id_ratchet(runtime, [advanced], errors)
        self.assertEqual(errors, [])

        replaced = {
            "id": "LOC-020",
            "state": "reproduced",
            "history": [
                prior_history[0],
                {
                    **prior_history[1],
                    "evidence": "rewritten evidence",
                },
            ],
        }
        errors = []
        CHECKER.validate_prior_id_ratchet(runtime, [replaced], errors)
        self.assertIn(
            "LOC-020 origin/main history is not an exact prefix of "
            "current history",
            errors,
        )

        trimmed = {
            "id": "LOC-020",
            "state": "reported",
            "history": prior_history[:1],
        }
        errors = []
        CHECKER.validate_prior_id_ratchet(runtime, [trimmed], errors)
        self.assertIn(
            "LOC-020 current state 'reported' regresses origin/main state "
            "'reproduced' by trimming history",
            errors,
        )

    def test_prior_terminal_history_cannot_be_replaced(self) -> None:
        runtime = pathlib.Path(self.temp.name) / "runtime-terminal-history"
        self.init_git_repo(runtime)
        (runtime / "docs").mkdir()
        prior_history = [
            {
                "state": "reported",
                "actor": "editor-cutover",
                "evidence": "intake",
            },
            {
                "state": "reproduced",
                "actor": "executor",
                "evidence": "reproducer",
            },
            {
                "state": "stale-oracle",
                "actor": "executor",
                "evidence": "oracle proof",
            },
            {
                "state": "closed",
                "actor": "executor",
                "evidence": "terminal disposition",
            },
        ]
        rows = "\n\n".join(
            textwrap.dedent(
                f"""
                [[defect.history]]
                state = "{entry["state"]}"
                actor = "{entry["actor"]}"
                evidence = "{entry["evidence"]}"
                """
            ).strip()
            for entry in prior_history
        )
        (runtime / "docs/editor-next-runtime-defect-atlas.toml").write_text(
            '[[defect]]\nid = "LOC-020"\nstate = "closed"\n\n'
            + rows
            + "\n"
        )
        subprocess.run(["git", "add", "."], cwd=runtime, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "terminal history"],
            cwd=runtime,
            check=True,
        )
        prior = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=runtime,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        subprocess.run(
            ["git", "update-ref", "refs/remotes/origin/main", prior],
            cwd=runtime,
            check=True,
        )
        replaced = [dict(entry) for entry in prior_history]
        replaced[-1]["evidence"] = "rewritten terminal disposition"
        errors: list[str] = []
        CHECKER.validate_prior_id_ratchet(
            runtime,
            [
                {
                    "id": "LOC-020",
                    "state": "closed",
                    "history": replaced,
                }
            ],
            errors,
        )
        self.assertIn(
            "LOC-020 origin/main history is not an exact prefix of "
            "current history",
            errors,
        )

    def test_prior_literal_immutable_revisions_cannot_change_or_revert(
        self,
    ) -> None:
        runtime = pathlib.Path(self.temp.name) / "runtime-revision-ratchet"
        self.init_git_repo(runtime)
        (runtime / "docs").mkdir()
        original = "a" * 40
        consumed = "b" * 40
        superproject = "c" * 40
        newly_landed = "d" * 40
        (runtime / "docs/editor-next-runtime-defect-atlas.toml").write_text(
            textwrap.dedent(
                f"""
                schema = "nuxie.editor-next.runtime-defect-atlas/v2"
                version = 2

                [[defect]]
                id = "LOC-020"
                state = "reported"

                [defect.revisions]
                original_localization_rust_sha = "{original}"
                merged_repair_sha = {{ status = "pending", reason = "Not landed." }}
                consumed_runtime_sha = "{consumed}"
                consumed_superproject_sha = "{superproject}"

                [[defect.history]]
                state = "reported"
                actor = "editor-cutover"
                evidence = "intake"
                """
            ).lstrip()
        )
        subprocess.run(["git", "add", "."], cwd=runtime, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "prior revisions"],
            cwd=runtime,
            check=True,
        )
        prior = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=runtime,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        subprocess.run(
            ["git", "update-ref", "refs/remotes/origin/main", prior],
            cwd=runtime,
            check=True,
        )
        current = {
            "id": "LOC-020",
            "state": "reported",
            "revisions": {
                "original_localization_rust_sha": original,
                "merged_repair_sha": newly_landed,
                "consumed_runtime_sha": consumed,
                "consumed_superproject_sha": superproject,
            },
            "history": [
                {
                    "state": "reported",
                    "actor": "editor-cutover",
                    "evidence": "intake",
                }
            ],
        }
        errors: list[str] = []
        CHECKER.validate_prior_id_ratchet(runtime, [current], errors)
        self.assertEqual(errors, [])

        changed = {
            **current,
            "revisions": {
                **current["revisions"],
                "original_localization_rust_sha": "e" * 40,
                "consumed_runtime_sha": {
                    "status": "pending",
                    "reason": "regressed",
                },
            },
        }
        errors = []
        CHECKER.validate_prior_id_ratchet(runtime, [changed], errors)
        self.assertTrue(
            any(
                "LOC-020 immutable revision "
                "original_localization_rust_sha changed" in error
                for error in errors
            ),
            errors,
        )
        self.assertTrue(
            any(
                "LOC-020 immutable revision consumed_runtime_sha changed"
                in error
                for error in errors
            ),
            errors,
        )

    def test_require_closed_binds_zero_intake_to_canonical_branch_tip(
        self,
    ) -> None:
        errors: list[str] = []
        CHECKER.validate_closed_inbox(
            {
                "unconsumed_records": 1,
                "newest_available_editor_ref": "a" * 40,
            },
            "b" * 40,
            errors,
        )
        self.assertIn(
            "--require-closed requires unconsumed_records = 0",
            errors,
        )
        self.assertIn(
            "--require-closed requires newest_available_editor_ref to equal "
            "the canonical Editor branch tip",
            errors,
        )

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
        (self.source / DEFECTS_NAME).write_text("changed\n")
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
