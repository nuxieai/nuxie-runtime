#!/usr/bin/env python3
"""Generate the checked-in whole-Metal renderer progress dashboard."""

from __future__ import annotations

import argparse
import csv
import html
import tomllib
from collections import Counter
from pathlib import Path


STATUS_LABEL = {
    "ported": "Ported",
    "partial": "Partial",
    "missing": "Missing",
    "verified": "Verified",
    "in-progress": "In progress",
    "pending": "Pending",
    "exact": "Exact oracle",
    "green": "Green",
    "amber": "Attention",
    "active": "Active",
    "queued": "Queued",
    "frozen": "Frozen",
    "ready": "Ready",
    "translated": "Translated",
    "reviewed": "Reviewed",
    "fixed": "Fixed",
    "compiled": "Legacy compiled",
}


def escape(value: object) -> str:
    return html.escape(str(value), quote=True)


def load_toml(path: Path) -> dict:
    with path.open("rb") as source:
        return tomllib.load(source)


def parse_line_map(path: Path) -> list[dict[str, object]]:
    with path.open(newline="", encoding="utf-8") as source:
        rows = list(csv.DictReader(source, delimiter="\t"))
    parsed: list[dict[str, object]] = []
    for row in rows:
        start_text, end_text = row["lines"].split("-", 1)
        start = int(start_text)
        end = int(end_text)
        parsed.append({**row, "start": start, "end": end, "length": end - start + 1})
    return parsed


def parse_tsv(path: Path) -> list[dict[str, str]]:
    with path.open(newline="", encoding="utf-8") as source:
        return [dict(row) for row in csv.DictReader(source, delimiter="\t")]


def source_label(path: str) -> str:
    return Path(path).name


def friendly_id(value: str) -> str:
    words = value.removeprefix("native-metal-").replace("-generic-atomic", "")
    return words.replace("-", " ").title()


def status_chip(status: str) -> str:
    label = STATUS_LABEL.get(status, status.replace("-", " ").title())
    return f'<span class="status status-{escape(status)}">{escape(label)}</span>'


def source_summary(rows: list[dict[str, object]]) -> str:
    sections: list[str] = []
    for upstream in dict.fromkeys(str(row["upstream_file"]) for row in rows):
        source_rows = [row for row in rows if row["upstream_file"] == upstream]
        total = sum(int(row["length"]) for row in source_rows)
        counts = Counter()
        for row in source_rows:
            counts[str(row["status"])] += int(row["length"])
        segments = "".join(
            f'<span class="bar-part bar-{status}" style="width:{counts[status] / total * 100:.5f}%" '
            f'title="{escape(STATUS_LABEL.get(status, status))}: {counts[status]} lines"></span>'
            for status in ("ported", "partial", "missing")
            if counts[status]
        )
        range_counts = Counter(str(row["status"]) for row in source_rows)
        sections.append(
            f"""
            <article class="source-card">
              <div class="source-card-head">
                <div><h3>{escape(source_label(upstream))}</h3><code>{total:,} pinned lines</code></div>
                <strong>{counts['ported'] / total * 100:.1f}% source-closed</strong>
              </div>
              <div class="stacked-bar" aria-label="{escape(source_label(upstream))} source status">{segments}</div>
              <div class="bar-legend">
                <span><i class="dot dot-ported"></i>{counts['ported']:,} ported lines ({range_counts['ported']} ranges)</span>
                <span><i class="dot dot-partial"></i>{counts['partial']:,} partial lines ({range_counts['partial']} ranges)</span>
                <span><i class="dot dot-missing"></i>{counts['missing']:,} missing lines ({range_counts['missing']} ranges)</span>
              </div>
            </article>
            """
        )
    return "".join(sections)


def field_summary(rows: list[dict[str, str]]) -> str:
    cards = []
    for cpp_type in dict.fromkeys(row["cpp_type"] for row in rows):
        type_rows = [row for row in rows if row["cpp_type"] == cpp_type]
        mapped = sum(row["mapping_status"] == "prepared" for row in type_rows)
        translated = sum(
            row["translation_status"] in {"translated", "verified"}
            for row in type_rows
        )
        mapping_review = [
            row["cpp_field"]
            for row in type_rows
            if row["mapping_status"] == "review-needed"
        ]
        translation_pending = [
            row["cpp_field"]
            for row in type_rows
            if row["translation_status"] == "pending"
        ]
        percent = translated / len(type_rows) * 100
        details = []
        if mapping_review:
            details.append(
                f'Mapping review: <code>{escape(", ".join(mapping_review))}</code>'
            )
        if translation_pending:
            details.append(
                f'Translation pending: <code>{escape(", ".join(translation_pending))}</code>'
            )
        review_text = (
            f'<p class="field-review">{" · ".join(details)}</p>'
            if details
            else '<p class="field-ready">Every declared field is mapped and translated.</p>'
        )
        cards.append(
            f"""
            <article class="field-card">
              <div class="source-card-head"><div><h3>{escape(cpp_type)}</h3><code>{len(type_rows)} declared fields</code></div><strong>{mapped}/{len(type_rows)} mapped · {translated}/{len(type_rows)} translated</strong></div>
              <div class="stacked-bar" aria-label="{escape(cpp_type)} field translation"><span class="bar-part bar-ported" style="width:{percent:.5f}%"></span><span class="bar-part bar-missing" style="width:{100 - percent:.5f}%"></span></div>
              {review_text}
            </article>
            """
        )
    return "".join(cards)


def configuration_summary(rows: list[dict[str, str]]) -> str:
    cards = []
    for upstream_file in dict.fromkeys(row["upstream_file"] for row in rows):
        source_rows = [row for row in rows if row["upstream_file"] == upstream_file]
        prepared = sum(row["mapping_status"] == "prepared" for row in source_rows)
        translated = sum(
            row["translation_status"] in {"translated", "verified"}
            for row in source_rows
        )
        review = [
            row["block_id"]
            for row in source_rows
            if row["mapping_status"] == "review-needed"
        ]
        block_count = len({row["block_id"] for row in source_rows})
        percent = prepared / len(source_rows) * 100
        review_text = (
            f'<p class="field-review">Open branches: <code>{escape(", ".join(review))}</code></p>'
            if review
            else '<p class="field-ready">Every conditional branch entry is prepared.</p>'
        )
        cards.append(
            f"""
            <article class="field-card">
              <div class="source-card-head"><div><h3>{escape(source_label(upstream_file))}</h3><code>{block_count} blocks · {len(source_rows)} branch entries</code></div><strong>{prepared}/{len(source_rows)} mapped · {translated}/{len(source_rows)} translated</strong></div>
              <div class="stacked-bar" aria-label="{escape(source_label(upstream_file))} configuration preparation"><span class="bar-part bar-ported" style="width:{percent:.5f}%"></span><span class="bar-part bar-missing" style="width:{100 - percent:.5f}%"></span></div>
              {review_text}
            </article>
            """
        )
    return "".join(cards)


def dependency_summary(rows: list[dict[str, str]]) -> str:
    return "".join(
        f"""
        <article class="field-card">
          <div class="source-card-head"><div><h3>{escape(source_label(row['upstream_file']))}</h3><code>{escape(row['translation_unit'])}</code></div><strong>{escape(row['mapping_status'])} · {escape(row['translation_status'])}</strong></div>
          <div class="stacked-bar" aria-label="{escape(source_label(row['upstream_file']))} whole-file translation"><span class="bar-part bar-ported" style="width:{100 if row['translation_status'] in {'translated', 'verified'} else 0}%"></span><span class="bar-part bar-missing" style="width:{0 if row['translation_status'] in {'translated', 'verified'} else 100}%"></span></div>
          <p class="field-review">Lifetime coverage: <code>{escape(row['field_coverage'])}</code> · target: <code>{escape(row['translation_target'])}</code></p>
        </article>
        """
        for row in rows
    )


def include_summary(rows: list[dict[str, str]]) -> str:
    kinds = Counter(row["resolution_kind"] for row in rows)
    tokens = len({row["include_token"] for row in rows})
    files = len({row["upstream_file"] for row in rows})
    prepared = sum(
        row["mapping_status"] in {"prepared", "existing-complete"} for row in rows
    )
    directives = Counter(row["directive"] for row in rows)
    exact_globals = sum(
        row["resolution_kind"] == "upstream-global-source"
        and row["mapping_status"] == "existing-complete"
        for row in rows
    )
    return f"""
    <article class="field-card">
      <div class="source-card-head"><div><h3>Direct include/import graph</h3><code>{len(rows)} occurrences · {tokens} tokens · {files} files</code></div><strong>{prepared}/{len(rows)} mapped</strong></div>
      <div class="stacked-bar" aria-label="direct include and import correspondence"><span class="bar-part bar-ported" style="width:{prepared / len(rows) * 100:.5f}%"></span><span class="bar-part bar-missing" style="width:{(len(rows) - prepared) / len(rows) * 100:.5f}%"></span></div>
      <p class="field-ready">{directives['include']} #include · {directives['import']} #import · {kinds['campaign-source']} campaign · {exact_globals}/{kinds['upstream-global-source']} upstream-global with exact Rust correspondence · {kinds['generated-shader-source']} generated-source · {kinds['generated-shader-artifact']} generated-artifact · {kinds['toolchain-header']} toolchain occurrences.</p>
    </article>
    """


def authority_graph_summary(
    source_dependencies: list[dict[str, str]],
    dispatch_rows: list[dict[str, str]],
    build_rows: list[dict[str, str]],
) -> str:
    dependency_occurrences = sum(
        int(row["occurrence_count"]) for row in source_dependencies
    )
    source_only = [
        row
        for row in source_dependencies
        if row["unit_edge_status"] == "source-only-dependency"
    ]
    source_only_edges = {
        (row["source_unit"], row["dependency_unit"]) for row in source_only
    }
    source_only_occurrences = sum(int(row["occurrence_count"]) for row in source_only)
    scc_members = {
        row["translation_unit"]
        for row in dispatch_rows
        if row["source_dependency_scc"] != "-"
    }
    make_rows = sum(row["authority_kind"] == "make-rule-family" for row in build_rows)
    python_rows = len(build_rows) - make_rows
    return f"""
    <article class="field-card">
      <div class="source-card-head"><div><h3>Source dependencies</h3><code>{len(source_dependencies)} normalized edges · {dependency_occurrences} occurrences</code></div><strong>cycle-allowing</strong></div>
      <p class="field-ready">{len(source_only_edges)} source-only unit edges across {source_only_occurrences} occurrences · {len(scc_members)} units in the real SCC.</p>
    </article>
    <article class="field-card">
      <div class="source-card-head"><div><h3>Dispatch prerequisites</h3><code>{len(dispatch_rows)} translation units</code></div><strong>acyclic</strong></div>
      <p class="field-ready">Scheduling order is intentionally separate from the complete source include graph; every prerequisite ordinal strictly precedes its consumer.</p>
    </article>
    <article class="field-card">
      <div class="source-card-head"><div><h3>Build behavior</h3><code>{len(build_rows)} branch rows</code></div><strong>required</strong></div>
      <p class="field-ready">{make_rows} Make rule families · {python_rows} Python If/IfExp branches, including all Apple Metal and SPIR-V/WGSL/D3D dispositions.</p>
    </article>
    """


def convention_summary(rows: list[dict[str, str]]) -> str:
    return "".join(
        f"""
        <article class="field-card">
          <div class="source-card-head"><h3>{escape(row['convention'].replace('-', ' ').title())}</h3>{status_chip(row['status'])}</div>
          <p class="convention-shape">{escape(row['cpp_shape'])}</p>
          <p>{escape(row['invariant'])}</p>
        </article>
        """
        for row in rows
    )


def phase_rail(phases: list[dict]) -> str:
    return "".join(
        f"""
        <li class="phase phase-{escape(phase['status'])}">
          <span class="phase-marker" aria-hidden="true"></span>
          <div><strong>{escape(phase['label'])}</strong>{status_chip(phase['status'])}<p>{escape(phase['detail'])}</p></div>
        </li>
        """
        for phase in phases
    )


def translation_unit_summary(units: list[dict], repo_root: Path) -> str:
    def order(unit: dict) -> tuple[int, str]:
        ordinal = unit.get("dispatch_ordinal")
        return (int(ordinal) if ordinal is not None else 10_000, str(unit["id"]))

    cards = []
    for unit in sorted(units, key=order):
        receipts = [
            str(unit.get(name, "unrecorded"))
            for name in (
                "translation_receipt",
                "source_review_receipt",
                "ownership_review_receipt",
                "fix_receipt",
                "compile_receipt",
                "verification_receipt",
            )
        ]
        suffixes = (
            "translation.toml",
            "source-review.toml",
            "ownership-review.toml",
            "fix.toml",
            "compile.toml",
            "verification.toml",
        )
        canonical_receipts = [
            f"docs/metal-port-receipts/{unit['id']}.{suffix}" for suffix in suffixes
        ]
        loop_complete = (
            receipts[:4] == canonical_receipts[:4]
            and all((repo_root / receipt).is_file() for receipt in receipts[:4])
            and unit.get("open_findings") == 0
        )
        if loop_complete and unit.get("status") in {"fixed", "compiled", "verified"}:
            gate_status = "green"
        elif unit.get("status") != "pending" or unit.get("worker_claim") != "unclaimed":
            gate_status = "in-progress"
        else:
            gate_status = "pending"
        ordinal = unit.get("dispatch_ordinal", "legacy")
        targets = len(unit.get("rust_targets", [])) + len(unit.get("artifact_targets", []))
        cards.append(
            f"""
            <article class="unit-card unit-{escape(gate_status)}">
              <div class="suite-head"><code>#{escape(ordinal)} · {escape(unit['id'])}</code>{status_chip(gate_status)}</div>
              <p>{len(unit.get('sources', []))} complete source files · {targets} reserved outputs</p>
              <dl>
                <dt>Translator</dt><dd>{escape(unit.get('worker_role', 'missing'))} · {escape(unit.get('worker_claim', 'unclaimed'))}</dd>
                <dt>Source review</dt><dd>{escape(unit.get('source_reviewer_role', 'missing'))} · {escape(receipts[1])}</dd>
                <dt>Ownership review</dt><dd>{escape(unit.get('ownership_reviewer_role', 'missing'))} · {escape(receipts[2])}</dd>
                <dt>Translation</dt><dd>{escape(receipts[0])}</dd>
                <dt>Fix</dt><dd>{escape(receipts[3])} · {escape(unit.get('open_findings', 'unrecorded'))} open findings</dd>
              </dl>
            </article>
            """
        )
    return "".join(cards)


def validation_suites(suites: list[dict]) -> str:
    return "".join(
        f"""
        <article class="suite suite-{escape(suite['status'])}">
          <div class="suite-head"><code>{escape(suite['id'])}</code>{status_chip(suite['status'])}</div>
          <h3>{escape(suite['label'])}</h3>
          <p>{escape(suite['current'])}</p>
          <dl>
            <dt>Authority</dt><dd>{escape(suite['authority'])}</dd>
            <dt>Scope</dt><dd>{escape(suite['scope'])}</dd>
            <dt>Command</dt><dd><code>{escape(suite['command'])}</code></dd>
            <dt>Green when</dt><dd>{escape(suite['acceptance'])}</dd>
          </dl>
        </article>
        """
        for suite in suites
    )


def line_map_table(rows: list[dict[str, object]]) -> str:
    body = []
    for row in rows:
        remaining = str(row["remaining"])
        remaining_html = "—" if remaining == "-" else escape(remaining)
        body.append(
            f"""
            <tr data-status="{escape(row['status'])}">
              <td><code>{escape(source_label(str(row['upstream_file'])))}</code><small>:{escape(row['lines'])}</small></td>
              <td><strong>{escape(row['symbol'])}</strong></td>
              <td>{status_chip(str(row['status']))}</td>
              <td>{remaining_html}</td>
              <td><code>{escape(row['rust_owner'])}</code></td>
            </tr>
            """
        )
    return "".join(body)


def ownership_rows(owners: list[dict]) -> str:
    renderer_owners = [owner for owner in owners if str(owner["id"]).startswith("renderer.")]
    return "".join(
        f"""
        <details class="owner owner-{escape(owner['status'])}">
          <summary><span>{escape(owner['id'])}</span>{status_chip(owner['status'])}</summary>
          <p>{escape(owner['rule'])}</p>
          <div class="owner-meta"><strong>Required evidence</strong><span>{escape(' · '.join(owner.get('required_tests', [])))}</span></div>
        </details>
        """
        for owner in renderer_owners
    )


def report_cards(reports: list[dict], page_path: Path, repo_root: Path) -> str:
    cards = []
    for report in reports:
        commands = report.get("commands", [])
        results = report.get("results", [])
        command_block = ""
        if commands or results:
            pairs = []
            for index in range(max(len(commands), len(results))):
                command = commands[index] if index < len(commands) else ""
                result = results[index] if index < len(results) else ""
                pairs.append(
                    f'<div class="run"><code>{escape(command)}</code><span>{escape(result)}</span></div>'
                )
            command_block = f'<div class="runs">{"".join(pairs)}</div>'
        evidence_path = repo_root / report["evidence"]
        evidence_href = Path("..").joinpath(evidence_path.relative_to(repo_root)).as_posix()
        commit = f'<code class="commit">{escape(report["commit"])}</code>' if report.get("commit") else ""
        cards.append(
            f"""
            <article class="report report-{escape(report['status'])}">
              <div class="report-head"><time>{escape(report['date'])}</time>{status_chip(report['status'])}</div>
              <h3>{escape(report['title'])}</h3>
              <p>{escape(report['summary'])}</p>
              {command_block}
              <footer>{commit}<a href="{escape(evidence_href)}">Open evidence</a></footer>
            </article>
            """
        )
    return "".join(cards)


def gallery(corpus_paths: list[Path], repo_root: Path) -> str:
    entries: list[dict] = []
    seen: set[str] = set()
    for corpus_path in corpus_paths:
        for entry in load_toml(corpus_path).get("entry", []):
            reference = entry["reference"]
            if reference in seen or not (repo_root / reference).is_file():
                continue
            seen.add(reference)
            entries.append(entry)
    cards = []
    for entry in entries:
        image_href = Path("..").joinpath(entry["reference"]).as_posix()
        stream_href = Path("..").joinpath(entry["stream"]).as_posix()
        exact = entry["max_channel_delta"] == 0 and entry["max_different_pixels"] == 0
        contract = (
            "byte-exact 0/0"
            if exact
            else f"≤{entry['max_channel_delta']} LSB · ≤{entry['max_different_pixels']} pixels"
        )
        cards.append(
            f"""
            <figure class="gallery-card">
              <a href="{escape(image_href)}"><img src="{escape(image_href)}" alt="{escape(friendly_id(entry['id']))} Metal reference output"></a>
              <figcaption>
                <strong>{escape(friendly_id(entry['id']))}</strong>
                <span>{status_chip(entry['status'])}<code>{escape(contract)}</code></span>
                <a href="{escape(stream_href)}">Source stream</a>
              </figcaption>
            </figure>
            """
        )
    return "".join(cards)


def render(repo_root: Path) -> str:
    progress = load_toml(repo_root / "docs/metal-renderer-progress.toml")
    rows = parse_line_map(repo_root / "docs/render-context-metal-file-map.tsv")
    ownership = load_toml(repo_root / "docs/metal-port-ownership.toml")
    manifest = load_toml(repo_root / "docs/metal-port-manifest.toml")
    field_rows = parse_tsv(repo_root / manifest["render_context_field_map"])
    configuration_rows = parse_tsv(
        repo_root / manifest["preprocessor_authority"]
    )
    dependency_rows = parse_tsv(repo_root / manifest["render_context_dependency_map"])
    include_rows = parse_tsv(repo_root / manifest["direct_include_authority"])
    source_dependency_rows = parse_tsv(
        repo_root / manifest["source_dependency_authority"]
    )
    dispatch_rows = parse_tsv(
        repo_root / manifest["dispatch_prerequisite_authority"]
    )
    build_branch_rows = parse_tsv(repo_root / manifest["build_branch_authority"])
    convention_rows = parse_tsv(repo_root / manifest["translation_conventions"])
    metal_sources = [
        source
        for source in manifest.get("source", [])
        if source["upstream"].startswith("renderer/src/metal/")
        or source["upstream"].startswith("renderer/include/rive/renderer/metal/")
        or source["upstream"] == "renderer/include/rive/renderer/buffer_ring.hpp"
        or source["upstream"].startswith("renderer/src/shaders/metal/")
    ]
    source_statuses = Counter(source["status"] for source in metal_sources)
    translation_units = manifest.get("translation_unit", [])
    total_campaign_files = sum(len(unit.get("sources", [])) for unit in translation_units)
    translated_campaign_files = 0
    for unit in translation_units:
        sources = unit.get("sources", [])
        rust_targets = unit.get("rust_targets", [])
        if rust_targets:
            # Most units have a complete one-to-one source/target list. The
            # shader batch grows that list file-by-file during the bulk pass,
            # so count already-materialized source-shaped targets without
            # waiting for the entire 40-file batch to close.
            translated_campaign_files += min(
                len(sources),
                sum((repo_root / target).is_file() for target in rust_targets),
            )
        elif unit.get("status") in {"translated", "reviewed", "fixed", "compiled", "verified"}:
            # Artifact batches, such as the globally coupled shader build,
            # advance atomically because outputs do not map one-to-one to inputs.
            translated_campaign_files += len(sources)
    claimed_campaign_files = sum(
        len(unit.get("sources", []))
        for unit in translation_units
        if unit.get("status") != "pending" or unit.get("worker_claim") != "unclaimed"
    )
    owner_statuses = Counter(
        owner["status"]
        for owner in ownership.get("owner", [])
        if owner["id"].startswith("renderer.")
    )
    total_lines = sum(int(row["length"]) for row in rows)
    ported_lines = sum(int(row["length"]) for row in rows if row["status"] == "ported")
    partial_lines = sum(int(row["length"]) for row in rows if row["status"] == "partial")
    missing_lines = sum(int(row["length"]) for row in rows if row["status"] == "missing")
    mapped_fields = sum(row["mapping_status"] == "prepared" for row in field_rows)
    translated_fields = sum(
        row["translation_status"] in {"translated", "verified"}
        for row in field_rows
    )
    mapped_configurations = sum(
        row["mapping_status"] == "prepared" for row in configuration_rows
    )
    translated_configurations = sum(
        row["translation_status"] in {"translated", "verified"}
        for row in configuration_rows
    )
    mapped_dependencies = sum(
        row["mapping_status"] == "prepared" for row in dependency_rows
    )
    translated_dependencies = sum(
        row["translation_status"] in {"translated", "verified"}
        for row in dependency_rows
    )
    mapped_includes = sum(
        row["mapping_status"] in {"prepared", "existing-complete"}
        for row in include_rows
    )
    configuration_blocks = len({row["block_id"] for row in configuration_rows})
    frozen_conventions = sum(
        row["status"] in {"frozen", "verified"} for row in convention_rows
    )
    corpus_config = progress["corpus"]
    corpus = load_toml(repo_root / corpus_config["manifest"])
    corpus_entries = corpus.get("entry", [])
    corpus_ids = [entry["id"] for entry in corpus_entries]
    if len(corpus_ids) != len(set(corpus_ids)):
        raise ValueError("renderer corpus entry IDs are not unique")
    mode_counts = Counter(entry.get("mode") for entry in corpus_entries)
    expected_counts = {
        "total": int(corpus_config["expected_total_rows"]),
        "clockwise-atomic": int(corpus_config["expected_clockwise_atomic_rows"]),
        "msaa": int(corpus_config["expected_msaa_rows"]),
    }
    actual_counts = {
        "total": len(corpus_entries),
        "clockwise-atomic": mode_counts["clockwise-atomic"],
        "msaa": mode_counts["msaa"],
    }
    if actual_counts != expected_counts:
        raise ValueError(
            f"renderer corpus inventory drifted: expected {expected_counts}, got {actual_counts}"
        )
    corpus_paths = [
        repo_root / "tools/metal-port/tracer-corpus-atomic.toml",
        repo_root / "tools/metal-port/tracer-corpus.toml",
    ]
    return f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="icon" href="data:,">
<title>Whole Metal Renderer Port Progress</title>
<style>
:root {{ color-scheme: light dark; --bg:#f4f1ea; --panel:#fffdf8; --ink:#18211d; --muted:#66706a; --line:#d7d2c7; --green:#16845b; --green-soft:#d9f2e7; --amber:#b36b00; --amber-soft:#fff0ce; --red:#bd3d3d; --red-soft:#fde1df; --blue:#315b9d; --queued:#8a8f8c; --shadow:0 12px 36px rgba(26,36,31,.08); }}
@media (prefers-color-scheme:dark) {{ :root {{ --bg:#101512; --panel:#171e1a; --ink:#edf4ef; --muted:#a9b5ad; --line:#344039; --green:#4bc28e; --green-soft:#173b2c; --amber:#efad48; --amber-soft:#402f16; --red:#ef7770; --red-soft:#44201f; --blue:#8eb5f0; --queued:#89928c; --shadow:none; }} }}
* {{ box-sizing:border-box; }} body {{ margin:0; background:var(--bg); color:var(--ink); font-family:ui-sans-serif,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif; }}
a {{ color:var(--blue); }} code {{ font-family:ui-monospace,SFMono-Regular,Menlo,monospace; font-size:.88em; }}
.shell {{ max-width:1280px; margin:auto; padding:40px 24px 80px; }}
.hero {{ display:grid; grid-template-columns:minmax(0,1.5fr) minmax(280px,.7fr); gap:24px; align-items:end; padding-bottom:28px; border-bottom:1px solid var(--line); }}
.eyebrow {{ margin:0 0 8px; color:var(--green); font-weight:700; letter-spacing:.08em; text-transform:uppercase; font-size:.78rem; }}
h1 {{ margin:0; max-width:820px; font-size:clamp(2rem,5vw,4.8rem); line-height:.98; letter-spacing:-.055em; }}
.hero p {{ margin:18px 0 0; max-width:760px; color:var(--muted); font-size:1.05rem; line-height:1.55; }}
.hero-meta {{ border-left:3px solid var(--green); padding:14px 0 14px 18px; }} .hero-meta span,.hero-meta strong {{ display:block; }} .hero-meta span {{ color:var(--muted); font-size:.85rem; margin-top:5px; }}
.section {{ margin-top:44px; }} .section-head {{ display:flex; justify-content:space-between; gap:20px; align-items:end; margin-bottom:18px; }}
h2 {{ margin:0; font-size:1.45rem; letter-spacing:-.025em; }} .section-head p {{ margin:0; color:var(--muted); max-width:680px; line-height:1.45; }}
.overview {{ display:grid; grid-template-columns:repeat(auto-fit,minmax(190px,1fr)); gap:14px; }}
.metric {{ background:var(--panel); border:1px solid var(--line); padding:18px; box-shadow:var(--shadow); }} .metric strong {{ font-size:2rem; display:block; }} .metric span {{ color:var(--muted); }}
.source-grid {{ display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:18px; margin-top:18px; }}
.source-card {{ background:var(--panel); border:1px solid var(--line); padding:20px; box-shadow:var(--shadow); }} .source-card-head {{ display:flex; justify-content:space-between; gap:12px; align-items:start; }} .source-card h3 {{ margin:0 0 5px; }}
.field-grid {{ display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:14px; }} .field-card {{ min-width:0; background:var(--panel); border:1px solid var(--line); padding:18px; box-shadow:var(--shadow); }} .field-card h3 {{ margin:0 0 5px; overflow-wrap:anywhere; }} .field-review {{ color:var(--red); margin:0; overflow-wrap:anywhere; }} .field-review code {{ word-break:break-word; }} .field-ready {{ color:var(--green); margin:0; }} .source-card-head>* {{ min-width:0; }}
.stacked-bar {{ display:flex; width:100%; height:18px; overflow:hidden; margin:22px 0 13px; background:var(--line); }} .bar-part {{ display:block; }} .bar-ported {{ background:var(--green); }} .bar-partial {{ background:var(--amber); }} .bar-missing {{ background:var(--red); }}
.bar-legend {{ display:flex; flex-wrap:wrap; gap:8px 16px; color:var(--muted); font-size:.84rem; }} .dot {{ display:inline-block; width:9px; height:9px; margin-right:6px; }} .dot-ported {{ background:var(--green); }} .dot-partial {{ background:var(--amber); }} .dot-missing {{ background:var(--red); }}
.phases {{ list-style:none; margin:0; padding:0; display:grid; grid-template-columns:repeat(auto-fit,minmax(145px,1fr)); gap:0; }} .phase {{ position:relative; padding:32px 16px 0 0; border-top:2px solid var(--line); }} .phase-marker {{ position:absolute; top:-7px; left:0; width:12px; height:12px; border-radius:50%; background:var(--queued); }} .phase-active {{ border-color:var(--green); }} .phase-active .phase-marker {{ background:var(--green); box-shadow:0 0 0 5px var(--green-soft); }} .phase strong {{ display:block; margin-bottom:8px; }} .phase p {{ color:var(--muted); font-size:.82rem; line-height:1.45; margin:9px 0 0; }}
.workflow-grid {{ display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:14px; }} .workflow-card {{ background:var(--panel); border:1px solid var(--line); border-top:4px solid var(--green); padding:18px; box-shadow:var(--shadow); }} .workflow-card h3 {{ margin:0 0 8px; }} .workflow-card p {{ margin:0; color:var(--muted); line-height:1.5; }} .workflow-flow {{ margin-top:14px; border-left:4px solid var(--green); padding:12px 16px; background:var(--panel); color:var(--muted); line-height:1.6; }}
.status {{ display:inline-block; padding:3px 7px; margin-left:6px; font-size:.7rem; font-weight:700; text-transform:uppercase; letter-spacing:.05em; border:1px solid currentColor; }} .status-ported,.status-verified,.status-exact,.status-green,.status-active,.status-frozen,.status-fixed {{ color:var(--green); background:var(--green-soft); }} .status-partial,.status-in-progress,.status-amber,.status-translated,.status-reviewed,.status-compiled {{ color:var(--amber); background:var(--amber-soft); }} .status-missing,.status-pending,.status-ready,.status-red {{ color:var(--red); background:var(--red-soft); }} .status-queued {{ color:var(--queued); }}
.convention-shape {{ color:var(--muted); font-size:.84rem; }} .field-card>p:last-child {{ line-height:1.45; }}
.owner-list {{ display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:10px; }} .owner {{ min-width:0; background:var(--panel); border:1px solid var(--line); }} .owner summary {{ min-width:0; cursor:pointer; display:flex; justify-content:space-between; gap:12px; align-items:center; padding:14px 16px; font-family:ui-monospace,SFMono-Regular,Menlo,monospace; }} .owner summary>span:first-child {{ min-width:0; overflow-wrap:anywhere; word-break:break-word; }} .owner p,.owner-meta {{ min-width:0; margin:0; padding:0 16px 15px; color:var(--muted); line-height:1.45; overflow-wrap:anywhere; }} .owner-meta {{ display:grid; gap:5px; font-size:.82rem; }}
.filters {{ display:flex; flex-wrap:wrap; gap:8px; margin-bottom:12px; }} .filters button {{ appearance:none; background:transparent; color:var(--ink); border:1px solid var(--line); padding:7px 10px; cursor:pointer; }} .filters button[aria-pressed="true"] {{ border-color:var(--green); color:var(--green); background:var(--green-soft); }}
.table-wrap {{ overflow-x:auto; border:1px solid var(--line); background:var(--panel); }} table {{ width:100%; border-collapse:collapse; min-width:920px; }} th,td {{ padding:11px 13px; text-align:left; border-bottom:1px solid var(--line); vertical-align:top; }} th {{ color:var(--muted); font-size:.75rem; text-transform:uppercase; letter-spacing:.05em; }} td small {{ color:var(--muted); }} td:nth-child(1) {{ width:220px; }} td:nth-child(3) {{ width:120px; }} td:nth-child(5) {{ max-width:340px; overflow-wrap:anywhere; }} tr:last-child td {{ border-bottom:0; }}
	.reports {{ display:grid; grid-template-columns:repeat(3,minmax(0,1fr)); gap:14px; }} .report {{ background:var(--panel); border:1px solid var(--line); border-top:4px solid var(--line); padding:18px; box-shadow:var(--shadow); }} .report-green {{ border-top-color:var(--green); }} .report-amber {{ border-top-color:var(--amber); }} .report-red {{ border-top-color:var(--red); }} .report-head,.report footer {{ display:flex; justify-content:space-between; gap:10px; align-items:center; }} .report time {{ color:var(--muted); }} .report h3 {{ margin:15px 0 8px; }} .report p {{ color:var(--muted); line-height:1.5; }} .report footer {{ margin-top:16px; }} .runs {{ display:grid; gap:8px; margin-top:14px; }} .run {{ border-left:2px solid var(--line); padding-left:10px; }} .run code,.run span {{ display:block; overflow-wrap:anywhere; }} .run span {{ color:var(--muted); margin-top:3px; font-size:.82rem; }}
	.suites {{ display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:14px; }} .suite {{ background:var(--panel); border:1px solid var(--line); border-left:4px solid var(--queued); padding:18px; box-shadow:var(--shadow); }} .suite-green {{ border-left-color:var(--green); }} .suite-active {{ border-left-color:var(--green); }} .suite-amber {{ border-left-color:var(--amber); }} .suite-head {{ display:flex; align-items:center; justify-content:space-between; gap:12px; }} .suite h3 {{ margin:12px 0 7px; }} .suite>p {{ color:var(--muted); line-height:1.5; }} .suite dl {{ display:grid; grid-template-columns:90px 1fr; gap:8px 12px; margin:16px 0 0; font-size:.84rem; }} .suite dt {{ color:var(--muted); font-weight:700; }} .suite dd {{ margin:0; overflow-wrap:anywhere; }}
.unit-grid {{ display:grid; grid-template-columns:repeat(2,minmax(0,1fr)); gap:14px; }} .unit-card {{ background:var(--panel); border:1px solid var(--line); border-left:4px solid var(--red); padding:18px; box-shadow:var(--shadow); }} .unit-in-progress {{ border-left-color:var(--amber); }} .unit-green {{ border-left-color:var(--green); }} .unit-card>p {{ color:var(--muted); }} .unit-card dl {{ display:grid; grid-template-columns:110px 1fr; gap:7px 12px; margin:14px 0 0; font-size:.82rem; }} .unit-card dt {{ color:var(--muted); font-weight:700; }} .unit-card dd {{ margin:0; overflow-wrap:anywhere; }}
.gallery {{ display:grid; grid-template-columns:repeat(4,minmax(0,1fr)); gap:14px; }} .gallery-card {{ margin:0; background:var(--panel); border:1px solid var(--line); overflow:hidden; }} .gallery-card>a {{ display:grid; place-items:center; aspect-ratio:1/1; background:repeating-conic-gradient(from 45deg,var(--line) 0 25%,transparent 0 50%) 50%/18px 18px; }} .gallery-card img {{ display:block; width:100%; height:100%; object-fit:contain; image-rendering:auto; }} .gallery-card figcaption {{ padding:12px; display:grid; gap:8px; }} .gallery-card figcaption>span {{ display:flex; flex-wrap:wrap; gap:6px; align-items:center; }} .gallery-card .status {{ margin-left:0; }} .gallery-card figcaption>a {{ font-size:.82rem; }}
.contract {{ border-left:4px solid var(--green); padding:5px 0 5px 18px; color:var(--muted); line-height:1.55; }}
@media (max-width:900px) {{ .hero {{ grid-template-columns:1fr; }} .hero-meta {{ border-left:0; border-top:3px solid var(--green); padding-left:0; }} .overview,.reports,.suites,.workflow-grid,.unit-grid {{ grid-template-columns:1fr; }} .source-grid,.owner-list,.field-grid {{ grid-template-columns:1fr; }} .phases {{ grid-template-columns:repeat(2,minmax(0,1fr)); gap:24px 0; }} .gallery {{ grid-template-columns:repeat(2,minmax(0,1fr)); }} }}
@media (max-width:520px) {{ .shell {{ padding:28px 15px 60px; }} .section-head {{ display:block; }} .section-head p {{ margin-top:8px; }} .phases,.gallery {{ grid-template-columns:1fr; }} .source-card-head {{ display:block; }} .source-card-head strong {{ display:block; margin-top:10px; }} }}
</style>
</head>
<body>
<main class="shell">
  <header class="hero">
    <div><p class="eyebrow">Pinned source · mechanical translation</p><h1>Whole Metal renderer progress</h1><p>Red, amber, and green reflect source and ownership closure. Renderer images are verification evidence after translation; they never choose the next implementation task.</p></div>
    <div class="hero-meta"><strong>Active: {escape(next(phase['label'] for phase in progress['phase'] if phase['id'] == progress['active_phase']))}</strong><span>Updated {escape(progress['updated'])}</span><span>Upstream {escape(progress['upstream_ref'][:12])}</span></div>
  </header>

  <section class="section">
    <div class="section-head"><h2>Source closure</h2><p>Line-weighted status from the exhaustive header and implementation map.</p></div>
    <div class="overview">
      <div class="metric"><strong>{translated_campaign_files}/{total_campaign_files}</strong><span>individual pinned files mechanically translated</span></div>
      <div class="metric"><strong>{claimed_campaign_files}/{total_campaign_files}</strong><span>individual pinned files claimed in parallel waves</span></div>
      <div class="metric"><strong>{ported_lines / total_lines * 100:.1f}%</strong><span>{ported_lines:,} of {total_lines:,} primary-context lines ported</span></div>
      <div class="metric"><strong>{partial_lines:,}</strong><span>partial source lines still under translation</span></div>
      <div class="metric"><strong>{missing_lines:,}</strong><span>missing source lines with no Rust owner</span></div>
      <div class="metric"><strong>{mapped_fields}/{len(field_rows)}</strong><span>field ownership mappings prepared</span></div>
      <div class="metric"><strong>{translated_fields}/{len(field_rows)}</strong><span>field owners translated</span></div>
      <div class="metric"><strong>{configuration_blocks}/{configuration_blocks}</strong><span>semantic preprocessor blocks inventoried</span></div>
      <div class="metric"><strong>{mapped_configurations}/{len(configuration_rows)}</strong><span>conditional branch mappings prepared</span></div>
      <div class="metric"><strong>{translated_configurations}/{len(configuration_rows)}</strong><span>conditional branch entries translated</span></div>
      <div class="metric"><strong>{mapped_dependencies}/{len(dependency_rows)}</strong><span>generic source mappings prepared</span></div>
      <div class="metric"><strong>{translated_dependencies}/{len(dependency_rows)}</strong><span>generic sources translated</span></div>
      <div class="metric"><strong>{mapped_includes}/{len(include_rows)}</strong><span>direct #include/#import occurrences mapped</span></div>
      <div class="metric"><strong>{len(source_dependency_rows)}</strong><span>normalized source-dependency edges</span></div>
      <div class="metric"><strong>{len(build_branch_rows)}</strong><span>build behavior branches mapped</span></div>
      <div class="metric"><strong>{frozen_conventions}/{len(convention_rows)}</strong><span>translation conventions frozen</span></div>
    </div>
    <div class="source-grid">{source_summary(rows)}</div>
  </section>

  <section class="section">
    <div class="section-head"><h2>Bun mechanical-port operating model</h2><p>Roles and queue order are machine-checked. Translation and review are separate contexts; validation cannot choose implementation scope.</p></div>
    <div class="workflow-grid">
      <article class="workflow-card"><h3>{escape(progress['workflow']['translator'])} · translator</h3><p>Mechanically translates complete pinned C++/Objective-C++ source owners in source order. No feature slices, self-review, stubs, cleanup, or architecture redesign.</p></article>
      <article class="workflow-card"><h3>{escape(progress['workflow']['orchestrator'])} · adversarial driver</h3><p>Orchestrates ownership, performs separate source-semantics and ownership/lifetime reviews, drives corrections, then owns compiler and validation queues.</p></article>
      <article class="workflow-card"><h3>Two independent review passes</h3><p>{progress['workflow']['source_reviews_required']} source review + {progress['workflow']['ownership_reviews_required']} ownership review for every translation. Reviewers assume the diff is wrong and do not inherit translator rationale.</p></article>
    </div>
    <div class="workflow-flow"><strong>Ordered queue:</strong> preparation → parallel Luna translation of {total_campaign_files} files → global Sol source review → global Sol ownership review → correction → compiler queue → rooted smoke → V0–V9 parity → post-green cleanup. The {len(translation_units)} groups are integration boundaries, not the file denominator. <strong>Feature slices allowed:</strong> {str(progress['workflow']['feature_slices_allowed']).lower()}.</div>
  </section>

  <section class="section"><div class="section-head"><h2>Campaign phases</h2><p>Translation completes before compiler diagnostics and behavior fixtures become work queues.</p></div><ol class="phases">{phase_rail(progress['phase'])}</ol></section>

  <section class="section">
    <div class="section-head"><h2>{total_campaign_files} files · {len(translation_units)} integration groups</h2><p>Files are the primary transliteration denominator. Groups only aggregate coupled file receipts and later compiler diagnostics; reviews run as separate global passes after all file targets exist.</p></div>
    <div class="unit-grid">{translation_unit_summary(manifest.get('translation_unit', []), repo_root)}</div>
  </section>

  <section class="section">
    <div class="section-head"><h2>State-bearing field closure</h2><p>The checker derives all {len(field_rows)} direct and inherited declarations from the pinned source owners. Any omitted, invented, configuration-drifted, or line-drifted ledger row fails the campaign gate.</p></div>
    <div class="field-grid">{field_summary(field_rows)}</div>
  </section>

  <section class="section">
    <div class="section-head"><h2>Conditional-configuration closure</h2><p>The checker extracts every preprocessor block and branch entry from all {total_campaign_files} pinned sources, including continued directives and semantic elif/else paths. Missing ORE, shader, platform, tools, canvas, simulator, or header modes stay red.</p></div>
    <div class="field-grid">{configuration_summary(configuration_rows)}</div>
  </section>

  <section class="section">
    <div class="section-head"><h2>Generic dependency translation queue</h2><p>Every complete generic renderer source has one source-shaped Rust target, one dispatch owner, and field-lifetime coverage. Prepared mappings stay red until the literal target is translated.</p></div>
    <div class="field-grid">{dependency_summary(dependency_rows)}</div>
  </section>

  <section class="section">
    <div class="section-head"><h2>Direct include/import correspondence</h2><p>The checker derives every direct #include and Objective-C #import occurrence across the complete campaign and resolves campaign, exact existing Rust, generated shader/artifact, and toolchain boundaries.</p></div>
    <div class="field-grid">{include_summary(include_rows)}</div>
  </section>

  <section class="section">
    <div class="section-head"><h2>Source, dispatch, and build authority</h2><p>Cycle-allowing source dependencies are exhaustive and independent from acyclic translation scheduling. Make and Python build behavior is also translation authority, including non-Metal rule dispositions.</p></div>
    <div class="field-grid">{authority_graph_summary(source_dependency_rows, dispatch_rows, build_branch_rows)}</div>
  </section>

  <section class="section">
    <div class="section-head"><h2>Frozen translation conventions</h2><p>These mappings govern the literal source pass. They prevent a convenient Rust rewrite from silently changing ownership, bytes, failure order, configuration behavior, or destruction.</p></div>
    <div class="field-grid">{convention_summary(convention_rows)}</div>
  </section>

  <section class="section">
    <div class="section-head"><h2>Validation exit gates</h2><p>Pinned C++ Metal proves source parity. Rust-WGPU is a separate product-behavior differential, and its current 4/736 coverage remains amber.</p></div>
    <div class="suites">{validation_suites(progress.get('suite', []))}</div>
  </section>

  <section class="section">
    <div class="section-head"><h2>Whole-owner state</h2><p>{len(metal_sources)} pinned Metal source/support files: {source_statuses['ported']} ported, {source_statuses['in-progress']} in progress. Renderer ownership contracts: {owner_statuses['ported']} ported, {owner_statuses['in-progress']} in progress, {owner_statuses['pending']} pending.</p></div>
    <div class="owner-list">{ownership_rows(ownership.get('owner', []))}</div>
  </section>

  <section class="section">
    <div class="section-head"><h2>Complete line map</h2><p>Every red or amber row blocks whole-file promotion, regardless of passing feature images.</p></div>
    <div class="filters" role="group" aria-label="Filter source status"><button type="button" data-filter="all" aria-pressed="true">All</button><button type="button" data-filter="ported" aria-pressed="false">Ported</button><button type="button" data-filter="partial" aria-pressed="false">Partial</button><button type="button" data-filter="missing" aria-pressed="false">Missing</button></div>
    <div class="table-wrap"><table><thead><tr><th>Source range</th><th>Responsibility</th><th>Status</th><th>Remaining work</th><th>Rust owner</th></tr></thead><tbody>{line_map_table(rows)}</tbody></table></div>
  </section>

  <section class="section"><div class="section-head"><h2>Progress reports and verification logs</h2><p>Checkpoint results are preserved without using them to define translation scope.</p></div><div class="reports">{report_cards(progress.get('report', []), repo_root / 'docs/metal-renderer-progress.html', repo_root)}</div></section>

  <section class="section"><div class="section-head"><h2>Rendered evidence</h2><p>Checked-in Metal outputs currently covered by the regression corpus. Green images demonstrate preserved behavior, not complete source support.</p></div><div class="gallery">{gallery(corpus_paths, repo_root)}</div></section>

  <section class="section"><div class="contract"><strong>Green means complete.</strong> The primary header and implementation remain in progress until all mapped ranges, state-bearing fields, platform configurations, and upstream-reachable branches are closed and the complete verification queue passes.</div></section>
</main>
<script>
const buttons = document.querySelectorAll('[data-filter]');
const rows = document.querySelectorAll('tbody tr[data-status]');
buttons.forEach((button) => button.addEventListener('click', () => {{
  const filter = button.dataset.filter;
  buttons.forEach((candidate) => candidate.setAttribute('aria-pressed', String(candidate === button)));
  rows.forEach((row) => {{ row.hidden = filter !== 'all' && row.dataset.status !== filter; }});
}}));
</script>
</body>
</html>
"""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    repo_root = args.repo_root.resolve()
    output = args.output if args.output.is_absolute() else repo_root / args.output
    output.parent.mkdir(parents=True, exist_ok=True)
    rendered = render(repo_root)
    normalized = "\n".join(line.rstrip() for line in rendered.splitlines()) + "\n"
    output.write_text(normalized, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
