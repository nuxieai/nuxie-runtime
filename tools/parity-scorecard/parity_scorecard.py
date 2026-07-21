#!/usr/bin/env python3
"""Build the Nuxie parity scorecard from recorded gate evidence."""

from __future__ import annotations

import argparse
import json
import math
import re
import subprocess
import sys
import tempfile
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


EVIDENCE_SCHEMA = "nuxie-parity-gate-evidence-v1"
REPORT_SCHEMA = "nuxie-parity-scorecard-v1"

GOLDEN_SUMMARY = re.compile(
    r"^golden-compare summary: entries=(?P<entries>\d+) "
    r"exact=(?P<exact>\d+) exact-segments=(?P<exact_segments>\d+) "
    r"diverges=(?P<diverges>\d+) unsupported-feature=(?P<unsupported>\d+) "
    r"not-yet=(?P<not_yet>\d+)$",
    re.MULTILINE,
)
RENDERER_SUMMARY = re.compile(
    r"^renderer-corpus exact=(?P<exact>\d+) byte-exact=(?P<byte_exact>\d+) "
    r"diverges=(?P<diverges>\d+) gated=(?P<gated>\d+) total=(?P<total>\d+)$",
    re.MULTILINE,
)


@dataclass(frozen=True)
class Evidence:
    gate: str
    source_sha: str
    exit_code: int
    output: str


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    check = subparsers.add_parser("check", help="validate evidence and print the scorecard")
    check.add_argument("--repo-root", type=Path, default=Path.cwd())
    check.add_argument("--evidence-dir", type=Path)
    check.add_argument("--source-sha")
    check.add_argument("--json", type=Path, dest="json_output")

    record = subparsers.add_parser(
        "record", help="run one gate and write its output plus exit status as evidence"
    )
    record.add_argument("--gate", required=True)
    record.add_argument("--output", required=True, type=Path)
    record.add_argument("--source-sha")
    record.add_argument("gate_command", nargs=argparse.REMAINDER)

    options = parser.parse_args(argv)
    if options.command == "check":
        return check_scorecard(options)
    if options.command == "record":
        return record_evidence(options)
    parser.error(f"unsupported command {options.command}")
    return 2


def record_evidence(options: argparse.Namespace) -> int:
    command = list(options.gate_command)
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        print("parity-scorecard record requires a command after --", file=sys.stderr)
        return 2

    source_errors: list[str] = []
    source_sha = options.source_sha or git_source_sha(Path.cwd(), source_errors)
    if source_errors:
        print(f"parity-scorecard error: {source_errors[0]}", file=sys.stderr)
        return 2

    captured: list[str] = []
    try:
        process = subprocess.Popen(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            encoding="utf-8",
            errors="replace",
            bufsize=1,
        )
    except OSError as error:
        line = f"failed to launch gate command {command[0]!r}: {error}\n"
        print(line, end="", file=sys.stderr)
        captured.append(line)
        exit_code = 127
    else:
        assert process.stdout is not None
        for line in process.stdout:
            print(line, end="")
            sys.stdout.flush()
            captured.append(line)
        exit_code = process.wait()

    document = {
        "schema": EVIDENCE_SCHEMA,
        "gate": options.gate,
        "source_sha": source_sha,
        "exit_code": exit_code,
        "command": command,
        "output": "".join(captured),
    }
    write_json_atomic(options.output, document)
    return exit_code


def write_json_atomic(path: Path, document: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        "w", encoding="utf-8", dir=path.parent, prefix=f".{path.name}.", delete=False
    ) as temporary:
        temporary.write(json.dumps(document, indent=2, sort_keys=True) + "\n")
        temporary_path = Path(temporary.name)
    temporary_path.replace(path)


def check_scorecard(options: argparse.Namespace) -> int:
    repo_root = options.repo_root.resolve()
    evidence_dir = options.evidence_dir or (
        repo_root / "target" / "parity-scorecard" / "evidence"
    )
    errors: list[str] = []
    source_sha = options.source_sha or git_source_sha(repo_root, errors)
    definition = load_toml(repo_root / "parity-scorecard.toml", "scorecard definition", errors)
    corpus = load_toml(repo_root / "corpus.toml", "runtime corpus", errors)
    renderer_corpus = load_toml(repo_root / "corpus-r.toml", "renderer corpus", errors)

    expected_entries, expected_segments = runtime_ratchet(corpus, errors)
    expected_pixels = renderer_ratchet(renderer_corpus, errors)

    golden = validate_golden_evidence(
        evidence_dir / "golden-compare.json",
        "golden-compare",
        source_sha,
        expected_entries,
        expected_segments,
        errors,
    )
    scripted = validate_golden_evidence(
        evidence_dir / "scripted-golden-compare.json",
        "scripted-golden-compare",
        source_sha,
        expected_entries,
        expected_segments,
        errors,
    )
    renderer = validate_renderer_evidence(
        evidence_dir / "renderer-golden.json",
        source_sha,
        expected_pixels,
        errors,
    )

    tiers = build_tiers(
        definition,
        expected_segments,
        expected_pixels,
        golden,
        scripted,
        renderer,
        repo_root,
        source_sha,
        errors,
    )
    tiers_green = sum(tier["state"] == "GREEN" for tier in tiers)
    report = {
        "schema": REPORT_SCHEMA,
        "source_sha": source_sha,
        "tiers_green": tiers_green,
        "tiers_total": 5,
        "evidence_valid": not errors,
        "tiers": tiers,
        "errors": errors,
    }
    rendered = render_markdown(report)
    print(rendered, end="")

    if options.json_output:
        options.json_output.parent.mkdir(parents=True, exist_ok=True)
        options.json_output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")

    if errors:
        for error in errors:
            print(f"parity-scorecard error: {error}", file=sys.stderr)
        return 1
    return 0


def load_toml(path: Path, label: str, errors: list[str]) -> dict[str, Any]:
    try:
        with path.open("rb") as source:
            document = tomllib.load(source)
    except (OSError, tomllib.TOMLDecodeError) as error:
        errors.append(f"cannot read {label} {path}: {error}")
        return {}
    if not isinstance(document, dict):
        errors.append(f"{label} {path} must contain a TOML table")
        return {}
    return document


def git_source_sha(repo_root: Path, errors: list[str]) -> str:
    completed = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=repo_root,
        text=True,
        capture_output=True,
    )
    if completed.returncode != 0:
        errors.append(f"cannot resolve source commit for {repo_root}")
        return "unknown"
    return completed.stdout.strip()


def runtime_ratchet(corpus: dict[str, Any], errors: list[str]) -> tuple[int, int]:
    rows = corpus.get("file")
    if not isinstance(rows, list) or not rows:
        errors.append("runtime corpus must contain at least one [[file]] row")
        return 0, 0
    exact_rows = [row for row in rows if isinstance(row, dict) and row.get("status") == "exact"]
    if len(exact_rows) != len(rows):
        errors.append(
            "runtime corpus contains a non-exact row; "
            "the completed floor requires all rows exact"
        )
    segments = 0
    for row in exact_rows:
        samples = row.get("samples")
        if not isinstance(samples, list) or not samples:
            errors.append(f"runtime corpus row {row.get('id', '<unknown>')} has no samples")
            continue
        if row.get("verification") != "rejects-malformed":
            segments += len(samples)
    return len(rows), segments


def renderer_ratchet(corpus: dict[str, Any], errors: list[str]) -> int:
    rows = corpus.get("entry")
    if not isinstance(rows, list) or not rows:
        errors.append("renderer corpus must contain at least one [[entry]] row")
        return 0
    exact = sum(
        isinstance(row, dict) and row.get("status") == "exact" for row in rows
    )
    if exact != len(rows):
        errors.append(
            "renderer corpus contains a non-exact row; "
            "the completed floor requires all rows exact"
        )
    return len(rows)


def read_evidence(path: Path, expected_gate: str, errors: list[str]) -> Evidence | None:
    try:
        document = json.loads(path.read_text())
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        errors.append(f"required {expected_gate} evidence is unavailable at {path}: {error}")
        return None
    if not isinstance(document, dict) or document.get("schema") != EVIDENCE_SCHEMA:
        errors.append(f"{expected_gate} evidence has an unsupported schema")
        return None
    gate = document.get("gate")
    source_sha = document.get("source_sha")
    exit_code = document.get("exit_code")
    output = document.get("output")
    if gate != expected_gate:
        errors.append(f"expected {expected_gate} evidence, found {gate!r}")
        return None
    if not isinstance(source_sha, str) or not source_sha:
        errors.append(f"{expected_gate} evidence has no source_sha")
        return None
    if not isinstance(exit_code, int) or isinstance(exit_code, bool):
        errors.append(f"{expected_gate} evidence has no integer exit_code")
        return None
    if not isinstance(output, str):
        errors.append(f"{expected_gate} evidence has no output text")
        return None
    return Evidence(gate, source_sha, exit_code, output)


def validate_evidence_identity(
    evidence: Evidence, source_sha: str, errors: list[str]
) -> bool:
    valid = True
    if evidence.source_sha != source_sha:
        errors.append(
            f"{evidence.gate} evidence is stale: expected {source_sha}, got {evidence.source_sha}"
        )
        valid = False
    if evidence.exit_code != 0:
        errors.append(f"{evidence.gate} gate exited {evidence.exit_code}")
        valid = False
    return valid


def one_summary(
    pattern: re.Pattern[str], evidence: Evidence, errors: list[str]
) -> dict[str, int] | None:
    matches = list(pattern.finditer(evidence.output))
    if len(matches) != 1:
        errors.append(
            f"{evidence.gate} evidence must contain exactly one summary line; found {len(matches)}"
        )
        return None
    return {key: int(value) for key, value in matches[0].groupdict().items()}


def validate_golden_evidence(
    path: Path,
    gate: str,
    source_sha: str,
    expected_entries: int,
    expected_segments: int,
    errors: list[str],
) -> bool:
    evidence = read_evidence(path, gate, errors)
    if evidence is None:
        return False
    valid = validate_evidence_identity(evidence, source_sha, errors)
    summary = one_summary(GOLDEN_SUMMARY, evidence, errors)
    if summary is None:
        return False
    expected = {
        "entries": expected_entries,
        "exact": expected_entries,
        "exact_segments": expected_segments,
        "diverges": 0,
        "unsupported": 0,
        "not_yet": 0,
    }
    if summary != expected:
        errors.append(f"{gate} ratchet mismatch: expected {expected}, got {summary}")
        valid = False
    return valid


def validate_renderer_evidence(
    path: Path,
    source_sha: str,
    expected_pixels: int,
    errors: list[str],
) -> bool:
    gate = "renderer-golden"
    evidence = read_evidence(path, gate, errors)
    if evidence is None:
        return False
    valid = validate_evidence_identity(evidence, source_sha, errors)
    summary = one_summary(RENDERER_SUMMARY, evidence, errors)
    if summary is None:
        return False
    expected = {
        "exact": expected_pixels,
        "byte_exact": summary["byte_exact"],
        "diverges": 0,
        "gated": 0,
        "total": expected_pixels,
    }
    if summary != expected or summary["byte_exact"] > expected_pixels:
        errors.append(f"renderer-golden ratchet mismatch: expected {expected}, got {summary}")
        valid = False
    return valid


def build_tiers(
    definition: dict[str, Any],
    expected_segments: int,
    expected_pixels: int,
    golden: bool,
    scripted: bool,
    renderer: bool,
    repo_root: Path,
    source_sha: str,
    errors: list[str],
) -> list[dict[str, Any]]:
    if definition.get("schema_version") != 1:
        errors.append("scorecard definition schema_version must be 1")
    sdk = definition.get("sdk", {})
    sdk_rows = string_list(sdk.get("rows"), "sdk.rows", errors)
    sdk_closed = string_list(sdk.get("closed"), "sdk.closed", errors)
    register_rows = sdk_rows_from_register(
        repo_root / "docs" / "parity-gap-register.md", errors
    )
    if sdk_rows != register_rows:
        errors.append(
            "sdk.rows must match the register A-row checklist exactly: "
            f"expected {register_rows}, got {sdk_rows}"
        )
    unknown_closed = sorted(set(sdk_closed) - set(register_rows))
    if unknown_closed:
        errors.append(f"sdk.closed contains unknown rows: {', '.join(unknown_closed)}")

    platform = definition.get("platform", {})
    adapters = string_list(
        platform.get("verified_adapters"), "platform.verified_adapters", errors
    )
    required_adapters = platform.get("required_adapters")
    if (
        not isinstance(required_adapters, int)
        or isinstance(required_adapters, bool)
        or required_adapters < 2
    ):
        errors.append("platform.required_adapters must be at least 2")
        required_adapters = 2

    minimum_perf_entries, maximum_perf_ratio = performance_requirements(
        definition, errors
    )
    perf = provisional_perf(
        repo_root,
        source_sha,
        minimum_perf_entries,
        maximum_perf_ratio,
        errors,
    )
    tiers = [
        tier(
            1,
            "Frame parity",
            [
                ratchet(
                    "exact-segments",
                    "GREEN" if golden else "RED",
                    f"exact-segments {expected_segments}/{expected_segments}"
                    if golden
                    else "exact-segments unavailable/red",
                ),
                ratchet(
                    "scripted-exact-segments",
                    "GREEN" if scripted else "RED",
                    f"scripted {expected_segments}/{expected_segments}"
                    if scripted
                    else "scripted unavailable/red",
                ),
                ratchet("e2e-exact", "NOT_BUILT", "e2e-exact not built (#OR-6)"),
            ],
        ),
        tier(
            2,
            "Interaction parity",
            [
                ratchet(
                    "side-channel-segments",
                    "NOT_BUILT",
                    "side-channel-segments not built (#OR-1/#OR-2)",
                ),
                ratchet(
                    "script-verbs",
                    "NOT_BUILT",
                    "script verbs not built (#OR-3)",
                ),
                ratchet(
                    "sampling-density",
                    "NOT_BUILT",
                    "sampling densification not built (#OR-4)",
                ),
                ratchet(
                    "input-script-coverage",
                    "NOT_BUILT",
                    "input-script coverage not built (#OR-5)",
                ),
                ratchet(
                    "fuzz-clean-nights",
                    "NOT_BUILT",
                    "fuzz-clean-nights not built (#OR-7)",
                ),
            ],
        ),
        tier(
            3,
            "SDK parity",
            [
                ratchet(
                    "a-rows-closed",
                    "GREEN"
                    if register_rows and len(sdk_closed) == len(register_rows)
                    else "RED",
                    sdk_display(register_rows, sdk_closed),
                )
            ],
        ),
        tier(
            4,
            "Platform parity",
            [
                ratchet(
                    "pixel-exact",
                    "GREEN" if renderer else "RED",
                    f"pixel-exact {expected_pixels}/{expected_pixels}"
                    if renderer
                    else "pixel-exact unavailable/red",
                ),
                ratchet(
                    "verified-adapters",
                    "GREEN" if len(adapters) >= required_adapters else "RED",
                    f"adapters {len(adapters)}/{required_adapters}",
                ),
                ratchet(
                    "webgl2-decision",
                    "NOT_BUILT",
                    "WebGL2 decision not built (#HD-3)",
                ),
            ],
        ),
        tier(
            5,
            "Performance & size",
            [
                perf,
                ratchet("size-mib", "NOT_BUILT", "size MiB not built (#B-3)"),
            ],
        ),
    ]
    return tiers


def sdk_display(register_rows: list[str], closed_rows: list[str]) -> str:
    open_rows = [row for row in register_rows if row not in closed_rows]
    display = f"A-rows closed {len(closed_rows)}/{len(register_rows)}"
    if open_rows:
        display += f" (open: {','.join(open_rows)})"
    return display


def sdk_rows_from_register(path: Path, errors: list[str]) -> list[str]:
    try:
        register = path.read_text()
    except (OSError, UnicodeError) as error:
        errors.append(f"cannot read SDK checklist from {path}: {error}")
        return []
    section = re.search(r"^## A\b.*?(?=^##\s|\Z)", register, re.MULTILINE | re.DOTALL)
    if section is None:
        errors.append(f"SDK checklist section is missing from {path}")
        return []
    rows = re.findall(r"^\|\s*(A\d+)\s*\|", section.group(0), re.MULTILINE)
    if not rows:
        errors.append(f"SDK checklist contains no A-rows in {path}")
        return []
    if len(set(rows)) != len(rows):
        errors.append(f"SDK checklist contains duplicate A-row ids in {path}")
    return rows


def string_list(value: Any, label: str, errors: list[str]) -> list[str]:
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        errors.append(f"{label} must be a list of strings")
        return []
    if len(set(value)) != len(value):
        errors.append(f"{label} must not contain duplicates")
    return value


def provisional_perf(
    repo_root: Path,
    source_sha: str,
    minimum: int,
    maximum: float,
    errors: list[str],
) -> dict[str, Any]:
    perf_path = repo_root / "target" / "perf-compare.json"
    if not perf_path.is_file():
        return ratchet(
            "runtime-ratio",
            "NOT_BUILT",
            "runtime ratio not built (#OR-9; provisional evidence unavailable)",
        )
    try:
        document = json.loads(perf_path.read_text())
        aggregate = document["aggregate"]
        ratio = aggregate["rust_over_cpp"]
        entries = aggregate["entries"]
        report_sha = document["meta"]["git_sha"]
    except (OSError, UnicodeError, json.JSONDecodeError, KeyError, TypeError) as error:
        errors.append(f"malformed provisional perf evidence {perf_path}: {error}")
        return ratchet("runtime-ratio", "RED", "runtime ratio malformed")
    if (
        not isinstance(ratio, (int, float))
        or isinstance(ratio, bool)
        or not math.isfinite(ratio)
        or ratio <= 0
        or not isinstance(entries, int)
        or isinstance(entries, bool)
        or entries < 1
        or report_sha != source_sha
    ):
        errors.append(f"invalid or stale provisional perf evidence {perf_path}")
        return ratchet("runtime-ratio", "RED", "runtime ratio invalid/stale")
    state = "PARTIAL" if ratio <= maximum and entries < minimum else "RED"
    if ratio <= maximum and entries >= minimum:
        # #OR-9 must still make this CI lane blocking before it can be GREEN.
        state = "PARTIAL"
    return ratchet(
        "runtime-ratio",
        state,
        f"runtime ratio {ratio:.3f} over {entries}/{minimum} files (non-blocking; #OR-9)",
        value=ratio,
        target=maximum,
    )


def performance_requirements(
    definition: dict[str, Any], errors: list[str]
) -> tuple[int, float]:
    performance = definition.get("performance", {})
    minimum = performance.get("blocking_min_entries")
    maximum = performance.get("max_ratio")
    if (
        not isinstance(minimum, int)
        or isinstance(minimum, bool)
        or minimum < 20
    ):
        errors.append("performance.blocking_min_entries must be at least 20")
        minimum = 20
    if (
        not isinstance(maximum, (int, float))
        or isinstance(maximum, bool)
        or not math.isfinite(maximum)
        or maximum <= 0
        or maximum > 1.0
    ):
        errors.append("performance.max_ratio must be at most 1.0 and greater than 0")
        maximum = 1.0
    return minimum, float(maximum)


def ratchet(
    ratchet_id: str,
    state: str,
    display: str,
    *,
    value: int | float | None = None,
    target: int | float | None = None,
) -> dict[str, Any]:
    result: dict[str, Any] = {"id": ratchet_id, "state": state, "display": display}
    if value is not None:
        result["value"] = value
    if target is not None:
        result["target"] = target
    return result


def tier(tier_id: int, name: str, ratchets: list[dict[str, Any]]) -> dict[str, Any]:
    states = {ratchet["state"] for ratchet in ratchets}
    if states == {"GREEN"}:
        state = "GREEN"
    elif "GREEN" in states or "PARTIAL" in states:
        state = "PARTIAL"
    else:
        state = "RED"
    return {"id": tier_id, "name": name, "state": state, "ratchets": ratchets}


def render_markdown(report: dict[str, Any]) -> str:
    lines = [
        "# Nuxie parity scorecard",
        "",
        f"source-sha: {report['source_sha']}",
        f"tiers-green: {report['tiers_green']}/{report['tiers_total']}",
        "",
        "| tier | state | ratchets |",
        "|---|---|---|",
    ]
    for item in report["tiers"]:
        details = "; ".join(ratchet["display"] for ratchet in item["ratchets"])
        lines.append(f"| {item['id']} {item['name']} | {item['state']} | {details} |")
    lines.append("")
    if report["errors"]:
        lines.append("Evidence errors: " + "; ".join(report["errors"]))
        lines.append("")
    return "\n".join(lines)


if __name__ == "__main__":
    raise SystemExit(main())
