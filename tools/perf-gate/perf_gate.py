#!/usr/bin/env python3
"""Validate and operate the checked-in hot-loop performance ratchet."""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


MANIFEST_SCHEMA = "nuxie-perf-corpus-v1"
REQUIRED_DIVERSITY = {
    "text-heavy",
    "list-virtualization",
    "nested-artboards",
    "scripted",
    "layout-heavy",
}


@dataclass(frozen=True)
class PerfFile:
    id: str
    file_bytes: int
    categories: tuple[str, ...]
    note: str
    baseline_ratio: float
    ceiling: int


@dataclass(frozen=True)
class PerfManifest:
    source: str
    minimum_files: int
    files: tuple[PerfFile, ...]


@dataclass(frozen=True)
class ReportRow:
    id: str
    cpp_ms_per_frame: float
    rust_ms_per_frame: float
    ratio: float
    ceiling: int

    @property
    def passed(self) -> bool:
        return self.ratio <= self.ceiling


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    for name, help_text in (
        ("check-manifest", "validate the perf corpus against corpus.toml"),
        ("ids", "print the validated comma-separated perf corpus ids"),
    ):
        command = subparsers.add_parser(name, help=help_text)
        command.add_argument("--manifest", type=Path, default=Path("perf-corpus.toml"))
        command.add_argument("--corpus", type=Path, default=Path("corpus.toml"))
        command.add_argument("--rive-runtime-dir", type=Path)

    for name, help_text in (
        ("check-report", "print and enforce every per-file ratio ceiling"),
        ("tighten", "lower baselines and ceilings after measured improvements"),
    ):
        command = subparsers.add_parser(name, help=help_text)
        command.add_argument("--manifest", type=Path, default=Path("perf-corpus.toml"))
        command.add_argument("--corpus", type=Path, default=Path("corpus.toml"))
        command.add_argument("--rive-runtime-dir", type=Path)
        command.add_argument("--report", type=Path, action="append", required=True)

    options = parser.parse_args(argv)
    try:
        manifest = load_manifest(options.manifest)
        corpus = load_toml(options.corpus)
        validate_manifest(
            manifest,
            corpus,
            corpus_path=options.corpus,
            rive_runtime_dir=options.rive_runtime_dir,
        )
    except (OSError, ValueError, tomllib.TOMLDecodeError) as error:
        print(f"perf-gate error: {error}", file=sys.stderr)
        return 1

    if options.command == "ids":
        print(",".join(file.id for file in manifest.files))
    elif options.command == "check-manifest":
        categories = sorted(
            {category for file in manifest.files for category in file.categories}
        )
        print(
            f"perf-corpus ok files={len(manifest.files)} "
            f"categories={','.join(categories)} source={manifest.source}"
        )
    else:
        try:
            if options.command == "check-report" and len(options.report) != 1:
                raise ValueError("check-report requires exactly one --report")
            if options.command == "tighten" and len(options.report) != 3:
                raise ValueError("tighten requires exactly three independent --report values")
            sessions = tuple(
                evaluate_report(manifest, load_json(path), path)
                for path in options.report
            )
            rows = maximum_rows(sessions)
        except (OSError, ValueError, json.JSONDecodeError) as error:
            print(f"perf-gate error: {error}", file=sys.stderr)
            return 1
        if options.command == "tighten":
            print("perf-gate tighten candidate: per-file maximum of 3 sessions")
        print_ratio_table(rows)
        failed = [row for row in rows if not row.passed]
        if failed:
            print(
                "perf-gate error: ceilings exceeded: "
                + ", ".join(
                    f"{row.id}={row.ratio:.3f}>{row.ceiling}" for row in failed
                ),
                file=sys.stderr,
            )
            return 1
        if options.command == "tighten":
            try:
                updates = ratchet_updates(manifest, rows)
                write_tightened_manifest(options.manifest, updates)
            except (OSError, ValueError) as error:
                print(f"perf-gate error: {error}", file=sys.stderr)
                return 1
            if updates:
                print(
                    "perf-gate tightened: "
                    + ", ".join(
                        f"{file_id}={ratio:.6f}/{ceiling}"
                        for file_id, (ratio, ceiling) in updates.items()
                    )
                )
            else:
                print("perf-gate tightened: no improved baselines")
        else:
            print(f"perf-gate PASS files={len(rows)}")
    return 0


def load_toml(path: Path) -> dict[str, Any]:
    with path.open("rb") as source:
        return tomllib.load(source)


def load_json(path: Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as source:
        data = json.load(source)
    if not isinstance(data, dict):
        raise ValueError(f"{path}: report root must be an object")
    return data


def load_manifest(path: Path) -> PerfManifest:
    data = load_toml(path)
    if data.get("schema") != MANIFEST_SCHEMA:
        raise ValueError(
            f"{path}: schema must be {MANIFEST_SCHEMA!r}, got {data.get('schema')!r}"
        )
    source = require_nonempty_string(data, "source", path)
    minimum_files = data.get("minimum_files")
    if not isinstance(minimum_files, int) or isinstance(minimum_files, bool):
        raise ValueError(f"{path}: minimum_files must be an integer")
    if minimum_files < 20:
        raise ValueError(f"{path}: minimum_files must be at least 20")

    raw_files = data.get("file")
    if not isinstance(raw_files, list):
        raise ValueError(f"{path}: expected one or more [[file]] entries")
    files = []
    for index, raw_file in enumerate(raw_files, start=1):
        context = Path(f"{path} [[file]] #{index}")
        if not isinstance(raw_file, dict):
            raise ValueError(f"{context}: entry must be a table")
        file_id = require_nonempty_string(raw_file, "id", context)
        file_bytes = raw_file.get("bytes")
        if (
            not isinstance(file_bytes, int)
            or isinstance(file_bytes, bool)
            or file_bytes <= 0
        ):
            raise ValueError(f"{context}: bytes must be a positive integer")
        raw_categories = raw_file.get("categories")
        if not isinstance(raw_categories, list) or not raw_categories:
            raise ValueError(f"{context}: categories must be a non-empty string array")
        categories = tuple(raw_categories)
        if any(not isinstance(category, str) or not category for category in categories):
            raise ValueError(f"{context}: categories must be a non-empty string array")
        if len(categories) != len(set(categories)):
            raise ValueError(f"{context}: categories must not contain duplicates")
        note = require_nonempty_string(raw_file, "note", context)
        baseline_ratio = require_positive_number(raw_file, "baseline_ratio", context)
        ceiling = raw_file.get("ceiling")
        if not isinstance(ceiling, int) or isinstance(ceiling, bool) or ceiling <= 0:
            raise ValueError(f"{context}: ceiling must be a positive integer")
        expected_ceiling = math.ceil(baseline_ratio * 1.15)
        if ceiling != expected_ceiling:
            raise ValueError(
                f"{context}: ceiling {ceiling} must equal "
                f"ceil({baseline_ratio:.6f} * 1.15) = {expected_ceiling}"
            )
        files.append(
            PerfFile(
                file_id,
                file_bytes,
                categories,
                note,
                baseline_ratio,
                ceiling,
            )
        )
    return PerfManifest(source, minimum_files, tuple(files))


def require_nonempty_string(data: dict[str, Any], key: str, context: Path) -> str:
    value = data.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{context}: {key} must be a non-empty string")
    return value


def require_positive_number(data: dict[str, Any], key: str, context: Path) -> float:
    value = data.get(key)
    if (
        not isinstance(value, (int, float))
        or isinstance(value, bool)
        or not math.isfinite(value)
        or value <= 0
    ):
        raise ValueError(f"{context}: {key} must be a finite positive number")
    return float(value)


def validate_manifest(
    manifest: PerfManifest,
    corpus: dict[str, Any],
    *,
    corpus_path: Path,
    rive_runtime_dir: Path | None,
) -> None:
    if manifest.source != corpus_path.name:
        raise ValueError(
            f"manifest source {manifest.source!r} does not match {corpus_path.name!r}"
        )
    if len(manifest.files) < manifest.minimum_files:
        raise ValueError(
            f"manifest has {len(manifest.files)} files; minimum is {manifest.minimum_files}"
        )
    ids = [file.id for file in manifest.files]
    if len(ids) != len(set(ids)):
        duplicates = sorted({file_id for file_id in ids if ids.count(file_id) > 1})
        raise ValueError(f"manifest contains duplicate ids: {','.join(duplicates)}")

    selected_categories = {
        category for file in manifest.files for category in file.categories
    }
    missing_categories = sorted(REQUIRED_DIVERSITY - selected_categories)
    if missing_categories:
        raise ValueError(
            "manifest is missing required diversity categories: "
            + ",".join(missing_categories)
        )

    corpus_files = corpus.get("file")
    if not isinstance(corpus_files, list):
        raise ValueError(f"{corpus_path}: expected [[file]] entries")
    corpus_by_id = {
        entry.get("id"): entry
        for entry in corpus_files
        if isinstance(entry, dict) and isinstance(entry.get("id"), str)
    }
    for file in manifest.files:
        source = corpus_by_id.get(file.id)
        if source is None:
            raise ValueError(f"manifest id {file.id!r} is absent from {corpus_path}")
        if source.get("status") != "exact":
            raise ValueError(
                f"manifest id {file.id!r} must remain exact in {corpus_path}"
            )
        if source.get("input_script") is not None:
            raise ValueError(
                f"manifest id {file.id!r} has input_script; the perf method requires none"
            )
        if rive_runtime_dir is not None:
            source_path = source.get("path")
            if not isinstance(source_path, str) or not source_path:
                raise ValueError(f"corpus id {file.id!r} has no source path")
            actual_bytes = (rive_runtime_dir / source_path).stat().st_size
            if actual_bytes != file.file_bytes:
                raise ValueError(
                    f"manifest id {file.id!r} records {file.file_bytes} bytes; "
                    f"{source_path} has {actual_bytes}"
                )


def evaluate_report(
    manifest: PerfManifest, report: dict[str, Any], report_path: Path
) -> tuple[ReportRow, ...]:
    expected_scalars = {
        "schema": "rive-perf-compare-json-v1",
        "metric": "runner_hot_loop_ms",
        "iterations": 5,
        "warmups": 0,
        "benchmark_repeat": 1,
        "benchmark_frames": 100,
        "rust_execute_scripts": True,
    }
    for key, expected in expected_scalars.items():
        if report.get(key) != expected:
            raise ValueError(
                f"{report_path}: {key} must be {expected!r}, got {report.get(key)!r}"
            )
    benchmark_hz = report.get("benchmark_hz")
    if not isinstance(benchmark_hz, (int, float)) or benchmark_hz != 60:
        raise ValueError(
            f"{report_path}: benchmark_hz must be 60, got {benchmark_hz!r}"
        )
    meta = report.get("meta")
    if not isinstance(meta, dict) or meta.get("build_profile") != "release":
        raise ValueError(f"{report_path}: meta.build_profile must be 'release'")
    report_files = report.get("files")
    if not isinstance(report_files, list):
        raise ValueError(f"{report_path}: files must be an array")
    expected_ids = [file.id for file in manifest.files]
    actual_ids = [
        file.get("id") if isinstance(file, dict) else None for file in report_files
    ]
    if actual_ids != expected_ids:
        raise ValueError(
            f"{report_path}: file ids/order do not match {','.join(expected_ids)}"
        )

    rows = []
    for manifest_file, report_file in zip(manifest.files, report_files, strict=True):
        assert isinstance(report_file, dict)
        if report_file.get("segments") != 100:
            raise ValueError(
                f"{report_path}: {manifest_file.id} must report 100 segments"
            )
        cpp_ms = phase_median(report_file, "cpp", report_path)
        rust_ms = phase_median(report_file, "rust", report_path)
        if cpp_ms <= 0:
            raise ValueError(
                f"{report_path}: {manifest_file.id} C++ advance_draw median must be positive"
            )
        rows.append(
            ReportRow(
                manifest_file.id,
                cpp_ms / 100,
                rust_ms / 100,
                rust_ms / cpp_ms,
                manifest_file.ceiling,
            )
        )
    return tuple(rows)


def maximum_rows(sessions: tuple[tuple[ReportRow, ...], ...]) -> tuple[ReportRow, ...]:
    if not sessions:
        raise ValueError("at least one performance session is required")
    row_count = len(sessions[0])
    if any(len(session) != row_count for session in sessions):
        raise ValueError("performance sessions have inconsistent row counts")
    return tuple(
        max((session[index] for session in sessions), key=lambda row: row.ratio)
        for index in range(row_count)
    )


def phase_median(report_file: dict[str, Any], runner: str, report_path: Path) -> float:
    try:
        value = report_file["runners"][runner]["phases"]["advance_draw"]["median_ms"]
    except (KeyError, TypeError) as error:
        raise ValueError(
            f"{report_path}: {report_file.get('id')} has no {runner} advance_draw median"
        ) from error
    if (
        not isinstance(value, (int, float))
        or isinstance(value, bool)
        or not math.isfinite(value)
        or value < 0
    ):
        raise ValueError(
            f"{report_path}: {report_file.get('id')} {runner} advance_draw median is invalid"
        )
    return float(value)


def print_ratio_table(rows: tuple[ReportRow, ...]) -> None:
    print("perf-gate advance+draw: release median of 5, 100 frames at 60 Hz")
    print("| file | C++ ms/frame | Rust ms/frame | Rust/C++ | ceiling | result |")
    print("|---|---:|---:|---:|---:|---|")
    for row in rows:
        result = "PASS" if row.passed else "FAIL"
        print(
            f"| {row.id} | {row.cpp_ms_per_frame:.6f} | "
            f"{row.rust_ms_per_frame:.6f} | {row.ratio:.3f}x | "
            f"{row.ceiling}x | {result} |"
        )


def ratchet_updates(
    manifest: PerfManifest, rows: tuple[ReportRow, ...]
) -> dict[str, tuple[float, int]]:
    by_id = {row.id: row for row in rows}
    updates = {}
    for file in manifest.files:
        measured = round(by_id[file.id].ratio, 6)
        if measured >= file.baseline_ratio:
            continue
        ceiling = math.ceil(measured * 1.15)
        if ceiling > file.ceiling:
            raise ValueError(
                f"tightening {file.id} would loosen ceiling {file.ceiling} to {ceiling}"
            )
        updates[file.id] = (measured, ceiling)
    return updates


def write_tightened_manifest(
    path: Path, updates: dict[str, tuple[float, int]]
) -> None:
    if not updates:
        return
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
    current_id = None
    seen_baselines = set()
    seen_ceilings = set()
    rendered = []
    for line in lines:
        id_match = re.fullmatch(r'id = "([^"]+)"\n?', line)
        if id_match:
            current_id = id_match.group(1)
        if current_id in updates and line.startswith("baseline_ratio = "):
            rendered.append(f"baseline_ratio = {updates[current_id][0]:.6f}\n")
            seen_baselines.add(current_id)
        elif current_id in updates and line.startswith("ceiling = "):
            rendered.append(f"ceiling = {updates[current_id][1]}\n")
            seen_ceilings.add(current_id)
        else:
            rendered.append(line)
    missing = set(updates) - (seen_baselines & seen_ceilings)
    if missing:
        raise ValueError(f"manifest is missing ratchet fields for: {','.join(sorted(missing))}")
    temporary = path.with_name(f".{path.name}.{os.getpid()}.new")
    temporary.write_text("".join(rendered), encoding="utf-8")
    temporary.replace(path)


if __name__ == "__main__":
    raise SystemExit(main())
