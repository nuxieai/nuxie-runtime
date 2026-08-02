"""Aggregate the checked-in parity ledgers without adding verdicts."""

from __future__ import annotations

import collections
import re
import tomllib
from pathlib import Path
from typing import Any


SILVER_STATUS_LABELS = {
    "diverges": "divergent",
    "unsupported-feature": "unsupported",
}
D_SECTION = re.compile(r"^## D\b")
SECTION = re.compile(r"^## ")
D_ROW = re.compile(r"^(\d+)\.\s+(.*)$")
SENTENCE_END = re.compile(r"^(.+?[.!?])(?:\s+(?=[A-Z\[`*_])|$)")


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

    file_rows = table_rows(file_manifest, "file")
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
            "test_cases": sum(
                integer_value(row.get("test_case_count"), "file.test_case_count")
                for row in test_rows
            ),
        },
        "silver": {
            "min_exact": minimum_exact,
            "ratchet_met": silver_counts.get("exact", 0) >= minimum_exact,
            "status_counts": dict(sorted(silver_counts.items())),
            "total": len(silver_rows),
        },
        "golden": {"entries": len(golden_rows)},
        "frame_loop": {
            "file_status_counts": status_counts(frame_files),
            "files": len(frame_files),
            "gap_status_counts": gap_counts,
            "gaps": len(gap_rows),
            "member_status_counts": status_counts(frame_members),
            "members": len(frame_members),
        },
        "d_rows": d_rows,
    }


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


def render_ledger_scorecard(scorecard: dict[str, Any]) -> str:
    """Render one deterministic Markdown view suitable for a terminal or docs."""
    cpp_to_rust = scorecard["cpp_to_rust"]
    rust_to_cpp = scorecard["rust_to_cpp"]
    tests = scorecard["tests"]
    silver = scorecard["silver"]
    golden = scorecard["golden"]
    frame_loop = scorecard["frame_loop"]
    d_rows = scorecard["d_rows"]

    exact = silver["status_counts"].get("exact", 0)
    ratchet_state = "met" if silver["ratchet_met"] else "MISSED"
    pending_total = sum(
        len(paths) for paths in cpp_to_rust["pending_by_family"].values()
    )
    lines = [
        "# Parity scorecard",
        "",
        "## C++ → Rust file correspondence",
        "",
        f"Files: {cpp_to_rust['total']}",
        f"Status counts: {render_counts(cpp_to_rust['status_counts'])}",
        f"Named pending files: {pending_total}",
        "",
    ]
    for family, paths in sorted(cpp_to_rust["pending_by_family"].items()):
        lines.extend([f"### {family}", ""])
        lines.extend(f"- `{path}`" for path in sorted(paths))
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
    lines.append("")
    return "\n".join(lines)


def render_counts(counts: dict[str, int]) -> str:
    return "; ".join(f"`{key}`: {counts[key]}" for key in sorted(counts))
