#!/usr/bin/env python3
"""Exercise contamination guards and the complete orchestration without compiling Rust."""
import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("runner_pgo", Path(__file__).with_name("runner_pgo.py"))
pgo = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pgo)


class PgoTests(unittest.TestCase):
    def test_environment_cannot_inherit_instrumentation_or_profile_overrides(self):
        dirty = {key: "bad" for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER", "CARGO_PROFILE_RELEASE_LTO", "CARGO_INCREMENTAL", "CARGO_TARGET_X_RUSTFLAGS", "LLVM_PROFILE_FILE", "CC_aarch64_apple_darwin", "HOST_CXXFLAGS")}
        dirty.update(PATH="bin", CARGO_HOME="cache")
        self.assertEqual(pgo.clean_environment(dirty), {"PATH": "bin", "CARGO_HOME": "cache"})

    def test_profile_diagnostics_preserve_parallel_log_context_without_claiming_origin(self):
        result = pgo.profile_diagnostics("warning: no profile data available for function x Hash = 1\n     Running `rustc --crate-name nuxie_runtime`\n     Running `rustc --crate-name nuxie_scripting`\nnote: pgo-icall-prom (success): Promote indirect call to runtime_target\n     Running `rustc --crate-name rust_golden_runner`\nnote: pgo-icall-prom (success): Promote indirect call to target\nwarning: profile data may be out of date (hash mismatch)\n")
        self.assertEqual(len(result["missing_profile_warnings"]), 1)
        self.assertEqual(len(result["hash_mismatch_warnings"]), 1)
        self.assertEqual(result["indirect_call_promotion_count"], 2)
        self.assertEqual(result["final_runner_invocation_line"], 5)
        self.assertEqual(result["promotions_before_final_runner_invocation"], 1)
        self.assertEqual(result["promotions_after_final_runner_invocation"], 1)
        first = result["indirect_call_promotions"][0]
        self.assertEqual(first["preceding_invocation_crate"], "nuxie_scripting")
        self.assertNotIn("crate", first)
        missing_invocation = pgo.profile_diagnostics("note: pgo-icall-prom (success): Promote indirect call to target\n")
        self.assertEqual(missing_invocation["indirect_call_promotion_count"], 1)
        self.assertIsNone(missing_invocation["final_runner_invocation_line"])
        self.assertIsNone(missing_invocation["promotions_before_final_runner_invocation"])
        self.assertIsNone(missing_invocation["promotions_after_final_runner_invocation"])

    def experiment(self, directory):
        repo, assets = directory / "repo", directory / "assets"
        repo.mkdir(); assets.mkdir()
        (repo / "Cargo.toml").write_text('[profile.release]\nlto="fat"\ncodegen-units=1\npanic="unwind"\n')
        (repo / "Cargo.lock").write_text("# frozen lock\n")
        fixtures = {}
        for name in (*pgo.TRAINING, *(f"held_out_{n}" for n in range(21))):
            file = assets / (name + ".riv")
            file.write_bytes(name.encode())
            fixtures[name] = {"file": file.name, "sha256": pgo.sha(file)}
        manifest = directory / "manifest.json"
        manifest.write_text(json.dumps({"schema": "nuxie-runner-pgo-fixtures/v1", "training": list(pgo.TRAINING), "fixtures": fixtures}))
        output = directory / "result"
        args = ["--repo", str(repo), "--revision", "test-commit", "--assets", str(assets), "--manifest", str(manifest), "--output", str(output), "--target", "test-host", "--cc", "cc", "--cxx", "cxx", "--llvm-profdata", "llvm-profdata"]
        return repo, assets, manifest, output, args

    def test_manifest_rejects_changed_assets_and_training_contamination(self):
        with tempfile.TemporaryDirectory() as tmp:
            _, assets, manifest, _, _ = self.experiment(Path(tmp))
            data = json.loads(manifest.read_text())
            data["training"][0] = "held_out_0"
            manifest.write_text(json.dumps(data))
            with self.assertRaisesRegex(ValueError, "four named fixtures"):
                pgo.load_manifest(manifest, assets)
            data["training"] = list(pgo.TRAINING)
            manifest.write_text(json.dumps(data))
            (assets / "held_out_0.riv").write_bytes(b"changed")
            with self.assertRaisesRegex(ValueError, "fixture hash mismatch"):
                pgo.load_manifest(manifest, assets)

    def test_complete_pipeline_preserves_training_holdout_split_and_matched_flags(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo, _, _, output, args = self.experiment(root)
            def version(argv, **kwargs):
                if argv[0] == "git": return "frozen-commit\n"
                if argv[0] == "rustc": return "rustc test\nhost: test-host\nLLVM version: 22.1.8\n"
                return "LLVM version 22.1.8\n"
            def execute(argv, cwd, env, stdout, stderr):
                if argv[0] == "git":
                    with tarfile.open(argv[3].removeprefix("--output="), "w") as archive:
                        for file in repo.iterdir(): archive.add(file, arcname=file.name)
                elif argv[0] == "cargo":
                    flags = env["CARGO_ENCODED_RUSTFLAGS"]
                    self.assertNotIn("RUSTFLAGS", env)
                    lane = "generate" if "profile-generate" in flags else "pgo" if "profile-use" in flags else "control"
                    binary = output / "target/test-host/release/rust-golden-runner"
                    binary.parent.mkdir(parents=True, exist_ok=True)
                    binary.write_text(lane)
                    stderr.write(b"Running `rustc --crate-name rust_golden_runner`\nnote: pgo-icall-prom (success): Promote indirect call to target\n")
                elif argv[0] == "llvm-profdata":
                    if argv[1] == "merge": Path(argv[4]).write_bytes(b"profile")
                elif "--benchmark" in argv:
                    samples = argv[argv.index("--samples") + 1].split(",")
                    self.assertEqual(len(samples), 10000)
                    self.assertEqual(samples[1], "0.016666667")
                    raw = env["LLVM_PROFILE_FILE"].replace("%m", "module").replace("%p", "pid")
                    Path(raw).write_bytes(b"raw")
                    stdout.write(b"segments=10000\n")
                else:
                    self.assertNotIn("LLVM_PROFILE_FILE", env)
                    stdout.write(b"recording\n"); stderr.write(b"side-channel\n")
                return subprocess.CompletedProcess(argv, 0)
            environment = {"PATH": os.environ["PATH"], "CARGO_HOME": str(root / "cargo-home"), "RUSTFLAGS": "contamination"}
            with patch.dict(os.environ, environment, clear=True), patch.object(pgo, "executable", side_effect=lambda name: name), patch.object(pgo, "pin_toolchain", side_effect=lambda tools, env: tools), patch.object(pgo, "tool_hashes", return_value={"rustc": "compiler"}), patch.object(pgo.subprocess, "check_output", side_effect=version), patch.object(pgo.subprocess, "run", side_effect=execute), contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(pgo.main(args), 0)
            commands = json.loads((output / "commands.json").read_text())
            self.assertEqual([r["name"][6:] for r in commands if r["name"].startswith("train-")], list(pgo.TRAINING))
            builds = [r for r in commands if r["name"].endswith("-build")]
            self.assertEqual(len(builds), 3)
            self.assertEqual(builds[0]["overrides"]["CARGO_ENCODED_RUSTFLAGS"], "")
            self.assertIn("-Cprofile-generate=", builds[1]["overrides"]["CARGO_ENCODED_RUSTFLAGS"])
            self.assertIn("-Cprofile-use=", builds[2]["overrides"]["CARGO_ENCODED_RUSTFLAGS"])
            validation = json.loads((output / "validation.json").read_text())
            self.assertEqual(sum(row["held_out"] for row in validation), 21)
            self.assertTrue(all(row["exact"] for row in validation))
            self.assertEqual(json.loads((output / "provenance.json").read_text())["status"], "validated")

    def test_incompatible_llvm_fails_before_any_build_or_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            _, _, _, output, args = self.experiment(Path(tmp))
            def version(argv, **kwargs):
                return "host: test-host\nLLVM version: 22.1.8\n" if argv[0] == "rustc" else "LLVM version 21.1.0\n"
            with patch.object(pgo, "executable", side_effect=lambda name: name), patch.object(pgo, "pin_toolchain", side_effect=lambda tools, env: tools), patch.object(pgo, "tool_hashes", return_value={"rustc": "compiler"}), patch.object(pgo.subprocess, "check_output", side_effect=version), patch.object(pgo.subprocess, "run") as execute:
                with self.assertRaisesRegex(ValueError, "LLVM major versions must match"):
                    pgo.main(args)
                execute.assert_not_called()
            self.assertFalse(output.exists())

    def test_toolchain_pinning_replaces_rustup_proxies_with_actual_binaries(self):
        with tempfile.TemporaryDirectory() as tmp:
            sysroot = Path(tmp).resolve()
            (sysroot / "bin").mkdir()
            for name in ("rustc", "cargo"):
                (sysroot / "bin" / name).write_bytes(name.encode())
            with patch.object(pgo.subprocess, "check_output", return_value=str(sysroot) + "\n"):
                tools = pgo.pin_toolchain({"rustc": "/proxy/rustc", "cargo": "/proxy/cargo"}, {})
            self.assertEqual(tools, {name: str(sysroot / "bin" / name) for name in ("rustc", "cargo")})

    def test_llvm_versions_are_read_explicitly(self):
        self.assertEqual(pgo.llvm_major("LLVM version: 22.1.8"), 22)
        self.assertEqual(pgo.llvm_major("Homebrew LLVM version 22.1.8"), 22)
        with self.assertRaisesRegex(ValueError, "cannot read LLVM version"):
            pgo.llvm_major("unknown profiler")


if __name__ == "__main__":
    unittest.main()
