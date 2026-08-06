#!/usr/bin/env python3
"""Guard and compare the pinned upstream/Rust microbenchmark mirror."""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import pathlib
import re
import shutil
import struct
import subprocess
import sys
import tarfile
import tomllib
from typing import NamedTuple


class ContractError(RuntimeError):
    pass


RUN_SCHEMA = "nuxie-upstream-microbench-run-v5"
CPP_BUILD_INPUTS_SCHEMA = "nuxie-upstream-microbench-cpp-build-inputs-v1"
FIXED_RUN_ARTIFACTS = {
    "inventory",
    "cpp_source_archive",
    "cpp_build_inputs",
    "cpp_binary",
    "cpp_build_log",
    "cpp_output",
}
RATIO_CASE_NAMES = {
    "BuildRawPath",
    "IntersectionBoardBench_marty",
    "IntersectionBoardBench_paper",
    "IntersectionTileBench",
    "IntersectionTileBenchWithOverlap",
    "IterateRawPath",
    "MapPointsAffine",
    "MapPointsScaleTrans",
    "MeasurePath",
    "RawPathBounds",
}
DIRECTIONAL_CASE_NAMES = {
    "DrawCustomFeathers",
    "DrawFeatheredPaths_paper",
    "DrawOneChopStrokes",
    "DrawOneCuspStrokes",
    "DrawRiveRenderPaths",
    "DrawRiveRenderPathsAsRoundJoinStrokes",
    "DrawRiveRenderPathsAsStrokes",
    "DrawTwoChopStrokes",
    "DrawTwoCuspStrokes",
    "DrawZeroChopStrokes",
}
CPP_PREMAKE_ARGS = "--with_rive_text --with_rive_layout --with_rive_canvas"
CPP_BUILD_ENVIRONMENT_KEYS = {
    "CC",
    "CXX",
    "GIT_CONFIG_GLOBAL",
    "GIT_CONFIG_NOSYSTEM",
    "GIT_TERMINAL_PROMPT",
    "HOME",
    "LANG",
    "LC_ALL",
    "PATH",
    "RIVE_BUILD_SYSTEM",
    "RIVE_CONFIG",
    "RIVE_OUT",
    "RIVE_PREMAKE_ARGS",
    "RIVE_PREMAKE_TAG",
    "TMPDIR",
}
CPP_BUILD_TOOL_NAMES = ("bash", "cc", "cxx", "curl", "git", "make", "python3", "unzip")


class Case(NamedTuple):
    name: str
    crate: str
    source: str
    source_sha256: str
    comparison: str
    equivalence: str


class Dataset(NamedTuple):
    name: str
    kind: str
    path: pathlib.Path
    source: str
    source_sha256: str
    sha256: str
    count: int
    width: int | None
    height: int | None


class Inventory(NamedTuple):
    schema: str
    upstream_ref: str
    draw_capability_source: str
    draw_capability_source_sha256: str
    cases: list[Case]
    datasets: list[Dataset]


class LoadedRun:
    """A validated manifest plus the exact artifact bytes validated with it."""

    __slots__ = ("manifest", "artifact_bytes")

    def __init__(self, manifest: dict, artifact_bytes: dict[str, bytes]) -> None:
        self.manifest = manifest
        self.artifact_bytes = artifact_bytes

    def __getitem__(self, key: str):
        return self.manifest[key]


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def content_identity(contents: bytes) -> dict[str, int | str]:
    return {"bytes": len(contents), "sha256": hashlib.sha256(contents).hexdigest()}


def directory_content_identity(path: pathlib.Path) -> dict[str, int | str]:
    """Hash a dependency tree without trusting filesystem metadata timestamps."""
    if not path.is_dir():
        raise ContractError(f"missing C++ dependency tree: {path}")
    digest = hashlib.sha256(b"nuxie-cpp-dependency-tree-v1\0")
    entries = 0
    for entry in sorted(path.rglob("*"), key=lambda item: item.relative_to(path).as_posix()):
        relative = entry.relative_to(path).as_posix().encode()
        if entry.is_symlink():
            kind = b"link"
            contents = os.readlink(entry).encode()
        elif entry.is_dir():
            kind = b"dir"
            contents = b""
        elif entry.is_file():
            kind = b"file"
            contents = entry.read_bytes()
        else:
            raise ContractError(f"unsupported C++ dependency entry: {entry}")
        digest.update(kind + b"\0" + relative + b"\0")
        digest.update(hashlib.sha256(contents).digest())
        digest.update(b"\0")
        entries += 1
    return {"entries": entries, "sha256": digest.hexdigest()}


def sanitized_build_path() -> str:
    candidates = (
        "/usr/bin",
        "/bin",
        "/usr/sbin",
        "/sbin",
        "/opt/homebrew/bin",
        "/usr/local/bin",
    )
    present = [candidate for candidate in candidates if pathlib.Path(candidate).is_dir()]
    if not present:
        raise ContractError("no fixed system tool directories exist for C++ build")
    return os.pathsep.join(present)


def resolve_build_tool(name: str, environment: dict[str, str]) -> pathlib.Path:
    command = {"bash": "/bin/bash", "cc": environment["CC"], "cxx": environment["CXX"]}.get(
        name, name
    )
    resolved = shutil.which(command, path=environment["PATH"])
    if resolved is None:
        raise ContractError(f"missing sanitized C++ build tool: {name}")
    path = pathlib.Path(resolved).resolve()
    if not path.is_file():
        raise ContractError(f"sanitized C++ build tool is not a file: {path}")
    return path


def cpp_build_environment(source_dir: pathlib.Path, run_dir: pathlib.Path) -> dict[str, str]:
    """Construct the C++ build environment from an allowlist, never ambient overrides."""
    fixed_path = sanitized_build_path()
    cc = shutil.which("cc", path=fixed_path)
    cxx = shutil.which("c++", path=fixed_path)
    if cc is None or cxx is None:
        raise ContractError("sanitized C++ compiler commands are unavailable")
    home = run_dir / "cpp-home"
    temporary = run_dir / "cpp-tmp"
    home.mkdir()
    temporary.mkdir()
    return {
        "CC": str(pathlib.Path(cc).resolve()),
        "CXX": str(pathlib.Path(cxx).resolve()),
        "GIT_CONFIG_GLOBAL": "/dev/null",
        "GIT_CONFIG_NOSYSTEM": "1",
        "GIT_TERMINAL_PROMPT": "0",
        "HOME": str(home.resolve()),
        "LANG": "C",
        "LC_ALL": "C",
        "PATH": fixed_path,
        "RIVE_BUILD_SYSTEM": "gmake2",
        "RIVE_CONFIG": "release",
        "RIVE_OUT": os.path.relpath(run_dir / "cpp-build", source_dir / "tests"),
        "RIVE_PREMAKE_ARGS": CPP_PREMAKE_ARGS,
        "RIVE_PREMAKE_TAG": "v5.0.0-beta7",
        "TMPDIR": str(temporary.resolve()),
    }


def write_cpp_build_inputs(
    source_dir: pathlib.Path,
    run_dir: pathlib.Path,
    environment: dict[str, str],
    command: list[str],
) -> pathlib.Path:
    dependency_roots = ("build/dependencies", "tests/dependencies")
    for relative in dependency_roots:
        (source_dir / relative).mkdir(parents=True, exist_ok=True)
    tools = {}
    for name in CPP_BUILD_TOOL_NAMES:
        path = resolve_build_tool(name, environment)
        tools[name] = {"path": str(path), **content_identity(path.read_bytes())}
    document = {
        "schema": CPP_BUILD_INPUTS_SCHEMA,
        "command": command,
        "environment": environment,
        "tools": tools,
        "dependency_trees": {
            relative: directory_content_identity(source_dir / relative)
            for relative in dependency_roots
        },
    }
    output = run_dir / "cpp-build-inputs.json"
    output.write_text(json.dumps(document, indent=2, sort_keys=True) + "\n")
    return output


def validate_cpp_build_inputs(contents: bytes, run_manifest: pathlib.Path) -> None:
    try:
        document = json.loads(contents)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ContractError("invalid sealed C++ build inputs") from error
    if not isinstance(document, dict) or set(document) != {
        "schema",
        "command",
        "environment",
        "tools",
        "dependency_trees",
    }:
        raise ContractError("invalid sealed C++ build input fields")
    if document["schema"] != CPP_BUILD_INPUTS_SCHEMA:
        raise ContractError("unsupported sealed C++ build input schema")
    environment = document["environment"]
    if not isinstance(environment, dict) or set(environment) != CPP_BUILD_ENVIRONMENT_KEYS:
        raise ContractError("sealed C++ build environment differs from the allowlist")
    run_dir = run_manifest.parent.resolve()
    source_dir = run_dir / "cpp-source"
    if document["command"] != [
        str((source_dir / "build" / "build_rive.sh").resolve()),
        "release",
        "--",
        "bench",
    ]:
        raise ContractError("sealed C++ build command differs")
    expected_environment = {
        "GIT_CONFIG_GLOBAL": "/dev/null",
        "GIT_CONFIG_NOSYSTEM": "1",
        "GIT_TERMINAL_PROMPT": "0",
        "HOME": str((run_dir / "cpp-home").resolve()),
        "LANG": "C",
        "LC_ALL": "C",
        "PATH": sanitized_build_path(),
        "RIVE_BUILD_SYSTEM": "gmake2",
        "RIVE_CONFIG": "release",
        "RIVE_OUT": os.path.relpath(run_dir / "cpp-build", source_dir / "tests"),
        "RIVE_PREMAKE_ARGS": CPP_PREMAKE_ARGS,
        "RIVE_PREMAKE_TAG": "v5.0.0-beta7",
        "TMPDIR": str((run_dir / "cpp-tmp").resolve()),
    }
    for name, expected in expected_environment.items():
        if environment.get(name) != expected:
            raise ContractError(f"sealed C++ build environment mismatch for {name}")
    tools = document["tools"]
    if not isinstance(tools, dict) or set(tools) != set(CPP_BUILD_TOOL_NAMES):
        raise ContractError("sealed C++ build tool set differs")
    for compiler in ("cc", "cxx"):
        if environment.get(compiler.upper()) != tools[compiler].get("path"):
            raise ContractError(f"sealed C++ compiler path mismatch for {compiler}")
    for name, identity in tools.items():
        if not isinstance(identity, dict) or set(identity) != {"path", "bytes", "sha256"}:
            raise ContractError(f"invalid sealed C++ build tool identity for {name}")
        if (
            not isinstance(identity["path"], str)
            or not isinstance(identity["bytes"], int)
            or identity["bytes"] <= 0
            or not isinstance(identity["sha256"], str)
            or not re.fullmatch(r"[0-9a-f]{64}", identity["sha256"])
        ):
            raise ContractError(f"invalid sealed C++ build tool identity for {name}")
        current_path = resolve_build_tool(name, environment)
        if identity["path"] != str(current_path) or {
            "bytes": identity["bytes"],
            "sha256": identity["sha256"],
        } != content_identity(current_path.read_bytes()):
            raise ContractError(f"sealed C++ build tool changed: {name}")
    dependencies = document["dependency_trees"]
    expected_dependencies = {"build/dependencies", "tests/dependencies"}
    if not isinstance(dependencies, dict) or set(dependencies) != expected_dependencies:
        raise ContractError("sealed C++ dependency tree set differs")
    for relative, expected in dependencies.items():
        if expected != directory_content_identity(source_dir / relative):
            raise ContractError(f"sealed C++ dependency tree changed: {relative}")


def benchmark_content_identity(repo_root: pathlib.Path, revision: str = "HEAD") -> str:
    """Hash committed repository content while excluding generated evidence docs."""
    tree = subprocess.run(
        ["git", "ls-tree", "-r", "-z", "--full-tree", revision],
        cwd=repo_root,
        check=True,
        capture_output=True,
    ).stdout
    digest = hashlib.sha256(b"nuxie-upstream-microbench-content-v1\0")
    for entry in tree.split(b"\0"):
        if not entry:
            continue
        _, path = entry.split(b"\t", 1)
        if path.startswith(b"docs/evidence/"):
            continue
        digest.update(entry)
        digest.update(b"\0")
    return digest.hexdigest()


def uncommitted_benchmark_paths(repo_root: pathlib.Path) -> list[str]:
    status = subprocess.run(
        ["git", "status", "--porcelain=v1", "-z", "--untracked-files=all"],
        cwd=repo_root,
        check=True,
        capture_output=True,
    ).stdout.split(b"\0")
    changed: list[str] = []
    index = 0
    while index < len(status):
        entry = status[index]
        index += 1
        if not entry:
            continue
        code = entry[:2]
        paths = [entry[3:].decode()]
        if b"R" in code or b"C" in code:
            paths.append(status[index].decode())
            index += 1
        changed.extend(path for path in paths if not path.startswith("docs/evidence/"))
    return changed


def load_inventory(path: pathlib.Path) -> Inventory:
    with path.open("rb") as source:
        raw = tomllib.load(source)
    if raw.get("schema") != "nuxie-upstream-microbench-v1":
        raise ContractError(f"unsupported microbenchmark schema in {path}")
    cases = [Case(**case) for case in raw.get("case", [])]
    datasets = [
        Dataset(
            name=dataset["name"],
            kind=dataset["kind"],
            path=pathlib.Path(dataset["path"]),
            source=dataset["source"],
            source_sha256=dataset["source_sha256"],
            sha256=dataset["sha256"],
            count=dataset["count"],
            width=dataset.get("width"),
            height=dataset.get("height"),
        )
        for dataset in raw.get("dataset", [])
    ]
    return Inventory(
        raw["schema"],
        raw["upstream_ref"],
        raw["draw_capability_source"],
        raw["draw_capability_source_sha256"],
        cases,
        datasets,
    )


def check_dataset(repo_root: pathlib.Path, dataset: Dataset) -> None:
    path = repo_root / dataset.path
    if not path.is_file():
        raise ContractError(f"missing microbenchmark dataset: {dataset.path}")
    actual = sha256(path)
    if actual != dataset.sha256:
        raise ContractError(
            f"{dataset.name} sha256 mismatch: expected {dataset.sha256}, got {actual}"
        )
    expected_size = dataset.count * (16 if dataset.kind == "i32-ltrb" else 1)
    if path.stat().st_size != expected_size:
        raise ContractError(
            f"{dataset.name} size mismatch: expected {expected_size}, got {path.stat().st_size}"
        )


def check_datasets(repo_root: pathlib.Path, inventory: Inventory) -> None:
    for dataset in inventory.datasets:
        check_dataset(repo_root, dataset)


def check_bench_sources(repo_root: pathlib.Path, inventory: Inventory) -> None:
    check_case_comparison_contract(inventory)
    expected = {case.name for case in inventory.cases}
    if len(expected) != 20 or len(inventory.cases) != 20:
        raise ContractError("microbenchmark inventory must contain 20 unique cases")
    registered: set[str] = set()
    for crate in {case.crate for case in inventory.cases}:
        source = repo_root / "crates" / crate / "benches" / "upstream_microbenchmarks.rs"
        if not source.is_file():
            raise ContractError(f"missing criterion target: {source.relative_to(repo_root)}")
        text = source.read_text()
        registered.update(re.findall(r'bench_function\(\s*"([^"]+)"', text))
    if registered != expected:
        missing = sorted(expected - registered)
        extra = sorted(registered - expected)
        raise ContractError(f"criterion registry mismatch: missing={missing}, extra={extra}")
    comparisons = {case.comparison for case in inventory.cases}
    if not comparisons <= {"ratio", "directional", "blocked"}:
        raise ContractError(f"unsupported comparison classifications: {sorted(comparisons)}")


def check_case_comparison_contract(inventory: Inventory) -> None:
    ratio = {case.name for case in inventory.cases if case.comparison == "ratio"}
    if ratio != RATIO_CASE_NAMES:
        raise ContractError(
            "ratio case names differ: "
            f"missing={sorted(RATIO_CASE_NAMES - ratio)}, "
            f"extra={sorted(ratio - RATIO_CASE_NAMES)}"
        )
    directional = {
        case.name for case in inventory.cases if case.comparison == "directional"
    }
    if directional != DIRECTIONAL_CASE_NAMES:
        raise ContractError(
            "directional case names differ: "
            f"missing={sorted(DIRECTIONAL_CASE_NAMES - directional)}, "
            f"extra={sorted(directional - DIRECTIONAL_CASE_NAMES)}"
        )


def check_runnable_inventory(inventory: Inventory) -> None:
    unknown = [
        case.name
        for case in inventory.cases
        if case.comparison not in {"ratio", "directional", "blocked"}
    ]
    if unknown:
        raise ContractError(f"unsupported comparison classification for {unknown}")
    blocked = [case.name for case in inventory.cases if case.comparison == "blocked"]
    if blocked:
        raise ContractError(
            "benchmark evidence requires runnable ratio or directional boundaries; "
            f"blocked={blocked}"
        )


def discover_upstream_cases(upstream: pathlib.Path, inventory: Inventory) -> set[str]:
    registered: set[str] = set()
    for source in {case.source for case in inventory.cases}:
        path = upstream / source
        if not path.is_file():
            raise ContractError(f"missing pinned upstream benchmark source: {source}")
        registered.update(re.findall(r"REGISTER_BENCH\(\s*([A-Za-z0-9_]+)\s*\)", path.read_text()))
    return registered


def check_upstream_case_contract(upstream: pathlib.Path, inventory: Inventory) -> None:
    expected = {case.name for case in inventory.cases}
    registered = discover_upstream_cases(upstream, inventory)
    if registered != expected:
        raise ContractError(
            "upstream registry mismatch: "
            f"missing={sorted(expected - registered)}, extra={sorted(registered - expected)}"
        )
    for case in inventory.cases:
        actual = sha256(upstream / case.source)
        if actual != case.source_sha256:
            raise ContractError(
                f"{case.source} sha256 mismatch: expected {case.source_sha256}, got {actual}"
            )
    capability_source = upstream / inventory.draw_capability_source
    if not capability_source.is_file():
        raise ContractError(
            f"missing pinned upstream draw capability source: "
            f"{inventory.draw_capability_source}"
        )
    actual = sha256(capability_source)
    if actual != inventory.draw_capability_source_sha256:
        raise ContractError(
            f"{inventory.draw_capability_source} sha256 mismatch: "
            f"expected {inventory.draw_capability_source_sha256}, got {actual}"
        )
    if "m_platformFeatures.supportsRasterOrderingMode = true;" not in (
        capability_source.read_text()
    ):
        raise ContractError(
            "pinned upstream RenderContextNULL must enable RasterOrdering"
        )


def parse_bbox_header(path: pathlib.Path, expected_count: int) -> bytes:
    text = path.read_text()
    rows = re.findall(
        r"\{\s*(-?\d+)\s*,\s*(-?\d+)\s*,\s*(-?\d+)\s*,\s*(-?\d+)\s*\}",
        text,
    )
    if len(rows) != expected_count:
        raise ContractError(f"{path} contains {len(rows)} boxes, expected {expected_count}")
    return b"".join(struct.pack("<4i", *(int(value) for value in row)) for row in rows)


def parse_cpp_byte_array(path: pathlib.Path, expected_count: int) -> bytes:
    text = path.read_text()
    try:
        body = text.split("paper_riv_data[paper_riv_data_len] = {", 1)[1].rsplit("};", 1)[0]
    except IndexError as error:
        raise ContractError(f"could not find paper_riv_data in {path}") from error
    result = bytes(int(value, 16) for value in re.findall(r"0x([0-9a-fA-F]{2})", body))
    if len(result) != expected_count:
        raise ContractError(f"{path} contains {len(result)} bytes, expected {expected_count}")
    return result


def check_upstream_ref(upstream: pathlib.Path, inventory: Inventory) -> None:
    actual_ref = subprocess.run(
        ["git", "-C", str(upstream), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if actual_ref != inventory.upstream_ref:
        raise ContractError(
            f"upstream ref mismatch: expected {inventory.upstream_ref}, got {actual_ref}"
        )


def converted_dataset_content(source: pathlib.Path, dataset: Dataset) -> bytes:
    actual_source_hash = sha256(source)
    if actual_source_hash != dataset.source_sha256:
        raise ContractError(
            f"{dataset.source} sha256 mismatch: expected {dataset.source_sha256}, "
            f"got {actual_source_hash}"
        )
    if dataset.kind == "i32-ltrb":
        return parse_bbox_header(source, dataset.count)
    if dataset.kind == "rive-bytes":
        return parse_cpp_byte_array(source, dataset.count)
    raise ContractError(f"unsupported dataset kind: {dataset.kind}")


def check_upstream_datasets(
    repo_root: pathlib.Path, upstream: pathlib.Path, inventory: Inventory
) -> None:
    check_upstream_ref(upstream, inventory)
    check_upstream_case_contract(upstream, inventory)
    for dataset in inventory.datasets:
        source = upstream / dataset.source
        content = converted_dataset_content(source, dataset)
        checked_in = (repo_root / dataset.path).read_bytes()
        if content != checked_in:
            raise ContractError(
                f"{dataset.name} differs from deterministic conversion of {dataset.source}"
            )


def extract_datasets(repo_root: pathlib.Path, upstream: pathlib.Path, inventory: Inventory) -> None:
    check_upstream_ref(upstream, inventory)
    for dataset in inventory.datasets:
        content = converted_dataset_content(upstream / dataset.source, dataset)
        target = repo_root / dataset.path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(content)
        print(f"wrote {dataset.path} sha256={hashlib.sha256(content).hexdigest()}")


def parse_cpp_timings(contents: bytes) -> dict[str, float]:
    timings: dict[str, float] = {}
    for line in contents.decode().splitlines():
        match = re.fullmatch(r"\s*([0-9.eE+-]+)ms\s+(\S+)\s*", line)
        if match:
            timings[match.group(2)] = float(match.group(1)) * 1_000_000.0
    return timings


def load_cpp_timings(path: pathlib.Path) -> dict[str, float]:
    return parse_cpp_timings(path.read_bytes())


def load_sealed_cpp_timings(run: LoadedRun) -> dict[str, float]:
    return parse_cpp_timings(run.artifact_bytes["cpp_output"])


def parse_criterion_minimum(contents: bytes, description: str) -> float:
    raw = json.loads(contents)
    per_iteration = [float(elapsed) / float(iters) for iters, elapsed in zip(raw["iters"], raw["times"])]
    if not per_iteration:
        raise ContractError(f"empty criterion sample: {description}")
    return min(per_iteration)


def load_criterion_minimum(sample_path: pathlib.Path) -> float:
    return parse_criterion_minimum(sample_path.read_bytes(), str(sample_path))


def load_sealed_criterion_timings(
    run: LoadedRun, inventory: Inventory
) -> dict[str, float]:
    return {
        case.name: parse_criterion_minimum(
            run.artifact_bytes[f"criterion:{case.name}"],
            f"sealed criterion:{case.name}",
        )
        for case in inventory.cases
    }


def render_report(
    inventory: Inventory, cpp_nanoseconds: dict[str, float], rust_nanoseconds: dict[str, float]
) -> str:
    lines = [
        "## Equivalent boundaries (minimum sample versus minimum sample)",
        "",
        "| Benchmark | C++ | Rust | Rust/C++ |",
        "|---|---:|---:|---:|",
    ]
    for case in inventory.cases:
        if case.comparison != "ratio":
            continue
        if case.name not in cpp_nanoseconds or case.name not in rust_nanoseconds:
            raise ContractError(f"missing timing for {case.name}")
        cpp = cpp_nanoseconds[case.name]
        rust = rust_nanoseconds[case.name]
        lines.append(
            f"| `{case.name}` | {cpp / 1_000_000:.6f} ms | "
            f"{rust / 1_000_000:.6f} ms | {rust / cpp:.3f}x |"
        )
    directional = [case for case in inventory.cases if case.comparison == "directional"]
    if directional:
        lines.extend([
            "",
            "## Directional timings (not ratio-comparable)",
            "",
            "| Benchmark | C++ workload | Rust primitive | Why no ratio |",
            "|---|---:|---:|---|",
        ])
        for case in directional:
            cpp = cpp_nanoseconds[case.name]
            rust = rust_nanoseconds[case.name]
            lines.append(
                f"| `{case.name}` | {cpp / 1_000_000:.6f} ms | "
                f"{rust / 1_000_000:.6f} ms | {case.equivalence} |"
            )
    blocked = [case for case in inventory.cases if case.comparison == "blocked"]
    if blocked:
        lines.extend([
            "",
            "## Blocked equivalence",
            "",
            "| Benchmark | Missing production seam |",
            "|---|---|",
        ])
        for case in blocked:
            lines.append(f"| `{case.name}` | {case.equivalence} |")
    return "\n".join(lines) + "\n"


def expected_run_artifact_keys(inventory: Inventory) -> set[str]:
    return FIXED_RUN_ARTIFACTS | {
        f"criterion:{case.name}" for case in inventory.cases
    }


def validate_run_artifact_paths(
    run: dict,
    inventory: Inventory,
    run_manifest: pathlib.Path,
    manifest_path: pathlib.Path,
) -> None:
    artifacts = run["artifacts"]
    run_dir = run_manifest.parent.resolve()
    expected_fixed_paths = {
        "inventory": manifest_path.resolve(),
        "cpp_source_archive": run_dir / "cpp-source.tar",
        "cpp_build_inputs": run_dir / "cpp-build-inputs.json",
        "cpp_binary": run_dir / "cpp-build" / "bench",
        "cpp_build_log": run_dir / "cpp-build.log",
        "cpp_output": run_dir / "cpp.txt",
    }
    for name, expected in expected_fixed_paths.items():
        actual = pathlib.Path(artifacts[name]["path"]).resolve()
        if actual != expected:
            raise ContractError(
                f"run artifact path mismatch for {name}: expected {expected}, got {actual}"
            )

    run_id = run.get("run_id")
    if not isinstance(run_id, str) or not run_id:
        raise ContractError("benchmark run has no run ID")
    criterion_root: pathlib.Path | None = None
    for case in inventory.cases:
        name = f"criterion:{case.name}"
        sample = pathlib.Path(artifacts[name]["path"]).resolve()
        try:
            case_root = sample.parents[2]
        except IndexError as error:
            raise ContractError(
                f"Criterion artifact path for {case.name} is invalid: {sample}"
            ) from error
        expected = case_root / case.name / "new" / "sample.json"
        if case_root.name != run_id or sample != expected:
            raise ContractError(
                f"Criterion artifact path for {case.name} is outside run {run_id}: {sample}"
            )
        if criterion_root is None:
            criterion_root = case_root
        elif case_root != criterion_root:
            raise ContractError(
                f"Criterion artifact path for {case.name} mixes run namespaces: {sample}"
            )


def validate_run_artifacts(
    run: dict,
    inventory: Inventory,
    run_manifest: pathlib.Path,
    manifest_path: pathlib.Path,
) -> dict[str, bytes]:
    if run.get("schema") != RUN_SCHEMA:
        raise ContractError(
            f"unsupported benchmark run schema: expected {RUN_SCHEMA}, got {run.get('schema')}"
        )
    if run.get("status") != "complete":
        raise ContractError("benchmark run is not complete")
    artifacts = run.get("artifacts")
    if not isinstance(artifacts, dict):
        raise ContractError("benchmark run has no artifact map")
    expected = expected_run_artifact_keys(inventory)
    actual = set(artifacts)
    if actual != expected:
        raise ContractError(
            "artifact set mismatch: "
            f"missing={sorted(expected - actual)}, extra={sorted(actual - expected)}"
        )
    for name, artifact in artifacts.items():
        if not isinstance(artifact, dict) or set(artifact) != {"path", "sha256"}:
            raise ContractError(f"invalid run artifact entry for {name}")
        if (
            not isinstance(artifact["path"], str)
            or not isinstance(artifact["sha256"], str)
            or not re.fullmatch(r"[0-9a-f]{64}", artifact["sha256"])
        ):
            raise ContractError(f"invalid run artifact entry for {name}")
    validate_run_artifact_paths(run, inventory, run_manifest, manifest_path)
    validated: dict[str, bytes] = {}
    for name, artifact in artifacts.items():
        path = pathlib.Path(artifact["path"])
        if not path.is_file():
            raise ContractError(f"missing run artifact {name}: {path}")
        contents = path.read_bytes()
        actual = hashlib.sha256(contents).hexdigest()
        if actual != artifact["sha256"]:
            raise ContractError(
                f"artifact hash mismatch for {name}: expected {artifact['sha256']}, got {actual}"
            )
        validated[name] = contents
    return validated


def command_output(command: list[str], cwd: pathlib.Path | None = None) -> str:
    return subprocess.run(
        command, cwd=cwd, check=True, capture_output=True, text=True
    ).stdout.strip()


def record_artifact(path: pathlib.Path) -> dict[str, str]:
    return {"path": str(path.resolve()), "sha256": sha256(path)}


def build_cpp_benchmark(
    upstream: pathlib.Path, run_dir: pathlib.Path
) -> tuple[pathlib.Path, pathlib.Path, list[str], pathlib.Path]:
    """Build the pinned C++ benchmark into the sealed run namespace."""
    build_dir = run_dir / "cpp-build"
    if build_dir.exists():
        raise ContractError(f"C++ build directory must be absent: {build_dir}")
    log = run_dir / "cpp-build.log"
    command = [str((upstream / "build" / "build_rive.sh").resolve()), "release", "--", "bench"]
    environment = cpp_build_environment(upstream, run_dir)
    result = subprocess.run(
        command,
        cwd=upstream / "tests",
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    log.write_text(result.stdout)
    if result.returncode != 0:
        raise ContractError(
            f"pinned C++ benchmark build failed with status {result.returncode}; see {log}"
        )
    binary = build_dir / "bench"
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise ContractError(f"pinned C++ build did not produce executable {binary}")
    inputs = write_cpp_build_inputs(upstream, run_dir, environment, command)
    return binary, log, command, inputs


def stage_upstream_source(
    upstream: pathlib.Path, revision: str, run_dir: pathlib.Path
) -> tuple[pathlib.Path, pathlib.Path]:
    """Materialize only committed pinned source inside the sealed run directory."""
    archive = run_dir / "cpp-source.tar"
    archived = subprocess.run(
        ["git", "archive", "--format=tar", revision],
        cwd=upstream,
        check=True,
        capture_output=True,
    ).stdout
    archive.write_bytes(archived)
    source_dir = run_dir / "cpp-source"
    source_dir.mkdir()
    with tarfile.open(archive) as source:
        source.extractall(source_dir, filter="data")
    return source_dir, archive


def run_benchmarks(
    repo_root: pathlib.Path,
    manifest_path: pathlib.Path,
    inventory: Inventory,
    upstream: pathlib.Path,
    run_dir: pathlib.Path,
    duration: int,
    warm_up: int,
    measurement: int,
    sample_size: int,
) -> pathlib.Path:
    check_case_comparison_contract(inventory)
    check_runnable_inventory(inventory)
    if run_dir.exists() and any(run_dir.iterdir()):
        raise ContractError(f"run directory must be absent or empty: {run_dir}")
    if command_output(["git", "status", "--porcelain"], repo_root):
        raise ContractError("benchmark evidence requires a clean committed worktree")
    check_upstream_ref(upstream, inventory)
    check_upstream_case_contract(upstream, inventory)
    check_upstream_datasets(repo_root, upstream, inventory)
    upstream_status = command_output(
        ["git", "status", "--porcelain", "--untracked-files=all"], upstream
    )
    if upstream_status:
        raise ContractError("pinned upstream checkout must be clean before the sealed build")

    revision = command_output(["git", "rev-parse", "HEAD"], repo_root)
    timestamp = datetime.datetime.now(datetime.UTC).strftime("%Y%m%dT%H%M%SZ")
    run_id = f"{timestamp}-{revision[:12]}"
    run_dir.mkdir(parents=True, exist_ok=True)
    criterion_base = pathlib.Path(os.environ.get("CRITERION_HOME", run_dir / "criterion-root"))
    criterion_dir = (criterion_base / "nuxie-upstream-microbenchmarks" / run_id).resolve()
    if criterion_dir.exists():
        raise ContractError(f"Criterion run directory already exists: {criterion_dir}")
    criterion_dir.mkdir(parents=True)
    cargo_target_dir = pathlib.Path(
        os.environ.get("CARGO_TARGET_DIR", repo_root / "target")
    ).resolve()
    cpp_output = run_dir / "cpp.txt"
    run_manifest = run_dir / "run.json"
    cpp_source, cpp_source_archive = stage_upstream_source(
        upstream, inventory.upstream_ref, run_dir
    )
    cpp_bench, cpp_build_log, cpp_build_command, cpp_build_inputs = build_cpp_benchmark(
        cpp_source, run_dir
    )
    check_upstream_datasets(repo_root, upstream, inventory)
    if command_output(
        ["git", "status", "--porcelain", "--untracked-files=all"], upstream
    ):
        raise ContractError("pinned upstream checkout changed during the sealed build")
    run: dict = {
        "schema": RUN_SCHEMA,
        "status": "running",
        "run_id": run_id,
        "repo_revision": revision,
        "benchmark_content_sha256": benchmark_content_identity(repo_root),
        "upstream_revision": inventory.upstream_ref,
        "settings": {
            "cpp_duration_seconds": duration,
            "criterion_warm_up_seconds": warm_up,
            "criterion_measurement_seconds": measurement,
            "criterion_sample_size": sample_size,
            "statistic": "minimum individually timed invocation",
            "criterion_home": str(criterion_dir),
            "cargo_target_dir": str(cargo_target_dir),
            "cpp_build_cwd": str((cpp_source / "tests").resolve()),
            "cpp_build_output_dir": str((run_dir / "cpp-build").resolve()),
            "cpp_build_command": cpp_build_command,
        },
        "tools": {
            "rustc": command_output(["rustc", "--version"]),
            "cargo": command_output(["cargo", "--version"]),
            "cxx": command_output(["c++", "--version"]),
            "platform": command_output(["uname", "-a"]),
        },
        "artifacts": {
            "inventory": record_artifact(manifest_path),
            "cpp_source_archive": record_artifact(cpp_source_archive),
            "cpp_build_inputs": record_artifact(cpp_build_inputs),
            "cpp_binary": record_artifact(cpp_bench),
            "cpp_build_log": record_artifact(cpp_build_log),
        },
    }
    run_manifest.write_text(json.dumps(run, indent=2) + "\n")
    environment = os.environ.copy()
    environment["CRITERION_HOME"] = str(criterion_dir)
    environment["CARGO_TARGET_DIR"] = str(cargo_target_dir)
    criterion_args = [
        "--",
        "--warm-up-time",
        str(warm_up),
        "--measurement-time",
        str(measurement),
        "--sample-size",
        str(sample_size),
    ]
    for package in ("nuxie-runtime", "nuxie-renderer"):
        subprocess.run(
            [
                "cargo",
                "bench",
                "-p",
                package,
                "--features",
                "upstream-microbenchmarks",
                "--bench",
                "upstream_microbenchmarks",
                *criterion_args,
            ],
            cwd=repo_root,
            env=environment,
            check=True,
        )
    run_cpp(cpp_bench, inventory, duration, cpp_output)
    run["artifacts"]["cpp_output"] = record_artifact(cpp_output)
    for case in inventory.cases:
        sample = criterion_dir / case.name / "new" / "sample.json"
        if not sample.is_file():
            raise ContractError(f"missing run-scoped Criterion sample for {case.name}: {sample}")
        run["artifacts"][f"criterion:{case.name}"] = record_artifact(sample)
    run["status"] = "complete"
    run_manifest.write_text(json.dumps(run, indent=2) + "\n")
    return run_manifest


def load_run(
    repo_root: pathlib.Path,
    manifest_path: pathlib.Path,
    run_manifest: pathlib.Path,
    inventory: Inventory,
) -> LoadedRun:
    run = json.loads(run_manifest.read_text())
    artifact_bytes = validate_run_artifacts(run, inventory, run_manifest, manifest_path)
    validate_cpp_build_inputs(artifact_bytes["cpp_build_inputs"], run_manifest)
    dirty = uncommitted_benchmark_paths(repo_root)
    if dirty:
        raise ContractError(f"uncommitted benchmark content: {dirty}")
    measured_identity = run.get("benchmark_content_sha256")
    if not measured_identity:
        raise ContractError("run manifest has no benchmark content identity")
    current_identity = benchmark_content_identity(repo_root)
    if measured_identity != current_identity:
        raise ContractError(
            "stale benchmark content: "
            f"expected measured {measured_identity}, got current {current_identity}"
        )
    if run["artifacts"]["inventory"]["sha256"] != hashlib.sha256(
        artifact_bytes["inventory"]
    ).hexdigest():
        raise ContractError("stale run inventory hash")
    return LoadedRun(run, artifact_bytes)


def run_cpp(cpp_bench: pathlib.Path, inventory: Inventory, duration: int, output: pathlib.Path) -> None:
    lines = []
    for case in inventory.cases:
        result = subprocess.run(
            [str(cpp_bench), "--duration", str(duration), case.name],
            check=True,
            capture_output=True,
            text=True,
        )
        lines.append(result.stdout.strip())
        print(lines[-1], flush=True)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text("\n".join(lines) + "\n")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=pathlib.Path, default=pathlib.Path.cwd())
    parser.add_argument("--manifest", type=pathlib.Path, default=pathlib.Path("microbenchmarks.toml"))
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("check")
    check_upstream = commands.add_parser("check-upstream")
    check_upstream.add_argument("--upstream", type=pathlib.Path, required=True)
    extract = commands.add_parser("extract")
    extract.add_argument("--upstream", type=pathlib.Path, required=True)
    cpp = commands.add_parser("run-cpp")
    cpp.add_argument("--cpp-bench", type=pathlib.Path, required=True)
    cpp.add_argument("--duration", type=int, default=5)
    cpp.add_argument("--output", type=pathlib.Path, required=True)
    run = commands.add_parser("run")
    run.add_argument("--upstream", type=pathlib.Path, required=True)
    run.add_argument("--run-dir", type=pathlib.Path, required=True)
    run.add_argument("--duration", type=int, default=5)
    run.add_argument("--warm-up", type=int, default=3)
    run.add_argument("--measurement", type=int, default=10)
    run.add_argument("--sample-size", type=int, default=20)
    compare = commands.add_parser("compare")
    compare.add_argument("--run-manifest", type=pathlib.Path, required=True)
    compare.add_argument("--output", type=pathlib.Path)
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv or sys.argv[1:])
    repo_root = args.repo_root.resolve()
    manifest = args.manifest if args.manifest.is_absolute() else repo_root / args.manifest
    inventory = load_inventory(manifest)
    if args.command == "check":
        check_datasets(repo_root, inventory)
        check_bench_sources(repo_root, inventory)
        print(f"microbenchmark contract: {len(inventory.cases)} cases, {len(inventory.datasets)} datasets")
    elif args.command == "check-upstream":
        check_upstream_datasets(repo_root, args.upstream.resolve(), inventory)
        print(
            f"microbenchmark provenance: {len(inventory.datasets)} datasets match "
            f"upstream {inventory.upstream_ref}"
        )
    elif args.command == "extract":
        extract_datasets(repo_root, args.upstream.resolve(), inventory)
    elif args.command == "run-cpp":
        run_cpp(args.cpp_bench.resolve(), inventory, args.duration, args.output)
    elif args.command == "run":
        result = run_benchmarks(
            repo_root,
            manifest,
            inventory,
            args.upstream.resolve(),
            args.run_dir.resolve(),
            args.duration,
            args.warm_up,
            args.measurement,
            args.sample_size,
        )
        print(result)
    elif args.command == "compare":
        run = load_run(repo_root, manifest, args.run_manifest.resolve(), inventory)
        table = render_report(
            inventory,
            load_sealed_cpp_timings(run),
            load_sealed_criterion_timings(run, inventory),
        )
        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(table)
        print(table, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
