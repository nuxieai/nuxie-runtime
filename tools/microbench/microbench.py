#!/usr/bin/env python3
"""Guard and compare the pinned upstream/Rust microbenchmark mirror."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
import struct
import subprocess
import sys
import tomllib
from typing import NamedTuple


class ContractError(RuntimeError):
    pass


class Case(NamedTuple):
    name: str
    crate: str
    source: str


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
    cases: list[Case]
    datasets: list[Dataset]


def sha256(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


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
    return Inventory(raw["schema"], raw["upstream_ref"], cases, datasets)


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


def load_cpp_timings(path: pathlib.Path) -> dict[str, float]:
    timings: dict[str, float] = {}
    for line in path.read_text().splitlines():
        match = re.fullmatch(r"\s*([0-9.eE+-]+)ms\s+(\S+)\s*", line)
        if match:
            timings[match.group(2)] = float(match.group(1)) * 1_000_000.0
    return timings


def load_criterion_timings(criterion_dir: pathlib.Path, inventory: Inventory) -> dict[str, float]:
    timings: dict[str, float] = {}
    for case in inventory.cases:
        estimates = criterion_dir / case.name / "new" / "estimates.json"
        if not estimates.is_file():
            raise ContractError(f"missing criterion estimate for {case.name}: {estimates}")
        raw = json.loads(estimates.read_text())
        timings[case.name] = float(raw["median"]["point_estimate"])
    return timings


def render_ratio_table(
    inventory: Inventory, cpp_nanoseconds: dict[str, float], rust_nanoseconds: dict[str, float]
) -> str:
    lines = [
        "| Benchmark | C++ | Rust | Rust/C++ |",
        "|---|---:|---:|---:|",
    ]
    for case in inventory.cases:
        if case.name not in cpp_nanoseconds or case.name not in rust_nanoseconds:
            raise ContractError(f"missing timing for {case.name}")
        cpp = cpp_nanoseconds[case.name]
        rust = rust_nanoseconds[case.name]
        lines.append(
            f"| `{case.name}` | {cpp / 1_000_000:.6f} ms | "
            f"{rust / 1_000_000:.6f} ms | {rust / cpp:.3f}x |"
        )
    return "\n".join(lines) + "\n"


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
    compare = commands.add_parser("compare")
    compare.add_argument("--cpp-output", type=pathlib.Path, required=True)
    compare.add_argument("--criterion-dir", type=pathlib.Path, default=pathlib.Path("target/criterion"))
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
    elif args.command == "compare":
        cpp = load_cpp_timings(args.cpp_output)
        criterion_dir = args.criterion_dir
        if not criterion_dir.is_absolute():
            criterion_dir = repo_root / criterion_dir
        table = render_ratio_table(inventory, cpp, load_criterion_timings(criterion_dir, inventory))
        if args.output:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_text(table)
        print(table, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
