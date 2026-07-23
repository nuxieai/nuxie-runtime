#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import pathlib
import subprocess
import tempfile
import textwrap
import unittest


TOOL = pathlib.Path(__file__).with_name("check.py")
SPEC = importlib.util.spec_from_file_location("runtime_drawing_port_check", TOOL)
assert SPEC is not None and SPEC.loader is not None
CHECKER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECKER)


class RuntimeDrawingPortCheckTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        self.repo = self.root / "repo"
        self.upstream = self.root / "rive-runtime"
        (self.repo / "docs").mkdir(parents=True)
        (self.repo / "crates/runtime/src").mkdir(parents=True)
        (self.upstream / "src").mkdir(parents=True)
        (self.repo / "docs/PORTING.md").write_text(
            "- **RF-27 Test ownership rule.** Fixture.\n"
        )
        (self.repo / "crates/runtime/src/draw.rs").write_text(
            "struct RuntimeShapeOwner;\n"
        )
        (self.upstream / "src/shape.cpp").write_text(
            "\n".join(
                (
                    "struct Shape { int member; };",
                    "void construct() {}",
                    "void update() {}",
                    "void draw() {}",
                    "void clone_drop() {}",
                )
            )
            + "\n"
        )
        subprocess.run(
            ["git", "init", "-q"],
            cwd=self.upstream,
            check=True,
        )
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
            ["git", "commit", "-qm", "fixture"],
            cwd=self.upstream,
            check=True,
        )
        self.upstream_ref = (
            subprocess.run(
                ["git", "rev-parse", "HEAD"],
                cwd=self.upstream,
                text=True,
                capture_output=True,
                check=True,
            )
            .stdout.strip()
        )
        self.ledger = self.repo / "docs/ownership.toml"
        self.gaps = self.repo / "docs/gaps.toml"
        self.write_gaps()
        self.write_ledger()

    def write_gaps(self) -> None:
        self.gaps.write_text(
            textwrap.dedent(
                f"""
                version = 1
                upstream_ref = "{self.upstream_ref}"

                [[gap]]
                id = "G1"
                rule = "RF-27"
                citations = ["cpp:src/shape.cpp:1", "rust:crates/runtime/src/draw.rs:1"]
                decision = "Retain one owner."
                closure_test = "The structural checker rejects a cache."
                """
            ).lstrip()
        )

    def write_ledger(
        self,
        *,
        status: str = "exact",
        max_occurrences: int = 0,
        owner_lifecycle: str | None = None,
        dependency: str = "",
    ) -> None:
        lifecycle = owner_lifecycle or textwrap.dedent(
            """
            construct = ["cpp:src/shape.cpp:2"]
            update = ["cpp:src/shape.cpp:3"]
            draw = ["cpp:src/shape.cpp:4"]
            clone_drop = ["cpp:src/shape.cpp:5"]
            """
        ).strip()
        open_count = 1 if status == "pending" else 0
        exact_count = 1 if status == "exact" else 0
        self.ledger.write_text(
            textwrap.dedent(
                f"""
                version = 1
                upstream_ref = "{self.upstream_ref}"
                porting_rules_file = "docs/PORTING.md"

                [expected_status_counts]
                exact = {exact_count}
                adapted = 0
                pending = {open_count}
                compensation = 0

                [[batch]]
                id = "shape"
                sequence = 1
                depends_on = [{dependency}]

                [[ratchet]]
                id = "no_scene_cache"
                globs = ["crates/runtime/src/**/*.rs"]
                pattern = "\\\\bRuntimeRenderPathCache\\\\b"
                max_occurrences = {max_occurrences}

                [[owner]]
                id = "shape.owner"
                batch = "shape"
                status = "{status}"
                rule = "RF-27"
                rust_file = "crates/runtime/src/draw.rs"
                rust_anchor = "RuntimeShapeOwner"
                depends_on = []
                legacy_ratchets = []

                [owner.lifecycle]
                {lifecycle}
                """
            ).lstrip()
        )

    def run_check(self, require_closed: bool = False) -> subprocess.CompletedProcess[str]:
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
        ]
        if require_closed:
            command.append("--require-closed")
        return subprocess.run(
            command,
            text=True,
            capture_output=True,
            check=False,
        )

    def test_clean_fixture_passes(self) -> None:
        result = self.run_check()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("owners=1", result.stdout)
        self.assertIn("no_scene_cache=0/0", result.stdout)

    def test_negative_control_rejects_external_scene_cache(self) -> None:
        (self.repo / "crates/runtime/src/draw.rs").write_text(
            "struct RuntimeShapeOwner;\nstruct RuntimeRenderPathCache;\n"
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("ratchet no_scene_cache increased to 1 > 0", result.stderr)

    def test_missing_lifecycle_phase_fails_closed(self) -> None:
        self.write_ledger(
            owner_lifecycle=textwrap.dedent(
                """
                construct = ["cpp:src/shape.cpp:2"]
                update = ["cpp:src/shape.cpp:3"]
                draw = ["cpp:src/shape.cpp:4"]
                clone_drop = []
                """
            ).strip()
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("lifecycle clone_drop is empty", result.stderr)

    def test_require_closed_rejects_pending_owner(self) -> None:
        self.write_ledger(status="pending")
        result = self.run_check(require_closed=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("rows remain open: shape.owner", result.stderr)

    def test_upstream_provenance_change_fails(self) -> None:
        (self.upstream / "src/shape.cpp").write_text("// changed\n")
        subprocess.run(["git", "add", "."], cwd=self.upstream, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "drift"],
            cwd=self.upstream,
            check=True,
        )
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("ownership ledger pins", result.stderr)

    def test_dependency_cycle_fails(self) -> None:
        content = self.ledger.read_text()
        content = content.replace(
            "depends_on = []",
            'depends_on = ["shape"]',
            1,
        )
        self.ledger.write_text(content)
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("batch dependency cycle: shape", result.stderr)

    def test_owner_cannot_depend_on_a_later_batch(self) -> None:
        content = self.ledger.read_text()
        content = content.replace("exact = 1", "exact = 2", 1)
        content = content.replace(
            "[[ratchet]]",
            textwrap.dedent(
                """
                [[batch]]
                id = "image"
                sequence = 2
                depends_on = ["shape"]

                [[ratchet]]
                """
            ).lstrip(),
            1,
        )
        content = content.replace(
            'rust_anchor = "RuntimeShapeOwner"\n'
            "                depends_on = []",
            'rust_anchor = "RuntimeShapeOwner"\n'
            '                depends_on = ["image.owner"]',
            1,
        )
        content += textwrap.dedent(
            """

            [[owner]]
            id = "image.owner"
            batch = "image"
            status = "exact"
            rule = "RF-27"
            rust_file = "crates/runtime/src/draw.rs"
            rust_anchor = "RuntimeShapeOwner"
            depends_on = []
            legacy_ratchets = []

            [owner.lifecycle]
            construct = ["cpp:src/shape.cpp:2"]
            update = ["cpp:src/shape.cpp:3"]
            draw = ["cpp:src/shape.cpp:4"]
            clone_drop = ["cpp:src/shape.cpp:5"]
            """
        )
        self.ledger.write_text(content)
        result = self.run_check()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "owner shape.owner in batch shape depends on later owner "
            "image.owner in batch image",
            result.stderr,
        )


if __name__ == "__main__":
    unittest.main()
