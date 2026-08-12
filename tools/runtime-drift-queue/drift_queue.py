#!/usr/bin/env python3
"""Build the generated runtime drift investigation queue."""

from __future__ import annotations

import argparse
import collections
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path


SCHEMA = "nuxie-runtime-drift-queue/v1"
KNOWN_DIVERGENCES = {"diverges", "divergent", "known-divergent", "regressed"}
UNSUPPORTED = {"unsupported", "unsupported-feature", "n-a"}
UNKNOWN = {"provenance-unknown", "unknown"}
TRACKED_GAP_ID = re.compile(r"(?:V|F|A|C|H|W)\d+|RB-\d+")
DIFFERENTIAL_LANES = {"golden-ordinary", "golden-scripted", "silver"}
DIFFERENTIAL_OUTCOMES = {
    "exact",
    "divergent",
    "unsupported",
    "pending",
    "newly-exact",
    "regressed",
}
SHA256 = re.compile(r"[0-9a-f]{64}")
COMMIT = re.compile(r"[0-9a-f]{40}")
ARTIFACT_OUTCOME_PRIORITY = {
    "pending": 1,
    "unsupported": 2,
    "diverges": 3,
    "divergent": 3,
    "newly-exact": 4,
    "regressed": 5,
}


def disposition_for(evidence_state: str, source_kind: str) -> str:
    if source_kind == "decision":
        return "intentional-decision"
    if source_kind == "extension":
        return "extension"
    if evidence_state in KNOWN_DIVERGENCES:
        return "known-divergence"
    if evidence_state in UNSUPPORTED:
        return "unsupported"
    if evidence_state in UNKNOWN:
        return "unknown"
    if evidence_state == "stale":
        return "stale-proof"
    return "pending-proof"


def owner_coordinates(upstream_owner: str | None, source_row: str) -> tuple[str, str]:
    path = upstream_owner or source_row
    parts = Path(path).parts
    if len(parts) >= 3 and parts[0] == "src":
        family = parts[1]
    elif len(parts) == 2 and parts[0] == "src":
        family = "runtime-core"
    elif len(parts) >= 3 and parts[0] == "tests":
        family = "runtime-tests"
    else:
        family = "unresolved"
    stem = Path(path).stem
    subsystem = stem.removesuffix("_test") or "unresolved"
    return family, subsystem


def semantic_boundary(evidence_state: str, signal: str) -> str:
    lowered = signal.lower()
    if evidence_state in UNSUPPORTED:
        return "unsupported-observable"
    if re.search(r"\b(rewind|draw|order|sequence|slot)\b", lowered):
        return "ordering"
    if re.search(r"\b(lifecycle|advance|frame|mount|dispose|retain)\b", lowered):
        return "lifecycle"
    if re.search(r"\b(invalidat|cache|dirty|dirt|refresh)\w*", lowered):
        return "invalidation"
    if re.search(r"\b(float|epsilon|gradient|opacity|transform|tx|ty)\b", lowered):
        return "float-behavior"
    if re.search(r"\b(mutatio|property|input|binding|setter|change)\w*", lowered):
        return "mutation"
    return "ownership"


def product_reach_for(upstream_owner: str | None, source_row: str, signal: str) -> dict:
    text = " ".join(filter(None, (upstream_owner, source_row, signal))).lower()
    if re.search(
        r"artboard|layout|state.machine|data.bind|draw|shape|text|input|focus|animation",
        text,
    ):
        level = "high"
    elif re.search(r"renderer|lua|script|asset|audio|image|font", text):
        level = "medium"
    else:
        level = "low"
    return {"level": level, "basis": "deterministic subsystem keyword policy"}


def discovery_value(disposition: str, reach: str, confidence: str) -> int:
    base = {
        "unknown": 100,
        "known-divergence": 90,
        "pending-proof": 80,
        "stale-proof": 70,
        "unsupported": 50,
        "intentional-decision": 30,
        "extension": 25,
    }[disposition]
    return base + {"high": 20, "medium": 10, "low": 0}[reach] + {
        "high": 10,
        "medium": 5,
        "low": 0,
    }[confidence]


def owner_missing_proofs(owner: dict) -> list[str]:
    missing = []
    if owner["mapping"] != "mapped":
        missing.append(f"mapping is {owner['mapping']}")
    if owner["structural"] not in {"isomorphic", "adapted", "not-applicable"}:
        missing.append(f"structural proof is {owner['structural']}")
    if owner["behavioral"] != "behaviorally-proven":
        missing.append(f"behavioral proof is {owner['behavioral']}")
    if owner["verification"] != "orchestrator-verified":
        missing.append(f"verification is {owner['verification']}")
    if owner["freshness"] != "current":
        missing.append(f"freshness is {owner['freshness']}")
    return missing


def read_toml(path: Path) -> dict:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def valid_file_record(
    record: object, *, allow_missing: bool = False, allow_virtual: bool = False
) -> bool:
    if not isinstance(record, dict) or not isinstance(record.get("path"), str):
        return False
    sha = record.get("sha256")
    if isinstance(sha, str) and SHA256.fullmatch(sha):
        return True
    return sha is None and (
        (allow_missing and record.get("missing") is True)
        or (allow_virtual and record.get("virtual") is True)
    )


def validate_differential_report(
    path: Path,
    report: dict,
    repo_root: Path,
    source_rows: dict[str, dict[str, dict]],
) -> str:
    lane = report.get("lane")
    if lane not in DIFFERENTIAL_LANES:
        raise ValueError(f"{path} has unsupported differential lane {lane}")
    if not COMMIT.fullmatch(str(report.get("cpp_ref", ""))) or not COMMIT.fullmatch(
        str(report.get("rust_commit", ""))
    ):
        raise ValueError(f"{path} has malformed commit provenance")
    source_kind = "silver" if lane == "silver" else "golden"
    manifest_name = "silver-corpus.toml" if source_kind == "silver" else "corpus.toml"
    manifest = report.get("manifest")
    expected_manifest = repo_root / manifest_name
    if (
        not valid_file_record(manifest)
        or Path(manifest["path"]).name != manifest_name
        or manifest["sha256"] != file_sha256(expected_manifest)
    ):
        raise ValueError(f"{path} has malformed manifest provenance")
    runners = report.get("runners")
    expected_roles = {"validator"} if source_kind == "silver" else {"cpp", "rust"}
    if (
        not isinstance(runners, list)
        or len(runners) != len(expected_roles)
        or {runner.get("role") for runner in runners if isinstance(runner, dict)}
        != expected_roles
        or not all(
            valid_file_record(
                runner, allow_missing=report.get("gate_status") == "failed"
            )
            for runner in runners
        )
    ):
        raise ValueError(f"{path} has malformed runner provenance")
    cases = report.get("cases")
    if not isinstance(cases, list) or any(not isinstance(case, dict) for case in cases):
        raise ValueError(f"{path} has malformed cases")
    case_ids = [case.get("id") for case in cases]
    if (
        len(case_ids) != len(set(case_ids))
        or set(case_ids) != set(source_rows[source_kind])
    ):
        raise ValueError(f"{path} does not account for every {source_kind} case")
    gate_status = report.get("gate_status")
    if gate_status not in {"passed", "failed"}:
        raise ValueError(f"{path} has malformed gate status")
    for case in cases:
        manifest_row = source_rows[source_kind][case["id"]]
        expected_status = manifest_row["status"]
        if source_kind == "golden" and lane == "golden-scripted":
            features = manifest_row.get("features", [])
            if "scripted-status:exact" in features:
                expected_status = "exact"
            elif "scripted-status:diverges" in features:
                expected_status = "diverges"
        fixture_label = (
            manifest_row["source"] if source_kind == "silver" else manifest_row["path"]
        )
        expected_lane = manifest_row.get("lane") if source_kind == "silver" else None
        expected_scripts = {
            field: manifest_row[field]
            for field in ("input_script", "view_model_script")
            if field in manifest_row
        }
        expected_outcome = {
            "exact": "exact",
            "diverges": "divergent",
            "unsupported-feature": "unsupported",
            "provenance-unknown": "unsupported",
            "not-yet": "pending",
            "pending": "pending",
            "pending-scripted": "pending",
        }.get(expected_status)
        allowed_outcomes = {
            "exact": {"exact", "regressed"},
            "divergent": {"divergent", "newly-exact"},
            "unsupported": {"unsupported"},
            "pending": {"pending"},
        }.get(expected_outcome, set())
        if case.get("outcome") not in allowed_outcomes:
            raise ValueError(
                f"{path} has impossible outcome for {case.get('id')}: "
                f"{case.get('outcome')} from {expected_status}"
            )
        anomalous = case.get("outcome") in {"regressed", "newly-exact"} or (
            case.get("divergence_check") == "changed"
        )
        if (gate_status == "passed" and anomalous) or (
            case.get("outcome") in {"pending", "unsupported"} and case.get("executed")
        ) or (case.get("outcome") in {"regressed", "newly-exact"} and not case.get("executed")):
            raise ValueError(f"{path} has impossible execution state for {case.get('id')}")
        if gate_status == "passed":
            selected = (
                manifest_row.get("lane") == "runtime"
                and expected_outcome in {"exact", "divergent"}
                if source_kind == "silver"
                else expected_outcome in {"exact", "divergent"}
                if lane == "golden-scripted"
                else expected_outcome == "exact"
                and not (
                    manifest_row.get("rust_execute_scripts", False)
                    or "scripted-runner-only" in manifest_row.get("features", [])
                )
            )
            if case.get("executed") != selected:
                raise ValueError(
                    f"{path} has impossible successful execution state for {case.get('id')}"
                )
        if (
            case.get("outcome") not in DIFFERENTIAL_OUTCOMES
            or not isinstance(case.get("executed"), bool)
            or case.get("declared_status") != expected_status
            or case.get("fixture", {}).get("path") != fixture_label
            or (source_kind == "silver" and case.get("lane") != expected_lane)
            or not valid_file_record(
                case.get("fixture"),
                allow_missing=(
                    source_kind == "silver"
                    and manifest_row["status"] == "provenance-unknown"
                ),
                allow_virtual=(
                    source_kind == "silver"
                    and manifest_row["lane"] == "scripted"
                    and manifest_row["source"] == "inline-script"
                ),
            )
        ):
            raise ValueError(f"{path} has malformed case provenance for {case.get('id')}")
        if source_kind == "silver" and (
            not valid_file_record(case.get("baseline"))
            or case["baseline"].get("path") != manifest_row["expected"]
            or not isinstance(case.get("dependencies"), list)
            or not all(valid_file_record(item) for item in case["dependencies"])
            or sorted(item["path"] for item in case["dependencies"])
            != sorted(manifest_row.get("dependencies", []))
            or not isinstance(case.get("action_fixtures"), list)
            or not all(valid_file_record(item) for item in case["action_fixtures"])
            or sorted(item["path"] for item in case["action_fixtures"])
            != sorted(
                action["source"]
                for action in manifest_row.get("actions", [])
                if isinstance(action, dict)
                and action.get("kind") == "set-view-model-font-bytes"
            )
        ):
            raise ValueError(f"{path} has malformed silver provenance for {case.get('id')}")
        if source_kind == "golden":
            actual_scripts = {
                field: case[field]
                for field in ("input_script", "view_model_script")
                if field in case
            }
            if set(actual_scripts) != set(expected_scripts) or any(
                not valid_file_record(record)
                or record["path"] != expected_scripts[field]
                for field, record in actual_scripts.items()
            ):
                raise ValueError(
                    f"{path} has malformed script provenance for {case.get('id')}"
                )
    expected_summary = dict(
        sorted(collections.Counter(case["outcome"] for case in cases).items())
    )
    if report.get("summary") != expected_summary:
        raise ValueError(f"{path} has malformed outcome summary")
    return source_kind


def validate_stale_differential_report(path: Path, report: dict) -> str:
    """Validate an artifact envelope without binding it to today's manifest."""
    lane = report.get("lane")
    if lane not in DIFFERENTIAL_LANES:
        raise ValueError(f"{path} has unsupported differential lane {lane}")
    if not COMMIT.fullmatch(str(report.get("cpp_ref", ""))) or not COMMIT.fullmatch(
        str(report.get("rust_commit", ""))
    ):
        raise ValueError(f"{path} has malformed commit provenance")
    source_kind = "silver" if lane == "silver" else "golden"
    manifest_name = "silver-corpus.toml" if source_kind == "silver" else "corpus.toml"
    manifest = report.get("manifest")
    if (
        not valid_file_record(manifest)
        or Path(manifest["path"]).name != manifest_name
    ):
        raise ValueError(f"{path} has malformed manifest provenance")
    runners = report.get("runners")
    expected_roles = {"validator"} if source_kind == "silver" else {"cpp", "rust"}
    if (
        not isinstance(runners, list)
        or len(runners) != len(expected_roles)
        or {runner.get("role") for runner in runners if isinstance(runner, dict)}
        != expected_roles
        or not all(
            valid_file_record(
                runner, allow_missing=report.get("gate_status") == "failed"
            )
            for runner in runners
        )
    ):
        raise ValueError(f"{path} has malformed runner provenance")
    cases = report.get("cases")
    if not isinstance(cases, list) or any(not isinstance(case, dict) for case in cases):
        raise ValueError(f"{path} has malformed cases")
    case_ids = [case.get("id") for case in cases]
    if (
        not case_ids
        or any(not isinstance(case_id, str) for case_id in case_ids)
        or len(case_ids) != len(set(case_ids))
        or any(
            case.get("outcome") not in DIFFERENTIAL_OUTCOMES
            or not isinstance(case.get("executed"), bool)
            or not isinstance(case.get("declared_status"), str)
            or not isinstance(case.get("verification"), str)
            for case in cases
        )
    ):
        raise ValueError(f"{path} has malformed cases")
    for case in cases:
        if source_kind == "golden":
            if not valid_file_record(case.get("fixture")) or any(
                field in case and not valid_file_record(case[field])
                for field in ("input_script", "view_model_script")
            ):
                raise ValueError(f"{path} has malformed golden case provenance")
        if source_kind == "silver" and (
            not isinstance(case.get("lane"), str)
            or not valid_file_record(
                case.get("fixture"),
                allow_missing=case.get("declared_status") == "provenance-unknown",
                allow_virtual=(
                    case.get("lane") == "scripted"
                    and case.get("fixture", {}).get("path") == "inline-script"
                ),
            )
            or not valid_file_record(case.get("baseline"))
            or not isinstance(case.get("dependencies"), list)
            or not all(
                valid_file_record(item) for item in case["dependencies"]
            )
            or not isinstance(case.get("action_fixtures"), list)
            or not all(
                valid_file_record(item) for item in case["action_fixtures"]
            )
        ):
            raise ValueError(f"{path} has malformed silver case provenance")
    if report.get("gate_status") not in {"passed", "failed"}:
        raise ValueError(f"{path} has malformed gate status")
    expected_summary = dict(
        sorted(collections.Counter(case["outcome"] for case in cases).items())
    )
    if report.get("summary") != expected_summary:
        raise ValueError(f"{path} has malformed outcome summary")
    return source_kind


def candidate(
    *,
    candidate_id: str,
    source_kind: str,
    source_row: str,
    upstream_owner: str | None,
    evidence_links: list[str],
    first_signal: str,
    evidence_state: str,
    freshness_state: str = "manifest-current",
    missing_proofs: list[str] | None = None,
) -> dict:
    row = {
        "id": candidate_id,
        "source_kind": source_kind,
        "source_row": source_row,
        "upstream_owner": upstream_owner,
        "owner_state": "resolved" if upstream_owner else "unresolved",
        "evidence_links": sorted(set(evidence_links)),
        "first_signal": first_signal,
        "missing_proofs": missing_proofs or [],
        "evidence_state": evidence_state,
        "churn_freshness": {
            "state": freshness_state,
            "churn": (
                "current-commit-observation"
                if freshness_state == "runtime-artifact-current"
                else "unmeasured"
            ),
            "basis": (
                "artifact pins current Rust commit and upstream ref"
                if freshness_state == "runtime-artifact-current"
                else "owner proof audit pin"
                if freshness_state == "stale"
                else "checked-in ledger"
            ),
        },
    }
    finalize_candidate(row)
    return row


def finalize_candidate(row: dict, *, runtime_verified: bool = False) -> None:
    disposition = disposition_for(row["evidence_state"], row["source_kind"])
    family, subsystem = owner_coordinates(
        row["upstream_owner"], row["source_row"]
    )
    boundary = semantic_boundary(row["evidence_state"], row["first_signal"])
    confidence = (
        "high"
        if runtime_verified
        else "medium"
        if disposition
        in {"known-divergence", "intentional-decision", "extension", "unsupported"}
        else "low"
    )
    reach = product_reach_for(
        row["upstream_owner"], row["source_row"], row["first_signal"]
    )
    cluster_id = f"cluster:{family}:{boundary}"
    row.update(
        {
            "confidence": confidence,
            "product_reach": reach,
            "disposition": disposition,
            "semantic_boundary": boundary,
            "owner_family": family,
            "subsystem": subsystem,
            "discovery_value": discovery_value(
                disposition, reach["level"], confidence
            ),
            "cluster_id": cluster_id,
        }
    )


def partition_rows(
    rows: list[dict], proven_statuses: set[str], make_candidate
) -> tuple[list[dict], dict]:
    candidates = []
    proven_rows = []
    for row in rows:
        if row["status"] in proven_statuses and not row.get(
            "scripted_divergence_signature"
        ):
            proven_rows.append(row.get("upstream", row.get("id")))
        else:
            candidates.append(make_candidate(row))
    return candidates, {
        "rows": len(rows),
        "candidates": len(candidates),
        "proven": len(proven_rows),
        "proven_rows": sorted(proven_rows),
    }


def owner_candidates(repo_root: Path) -> tuple[list[dict], dict]:
    document = json.loads((repo_root / "docs/parity-owner-proofs.json").read_text())
    candidates = []
    proven_rows = []
    for owner in document["owners"]:
        if owner["effective_state"] == "behaviorally-proven":
            proven_rows.append(owner["upstream"])
            continue
        upstream = owner["upstream"]
        missing_proofs = owner_missing_proofs(owner)
        candidates.append(
            candidate(
                candidate_id=f"owner:{upstream}",
                source_kind="owner-proof",
                source_row=upstream,
                upstream_owner=upstream,
                evidence_links=owner["structural_evidence"]
                + owner["behavioral_evidence"],
                first_signal="; ".join(missing_proofs),
                evidence_state=owner["effective_state"],
                freshness_state=owner["freshness"],
                missing_proofs=missing_proofs,
            )
        )
    return candidates, {
        "rows": len(document["owners"]),
        "candidates": len(candidates),
        "proven": len(proven_rows),
        "proven_rows": sorted(proven_rows),
    }


def upstream_test_candidates(repo_root: Path) -> tuple[list[dict], dict]:
    rows = read_toml(repo_root / "test-correspondence-manifest.toml")["file"]

    def make(row: dict) -> dict:
        covered = len(row.get("covered_test_cases", []))
        missing = row["test_case_count"] - covered
        return candidate(
            candidate_id=f"test:{row['upstream']}",
            source_kind="upstream-tests",
            source_row=row["upstream"],
            upstream_owner=row["upstream"],
            evidence_links=["test-correspondence-manifest.toml"]
            + row.get("evidence", []),
            first_signal=row.get("note")
            or f"{missing} of {row['test_case_count']} upstream test cases lack proof",
            evidence_state=row["status"],
        )

    return partition_rows(rows, {"ported-direct", "ported-differential"}, make)


def golden_candidates(repo_root: Path) -> tuple[list[dict], dict]:
    rows = read_toml(repo_root / "corpus.toml")["file"]

    def make(row: dict) -> dict:
        signal = row.get("scripted_divergence_signature") or row.get(
            "divergence_signature"
        )
        return candidate(
            candidate_id=f"golden:{row['id']}",
            source_kind="golden",
            source_row=row["id"],
            upstream_owner=None,
            evidence_links=["corpus.toml", row["path"]],
            first_signal=signal or "fixture has no passing Rust/C++ differential proof",
            evidence_state=(
                "diverges"
                if row.get("scripted_divergence_signature")
                else row["status"]
            ),
        )

    return partition_rows(rows, {"exact"}, make)


def decision_extension_candidates(repo_root: Path) -> tuple[list[dict], dict]:
    dimensions = json.loads(
        (repo_root / "docs/parity-owner-proofs.json").read_text()
    )["evidence_dimensions"]
    candidates = []
    for source_kind, rows in (
        ("decision", dimensions["decisions"]),
        ("extension", dimensions["extensions"]),
    ):
        for row in rows:
            row_id = row["id"]
            candidates.append(
                candidate(
                    candidate_id=f"{source_kind}:{row_id}",
                    source_kind=source_kind,
                    source_row=row_id,
                    upstream_owner=None,
                    evidence_links=[
                        "docs/parity-gap-register.md",
                        "docs/parity-owner-proofs.json",
                    ],
                    first_signal=row["summary"],
                    evidence_state=source_kind,
                )
            )
    return candidates, {
        "rows": len(candidates),
        "candidates": len(candidates),
        "proven": 0,
        "proven_rows": [],
    }


def tracked_gap_candidates(repo_root: Path) -> tuple[list[dict], dict]:
    path = repo_root / "docs/parity-gap-register.md"
    rows = []
    for line_number, line in enumerate(path.read_text().splitlines(), 1):
        if not line.startswith("|"):
            continue
        cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
        if not cells or TRACKED_GAP_ID.fullmatch(cells[0]) is None:
            continue
        rows.append((line_number, cells))

    candidates = []
    proven_rows = []
    for line_number, cells in rows:
        row_id = cells[0]
        signal = " | ".join(cells[1:])
        row_family = row_id.split("-", 1)[0].rstrip("0123456789")
        primary = cells[1].lstrip("*").upper()
        explicit_status = cells[-1].lstrip("*").upper()
        status = (
            cells[3].lstrip("*").upper()
            if row_family == "F" and len(cells) >= 5
            else cells[2].lstrip("*").upper()
            if row_family == "W" and len(cells) >= 4
            else explicit_status
            if row_family == "RB"
            else primary
        )
        closed = (
            primary.startswith(("CLOSED", "RESOLVED", "SUPERSEDED"))
            if row_family in {"V", "A", "C", "H"}
            else status.startswith(("CLOSED", "RESOLVED", "SUPERSEDED"))
            if row_family in {"F", "W", "RB"}
            else False
        )
        if closed:
            proven_rows.append(row_id)
            continue
        state = (
            "unknown"
            if status.startswith("UNKNOWN")
            else "unsupported-feature"
            if status.startswith(("ABSENT", "DEFERRED"))
            else "tracked-gap"
        )
        candidates.append(
            candidate(
                candidate_id=f"gap:{row_id}",
                source_kind="tracked-gap",
                source_row=row_id,
                upstream_owner=None,
                evidence_links=[f"docs/parity-gap-register.md:{line_number}"],
                first_signal=signal,
                evidence_state=state,
            )
        )
    return candidates, {
        "rows": len(rows),
        "candidates": len(candidates),
        "proven": len(proven_rows),
        "proven_rows": sorted(proven_rows),
    }


def silver_candidates(repo_root: Path) -> tuple[list[dict], dict]:
    rows = read_toml(repo_root / "silver-corpus.toml")["case"]

    def make(row: dict) -> dict:
        return candidate(
            candidate_id=f"silver:{row['id']}",
            source_kind="silver",
            source_row=row["id"],
            upstream_owner=row.get("provenance_file"),
            evidence_links=["silver-corpus.toml", row["expected"]],
            first_signal=row.get("note") or "silver case lacks exact proof",
            evidence_state=row["status"],
        )

    return partition_rows(rows, {"exact"}, make)


def current_commit(repo_root: Path) -> str:
    return subprocess.run(
        ["git", "-C", str(repo_root), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def apply_differential_artifacts(
    repo_root: Path,
    differential_dir: Path,
    candidates: list[dict],
    accounting: dict[str, dict],
) -> list[dict]:
    expected_ref = json.loads(
        (repo_root / "docs/parity-owner-proofs.json").read_text()
    )["upstream_ref"]
    expected_commit = current_commit(repo_root)
    by_id = {row["id"]: row for row in candidates}
    source_rows = {
        "golden": {
            row["id"]: row
            for row in read_toml(repo_root / "corpus.toml")["file"]
        },
        "silver": {
            row["id"]: row
            for row in read_toml(repo_root / "silver-corpus.toml")["case"]
        },
    }
    artifacts = []
    for path in sorted(differential_dir.glob("*.json")):
        artifact_label = f"runtime-differential:{path.relative_to(differential_dir)}"
        report = json.loads(path.read_text())
        if report.get("schema") != "nuxie-runtime-differentials/v1":
            raise ValueError(f"{path} has unsupported differential report schema")
        fresh = (
            report.get("cpp_ref") == expected_ref
            and report.get("rust_commit") == expected_commit
        )
        source_kind = (
            validate_differential_report(path, report, repo_root, source_rows)
            if fresh
            else validate_stale_differential_report(path, report)
        )
        artifacts.append(
            {
                "path": artifact_label,
                "lane": report.get("lane"),
                "gate_status": report.get("gate_status"),
                "status": "accepted" if fresh else "stale",
                "cpp_ref": report.get("cpp_ref"),
                "rust_commit": report.get("rust_commit"),
            }
        )
        if not fresh:
            continue
        for case in report.get("cases", []):
            outcome = case.get("outcome")
            case_id = case.get("id")
            if not case["executed"] or outcome == "exact":
                continue
            if case_id not in source_rows[source_kind]:
                raise ValueError(
                    f"{path} references unknown {source_kind} case {case_id}"
                )
            candidate_id = f"{source_kind}:{case_id}"
            row = by_id.get(candidate_id)
            signal = case.get("diagnostic") or case.get("signature")
            if not isinstance(signal, str):
                signal = f"fresh differential outcome: {outcome}"
            observation = {
                "lane": report["lane"],
                "outcome": outcome,
                "signal": signal,
                "artifact": artifact_label,
            }
            if row is None:
                manifest_row = source_rows[source_kind][case_id]
                if source_kind == "silver":
                    upstream_owner = manifest_row.get("provenance_file")
                    evidence_links = [
                        "silver-corpus.toml",
                        manifest_row["expected"],
                        artifact_label,
                    ]
                else:
                    upstream_owner = None
                    evidence_links = [
                        "corpus.toml",
                        manifest_row["path"],
                        artifact_label,
                    ]
                row = candidate(
                    candidate_id=candidate_id,
                    source_kind=source_kind,
                    source_row=case_id,
                    upstream_owner=upstream_owner,
                    evidence_links=evidence_links,
                    first_signal=signal,
                    evidence_state=str(outcome),
                    freshness_state="runtime-artifact-current",
                )
                row["differential_observations"] = [observation]
                finalize_candidate(row, runtime_verified=True)
                candidates.append(row)
                by_id[candidate_id] = row
                accounting[source_kind]["candidates"] += 1
                accounting[source_kind]["proven"] -= 1
                accounting[source_kind]["proven_rows"].remove(case_id)
            else:
                observations = row.setdefault("differential_observations", [])
                observations.append(observation)
                observations.sort(
                    key=lambda item: (item["lane"], item["outcome"], item["artifact"])
                )
                if len(observations) == 1 or ARTIFACT_OUTCOME_PRIORITY.get(
                    str(outcome), 0
                ) > ARTIFACT_OUTCOME_PRIORITY.get(row["evidence_state"], 0):
                    row["first_signal"] = signal
                    row["evidence_state"] = str(outcome)
                row["evidence_links"] = sorted(
                    set(row["evidence_links"] + [artifact_label])
                )
                row["churn_freshness"] = {
                    "state": "runtime-artifact-current",
                    "churn": "current-commit-observation",
                    "basis": "artifact pins current Rust commit and upstream ref",
                }
                finalize_candidate(row, runtime_verified=bool(case.get("executed")))
    return artifacts


def normalized_evidence_path(repo_root: Path, value: str) -> str | None:
    candidate_path = value.split("::", 1)[0]
    candidate_path = re.sub(r":\d+$", "", candidate_path)
    path = Path(candidate_path)
    if path.is_absolute():
        try:
            candidate_path = str(path.relative_to(repo_root))
        except ValueError:
            return None
    return candidate_path if (repo_root / candidate_path).is_file() else None


def apply_churn(repo_root: Path, candidates: list[dict]) -> None:
    file_rows = read_toml(repo_root / "file-correspondence-manifest.toml")["file"]
    owner_paths = {
        row["upstream"]: [
            path.strip() for path in row.get("rust_module", "").split(";") if path.strip()
        ]
        for row in file_rows
    }
    fallback = {
        "upstream-tests": ["test-correspondence-manifest.toml"],
        "golden": ["corpus.toml"],
        "silver": ["silver-corpus.toml"],
        "decision": ["docs/parity-gap-register.md"],
        "extension": ["docs/parity-gap-register.md"],
        "tracked-gap": ["docs/parity-gap-register.md"],
    }
    candidate_paths = {}
    relevant_paths = set()
    for row in candidates:
        paths = list(owner_paths.get(row["source_row"], []))
        paths += [
            path
            for value in row["evidence_links"]
            if (path := normalized_evidence_path(repo_root, value)) is not None
        ]
        paths += fallback.get(row["source_kind"], [])
        candidate_paths[row["id"]] = sorted(set(paths))
        relevant_paths.update(candidate_paths[row["id"]])
    touch_counts = recent_touch_counts(repo_root, relevant_paths)
    for row in candidates:
        touches = sum(touch_counts[path] for path in candidate_paths[row["id"]])
        churn = "high" if touches >= 5 else "medium" if touches >= 2 else "low"
        row["churn_freshness"].update(
            {
                "churn": churn,
                "recent_touch_count": touches,
                "churn_basis": "mapped Rust and evidence paths in the last 100 changes",
            }
        )
        row["discovery_value"] += {"high": 10, "medium": 5, "low": 0}[churn]
        row["discovery_value"] += {
            "runtime-artifact-current": 5,
            "current": 3,
            "manifest-current": 0,
            "stale": -10,
        }[row["churn_freshness"]["state"]]


def recent_touch_counts(repo_root: Path, relevant_paths: set[str]) -> collections.Counter:
    pending = subprocess.run(
        ["git", "-C", str(repo_root), "status", "--porcelain=v1", "-z"],
        check=True,
        capture_output=True,
    ).stdout.decode(errors="surrogateescape")
    pending_paths = set()
    fields = pending.split("\0")
    index = 0
    while index < len(fields) and fields[index]:
        entry = fields[index]
        status = entry[:2]
        path = entry[3:]
        pending_paths.add(path)
        if "R" in status or "C" in status:
            index += 1
            if index < len(fields):
                pending_paths.add(fields[index])
        index += 1
    pending_relevant = pending_paths & relevant_paths
    history = subprocess.run(
        [
            "git",
            "-C",
            str(repo_root),
            "log",
            "-99" if pending_relevant else "-100",
            "--format=",
            "--name-only",
        ],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()
    touch_counts = collections.Counter(path for path in history if path)
    touch_counts.update(pending_relevant)
    return touch_counts


def build_report(repo_root: Path, differential_dir: Path | None = None) -> dict:
    candidates = []
    accounting = {}
    for source_kind, builder in (
        ("owner-proofs", owner_candidates),
        ("upstream-tests", upstream_test_candidates),
        ("golden", golden_candidates),
        ("silver", silver_candidates),
        ("decisions-and-extensions", decision_extension_candidates),
        ("tracked-gaps", tracked_gap_candidates),
    ):
        source_candidates, source_accounting = builder(repo_root)
        candidates.extend(source_candidates)
        accounting[source_kind] = source_accounting
    artifacts = (
        apply_differential_artifacts(
            repo_root, differential_dir, candidates, accounting
        )
        if differential_dir is not None and differential_dir.is_dir()
        else []
    )
    apply_churn(repo_root, candidates)
    candidates.sort(key=lambda row: (-row["discovery_value"], row["id"]))
    clustered: dict[str, list[dict]] = collections.defaultdict(list)
    for row in candidates:
        clustered[row["cluster_id"]].append(row)
    clusters = [
        {
            "id": cluster_id,
            "owner_family": rows[0]["owner_family"],
            "semantic_boundary": rows[0]["semantic_boundary"],
            "candidate_ids": sorted(row["id"] for row in rows),
            "candidate_count": len(rows),
            "max_discovery_value": max(row["discovery_value"] for row in rows),
        }
        for cluster_id, rows in sorted(clustered.items())
    ]
    dispositions = collections.Counter(row["disposition"] for row in candidates)
    owner_document = json.loads(
        (repo_root / "docs/parity-owner-proofs.json").read_text()
    )
    return {
        "schema": SCHEMA,
        "upstream_ref": owner_document["upstream_ref"],
        "audit_upstream_ref": owner_document["audit_upstream_ref"],
        "summary": {
            "candidate_count": len(candidates),
            "cluster_count": len(clusters),
            "disposition_counts": dict(sorted(dispositions.items())),
        },
        "candidates": candidates,
        "clusters": clusters,
        "accounting": accounting,
        "differential_artifacts": artifacts,
        "filters": {
            "owner_families": sorted({row["owner_family"] for row in candidates}),
            "subsystems": sorted({row["subsystem"] for row in candidates}),
            "evidence_states": sorted({row["evidence_state"] for row in candidates}),
            "dispositions": sorted({row["disposition"] for row in candidates}),
        },
    }


def json_text(payload: dict) -> str:
    return json.dumps(payload, indent=2, sort_keys=True) + "\n"


def markdown_cell(value: object, limit: int = 180) -> str:
    text = str(value).replace("\n", " ").replace("|", "\\|")
    return text if len(text) <= limit else text[: limit - 1] + "…"


def render_markdown(payload: dict) -> str:
    lines = [
        "# Runtime drift queue",
        "",
        "Generated from the checked-in parity ledgers. JSON is authoritative; this view highlights clusters and the highest-discovery candidates.",
        "",
        f"- Upstream ref: `{payload['upstream_ref']}`",
        f"- Candidates: {payload['summary']['candidate_count']}",
        f"- Clusters: {payload['summary']['cluster_count']}",
        "",
        "## Dispositions",
        "",
        "| disposition | candidates |",
        "|---|---:|",
    ]
    for disposition, count in payload["summary"]["disposition_counts"].items():
        lines.append(f"| {disposition} | {count} |")
    lines.extend(
        [
            "",
            "## Filter fields",
            "",
            "Filter the JSON `candidates` array by `owner_family`, `subsystem`, `evidence_state`, `disposition`, or sort by descending `discovery_value`. The complete deterministic value sets are in `filters`.",
            "",
            "## Clusters",
            "",
            "| cluster | boundary | owner family | candidates | max discovery value |",
            "|---|---|---|---:|---:|",
        ]
    )
    for cluster in sorted(
        payload["clusters"],
        key=lambda row: (-row["max_discovery_value"], row["id"]),
    ):
        lines.append(
            f"| `{cluster['id']}` | {cluster['semantic_boundary']} | {cluster['owner_family']} | {cluster['candidate_count']} | {cluster['max_discovery_value']} |"
        )
    lines.extend(
        [
            "",
            "## Highest-discovery candidates",
            "",
            "| value | candidate | disposition | owner | first signal |",
            "|---:|---|---|---|---|",
        ]
    )
    for row in payload["candidates"][:100]:
        owner = row["upstream_owner"] or "unresolved-owner"
        lines.append(
            f"| {row['discovery_value']} | `{row['id']}` | {row['disposition']} | `{markdown_cell(owner, 80)}` | {markdown_cell(row['first_signal'])} |"
        )
    lines.append("")
    return "\n".join(lines)


def write_text(path: Path, contents: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    handle = tempfile.NamedTemporaryFile(
        "w", encoding="utf-8", dir=path.parent, prefix=path.name, delete=False
    )
    try:
        handle.write(contents)
        handle.close()
        os.replace(handle.name, path)
    finally:
        if os.path.exists(handle.name):
            os.unlink(handle.name)


def check_snapshot(repo_root: Path, json_path: Path, markdown_path: Path) -> bool:
    payload = build_report(repo_root)
    return (
        json_path.is_file()
        and markdown_path.is_file()
        and json_path.read_text() == json_text(payload)
        and markdown_path.read_text() == render_markdown(payload)
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    build = subparsers.add_parser("build")
    build.add_argument("--repo-root", type=Path, default=Path.cwd())
    build.add_argument("--output", type=Path, required=True)
    build.add_argument("--differential-dir", type=Path)
    build.add_argument("--markdown-output", type=Path)
    check = subparsers.add_parser("check")
    check.add_argument("--repo-root", type=Path, default=Path.cwd())
    check.add_argument("--json", type=Path, required=True)
    check.add_argument("--markdown", type=Path, required=True)
    args = parser.parse_args()
    if args.command == "check":
        if check_snapshot(args.repo_root.resolve(), args.json, args.markdown):
            return 0
        print("runtime drift queue snapshot is stale", file=sys.stderr)
        return 1
    payload = build_report(
        args.repo_root.resolve(),
        args.differential_dir.resolve() if args.differential_dir else None,
    )
    write_text(args.output, json_text(payload))
    if args.markdown_output:
        write_text(args.markdown_output, render_markdown(payload))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
