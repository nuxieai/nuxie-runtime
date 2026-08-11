"""Aggregate the checked-in parity ledgers without adding verdicts."""

from __future__ import annotations

import collections
import hashlib
import re
import subprocess
import tomllib
from pathlib import Path
from typing import Any


SILVER_STATUS_LABELS = {
    "diverges": "divergent",
    "unsupported-feature": "unsupported",
}
D_SECTION = re.compile(r"^## D\b")
X_SECTION = re.compile(r"^## Additive host-extension register$")
SECTION = re.compile(r"^## ")
D_ROW = re.compile(r"^(\d+)\.\s+(.*)$")
X_ROW = re.compile(r"^- \*\*(X\d+) — (.+?)\.\*\*\s*(.*)$")
SENTENCE_END = re.compile(r"^(.+?[.!?])(?:\s+(?=[A-Z\[`*_])|$)")
DECISION_REFERENCE = re.compile(r"(?<![A-Za-z0-9-])D\d+\b")
EXTENSION_REFERENCE = re.compile(r"(?<![A-Za-z0-9-])X\d+\b")


def aggregate_ledger_scorecard(repo_root: Path) -> dict[str, Any]:
    """Return the parity facts recorded by the repository's existing ledgers."""
    file_manifest = load_toml(repo_root / "file-correspondence-manifest.toml")
    rust_additions = load_toml(repo_root / "rust-additions.toml")
    test_manifest = load_toml(repo_root / "test-correspondence-manifest.toml")
    silver_corpus = load_toml(repo_root / "silver-corpus.toml")
    golden_corpus = load_toml(repo_root / "corpus.toml")
    frame_ownership = load_toml(
        repo_root / "docs" / "runtime-frame-loop-ownership.toml"
    )
    frame_gaps = load_toml(repo_root / "docs" / "runtime-frame-loop-gaps.toml")
    d_rows = parse_d_rows((repo_root / "docs" / "parity-gap-register.md").read_text())
    x_rows = parse_x_rows((repo_root / "docs" / "parity-gap-register.md").read_text())

    file_rows = table_rows(file_manifest, "file")
    evidence_by_upstream = validate_file_correspondence_manifest(
        repo_root, file_manifest, file_rows, d_rows, x_rows
    )
    pending_by_family: dict[str, list[str]] = collections.defaultdict(list)
    for row in file_rows:
        if row.get("status") != "pending":
            continue
        family = required_string(row, "b6_cluster", "pending file family")
        upstream = required_string(row, "upstream", "pending upstream file")
        pending_by_family[family].append(upstream)

    attributed = {
        module.strip()
        for row in file_rows
        for module in str(row.get("rust_module", "")).split(";")
        if module.strip()
    }
    addition_rows = table_rows(rust_additions, "addition")
    classified = {
        required_string(row, "path", "classified Rust path")
        for row in addition_rows
    }

    test_rows = table_rows(test_manifest, "file")
    test_case_total = sum(
        integer_value(row.get("test_case_count"), "file.test_case_count")
        for row in test_rows
    )
    covered_test_cases = sum(test_case_coverage(row) for row in test_rows)
    silver_rows = table_rows(silver_corpus, "case")
    silver_counts = collections.Counter(
        SILVER_STATUS_LABELS.get(str(row.get("status")), str(row.get("status")))
        for row in silver_rows
    )
    for status in ("divergent", "exact", "pending-scripted", "unsupported"):
        silver_counts.setdefault(status, 0)
    minimum_exact = integer_value(
        silver_corpus.get("corpus", {}).get("min_cpp_rust_exact"),
        "corpus.min_cpp_rust_exact",
    )
    golden_rows = table_rows(golden_corpus, "file")
    frame_files = table_rows(frame_ownership, "file")
    frame_members = table_rows(frame_ownership, "member")
    gap_rows = table_rows(frame_gaps, "gap")
    gap_counts = status_counts(gap_rows)
    for status in ("closed", "open"):
        gap_counts.setdefault(status, 0)
    gap_counts = dict(sorted(gap_counts.items()))

    return {
        "owner_proofs": build_owner_proof_report(
            required_string(file_manifest, "upstream_ref", "manifest upstream ref"),
            required_string(
                file_manifest, "audit_upstream_ref", "structural audit upstream ref"
            ),
            set(file_manifest.get("current_audit_rows", [])),
            evidence_by_upstream,
            file_rows,
            d_rows,
            x_rows,
        ),
        "cpp_to_rust": {
            "status_counts": status_counts(file_rows),
            "pending_by_family": {
                family: sorted(paths)
                for family, paths in sorted(pending_by_family.items())
            },
            "total": len(file_rows),
        },
        "rust_to_cpp": {
            "addition_category_counts": sorted_counts(
                str(row.get("category")) for row in addition_rows
            ),
            "attributed": len(attributed),
            "classified": len(classified),
            "total": len(attributed | classified),
        },
        "tests": {
            "file_status_counts": status_counts(test_rows),
            "files": len(test_rows),
            "test_cases": test_case_total,
            "covered_test_cases": covered_test_cases,
            "uncovered_test_cases": test_case_total - covered_test_cases,
        },
        "silver": {
            "min_exact": minimum_exact,
            "ratchet_met": silver_counts.get("exact", 0) >= minimum_exact,
            "status_counts": dict(sorted(silver_counts.items())),
            "total": len(silver_rows),
        },
        "golden": {
            "entries": len(golden_rows),
            "status_counts": status_counts(golden_rows),
        },
        "frame_loop": {
            "file_status_counts": status_counts(frame_files),
            "files": len(frame_files),
            "gap_status_counts": gap_counts,
            "gaps": len(gap_rows),
            "member_status_counts": status_counts(frame_members),
            "members": len(frame_members),
        },
        "d_rows": d_rows,
        "x_rows": x_rows,
    }


def validate_file_correspondence_manifest(
    repo_root: Path,
    document: dict[str, Any],
    file_rows: list[dict[str, Any]],
    d_rows: list[dict[str, str]],
    x_rows: list[dict[str, str]],
) -> dict[str, str]:
    """Reject owner rows whose verification state cannot support a proof claim."""
    if document.get("schema") != "nuxie-file-correspondence/v1":
        raise ValueError("file correspondence manifest has an unsupported schema")
    declared_count = integer_value(document.get("row_count"), "row_count")
    if declared_count != len(file_rows):
        raise ValueError(
            f"manifest declares {declared_count} owner rows but contains {len(file_rows)}"
        )
    verification_values = set(document.get("verification_values", []))
    status_values = set(document.get("status_values", []))
    audit_verdict_values = set(document.get("audit_verdict_values", []))
    upstream_ref = required_string(document, "upstream_ref", "manifest upstream ref")
    declared_current_rows = set(document.get("current_audit_rows", []))
    if not status_values or not verification_values or not audit_verdict_values:
        raise ValueError(
            "file correspondence manifest must declare status, verification, "
            "and structural verdict values"
        )
    known_decisions = {row["id"] for row in d_rows}
    known_extensions = {row["id"] for row in x_rows}
    seen_upstream: set[str] = set()
    seen_b6_rows: set[str] = set()
    evidence_by_upstream: dict[str, str] = {}
    for row in file_rows:
        upstream = required_string(row, "upstream", "upstream owner")
        if upstream in seen_upstream:
            raise ValueError(f"duplicate upstream owner {upstream}")
        seen_upstream.add(upstream)
        status = required_string(row, "status", f"{upstream} correspondence status")
        if status_values and status not in status_values:
            raise ValueError(
                f"{upstream}: invalid correspondence status {status!r}; "
                f"expected one of {sorted(status_values)}"
            )
        verification = row.get("verification")
        if not isinstance(verification, str) or not verification:
            raise ValueError(f"{upstream}: missing verification")
        if verification_values and verification not in verification_values:
            raise ValueError(
                f"{upstream}: invalid verification {verification!r}; "
                f"expected one of {sorted(verification_values)}"
            )
        rust_module = row.get("rust_module")
        if not isinstance(rust_module, str):
            raise ValueError(f"{upstream}: rust_module must be a string")
        if status != "pending" and not rust_module:
            raise ValueError(f"{upstream}: missing Rust owner mapping")
        b6_row_id = required_string(row, "b6_row_id", f"{upstream} B6 row id")
        if not re.fullmatch(r"B6-\d{4}", b6_row_id):
            raise ValueError(f"{upstream}: invalid B6 row id {b6_row_id!r}")
        if b6_row_id in seen_b6_rows:
            raise ValueError(f"duplicate B6 row id {b6_row_id}")
        seen_b6_rows.add(b6_row_id)
        b6_verdict = required_string(
            row, "b6_verdict", f"{upstream} structural verdict"
        )
        if b6_verdict not in audit_verdict_values:
            raise ValueError(
                f"{upstream}: invalid structural verdict {b6_verdict!r}; "
                f"expected one of {sorted(audit_verdict_values)}"
            )
        required_string(row, "b6_cluster", f"{upstream} structural cluster")
        audit_record = required_string(
            row, "audit_record", f"{upstream} structural evidence record"
        )
        audit_contents, evidence_locator, _ = resolve_evidence_record(
            repo_root, audit_record
        )
        if audit_contents is None:
            raise ValueError(f"{upstream}: evidence record does not exist: {audit_record}")
        if not audit_record_covers_row(audit_contents, b6_row_id):
            raise ValueError(
                f"{upstream}: evidence record {audit_record} does not contain "
                f"{b6_row_id}"
            )
        second_pass = Path(audit_record).name == "SECOND_PASS.md"
        substantiated = (
            second_pass_record_substantiates(audit_contents, b6_row_id, b6_verdict)
            if second_pass
            else audit_record_substantiates(
                audit_contents, b6_row_id, upstream, b6_verdict
            )
        )
        if not substantiated:
            raise ValueError(
                f"{upstream}: evidence record {audit_record} does not "
                f"substantiate {b6_verdict}"
            )
        if (
            b6_row_id in declared_current_rows
            and upstream_ref[:8]
            not in audit_record_section(audit_contents, b6_row_id)
        ):
            raise ValueError(
                f"{upstream}: current audit record does not cite {upstream_ref[:8]}"
            )
        if b6_row_id in declared_current_rows:
            reviewed_fingerprint = required_string(
                row,
                "audit_rust_source_sha256",
                f"{upstream} reviewed Rust source fingerprint",
            )
            actual_fingerprint = rust_source_fingerprint(repo_root, rust_module)
            if reviewed_fingerprint != actual_fingerprint:
                raise ValueError(
                    f"{upstream}: reviewed Rust source fingerprint mismatch; "
                    f"expected {reviewed_fingerprint}, got {actual_fingerprint}"
                )
        evidence_by_upstream[upstream] = evidence_locator
        note = required_string(row, "note", f"{upstream} evidence note")
        decision_references = set(DECISION_REFERENCE.findall(note))
        extension_references = set(EXTENSION_REFERENCE.findall(note))
        unknown_decisions = sorted(
            decision_references - known_decisions,
            key=lambda value: int(value[1:]),
        )
        if unknown_decisions:
            raise ValueError(
                f"{upstream}: unknown decision {unknown_decisions[0]}"
            )
        unknown_extensions = sorted(
            extension_references - known_extensions,
            key=lambda value: int(value[1:]),
        )
        if unknown_extensions:
            raise ValueError(
                f"{upstream}: unknown extension {unknown_extensions[0]}"
            )
        if row.get("status") == "divergent-by-decision" and not decision_references:
            raise ValueError(
                f"{upstream}: divergent-by-decision requires a D-row reference"
            )
    unknown_current_rows = sorted(declared_current_rows - seen_b6_rows)
    if unknown_current_rows:
        raise ValueError(
            f"current_audit_rows references unknown row {unknown_current_rows[0]}"
        )
    return evidence_by_upstream


def test_case_coverage(row: dict[str, Any]) -> int:
    count = integer_value(row.get("test_case_count"), "file.test_case_count")
    status = row.get("status")
    if status in {"ported-direct", "ported-differential"}:
        return count
    if status == "partial":
        covered = row.get("covered_test_cases", [])
        if not isinstance(covered, list) or not all(
            isinstance(name, str) and name for name in covered
        ):
            raise ValueError("partial test row covered_test_cases must be strings")
        if len(covered) > count:
            raise ValueError("partial test row covers more cases than it declares")
        return len(covered)
    return 0


def rust_source_fingerprint(repo_root: Path, rust_module: str) -> str:
    """Hash the path and bytes of every Rust source mapped to one reviewed owner."""
    sources = sorted(
        source.strip().split("::", 1)[0]
        for source in rust_module.split(";")
        if source.strip()
    )
    digest = hashlib.sha256(b"nuxie-reviewed-rust-sources/v1\0")
    for source in sources:
        path = repo_root / source
        if not path.is_file():
            raise ValueError(f"reviewed Rust source does not exist: {source}")
        digest.update(source.encode())
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def build_owner_proof_report(
    upstream_ref: str,
    audit_upstream_ref: str,
    current_audit_rows: set[str],
    evidence_by_upstream: dict[str, str],
    file_rows: list[dict[str, Any]],
    d_rows: list[dict[str, str]],
    x_rows: list[dict[str, str]],
) -> dict[str, Any]:
    """Derive proof dimensions without treating correspondence as verification."""
    known_decisions = {row["id"] for row in d_rows}
    known_extensions = {row["id"] for row in x_rows}
    by_upstream: dict[str, dict[str, Any]] = {}
    for row in file_rows:
        upstream = required_string(row, "upstream", "upstream owner")
        status = required_string(row, "status", f"{upstream} correspondence status")
        mapping = {
            "faithful": "mapped",
            "divergent-by-decision": "mapped",
            "partial": "partial",
            "pending": "pending",
        }.get(status, "unmapped")
        # File-classification verification is not executable behavioral proof.
        # Later differential lanes promote this only when they bind evidence to
        # the exact owner; until then the honest owner state is unverified.
        behavioral = "unverified"
        structural = {
            "ISOMORPHIC": "isomorphic",
            "ADAPTED": "adapted",
            "DIVERGENT": "divergent",
            "TRACKED-GAP": "tracked-gap",
            "N/A": "not-applicable",
        }.get(row.get("b6_verdict"), "unreviewed")
        audit_record = str(row.get("audit_record", ""))
        row_id = str(row.get("b6_row_id"))
        row_audit_upstream_ref = (
            upstream_ref if row_id in current_audit_rows else audit_upstream_ref
        )
        freshness = (
            "current" if row_audit_upstream_ref == upstream_ref else "stale"
        )
        note = str(row.get("note", ""))
        decisions = sorted(
            set(DECISION_REFERENCE.findall(note)) & known_decisions,
            key=lambda value: int(value[1:]),
        )
        extensions = sorted(
            set(EXTENSION_REFERENCE.findall(note)) & known_extensions,
            key=lambda value: int(value[1:]),
        )
        exception = (
            "intentional-extension"
            if extensions
            else "intentional-divergence"
            if status == "divergent-by-decision"
            else "none"
        )
        effective_state = (
            "stale"
            if freshness == "stale"
            else "known-divergent"
            if structural in {"divergent", "tracked-gap"}
            else behavioral
        )
        by_upstream[upstream] = {
            "upstream": upstream,
            "mapping": mapping,
            "structural": structural,
            "behavioral": behavioral,
            "verification": str(row.get("verification")),
            "freshness": freshness,
            "freshness_basis": (
                "row-audit-pin-and-rust-source"
                if row_id in current_audit_rows
                else "manifest-audit-pin"
            ),
            "freshness_reason": (
                "structural audit upstream ref differs from owner upstream ref"
                if freshness == "stale"
                else "structural audit pin and reviewed Rust source fingerprint match"
            ),
            "reviewed_rust_source_sha256": row.get("audit_rust_source_sha256"),
            "audit_record": audit_record,
            "structural_evidence": [evidence_by_upstream[upstream]],
            "behavioral_evidence": [],
            "decisions": decisions,
            "extensions": extensions,
            "exception": exception,
            "effective_state": effective_state,
        }
    dimension_counts = {
        dimension: sorted_counts(
            proof[dimension] for proof in by_upstream.values()
        )
        for dimension in (
            "mapping",
            "structural",
            "behavioral",
            "verification",
            "freshness",
            "exception",
        )
    }
    non_proven: dict[str, list[str]] = collections.defaultdict(list)
    for proof in by_upstream.values():
        if proof["effective_state"] != "behaviorally-proven":
            non_proven[proof["effective_state"]].append(proof["upstream"])
    non_proven_by_dimension = {
        "behaviorally-unverified": sorted(
            proof["upstream"]
            for proof in by_upstream.values()
            if proof["behavioral"] != "behaviorally-proven"
        ),
        "incomplete-mapping": sorted(
            proof["upstream"]
            for proof in by_upstream.values()
            if proof["mapping"] != "mapped"
        ),
        "known-divergent": sorted(
            proof["upstream"]
            for proof in by_upstream.values()
            if proof["structural"] in {"divergent", "tracked-gap"}
        ),
        "stale": sorted(
            proof["upstream"]
            for proof in by_upstream.values()
            if proof["freshness"] == "stale"
        ),
    }
    return {
        "upstream_ref": upstream_ref,
        "audit_upstream_ref": audit_upstream_ref,
        "by_upstream": dict(sorted(by_upstream.items())),
        "dimension_counts": dimension_counts,
        "freshness_counts": dimension_counts["freshness"],
        "effective_state_counts": sorted_counts(
            proof["effective_state"] for proof in by_upstream.values()
        ),
        "non_proven_by_state": {
            state: sorted(paths) for state, paths in sorted(non_proven.items())
        },
        "non_proven_by_dimension": non_proven_by_dimension,
    }


def owner_proof_document(scorecard: dict[str, Any]) -> dict[str, Any]:
    """Return the stable machine-readable owner proof and its evidence dimensions."""
    owner_proofs = scorecard["owner_proofs"]
    return {
        "schema": "nuxie-owner-parity-proof/v1",
        "upstream_ref": owner_proofs["upstream_ref"],
        "audit_upstream_ref": owner_proofs["audit_upstream_ref"],
        "owners": list(owner_proofs["by_upstream"].values()),
        "summary": {
            "dimension_counts": owner_proofs["dimension_counts"],
            "effective_state_counts": owner_proofs["effective_state_counts"],
            "non_proven_by_state": owner_proofs["non_proven_by_state"],
            "non_proven_by_dimension": owner_proofs["non_proven_by_dimension"],
        },
        "evidence_dimensions": {
            "tests": scorecard["tests"],
            "silver": scorecard["silver"],
            "golden": scorecard["golden"],
            "frame_loop": scorecard["frame_loop"],
            "decisions": scorecard["d_rows"],
            "extensions": scorecard["x_rows"],
        },
    }


def audit_record_covers_row(contents: str, row_id: str) -> bool:
    """Return whether an audit record names a row directly or in a B6 range."""
    if row_id in contents:
        return True
    target = int(row_id.removeprefix("B6-"))
    for start, end in re.findall(r"B6-(\d{4})[–-]B6-(\d{4})", contents):
        if int(start) <= target <= int(end):
            return True
    return False


def audit_record_substantiates(
    contents: str, row_id: str, upstream: str, verdict: str
) -> bool:
    """Validate the owner and verdict inside one local structural record."""
    row_marker = re.search(
        rf'(?:row_id:\s*"?|"row_id"\s*:\s*"){re.escape(row_id)}', contents
    )
    if row_marker is None:
        return False
    start = row_marker.start()
    end = len(contents)
    next_marker = re.search(
        r'(?:row_id:\s*"?|"row_id"\s*:\s*")B6-\d{4}',
        contents[row_marker.end() :],
    )
    if next_marker is not None:
        end = row_marker.end() + next_marker.start()
    record = contents[start:end]
    verdict_match = re.search(
        rf'(?:"verdict"\s*:\s*"{re.escape(verdict)}"|'
        rf'verdict:\s*{re.escape(verdict)}(?=[;,\s]))',
        record,
    )
    return upstream in record and verdict_match is not None


def audit_record_section(contents: str, row_id: str) -> str:
    """Return the owner-local section used for provenance assertions."""
    heading = contents.find(f"## {row_id}")
    if heading >= 0:
        next_heading = contents.find("\n## B6-", heading + len(row_id) + 3)
        return contents[heading : next_heading if next_heading >= 0 else len(contents)]
    row_marker = re.search(
        rf'(?:row_id:\s*"?|"row_id"\s*:\s*"){re.escape(row_id)}', contents
    )
    if row_marker is None:
        return ""
    next_marker = re.search(
        r'(?:row_id:\s*"?|"row_id"\s*:\s*")B6-\d{4}',
        contents[row_marker.end() :],
    )
    end = (
        row_marker.end() + next_marker.start()
        if next_marker is not None
        else len(contents)
    )
    return contents[row_marker.start() : end]


def second_pass_record_substantiates(
    contents: str, row_id: str, verdict: str
) -> bool:
    """Validate a row or compact row range in the B6 second-pass table."""
    return any(
        audit_record_covers_row(line, row_id) and verdict in line
        for line in contents.splitlines()
    )


def resolve_evidence_record(
    repo_root: Path, record: str
) -> tuple[str | None, str, bool]:
    """Resolve a checked-in record or a repository-approved historical citation."""
    local_path = repo_root / record
    if local_path.is_file():
        return local_path.read_text(), record, True
    try:
        history = subprocess.run(
            ["git", "-C", str(repo_root), "rev-list", "--all", "--", record],
            check=False,
            capture_output=True,
            text=True,
        )
    except OSError:
        return None, record, False
    if history.returncode != 0:
        return None, record, False
    for commit in history.stdout.splitlines():
        historical = subprocess.run(
            ["git", "-C", str(repo_root), "show", f"{commit}:{record}"],
            check=False,
            capture_output=True,
            text=True,
        )
        if historical.returncode == 0:
            return historical.stdout, f"git:{commit}:{record}", False
    return None, record, False


def load_toml(path: Path) -> dict[str, Any]:
    with path.open("rb") as source:
        document = tomllib.load(source)
    if not isinstance(document, dict):
        raise ValueError(f"{path} must contain a TOML table")
    return document


def table_rows(document: dict[str, Any], key: str) -> list[dict[str, Any]]:
    rows = document.get(key)
    if not isinstance(rows, list):
        raise ValueError(f"missing [[{key}]] rows")
    if not all(isinstance(row, dict) for row in rows):
        raise ValueError(f"[[{key}]] rows must be TOML tables")
    return rows


def status_counts(rows: list[dict[str, Any]]) -> dict[str, int]:
    return sorted_counts(str(row.get("status")) for row in rows)


def sorted_counts(values: Any) -> dict[str, int]:
    return dict(sorted(collections.Counter(values).items()))


def required_string(row: dict[str, Any], key: str, label: str) -> str:
    value = row.get(key)
    if not isinstance(value, str) or not value:
        raise ValueError(f"missing {label}")
    return value


def integer_value(value: Any, label: str) -> int:
    if not isinstance(value, int) or isinstance(value, bool):
        raise ValueError(f"{label} must be an integer")
    return value


def parse_d_rows(register: str) -> list[dict[str, str]]:
    """Extract each recorded D-row's first sentence as its summary."""
    in_d_section = False
    rows: list[tuple[int, list[str]]] = []
    current: tuple[int, list[str]] | None = None

    for line in register.splitlines():
        if not in_d_section:
            in_d_section = bool(D_SECTION.match(line))
            continue
        if SECTION.match(line):
            break
        match = D_ROW.match(line)
        if match:
            if current:
                rows.append(current)
            current = (int(match.group(1)), [match.group(2)])
        elif current and line.strip():
            current[1].append(line.strip())
    if current:
        rows.append(current)

    result = []
    for row_id, fragments in sorted(rows):
        full_text = " ".join(" ".join(fragments).split())
        if "SUPERSEDED" in full_text.upper():
            continue
        match = SENTENCE_END.match(full_text)
        summary = match.group(1) if match else full_text
        result.append({"id": f"D{row_id}", "summary": summary})
    return result


def parse_x_rows(register: str) -> list[dict[str, str]]:
    """Extract each additive host-extension row's name and first sentence."""
    in_x_section = False
    rows: list[tuple[str, str, list[str]]] = []
    current: tuple[str, str, list[str]] | None = None

    for line in register.splitlines():
        if not in_x_section:
            in_x_section = bool(X_SECTION.match(line))
            continue
        if SECTION.match(line):
            break
        match = X_ROW.match(line)
        if match:
            if current:
                rows.append(current)
            current = (match.group(1), match.group(2), [match.group(3)])
        elif current and line.strip():
            current[2].append(line.strip())
    if current:
        rows.append(current)

    result = []
    for row_id, name, fragments in sorted(rows, key=lambda row: int(row[0][1:])):
        full_text = " ".join(" ".join(fragments).split())
        match = SENTENCE_END.match(full_text)
        summary = match.group(1) if match else full_text
        result.append({"id": row_id, "name": name, "summary": summary})
    return result


def render_ledger_scorecard(scorecard: dict[str, Any]) -> str:
    """Render one deterministic Markdown view suitable for a terminal or docs."""
    cpp_to_rust = scorecard["cpp_to_rust"]
    rust_to_cpp = scorecard["rust_to_cpp"]
    tests = scorecard["tests"]
    silver = scorecard["silver"]
    golden = scorecard["golden"]
    frame_loop = scorecard["frame_loop"]
    d_rows = scorecard["d_rows"]
    x_rows = scorecard["x_rows"]
    owner_proofs = scorecard.get("owner_proofs")

    exact = silver["status_counts"].get("exact", 0)
    ratchet_state = "met" if silver["ratchet_met"] else "MISSED"
    pending_total = sum(
        len(paths) for paths in cpp_to_rust["pending_by_family"].values()
    )
    lines = [
        "# Parity scorecard",
        "",
        "## C++ → Rust correspondence inputs (non-authoritative)",
        "",
        f"Files: {cpp_to_rust['total']}",
        f"Raw classification counts: {render_counts(cpp_to_rust['status_counts'])}",
        f"Named pending files: {pending_total}",
        "",
    ]
    for family, paths in sorted(cpp_to_rust["pending_by_family"].items()):
        lines.extend([f"### {family}", ""])
        lines.extend(f"- `{path}`" for path in sorted(paths))
        lines.append("")

    if owner_proofs is not None:
        dimensions = owner_proofs["dimension_counts"]
        lines.extend(
            [
                "## C++ → Rust owner proof",
                "",
                f"Owners: {len(owner_proofs['by_upstream'])}",
                "Effective proof states: "
                + render_counts(owner_proofs["effective_state_counts"]),
                "Mapping states: " + render_counts(dimensions["mapping"]),
                "Structural states: " + render_counts(dimensions["structural"]),
                "Behavioral states: " + render_counts(dimensions["behavioral"]),
                "Verification states: " + render_counts(dimensions["verification"]),
                "Freshness states: " + render_counts(dimensions["freshness"]),
                "Exception states: " + render_counts(dimensions["exception"]),
                "",
            ]
        )
        for state, paths in owner_proofs["non_proven_by_dimension"].items():
            lines.extend([f"### {state} owners ({len(paths)})", ""])
            for upstream in paths:
                proof = owner_proofs["by_upstream"][upstream]
                exceptions = proof["decisions"] + proof["extensions"]
                suffix = (
                    "; exceptions=" + ",".join(exceptions) if exceptions else ""
                )
                lines.append(
                    f"- `{upstream}` — mapping={proof['mapping']}; "
                    f"structural={proof['structural']}; "
                    f"behavioral={proof['behavioral']}; "
                    f"verification={proof['verification']}; "
                    f"freshness={proof['freshness']}{suffix}"
                )
            lines.append("")

    lines.extend(
        [
            "## Rust → C++ attribution",
            "",
            f"Ledger coverage: {rust_to_cpp['total']} Rust files "
            f"({rust_to_cpp['attributed']} attributed by manifest inversion; "
            f"{rust_to_cpp['classified']} classified additions)",
            "Addition categories: "
            + render_counts(rust_to_cpp["addition_category_counts"]),
            "",
            "## Test correspondence",
            "",
            f"Files: {tests['files']}",
            f"Test cases: {tests['test_cases']}",
            f"Covered test cases: {tests['covered_test_cases']}/{tests['test_cases']}",
            f"Uncovered test cases: {tests['uncovered_test_cases']}",
            f"Status counts: {render_counts(tests['file_status_counts'])}",
            "",
            "## Silver corpus",
            "",
            f"Entries: {silver['total']}",
            f"Status counts: {render_counts(silver['status_counts'])}",
            f"Exact ratchet: {exact}/{silver['min_exact']} ({ratchet_state})",
            "",
            "## Golden corpus",
            "",
            f"Entries: {golden['entries']}",
            f"Status counts: {render_counts(golden['status_counts'])}",
            "",
            "## Runtime frame-loop ledger",
            "",
            f"Files: {frame_loop['files']} "
            f"({render_counts(frame_loop['file_status_counts'])})",
            f"Members: {frame_loop['members']} "
            f"({render_counts(frame_loop['member_status_counts'])})",
            f"Gaps: {frame_loop['gaps']} "
            f"({render_counts(frame_loop['gap_status_counts'])})",
            "",
            "## D-row register — approved divergences and adaptations",
            "",
            f"Rows: {len(d_rows)}",
            "",
        ]
    )
    lines.extend(
        f"- {row['id']} — {row['summary']}"
        for row in sorted(d_rows, key=lambda row: int(row["id"][1:]))
    )
    lines.extend(
        [
            "",
            "## Additive host-extension register",
            "",
            f"Rows: {len(x_rows)}",
            "",
        ]
    )
    lines.extend(
        f"- {row['id']} — **{row['name']}.** {row['summary']}"
        for row in sorted(x_rows, key=lambda row: int(row["id"][1:]))
    )
    lines.append("")
    return "\n".join(lines)


def render_counts(counts: dict[str, int]) -> str:
    return "; ".join(f"`{key}`: {counts[key]}" for key in sorted(counts))
