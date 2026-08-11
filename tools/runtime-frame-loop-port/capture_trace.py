#!/usr/bin/env python3
"""Capture canonical, mechanism, and unchanged-frame LLVM trace evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import re
import subprocess
import sys
import tomllib
from typing import Any


TOOL_DIR = pathlib.Path(__file__).resolve().parent
if str(TOOL_DIR) not in sys.path:
    sys.path.insert(0, str(TOOL_DIR))

from source_fingerprint import (
    SourceFingerprintError,
    candidate_source_fingerprint,
    require_rust_runner_provenance,
    rust_runner_provenance,
    rust_runner_provenance_path,
)


PINNED_CPP = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"
CANONICAL_IDS = (
    "advance_blend_mode",
    "ai_assitant",
    "align_target",
    "animated_clipping",
    "animation_reset_cases",
    "spotify_kids_demo",
)
MECHANISM_IDS = (
    "component_list_follow_path",
    "complex_ik_dependency",
    "component_list_virtualized",
    "scroll_test",
    "scroll_intent",
)
ALLOCATION_PATTERN = re.compile(r"^frame_loop_allocations=(\d+)$", re.MULTILINE)


def checked_output(command: list[str], *, cwd: pathlib.Path) -> str:
    return subprocess.run(
        command,
        cwd=cwd,
        text=True,
        capture_output=True,
        check=True,
    ).stdout.strip()


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def fixture_arguments(
    row: dict[str, Any], *, upstream: pathlib.Path
) -> list[str]:
    arguments = ["--file", str(upstream / str(row["path"]))]
    if row.get("artboard"):
        arguments.extend(["--artboard", str(row["artboard"])])
    if row.get("state_machine"):
        arguments.extend(["--state-machine", str(row["state_machine"])])
    samples = ",".join(format(float(value), ".9g") for value in row["samples"])
    arguments.extend(["--samples", samples, "--benchmark-repeat", "1"])
    if row.get("input_script"):
        arguments.extend(["--input-script", str(row["input_script"])])
    return arguments


def effective_fixture_row(
    row: dict[str, Any],
    *,
    frame_only: bool,
    occurrence_only: bool,
    steady_only: bool,
) -> tuple[dict[str, Any], bool]:
    effective = dict(row)
    mechanism_input = (
        frame_only
        and not occurrence_only
        and not steady_only
        and bool(effective.get("input_script"))
    )
    if occurrence_only or steady_only:
        effective["samples"] = [0.0]
        effective.pop("input_script", None)
    return effective, mechanism_input


def run_fixture(
    *,
    runner: pathlib.Path,
    side: str,
    row: dict[str, Any],
    upstream: pathlib.Path,
    profile: pathlib.Path,
    frame_only: bool,
    occurrence_only: bool,
    steady_only: bool,
    allocations: bool,
    benchmark: bool,
) -> tuple[str, int | None]:
    effective_row, mechanism_input = effective_fixture_row(
        row,
        frame_only=frame_only,
        occurrence_only=occurrence_only,
        steady_only=steady_only,
    )
    environment = os.environ.copy()
    environment["LLVM_PROFILE_FILE"] = str(profile)
    if side == "cpp":
        environment["RIVE_GOLDEN_COVERAGE_FLUSH"] = "1"
    if frame_only:
        environment["RIVE_GOLDEN_COVERAGE_FRAME_ONLY"] = "1"
    if occurrence_only:
        environment["RIVE_GOLDEN_COVERAGE_OCCURRENCE_ONLY"] = "1"
    if steady_only:
        environment["RIVE_GOLDEN_COVERAGE_STEADY_ONLY"] = "1"
    if mechanism_input:
        environment["RIVE_GOLDEN_COVERAGE_MECHANISM_INPUT"] = "1"
    if allocations:
        environment["RIVE_GOLDEN_ALLOCATION_COUNTER"] = "1"
    command = [
        str(runner),
        *fixture_arguments(effective_row, upstream=upstream),
    ]
    if benchmark:
        command.append("--benchmark")
    result = subprocess.run(
        command,
        text=True,
        capture_output=True,
        env=environment,
        check=True,
    )
    allocation_count = None
    if allocations:
        match = ALLOCATION_PATTERN.search(result.stderr)
        if match is None:
            raise RuntimeError(
                f"{side} {row['id']} did not report frame-loop allocations"
            )
        allocation_count = int(match.group(1))
    return result.stdout, allocation_count


def merge_and_export(
    *,
    profiles: list[pathlib.Path],
    runner: pathlib.Path,
    output_stem: pathlib.Path,
    llvm_profdata: pathlib.Path,
    llvm_cov: pathlib.Path,
) -> pathlib.Path:
    profdata = output_stem.with_suffix(".profdata")
    coverage = output_stem.with_suffix(".json")
    subprocess.run(
        [
            str(llvm_profdata),
            "merge",
            "-sparse",
            *(str(path) for path in profiles),
            "-o",
            str(profdata),
        ],
        check=True,
    )
    with coverage.open("w", encoding="utf-8") as output:
        subprocess.run(
            [
                str(llvm_cov),
                "export",
                str(runner),
                f"-instr-profile={profdata}",
            ],
            check=True,
            stdout=output,
        )
    return coverage


def normalize_materialized_cpp_coverage_paths(
    coverage: dict[str, Any], *, upstream: pathlib.Path
) -> None:
    """Map isolated patched-oracle source paths back to the pinned checkout."""

    def normalize(path: str) -> str:
        parts = pathlib.PurePath(path).parts
        materialized_index = next(
            (
                index
                for index, part in enumerate(parts)
                if part.startswith("patched-runtime-src.")
            ),
            None,
        )
        if materialized_index is None:
            return path
        relative = pathlib.Path(*parts[materialized_index + 1 :])
        return str(upstream / relative)

    def visit(value: Any) -> None:
        if isinstance(value, dict):
            for key, child in value.items():
                if key == "filename":
                    value[key] = normalize(str(child))
                elif key == "filenames":
                    value[key] = [normalize(str(path)) for path in child]
                else:
                    visit(child)
        elif isinstance(value, list):
            for child in value:
                visit(child)

    visit(coverage)


def normalize_materialized_cpp_coverage_file(
    coverage_path: pathlib.Path, *, upstream: pathlib.Path
) -> None:
    coverage = json.loads(coverage_path.read_text(encoding="utf-8"))
    normalize_materialized_cpp_coverage_paths(coverage, upstream=upstream)
    coverage_path.write_text(
        json.dumps(coverage, separators=(",", ":")), encoding="utf-8"
    )


def capture_group(
    *,
    group: str,
    rows: list[dict[str, Any]],
    cpp_runner: pathlib.Path,
    rust_runner: pathlib.Path,
    upstream: pathlib.Path,
    output_dir: pathlib.Path,
    frame_only: bool,
    occurrence_only: bool,
    steady_only: bool,
    allocations: bool,
    benchmark: bool,
    retain_streams: bool,
    llvm_profdata: pathlib.Path,
    llvm_cov: pathlib.Path,
) -> tuple[pathlib.Path, pathlib.Path, dict[str, dict[str, int]]]:
    profiles: dict[str, list[pathlib.Path]] = {"cpp": [], "rust": []}
    allocation_counts: dict[str, dict[str, int]] = {"cpp": {}, "rust": {}}
    stream_dir = output_dir / f"{group}-streams"
    if retain_streams:
        stream_dir.mkdir()
    for side, runner in (("cpp", cpp_runner), ("rust", rust_runner)):
        profile_dir = output_dir / f"{group}-{side}-profiles"
        profile_dir.mkdir()
        for row in rows:
            profile_pattern = profile_dir / f"{row['id']}-%p.profraw"
            stream, allocation_count = run_fixture(
                runner=runner,
                side=side,
                row=row,
                upstream=upstream,
                profile=profile_pattern,
                frame_only=frame_only,
                occurrence_only=occurrence_only,
                steady_only=steady_only,
                allocations=allocations,
                benchmark=benchmark,
            )
            if retain_streams:
                (stream_dir / f"{side}-{row['id']}.txt").write_text(
                    stream, encoding="utf-8"
                )
            if allocation_count is not None:
                allocation_counts[side][str(row["id"])] = allocation_count
        profiles[side] = sorted(profile_dir.glob("*.profraw"))
        if not profiles[side]:
            raise RuntimeError(f"{group} {side} produced no LLVM profiles")
    cpp_coverage = merge_and_export(
        profiles=profiles["cpp"],
        runner=cpp_runner,
        output_stem=output_dir / f"{group}-cpp",
        llvm_profdata=llvm_profdata,
        llvm_cov=llvm_cov,
    )
    normalize_materialized_cpp_coverage_file(
        cpp_coverage, upstream=upstream
    )
    rust_coverage = merge_and_export(
        profiles=profiles["rust"],
        runner=rust_runner,
        output_stem=output_dir / f"{group}-rust",
        llvm_profdata=llvm_profdata,
        llvm_cov=llvm_cov,
    )
    return cpp_coverage, rust_coverage, allocation_counts


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument("--upstream", type=pathlib.Path, required=True)
    parser.add_argument("--output-dir", type=pathlib.Path, required=True)
    parser.add_argument("--output", type=pathlib.Path, required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    repo_root = args.repo_root.resolve()
    upstream = args.upstream.resolve()
    output_dir = args.output_dir.resolve()
    if output_dir.exists() and any(output_dir.iterdir()):
        raise RuntimeError(f"trace output directory is not empty: {output_dir}")
    output_dir.mkdir(parents=True, exist_ok=True)

    actual_cpp = checked_output(["git", "rev-parse", "HEAD"], cwd=upstream)
    if actual_cpp != PINNED_CPP:
        raise RuntimeError(
            f"trace requires pinned C++ {PINNED_CPP}; got {actual_cpp}"
        )
    rust_ref = checked_output(["git", "rev-parse", "HEAD"], cwd=repo_root)
    rust_candidate_source = candidate_source_fingerprint(
        repo_root, evidence_path=args.output
    )
    cpp_runner = (
        repo_root
        / "tools/golden-runner/build/macosx/bin/debug/"
        "rive_golden_runner_coverage"
    )
    rust_runner = (
        repo_root / "target/frame-loop-coverage/debug/rust-golden-runner"
    )
    for runner in (cpp_runner, rust_runner):
        if not runner.is_file():
            raise RuntimeError(
                f"missing trace runner {runner}; run "
                "`make runtime-frame-loop-trace-runners` first"
            )
    try:
        rust_runner_provenance = require_rust_runner_provenance(
            rust_runner, rust_candidate_source
        )
    except SourceFingerprintError as error:
        raise RuntimeError(str(error)) from error
    rust_runner_sha256 = sha256(rust_runner)
    llvm_profdata = pathlib.Path(
        checked_output(["xcrun", "--find", "llvm-profdata"], cwd=repo_root)
    )
    llvm_cov = pathlib.Path(
        checked_output(["xcrun", "--find", "llvm-cov"], cwd=repo_root)
    )
    demangler = pathlib.Path(
        checked_output(["xcrun", "--find", "llvm-cxxfilt"], cwd=repo_root)
    )

    corpus = tomllib.loads((repo_root / "corpus.toml").read_text())
    rows_by_id = {str(row["id"]): row for row in corpus["file"]}
    canonical_rows = [rows_by_id[value] for value in CANONICAL_IDS]
    ledger = tomllib.loads(
        (repo_root / "docs/runtime-frame-loop-ownership.toml").read_text()
    )
    mechanism_specs = {
        str(row["id"]): row
        for row in ledger.get("trace_mechanism_fixture", [])
    }
    if set(mechanism_specs) != set(MECHANISM_IDS):
        raise RuntimeError(
            "trace mechanism fixture IDs do not match the capture contract"
        )
    mechanism_rows = []
    for value in MECHANISM_IDS:
        row = dict(rows_by_id[value])
        spec = mechanism_specs[value]
        if spec.get("samples"):
            row["samples"] = list(spec["samples"])
        if spec.get("input_script"):
            row["input_script"] = (
                repo_root / str(spec["input_script"])
            ).resolve()
        mechanism_rows.append(row)
    steady_ids = tuple(
        value
        for value in MECHANISM_IDS
        if mechanism_specs[value].get("steady", True)
    )
    steady_rows = [
        row for row in mechanism_rows if str(row["id"]) in steady_ids
    ]

    canonical_full_cpp, canonical_full_rust, _ = capture_group(
        group="canonical-full",
        rows=canonical_rows,
        cpp_runner=cpp_runner,
        rust_runner=rust_runner,
        upstream=upstream,
        output_dir=output_dir,
        frame_only=False,
        occurrence_only=True,
        steady_only=False,
        allocations=False,
        benchmark=False,
        retain_streams=False,
        llvm_profdata=llvm_profdata,
        llvm_cov=llvm_cov,
    )
    canonical_cpp, canonical_rust, canonical_allocations = capture_group(
        group="canonical-frame",
        rows=canonical_rows,
        cpp_runner=cpp_runner,
        rust_runner=rust_runner,
        upstream=upstream,
        output_dir=output_dir,
        frame_only=True,
        occurrence_only=False,
        steady_only=False,
        allocations=True,
        benchmark=False,
        retain_streams=True,
        llvm_profdata=llvm_profdata,
        llvm_cov=llvm_cov,
    )
    mechanism_full_cpp, mechanism_full_rust, _ = capture_group(
        group="mechanism-full",
        rows=mechanism_rows,
        cpp_runner=cpp_runner,
        rust_runner=rust_runner,
        upstream=upstream,
        output_dir=output_dir,
        frame_only=False,
        occurrence_only=True,
        steady_only=False,
        allocations=False,
        benchmark=False,
        retain_streams=False,
        llvm_profdata=llvm_profdata,
        llvm_cov=llvm_cov,
    )
    mechanism_cpp, mechanism_rust, mechanism_allocations = capture_group(
        group="mechanism-frame",
        rows=mechanism_rows,
        cpp_runner=cpp_runner,
        rust_runner=rust_runner,
        upstream=upstream,
        output_dir=output_dir,
        frame_only=True,
        occurrence_only=False,
        steady_only=False,
        allocations=True,
        benchmark=False,
        retain_streams=True,
        llvm_profdata=llvm_profdata,
        llvm_cov=llvm_cov,
    )
    steady_cpp, steady_rust, steady_allocations = capture_group(
        group="steady",
        rows=steady_rows,
        cpp_runner=cpp_runner,
        rust_runner=rust_runner,
        upstream=upstream,
        output_dir=output_dir,
        frame_only=True,
        occurrence_only=False,
        steady_only=True,
        allocations=True,
        benchmark=True,
        retain_streams=False,
        llvm_profdata=llvm_profdata,
        llvm_cov=llvm_cov,
    )

    allocation_paths: dict[str, pathlib.Path] = {}
    for name, counts in (
        ("canonical", canonical_allocations),
        ("mechanism", mechanism_allocations),
        ("steady", steady_allocations),
    ):
        path = output_dir / f"{name}-allocations.json"
        path.write_text(json.dumps(counts, indent=2, sort_keys=True) + "\n")
        allocation_paths[name] = path

    summarize_command = [
        sys.executable,
        str(repo_root / "tools/runtime-frame-loop-port/summarize_trace.py"),
        "--ledger",
        str(repo_root / "docs/runtime-frame-loop-ownership.toml"),
        "--upstream",
        str(upstream),
        "--cpp-coverage",
        str(canonical_cpp),
        "--rust-coverage",
        str(canonical_rust),
        "--cpp-full-coverage",
        str(canonical_full_cpp),
        "--rust-full-coverage",
        str(canonical_full_rust),
        "--cpp-mechanism-coverage",
        str(mechanism_cpp),
        "--rust-mechanism-coverage",
        str(mechanism_rust),
        "--cpp-mechanism-full-coverage",
        str(mechanism_full_cpp),
        "--rust-mechanism-full-coverage",
        str(mechanism_full_rust),
        "--cpp-steady-coverage",
        str(steady_cpp),
        "--rust-steady-coverage",
        str(steady_rust),
        "--cpp-binary",
        str(cpp_runner),
        "--rust-binary",
        str(rust_runner),
        "--stream-directory",
        str(output_dir / "canonical-frame-streams"),
        "--mechanism-stream-directory",
        str(output_dir / "mechanism-frame-streams"),
        "--allocation-counts",
        str(allocation_paths["canonical"]),
        "--mechanism-allocation-counts",
        str(allocation_paths["mechanism"]),
        "--steady-allocation-counts",
        str(allocation_paths["steady"]),
        "--demangler",
        str(demangler),
        "--rust-ref",
        rust_ref,
        "--output",
        str(args.output),
    ]
    for value in CANONICAL_IDS:
        summarize_command.extend(["--corpus-id", value])
    for value in MECHANISM_IDS:
        summarize_command.extend(["--mechanism-corpus-id", value])
    for value in steady_ids:
        summarize_command.extend(["--steady-corpus-id", value])
    subprocess.run(summarize_command, cwd=repo_root, check=True)

    final_rust_candidate_source = candidate_source_fingerprint(
        repo_root, evidence_path=args.output
    )
    if final_rust_candidate_source != rust_candidate_source:
        raise RuntimeError(
            "Rust candidate source changed during frame-loop trace capture"
        )
    try:
        final_rust_runner_provenance = require_rust_runner_provenance(
            rust_runner, final_rust_candidate_source
        )
    except SourceFingerprintError as error:
        raise RuntimeError(str(error)) from error
    if (
        final_rust_runner_provenance != rust_runner_provenance
        or sha256(rust_runner) != rust_runner_sha256
    ):
        raise RuntimeError("Rust trace runner changed during trace capture")
    trace = json.loads(args.output.read_text())
    trace["rust_candidate_source"] = rust_candidate_source
    trace["rust_runner_provenance"] = rust_runner_provenance
    trace["mechanism_fixture_sha256"] = {
        str(row["id"]): sha256(upstream / str(row["path"]))
        for row in mechanism_rows
    }
    trace["mechanism_input_sha256"] = {
        str(row["id"]): sha256(pathlib.Path(str(row["input_script"])))
        for row in mechanism_rows
        if row.get("input_script")
    }
    args.output.write_text(
        json.dumps(trace, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(f"runtime-frame-loop-trace: wrote {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
