#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import json
import pathlib
import subprocess
import tempfile
import unittest


MODULE_PATH = pathlib.Path(__file__).with_name("capture_trace.py")
SPEC = importlib.util.spec_from_file_location("capture_trace", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
CAPTURE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CAPTURE)


class CaptureTraceTest(unittest.TestCase):
    def test_fixture_arguments_pin_declared_file_hash(self) -> None:
        arguments = CAPTURE.fixture_arguments(
            {
                "path": "tests/assets/tail.riv",
                "samples": [0.0],
                "expected_file_sha256": "a" * 64,
            },
            upstream=pathlib.Path("/pinned-rive-runtime"),
            include_expected_file_sha256=True,
        )

        self.assertEqual(
            arguments,
            [
                "--file",
                "/pinned-rive-runtime/tests/assets/tail.riv",
                "--expected-file-sha256",
                "a" * 64,
                "--samples",
                "0",
                "--benchmark-repeat",
                "1",
            ],
        )

    def test_rust_runner_provenance_accepts_exact_candidate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            runner = pathlib.Path(directory) / "rust-golden-runner"
            runner.write_bytes(b"runner")
            candidate = {
                "schema": "nuxie-runtime-frame-loop-rust-source/v1",
                "sha256": "a" * 64,
                "file_count": 12,
            }
            provenance = CAPTURE.rust_runner_provenance(candidate)
            CAPTURE.rust_runner_provenance_path(runner).write_text(
                json.dumps(provenance)
            )

            self.assertEqual(
                CAPTURE.require_rust_runner_provenance(runner, candidate),
                provenance,
            )

    def test_rust_runner_provenance_rejects_stale_candidate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            runner = pathlib.Path(directory) / "rust-golden-runner"
            runner.write_bytes(b"runner")
            built_candidate = {
                "schema": "nuxie-runtime-frame-loop-rust-source/v1",
                "sha256": "a" * 64,
                "file_count": 12,
            }
            current_candidate = dict(built_candidate, sha256="b" * 64)
            CAPTURE.rust_runner_provenance_path(runner).write_text(
                json.dumps(CAPTURE.rust_runner_provenance(built_candidate))
            )

            with self.assertRaisesRegex(
                CAPTURE.SourceFingerprintError,
                "Rust trace runner provenance is stale",
            ):
                CAPTURE.require_rust_runner_provenance(
                    runner, current_candidate
                )

    def test_candidate_source_fingerprint_is_deterministic_and_self_excluding(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = pathlib.Path(directory)
            subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
            source = repo / "crates/runtime/src/lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text("pub fn frame() {}\n")
            output = repo / "docs/runtime-frame-loop-trace.json"
            output.parent.mkdir()
            output.write_text("{}\n")
            status = repo / "docs/runtime-frame-loop-status.md"
            status.write_text("before\n")
            local_fixture = repo / "fixtures/animation"
            local_fixture.parent.mkdir()
            local_fixture.symlink_to("/developer-only/fixtures")
            orchestration_log = repo / ".flc5/out/W29.log"
            orchestration_log.parent.mkdir(parents=True)
            orchestration_log.write_text("before\n")
            e8_orchestration_log = repo / ".fle8/assembly.log"
            e8_orchestration_log.parent.mkdir(parents=True)
            e8_orchestration_log.write_text("before\n")
            root_wave_log = repo / "W119.log"
            root_wave_log.write_text("before\n")
            root_wave_report = repo / "W117-report.md"
            root_wave_report.write_text("before\n")
            root_inventory_log = repo / "E8-inv-text.log"
            root_inventory_log.write_text("before\n")
            root_inventory_report = repo / "E8-inv-text.md"
            root_inventory_report.write_text("before\n")

            first = CAPTURE.candidate_source_fingerprint(
                repo, evidence_path=output
            )
            output.write_text('{"generated": true}\n')
            status.write_text("after\n")
            orchestration_log.write_text("after\n")
            e8_orchestration_log.write_text("after\n")
            root_wave_log.write_text("after\n")
            root_wave_report.write_text("after\n")
            root_inventory_log.write_text("after\n")
            root_inventory_report.write_text("after\n")
            generated = repo / "tools/trace/__pycache__/capture.pyc"
            generated.parent.mkdir(parents=True)
            generated.write_bytes(b"generated")
            second = CAPTURE.candidate_source_fingerprint(
                repo, evidence_path=output
            )

            self.assertEqual(first, second)
            self.assertEqual(first["file_count"], 1)

    def test_candidate_source_fingerprint_excludes_generated_artifact_receipts(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = pathlib.Path(directory)
            subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
            output = repo / "docs/runtime-frame-loop-trace.json"
            output.parent.mkdir()
            output.write_text("{}\n")
            ledger = repo / "docs/runtime-frame-loop-ownership.toml"
            ledger.write_text(
                "schema = 8\n\n"
                "[expected_trace_artifacts]\n"
                f'cpp_binary_sha256 = "{"a" * 64}"\n\n'
                "[active_owner_family]\n"
                'id = "FL-E7"\n'
            )

            first = CAPTURE.candidate_source_fingerprint(
                repo, evidence_path=output
            )
            ledger.write_text(
                "schema = 8\n\n"
                "[expected_trace_artifacts]\n"
                f'cpp_binary_sha256 = "{"b" * 64}"\n\n'
                "[active_owner_family]\n"
                'id = "FL-E7"\n'
            )
            receipt_only = CAPTURE.candidate_source_fingerprint(
                repo, evidence_path=output
            )
            ledger.write_text(
                "schema = 9\n\n"
                "[expected_trace_artifacts]\n"
                f'cpp_binary_sha256 = "{"b" * 64}"\n\n'
                "[active_owner_family]\n"
                'id = "FL-E7"\n'
            )
            semantic_change = CAPTURE.candidate_source_fingerprint(
                repo, evidence_path=output
            )

            self.assertEqual(first, receipt_only)
            self.assertNotEqual(first["sha256"], semantic_change["sha256"])

    def test_candidate_source_fingerprint_excludes_upstream_oracle_checkout(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repo = pathlib.Path(directory)
            subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
            source = repo / "crates/runtime/src/lib.rs"
            source.parent.mkdir(parents=True)
            source.write_text("pub fn frame() {}\n")
            output = repo / "docs/runtime-frame-loop-trace.json"
            output.parent.mkdir()
            output.write_text("{}\n")

            without_oracle = CAPTURE.candidate_source_fingerprint(
                repo, evidence_path=output
            )
            upstream = repo / "rive-runtime"
            subprocess.run(["git", "init", "-q", str(upstream)], check=True)
            upstream_source = upstream / "src/artboard.cpp"
            upstream_source.parent.mkdir()
            upstream_source.write_text("void advance() {}\n")
            with_oracle = CAPTURE.candidate_source_fingerprint(
                repo, evidence_path=output
            )

            self.assertEqual(without_oracle, with_oracle)
            self.assertEqual(with_oracle["file_count"], 1)

    def test_interactive_input_is_only_enabled_for_mechanism_frame(self) -> None:
        row = {
            "id": "scroll",
            "path": "scroll.riv",
            "samples": [0.0, 0.1],
            "input_script": pathlib.Path("/tmp/scroll.txt"),
        }

        frame, frame_input = CAPTURE.effective_fixture_row(
            row,
            frame_only=True,
            occurrence_only=False,
            steady_only=False,
        )
        self.assertTrue(frame_input)
        self.assertEqual(frame["samples"], [0.0, 0.1])
        self.assertIn("input_script", frame)

        construction, construction_input = CAPTURE.effective_fixture_row(
            row,
            frame_only=False,
            occurrence_only=True,
            steady_only=False,
        )
        self.assertFalse(construction_input)
        self.assertEqual(construction["samples"], [0.0])
        self.assertNotIn("input_script", construction)

        steady, steady_input = CAPTURE.effective_fixture_row(
            row,
            frame_only=True,
            occurrence_only=False,
            steady_only=True,
        )
        self.assertFalse(steady_input)
        self.assertEqual(steady["samples"], [0.0])
        self.assertNotIn("input_script", steady)

    def test_materialized_cpp_coverage_paths_are_normalized_to_upstream(
        self,
    ) -> None:
        upstream = pathlib.Path("/fixtures/rive-runtime")
        coverage = {
            "data": [
                {
                    "files": [
                        {
                            "filename": (
                                "/repo/target/golden-runner-librive/"
                                "patched-runtime-src.AbCd12/src/artboard.cpp"
                            ),
                            "expansions": [
                                {
                                    "filenames": [
                                        "/system/include/vector",
                                        (
                                            "/repo/target/golden-runner-librive/"
                                            "patched-runtime-src.AbCd12/"
                                            "include/rive/artboard.hpp"
                                        ),
                                    ]
                                }
                            ],
                        }
                    ],
                    "functions": [
                        {
                            "filenames": [
                                (
                                    "/repo/target/golden-runner-librive/"
                                    "patched-runtime-src.AbCd12/"
                                    "src/animation/state_machine_instance.cpp"
                                ),
                                "/system/include/vector",
                            ]
                        }
                    ],
                }
            ]
        }

        CAPTURE.normalize_materialized_cpp_coverage_paths(
            coverage, upstream=upstream
        )

        self.assertEqual(
            coverage["data"][0]["files"][0]["filename"],
            "/fixtures/rive-runtime/src/artboard.cpp",
        )
        self.assertEqual(
            coverage["data"][0]["functions"][0]["filenames"],
            [
                (
                    "/fixtures/rive-runtime/"
                    "src/animation/state_machine_instance.cpp"
                ),
                "/system/include/vector",
            ],
        )
        self.assertEqual(
            coverage["data"][0]["files"][0]["expansions"][0]["filenames"],
            [
                "/system/include/vector",
                "/fixtures/rive-runtime/include/rive/artboard.hpp",
            ],
        )


if __name__ == "__main__":
    unittest.main()
