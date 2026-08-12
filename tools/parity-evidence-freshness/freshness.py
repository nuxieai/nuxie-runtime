#!/usr/bin/env python3
"""Report whether captured runtime parity proofs still match their source inputs."""

from __future__ import annotations

import argparse
import collections
import fnmatch
import hashlib
import json
import re
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path
from typing import Any

PARITY_SCORECARD_TOOL_DIR = Path(__file__).resolve().parents[1] / "parity-scorecard"
if str(PARITY_SCORECARD_TOOL_DIR) not in sys.path:
    sys.path.insert(0, str(PARITY_SCORECARD_TOOL_DIR))

from ledger_scorecard import (  # noqa: E402
    audit_record_section,
    audit_record_substantiates,
    resolve_evidence_record,
    second_pass_record_substantiates,
)

REGISTRY_SCHEMA = "nuxie-parity-evidence-proofs/v1"
REPORT_SCHEMA = "nuxie-parity-evidence-freshness/v1"
SHA256_LENGTH = 64


class FreshnessError(ValueError):
    """Raised when proof provenance is malformed or cannot be validated."""


def sha256(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def load_json(path: Path) -> dict[str, Any]:
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise FreshnessError(f"cannot read proof registry {path}: {error}") from error
    if not isinstance(document, dict):
        raise FreshnessError("proof registry must be a JSON object")
    return document


def load_toml(path: Path) -> dict[str, Any]:
    try:
        with path.open("rb") as source:
            document = tomllib.load(source)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise FreshnessError(
            f"cannot read correspondence manifest {path}: {error}"
        ) from error
    if not isinstance(document, dict):
        raise FreshnessError("correspondence manifest must be a TOML table")
    return document


def checked_git(root: Path, *arguments: str) -> str:
    result = subprocess.run(
        ["git", "-C", str(root), *arguments],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        diagnostic = result.stderr.strip() or result.stdout.strip()
        raise FreshnessError(
            f"git {' '.join(arguments)} failed in {root}: {diagnostic}"
        )
    return result.stdout.strip()


def touch_counts_since(root: Path, revision: str) -> collections.Counter[str]:
    output = checked_git(root, "log", "--format=", "--name-only", f"{revision}..HEAD")
    counts: collections.Counter[str] = collections.Counter(
        line for line in output.splitlines() if line
    )
    dirty = checked_git(root, "diff", "--name-only", "HEAD")
    counts.update(set(line for line in dirty.splitlines() if line))
    untracked = checked_git(root, "ls-files", "--others", "--exclude-standard")
    counts.update(set(line for line in untracked.splitlines() if line))
    return counts


def recent_touch_counts(root: Path) -> collections.Counter[str]:
    output = checked_git(root, "log", "-100", "--format=", "--name-only")
    return collections.Counter(line for line in output.splitlines() if line)


def git_bytes(root: Path, revision: str, path: str) -> bytes:
    result = subprocess.run(
        ["git", "-C", str(root), "show", f"{revision}:{path}"],
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise FreshnessError(f"captured source does not exist: git:{revision}:{path}")
    return result.stdout


def required_string(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value:
        raise FreshnessError(f"missing {label}")
    return value


def required_sha(value: Any, label: str) -> str:
    digest = required_string(value, label)
    if len(digest) != SHA256_LENGTH or any(
        character not in "0123456789abcdef" for character in digest
    ):
        raise FreshnessError(f"{label} must be a lowercase SHA-256 digest")
    return digest


def validate_binding(binding: Any, label: str) -> dict[str, Any]:
    if not isinstance(binding, dict):
        raise FreshnessError(f"{label} must be an object")
    result: dict[str, Any] = {
        "id": required_string(binding.get("id"), f"{label} id"),
        "path": required_string(binding.get("path"), f"{label} path"),
        "sha256": required_sha(binding.get("sha256"), f"{label} sha256"),
    }
    selector = binding.get("selector")
    if selector is not None:
        if not isinstance(selector, dict):
            raise FreshnessError(f"{label} selector must be an object")
        kind = required_string(selector.get("kind"), f"{label} selector kind")
        if kind == "line-window":
            start = selector.get("start")
            end = selector.get("end")
            if (
                not isinstance(start, int)
                or isinstance(start, bool)
                or not isinstance(end, int)
                or isinstance(end, bool)
                or start < 1
                or end < start
            ):
                raise FreshnessError(f"{label} line-window selector is invalid")
            result["selector"] = {"kind": kind, "start": start, "end": end}
        elif kind in {"b6-section", "audit-row", "toml-member"}:
            result["selector"] = {
                "kind": kind,
                "id": required_string(selector.get("id"), f"{label} B6 section id"),
            }
        else:
            raise FreshnessError(f"{label} has unsupported selector {kind}")
    if "root" in binding:
        root = required_string(binding.get("root"), f"{label} root")
        if root not in {"repo", "upstream"}:
            raise FreshnessError(f"{label} root must be repo or upstream")
        result["root"] = root
    if "revision" in binding:
        revision = required_string(binding.get("revision"), f"{label} revision")
        if revision not in {"source", "evidence", "upstream"}:
            raise FreshnessError(
                f"{label} revision must be source, evidence, or upstream"
            )
        result["revision"] = revision
    return result


def validate_bindings(
    value: Any, label: str, *, required: bool
) -> list[dict[str, Any]]:
    if not isinstance(value, list):
        raise FreshnessError(f"{label} must be a list")
    bindings = [
        validate_binding(binding, f"{label}[{index}]")
        for index, binding in enumerate(value)
    ]
    if required and not bindings:
        raise FreshnessError(f"{label} must not be empty")
    identifiers = [binding["id"] for binding in bindings]
    if len(identifiers) != len(set(identifiers)):
        raise FreshnessError(f"{label} contains duplicate ids")
    return bindings


def historical_binding_matches(
    *, binding: dict[str, Any], root: Path, revision: str
) -> bool:
    payload = selected_payload(git_bytes(root, revision, binding["path"]), binding)
    return bool(payload) and sha256(payload) == binding["sha256"]


def selected_payload(payload: bytes, binding: dict[str, Any]) -> bytes:
    selector = binding.get("selector")
    if selector is None:
        return payload
    if selector["kind"] == "line-window":
        lines = payload.splitlines(keepends=True)
        return b"".join(lines[selector["start"] - 1 : selector["end"]])
    if selector["kind"] == "audit-row":
        return audit_record_section(payload.decode("utf-8"), selector["id"]).encode(
            "utf-8"
        )
    if selector["kind"] == "toml-member":
        expected = selector["id"]
        for section in payload.decode("utf-8").split("[[member]]")[1:]:
            match = re.search(r'^id\s*=\s*"([^"]+)"', section, re.MULTILINE)
            if match is not None and match.group(1) == expected:
                return ("[[member]]" + section).encode("utf-8")
        return b""
    section_id = selector["id"].encode()
    marker = b"## " + section_id
    start = payload.find(marker)
    if start < 0:
        return b""
    next_section = payload.find(b"\n## B6-", start + len(marker))
    return payload[start : next_section if next_section >= 0 else len(payload)]


def current_binding_matches(binding: dict[str, Any], root: Path) -> bool:
    path = root / binding["path"]
    if not path.is_file():
        return False
    payload = path.read_bytes()
    selector = binding.get("selector")
    if selector is None or selector["kind"] != "line-window":
        return sha256(selected_payload(payload, binding)) == binding["sha256"]
    line_count = selector["end"] - selector["start"] + 1
    lines = payload.splitlines(keepends=True)
    matches = sum(
        sha256(b"".join(lines[index : index + line_count])) == binding["sha256"]
        for index in range(max(0, len(lines) - line_count + 1))
    )
    return matches == 1


def infer_product_reach(owner: str, owner_family: str) -> str:
    text = f"{owner} {owner_family}".lower()
    if any(
        token in text
        for token in (
            "artboard",
            "component",
            "layout",
            "state-machine",
            "data-bind",
            "draw",
            "shape",
            "text",
            "input",
            "focus",
            "animation",
        )
    ):
        return "high"
    if any(
        token in text
        for token in ("renderer", "lua", "script", "asset", "audio", "image")
    ):
        return "medium"
    return "low"


def infer_subsystem(owner: str, owner_family: str) -> str:
    if owner.startswith("src/"):
        relative = owner.removeprefix("src/")
        parent = Path(relative).parent.as_posix()
        return parent if parent != "." else Path(relative).stem
    return owner.split(".", 1)[0] or owner_family


def build_report(
    repo_root: Path, upstream_root: Path, registry_path: Path
) -> dict[str, Any]:
    registry = load_json(registry_path)
    if registry.get("schema") != REGISTRY_SCHEMA:
        raise FreshnessError("proof registry has an unsupported schema")
    captured_upstream_ref = required_string(
        registry.get("upstream_ref"), "registry upstream_ref"
    )
    current_upstream_ref = checked_git(upstream_root, "rev-parse", "HEAD")

    manifest = load_toml(repo_root / "file-correspondence-manifest.toml")
    if manifest.get("schema") != "nuxie-file-correspondence/v1":
        raise FreshnessError("file correspondence manifest has an unsupported schema")
    if manifest.get("upstream_ref") != current_upstream_ref:
        raise FreshnessError(
            "current upstream checkout and correspondence manifest upstream refs differ"
        )
    captures = registry.get("captures")
    if not isinstance(captures, dict):
        raise FreshnessError("proof registry captures must be an object")
    structural_capture = required_string(
        captures.get("structural_rust_commit"), "structural capture commit"
    )
    captured_manifest = tomllib.loads(
        git_bytes(
            repo_root, structural_capture, "file-correspondence-manifest.toml"
        ).decode("utf-8")
    )
    captured_rows = captured_manifest.get("file")
    if not isinstance(captured_rows, list) or not all(
        isinstance(row, dict) for row in captured_rows
    ):
        raise FreshnessError("captured correspondence manifest has invalid rows")
    captured_row_by_owner = {
        required_string(row.get("upstream"), "captured manifest upstream owner"): row
        for row in captured_rows
    }
    current_rows = set(manifest.get("current_audit_rows", []))
    rows = manifest.get("file")
    if not isinstance(rows, list) or not all(isinstance(row, dict) for row in rows):
        raise FreshnessError("file correspondence manifest must contain [[file]] rows")
    row_by_owner = {
        required_string(row.get("upstream"), "manifest upstream owner"): row
        for row in rows
    }
    repo_recent_churn = recent_touch_counts(repo_root)
    churn_cache: dict[tuple[Path, str], collections.Counter[str]] = {}
    historical_cache: dict[tuple[str, str, str], bool] = {}
    current_cache: dict[tuple[str, str], bool] = {}

    def source_churn(root: Path, revision: str, paths: set[str]) -> int:
        key = (root, revision)
        if key not in churn_cache:
            churn_cache[key] = touch_counts_since(root, revision)
        return sum(churn_cache[key][path] for path in paths)

    def historical_matches(binding: dict[str, Any], root: Path, revision: str) -> bool:
        identity = json.dumps(binding, sort_keys=True, separators=(",", ":"))
        key = (str(root), revision, identity)
        if key not in historical_cache:
            historical_cache[key] = historical_binding_matches(
                binding=binding, root=root, revision=revision
            )
        return historical_cache[key]

    def current_matches(binding: dict[str, Any], root: Path) -> bool:
        identity = json.dumps(binding, sort_keys=True, separators=(",", ":"))
        key = (str(root), identity)
        if key not in current_cache:
            current_cache[key] = current_binding_matches(binding, root)
        return current_cache[key]

    raw_proofs = registry.get("proofs")
    if not isinstance(raw_proofs, list) or not all(
        isinstance(proof, dict) for proof in raw_proofs
    ):
        raise FreshnessError("proof registry must contain proof objects")
    proof_ids: set[str] = set()
    structural_owners: set[str] = set()
    report_proofs = []
    for raw in raw_proofs:
        proof_id = required_string(raw.get("id"), "proof id")
        if proof_id in proof_ids:
            raise FreshnessError(f"duplicate proof id {proof_id}")
        proof_ids.add(proof_id)
        kind = required_string(raw.get("kind"), f"{proof_id} kind")
        if kind not in {"structural", "behavioral"}:
            raise FreshnessError(f"{proof_id}: unsupported proof kind {kind}")
        owner = required_string(raw.get("owner"), f"{proof_id} owner")
        owner_family = required_string(
            raw.get("owner_family"), f"{proof_id} owner_family"
        )
        product_reach = required_string(
            raw.get("product_reach"), f"{proof_id} product_reach"
        )
        if product_reach not in {"high", "medium", "low"}:
            raise FreshnessError(f"{proof_id}: invalid product reach {product_reach}")
        captured_rust_commit = required_string(
            raw.get("captured_rust_commit"), f"{proof_id} captured_rust_commit"
        )
        captured_evidence_commit = required_string(
            raw.get("captured_evidence_commit", captured_rust_commit),
            f"{proof_id} captured_evidence_commit",
        )
        if (
            required_string(raw.get("upstream_ref"), f"{proof_id} upstream_ref")
            != captured_upstream_ref
        ):
            raise FreshnessError(f"{proof_id}: upstream ref differs from registry")
        evidence = validate_binding(
            (
                {"id": "evidence", **raw.get("evidence", {})}
                if isinstance(raw.get("evidence"), dict)
                else raw.get("evidence")
            ),
            f"{proof_id} evidence",
        )
        cpp_items = validate_bindings(
            raw.get("cpp_items"), f"{proof_id} cpp_items", required=True
        )
        rust_items = validate_bindings(
            raw.get("rust_items"), f"{proof_id} rust_items", required=True
        )
        probes = validate_bindings(
            raw.get("probes"), f"{proof_id} probes", required=False
        )
        fixtures = validate_bindings(
            raw.get("fixtures"), f"{proof_id} fixtures", required=False
        )

        if kind == "structural":
            structural_owners.add(owner)
            row = row_by_owner.get(owner) or captured_row_by_owner.get(owner)
            if row is None:
                raise FreshnessError(
                    f"{proof_id}: structural owner has no captured row"
                )
            claim = raw.get("structural_claim")
            if not isinstance(claim, dict):
                raise FreshnessError(f"{proof_id}: missing captured structural claim")
            row_id = required_string(claim.get("row_id"), f"{proof_id} B6 row")
            verdict = required_string(
                claim.get("verdict"), f"{proof_id} captured verdict"
            )
            audit_record = required_string(
                claim.get("audit_record"), f"{proof_id} captured audit record"
            )
            raw_mapping = raw.get("rust_mapping_paths")
            if (
                not isinstance(raw_mapping, list)
                or not raw_mapping
                or not all(isinstance(path, str) and path for path in raw_mapping)
                or raw_mapping != sorted(set(raw_mapping))
            ):
                raise FreshnessError(f"{proof_id}: invalid captured Rust mapping paths")
            cpp_paths = {item["path"] for item in cpp_items}
            rust_paths = {item["path"] for item in rust_items}
            if owner not in cpp_paths:
                raise FreshnessError(
                    f"{proof_id}: captured bindings omit upstream owner {owner}"
                )
            missing_mappings = sorted(set(raw_mapping) - rust_paths)
            if missing_mappings:
                raise FreshnessError(
                    f"{proof_id}: captured bindings omit Rust mappings "
                    + ", ".join(missing_mappings)
                )
            if audit_record != evidence["path"]:
                raise FreshnessError(
                    f"{proof_id}: evidence path differs from audit_record"
                )
            evidence_contents = git_bytes(
                repo_root, captured_evidence_commit, evidence["path"]
            ).decode("utf-8")
            substantiated = (
                second_pass_record_substantiates(evidence_contents, row_id, verdict)
                if Path(evidence["path"]).name == "SECOND_PASS.md"
                else audit_record_substantiates(
                    evidence_contents, row_id, owner, verdict
                )
            )
            if not substantiated:
                raise FreshnessError(
                    f"{proof_id}: evidence record does not substantiate {row_id}"
                )

        historical_bindings = [
            *(
                historical_matches(binding, upstream_root, captured_upstream_ref)
                for binding in cpp_items
            ),
            *(
                historical_matches(binding, repo_root, captured_rust_commit)
                for binding in rust_items
            ),
            historical_matches(evidence, repo_root, captured_evidence_commit),
            *(
                historical_matches(
                    binding,
                    repo_root,
                    (
                        captured_evidence_commit
                        if binding.get("revision") == "evidence"
                        else captured_rust_commit
                    ),
                )
                for binding in probes
            ),
        ]
        for fixture in fixtures:
            fixture_root = (
                upstream_root if fixture.get("root") == "upstream" else repo_root
            )
            fixture_revision = (
                captured_upstream_ref
                if fixture_root == upstream_root
                else (
                    captured_evidence_commit
                    if fixture.get("revision") == "evidence"
                    else captured_rust_commit
                )
            )
            historical_bindings.append(
                historical_matches(fixture, fixture_root, fixture_revision)
            )
        if not all(historical_bindings):
            raise FreshnessError(
                f"{proof_id}: captured hashes do not match historical source"
            )

        cpp_paths = {binding["path"] for binding in cpp_items}
        rust_paths = {binding["path"] for binding in [*rust_items, *probes, evidence]}
        cpp_paths.update(
            fixture["path"] for fixture in fixtures if fixture.get("root") == "upstream"
        )
        rust_paths.update(
            fixture["path"] for fixture in fixtures if fixture.get("root") != "upstream"
        )
        churn = source_churn(
            upstream_root, captured_upstream_ref, cpp_paths
        ) + source_churn(repo_root, captured_rust_commit, rust_paths)

        stale_reasons = []
        if kind == "structural" and row_id not in current_rows:
            stale_reasons.append("current-audit-declaration-changed")
        if kind == "structural":
            current_row = row_by_owner.get(owner)
            if current_row is None:
                stale_reasons.append("structural-owner-removed")
            else:
                current_row_id = required_string(
                    current_row.get("b6_row_id"), f"{owner} current B6 row"
                )
                current_verdict = required_string(
                    current_row.get("b6_verdict"), f"{owner} current B6 verdict"
                )
                current_audit_record = required_string(
                    current_row.get("audit_record"), f"{owner} current audit record"
                )
                current_contents, current_locator, current_checked_in = (
                    resolve_evidence_record(repo_root, current_audit_record)
                )
                if current_contents is None or not current_checked_in:
                    raise FreshnessError(
                        f"{owner}: current evidence record does not exist: "
                        f"{current_audit_record}"
                    )
                current_substantiated = (
                    second_pass_record_substantiates(
                        current_contents, current_row_id, current_verdict
                    )
                    if Path(current_audit_record).name == "SECOND_PASS.md"
                    else audit_record_substantiates(
                        current_contents, current_row_id, owner, current_verdict
                    )
                )
                if not current_substantiated:
                    raise FreshnessError(
                        f"{owner}: current evidence record {current_locator} does not "
                        f"substantiate {current_row_id}"
                    )
                if current_row_id != row_id:
                    stale_reasons.append("structural-row-id-changed")
                if current_verdict != verdict:
                    stale_reasons.append("structural-verdict-changed")
                if current_audit_record != audit_record:
                    stale_reasons.append("structural-audit-record-changed")
            current_mapping = (
                sorted(
                    source.strip().split("::", 1)[0]
                    for source in str(current_row.get("rust_module", "")).split(";")
                    if source.strip()
                )
                if current_row is not None
                else []
            )
            if current_mapping != raw["rust_mapping_paths"]:
                stale_reasons.append("rust-owner-mapping-changed")
        for prefix, bindings, root in (
            ("cpp-item", cpp_items, upstream_root),
            ("rust-item", rust_items, repo_root),
            ("probe", probes, repo_root),
        ):
            stale_reasons.extend(
                f"{prefix}-changed:{binding['id']}"
                for binding in bindings
                if not current_matches(binding, root)
            )
        if not current_matches(evidence, repo_root):
            stale_reasons.append("evidence-record-changed")
        for fixture in fixtures:
            fixture_root = (
                upstream_root if fixture.get("root") == "upstream" else repo_root
            )
            if not current_matches(fixture, fixture_root):
                stale_reasons.append(f"fixture-changed:{fixture['id']}")

        report_proofs.append(
            {
                "id": proof_id,
                "kind": kind,
                "owner": owner,
                "owner_family": owner_family,
                "subsystem": infer_subsystem(owner, owner_family),
                "product_reach": product_reach,
                "historical_validity": "valid",
                "current_validity": "stale" if stale_reasons else "current",
                "binding_completeness": "complete",
                "source_churn": churn,
                "stale_reasons": stale_reasons,
            }
        )

    audit_upstream_ref = required_string(
        manifest.get("audit_upstream_ref"), "manifest audit_upstream_ref"
    )
    legacy_rows_by_owner = {**captured_row_by_owner, **row_by_owner}
    for owner, row in sorted(legacy_rows_by_owner.items()):
        row_id = required_string(row.get("b6_row_id"), f"{owner} B6 row")
        if owner in structural_owners:
            continue
        audit_record = required_string(row.get("audit_record"), f"{owner} audit record")
        contents, evidence_locator, _ = resolve_evidence_record(repo_root, audit_record)
        if contents is None:
            raise FreshnessError(
                f"{owner}: evidence record does not exist: {audit_record}"
            )
        verdict = required_string(row.get("b6_verdict"), f"{owner} B6 verdict")
        substantiated = (
            second_pass_record_substantiates(contents, row_id, verdict)
            if Path(audit_record).name == "SECOND_PASS.md"
            else audit_record_substantiates(contents, row_id, owner, verdict)
        )
        if not substantiated:
            raise FreshnessError(
                f"{owner}: evidence record {evidence_locator} does not substantiate {row_id}"
            )
        owner_family = required_string(row.get("b6_cluster"), f"{owner} owner family")
        stale_reasons = [
            (
                "current-proof-missing-content-bindings"
                if row_id in current_rows
                else "legacy-proof-missing-content-bindings"
            )
        ]
        if owner not in row_by_owner:
            stale_reasons.append("structural-owner-removed")
        if audit_upstream_ref != current_upstream_ref and row_id not in current_rows:
            stale_reasons.append(
                f"upstream-pin-changed:{audit_upstream_ref[:8]}->{current_upstream_ref[:8]}"
            )
        report_proofs.append(
            {
                "id": f"structural:{row_id}",
                "kind": "structural",
                "owner": owner,
                "owner_family": owner_family,
                "subsystem": infer_subsystem(owner, owner_family),
                "product_reach": infer_product_reach(owner, owner_family),
                "historical_validity": "valid",
                "current_validity": "stale",
                "binding_completeness": "legacy-unbound",
                "source_churn": sum(
                    repo_recent_churn[path.strip().split("::", 1)[0]]
                    for path in str(row.get("rust_module", "")).split(";")
                    if path.strip()
                ),
                "stale_reasons": stale_reasons,
                "evidence": evidence_locator,
            }
        )

    report_proofs.sort(key=lambda proof: proof["id"])
    current = sum(proof["current_validity"] == "current" for proof in report_proofs)
    stale = len(report_proofs) - current
    reach_priority = {"high": 0, "medium": 1, "low": 2}
    stale_owners = sorted(
        (proof for proof in report_proofs if proof["current_validity"] == "stale"),
        key=lambda proof: (
            proof["subsystem"],
            reach_priority[proof["product_reach"]],
            -proof["source_churn"],
            proof["id"],
        ),
    )
    stale_by_family = dict(
        sorted(
            collections.Counter(proof["owner_family"] for proof in stale_owners).items()
        )
    )
    stale_by_subsystem = dict(
        sorted(
            collections.Counter(proof["subsystem"] for proof in stale_owners).items()
        )
    )
    source_glob = str(manifest.get("source_glob", "src/**/*.cpp"))
    exclude_glob = str(manifest.get("exclude_glob", "src/generated/**"))
    discovered = {
        path.relative_to(upstream_root).as_posix()
        for path in upstream_root.glob(source_glob)
        if path.is_file()
        and not fnmatch.fnmatch(
            path.relative_to(upstream_root).as_posix(), exclude_glob
        )
    }
    declared = set(captured_row_by_owner)
    changed = sorted(
        {
            proof["owner"]
            for proof in report_proofs
            if any(
                reason.startswith("cpp-item-changed:")
                for reason in proof["stale_reasons"]
            )
        }
    )
    repo_source_paths = {
        "file-correspondence-manifest.toml",
        registry_path.resolve().relative_to(repo_root).as_posix(),
    }
    for proof in raw_proofs:
        if proof.get("kind") != "structural":
            continue
        evidence = proof.get("evidence")
        if isinstance(evidence, dict) and isinstance(evidence.get("path"), str):
            repo_source_paths.add(evidence["path"])
        for field in ("rust_items", "probes", "fixtures"):
            for binding in proof.get(field, []):
                if (
                    isinstance(binding, dict)
                    and binding.get("root", "repo") == "repo"
                    and isinstance(binding.get("path"), str)
                ):
                    repo_source_paths.add(binding["path"])
    repo_source_state = []
    for path in sorted(repo_source_paths):
        source = repo_root / path
        repo_source_state.append(
            (
                {"path": path, "sha256": sha256(source.read_bytes())}
                if source.is_file()
                else {"path": path, "missing": True}
            )
        )
    upstream_source_paths = {
        binding["path"]
        for proof in raw_proofs
        for binding in proof.get("cpp_items", [])
        if isinstance(binding, dict) and isinstance(binding.get("path"), str)
    }
    upstream_source_paths.update(
        binding["path"]
        for proof in raw_proofs
        for binding in proof.get("fixtures", [])
        if isinstance(binding, dict)
        and binding.get("root") == "upstream"
        and isinstance(binding.get("path"), str)
    )
    upstream_source_state = []
    for path in sorted(upstream_source_paths):
        source = upstream_root / path
        upstream_source_state.append(
            (
                {"path": path, "sha256": sha256(source.read_bytes())}
                if source.is_file()
                else {"path": path, "missing": True}
            )
        )
    return {
        "schema": REPORT_SCHEMA,
        "upstream_ref": current_upstream_ref,
        "captured_upstream_ref": captured_upstream_ref,
        "current_upstream_ref": current_upstream_ref,
        "rust_commit": checked_git(repo_root, "rev-parse", "HEAD"),
        "repo_source_state": repo_source_state,
        "upstream_source_state": upstream_source_state,
        "proofs": report_proofs,
        "stale_owners": stale_owners,
        "upstream_owner_changes": {
            "new": sorted(discovered - declared),
            "removed": sorted(declared - discovered),
            "changed": changed,
        },
        "summary": {
            "current": current,
            "stale": stale,
            "total": len(report_proofs),
            "stale_by_owner_family": stale_by_family,
            "stale_by_subsystem": stale_by_subsystem,
        },
    }


def write_json_atomic(path: Path, document: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        "w", encoding="utf-8", dir=path.parent, prefix=f".{path.name}.", delete=False
    ) as temporary:
        json.dump(document, temporary, indent=2, sort_keys=True)
        temporary.write("\n")
        temporary_path = Path(temporary.name)
    temporary_path.replace(path)


def render_markdown(document: dict[str, Any]) -> str:
    summary = document["summary"]
    changes = document["upstream_owner_changes"]
    lines = [
        "# Runtime parity evidence freshness",
        "",
        f"Upstream pin: `{document['upstream_ref']}`",
        f"Rust commit: `{document['rust_commit']}`",
        "",
        f"Proofs: {summary['total']} ({summary['current']} current, {summary['stale']} stale)",
        f"Upstream owners: {len(changes['new'])} new, {len(changes['removed'])} removed, "
        f"{len(changes['changed'])} changed",
        "",
        "## Stale owner families",
        "",
        "| Owner family | Stale proofs |",
        "| --- | ---: |",
    ]
    lines.extend(
        f"| `{family}` | {count} |"
        for family, count in summary["stale_by_owner_family"].items()
    )
    lines.extend(
        [
            "",
            "## Ranked stale proofs",
            "",
            "| Proof | Owner | Subsystem | Family | Reach | Churn | Reasons |",
            "| --- | --- | --- | --- | --- | ---: | --- |",
        ]
    )
    for proof in document["stale_owners"]:
        reasons = "; ".join(proof["stale_reasons"])
        lines.append(
            f"| `{proof['id']}` | `{proof['owner']}` | `{proof['subsystem']}` | "
            f"`{proof['owner_family']}` | "
            f"{proof['product_reach']} | {proof['source_churn']} | {reasons} |"
        )
    if changes["new"] or changes["removed"] or changes["changed"]:
        lines.extend(["", "## Upstream owner changes", ""])
        lines.extend(f"- New: `{owner}`" for owner in changes["new"])
        lines.extend(f"- Removed: `{owner}`" for owner in changes["removed"])
        lines.extend(
            f"- Changed proof owner: `{owner}`" for owner in changes["changed"]
        )
    return "\n".join(lines) + "\n"


def write_text_atomic(path: Path, contents: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        "w", encoding="utf-8", dir=path.parent, prefix=f".{path.name}.", delete=False
    ) as temporary:
        temporary.write(contents)
        temporary_path = Path(temporary.name)
    temporary_path.replace(path)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    report = subparsers.add_parser("report")
    report.add_argument("--repo-root", type=Path, default=Path.cwd())
    report.add_argument("--rive-runtime-dir", type=Path, required=True)
    report.add_argument("--registry", type=Path)
    report.add_argument("--output", type=Path, required=True)
    report.add_argument("--markdown-output", type=Path)
    options = parser.parse_args(argv)
    try:
        repo_root = options.repo_root.resolve()
        registry = options.registry or repo_root / "parity-evidence-proofs.json"
        document = build_report(repo_root, options.rive_runtime_dir.resolve(), registry)
        write_json_atomic(options.output, document)
        if options.markdown_output:
            write_text_atomic(options.markdown_output, render_markdown(document))
    except FreshnessError as error:
        print(f"parity-evidence-freshness error: {error}", file=sys.stderr)
        return 1
    print(
        "parity-evidence-freshness: "
        f"current={document['summary']['current']} stale={document['summary']['stale']} "
        f"total={document['summary']['total']}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
