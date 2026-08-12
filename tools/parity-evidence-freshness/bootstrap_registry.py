#!/usr/bin/env python3
"""Build the content-bound parity proof registry from reviewed evidence."""

from __future__ import annotations

import argparse
import functools
import json
import re
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path
from typing import Any

TOOL_DIR = Path(__file__).resolve().parent
if str(TOOL_DIR) not in sys.path:
    sys.path.insert(0, str(TOOL_DIR))

from freshness import (  # noqa: E402
    REGISTRY_SCHEMA,
    FreshnessError,
    infer_product_reach,
    selected_payload,
    sha256,
)

PARITY_SCORECARD_TOOL_DIR = TOOL_DIR.parent / "parity-scorecard"
if str(PARITY_SCORECARD_TOOL_DIR) not in sys.path:
    sys.path.insert(0, str(PARITY_SCORECARD_TOOL_DIR))

from ledger_scorecard import audit_record_section  # noqa: E402

CPP_AUDIT_ANCHOR = re.compile(
    r"cpp@(?P<ref>[0-9a-f]{8,40}):(?P<path>src/[A-Za-z0-9_./-]+):(?P<lines>\d+(?:-\d+)?(?:,\d+(?:-\d+)?)*)"
)
RUST_AUDIT_ANCHOR = re.compile(
    r"(?P<path>crates/[A-Za-z0-9_./-]+\.rs):(?P<lines>\d+(?:-\d+)?(?:,\d+(?:-\d+)?)*)"
)
LIFECYCLE_ANCHOR = re.compile(
    r"^(?P<root>cpp|rust):(?P<path>.+):(?P<start>\d+)(?:-(?P<end>\d+))?$"
)
PROBE_PATHS = (
    "tools/runtime-frame-loop-port/build-trace-runners.sh",
    "tools/runtime-frame-loop-port/capture_trace.py",
    "tools/runtime-frame-loop-port/source_fingerprint.py",
    "tools/runtime-frame-loop-port/summarize_trace.py",
)


@functools.lru_cache(maxsize=None)
def git_bytes(root: Path, revision: str, path: str) -> bytes:
    result = subprocess.run(
        ["git", "-C", str(root), "show", f"{revision}:{path}"],
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        diagnostic = result.stderr.decode("utf-8", errors="replace").strip()
        raise FreshnessError(
            f"cannot read captured source git:{revision}:{path}: {diagnostic}"
        )
    return result.stdout


def git_object_id(root: Path, revision: str, *, required: bool) -> str | None:
    result = subprocess.run(
        ["git", "-C", str(root), "rev-parse", "--verify", revision],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode == 0:
        return result.stdout.strip()
    if required:
        diagnostic = result.stderr.strip() or result.stdout.strip()
        raise FreshnessError(
            f"cannot resolve captured Git object {revision}: {diagnostic}"
        )
    return None


def behavioral_trace_tree(
    *,
    repo_root: Path,
    rust_revision: str,
    trace_rust_ref: str,
    recorded_tree: str | None,
) -> str:
    """Bind a pre-linearization trace ref to its durable main-history tree."""
    capture_tree = git_object_id(repo_root, f"{rust_revision}^{{tree}}", required=True)
    trace_tree = git_object_id(repo_root, f"{trace_rust_ref}^{{tree}}", required=False)
    expected_tree = recorded_tree or trace_tree
    if expected_tree is None:
        raise FreshnessError(
            "captured frame-loop trace ref is unavailable and has no recorded tree"
        )
    if re.fullmatch(r"[0-9a-f]{40}", expected_tree) is None:
        raise FreshnessError("captured frame-loop trace tree is not a Git object ID")
    if capture_tree != expected_tree:
        raise FreshnessError(
            "captured frame-loop Rust commit has a different tree from the trace"
        )
    if trace_tree is not None and trace_tree != expected_tree:
        raise FreshnessError("captured frame-loop trace ref has a different tree")
    return expected_tree


def read_toml_bytes(payload: bytes, label: str) -> dict[str, Any]:
    try:
        document = tomllib.loads(payload.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise FreshnessError(f"cannot parse {label}: {error}") from error
    if not isinstance(document, dict):
        raise FreshnessError(f"{label} must be a TOML table")
    return document


def line_ranges(value: str) -> list[tuple[int, int]]:
    result = []
    for item in value.split(","):
        start_text, separator, end_text = item.partition("-")
        start = int(start_text)
        end = int(end_text) if separator else start
        result.append((start, end))
    return result


def source_binding(
    *,
    identifier: str,
    path: str,
    payload: bytes,
    start: int | None = None,
    end: int | None = None,
    root: str | None = None,
    revision: str | None = None,
) -> dict[str, Any]:
    binding: dict[str, Any] = {"id": identifier, "path": path}
    if start is not None and end is not None:
        lines = payload.splitlines(keepends=True)
        selected = b"".join(lines[start - 1 : end])
        while (
            sum(
                b"".join(lines[index : index + end - start + 1]) == selected
                for index in range(max(0, len(lines) - (end - start + 1) + 1))
            )
            != 1
        ):
            if start == 1 and end == len(lines):
                raise FreshnessError(
                    f"cannot uniquely locate captured item {identifier}"
                )
            start = max(1, start - 1)
            end = min(len(lines), end + 1)
            selected = b"".join(lines[start - 1 : end])
        binding["selector"] = {"kind": "line-window", "start": start, "end": end}
    if root is not None:
        binding["root"] = root
    if revision is not None:
        binding["revision"] = revision
    selected = selected_payload(payload, binding)
    if not selected:
        raise FreshnessError(
            f"empty captured item {identifier} at {path}:{start}-{end}"
        )
    binding["sha256"] = sha256(selected)
    return binding


def audit_binding(*, path: str, row_id: str, payload: bytes) -> dict[str, Any]:
    binding: dict[str, Any] = {
        "path": path,
        "selector": {"kind": "audit-row", "id": row_id},
    }
    selected = selected_payload(payload, binding)
    if not selected:
        raise FreshnessError(f"audit record {path} does not contain {row_id}")
    binding["sha256"] = sha256(selected)
    return binding


def ledger_member_binding(
    *, path: str, member_id: str, payload: bytes
) -> dict[str, Any]:
    binding: dict[str, Any] = {
        "id": f"ledger-member:{member_id}",
        "path": path,
        "revision": "evidence",
        "selector": {"kind": "toml-member", "id": member_id},
    }
    selected = selected_payload(payload, binding)
    if not selected:
        raise FreshnessError(f"ownership ledger does not contain member {member_id}")
    binding["sha256"] = sha256(selected)
    return binding


def deduplicate(bindings: list[dict[str, Any]]) -> list[dict[str, Any]]:
    by_id: dict[str, dict[str, Any]] = {}
    for binding in bindings:
        prior = by_id.get(binding["id"])
        if prior is not None and prior != binding:
            raise FreshnessError(f"binding id {binding['id']} has conflicting locators")
        by_id[binding["id"]] = binding
    return [by_id[identifier] for identifier in sorted(by_id)]


def anchors_from_audit(
    *,
    section: str,
    repo_root: Path,
    upstream_root: Path,
    rust_revision: str,
    upstream_revision: str,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    cpp_items = []
    rust_items = []
    for match in CPP_AUDIT_ANCHOR.finditer(section):
        anchor_ref = match.group("ref")
        if not upstream_revision.startswith(anchor_ref):
            raise FreshnessError(
                f"C++ audit anchor {anchor_ref} does not match {upstream_revision}"
            )
        path = match.group("path")
        payload = git_bytes(upstream_root, upstream_revision, path)
        for start, end in line_ranges(match.group("lines")):
            cpp_items.append(
                source_binding(
                    identifier=f"cpp@{anchor_ref}:{path}:{start}-{end}",
                    path=path,
                    payload=payload,
                    start=start,
                    end=end,
                )
            )
    for match in RUST_AUDIT_ANCHOR.finditer(section):
        path = match.group("path")
        payload = git_bytes(repo_root, rust_revision, path)
        for start, end in line_ranges(match.group("lines")):
            rust_items.append(
                source_binding(
                    identifier=f"rust:{path}:{start}-{end}",
                    path=path,
                    payload=payload,
                    start=start,
                    end=end,
                )
            )
    return deduplicate(cpp_items), deduplicate(rust_items)


def structural_proofs(
    *,
    repo_root: Path,
    upstream_root: Path,
    manifest: dict[str, Any],
    rust_revision: str,
    upstream_revision: str,
) -> list[dict[str, Any]]:
    current_rows = set(manifest.get("current_audit_rows", []))
    result = []
    for row in sorted(manifest.get("file", []), key=lambda value: value["b6_row_id"]):
        row_id = str(row["b6_row_id"])
        if row_id not in current_rows:
            continue
        evidence_path = str(row["audit_record"])
        evidence_payload = git_bytes(repo_root, rust_revision, evidence_path)
        section = audit_record_section(evidence_payload.decode("utf-8"), row_id)
        cpp_items, rust_items = anchors_from_audit(
            section=section,
            repo_root=repo_root,
            upstream_root=upstream_root,
            rust_revision=rust_revision,
            upstream_revision=upstream_revision,
        )
        if not cpp_items or not rust_items:
            raise FreshnessError(f"{row_id}: audit has no C++ or Rust item anchors")
        owner = str(row["upstream"])
        rust_mapping_paths = sorted(
            source.strip().split("::", 1)[0]
            for source in str(row["rust_module"]).split(";")
            if source.strip()
        )
        cpp_paths = {item["path"] for item in cpp_items}
        rust_paths = {item["path"] for item in rust_items}
        if owner not in cpp_paths:
            raise FreshnessError(f"{row_id}: audit omits upstream owner {owner}")
        missing_mappings = sorted(set(rust_mapping_paths) - rust_paths)
        if missing_mappings:
            raise FreshnessError(
                f"{row_id}: audit omits Rust mappings {', '.join(missing_mappings)}"
            )
        owner_family = str(row["b6_cluster"])
        result.append(
            {
                "id": f"structural:{row_id}",
                "kind": "structural",
                "owner": owner,
                "owner_family": owner_family,
                "product_reach": infer_product_reach(owner, owner_family),
                "structural_claim": {
                    "row_id": row_id,
                    "verdict": str(row["b6_verdict"]),
                    "audit_record": evidence_path,
                },
                "rust_mapping_paths": rust_mapping_paths,
                "captured_rust_commit": rust_revision,
                "captured_evidence_commit": rust_revision,
                "upstream_ref": upstream_revision,
                "evidence": audit_binding(
                    path=evidence_path, row_id=row_id, payload=evidence_payload
                ),
                "cpp_items": cpp_items,
                "rust_items": rust_items,
                "probes": [],
                "fixtures": [],
            }
        )
    return result


def probe_bindings(repo_root: Path, rust_revision: str) -> list[dict[str, Any]]:
    return [
        source_binding(
            identifier=f"probe:{path}",
            path=path,
            payload=git_bytes(repo_root, rust_revision, path),
        )
        for path in PROBE_PATHS
    ]


def trace_fixtures(
    *,
    repo_root: Path,
    upstream_root: Path,
    upstream_revision: str,
    rust_revision: str,
    evidence_revision: str,
    trace: dict[str, Any],
    ledger: dict[str, Any],
) -> list[dict[str, Any]]:
    corpus = read_toml_bytes(
        git_bytes(repo_root, evidence_revision, "corpus.toml"), "captured corpus"
    )
    corpus_by_id = {str(row["id"]): row for row in corpus.get("file", [])}
    rows: list[dict[str, Any]] = []
    for fixture_id in trace.get("corpus", []):
        row = corpus_by_id.get(str(fixture_id))
        if row is None:
            raise FreshnessError(
                f"trace fixture {fixture_id} is absent from corpus.toml"
            )
        rows.append(row)
    rows.extend(ledger.get("trace_mechanism_fixture", []))
    rows.extend(ledger.get("trace_dirty_text_clean_guard_fixture", []))

    bindings = []
    for row in rows:
        fixture_id = str(row["id"])
        path = str(row["path"])
        payload = git_bytes(upstream_root, upstream_revision, path)
        actual = sha256(payload)
        declared = row.get("sha256") or row.get("expected_file_sha256")
        if declared is not None and declared != actual:
            raise FreshnessError(f"captured fixture hash mismatch for {fixture_id}")
        bindings.append(
            source_binding(
                identifier=f"fixture:{fixture_id}",
                path=path,
                payload=payload,
                root="upstream",
                revision="upstream",
            )
        )
        input_script = row.get("input_script")
        if input_script:
            script_path = str(input_script)
            script_payload = git_bytes(repo_root, rust_revision, script_path)
            bindings.append(
                source_binding(
                    identifier=f"fixture-script:{fixture_id}",
                    path=script_path,
                    payload=script_payload,
                    root="repo",
                    revision="source",
                )
            )
    return deduplicate(bindings)


def lifecycle_items(
    *,
    member: dict[str, Any],
    repo_root: Path,
    upstream_root: Path,
    rust_revision: str,
    upstream_revision: str,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    cpp_items = []
    rust_items = []
    lifecycle = member.get("lifecycle")
    if not isinstance(lifecycle, dict):
        raise FreshnessError(f"member {member.get('id')} has no lifecycle table")
    for phase, locators in sorted(lifecycle.items()):
        if not isinstance(locators, list):
            raise FreshnessError(
                f"member {member.get('id')} {phase} locators must be a list"
            )
        for locator in locators:
            match = LIFECYCLE_ANCHOR.fullmatch(str(locator))
            if match is None:
                raise FreshnessError(f"invalid lifecycle locator {locator}")
            language = match.group("root")
            path = match.group("path")
            start = int(match.group("start"))
            end = int(match.group("end") or start)
            root = upstream_root if language == "cpp" else repo_root
            revision = upstream_revision if language == "cpp" else rust_revision
            binding = source_binding(
                identifier=f"{language}:{phase}:{path}:{start}-{end}",
                path=path,
                payload=git_bytes(root, revision, path),
                start=start,
                end=end,
            )
            (cpp_items if language == "cpp" else rust_items).append(binding)
    return deduplicate(cpp_items), deduplicate(rust_items)


def behavioral_proofs(
    *,
    repo_root: Path,
    upstream_root: Path,
    rust_revision: str,
    evidence_revision: str,
    upstream_revision: str,
    recorded_trace_rust_tree: str | None,
) -> tuple[list[dict[str, Any]], str, str]:
    trace_path = "docs/runtime-frame-loop-trace.json"
    ledger_path = "docs/runtime-frame-loop-ownership.toml"
    trace_payload = git_bytes(repo_root, evidence_revision, trace_path)
    trace = json.loads(trace_payload)
    if trace.get("schema") != "nuxie-runtime-frame-loop-trace/v2":
        raise FreshnessError("captured frame-loop trace has an unsupported schema")
    if trace.get("upstream_ref") != upstream_revision:
        raise FreshnessError("captured frame-loop trace has a different upstream ref")
    trace_rust_ref = str(trace.get("rust_ref", ""))
    if not trace_rust_ref:
        raise FreshnessError("captured frame-loop trace has no Rust ref")
    trace_tree = behavioral_trace_tree(
        repo_root=repo_root,
        rust_revision=rust_revision,
        trace_rust_ref=trace_rust_ref,
        recorded_tree=recorded_trace_rust_tree,
    )
    ledger_payload = git_bytes(repo_root, evidence_revision, ledger_path)
    ledger = read_toml_bytes(ledger_payload, "captured frame-loop ownership ledger")
    shared_probes = probe_bindings(repo_root, rust_revision)
    fixtures = trace_fixtures(
        repo_root=repo_root,
        upstream_root=upstream_root,
        upstream_revision=upstream_revision,
        rust_revision=rust_revision,
        evidence_revision=evidence_revision,
        trace=trace,
        ledger=ledger,
    )
    evidence = {
        "path": trace_path,
        "sha256": sha256(trace_payload),
    }
    result = []
    for member in sorted(ledger.get("member", []), key=lambda value: value["id"]):
        cpp_items, rust_items = lifecycle_items(
            member=member,
            repo_root=repo_root,
            upstream_root=upstream_root,
            rust_revision=rust_revision,
            upstream_revision=upstream_revision,
        )
        owner = str(member["id"])
        owner_family = str(member["wave"])
        probes = [
            *shared_probes,
            ledger_member_binding(
                path=ledger_path, member_id=owner, payload=ledger_payload
            ),
        ]
        result.append(
            {
                "id": f"behavioral:{owner}",
                "kind": "behavioral",
                "owner": owner,
                "owner_family": owner_family,
                "product_reach": infer_product_reach(owner, owner_family),
                "captured_rust_commit": rust_revision,
                "captured_evidence_commit": evidence_revision,
                "upstream_ref": upstream_revision,
                "evidence": evidence,
                "cpp_items": cpp_items,
                "rust_items": rust_items,
                "probes": probes,
                "fixtures": fixtures,
            }
        )
    return result, trace_rust_ref, trace_tree


def build_registry(
    *,
    repo_root: Path,
    upstream_root: Path,
    structural_rust_commit: str,
    behavioral_rust_commit: str,
    behavioral_evidence_commit: str,
    behavioral_trace_rust_tree: str | None = None,
) -> dict[str, Any]:
    manifest = read_toml_bytes(
        git_bytes(
            repo_root, structural_rust_commit, "file-correspondence-manifest.toml"
        ),
        "captured file correspondence manifest",
    )
    upstream_ref = str(manifest["upstream_ref"])
    # Reproduction reads captured Git objects; checkout HEAD may be a newer pin.
    proofs = structural_proofs(
        repo_root=repo_root,
        upstream_root=upstream_root,
        manifest=manifest,
        rust_revision=structural_rust_commit,
        upstream_revision=upstream_ref,
    )
    behavioral, trace_rust_ref, trace_rust_tree = behavioral_proofs(
        repo_root=repo_root,
        upstream_root=upstream_root,
        rust_revision=behavioral_rust_commit,
        evidence_revision=behavioral_evidence_commit,
        upstream_revision=upstream_ref,
        recorded_trace_rust_tree=behavioral_trace_rust_tree,
    )
    proofs.extend(behavioral)
    proofs.sort(key=lambda proof: proof["id"])
    return {
        "schema": REGISTRY_SCHEMA,
        "upstream_ref": upstream_ref,
        "correspondence_manifest": "file-correspondence-manifest.toml",
        "behavioral_ledger": "docs/runtime-frame-loop-ownership.toml",
        "captures": {
            "structural_rust_commit": structural_rust_commit,
            "behavioral_rust_commit": behavioral_rust_commit,
            "behavioral_evidence_commit": behavioral_evidence_commit,
            "behavioral_trace_rust_ref": trace_rust_ref,
            "behavioral_trace_rust_tree": trace_rust_tree,
        },
        "proofs": proofs,
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


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--rive-runtime-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--structural-rust-commit")
    parser.add_argument("--behavioral-rust-commit")
    parser.add_argument("--behavioral-evidence-commit")
    parser.add_argument("--behavioral-trace-rust-tree")
    parser.add_argument("--check", action="store_true")
    options = parser.parse_args(argv)
    try:
        existing = None
        if options.check:
            existing = json.loads(options.output.read_text(encoding="utf-8"))
            captures = existing.get("captures", {})
            structural = str(captures.get("structural_rust_commit", ""))
            behavioral = str(captures.get("behavioral_rust_commit", ""))
            evidence = str(captures.get("behavioral_evidence_commit", ""))
            trace_tree = str(captures.get("behavioral_trace_rust_tree", ""))
        else:
            structural = str(options.structural_rust_commit or "")
            behavioral = str(options.behavioral_rust_commit or "")
            evidence = str(options.behavioral_evidence_commit or "")
            trace_tree = str(options.behavioral_trace_rust_tree or "")
        if not all((structural, behavioral, evidence)):
            raise FreshnessError("all three capture commits are required")
        generated = build_registry(
            repo_root=options.repo_root.resolve(),
            upstream_root=options.rive_runtime_dir.resolve(),
            structural_rust_commit=structural,
            behavioral_rust_commit=behavioral,
            behavioral_evidence_commit=evidence,
            behavioral_trace_rust_tree=trace_tree or None,
        )
        if options.check:
            if generated != existing:
                raise FreshnessError("checked-in proof registry is not reproducible")
        else:
            write_json_atomic(options.output, generated)
    except (FreshnessError, OSError, json.JSONDecodeError, KeyError) as error:
        print(f"parity-evidence-registry error: {error}", file=sys.stderr)
        return 1
    print(f"parity-evidence-registry: proofs={len(generated['proofs'])}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
