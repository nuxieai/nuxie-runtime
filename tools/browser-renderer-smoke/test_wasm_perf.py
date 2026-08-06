import hashlib
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace

import wasm_perf


class SourceProvenanceTests(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        self.repo = self.root / "repo"
        self.runtime = self.root / "runtime"
        self._initialize_repo(self.repo, "measured.rs")
        self._initialize_repo(self.runtime, "fixture.riv")

    def tearDown(self):
        self.temp_dir.cleanup()

    @staticmethod
    def _initialize_repo(root: Path, filename: str) -> None:
        root.mkdir()
        subprocess.run(["git", "init", "-q"], cwd=root, check=True)
        subprocess.run(
            ["git", "config", "user.email", "wasm-perf@example.com"],
            cwd=root,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "Wasm Perf"], cwd=root, check=True
        )
        (root / filename).write_text("committed\n", encoding="utf-8")
        subprocess.run(["git", "add", filename], cwd=root, check=True)
        subprocess.run(["git", "commit", "-qm", "fixture"], cwd=root, check=True)

    def test_rejects_dirty_measured_source_before_capture(self):
        (self.repo / "measured.rs").write_text("dirty\n", encoding="utf-8")

        with self.assertRaisesRegex(wasm_perf.ContractError, "source checkout is dirty"):
            wasm_perf.capture_source_provenance(self.repo, self.runtime)

    def test_rejects_measured_source_mutated_after_capture(self):
        provenance = wasm_perf.capture_source_provenance(self.repo, self.runtime)
        (self.repo / "measured.rs").write_text("mutated mid-run\n", encoding="utf-8")

        with self.assertRaisesRegex(wasm_perf.ContractError, "changed after capture"):
            wasm_perf.verify_source_provenance(
                provenance, self.repo, self.runtime
            )

    def test_allows_declared_generated_outputs_without_allowing_other_files(self):
        generated = self.repo / "evidence" / "report.json"
        generated.parent.mkdir()
        generated.write_text("generated\n", encoding="utf-8")

        provenance = wasm_perf.capture_source_provenance(
            self.repo, self.runtime, allowed_outputs=[generated]
        )
        self.assertEqual(len(provenance["repo_sha"]), 40)
        self.assertEqual(len(provenance["repo_tree_sha"]), 40)

        (self.repo / "other-untracked.rs").write_text("source\n", encoding="utf-8")
        with self.assertRaisesRegex(wasm_perf.ContractError, "other-untracked.rs"):
            wasm_perf.verify_source_provenance(
                provenance,
                self.repo,
                self.runtime,
                allowed_outputs=[generated],
            )

    def test_rejects_output_allowance_that_contains_tracked_source(self):
        with self.assertRaisesRegex(
            wasm_perf.ContractError, "allowance contains tracked source"
        ):
            wasm_perf.capture_source_provenance(
                self.repo,
                self.runtime,
                allowed_outputs=[self.repo],
            )

    def test_seals_artifact_hashes_and_rejects_mid_run_artifact_mutation(self):
        wasm = self.repo / "generated" / "runner.wasm"
        native = self.repo / "generated" / "native-runner"
        wasm.parent.mkdir()
        wasm.write_bytes(b"wasm-v1")
        native.write_bytes(b"native-v1")
        allowed = [wasm.parent]
        sources = wasm_perf.capture_source_provenance(
            self.repo, self.runtime, allowed_outputs=allowed
        )

        sealed = wasm_perf.seal_run_provenance(
            sources,
            self.repo,
            self.runtime,
            artifacts={"wasm": wasm, "native_runner": native},
            allowed_outputs=allowed,
        )
        self.assertEqual(
            sealed["artifacts"]["wasm"]["sha256"],
            hashlib.sha256(b"wasm-v1").hexdigest(),
        )

        wasm.write_bytes(b"wasm-v2")
        with self.assertRaisesRegex(wasm_perf.ContractError, "artifact changed"):
            wasm_perf.verify_run_provenance(
                sealed,
                self.repo,
                self.runtime,
                allowed_outputs=allowed,
            )

    def test_post_seal_node_coordinator_swap_cannot_fabricate_browser_json(self):
        coordinator_sources = {}
        for name, contents in (
            (
                "run-wasm-perf.cjs",
                'require("node:fs").writeFileSync(process.argv[2], '
                'JSON.stringify({accepted: "sealed"}));\n',
            ),
            ("run-wasm-perf.sh", "#!/bin/sh\nexit 0\n"),
            ("wasm_perf.py", "#!/usr/bin/env python3\n"),
        ):
            path = self.repo / name
            path.write_text(contents, encoding="utf-8")
            coordinator_sources[name] = path
        subprocess.run(
            ["git", "add", *coordinator_sources], cwd=self.repo, check=True
        )
        subprocess.run(
            ["git", "commit", "-qm", "coordinators"], cwd=self.repo, check=True
        )
        generated = self.repo / "generated"
        generated.mkdir()
        bundle = wasm_perf.stage_coordinator_bundle_from_git(
            self.repo,
            {
                name: path.relative_to(self.repo).as_posix()
                for name, path in coordinator_sources.items()
            },
            generated / "coordinators",
        )
        sources = wasm_perf.capture_source_provenance(
            self.repo, self.runtime, allowed_outputs=[generated]
        )
        sealed = wasm_perf.seal_run_provenance(
            sources,
            self.repo,
            self.runtime,
            artifacts={"wasm_perf_node": bundle / "run-wasm-perf.cjs"},
            allowed_outputs=[generated],
        )

        bundled_node = bundle / "run-wasm-perf.cjs"
        original_bundled_node = bundled_node.read_bytes()
        descriptor = os.open(bundled_node, os.O_RDONLY)
        bundled_node.unlink()
        bundled_node.write_text(
            'require("node:fs").writeFileSync(process.argv[2], '
            'JSON.stringify({accepted: "fabricated"}));\n',
            encoding="utf-8",
        )
        output = generated / "browser.json"
        try:
            subprocess.run(
                ["node", f"/dev/fd/{descriptor}", str(output)],
                pass_fds=(descriptor,),
                check=True,
            )
        finally:
            os.close(descriptor)
        bundled_node.unlink()
        bundled_node.write_bytes(original_bundled_node)

        self.assertEqual(
            json.loads(output.read_text(encoding="utf-8")),
            {"accepted": "sealed"},
        )
        self.assertEqual(len(bundle.name), 64)
        wasm_perf.verify_run_provenance(
            sealed,
            self.repo,
            self.runtime,
            allowed_outputs=[generated],
        )
        wasm_perf.audit_python_coordinator_shell(
            Path(__file__).with_name("run-wasm-perf.sh").read_text(encoding="utf-8")
        )

    def test_rejects_pre_stage_node_coordinator_swap_then_restore(self):
        node = self.repo / "run-wasm-perf.cjs"
        committed = (
            'require("node:fs").writeFileSync(process.argv[2], '
            'JSON.stringify({accepted: "sealed"}));\n'
        )
        node.write_text(committed, encoding="utf-8")
        subprocess.run(["git", "add", node.name], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "coordinator"], cwd=self.repo, check=True
        )
        node.write_text(
            'require("node:fs").writeFileSync(process.argv[2], '
            'JSON.stringify({accepted: "fabricated"}));\n',
            encoding="utf-8",
        )

        output_root = self.repo / "generated" / "coordinators"
        try:
            with self.assertRaisesRegex(
                wasm_perf.ContractError, "coordinator source checkout is dirty"
            ):
                wasm_perf.stage_coordinator_bundle_from_git(
                    self.repo,
                    {node.name: node.name},
                    output_root,
                )
        finally:
            node.write_text(committed, encoding="utf-8")

        self.assertFalse(output_root.exists())
        self.assertEqual(
            subprocess.run(
                ["git", "status", "--porcelain=v1"],
                cwd=self.repo,
                text=True,
                capture_output=True,
                check=True,
            ).stdout,
            "",
        )

    def test_rejects_inherited_tracked_source_bundle_before_runtime_env(self):
        shell = Path(__file__).with_name("run-wasm-perf.sh")
        environment = os.environ.copy()
        environment.pop("RIVE_RUNTIME_DIR", None)
        environment.pop("RUST_GOLDEN_RUNNER", None)
        environment["WASM_PERF_SEALED_COORDINATOR_BUNDLE"] = str(shell.parent)

        completed = subprocess.run(
            [str(shell)],
            env=environment,
            text=True,
            capture_output=True,
            check=False,
        )

        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("invalid content-addressed coordinator bundle", completed.stderr)
        self.assertNotIn("RIVE_RUNTIME_DIR", completed.stderr)

    def test_rejects_python_replaced_between_validation_and_descriptor_open(self):
        source_dir = Path(__file__).parent
        coordinator_dir = self.repo / "tools" / "browser-renderer-smoke"
        coordinator_dir.mkdir(parents=True)
        names = (
            "run-wasm-perf.sh",
            "run-wasm-perf.cjs",
            "wasm_perf.py",
            "wasm-perf-driver-lib.cjs",
        )
        for name in names:
            (coordinator_dir / name).write_bytes((source_dir / name).read_bytes())
        subprocess.run(["git", "add", "tools"], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "coordinator bundle"],
            cwd=self.repo,
            check=True,
        )
        bundle = wasm_perf.stage_coordinator_bundle_from_git(
            self.repo,
            {
                name: f"tools/browser-renderer-smoke/{name}"
                for name in names
            },
            self.repo / "target" / "browser-wasm-perf" / "coordinators",
        )

        replacement = self.root / "replacement.py"
        replacement.write_text('raise SystemExit("fabricated coordinator executed")\n')
        shim_dir = self.root / "shim"
        shim_dir.mkdir()
        marker = self.root / "python-swapped"
        python_shim = shim_dir / "python3"
        python_shim.write_text(
            "#!/bin/sh\n"
            '"$REAL_PYTHON" "$@"\n'
            "result=$?\n"
            'if [ "$result" -eq 0 ] && [ ! -e "$SWAP_MARKER" ]; then\n'
            '  rm -f "$SWAP_TARGET"\n'
            '  mv "$SWAP_SOURCE" "$SWAP_TARGET"\n'
            '  : >"$SWAP_MARKER"\n'
            "fi\n"
            'exit "$result"\n',
            encoding="utf-8",
        )
        python_shim.chmod(0o755)
        environment = os.environ.copy()
        environment.pop("RIVE_RUNTIME_DIR", None)
        environment.pop("RUST_GOLDEN_RUNNER", None)
        environment.update(
            {
                "PATH": f"{shim_dir}{os.pathsep}{environment['PATH']}",
                "REAL_PYTHON": sys.executable,
                "SWAP_MARKER": str(marker),
                "SWAP_SOURCE": str(replacement),
                "SWAP_TARGET": str(bundle / "wasm_perf.py"),
                "WASM_PERF_SEALED_COORDINATOR_BUNDLE": str(bundle),
                "WASM_PERF_SOURCE_ROOT": str(self.repo),
            }
        )

        completed = subprocess.run(
            [str(bundle / "run-wasm-perf.sh")],
            env=environment,
            text=True,
            capture_output=True,
            check=False,
        )

        self.assertTrue(marker.is_file())
        self.assertNotEqual(completed.returncode, 0)
        self.assertIn(
            "opened coordinator descriptor differs from validated manifest",
            completed.stderr,
        )
        self.assertNotIn("fabricated coordinator executed", completed.stderr)
        self.assertNotIn("RIVE_RUNTIME_DIR", completed.stderr)

    def test_post_seal_python_swap_executes_original_open_descriptor(self):
        python_source = self.repo / "wasm_perf.py"
        python_source.write_text(
            'from pathlib import Path\nPath(__import__("sys").argv[1]).write_text("sealed")\n',
            encoding="utf-8",
        )
        subprocess.run(["git", "add", python_source.name], cwd=self.repo, check=True)
        subprocess.run(
            ["git", "commit", "-qm", "python coordinator"],
            cwd=self.repo,
            check=True,
        )
        generated = self.repo / "generated"
        bundle = wasm_perf.stage_coordinator_bundle_from_git(
            self.repo,
            {python_source.name: python_source.name},
            generated / "coordinators",
        )
        bundled_python = bundle / python_source.name
        descriptor = os.open(bundled_python, os.O_RDONLY)
        try:
            loader = (
                f"import os; os.lseek({descriptor}, 0, os.SEEK_SET); "
                f'source = b"".join(iter(lambda: os.read({descriptor}, 1048576), b"")); '
                'exec(compile(source, "<sealed-wasm-perf-coordinator>", "exec"), '
                '{"__name__": "__main__", '
                '"__file__": "<sealed-wasm-perf-coordinator>"})'
            )
            before_swap = generated / "python-result-before-swap.txt"
            subprocess.run(
                [sys.executable, "-c", loader, str(before_swap)],
                pass_fds=(descriptor,),
                check=True,
            )
            bundled_python.unlink()
            bundled_python.write_text(
                'from pathlib import Path\nPath(__import__("sys").argv[1]).write_text("fabricated")\n',
                encoding="utf-8",
            )
            output = generated / "python-result.txt"
            subprocess.run(
                [sys.executable, "-c", loader, str(output)],
                pass_fds=(descriptor,),
                check=True,
            )
        finally:
            os.close(descriptor)

        self.assertEqual(before_swap.read_text(encoding="utf-8"), "sealed")
        self.assertEqual(output.read_text(encoding="utf-8"), "sealed")
        wasm_perf.audit_python_coordinator_shell(
            Path(__file__).with_name("run-wasm-perf.sh").read_text(encoding="utf-8")
        )

    def test_seals_staged_fixture_and_rejects_mutation_after_seal(self):
        generated = self.repo / "generated"
        generated.mkdir()
        native = generated / "native-runner"
        native.write_bytes(b"native-v1")
        source_fixture = self.runtime / "fixture.riv"
        staged_fixture = generated / "fixture.riv"
        staged_fixture.write_bytes(source_fixture.read_bytes())
        fixture_bytes = source_fixture.read_bytes()
        fixture = {
            "id": "fixture",
            "path": str(source_fixture),
            "staged_path": str(staged_fixture),
            "bytes": len(fixture_bytes),
            "sha256": hashlib.sha256(fixture_bytes).hexdigest(),
        }
        allowed = [generated]
        sources = wasm_perf.capture_source_provenance(
            self.repo, self.runtime, allowed_outputs=allowed
        )
        sealed = wasm_perf.seal_run_provenance(
            sources,
            self.repo,
            self.runtime,
            artifacts={"native_runner": native},
            fixtures=[fixture],
            allowed_outputs=allowed,
        )

        staged_fixture.write_bytes(b"mutated")
        with self.assertRaisesRegex(
            wasm_perf.ContractError, "sealed fixture changed.*fixture.*staged"
        ):
            wasm_perf.verify_run_provenance(
                sealed,
                self.repo,
                self.runtime,
                allowed_outputs=allowed,
            )

    def test_rejects_browser_bytes_mutated_during_measurement_then_restored(self):
        expected_bytes = b"fixture-v1"
        expected = {
            "fixture": {
                "id": "fixture",
                "bytes": len(expected_bytes),
                "sha256": hashlib.sha256(expected_bytes).hexdigest(),
                "source_path": "/runtime/fixture.riv",
                "staged_path": "/target/fixture.riv",
            }
        }
        browser = {
            "loaded_fixtures": {
                "fixture": {
                    "bytes": len(b"fixture-v2"),
                    "sha256": hashlib.sha256(b"fixture-v2").hexdigest(),
                }
            }
        }

        with self.assertRaisesRegex(
            wasm_perf.ContractError, "browser loaded fixture identity mismatch.*fixture"
        ):
            wasm_perf.verify_browser_fixture_identities(expected, browser)

    def test_rejects_browser_artifacts_swapped_during_fetch_then_restored(self):
        expected_js = b"sealed wasm-bindgen javascript"
        expected_wasm = b"sealed wasm module"
        expected_html = b"sealed html"
        expected_driver = b"sealed driver"
        sealed = {
            "wasm_bindgen_js": {
                "path": "/pkg/browser_renderer_smoke.js",
                "bytes": len(expected_js),
                "sha256": hashlib.sha256(expected_js).hexdigest(),
            },
            "wasm": {
                "path": "/pkg/browser_renderer_smoke_bg.wasm",
                "bytes": len(expected_wasm),
                "sha256": hashlib.sha256(expected_wasm).hexdigest(),
            },
            "wasm_perf_html": {
                "path": "/harness/wasm-perf.html",
                "bytes": len(expected_html),
                "sha256": hashlib.sha256(expected_html).hexdigest(),
            },
            "wasm_perf_driver_js": {
                "path": "/harness/wasm-perf-driver-lib.cjs",
                "bytes": len(expected_driver),
                "sha256": hashlib.sha256(expected_driver).hexdigest(),
            },
        }
        browser = {
            "loaded_artifacts": {
                "wasm_bindgen_js": {
                    "bytes": len(b"swapped javascript"),
                    "sha256": hashlib.sha256(b"swapped javascript").hexdigest(),
                },
                "wasm": {
                    "bytes": len(expected_wasm),
                    "sha256": hashlib.sha256(expected_wasm).hexdigest(),
                },
                "wasm_perf_html": {
                    "bytes": len(expected_html),
                    "sha256": hashlib.sha256(expected_html).hexdigest(),
                },
                "wasm_perf_driver_js": {
                    "bytes": len(expected_driver),
                    "sha256": hashlib.sha256(expected_driver).hexdigest(),
                },
            }
        }

        with self.assertRaisesRegex(
            wasm_perf.ContractError,
            "browser loaded artifact identity mismatch.*wasm_bindgen_js",
        ):
            wasm_perf.verify_browser_artifact_identities(sealed, browser)

    def test_rejects_provenance_seal_that_omits_browser_harness(self):
        sealed = {
            name: {"path": f"/{name}", "bytes": 1, "sha256": "a" * 64}
            for name in ("wasm", "wasm_bindgen_js")
        }
        browser = {
            "loaded_artifacts": {
                name: {"bytes": 1, "sha256": "a" * 64}
                for name in ("wasm", "wasm_bindgen_js")
            }
        }

        with self.assertRaisesRegex(
            wasm_perf.ContractError, "seal omitted browser artifacts"
        ):
            wasm_perf.verify_browser_artifact_identities(sealed, browser)

    def test_rejects_harness_bytes_swapped_during_execution_then_restored(self):
        contents = {
            "wasm_perf_html": b"sealed html harness",
            "wasm_perf_driver_js": b"sealed driver harness",
            "wasm_bindgen_js": b"sealed wasm-bindgen javascript",
            "wasm": b"sealed wasm module",
        }
        sealed = {
            name: {
                "path": f"/harness/{name}",
                "bytes": len(value),
                "sha256": hashlib.sha256(value).hexdigest(),
            }
            for name, value in contents.items()
        }
        swapped_driver = b"fabricate accepted timings"
        loaded = {
            name: {
                "bytes": len(value),
                "sha256": hashlib.sha256(value).hexdigest(),
            }
            for name, value in contents.items()
        }
        loaded["wasm_perf_driver_js"] = {
            "bytes": len(swapped_driver),
            "sha256": hashlib.sha256(swapped_driver).hexdigest(),
        }

        with self.assertRaisesRegex(
            wasm_perf.ContractError,
            "browser loaded artifact identity mismatch.*wasm_perf_driver_js",
        ):
            wasm_perf.verify_browser_artifact_identities(
                sealed, {"loaded_artifacts": loaded}
            )

    def test_native_runs_execute_a_content_addressed_sealed_runner_copy(self):
        generated = self.repo / "generated"
        generated.mkdir()
        marker = generated / "executed-path.txt"
        runner = generated / "native-runner"
        runner.write_text(
            "#!/bin/sh\n"
            "set -eu\n"
            f"printf '%s' \"$0\" > '{marker}'\n"
            "printf 'rive-golden-benchmark-v1\\n"
            "elapsed_ms=1\\ntotal_ms=1\\nadvance_ms=0.4\\ninput_ms=0\\n"
            "prepare_ms=0\\ndraw_ms=0.5\\nbookkeeping_ms=0.1\\nsegments=1\\n"
            "scene_kind=state_machine\\ndefault_state_machine_id=0\\n"
            "view_model_initialization=schema-default\\n'\n",
            encoding="utf-8",
        )
        runner.chmod(0o755)
        fixture = generated / "fixture.riv"
        fixture.write_bytes(b"sealed fixture")
        fixture_sha256 = hashlib.sha256(fixture.read_bytes()).hexdigest()
        runner_bytes = runner.read_bytes()
        runner_sha256 = hashlib.sha256(runner_bytes).hexdigest()

        wasm_perf._native_runs(
            {
                "runs": 1,
                "repeat": 1,
                "fixtures": [
                    {
                        "id": "fixture@0s",
                        "fixture_id": "fixture",
                        "sample_index": 0,
                        "sample_seconds": 0.0,
                    }
                ],
            },
            runner,
            {
                "fixture@0s": {
                    "id": "fixture@0s",
                    "staged_path": str(fixture),
                    "sha256": fixture_sha256,
                    "sample_seconds": 0.0,
                }
            },
            {
                "path": str(runner),
                "bytes": len(runner_bytes),
                "sha256": runner_sha256,
            },
        )

        executed_path = Path(marker.read_text(encoding="utf-8"))
        self.assertNotEqual(executed_path, runner)
        self.assertEqual(executed_path.name, runner_sha256)

    def test_accepts_multiple_browser_fixtures_with_exact_sealed_identities(self):
        sealed = {}
        loaded = {}
        for fixture_id, contents in (("large", b"large"), ("small", b"small")):
            identity = {
                "bytes": len(contents),
                "sha256": hashlib.sha256(contents).hexdigest(),
            }
            sealed[fixture_id] = {
                "id": fixture_id,
                **identity,
                "source_path": f"/runtime/{fixture_id}.riv",
                "staged_path": f"/target/{fixture_id}.riv",
            }
            loaded[fixture_id] = identity

        wasm_perf.verify_browser_fixture_identities(
            sealed, {"loaded_fixtures": loaded}
        )

    def test_finalize_accepts_multiple_fixtures_with_one_sealed_byte_identity(self):
        args, fixture_ids = self._finalize_fixture_run(fixture_count=2)

        wasm_perf.finalize_run(args)

        report = json.loads(args.output.read_text(encoding="utf-8"))
        self.assertEqual([row["id"] for row in report["fixtures"]], fixture_ids)

    def test_finalize_rejects_fixture_mutated_during_native_measurement(self):
        args, _fixture_ids = self._finalize_fixture_run(
            fixture_count=1, mutate_during_native=True
        )

        with self.assertRaisesRegex(
            wasm_perf.ContractError, "sealed fixture changed.*fixture-0.*staged"
        ):
            wasm_perf.finalize_run(args)

    def test_finalize_rejects_native_fixture_redirected_after_seal(self):
        args, _fixture_ids = self._finalize_fixture_run(fixture_count=1)
        alternate = args.config.parent / "alternate.riv"
        alternate_bytes = b"alternate-fixture"
        alternate.write_bytes(alternate_bytes)
        config = json.loads(args.config.read_text(encoding="utf-8"))
        config["fixtures"][0]["staged_path"] = str(alternate)
        config["fixtures"][0]["bytes"] = len(alternate_bytes)
        config["fixtures"][0]["sha256"] = hashlib.sha256(alternate_bytes).hexdigest()
        config["fixtures"][0]["sample_seconds"] = 1.0
        args.config.write_text(wasm_perf.canonical_json(config), encoding="utf-8")

        with self.assertRaisesRegex(
            wasm_perf.ContractError, "config fixture differs from sealed fixture"
        ):
            wasm_perf.finalize_run(args)

    def test_finalize_rejects_browser_measurement_changed_after_seal(self):
        args, _fixture_ids = self._finalize_fixture_run(fixture_count=1)
        browser = json.loads(args.browser_results.read_text(encoding="utf-8"))
        browser["measurement"]["fixtures"][0]["sample_seconds"] = 1.0
        args.browser_results.write_text(
            wasm_perf.canonical_json(browser), encoding="utf-8"
        )

        with self.assertRaisesRegex(
            wasm_perf.ContractError,
            "browser measurement contract differs from sealed measurement",
        ):
            wasm_perf.finalize_run(args)

    def test_finalize_rejects_source_identity_forged_after_seal(self):
        args, _fixture_ids = self._finalize_fixture_run(fixture_count=1)
        config = json.loads(args.config.read_text(encoding="utf-8"))
        config["identity"]["git_sha"] = "forged-repo-sha"
        config["identity"]["rive_runtime_sha"] = "forged-runtime-sha"
        args.config.write_text(wasm_perf.canonical_json(config), encoding="utf-8")

        with self.assertRaisesRegex(
            wasm_perf.ContractError, "config identity differs from sealed identity"
        ):
            wasm_perf.finalize_run(args)

    def test_finalize_rejects_provenance_and_identity_forged_together(self):
        args, _fixture_ids = self._finalize_fixture_run(fixture_count=1)
        config = json.loads(args.config.read_text(encoding="utf-8"))
        config["provenance"]["run_identity"]["build_profile"] = "forged-profile"
        config["identity"] = wasm_perf.sealed_config_identity(config["provenance"])
        args.config.write_text(wasm_perf.canonical_json(config), encoding="utf-8")

        with self.assertRaisesRegex(
            wasm_perf.ContractError, "config provenance differs from anchored seal"
        ):
            wasm_perf.finalize_run(args)

    def _finalize_fixture_run(
        self, *, fixture_count: int, mutate_during_native: bool = False
    ) -> tuple[SimpleNamespace, list[str]]:
        generated = self.repo / "generated"
        generated.mkdir()
        runner = generated / "native-runner"
        mutation = 'printf "mutated" > "$2"\n' if mutate_during_native else ""
        runner.write_text(
            "#!/bin/sh\n"
            "set -eu\n"
            f"{mutation}"
            "printf 'rive-golden-benchmark-v1\\n"
            "elapsed_ms=1\\ntotal_ms=1\\nadvance_ms=0.4\\ninput_ms=0\\n"
            "prepare_ms=0\\ndraw_ms=0.5\\nbookkeeping_ms=0.1\\nsegments=1\\n"
            "scene_kind=state_machine\\ndefault_state_machine_id=0\\n"
            "view_model_initialization=schema-default\\n'\n",
            encoding="utf-8",
        )
        runner.chmod(0o755)
        source_fixture = self.runtime / "fixture.riv"
        fixture_bytes = source_fixture.read_bytes()
        fixture_identity = {
            "bytes": len(fixture_bytes),
            "sha256": hashlib.sha256(fixture_bytes).hexdigest(),
        }
        fixtures = []
        browser_runs = {}
        loaded_fixtures = {}
        fixture_ids = []
        for index in range(fixture_count):
            fixture_id = f"fixture-{index}"
            staged = generated / f"{fixture_id}.riv"
            staged.write_bytes(fixture_bytes)
            fixtures.append(
                {
                    "id": fixture_id,
                    "fixture_id": fixture_id,
                    "sample_index": 0,
                    **fixture_identity,
                    "path": str(source_fixture),
                    "staged_path": str(staged),
                    "relative_path": "fixture.riv",
                    "sample_seconds": 0.0,
                }
            )
            browser_runs[fixture_id] = [
                timing(2.0, 0.8, 1.0, 1, target=fixtures[-1])
            ]
            loaded_fixtures[fixture_id] = fixture_identity
            fixture_ids.append(fixture_id)
        allowed = [generated]
        sources = wasm_perf.capture_source_provenance(
            self.repo, self.runtime, allowed_outputs=allowed
        )
        artifacts = {"native_runner": runner}
        loaded_artifacts = {}
        for name in (
            "wasm",
            "wasm_bindgen_js",
            "wasm_perf_driver_js",
            "wasm_perf_html",
        ):
            path = generated / name
            contents = f"sealed-{name}".encode()
            path.write_bytes(contents)
            artifacts[name] = path
            loaded_artifacts[name] = {
                "bytes": len(contents),
                "sha256": hashlib.sha256(contents).hexdigest(),
            }
        node_path = generated / "wasm_perf_node"
        node_contents = b"sealed-wasm-perf-node"
        node_path.write_bytes(node_contents)
        artifacts["wasm_perf_node"] = node_path
        coordinator_artifacts = {
            "wasm_perf_node": {
                "bytes": len(node_contents),
                "sha256": hashlib.sha256(node_contents).hexdigest(),
            }
        }
        sealed = wasm_perf.seal_run_provenance(
            sources,
            self.repo,
            self.runtime,
            artifacts=artifacts,
            fixtures=fixtures,
            measurement={"repeat": 1, "runs": 1, "warmups": 0},
            run_identity={"build_profile": "release"},
            allowed_outputs=allowed,
        )
        seal = generated / "seal.json"
        seal_sha256 = wasm_perf.write_run_seal(seal, sealed)
        config = generated / "config.json"
        config.write_text(
            wasm_perf.canonical_json(
                {
                    "schema": "nuxie-wasm-perf-config-v1",
                    "repeat": 1,
                    "runs": 1,
                    "warmups": 0,
                    "identity": wasm_perf.sealed_config_identity(sealed),
                    "provenance": sealed,
                    "fixtures": fixtures,
                }
            ),
            encoding="utf-8",
        )
        browser_results = generated / "browser.json"
        browser_results.write_text(
            wasm_perf.canonical_json(
                {
                    "schema": "nuxie-wasm-perf-browser-raw-v1",
                    "browser": "chromium",
                    "browser_version": "test",
                    "seal_sha256": seal_sha256,
                    "measurement": {
                        "repeat": 1,
                        "runs": 1,
                        "warmups": 0,
                        "fixtures": [
                            {
                                key: fixture[key]
                                for key in (
                                    "id",
                                    "fixture_id",
                                    "sample_index",
                                    "sample_seconds",
                                    "bytes",
                                    "sha256",
                                )
                            }
                            for fixture in fixtures
                        ],
                    },
                    "loaded_fixtures": loaded_fixtures,
                    "loaded_artifacts": loaded_artifacts,
                    "coordinator_artifacts": coordinator_artifacts,
                    "fixtures": browser_runs,
                }
            ),
            encoding="utf-8",
        )
        return (
            SimpleNamespace(
                config=config,
                seal=seal,
                expected_seal_sha256=seal_sha256,
                browser_results=browser_results,
                native_runner=runner,
                repo_root=self.repo,
                rive_runtime_dir=self.runtime,
                allowed_output=allowed,
                output=generated / "report.json",
                markdown=None,
            ),
            fixture_ids,
        )


class CorpusSelectionTests(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        (self.root / "assets").mkdir()
        (self.root / "assets" / "large.riv").write_bytes(b"L" * 100)
        (self.root / "assets" / "small.riv").write_bytes(b"S" * 10)
        (self.root / "assets" / "scripted.riv").write_bytes(b"X" * 90)
        self.corpus = self.root / "corpus.toml"
        self.corpus.write_text(
            """
[[file]]
id = "large"
path = "assets/large.riv"
samples = [0.0, 0.5]

[[file]]
id = "scripted"
path = "assets/scripted.riv"
input_script = "inputs/scripted.txt"
samples = [0.0]

[[file]]
id = "small"
path = "assets/small.riv"
samples = [0.25]
""",
            encoding="utf-8",
        )
        self.perf = self.root / "perf.toml"
        self.perf.write_text(
            """
schema = "nuxie-perf-corpus-v1"
source = "corpus.toml"

[[file]]
id = "large"
bytes = 100
categories = ["largest"]

[[file]]
id = "scripted"
bytes = 90
categories = ["largest", "scripted"]

[[file]]
id = "small"
bytes = 10
categories = ["largest"]
""",
            encoding="utf-8",
        )

    def tearDown(self):
        self.temp_dir.cleanup()

    def test_selects_largest_supported_and_expands_every_sample_target(self):
        fixtures = wasm_perf.select_fixtures(
            self.perf, self.corpus, self.root, limit=2, requested_ids=[]
        )

        self.assertEqual(
            [fixture["id"] for fixture in fixtures],
            ["large@0s", "large@0.5s", "small@0.25s"],
        )
        self.assertEqual(
            [
                (
                    fixture["fixture_id"],
                    fixture["sample_index"],
                    fixture["sample_seconds"],
                )
                for fixture in fixtures
            ],
            [
                ("large", 0, 0.0),
                ("large", 1, 0.5),
                ("small", 0, 0.25),
            ],
        )

    def test_explicit_unsupported_fixture_fails_closed(self):
        with self.assertRaisesRegex(wasm_perf.ContractError, "scripted semantics"):
            wasm_perf.select_fixtures(
                self.perf, self.corpus, self.root, limit=1, requested_ids=["scripted"]
            )

    def test_explicit_image_fixture_fails_closed_without_production_decoder(self):
        corpus = self.corpus.read_text(encoding="utf-8").replace(
            'id = "large"\npath = "assets/large.riv"',
            'id = "large"\npath = "assets/large.riv"\nfeatures = ["type-key:105:ImageAsset"]',
        )
        self.corpus.write_text(corpus, encoding="utf-8")

        with self.assertRaisesRegex(wasm_perf.ContractError, "image decode semantics"):
            wasm_perf.select_fixtures(
                self.perf, self.corpus, self.root, limit=1, requested_ids=["large"]
            )

    def test_missing_fixture_fails_closed(self):
        (self.root / "assets" / "large.riv").unlink()

        with self.assertRaisesRegex(wasm_perf.ContractError, "missing fixture"):
            wasm_perf.select_fixtures(
                self.perf, self.corpus, self.root, limit=1, requested_ids=["large"]
            )

    def test_unknown_requested_id_fails_closed(self):
        with self.assertRaisesRegex(wasm_perf.ContractError, "unknown perf fixture"):
            wasm_perf.select_fixtures(
                self.perf, self.corpus, self.root, limit=1, requested_ids=["absent"]
            )


class ReportContractTests(unittest.TestCase):
    def test_audits_production_only_feature_tree_and_import(self):
        wasm_perf.audit_production_boundary(
            'browser-renderer-smoke\n└── nuxie feature "default"',
            """pub struct WasmPerfRunner;
impl WasmPerfRunner {
    set_runtime_deterministic_mode(true);
    File::import(bytes);
    instance.instantiate_view_model();
    instance.artboard().default_state_machine_index();
}
""",
        )

    def test_rejects_test_support_in_measured_feature_tree(self):
        with self.assertRaisesRegex(wasm_perf.ContractError, "test-support"):
            wasm_perf.audit_production_boundary(
                'nuxie feature "test-support"',
                """pub struct WasmPerfRunner;
File::import(bytes);
instance.instantiate_view_model();
instance.artboard().default_state_machine_index();
""",
            )

    def test_rejects_unsigned_import_in_measured_runner(self):
        with self.assertRaisesRegex(wasm_perf.ContractError, "production File::import"):
            wasm_perf.audit_production_boundary(
                'nuxie feature "default"',
                """pub struct WasmPerfRunner;
File::import_with_unsigned_scripts(bytes);
instance.instantiate_view_model();
instance.artboard().default_state_machine_index();
""",
            )

    def test_rejects_authored_default_view_model_initialization(self):
        with self.assertRaisesRegex(wasm_perf.ContractError, "schema-default view model"):
            wasm_perf.audit_production_boundary(
                'nuxie feature "default"',
                """pub struct WasmPerfRunner;
set_runtime_deterministic_mode(true);
File::import(bytes);
instance.instantiate_default_view_model_instance();
instance.artboard().default_state_machine_index();
""",
            )

    def test_rejects_fallback_to_state_machine_zero_without_authored_default(self):
        with self.assertRaisesRegex(wasm_perf.ContractError, "authored default state machine"):
            wasm_perf.audit_production_boundary(
                'nuxie feature "default"',
                """pub struct WasmPerfRunner;
set_runtime_deterministic_mode(true);
File::import(bytes);
instance.instantiate_view_model();
instance.default_state_machine_instance();
""",
            )

    def test_rejects_wasm_perf_runner_without_native_deterministic_mode(self):
        with self.assertRaisesRegex(
            wasm_perf.ContractError, "deterministic runtime mode before import"
        ):
            wasm_perf.audit_production_boundary(
                'nuxie feature "default"',
                """pub struct WasmPerfRunner;
impl WasmPerfRunner {
    File::import(bytes);
    instance.instantiate_view_model();
    instance.artboard().default_state_machine_index();
}
""",
            )

    def test_parses_native_report_contract(self):
        parsed = wasm_perf.parse_native_report(
            """rive-golden-benchmark-v1
elapsed_ms=12.5
total_ms=12.5
advance_ms=3.0
input_ms=0
prepare_ms=0
draw_ms=8.0
bookkeeping_ms=0.5
segments=10
scene_kind=state_machine
default_state_machine_id=0
view_model_initialization=schema-default
"""
        )

        self.assertEqual(parsed["schema"], "rive-golden-benchmark-v1")
        self.assertEqual(parsed["segments"], 10)
        self.assertEqual(parsed["accounted_ms"], 11.0)
        self.assertEqual(
            parsed["workload_identity"],
            workload_identity(),
        )

    def test_rejects_incomplete_browser_report(self):
        report = {
            "schema": "rive-golden-benchmark-v1",
            "elapsed_ms": 1.0,
            "advance_ms": 0.5,
            "draw_ms": 0.4,
            "segments": 1,
        }

        with self.assertRaisesRegex(wasm_perf.ContractError, "missing report field"):
            wasm_perf.validate_timing_report(report)

    def test_builds_report_only_comparison_with_variance(self):
        fixture = {
            "id": "large",
            "fixture_id": "large",
            "sample_index": 0,
            "bytes": 100,
            "relative_path": "assets/large.riv",
            "sample_seconds": 0.0,
        }
        wasm_runs = [
            timing(12.0, 3.0, 8.0, 10, target=fixture),
            timing(14.0, 4.0, 9.0, 10, target=fixture),
            timing(13.0, 3.5, 8.5, 10, target=fixture),
        ]
        native_runs = [
            timing(6.0, 1.0, 4.0, 10, target=fixture),
            timing(7.0, 1.5, 4.5, 10, target=fixture),
            timing(6.5, 1.25, 4.25, 10, target=fixture),
        ]

        report = wasm_perf.build_comparison_report(
            [fixture],
            {"large": wasm_runs},
            {"large": native_runs},
            identity={
                "git_sha": "abc123",
                "rive_runtime_sha": "def456",
                "browser": "chrome",
                "build_profile": "release",
            },
            repeat=10,
            warmups=1,
        )

        row = report["fixtures"][0]
        self.assertEqual(report["schema"], "nuxie-wasm-perf-v1")
        self.assertEqual(report["conclusion"], "report-only")
        self.assertEqual(row["wasm"]["run_count"], 3)
        self.assertAlmostEqual(row["ratio"]["elapsed"], 2.0)
        self.assertGreater(row["wasm"]["elapsed_ms"]["coefficient_of_variation"], 0)
        self.assertEqual(row["workload_identity"], workload_identity())
        self.assertIn(
            "state machine 0; VM schema-default",
            wasm_perf.render_markdown(report),
        )

    def test_rejects_data_bind_authored_default_against_native_schema_default(self):
        fixture = {
            "id": "data_bind_test_cmdq",
            "fixture_id": "data_bind_test_cmdq",
            "sample_index": 0,
            "bytes": 100,
            "relative_path": "assets/data_bind_test_cmdq.riv",
            "sample_seconds": 0.0,
        }
        wasm = timing(12.0, 3.0, 8.0, 10, target=fixture)
        wasm["workload_identity"] = {
            **workload_identity(),
            "view_model_initialization": "none",
        }

        with self.assertRaisesRegex(
            wasm_perf.ContractError, "data_bind_test_cmdq workload identity mismatch"
        ):
            wasm_perf.build_comparison_report(
                [fixture],
                {"data_bind_test_cmdq": [wasm, wasm]},
                {
                    "data_bind_test_cmdq": [
                        timing(6.0, 1.0, 4.0, 10, target=fixture),
                        timing(7.0, 1.5, 4.5, 10, target=fixture),
                    ]
                },
                identity={
                    "git_sha": "abc123",
                    "rive_runtime_sha": "def456",
                    "browser": "chrome",
                    "build_profile": "release",
                },
                repeat=10,
                warmups=1,
            )

    def test_rejects_native_and_wasm_runs_for_different_sample_targets(self):
        fixture = {
            "id": "large@0.5s",
            "fixture_id": "large",
            "sample_index": 1,
            "sample_seconds": 0.5,
            "bytes": 100,
            "relative_path": "assets/large.riv",
        }
        native = timing(6.0, 1.0, 4.0, 10, target=fixture)
        wrong_wasm_target = {**fixture, "sample_index": 0, "sample_seconds": 0.0}
        wasm = timing(12.0, 3.0, 8.0, 10, target=wrong_wasm_target)

        with self.assertRaisesRegex(
            wasm_perf.ContractError, "wasm sample target identity mismatch"
        ):
            wasm_perf.build_comparison_report(
                [fixture],
                {fixture["id"]: [wasm, wasm]},
                {fixture["id"]: [native, native]},
                identity={
                    "git_sha": "abc123",
                    "rive_runtime_sha": "def456",
                    "browser": "chrome",
                    "build_profile": "release",
                },
                repeat=10,
                warmups=1,
            )

    def test_round_trips_machine_readable_json(self):
        payload = {"schema": "nuxie-wasm-perf-v1", "conclusion": "report-only"}
        self.assertEqual(json.loads(wasm_perf.canonical_json(payload)), payload)


def timing(elapsed, advance, draw, segments, *, target=None):
    accounted = advance + draw
    report = {
        "schema": "rive-golden-benchmark-v1",
        "elapsed_ms": elapsed,
        "total_ms": elapsed,
        "advance_ms": advance,
        "input_ms": 0.0,
        "prepare_ms": 0.0,
        "draw_ms": draw,
        "accounted_ms": accounted,
        "bookkeeping_ms": max(elapsed - accounted, 0.0),
        "segments": segments,
        "workload_identity": workload_identity(),
    }
    if target is not None:
        report["target_identity"] = wasm_perf._target_identity(target)
    return report


def workload_identity():
    return {
        "scene_kind": "state_machine",
        "default_state_machine_id": 0,
        "view_model_initialization": "schema-default",
    }


if __name__ == "__main__":
    unittest.main()
