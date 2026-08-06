#!/usr/bin/env python3
"""Contract and orchestration helpers for the browser wasm perf lane."""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import math
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
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
    runner_impl = runner_source.split("impl WasmPerfRunner", 1)[-1].split("pub async fn", 1)[0]
    deterministic_mode = runner_impl.find("set_runtime_deterministic_mode(true)")
    production_import = runner_impl.find("File::import(bytes)")
    if deterministic_mode < 0 or deterministic_mode > production_import:
        raise ContractError(
            "WasmPerfRunner must enable deterministic runtime mode before import"
        )
    if (
        "instantiate_default_view_model_instance" in runner_impl
        or "instantiate_view_model()" not in runner_impl
    ):
        raise ContractError(
            "WasmPerfRunner must use schema-default view model initialization"
        )
    if (
        "default_state_machine_instance()" in runner_impl
        or ".default_state_machine_index()" not in runner_impl
    ):
        raise ContractError(
            "WasmPerfRunner must select only the authored default state machine"
        )


def audit_python_coordinator_shell(source: str) -> None:
    required = (
        'exec 9<"$PYTHON_COORDINATOR"',
        'PYTHON_COORDINATOR_FD_PATH="/dev/fd/9"',
        "os.lseek(9, 0, os.SEEK_SET)",
        'python3 -c "$PYTHON_COORDINATOR_LOADER" "$@"',
    )
    missing = [marker for marker in required if marker not in source]
    if missing:
        raise ContractError(
            "wasm perf shell must retain one open Python coordinator descriptor"
        )
    if 'python3 "$PYTHON_COORDINATOR"' in source:
        raise ContractError(
            "wasm perf shell must not reopen mutable Python coordinator paths"
        )
    descriptor_invocation = "run_python_coordinator"
    if source.count(descriptor_invocation) != 5:
        raise ContractError(
            "wasm perf shell must run audit, prepare, seal, and finalize "
            "through one Python coordinator descriptor"
        )
    if not all(
        f'{descriptor_invocation} {command}' in source
        for command in ("audit", "prepare", "seal", "finalize")
    ):
        raise ContractError(
            "wasm perf shell omitted a descriptor-backed coordinator phase"
        )


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
    selected_fixture_count = 0
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
        fixture_identity = {
            "fixture_id": fixture_id,
            "bytes": actual_bytes,
            "relative_path": relative_path,
            "path": str(fixture_path.resolve()),
            "sha256": hashlib.sha256(fixture_path.read_bytes()).hexdigest(),
        }
        seen_samples: set[float] = set()
        for sample_index, raw_sample in enumerate(samples):
            if isinstance(raw_sample, bool) or not isinstance(raw_sample, (int, float)):
                raise ContractError(
                    f"fixture {fixture_id!r} has invalid timing sample {raw_sample!r}"
                )
            sample_seconds = float(raw_sample)
            if not math.isfinite(sample_seconds) or sample_seconds < 0:
                raise ContractError(
                    f"fixture {fixture_id!r} has invalid timing sample {raw_sample!r}"
                )
            if sample_seconds in seen_samples:
                raise ContractError(
                    f"fixture {fixture_id!r} repeats timing sample {sample_seconds}"
                )
            seen_samples.add(sample_seconds)
            sample_label = format(sample_seconds, ".9g")
            selected.append(
                {
                    "id": f"{fixture_id}@{sample_label}s",
                    **fixture_identity,
                    "sample_index": sample_index,
                    "sample_seconds": sample_seconds,
                }
            )
        selected_fixture_count += 1
        if selected_fixture_count == target_count:
            break

    if selected_fixture_count < target_count:
        detail = "; ".join(rejected) or "no eligible rows"
        raise ContractError(
            f"only selected {selected_fixture_count} supported fixtures: {detail}"
        )
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
        elif key == "default_state_machine_id":
            values.setdefault("workload_identity", {})[key] = (
                None if raw_value == "none" else int(raw_value)
            )
        elif key in ("scene_kind", "view_model_initialization"):
            values.setdefault("workload_identity", {})[key] = raw_value
        elif key.endswith("_ms"):
            values[key] = float(raw_value)
    values.setdefault("total_ms", values.get("elapsed_ms"))
    values["accounted_ms"] = sum(
        float(values.get(key, 0.0))
        for key in ("advance_ms", "input_ms", "prepare_ms", "draw_ms")
    )
    return validate_timing_report(values)


def validate_timing_report(report: dict[str, Any]) -> dict[str, Any]:
    required = (
        "schema",
        *TIMING_FLOAT_FIELDS,
        "accounted_ms",
        "segments",
        "workload_identity",
    )
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
    validate_workload_identity(report["workload_identity"])
    return report


def validate_workload_identity(identity: Any) -> dict[str, Any]:
    required = {
        "scene_kind",
        "default_state_machine_id",
        "view_model_initialization",
    }
    if not isinstance(identity, dict) or set(identity) != required:
        raise ContractError(
            "workload_identity must contain exactly scene_kind, "
            "default_state_machine_id, and view_model_initialization"
        )
    scene_kind = identity["scene_kind"]
    default_id = identity["default_state_machine_id"]
    view_model_mode = identity["view_model_initialization"]
    if scene_kind not in ("static", "state_machine"):
        raise ContractError(f"invalid workload scene_kind {scene_kind!r}")
    if default_id is not None and (
        not isinstance(default_id, int) or isinstance(default_id, bool) or default_id < 0
    ):
        raise ContractError(
            "workload default_state_machine_id must be null or a non-negative integer"
        )
    if scene_kind == "static" and default_id is not None:
        raise ContractError("static workload must not identify a default state machine")
    if scene_kind == "state_machine" and default_id is None:
        raise ContractError("state-machine workload must identify its authored default")
    if view_model_mode not in ("none", "schema-default"):
        raise ContractError(
            f"invalid workload view_model_initialization {view_model_mode!r}"
        )
    return identity


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
        raw_wasm_runs = wasm_runs.get(fixture_id, [])
        raw_native_runs = native_runs.get(fixture_id, [])
        _matching_target_identity(fixture, raw_wasm_runs, raw_native_runs)
        workload_identity = _matching_workload_identity(
            fixture_id, raw_wasm_runs, raw_native_runs
        )
        wasm = _run_summary(raw_wasm_runs)
        native = _run_summary(raw_native_runs)
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
                    for key in (
                        "id",
                        "fixture_id",
                        "sample_index",
                        "sample_seconds",
                        "bytes",
                        "sha256",
                        "relative_path",
                    )
                    if key in fixture
                },
                "segments_per_run": repeat,
                "workload_identity": workload_identity,
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


def _matching_workload_identity(
    fixture_id: str,
    wasm_runs: list[dict[str, Any]],
    native_runs: list[dict[str, Any]],
) -> dict[str, Any]:
    if not wasm_runs or not native_runs:
        raise ContractError(f"{fixture_id} cannot compare empty run sets")
    wasm_identities = [
        validate_workload_identity(run.get("workload_identity")) for run in wasm_runs
    ]
    native_identities = [
        validate_workload_identity(run.get("workload_identity")) for run in native_runs
    ]
    expected = wasm_identities[0]
    if any(identity != expected for identity in wasm_identities[1:]):
        raise ContractError(f"{fixture_id} wasm workload identity changed between runs")
    native_expected = native_identities[0]
    if any(identity != native_expected for identity in native_identities[1:]):
        raise ContractError(f"{fixture_id} native workload identity changed between runs")
    if expected != native_expected:
        raise ContractError(
            f"{fixture_id} workload identity mismatch: wasm={expected!r} "
            f"native={native_expected!r}"
        )
    return expected


def _target_identity(fixture: dict[str, Any]) -> dict[str, Any]:
    return {
        "id": fixture["id"],
        "fixture_id": fixture.get("fixture_id", fixture["id"]),
        "sample_index": fixture.get("sample_index", 0),
        "sample_seconds": fixture["sample_seconds"],
    }


def _matching_target_identity(
    fixture: dict[str, Any],
    wasm_runs: list[dict[str, Any]],
    native_runs: list[dict[str, Any]],
) -> dict[str, Any]:
    expected = _target_identity(fixture)
    for label, runs in (("wasm", wasm_runs), ("native", native_runs)):
        if not runs:
            raise ContractError(f"{fixture['id']} cannot compare empty run sets")
        for run in runs:
            if run.get("target_identity") != expected:
                raise ContractError(
                    f"{fixture['id']} {label} sample target identity mismatch: "
                    f"expected={expected!r} current={run.get('target_identity')!r}"
                )
    return expected


def canonical_json(payload: dict[str, Any]) -> str:
    return json.dumps(payload, indent=2, sort_keys=True) + "\n"


def _write_coordinator_bundle(
    contents: dict[str, bytes],
    output_root: Path,
    *,
    source_identity: dict[str, str] | None = None,
) -> Path:
    if not contents:
        raise ContractError("coordinator bundle cannot be empty")
    records: dict[str, dict[str, Any]] = {}
    for name, value in sorted(contents.items()):
        if not name or Path(name).name != name:
            raise ContractError(f"invalid coordinator bundle name: {name!r}")
        records[name] = {
            "bytes": len(value),
            "sha256": hashlib.sha256(value).hexdigest(),
        }
    manifest = {
        "schema": "nuxie-wasm-perf-coordinator-bundle-v1",
        "source": source_identity,
        "files": records,
    }
    bundle_sha256 = hashlib.sha256(canonical_json(manifest).encode("utf-8")).hexdigest()
    output_root.mkdir(parents=True, exist_ok=True)
    destination = output_root / bundle_sha256
    if not destination.exists():
        with tempfile.TemporaryDirectory(
            prefix=".coordinator-", dir=output_root
        ) as temporary:
            staged = Path(temporary)
            for name, value in contents.items():
                (staged / name).write_bytes(value)
                (staged / name).chmod(0o500 if name.endswith(".sh") else 0o400)
            (staged / "manifest.json").write_text(
                canonical_json(manifest), encoding="utf-8"
            )
            (staged / "manifest.json").chmod(0o400)
            try:
                staged.rename(destination)
            except FileExistsError:
                pass
    expected_names = set(records) | {"manifest.json"}
    if not destination.is_dir() or {
        path.name for path in destination.iterdir()
    } != expected_names:
        raise ContractError(f"invalid content-addressed coordinator bundle: {destination}")
    for name, expected in records.items():
        current = _artifact_record(destination / name)
        current_identity = {key: current[key] for key in ("bytes", "sha256")}
        if current_identity != expected:
            raise ContractError(
                f"content-addressed coordinator changed: {name} "
                f"expected={expected!r} current={current_identity!r}"
            )
    return destination


def stage_coordinator_bundle_from_git(
    repo_root: Path,
    tracked_sources: dict[str, str],
    output_root: Path,
    *,
    expected_source_identity: dict[str, str] | None = None,
) -> Path:
    repo_root = repo_root.resolve()
    output_root = output_root.resolve()
    _assert_clean_checkout(
        "coordinator", repo_root, allowed_outputs=[output_root]
    )
    source_identity = {
        "repo_sha": _git_output(repo_root, "rev-parse", "HEAD"),
        "repo_tree_sha": _git_output(repo_root, "rev-parse", "HEAD^{tree}"),
    }
    if (
        expected_source_identity is not None
        and source_identity != expected_source_identity
    ):
        raise ContractError(
            "coordinator source differs from bootstrap capture: "
            f"expected={expected_source_identity!r} current={source_identity!r}"
        )
    contents: dict[str, bytes] = {}
    for name, relative_path in tracked_sources.items():
        relative = Path(relative_path)
        if relative.is_absolute() or ".." in relative.parts:
            raise ContractError(
                f"coordinator source must be a tracked repo-relative path: {relative_path!r}"
            )
        normalized = relative.as_posix()
        tracked = _git_output(
            repo_root, "ls-files", "--error-unmatch", "--", normalized
        )
        if tracked != normalized:
            raise ContractError(f"coordinator source is not tracked: {normalized}")
        contents[name] = _git_bytes(
            repo_root, "show", f"{source_identity['repo_sha']}:{normalized}"
        )
    destination = _write_coordinator_bundle(
        contents,
        output_root,
        source_identity=source_identity,
    )
    _assert_clean_checkout(
        "coordinator", repo_root, allowed_outputs=[output_root]
    )
    current_identity = {
        "repo_sha": _git_output(repo_root, "rev-parse", "HEAD"),
        "repo_tree_sha": _git_output(repo_root, "rev-parse", "HEAD^{tree}"),
    }
    if current_identity != source_identity:
        raise ContractError(
            "coordinator source changed during Git-blob staging: "
            f"expected={source_identity!r} current={current_identity!r}"
        )
    return destination


def _git_output(directory: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(directory), *args],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise ContractError(
            f"cannot inspect git checkout {directory}: {result.stderr.strip()}"
        )
    return result.stdout.strip()


def _git_bytes(directory: Path, *args: str) -> bytes:
    result = subprocess.run(
        ["git", "-C", str(directory), *args],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if result.returncode != 0:
        raise ContractError(
            f"cannot read git object in {directory}: "
            f"{result.stderr.decode('utf-8', errors='replace').strip()}"
        )
    return result.stdout


def _allowed_untracked_paths(
    directory: Path, allowed_outputs: list[Path]
) -> list[Path]:
    allowed = []
    for output in allowed_outputs:
        resolved = output.resolve()
        try:
            relative = resolved.relative_to(directory)
        except ValueError:
            continue
        tracked = _git_output(directory, "ls-files", "--", relative.as_posix())
        if tracked:
            raise ContractError(
                f"generated output allowance contains tracked source: {relative.as_posix()}"
            )
        allowed.append(resolved)
    return allowed


def _assert_clean_checkout(
    label: str, directory: Path, allowed_outputs: list[Path]
) -> None:
    tracked = _git_output(
        directory, "status", "--porcelain=v1", "--untracked-files=no"
    )
    if tracked:
        raise ContractError(f"{label} source checkout is dirty: {tracked.splitlines()[0]}")
    allowed = _allowed_untracked_paths(directory, allowed_outputs)
    untracked = _git_output(
        directory, "ls-files", "--others", "--exclude-standard"
    ).splitlines()
    for relative in untracked:
        candidate = (directory / relative).resolve()
        if any(
            candidate == output or (output.is_dir() and candidate.is_relative_to(output))
            for output in allowed
        ):
            continue
        raise ContractError(f"{label} source checkout is dirty: ?? {relative}")


def capture_source_provenance(
    repo_root: Path,
    rive_runtime_dir: Path,
    *,
    allowed_outputs: list[Path] | None = None,
) -> dict[str, str]:
    allowed_outputs = allowed_outputs or []
    sources = {}
    for label, directory in (
        ("repo", repo_root.resolve()),
        ("rive_runtime", rive_runtime_dir.resolve()),
    ):
        _assert_clean_checkout(label, directory, allowed_outputs)
        sources[f"{label}_sha"] = _git_output(directory, "rev-parse", "HEAD")
        sources[f"{label}_tree_sha"] = _git_output(
            directory, "rev-parse", "HEAD^{tree}"
        )
    return sources


def verify_source_provenance(
    expected: dict[str, str],
    repo_root: Path,
    rive_runtime_dir: Path,
    *,
    allowed_outputs: list[Path] | None = None,
) -> None:
    try:
        current = capture_source_provenance(
            repo_root,
            rive_runtime_dir,
            allowed_outputs=allowed_outputs,
        )
    except ContractError as error:
        raise ContractError(f"measured source changed after capture: {error}") from error
    if current != expected:
        raise ContractError(
            f"measured source changed after capture: expected={expected!r} current={current!r}"
        )


def _artifact_record(path: Path) -> dict[str, Any]:
    resolved = path.resolve()
    if not resolved.is_file():
        raise ContractError(f"missing measured artifact: {resolved}")
    contents = resolved.read_bytes()
    return {
        "path": str(resolved),
        "bytes": len(contents),
        "sha256": hashlib.sha256(contents).hexdigest(),
    }


def _seal_fixture_records(fixtures: list[dict[str, Any]]) -> dict[str, Any]:
    sealed: dict[str, Any] = {}
    for fixture in fixtures:
        fixture_id = fixture.get("id")
        if not isinstance(fixture_id, str) or not fixture_id:
            raise ContractError("measured fixture omitted identity")
        if fixture_id in sealed:
            raise ContractError(f"duplicate measured fixture identity: {fixture_id}")
        source = _artifact_record(Path(fixture["path"]))
        staged = _artifact_record(Path(fixture["staged_path"]))
        expected_identity = {
            "bytes": fixture.get("bytes"),
            "sha256": fixture.get("sha256"),
        }
        for label, record in (("source", source), ("staged", staged)):
            actual_identity = {key: record[key] for key in ("bytes", "sha256")}
            if actual_identity != expected_identity:
                raise ContractError(
                    f"measured fixture identity mismatch: {fixture_id} {label} "
                    f"expected={expected_identity!r} current={actual_identity!r}"
                )
        sealed[fixture_id] = {
            "id": fixture_id,
            "fixture_id": fixture.get("fixture_id", fixture_id),
            "sample_index": fixture.get("sample_index", 0),
            "bytes": expected_identity["bytes"],
            "sha256": expected_identity["sha256"],
            "source_path": source["path"],
            "staged_path": staged["path"],
            "relative_path": fixture.get("relative_path"),
            "sample_seconds": fixture.get("sample_seconds"),
            "url": fixture.get("url"),
        }
    return sealed


def seal_run_provenance(
    sources: dict[str, str],
    repo_root: Path,
    rive_runtime_dir: Path,
    *,
    artifacts: dict[str, Path],
    fixtures: list[dict[str, Any]] | None = None,
    measurement: dict[str, Any] | None = None,
    run_identity: dict[str, Any] | None = None,
    allowed_outputs: list[Path] | None = None,
) -> dict[str, Any]:
    verify_source_provenance(
        sources,
        repo_root,
        rive_runtime_dir,
        allowed_outputs=allowed_outputs,
    )
    return {
        "sources": sources,
        "artifacts": {
            name: _artifact_record(path) for name, path in sorted(artifacts.items())
        },
        "fixtures": _seal_fixture_records(fixtures or []),
        "measurement": measurement,
        "run_identity": run_identity,
    }


def sealed_config_identity(provenance: dict[str, Any]) -> dict[str, Any]:
    sources = provenance["sources"]
    return {
        "git_sha": sources["repo_sha"],
        "git_tree_sha": sources["repo_tree_sha"],
        "rive_runtime_sha": sources["rive_runtime_sha"],
        "rive_runtime_tree_sha": sources["rive_runtime_tree_sha"],
        "browser": "pending",
        **(provenance.get("run_identity") or {}),
        "artifacts": {
            name: {key: record[key] for key in ("bytes", "sha256")}
            for name, record in provenance["artifacts"].items()
        },
        "fixtures": {
            fixture_id: {
                key: record[key] for key in ("bytes", "sha256")
            }
            for fixture_id, record in provenance["fixtures"].items()
        },
    }


def write_run_seal(path: Path, provenance: dict[str, Any]) -> str:
    payload = canonical_json(
        {"schema": "nuxie-wasm-perf-seal-v1", "provenance": provenance}
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(payload, encoding="utf-8")
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()


def load_run_seal(path: Path, expected_sha256: str) -> dict[str, Any]:
    contents = path.read_bytes()
    actual_sha256 = hashlib.sha256(contents).hexdigest()
    if actual_sha256 != expected_sha256:
        raise ContractError(
            "run seal sha256 mismatch: "
            f"expected={expected_sha256} current={actual_sha256}"
        )
    payload = json.loads(contents)
    if payload.get("schema") != "nuxie-wasm-perf-seal-v1" or not isinstance(
        payload.get("provenance"), dict
    ):
        raise ContractError("invalid wasm perf run seal")
    return payload["provenance"]


def verify_run_provenance(
    sealed: dict[str, Any],
    repo_root: Path,
    rive_runtime_dir: Path,
    *,
    allowed_outputs: list[Path] | None = None,
) -> None:
    verify_source_provenance(
        sealed["sources"],
        repo_root,
        rive_runtime_dir,
        allowed_outputs=allowed_outputs,
    )
    for name, expected in sealed["artifacts"].items():
        current = _artifact_record(Path(expected["path"]))
        if current != expected:
            raise ContractError(
                f"measured artifact changed after seal: {name} "
                f"expected={expected!r} current={current!r}"
            )
    for fixture_id, expected in sealed.get("fixtures", {}).items():
        expected_identity = {key: expected[key] for key in ("bytes", "sha256")}
        for label, path_key in (("source", "source_path"), ("staged", "staged_path")):
            current = _artifact_record(Path(expected[path_key]))
            current_identity = {key: current[key] for key in ("bytes", "sha256")}
            if current_identity != expected_identity:
                raise ContractError(
                    f"sealed fixture changed after seal: {fixture_id} {label} "
                    f"expected={expected_identity!r} current={current_identity!r}"
                )


def verify_browser_fixture_identities(
    sealed_fixtures: dict[str, Any], browser: dict[str, Any]
) -> None:
    loaded_fixtures = browser.get("loaded_fixtures")
    if not isinstance(loaded_fixtures, dict):
        raise ContractError("browser results omitted loaded fixture identities")
    if set(loaded_fixtures) != set(sealed_fixtures):
        raise ContractError(
            "browser loaded fixture identities do not match sealed fixtures: "
            f"sealed={sorted(sealed_fixtures)} loaded={sorted(loaded_fixtures)}"
        )
    for fixture_id, expected in sealed_fixtures.items():
        loaded = loaded_fixtures[fixture_id]
        if not isinstance(loaded, dict) or set(loaded) != {"bytes", "sha256"}:
            raise ContractError(
                f"browser loaded fixture identity is invalid: {fixture_id} {loaded!r}"
            )
        expected_identity = {key: expected[key] for key in ("bytes", "sha256")}
        if loaded != expected_identity:
            raise ContractError(
                f"browser loaded fixture identity mismatch: {fixture_id} "
                f"expected={expected_identity!r} loaded={loaded!r}"
            )


def verify_browser_artifact_identities(
    sealed_artifacts: dict[str, Any], browser: dict[str, Any]
) -> None:
    measured_names = {
        "wasm",
        "wasm_bindgen_js",
        "wasm_perf_driver_js",
        "wasm_perf_html",
    }
    missing_sealed = measured_names.difference(sealed_artifacts)
    if missing_sealed:
        raise ContractError(
            "wasm perf seal omitted browser artifacts: "
            f"{sorted(missing_sealed)}"
        )
    expected_names = measured_names
    loaded_artifacts = browser.get("loaded_artifacts")
    if not isinstance(loaded_artifacts, dict):
        raise ContractError("browser results omitted loaded artifact identities")
    if set(loaded_artifacts) != expected_names:
        raise ContractError(
            "browser loaded artifact identities do not match sealed artifacts: "
            f"sealed={sorted(expected_names)} loaded={sorted(loaded_artifacts)}"
        )
    for name in sorted(expected_names):
        expected = sealed_artifacts[name]
        loaded = loaded_artifacts[name]
        if not isinstance(loaded, dict) or set(loaded) != {"bytes", "sha256"}:
            raise ContractError(
                f"browser loaded artifact identity is invalid: {name} {loaded!r}"
            )
        expected_identity = {key: expected[key] for key in ("bytes", "sha256")}
        if loaded != expected_identity:
            raise ContractError(
                f"browser loaded artifact identity mismatch: {name} "
                f"expected={expected_identity!r} loaded={loaded!r}"
            )


def verify_browser_coordinator_identity(
    sealed_artifacts: dict[str, Any], browser: dict[str, Any]
) -> None:
    name = "wasm_perf_node"
    expected = sealed_artifacts.get(name)
    loaded = browser.get("coordinator_artifacts", {}).get(name)
    if not isinstance(expected, dict):
        raise ContractError("wasm perf seal omitted Node coordinator")
    if not isinstance(loaded, dict) or set(loaded) != {"bytes", "sha256"}:
        raise ContractError("browser results omitted executed Node coordinator identity")
    expected_identity = {key: expected[key] for key in ("bytes", "sha256")}
    if loaded != expected_identity:
        raise ContractError(
            "executed Node coordinator identity mismatch: "
            f"expected={expected_identity!r} loaded={loaded!r}"
        )


def verify_config_against_seal(
    config: dict[str, Any], provenance: dict[str, Any]
) -> None:
    expected_identity = sealed_config_identity(provenance)
    if config.get("identity") != expected_identity:
        raise ContractError(
            "config identity differs from sealed identity: "
            f"sealed={expected_identity!r} current={config.get('identity')!r}"
        )
    sealed_measurement = provenance.get("measurement")
    current_measurement = {
        key: config.get(key) for key in ("repeat", "runs", "warmups")
    }
    if current_measurement != sealed_measurement:
        raise ContractError(
            "config measurement differs from sealed measurement: "
            f"sealed={sealed_measurement!r} current={current_measurement!r}"
        )
    fixtures = config.get("fixtures")
    sealed_fixtures = provenance.get("fixtures")
    if not isinstance(fixtures, list) or not isinstance(sealed_fixtures, dict):
        raise ContractError("config omitted sealed fixtures")
    configured_by_id = {fixture.get("id"): fixture for fixture in fixtures}
    if len(configured_by_id) != len(fixtures) or set(configured_by_id) != set(
        sealed_fixtures
    ):
        raise ContractError("config fixture identities differ from sealed fixtures")
    for fixture_id, sealed in sealed_fixtures.items():
        configured = configured_by_id[fixture_id]
        current = {
            "id": fixture_id,
            "fixture_id": configured.get("fixture_id", fixture_id),
            "sample_index": configured.get("sample_index", 0),
            "bytes": configured.get("bytes"),
            "sha256": configured.get("sha256"),
            "source_path": str(Path(configured["path"]).resolve()),
            "staged_path": str(Path(configured["staged_path"]).resolve()),
            "relative_path": configured.get("relative_path"),
            "sample_seconds": configured.get("sample_seconds"),
            "url": configured.get("url"),
        }
        if current != sealed:
            raise ContractError(
                f"config fixture differs from sealed fixture: {fixture_id} "
                f"sealed={sealed!r} current={current!r}"
            )


def verify_browser_measurement_contract(
    provenance: dict[str, Any], browser: dict[str, Any]
) -> None:
    measurement = browser.get("measurement")
    if not isinstance(measurement, dict):
        raise ContractError("browser results omitted measurement contract")
    current_measurement = {
        key: measurement.get(key) for key in ("repeat", "runs", "warmups")
    }
    if current_measurement != provenance["measurement"]:
        raise ContractError(
            "browser measurement contract differs from sealed measurement: "
            f"sealed={provenance['measurement']!r} browser={current_measurement!r}"
        )
    fixtures = measurement.get("fixtures")
    if not isinstance(fixtures, list):
        raise ContractError("browser measurement contract omitted fixtures")
    current_by_id = {fixture.get("id"): fixture for fixture in fixtures}
    expected_by_id = {
        fixture_id: {
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
        for fixture_id, fixture in provenance["fixtures"].items()
    }
    if len(current_by_id) != len(fixtures) or current_by_id != expected_by_id:
        raise ContractError(
            "browser measurement contract differs from sealed measurement: "
            f"sealed={expected_by_id!r} browser={current_by_id!r}"
        )


def prepare_run(args: argparse.Namespace) -> None:
    repo_root = args.repo_root.resolve()
    rive_runtime_dir = args.rive_runtime_dir.resolve()
    allowed_outputs = [path.resolve() for path in args.allowed_output]
    source_provenance = capture_source_provenance(
        repo_root,
        rive_runtime_dir,
        allowed_outputs=allowed_outputs,
    )
    fixtures = select_fixtures(
        args.perf_manifest,
        args.corpus,
        rive_runtime_dir,
        limit=args.limit,
        requested_ids=[value for value in args.ids.split(",") if value],
    )
    args.staging_dir.mkdir(parents=True, exist_ok=True)
    staged_fixtures = []
    for fixture in fixtures:
        staged_path = args.staging_dir / f"{fixture['fixture_id']}.riv"
        shutil.copyfile(fixture["path"], staged_path)
        try:
            url_path = staged_path.resolve().relative_to(repo_root)
        except ValueError as error:
            raise ContractError("staging directory must be inside the repository root") from error
        staged_fixtures.append(
            {
                **fixture,
                "staged_path": str(staged_path.resolve()),
                "url": f"/{url_path.as_posix()}",
            }
        )
    identity = {
        "git_sha": source_provenance["repo_sha"],
        "git_tree_sha": source_provenance["repo_tree_sha"],
        "rive_runtime_sha": source_provenance["rive_runtime_sha"],
        "rive_runtime_tree_sha": source_provenance["rive_runtime_tree_sha"],
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
        "provenance": {"sources": source_provenance},
        "fixtures": staged_fixtures,
    }
    args.config.parent.mkdir(parents=True, exist_ok=True)
    args.config.write_text(canonical_json(payload), encoding="utf-8")


def seal_run(args: argparse.Namespace) -> None:
    config = json.loads(args.config.read_text(encoding="utf-8"))
    if config.get("schema") != "nuxie-wasm-perf-config-v1":
        raise ContractError("unsupported wasm perf config schema")
    sources = config.get("provenance", {}).get("sources")
    if not isinstance(sources, dict):
        raise ContractError("wasm perf config omitted source provenance")
    sealed = seal_run_provenance(
        sources,
        args.repo_root.resolve(),
        args.rive_runtime_dir.resolve(),
        artifacts={
            "native_runner": args.native_runner,
            "wasm": args.wasm_artifact,
            "wasm_bindgen_js": args.wasm_bindgen_js,
            "wasm_perf_driver_js": args.wasm_perf_driver_js,
            "wasm_perf_html": args.wasm_perf_html,
            "wasm_perf_node": args.wasm_perf_node,
            "wasm_perf_python": args.wasm_perf_python,
            "wasm_perf_shell": args.wasm_perf_shell,
        },
        fixtures=config["fixtures"],
        measurement={key: config[key] for key in ("repeat", "runs", "warmups")},
        run_identity={
            key: config["identity"][key]
            for key in ("build_profile", "host", "timestamp_utc")
        },
        allowed_outputs=[path.resolve() for path in args.allowed_output],
    )
    seal_sha256 = write_run_seal(args.seal, sealed)
    config["provenance"] = sealed
    config["identity"] = sealed_config_identity(sealed)
    args.config.write_text(canonical_json(config), encoding="utf-8")
    print(seal_sha256)


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
    audit_python_coordinator_shell(
        args.shell_source.read_text(encoding="utf-8")
    )


def stage_coordinator_run(args: argparse.Namespace) -> None:
    sources: dict[str, str] = {}
    for value in args.coordinator:
        name, separator, raw_path = value.partition("=")
        if not separator:
            raise ContractError(
                f"coordinator must use name=path syntax: {value!r}"
            )
        if name in sources:
            raise ContractError(f"duplicate coordinator bundle name: {name}")
        sources[name] = raw_path
    print(
        stage_coordinator_bundle_from_git(
            args.repo_root,
            sources,
            args.output_root,
            expected_source_identity={
                "repo_sha": args.expected_repo_sha,
                "repo_tree_sha": args.expected_repo_tree_sha,
            },
        )
    )


def _native_runs_with_runner(
    config: dict[str, Any], runner: Path, sealed_fixtures: dict[str, Any]
) -> dict[str, list[dict[str, Any]]]:
    result: dict[str, list[dict[str, Any]]] = {}
    for fixture in config["fixtures"]:
        sealed_fixture = sealed_fixtures[fixture["id"]]
        runs = []
        for run_index in range(config["runs"]):
            command = [
                str(runner),
                "--file",
                sealed_fixture["staged_path"],
                "--expected-file-sha256",
                sealed_fixture["sha256"],
                "--samples",
                str(sealed_fixture["sample_seconds"]),
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
            report["target_identity"] = _target_identity(fixture)
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


def _native_runs(
    config: dict[str, Any],
    runner: Path,
    sealed_fixtures: dict[str, Any],
    sealed_runner: dict[str, Any],
) -> dict[str, list[dict[str, Any]]]:
    runner_bytes = runner.read_bytes()
    runner_identity = {
        "bytes": len(runner_bytes),
        "sha256": hashlib.sha256(runner_bytes).hexdigest(),
    }
    expected_identity = {
        key: sealed_runner[key] for key in ("bytes", "sha256")
    }
    if runner_identity != expected_identity:
        raise ContractError(
            "native runner changed before execution: "
            f"expected={expected_identity!r} current={runner_identity!r}"
        )

    # Never execute the mutable build output itself. The exact bytes captured
    # above are written once to a private, content-addressed path; swaps of the
    # original runner after this point cannot alter the measured executable.
    with tempfile.TemporaryDirectory(prefix="nuxie-wasm-perf-native-") as temp_dir:
        sealed_copy = Path(temp_dir) / runner_identity["sha256"]
        with sealed_copy.open("xb") as output:
            output.write(runner_bytes)
            output.flush()
        sealed_copy.chmod(0o500)
        if _artifact_record(sealed_copy) != {
            "path": str(sealed_copy.resolve()),
            **runner_identity,
        }:
            raise ContractError("content-addressed native runner copy changed before execution")
        return _native_runs_with_runner(config, sealed_copy, sealed_fixtures)


def _ratio(value: float | None) -> str:
    return "n/a" if value is None else f"{value:.3f}x"


def _workload_label(identity: dict[str, Any]) -> str:
    state_machine = identity["default_state_machine_id"]
    scene = (
        "static"
        if state_machine is None
        else f"state machine {state_machine}"
    )
    return f"{scene}; VM {identity['view_model_initialization']}"


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
        "| Fixture | Sample | Workload identity | Size | Wasm median (CV) | Native Rust median (CV) | Wasm/native | Advance | Draw |",
        "| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for row in report["fixtures"]:
        wasm = row["wasm"]["elapsed_ms"]
        native = row["native_rust"]["elapsed_ms"]
        lines.append(
            f"| `{row['fixture_id']}` | `{row['sample_seconds']:.9g}s` | {_workload_label(row['workload_identity'])} | {row['bytes']:,} | {wasm['median']:.3f} ms ({wasm['coefficient_of_variation']:.1%}) "
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
    provenance = load_run_seal(args.seal, args.expected_seal_sha256)
    if not provenance.get("artifacts") or not provenance.get("fixtures"):
        raise ContractError("wasm perf config omitted sealed run provenance")
    if config.get("provenance") != provenance:
        raise ContractError("config provenance differs from anchored seal")
    if browser.get("seal_sha256") != args.expected_seal_sha256:
        raise ContractError("browser results differ from anchored seal")
    allowed_outputs = [path.resolve() for path in args.allowed_output]
    verify_run_provenance(
        provenance,
        args.repo_root.resolve(),
        args.rive_runtime_dir.resolve(),
        allowed_outputs=allowed_outputs,
    )
    verify_config_against_seal(config, provenance)
    verify_browser_artifact_identities(provenance["artifacts"], browser)
    verify_browser_coordinator_identity(provenance["artifacts"], browser)
    verify_browser_fixture_identities(provenance["fixtures"], browser)
    verify_browser_measurement_contract(provenance, browser)
    expected_native = Path(provenance["artifacts"]["native_runner"]["path"])
    if args.native_runner.resolve() != expected_native:
        raise ContractError(
            f"native runner differs from sealed artifact: {args.native_runner}"
        )
    for fixture in config["fixtures"]:
        runs = browser.get("fixtures", {}).get(fixture["id"], [])
        if len(runs) != config["runs"]:
            raise ContractError(f"browser run count mismatch for {fixture['id']}")
        for report in runs:
            validate_timing_report(report)
            if report.get("target_identity") != _target_identity(fixture):
                raise ContractError(
                    f"browser sample target identity mismatch for {fixture['id']}"
                )
            if report["segments"] != config["repeat"]:
                raise ContractError(f"browser segment mismatch for {fixture['id']}")
    native = _native_runs(
        config,
        args.native_runner,
        provenance["fixtures"],
        provenance["artifacts"]["native_runner"],
    )
    if load_run_seal(args.seal, args.expected_seal_sha256) != provenance:
        raise ContractError("run seal changed during measurement")
    verify_run_provenance(
        provenance,
        args.repo_root.resolve(),
        args.rive_runtime_dir.resolve(),
        allowed_outputs=allowed_outputs,
    )
    identity = {
        **sealed_config_identity(provenance),
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
    prepare.add_argument("--allowed-output", type=Path, action="append", default=[])
    prepare.set_defaults(action=prepare_run)

    audit = subparsers.add_parser("audit")
    audit.add_argument("--repo-root", type=Path, required=True)
    audit.add_argument("--cargo", type=Path, required=True)
    audit.add_argument("--source", type=Path, required=True)
    audit.add_argument("--shell-source", type=Path, required=True)
    audit.set_defaults(action=audit_run)

    stage_coordinator = subparsers.add_parser("stage-coordinator")
    stage_coordinator.add_argument("--repo-root", type=Path, required=True)
    stage_coordinator.add_argument("--expected-repo-sha", required=True)
    stage_coordinator.add_argument("--expected-repo-tree-sha", required=True)
    stage_coordinator.add_argument("--output-root", type=Path, required=True)
    stage_coordinator.add_argument(
        "--coordinator", action="append", required=True, default=[]
    )
    stage_coordinator.set_defaults(action=stage_coordinator_run)

    seal = subparsers.add_parser("seal")
    seal.add_argument("--config", type=Path, required=True)
    seal.add_argument("--seal", type=Path, required=True)
    seal.add_argument("--repo-root", type=Path, required=True)
    seal.add_argument("--rive-runtime-dir", type=Path, required=True)
    seal.add_argument("--native-runner", type=Path, required=True)
    seal.add_argument("--wasm-artifact", type=Path, required=True)
    seal.add_argument("--wasm-bindgen-js", type=Path, required=True)
    seal.add_argument("--wasm-perf-driver-js", type=Path, required=True)
    seal.add_argument("--wasm-perf-html", type=Path, required=True)
    seal.add_argument("--wasm-perf-node", type=Path, required=True)
    seal.add_argument("--wasm-perf-python", type=Path, required=True)
    seal.add_argument("--wasm-perf-shell", type=Path, required=True)
    seal.add_argument("--allowed-output", type=Path, action="append", default=[])
    seal.set_defaults(action=seal_run)

    finalize = subparsers.add_parser("finalize")
    finalize.add_argument("--config", type=Path, required=True)
    finalize.add_argument("--seal", type=Path, required=True)
    finalize.add_argument("--expected-seal-sha256", required=True)
    finalize.add_argument("--browser-results", type=Path, required=True)
    finalize.add_argument("--native-runner", type=Path, required=True)
    finalize.add_argument("--repo-root", type=Path, required=True)
    finalize.add_argument("--rive-runtime-dir", type=Path, required=True)
    finalize.add_argument("--output", type=Path, required=True)
    finalize.add_argument("--markdown", type=Path)
    finalize.add_argument("--allowed-output", type=Path, action="append", default=[])
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
