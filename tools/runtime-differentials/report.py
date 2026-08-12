#!/usr/bin/env python3
"""Emit deterministic provenance for validated runtime differential lanes."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import tempfile
import tomllib
from collections import Counter
from pathlib import Path


SCHEMA = "nuxie-runtime-differentials/v1"


class ReportError(Exception):
    pass


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    try:
        with open(path, "rb") as handle:
            for block in iter(lambda: handle.read(1 << 20), b""):
                digest.update(block)
    except OSError as error:
        raise ReportError(f"cannot fingerprint {path}: {error}") from error
    return digest.hexdigest()


def validate_commit(name: str, value: str) -> None:
    if len(value) != 40 or any(character not in "0123456789abcdef" for character in value):
        raise ReportError(f"{name} must be a full lowercase 40-character commit")


def validate_git_ref(directory: Path, expected: str, label: str) -> None:
    result = subprocess.run(
        ["git", "-C", str(directory), "rev-parse", "HEAD"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    actual = result.stdout.strip()
    if result.returncode != 0 or actual != expected:
        shown = actual or "not-a-git-checkout"
        raise ReportError(f"{label} checkout is {shown}, expected {expected}")


def runner_records(
    runners: list[tuple[str, Path]], allow_missing: bool = False
) -> list[dict]:
    records = []
    roles = set()
    for role, path in runners:
        if role in roles:
            raise ReportError(f"duplicate runner role {role}")
        roles.add(role)
        if allow_missing and not path.is_file():
            records.append(
                {"role": role, "path": str(path), "sha256": None, "missing": True}
            )
        else:
            records.append({"role": role, "path": str(path), "sha256": sha256(path)})
    return records


def fixture_record(label: str, path: Path) -> dict:
    return {"path": label, "sha256": sha256(path)}


def optional_fixture_record(label: str, path: Path) -> dict:
    if path.is_file():
        return fixture_record(label, path)
    return {"path": label, "sha256": None, "missing": True}


def virtual_fixture_record(label: str) -> dict:
    return {"path": label, "sha256": None, "virtual": True}


def valid_signature(signature: object) -> bool:
    return (
        isinstance(signature, str)
        and 0 < len(signature) <= 600
        and "\n" not in signature
        and "\r" not in signature
        and (
            signature.startswith("line ")
            or signature.startswith("stream newline termination differs")
            or signature.startswith("frame ")
        )
    )


def outcome(status: str) -> str:
    return {
        "exact": "exact",
        "diverges": "divergent",
        "unsupported-feature": "unsupported",
        "provenance-unknown": "unsupported",
        "not-yet": "pending",
        "pending": "pending",
        "pending-scripted": "pending",
    }[status]


def base_report(
    lane: str,
    cpp_ref: str,
    rust_commit: str,
    manifest: Path,
    runners: list[tuple[str, Path]],
    allow_missing_runners: bool = False,
) -> dict:
    validate_commit("cpp ref", cpp_ref)
    validate_commit("Rust commit", rust_commit)
    return {
        "schema": SCHEMA,
        "lane": lane,
        "cpp_ref": cpp_ref,
        "rust_commit": rust_commit,
        "gate_status": "passed",
        "manifest": {"path": str(manifest), "sha256": sha256(manifest)},
        "runners": runner_records(runners, allow_missing=allow_missing_runners),
    }


def build_golden_report(
    manifest: Path,
    runtime_dir: Path,
    repo_root: Path,
    mode: str,
    cpp_ref: str,
    rust_commit: str,
    runners: list[tuple[str, Path]],
    allow_missing_runners: bool = False,
) -> dict:
    if mode not in {"ordinary", "scripted"}:
        raise ReportError(f"invalid golden mode {mode}")
    try:
        parsed = tomllib.loads(manifest.read_text())
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise ReportError(f"cannot read golden manifest {manifest}: {error}") from error

    payload = base_report(
        f"golden-{mode}",
        cpp_ref,
        rust_commit,
        manifest,
        runners,
        allow_missing_runners,
    )
    cases = []
    counts = Counter()
    for entry in parsed.get("file", []):
        features = entry.get("features", [])
        declared = entry.get("status", "not-yet")
        scripted_override = mode == "scripted" and "scripted-status:diverges" in features
        if mode == "scripted" and "scripted-status:exact" in features:
            declared = "exact"
        elif scripted_override:
            declared = "diverges"
        try:
            classified = outcome(declared)
        except KeyError as error:
            raise ReportError(f"{entry.get('id', '<unknown>')} has unknown status {declared}") from error

        signature = None
        if classified == "divergent":
            field = "scripted_divergence_signature" if scripted_override else "divergence_signature"
            signature = entry.get(field)
            if not valid_signature(signature):
                raise ReportError(
                    f"{entry.get('id', '<unknown>')} has malformed divergence signature in {field}"
                )

        label = entry.get("path")
        if not isinstance(label, str):
            raise ReportError(f"{entry.get('id', '<unknown>')} has no fixture path")
        fixture_path = Path(label)
        if not fixture_path.is_absolute():
            fixture_path = (
                repo_root / fixture_path
                if fixture_path.parts and fixture_path.parts[0] == "fixtures"
                else runtime_dir / fixture_path
            )
        record = {
            "id": entry.get("id"),
            "declared_status": declared,
            "outcome": classified,
            "executed": (
                classified == "exact"
                and (
                    mode == "scripted"
                    or not (
                        entry.get("rust_execute_scripts", False)
                        or "scripted-runner-only" in features
                    )
                )
            )
            or (mode == "scripted" and classified == "divergent"),
            "verification": entry.get("verification", "exact"),
            "fixture": fixture_record(label, fixture_path),
        }
        for script_field in ("input_script", "view_model_script"):
            script_label = entry.get(script_field)
            if script_label is None:
                continue
            if not isinstance(script_label, str):
                raise ReportError(
                    f"{entry.get('id', '<unknown>')} has malformed {script_field}"
                )
            script_path = Path(script_label)
            if not script_path.is_absolute():
                script_path = manifest.parent / script_path
            record[script_field] = fixture_record(script_label, script_path)
        if signature is not None:
            record["signature"] = signature
        cases.append(record)
        counts[classified] += 1

    if not cases:
        raise ReportError("golden manifest has no file entries")
    payload["summary"] = dict(sorted(counts.items()))
    payload["cases"] = cases
    return payload


def silver_signature(entry: dict) -> str:
    note = entry.get("note")
    marker = "first difference: "
    if not isinstance(note, str) or marker not in note:
        raise ReportError(f"{entry.get('id', '<unknown>')} has no recorded first difference")
    signature = note.split(marker, 1)[1]
    if signature.endswith("."):
        signature = signature[:-1]
    if not valid_signature(signature):
        raise ReportError(f"{entry.get('id', '<unknown>')} has malformed divergence signature")
    return signature


def build_silver_report(
    manifest: Path,
    runtime_dir: Path,
    rust_commit: str,
    runners: list[tuple[str, Path]],
    allow_missing_runners: bool = False,
) -> dict:
    try:
        parsed = tomllib.loads(manifest.read_text())
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise ReportError(f"cannot read silver manifest {manifest}: {error}") from error
    cpp_ref = parsed.get("corpus", {}).get("upstream_ref")
    if not isinstance(cpp_ref, str):
        raise ReportError("silver manifest has no upstream_ref")
    payload = base_report(
        "silver", cpp_ref, rust_commit, manifest, runners, allow_missing_runners
    )
    cases = []
    counts = Counter()
    for entry in parsed.get("case", []):
        declared = entry.get("status")
        try:
            classified = outcome(declared)
        except KeyError as error:
            raise ReportError(f"{entry.get('id', '<unknown>')} has unknown status {declared}") from error
        source = entry.get("source")
        expected = entry.get("expected")
        if not isinstance(source, str) or not isinstance(expected, str):
            raise ReportError(f"{entry.get('id', '<unknown>')} has incomplete fixture identity")
        signature = silver_signature(entry) if classified == "divergent" else None
        dependencies = entry.get("dependencies", [])
        if not isinstance(dependencies, list) or any(
            not isinstance(dependency, str) for dependency in dependencies
        ):
            raise ReportError(f"{entry.get('id', '<unknown>')} has malformed dependencies")
        action_fixtures = []
        actions = entry.get("actions", [])
        if isinstance(actions, list):
            for action in actions:
                if not isinstance(action, dict):
                    raise ReportError(f"{entry.get('id', '<unknown>')} has a malformed action")
                if action.get("kind") == "set-view-model-font-bytes":
                    action_source = action.get("source")
                    if not isinstance(action_source, str):
                        raise ReportError(
                            f"{entry.get('id', '<unknown>')} has a font action without a source"
                        )
                    action_fixtures.append(action_source)
        record = {
            "id": entry.get("id"),
            "lane": entry.get("lane"),
            "declared_status": declared,
            "outcome": classified,
            "executed": entry.get("lane") == "runtime" and classified in {"exact", "divergent"},
            "verification": entry.get("verification"),
            "fixture": (
                optional_fixture_record(source, runtime_dir / "tests/unit_tests/assets" / source)
                if declared == "provenance-unknown"
                else virtual_fixture_record(source)
                if entry.get("lane") == "scripted" and source == "inline-script"
                else fixture_record(source, runtime_dir / "tests/unit_tests/assets" / source)
            ),
            "dependencies": [
                fixture_record(dependency, runtime_dir / "tests/unit_tests/assets" / dependency)
                for dependency in dependencies
            ],
            "action_fixtures": [
                fixture_record(action_fixture, runtime_dir / "tests/unit_tests/assets" / action_fixture)
                for action_fixture in action_fixtures
            ],
            "baseline": fixture_record(expected, runtime_dir / expected),
        }
        if signature is not None:
            record["signature"] = signature
        cases.append(record)
        counts[classified] += 1

    if not cases:
        raise ReportError("silver manifest has no case entries")
    payload["summary"] = dict(sorted(counts.items()))
    payload["cases"] = cases
    return payload


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    handle = tempfile.NamedTemporaryFile(
        "w", encoding="utf-8", dir=path.parent, prefix=path.name, suffix=".tmp", delete=False
    )
    try:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")
        handle.close()
        os.replace(handle.name, path)
    finally:
        if os.path.exists(handle.name):
            os.unlink(handle.name)


def apply_diagnostics(payload: dict, diagnostics: str, gate_rc: int) -> None:
    payload["gate_status"] = "passed" if gate_rc == 0 else "failed"
    by_id = {case["id"]: case for case in payload["cases"]}

    # A successful gate completed every case selected by the lane, while a
    # failed fail-fast gate only completed cases named in its streamed log.
    # Reset first so a report can never claim that cases after the first
    # failure ran.
    if gate_rc != 0:
        for case in payload["cases"]:
            case["executed"] = False
        if payload.get("lane") == "silver":
            observed_patterns = (
                re.compile(
                    r"^\[(?:exact|epsilon|divergent)\] ([A-Za-z0-9_.-]+):",
                    re.MULTILINE,
                ),
            )
        else:
            observed_patterns = (
                re.compile(
                    r"^\[exact\] ([A-Za-z0-9_.-]+): rust comparison verified$",
                    re.MULTILINE,
                ),
                re.compile(
                    r"^\[exact\] ([A-Za-z0-9_.-]+): rust rejected the same malformed import as expected$",
                    re.MULTILINE,
                ),
                re.compile(
                    r"^\[diverges\] ([A-Za-z0-9_.-]+): signature verified$",
                    re.MULTILINE,
                ),
            )
        for pattern in observed_patterns:
            for match in pattern.finditer(diagnostics):
                if case := by_id.get(match.group(1)):
                    case["executed"] = True

    newly_exact_patterns = (
        re.compile(
            r"^(?:failure: )?([A-Za-z0-9_.-]+) now compares exact; promote it[^\n]*",
            re.MULTILINE,
        ),
        re.compile(
            r"^(?:silver-corpus error: )?([A-Za-z0-9_.-]+) is classified diverges but now compares exact; promote it[^\n]*",
            re.MULTILINE,
        ),
    )
    for pattern in newly_exact_patterns:
        for match in pattern.finditer(diagnostics):
            case_id = match.group(1)
            case = by_id.get(case_id)
            if case is None:
                raise ReportError(f"diagnostics reference unknown case {case_id}")
            case["outcome"] = "newly-exact"
            case["executed"] = True
            case["diagnostic"] = match.group(0)

    exact_regression_patterns = (
        re.compile(
            r"^(?:failure: )?([A-Za-z0-9_.-]+): stream differs from C\+\+ under [^\n]+",
            re.MULTILINE,
        ),
        re.compile(
            r"^(?:silver-corpus error: )?([A-Za-z0-9_.-]+) exact entry diverged: [^\n]+",
            re.MULTILINE,
        ),
    )
    for pattern in exact_regression_patterns:
        for match in pattern.finditer(diagnostics):
            case_id = match.group(1)
            case = by_id.get(case_id)
            if case is None:
                raise ReportError(f"diagnostics reference unknown case {case_id}")
            case["outcome"] = "regressed"
            case["executed"] = True
            case["diagnostic"] = match.group(0)

    changed_pattern = re.compile(
        r"^(?:failure: |silver-corpus error: )?([A-Za-z0-9_.-]+) divergence changed: "
        r"recorded [^\n]+; actual [^\n]+",
        re.MULTILINE,
    )
    for match in changed_pattern.finditer(diagnostics):
        case_id = match.group(1)
        case = by_id.get(case_id)
        if case is None:
            raise ReportError(f"diagnostics reference unknown case {case_id}")
        case["executed"] = True
        case["divergence_check"] = "changed"
        case["diagnostic"] = match.group(0)

    verified_pattern = re.compile(
        r"^(?:\[divergent\] ([A-Za-z0-9_.-]+): [^\n]+|"
        r"\[diverges\] ([A-Za-z0-9_.-]+): signature verified)$",
        re.MULTILINE,
    )
    for match in verified_pattern.finditer(diagnostics):
        case = by_id.get(match.group(1) or match.group(2))
        if (
            case is not None
            and case.get("outcome") == "divergent"
            and case.get("divergence_check") != "changed"
        ):
            case["divergence_check"] = "verified"
    payload["summary"] = dict(
        sorted(Counter(case["outcome"] for case in payload["cases"]).items())
    )


def parse_runner(value: str) -> tuple[str, Path]:
    try:
        role, raw_path = value.split("=", 1)
    except ValueError as error:
        raise argparse.ArgumentTypeError("runner must be ROLE=PATH") from error
    if not role or not raw_path:
        raise argparse.ArgumentTypeError("runner must be ROLE=PATH")
    return role, Path(raw_path)


def main() -> int:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)

    def common(subparser):
        subparser.add_argument("--manifest", type=Path, required=True)
        subparser.add_argument("--runtime-dir", type=Path, required=True)
        subparser.add_argument("--repo-root", type=Path, required=True)
        subparser.add_argument("--rust-commit", required=True)
        subparser.add_argument("--runner", action="append", default=[], type=parse_runner)
        subparser.add_argument("--output", type=Path, required=True)
        subparser.add_argument("--diagnostics", type=Path)
        subparser.add_argument("--gate-rc", type=int, required=True)

    golden = subparsers.add_parser("golden")
    common(golden)
    golden.add_argument("--mode", choices=("ordinary", "scripted"), required=True)
    golden.add_argument("--cpp-ref", required=True)

    silver = subparsers.add_parser("silver")
    common(silver)

    args = parser.parse_args()
    try:
        validate_git_ref(args.repo_root, args.rust_commit, "Rust")
        if args.command == "golden":
            validate_git_ref(args.runtime_dir, args.cpp_ref, "C++ runtime")
            payload = build_golden_report(
                args.manifest,
                args.runtime_dir,
                args.repo_root,
                args.mode,
                args.cpp_ref,
                args.rust_commit,
                args.runner,
                allow_missing_runners=args.gate_rc != 0,
            )
        else:
            payload = build_silver_report(
                args.manifest,
                args.runtime_dir,
                args.rust_commit,
                args.runner,
                allow_missing_runners=args.gate_rc != 0,
            )
            validate_git_ref(args.runtime_dir, payload["cpp_ref"], "C++ runtime")
        diagnostics = ""
        if args.diagnostics is not None:
            try:
                diagnostics = args.diagnostics.read_text()
            except OSError as error:
                raise ReportError(
                    f"cannot read diagnostics {args.diagnostics}: {error}"
                ) from error
        apply_diagnostics(payload, diagnostics, args.gate_rc)
        write_json(args.output, payload)
    except ReportError as error:
        parser.error(str(error))
    print(f"runtime differential report: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
