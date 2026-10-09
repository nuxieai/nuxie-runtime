#!/usr/bin/env python3
"""Frozen-source runner PGO experiment; correctness validation, never a timing gate."""
from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tomllib

TRAINING = ("zombie_skins", "spotify_kids_demo", "car_widgets_v01", "data_viz_demo")
TRAIN_FRAMES = 10_000
VALIDATE_FRAMES = 100


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    with Path(path).open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def executable(name):
    path = shutil.which(name)
    require(path is not None, f"executable not found: {name}")
    return os.path.abspath(path)


def pin_toolchain(tools, env):
    # Resolve rustup proxies before entering the archive: rust-toolchain files
    # and directory overrides must not silently change the selected compiler.
    sysroot = Path(subprocess.check_output([tools["rustc"], "--print", "sysroot"], env=env, text=True).strip())
    suffix = ".exe" if os.name == "nt" else ""
    for name in ("rustc", "cargo"):
        candidate = sysroot / "bin" / (name + suffix)
        if not candidate.is_file() and name == "cargo":
            candidate = Path(tools[name]).resolve()
        require(candidate.is_file() and candidate.stem != "rustup", f"cannot pin actual {name} executable in {sysroot}")
        tools[name] = str(candidate.resolve())
    return tools


def tool_hashes(tools):
    return {name: sha(path) for name, path in tools.items() if name != "git"}


def clean_environment(inherited):
    names = {"RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTC", "RUSTC_WRAPPER",
             "RUSTC_WORKSPACE_WRAPPER", "RUSTC_BOOTSTRAP", "LLVM_PROFILE_FILE",
             "CARGO_INCREMENTAL", "CARGO_BUILD_INCREMENTAL",
             "CARGO_BUILD_TARGET", "CARGO_BUILD_RUSTFLAGS", "CARGO_BUILD_RUSTC",
             "CARGO_BUILD_RUSTC_WRAPPER", "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
             "CC", "CXX", "AR", "RANLIB", "CFLAGS", "CXXFLAGS", "CPPFLAGS", "LDFLAGS"}
    prefixes = ("CARGO_PROFILE_", "CARGO_TARGET_", "CC_", "CXX_", "AR_", "RANLIB_",
                "CFLAGS_", "CXXFLAGS_", "CPPFLAGS_", "LDFLAGS_", "HOST_", "TARGET_")
    return {key: value for key, value in inherited.items()
            if key not in names and not key.startswith(prefixes)}


def llvm_major(version):
    match = re.search(r"LLVM version:?\s+(\d+)", version, re.IGNORECASE)
    require(match is not None, "cannot read LLVM version: " + version)
    return int(match[1])


def load_manifest(path, assets):
    assets = assets.resolve()
    manifest = json.loads(path.read_text())
    require(manifest.get("schema") == "nuxie-runner-pgo-fixtures/v1", "unknown fixture manifest schema")
    require(manifest.get("training") == list(TRAINING), "training must be exactly the four named fixtures in order")
    fixtures = manifest["fixtures"]
    require(len(fixtures) == 25 and all(name in fixtures for name in TRAINING), "need four training and 21 held-out fixtures")
    files = {}
    for name, item in fixtures.items():
        require(re.fullmatch(r"[a-z0-9_]+", name) is not None, f"unsafe fixture id: {name}")
        file = (assets / item["file"]).resolve()
        require(file.is_relative_to(assets) and file.is_file(), f"fixture must be inside assets: {name}")
        require(sha(file) == item["sha256"], f"fixture hash mismatch: {name}")
        files[name] = file
    require(len(set(files.values())) == 25 and len({item["sha256"] for item in fixtures.values()}) == 25,
            "training and held-out fixtures must have distinct paths and contents")
    return manifest, files


def source_manifest(source):
    return {str(path.relative_to(source)): ("symlink:" + os.readlink(path) if path.is_symlink() else sha(path))
            for path in sorted(source.rglob("*")) if path.is_file() or path.is_symlink()}


def cargo_configs(source, env):
    cargo_home = Path(env.get("CARGO_HOME", str(Path.home() / ".cargo")))
    roots = [source, *source.parents]
    candidates = {root / ".cargo" / name for root in roots for name in ("config", "config.toml")}
    candidates.update(cargo_home / name for name in ("config", "config.toml"))
    result = {}
    for path in sorted(candidates):
        if path.is_file():
            contents = tomllib.loads(path.read_text())
            require(not ({"build", "target", "profile", "env"} & contents.keys()),
                    f"build-related Cargo config is unsupported by this matched experiment: {path}")
            result[str(path)] = sha(path)
    return result


def profile_diagnostics(log):
    missing, mismatches, promotions = [], [], []
    current_crate = None
    final_runner_invocation_line = None
    for line_number, line in enumerate(log.splitlines(), 1):
        invocation = re.search(r"Running .*--crate-name (\w+)", line)
        if invocation:
            current_crate = invocation[1]
            if current_crate == "rust_golden_runner" and final_runner_invocation_line is None:
                final_runner_invocation_line = line_number
        # Parallel Cargo jobs can interleave diagnostics from earlier invocations.
        # This field records log context, never the diagnostic's owning crate.
        row = {"line": line_number, "preceding_invocation_crate": current_crate, "message": line}
        if "no profile data available for function" in line:
            missing.append(row)
        elif line.startswith("warning:") and any(fragment in line.lower() for fragment in (
                "hash mismatch", "profile data may be out of date", "profile data out of date",
                "profile data mismatch", "control flow change")):
            mismatches.append(row)
        if "pgo-icall-prom (success): Promote indirect call" in line:
            promotions.append(row)
    return {"missing_profile_warnings": missing, "hash_mismatch_warnings": mismatches,
            "indirect_call_promotions": promotions,
            "indirect_call_promotion_count": len(promotions),
            "final_runner_invocation_line": final_runner_invocation_line,
            "promotions_before_final_runner_invocation": None if final_runner_invocation_line is None else sum(row["line"] < final_runner_invocation_line for row in promotions),
            "promotions_after_final_runner_invocation": None if final_runner_invocation_line is None else sum(row["line"] > final_runner_invocation_line for row in promotions),
            "attribution_note": "Parallel compiler output can interleave. Preceding invocation and before/after counts describe log position, not a proven originating crate or compilation phase."}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--revision", required=True, help="committed Git revision to archive; working edits are excluded")
    parser.add_argument("--assets", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True, help="new experiment directory")
    parser.add_argument("--target", required=True, help="native Rust host target; training runs locally")
    parser.add_argument("--cc", required=True)
    parser.add_argument("--cxx", required=True)
    parser.add_argument("--llvm-profdata", required=True)
    parser.add_argument("--jobs", type=int, default=2)
    parser.add_argument("--check-only", action="store_true", help="freeze and verify inputs without building")
    args = parser.parse_args(argv)
    require(args.jobs > 0, "jobs must be positive")
    repo, assets, manifest_path, output = (path.resolve() for path in (args.repo, args.assets, args.manifest, args.output))
    require(not output.exists(), "output must be a new directory; existing profiles cannot be reused")
    manifest, fixture_files = load_manifest(manifest_path, assets)
    env = clean_environment(os.environ)
    tools = {name: executable(value) for name, value in {
        "cargo": "cargo", "rustc": "rustc", "git": "git", "cc": args.cc,
        "cxx": args.cxx, "llvm_profdata": args.llvm_profdata}.items()}
    tools = pin_toolchain(tools, env)
    compiler_hashes = tool_hashes(tools)
    env.update(CC=tools["cc"], CXX=tools["cxx"], RUSTC=tools["rustc"],
               CARGO_BUILD_JOBS=str(args.jobs), CARGO_TARGET_DIR=str(output / "target"))
    versions = {name: subprocess.check_output([path, "-vV" if name == "rustc" else "--version"], env=env, text=True)
                for name, path in tools.items() if name != "git"}
    require(llvm_major(versions["rustc"]) == llvm_major(versions["llvm_profdata"]), "rustc and llvm-profdata LLVM major versions must match")
    require(f"host: {args.target}\n" in versions["rustc"], "only the compiler's native host target is supported")
    commit = subprocess.check_output([tools["git"], "rev-parse", args.revision + "^{commit}"], cwd=repo, text=True).strip()
    output.mkdir(parents=True)
    for name in ("source", "fixtures", "logs", "raw", "binaries", "recordings"):
        (output / name).mkdir()
    source = output / "source"
    commands = []

    def run(name, command, overrides=None, cwd=source):
        effective = env | (overrides or {})
        row = {"name": name, "argv": command, "cwd": str(cwd), "overrides": overrides or {},
               "started": datetime.datetime.now(datetime.timezone.utc).isoformat()}
        commands.append(row)
        write_json(output / "commands.json", commands)
        print(name, flush=True)
        with (output / "logs" / (name + ".stdout")).open("wb") as stdout, (output / "logs" / (name + ".stderr")).open("wb") as stderr:
            result = subprocess.run(command, cwd=cwd, env=effective, stdout=stdout, stderr=stderr)
        row["exit"] = result.returncode
        write_json(output / "commands.json", commands)
        require(result.returncode == 0, f"{name} failed; inspect {output / 'logs'}")

    archive = output / "source.tar"
    run("archive", [tools["git"], "archive", "--format=tar", "--output=" + str(archive), commit], cwd=repo)
    with tarfile.open(archive) as tar:
        tar.extractall(source, filter="data")
    expected_source = source_manifest(source)
    write_json(output / "source-manifest.json", expected_source)
    release = tomllib.loads((source / "Cargo.toml").read_text())["profile"]["release"]
    require(release.get("lto") == "fat" and release.get("codegen-units") == 1 and release.get("panic") == "unwind",
            "expected repository release profile: fat LTO, one codegen unit, unwind")
    configs = cargo_configs(source, env)
    for name, file in fixture_files.items():
        shutil.copyfile(file, output / "fixtures" / (name + ".riv"))
        require(sha(output / "fixtures" / (name + ".riv")) == manifest["fixtures"][name]["sha256"], "fixture changed while copying: " + name)
    shutil.copyfile(manifest_path, output / "fixture-manifest.json")
    provenance = {"source_commit": commit, "source_archive_sha256": sha(archive),
                  "source_manifest_sha256": sha(output / "source-manifest.json"),
                  "cargo_lock_sha256": sha(source / "Cargo.lock"), "fixture_manifest_sha256": sha(manifest_path),
                  "script_sha256": sha(__file__), "tools": tools, "tool_sha256": compiler_hashes, "versions": versions, "target": args.target,
                  "profile": release, "features": ["scripting", "runtime/tools through runner dependency"],
                  "cargo_configs": configs, "training": list(TRAINING), "training_frames": TRAIN_FRAMES,
                  "validation_frames": VALIDATE_FRAMES, "held_out": [name for name in fixture_files if name not in TRAINING],
                  "environment": {key: env.get(key) for key in ("CC", "CXX", "RUSTC", "CARGO_BUILD_JOBS", "CARGO_TARGET_DIR", "SDKROOT", "MACOSX_DEPLOYMENT_TARGET")},
                  "cleared_environment_keys": sorted(set(os.environ) - set(clean_environment(os.environ))),
                  "tool_version_commands": {name: [path, "-vV" if name == "rustc" else "--version"] for name, path in tools.items() if name != "git"},
                  "revision_command": [tools["git"], "rev-parse", args.revision + "^{commit}"],
                  "status": "inputs_verified", "binaries": {}}
    write_json(output / "provenance.json", provenance)
    if args.check_only:
        print("Inputs verified and frozen; no builds or training performed.")
        return 0

    def verify_source():
        require(tool_hashes(tools) == compiler_hashes, "compiler or LLVM tool executable changed during experiment")
        require(source_manifest(source) == expected_source, "frozen source changed during experiment")
        require(cargo_configs(source, env) == configs, "Cargo configuration changed during experiment")

    def build(lane, flags):
        verify_source()
        run(lane + "-build", [tools["cargo"], "build", "--release", "--target", args.target,
                              "-p", "rust-golden-runner", "--features", "scripting", "--locked", "--offline", "-vv"],
            {"CARGO_ENCODED_RUSTFLAGS": "\x1f".join(flags)})
        verify_source()
        binary = output / "binaries" / lane
        suffix = ".exe" if "windows" in args.target else ""
        shutil.copy2(output / "target" / args.target / "release" / ("rust-golden-runner" + suffix), binary)
        provenance["binaries"][lane] = {"sha256": sha(binary), "size": binary.stat().st_size, "flags": flags}
        write_json(output / "provenance.json", provenance)

    build("control", [])
    build("generate", ["-Cprofile-generate=" + str(output / "raw")])
    samples = ",".join(f"{i / 60:.9f}" for i in range(TRAIN_FRAMES))
    (output / "training-samples.txt").write_text(samples + "\n")
    for name in TRAINING:
        run("train-" + name, [str(output / "binaries/generate"), "--file", str(output / "fixtures" / (name + ".riv")),
                              "--samples", samples, "--benchmark", "--execute-scripts"],
            {"LLVM_PROFILE_FILE": str(output / "raw" / (name + "-%m-%p.profraw"))})
        require(f"segments={TRAIN_FRAMES}\n" in (output / "logs" / ("train-" + name + ".stdout")).read_text(), "wrong training segment count: " + name)
        require(any((output / "raw").glob(name + "-*.profraw")), "missing raw profile: " + name)
    raw = sorted((output / "raw").glob("*.profraw"))
    require(all(any(file.name.startswith(name + "-") for name in TRAINING) for file in raw),
            "raw profile from outside the four named training runs")
    provenance["raw_profiles"] = {file.name: sha(file) for file in raw}
    merged = output / "merged.profdata"
    run("merge", [tools["llvm_profdata"], "merge", "--failure-mode=any", "-o", str(merged), *map(str, raw)])
    run("profile-summary", [tools["llvm_profdata"], "show", "--all-functions", "--counts", "--ic-targets", "--detailed-summary", str(merged)])
    provenance["merged_profile_sha256"] = sha(merged)
    build("pgo", ["-Cprofile-use=" + str(merged), "-Cllvm-args=-pgo-warn-missing-function", "-Cremark=pgo-icall-prom"])
    diagnostics = profile_diagnostics((output / "logs/pgo-build.stderr").read_text())
    write_json(output / "profile-diagnostics.json", diagnostics)
    require(not diagnostics["hash_mismatch_warnings"], "profile hash mismatch; inspect profile-diagnostics.json")
    validation = []
    samples = ",".join(f"{i / 60:.9f}" for i in range(VALIDATE_FRAMES))
    (output / "validation-samples.txt").write_text(samples + "\n")
    for name, item in manifest["fixtures"].items():
        file = output / "fixtures" / (name + ".riv")
        require(sha(file) == item["sha256"], "frozen fixture changed: " + name)
        row = {"id": name, "held_out": name not in TRAINING}
        for lane in ("control", "pgo"):
            label = "record-" + name + "-" + lane
            run(label, [str(output / "binaries" / lane), "--file", str(file), "--samples", samples, "--execute-scripts", "--side-channel"])
            row[lane] = {"exit": commands[-1]["exit"]}
            for stream in ("stdout", "stderr"):
                destination = output / "recordings" / (name + "." + lane + "." + stream)
                shutil.move(output / "logs" / (label + "." + stream), destination)
                row[lane][stream + "_sha256"] = sha(destination)
        row["exact"] = row["control"] == row["pgo"]
        validation.append(row)
        write_json(output / "validation.json", validation)
    verify_source()
    require(all(row["exact"] for row in validation), "recording mismatch; inspect validation.json")
    provenance["status"] = "validated"
    write_json(output / "provenance.json", provenance)
    print("25 recordings exact (4 trained, 21 held out). No performance comparison performed.")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
