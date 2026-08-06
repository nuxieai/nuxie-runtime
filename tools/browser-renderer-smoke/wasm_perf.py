#!/usr/bin/env python3
"""Contract and orchestration helpers for the browser wasm perf lane."""

from __future__ import annotations

import argparse
import datetime
import json
import hashlib
import math
import platform
import shutil
import statistics
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any


class ContractError(ValueError):
    """Raised when a fixture or timing report cannot be compared safely."""


UNSUPPORTED_CATEGORIES = {"scripted"}
TIMING_FLOAT_FIELDS = (
    "elapsed_ms",
    "total_ms",
    "advance_ms",
    "input_ms",
    "prepare_ms",
    "draw_ms",
    "bookkeeping_ms",
)


def _load_toml(path: Path) -> dict[str, Any]:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise ContractError(f"cannot read TOML manifest {path}: {error}") from error


def audit_production_boundary(feature_tree: str, source: str) -> None:
    for forbidden in ('nuxie feature "test-support"', 'nuxie feature "scripting"'):
        if forbidden in feature_tree:
            raise ContractError(f"measured wasm feature tree contains forbidden {forbidden}")
    marker = "pub struct WasmPerfRunner"
    if marker not in source:
        raise ContractError("wasm perf source omitted WasmPerfRunner")
    runner_source = source[source.index(marker) :]
    if "File::import(bytes)" not in runner_source or "import_with_unsigned_scripts" in runner_source.split("impl WasmPerfRunner", 1)[-1].split("pub async fn", 1)[0]:
        raise ContractError("WasmPerfRunner must use production File::import")


def select_fixtures(
    perf_manifest: Path,
    corpus_manifest: Path,
    rive_runtime_dir: Path,
    *,
    limit: int,
    requested_ids: list[str],
) -> list[dict[str, Any]]:
    """Join the perf and exact corpora, rejecting semantics this lane cannot execute."""
    if limit <= 0:
        raise ContractError("fixture limit must be a positive integer")
    perf = _load_toml(perf_manifest)
    corpus = _load_toml(corpus_manifest)
    perf_rows = perf.get("file")
    corpus_rows = corpus.get("file")
    if not isinstance(perf_rows, list) or not isinstance(corpus_rows, list):
        raise ContractError("perf and corpus manifests must contain [[file]] rows")

    exact_by_id = {row.get("id"): row for row in corpus_rows if isinstance(row, dict)}
    perf_by_id = {row.get("id"): row for row in perf_rows if isinstance(row, dict)}
    if requested_ids:
        if len(set(requested_ids)) != len(requested_ids):
            raise ContractError("requested perf fixture IDs must be unique")
        unknown = [fixture_id for fixture_id in requested_ids if fixture_id not in perf_by_id]
        if unknown:
            raise ContractError(f"unknown perf fixture(s): {', '.join(unknown)}")
        candidates = [perf_by_id[fixture_id] for fixture_id in requested_ids]
    else:
        candidates = sorted(
            perf_rows,
            key=lambda row: int(row.get("bytes", 0)),
            reverse=True,
        )

    target_count = len(requested_ids) if requested_ids else limit
    selected: list[dict[str, Any]] = []
    rejected: list[str] = []
    for perf_row in candidates:
        fixture_id = perf_row.get("id")
        exact = exact_by_id.get(fixture_id)
        if exact is None:
            message = f"fixture {fixture_id!r} is absent from exact corpus {corpus_manifest}"
            if requested_ids:
                raise ContractError(message)
            rejected.append(message)
            continue
        if exact.get("status") not in (None, "exact"):
            message = f"fixture {fixture_id!r} is not exact in {corpus_manifest}"
            if requested_ids:
                raise ContractError(message)
            rejected.append(message)
            continue
        categories = set(perf_row.get("categories", []))
        unsupported = categories & UNSUPPORTED_CATEGORIES
        if unsupported or exact.get("input_script") or exact.get("view_model_script"):
            message = (
                f"fixture {fixture_id!r} requires scripted semantics; "
                "the visual-only wasm timing lane cannot execute them"
            )
            if requested_ids:
                raise ContractError(message)
            rejected.append(message)
            continue
        features = exact.get("features", [])
        if any(
            feature.startswith("type-key:100:Image")
            or feature.startswith("type-key:105:ImageAsset")
            for feature in features
        ):
            message = (
                f"fixture {fixture_id!r} requires image decode semantics; "
                "the production NullFactory timing boundary has no image decoder"
            )
            if requested_ids:
                raise ContractError(message)
            rejected.append(message)
            continue
        samples = exact.get("samples")
        if not isinstance(samples, list) or not samples:
            message = f"fixture {fixture_id!r} has no timing samples"
            if requested_ids:
                raise ContractError(message)
            rejected.append(message)
            continue
        relative_path = exact.get("path")
        if not isinstance(relative_path, str):
            raise ContractError(f"fixture {fixture_id!r} has no path")
        fixture_path = rive_runtime_dir / relative_path
        if not fixture_path.is_file():
            raise ContractError(f"missing fixture {fixture_id!r}: {fixture_path}")
        actual_bytes = fixture_path.stat().st_size
        declared_bytes = int(perf_row.get("bytes", actual_bytes))
        if actual_bytes != declared_bytes:
            raise ContractError(
                f"fixture {fixture_id!r} size drifted: manifest={declared_bytes} actual={actual_bytes}"
            )
        selected.append(
            {
                "id": fixture_id,
                "bytes": actual_bytes,
                "relative_path": relative_path,
                "path": str(fixture_path.resolve()),
                "sha256": hashlib.sha256(fixture_path.read_bytes()).hexdigest(),
                # Repeat reports intentionally use one absolute sample, matching
                # rust-golden-runner's repeat-mode contract.
                "sample_seconds": float(samples[0]),
            }
        )
        if len(selected) == target_count:
            break

    if len(selected) < target_count:
        detail = "; ".join(rejected) or "no eligible rows"
        raise ContractError(f"only selected {len(selected)} supported fixtures: {detail}")
    return selected


def parse_native_report(output: str) -> dict[str, Any]:
    lines = [line.strip() for line in output.splitlines() if line.strip()]
    try:
        header_index = lines.index("rive-golden-benchmark-v1")
    except ValueError as error:
        raise ContractError("native output omitted rive-golden-benchmark-v1 header") from error
    values: dict[str, Any] = {"schema": lines[header_index]}
    for line in lines[header_index + 1 :]:
        if "=" not in line:
            continue
        key, raw_value = line.split("=", 1)
        if key == "segments":
            values[key] = int(raw_value)
        elif key.endswith("_ms"):
            values[key] = float(raw_value)
    values.setdefault("total_ms", values.get("elapsed_ms"))
    values["accounted_ms"] = sum(
        float(values.get(key, 0.0))
        for key in ("advance_ms", "input_ms", "prepare_ms", "draw_ms")
    )
    return validate_timing_report(values)


def validate_timing_report(report: dict[str, Any]) -> dict[str, Any]:
    required = ("schema", *TIMING_FLOAT_FIELDS, "accounted_ms", "segments")
    for field in required:
        if field not in report:
            raise ContractError(f"missing report field {field}")
    if report["schema"] != "rive-golden-benchmark-v1":
        raise ContractError(f"unsupported timing schema {report['schema']!r}")
    for field in (*TIMING_FLOAT_FIELDS, "accounted_ms"):
        value = report[field]
        if not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0:
            raise ContractError(f"invalid non-negative timing {field}={value!r}")
    if not isinstance(report["segments"], int) or report["segments"] <= 0:
        raise ContractError("segments must be a positive integer")
    expected = sum(float(report[key]) for key in ("advance_ms", "input_ms", "prepare_ms", "draw_ms"))
    if not math.isclose(float(report["accounted_ms"]), expected, rel_tol=1e-9, abs_tol=1e-9):
        raise ContractError("accounted_ms does not equal advance+input+prepare+draw")
    return report


def _metric_summary(values: list[float]) -> dict[str, float]:
    if not values:
        raise ContractError("cannot summarize an empty run set")
    mean = statistics.fmean(values)
    deviation = statistics.stdev(values) if len(values) > 1 else 0.0
    return {
        "median": statistics.median(values),
        "mean": mean,
        "min": min(values),
        "max": max(values),
        "standard_deviation": deviation,
        "coefficient_of_variation": deviation / mean if mean else 0.0,
    }


def _run_summary(runs: list[dict[str, Any]]) -> dict[str, Any]:
    for run in runs:
        validate_timing_report(run)
    fields = (
        "elapsed_ms",
        "advance_ms",
        "draw_ms",
        "accounted_ms",
        "bookkeeping_ms",
    )
    result: dict[str, Any] = {"run_count": len(runs), "runs": runs}
    for field in fields:
        result[field] = _metric_summary([float(run[field]) for run in runs])
    return result


def build_comparison_report(
    fixtures: list[dict[str, Any]],
    wasm_runs: dict[str, list[dict[str, Any]]],
    native_runs: dict[str, list[dict[str, Any]]],
    *,
    identity: dict[str, str],
    repeat: int,
    warmups: int,
) -> dict[str, Any]:
    required_identity = ("git_sha", "rive_runtime_sha", "browser", "build_profile")
    missing_identity = [key for key in required_identity if not identity.get(key)]
    if missing_identity:
        raise ContractError(f"missing identity metadata: {', '.join(missing_identity)}")
    rows = []
    for fixture in fixtures:
        fixture_id = fixture["id"]
        wasm = _run_summary(wasm_runs.get(fixture_id, []))
        native = _run_summary(native_runs.get(fixture_id, []))
        ratios = {}
        for metric, label in (
            ("elapsed_ms", "elapsed"),
            ("advance_ms", "advance"),
            ("draw_ms", "draw"),
            ("accounted_ms", "accounted"),
            ("bookkeeping_ms", "bookkeeping"),
        ):
            denominator = native[metric]["median"]
            ratios[label] = wasm[metric]["median"] / denominator if denominator else None
        rows.append(
            {
                **{
                    key: fixture[key]
                    for key in ("id", "bytes", "sha256", "relative_path", "sample_seconds")
                    if key in fixture
                },
                "segments_per_run": repeat,
                "wasm": wasm,
                "native_rust": native,
                "ratio": ratios,
            }
        )
    return {
        "schema": "nuxie-wasm-perf-v1",
        "conclusion": "report-only",
        "budget": None,
        "identity": identity,
        "measurement": {
            "clock": "browser performance.now()",
            "lifecycle": "fresh total and phase instances, retained topology primed before clocks",
            "repeat": repeat,
            "browser_warmups": warmups,
        },
        "fixtures": rows,
    }


def canonical_json(payload: dict[str, Any]) -> str:
    return json.dumps(payload, indent=2, sort_keys=True) + "\n"


def _git_sha(directory: Path) -> str:
    result = subprocess.run(
        ["git", "-C", str(directory), "rev-parse", "HEAD"],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0 or not result.stdout.strip():
        raise ContractError(f"cannot identify git checkout {directory}: {result.stderr.strip()}")
    return result.stdout.strip()


def prepare_run(args: argparse.Namespace) -> None:
    repo_root = args.repo_root.resolve()
    fixtures = select_fixtures(
        args.perf_manifest,
        args.corpus,
        args.rive_runtime_dir,
        limit=args.limit,
        requested_ids=[value for value in args.ids.split(",") if value],
    )
    args.staging_dir.mkdir(parents=True, exist_ok=True)
    staged_fixtures = []
    for fixture in fixtures:
        staged_path = args.staging_dir / f"{fixture['id']}.riv"
        shutil.copyfile(fixture["path"], staged_path)
        try:
            url_path = staged_path.resolve().relative_to(repo_root)
        except ValueError as error:
            raise ContractError("staging directory must be inside the repository root") from error
        staged_fixtures.append({**fixture, "url": f"/{url_path.as_posix()}"})
    identity = {
        "git_sha": _git_sha(repo_root),
        "rive_runtime_sha": _git_sha(args.rive_runtime_dir.resolve()),
        "browser": "pending",
        "build_profile": "release",
        "host": f"{platform.system()}-{platform.machine()}",
        "timestamp_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    }
    payload = {
        "schema": "nuxie-wasm-perf-config-v1",
        "repeat": args.repeat,
        "runs": args.runs,
        "warmups": args.warmups,
        "identity": identity,
        "fixtures": staged_fixtures,
    }
    args.config.parent.mkdir(parents=True, exist_ok=True)
    args.config.write_text(canonical_json(payload), encoding="utf-8")


def audit_run(args: argparse.Namespace) -> None:
    completed = subprocess.run(
        [
            str(args.cargo),
            "tree",
            "--package",
            "browser-renderer-smoke",
            "--target",
            "wasm32-unknown-unknown",
            "--no-default-features",
            "-e",
            "features",
        ],
        cwd=args.repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise ContractError(f"cannot audit cargo feature tree: {completed.stderr.strip()}")
    audit_production_boundary(
        completed.stdout,
        args.source.read_text(encoding="utf-8"),
    )


def _native_runs(config: dict[str, Any], runner: Path) -> dict[str, list[dict[str, Any]]]:
    result: dict[str, list[dict[str, Any]]] = {}
    for fixture in config["fixtures"]:
        runs = []
        for run_index in range(config["runs"]):
            command = [
                str(runner),
                "--file",
                fixture["path"],
                "--samples",
                str(fixture["sample_seconds"]),
                "--benchmark",
                "--benchmark-repeat",
                str(config["repeat"]),
            ]
            completed = subprocess.run(command, text=True, capture_output=True, check=False)
            if completed.returncode != 0:
                raise ContractError(
                    f"native runner failed for {fixture['id']} run {run_index + 1}: "
                    f"{completed.stderr.strip() or completed.stdout.strip()}"
                )
            report = parse_native_report(completed.stdout)
            if report["segments"] != config["repeat"]:
                raise ContractError(f"native runner segment mismatch for {fixture['id']}")
            runs.append(report)
            print(
                f"native {fixture['id']} run {run_index + 1}/{config['runs']}: "
                f"{report['elapsed_ms']:.3f} ms",
                file=sys.stderr,
            )
        result[fixture["id"]] = runs
    return result


def _ratio(value: float | None) -> str:
    return "n/a" if value is None else f"{value:.3f}x"


def render_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# Wasm advance+draw performance evidence",
        "",
        f"Status: **{report['conclusion']}**. This evidence establishes a baseline; it does not enforce a budget.",
        "",
        f"Git `{report['identity']['git_sha']}`; rive-runtime `{report['identity']['rive_runtime_sha']}`; "
        f"browser `{report['identity']['browser']} {report['identity'].get('browser_version', '')}`; "
        f"{report['measurement']['repeat']} segments/run; "
        f"{report['measurement']['browser_warmups']} discarded browser warmup(s).",
        "",
        "| Fixture | Size | Wasm median (CV) | Native Rust median (CV) | Wasm/native | Advance | Draw |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for row in report["fixtures"]:
        wasm = row["wasm"]["elapsed_ms"]
        native = row["native_rust"]["elapsed_ms"]
        lines.append(
            f"| `{row['id']}` | {row['bytes']:,} | {wasm['median']:.3f} ms ({wasm['coefficient_of_variation']:.1%}) "
            f"| {native['median']:.3f} ms ({native['coefficient_of_variation']:.1%}) "
            f"| {_ratio(row['ratio']['elapsed'])} | {_ratio(row['ratio']['advance'])} | {_ratio(row['ratio']['draw'])} |"
        )
    lines.extend(
        [
            "",
            "Each side uses fresh total and phase-pass instances, primes retained topology before timing, and reports multiple independent runs. Browser timings use monotonic `performance.now()` around the same production advance and draw calls as the native facade. Scripted, input-driven, and image-decoding fixtures are unsupported and fail closed; very small phase values may show browser-clock quantization.",
            "",
        ]
    )
    return "\n".join(lines)


def finalize_run(args: argparse.Namespace) -> None:
    config = json.loads(args.config.read_text(encoding="utf-8"))
    browser = json.loads(args.browser_results.read_text(encoding="utf-8"))
    if config.get("schema") != "nuxie-wasm-perf-config-v1":
        raise ContractError("unsupported wasm perf config schema")
    if browser.get("schema") != "nuxie-wasm-perf-browser-raw-v1":
        raise ContractError("unsupported browser results schema")
    for fixture in config["fixtures"]:
        runs = browser.get("fixtures", {}).get(fixture["id"], [])
        if len(runs) != config["runs"]:
            raise ContractError(f"browser run count mismatch for {fixture['id']}")
        for report in runs:
            validate_timing_report(report)
            if report["segments"] != config["repeat"]:
                raise ContractError(f"browser segment mismatch for {fixture['id']}")
    native = _native_runs(config, args.native_runner)
    identity = {
        **config["identity"],
        "browser": browser["browser"],
        "browser_version": browser["browser_version"],
    }
    report = build_comparison_report(
        config["fixtures"],
        browser["fixtures"],
        native,
        identity=identity,
        repeat=config["repeat"],
        warmups=config["warmups"],
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(canonical_json(report), encoding="utf-8")
    if args.markdown:
        args.markdown.parent.mkdir(parents=True, exist_ok=True)
        args.markdown.write_text(render_markdown(report), encoding="utf-8")


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    prepare = subparsers.add_parser("prepare")
    prepare.add_argument("--repo-root", type=Path, required=True)
    prepare.add_argument("--rive-runtime-dir", type=Path, required=True)
    prepare.add_argument("--perf-manifest", type=Path, required=True)
    prepare.add_argument("--corpus", type=Path, required=True)
    prepare.add_argument("--staging-dir", type=Path, required=True)
    prepare.add_argument("--config", type=Path, required=True)
    prepare.add_argument("--limit", type=int, default=5)
    prepare.add_argument("--ids", default="")
    prepare.add_argument("--repeat", type=int, default=100)
    prepare.add_argument("--runs", type=int, default=5)
    prepare.add_argument("--warmups", type=int, default=1)
    prepare.set_defaults(action=prepare_run)

    audit = subparsers.add_parser("audit")
    audit.add_argument("--repo-root", type=Path, required=True)
    audit.add_argument("--cargo", type=Path, required=True)
    audit.add_argument("--source", type=Path, required=True)
    audit.set_defaults(action=audit_run)

    finalize = subparsers.add_parser("finalize")
    finalize.add_argument("--config", type=Path, required=True)
    finalize.add_argument("--browser-results", type=Path, required=True)
    finalize.add_argument("--native-runner", type=Path, required=True)
    finalize.add_argument("--output", type=Path, required=True)
    finalize.add_argument("--markdown", type=Path)
    finalize.set_defaults(action=finalize_run)
    return parser


def main() -> int:
    try:
        args = _parser().parse_args()
        if hasattr(args, "repeat") and (args.repeat <= 0 or args.runs < 2 or args.warmups < 0):
            raise ContractError(
                "repeat must be positive, runs must be at least 2, and warmups cannot be negative"
            )
        args.action(args)
        return 0
    except (ContractError, OSError, json.JSONDecodeError) as error:
        print(f"wasm perf error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
