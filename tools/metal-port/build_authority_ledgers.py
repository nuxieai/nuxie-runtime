#!/usr/bin/env python3
"""Build exhaustive source/configuration authority for the pinned Metal port.

This is intentionally separate from check.py while the preparation ledgers are
being reviewed.  It derives facts from the source manifest and pinned upstream
tree; prose-only rows are not accepted as coverage.
"""

from __future__ import annotations

import argparse
import ast
import csv
import io
import pathlib
import re
import subprocess
import sys
import tomllib
from collections import defaultdict
from dataclasses import dataclass, field
from typing import Iterable


PIN = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"
DISPATCH_ORDER = (
    "ore-types",
    "ore-rstb-container",
    "ore-binding-map",
    "generic-rive-types",
    "generic-refcnt",
    "gpu-resource",
    "ore-bind-group-layout",
    "ore-buffer",
    "ore-texture",
    "ore-sampler",
    "ore-shader-module",
    "ore-pipeline",
    "ore-bind-group",
    "ore-context-render-pass",
    "generic-lite-rtti",
    "generic-image-sampler",
    "generic-renderer-contract",
    "generic-renderer-implementation",
    "generic-gpu-texture-format",
    "generic-astc-footprints",
    "generic-gpu-contract",
    "generic-buffer-ring",
    "generic-render-target",
    "generic-texture-image",
    "generic-rive-render-buffer",
    "generic-render-canvas",
    "generic-render-context-contract",
    "generic-render-context-impl-contract",
    "generic-render-context-helper",
    "generic-gpu-implementation",
    "generic-render-context-implementation",
    "metal-shader-source-batch",
    "metal-render-context-api",
    "metal-background-shader-compiler",
    "metal-render-context-implementation",
    "generic-rive-render-path",
    "generic-rive-render-paint",
    "generic-rive-renderer",
    "generic-factory-contract",
    "generic-gradient",
    "generic-rive-render-factory",
)
EXPECTED_SOURCE_COUNT = 111
EXPECTED_BLOCK_COUNT = 634
EXPECTED_BRANCH_COUNT = 845
EXPECTED_GUARD_COUNT = 6
EXPECTED_INCLUDE_COUNT = 366
EXPECTED_INCLUDE_TOKEN_COUNT = 142
EXPECTED_INCLUDE_FILE_COUNT = 74
EXPECTED_NORMALIZED_DEPENDENCY_COUNT = 359
EXPECTED_MISSING_UNIT_EDGE_COUNT = 29
EXPECTED_MISSING_UNIT_OCCURRENCE_COUNT = 36
EXPECTED_REAL_UNIT_SCCS = 1
EXPECTED_REAL_UNIT_SCC_MEMBERS = 10

PREPROCESSOR_PATH = pathlib.Path("docs/metal-port-preprocessor-authority.tsv")
INCLUDE_PATH = pathlib.Path("docs/metal-port-include-authority.tsv")
SOURCE_DEPENDENCY_PATH = pathlib.Path("docs/metal-port-source-dependencies.tsv")
DISPATCH_PATH = pathlib.Path("docs/metal-port-dispatch-prerequisites.tsv")
BUILD_BRANCH_PATH = pathlib.Path("docs/metal-port-build-branch-authority.tsv")

OPENING = re.compile(r"^\s*#\s*(if|ifdef|ifndef)\s+(.+?)\s*$")
BRANCH = re.compile(r"^\s*#\s*(elif|else)(?:\s+(.+?))?\s*$")
CLOSING = re.compile(r"^\s*#\s*endif(?:\s|$)")
DIRECT_INCLUDE = re.compile(
    r'^\s*#\s*(include|import)\s*([<"])([^>"]+)[>"]'
)
SEARCH_ROOTS = (
    pathlib.PurePosixPath("."),
    pathlib.PurePosixPath("include"),
    pathlib.PurePosixPath("renderer/include"),
    pathlib.PurePosixPath("decoders/include"),
    pathlib.PurePosixPath("renderer/src"),
    pathlib.PurePosixPath("renderer/src/shaders"),
)


@dataclass
class BranchEntry:
    line: int
    directive: str
    path: str


@dataclass
class Block:
    source: str
    start: int
    end: int
    depth: int
    opening: str
    branches: list[BranchEntry] = field(default_factory=list)
    prior_conditions: list[str] = field(default_factory=list)
    active_condition: str = ""


@dataclass(frozen=True)
class IncludeOccurrence:
    source: str
    line: int
    directive: str
    token: str
    syntax: str
    branch_path: str
    resolution_kind: str
    dependency_source: str


def clean_directive(line: str) -> str:
    return " ".join(line.strip().split())


def logical_directive(lines: list[str], index: int) -> str:
    """Join a continued preprocessor directive without changing its start line."""

    parts = [lines[index].rstrip()]
    while parts[-1].endswith("\\"):
        parts[-1] = parts[-1][:-1]
        index += 1
        if index >= len(lines):
            raise ValueError("preprocessor directive ends with a dangling continuation")
        parts.append(lines[index].rstrip())
    return " ".join(part.strip() for part in parts)


def opening_condition(kind: str, expression: str) -> str:
    expression = clean_directive(expression)
    if kind == "ifdef":
        return f"defined({expression})"
    if kind == "ifndef":
        return f"!defined({expression})"
    return expression


def condition_path(stack: list[Block]) -> str:
    return " && ".join(f"({block.active_condition})" for block in stack) or "all"


def source_scope(manifest: dict, upstream: pathlib.Path) -> list[str]:
    excluded = set(manifest["source_excludes"])
    sources = sorted(
        {
            path.relative_to(upstream).as_posix()
            for pattern in manifest["source_globs"]
            for path in upstream.glob(pattern)
            if path.is_file()
            and path.relative_to(upstream).as_posix() not in excluded
        }
    )
    assert_exact("source count", len(sources), EXPECTED_SOURCE_COUNT)
    return sources


def source_owners(manifest: dict) -> tuple[dict[str, str], dict[str, dict]]:
    units = {str(unit["id"]): unit for unit in manifest["translation_unit"]}
    owners: dict[str, str] = {}
    for unit_id, unit in units.items():
        for source in unit["sources"]:
            if source in owners:
                raise ValueError(f"source is owned twice: {source}")
            owners[str(source)] = unit_id
    return owners, units


def parse_source_structure(
    source: str, text: str
) -> tuple[list[Block], list[tuple[int, str, str, str, str]]]:
    """Return blocks and include sites with their exact active directive path."""

    lines = text.splitlines()
    blocks: list[Block] = []
    includes: list[tuple[int, str, str, str, str]] = []
    stack: list[Block] = []
    conditionals_enabled = pathlib.PurePosixPath(source).suffix in {
        ".h", ".hpp", ".cpp", ".mm", ".glsl", ".vert", ".frag", ".metal",
    }
    for line_number, line in enumerate(lines, 1):
        directive_line = (
            logical_directive(lines, line_number - 1)
            if conditionals_enabled
            and re.match(r"^\s*#\s*(?:if|ifdef|ifndef|elif|else|endif)(?:\s|$)", line)
            else line
        )
        opening = OPENING.match(directive_line) if conditionals_enabled else None
        branch = BRANCH.match(directive_line) if conditionals_enabled else None
        include = DIRECT_INCLUDE.match(line)
        if opening:
            directive = clean_directive(directive_line)
            semantic = opening_condition(opening.group(1), opening.group(2))
            block = Block(
                source=source,
                start=line_number,
                end=0,
                depth=len(stack),
                opening=directive,
                prior_conditions=[semantic],
                active_condition=semantic,
            )
            stack.append(block)
            block.branches.append(
                BranchEntry(line_number, directive, condition_path(stack))
            )
        elif branch:
            if not stack:
                raise ValueError(f"orphan preprocessor branch: {source}:{line_number}")
            directive = clean_directive(directive_line)
            block = stack[-1]
            prior = " || ".join(f"({condition})" for condition in block.prior_conditions)
            if branch.group(1) == "elif":
                semantic = clean_directive(branch.group(2) or "")
                block.active_condition = f"!({prior}) && ({semantic})"
                block.prior_conditions.append(semantic)
            else:
                block.active_condition = f"!({prior})"
            stack[-1].branches.append(
                BranchEntry(line_number, directive, condition_path(stack))
            )
        elif conditionals_enabled and CLOSING.match(directive_line):
            if not stack:
                raise ValueError(f"orphan #endif: {source}:{line_number}")
            block = stack.pop()
            block.end = line_number
            blocks.append(block)
        if include:
            includes.append(
                (
                    line_number,
                    include.group(1),
                    include.group(3),
                    "angle" if include.group(2) == "<" else "quote",
                    condition_path(stack),
                )
            )
    if stack:
        raise ValueError(
            f"unterminated preprocessor block(s): {source}:"
            + ",".join(str(block.start) for block in stack)
        )
    return blocks, includes


def is_canonical_guard(block: Block, lines: list[str]) -> bool:
    match = re.fullmatch(r"#\s*ifndef\s+([A-Za-z_][A-Za-z0-9_]*)", block.opening)
    return bool(
        block.start <= 10
        and block.end == len(lines)
        and match is not None
        and block.start < len(lines)
        and re.fullmatch(
            rf"#\s*define\s+{re.escape(match.group(1))}",
            lines[block.start].strip(),
        )
    )


def resolve_include(
    upstream: pathlib.Path,
    campaign_sources: set[str],
    source: str,
    token: str,
    syntax: str,
) -> tuple[str, str]:
    candidates: list[pathlib.PurePosixPath] = []
    if syntax == "quote":
        candidates.append(pathlib.PurePosixPath(source).parent / token)
    candidates.extend(root / token for root in SEARCH_ROOTS)
    for candidate in candidates:
        normalized = pathlib.PurePosixPath(candidate).as_posix()
        if (upstream / normalized).is_file():
            return (
                "campaign-source" if normalized in campaign_sources else "upstream-global-source",
                normalized,
            )

    if token.startswith("generated/shaders/"):
        basename = token.removeprefix("generated/shaders/")
        for suffix in (".exports.h", ".hpp"):
            if basename.endswith(suffix):
                origin = "renderer/src/shaders/" + basename[: -len(suffix)]
                if origin in campaign_sources:
                    return "generated-shader-source", origin
        if basename.endswith(".metallib.c"):
            return "generated-shader-artifact", "renderer/src/shaders/Makefile"

    if token.endswith(".minified.glsl"):
        origin = (
            "renderer/src/shaders/"
            + token[: -len(".minified.glsl")]
            + ".glsl"
        )
        if origin in campaign_sources:
            return "generated-shader-source", origin

    if token == "draw_combinations.metal":
        return (
            "generated-shader-source",
            "renderer/src/shaders/metal/generate_draw_combinations.py",
        )

    if syntax == "angle":
        return "toolchain-header", f"toolchain:{token}"
    raise ValueError(f"unresolved quoted include: {source}: {token}")


def collect_authority(
    manifest: dict, upstream: pathlib.Path, sources: list[str]
) -> tuple[list[Block], list[Block], list[IncludeOccurrence]]:
    semantic: list[Block] = []
    guards: list[Block] = []
    occurrences: list[IncludeOccurrence] = []
    source_set = set(sources)
    for source in sources:
        text = (upstream / source).read_text(encoding="utf-8")
        lines = text.splitlines()
        blocks, includes = parse_source_structure(source, text)
        for block in blocks:
            (guards if is_canonical_guard(block, lines) else semantic).append(block)
        for line, directive, token, syntax, branch_path in includes:
            kind, dependency = resolve_include(
                upstream, source_set, source, token, syntax
            )
            occurrences.append(
                IncludeOccurrence(
                    source,
                    line,
                    directive,
                    token,
                    syntax,
                    branch_path,
                    kind,
                    dependency,
                )
            )
    semantic.sort(key=lambda block: (block.source, block.start, block.end))
    guards.sort(key=lambda block: (block.source, block.start, block.end))
    occurrences.sort(key=lambda row: (row.source, row.line, row.token))
    assert_exact("semantic preprocessor block count", len(semantic), EXPECTED_BLOCK_COUNT)
    assert_exact(
        "semantic preprocessor branch count",
        sum(len(block.branches) for block in semantic),
        EXPECTED_BRANCH_COUNT,
    )
    assert_exact("canonical guard count", len(guards), EXPECTED_GUARD_COUNT)
    assert_exact("direct include count", len(occurrences), EXPECTED_INCLUDE_COUNT)
    assert_exact(
        "distinct direct include token count",
        len({row.token for row in occurrences}),
        EXPECTED_INCLUDE_TOKEN_COUNT,
    )
    assert_exact(
        "files with direct includes",
        len({row.source for row in occurrences}),
        EXPECTED_INCLUDE_FILE_COUNT,
    )
    ore_files = {block.source for block in semantic if "/ore/" in block.source}
    shader_files = {
        block.source
        for block in semantic
        if pathlib.PurePosixPath(block.source).suffix in {".glsl", ".vert", ".frag"}
    }
    assert_exact("ORE files with semantic branches", len(ore_files), 5)
    assert_exact("shader inputs with semantic branches", len(shader_files), 33)
    return semantic, guards, occurrences


def existing_rust_correspondences(
    repo: pathlib.Path, occurrences: list[IncludeOccurrence]
) -> dict[str, tuple[str, str]]:
    global_sources = {
        row.dependency_source
        for row in occurrences
        if row.resolution_kind == "upstream-global-source"
    }
    include_map = repo / "docs/render-context-metal-includes.tsv"
    with include_map.open(encoding="utf-8", newline="") as source:
        rows = list(csv.DictReader(source, delimiter="\t"))
    candidates: dict[str, set[str]] = defaultdict(set)
    for row in rows:
        dependency = str(row.get("source_resolution", ""))
        owner = str(row.get("correspondence_owner", ""))
        if dependency in global_sources and owner.startswith("rust:"):
            candidates[dependency].add(owner.removeprefix("rust:"))
    missing = sorted(global_sources - set(candidates))
    ambiguous = {
        dependency: sorted(owners)
        for dependency, owners in candidates.items()
        if len(owners) != 1
    }
    if missing or ambiguous:
        raise ValueError(
            f"upstream-global Rust correspondence incomplete: missing={missing}, "
            f"ambiguous={ambiguous}"
        )
    correspondences: dict[str, tuple[str, str]] = {}
    for dependency in sorted(global_sources):
        owner = next(iter(candidates[dependency]))
        owner_path = pathlib.PurePosixPath(owner)
        if owner_path.is_absolute() or ".." in owner_path.parts or owner_path.as_posix() != owner:
            raise ValueError(
                f"upstream-global Rust correspondence is not canonical: {dependency}: {owner}"
            )
        absolute_owner = repo / owner
        if not absolute_owner.is_file():
            raise ValueError(
                f"upstream-global Rust correspondence does not exist: {dependency}: {owner}"
            )
        tracked = subprocess.run(
            ["git", "ls-files", "--error-unmatch", "--", owner],
            cwd=repo,
            check=False,
            capture_output=True,
            text=True,
        )
        if tracked.returncode != 0:
            raise ValueError(
                f"upstream-global Rust correspondence is not tracked: {dependency}: {owner}"
            )
        line_count = len(absolute_owner.read_text(encoding="utf-8").splitlines())
        correspondences[dependency] = (owner, f"rust:{owner}:1-{line_count}")
    return correspondences


def source_target(unit: dict, source: str) -> str:
    sources = [str(value) for value in unit["sources"]]
    targets = [str(value) for value in unit.get("rust_targets", [])]
    if len(sources) == len(targets):
        return targets[sources.index(source)]
    if targets:
        return ";".join(targets)
    return ";".join(str(value) for value in unit.get("artifact_targets", []))


def authority_translation_status(unit: dict) -> str:
    """Derive mutable coverage state from the receipt-gated owning unit."""

    status = str(unit.get("status", "pending"))
    if status in {"translated", "reviewed", "fixed", "compiled"}:
        return "translated"
    if status == "verified":
        return "verified"
    return "pending"


def tsv(columns: tuple[str, ...], rows: Iterable[dict[str, object]]) -> str:
    output = io.StringIO(newline="")
    writer = csv.DictWriter(
        output, fieldnames=columns, delimiter="\t", lineterminator="\n"
    )
    writer.writeheader()
    for row in rows:
        writer.writerow({column: row.get(column, "") for column in columns})
    return output.getvalue()


def render_preprocessor(
    blocks: list[Block], owners: dict[str, str], units: dict[str, dict]
) -> str:
    columns = (
        "version", "upstream_sha", "upstream_file", "block_id", "block_start",
        "block_end", "block_depth", "branch_ordinal", "branch_line", "directive",
        "active_branch_path", "translation_unit", "translation_target",
        "mapping_status", "translation_status", "translation_disposition",
        "translation_behavior", "validation_disposition", "evidence",
    )
    rows = []
    for block_index, block in enumerate(blocks, 1):
        unit_id = owners[block.source]
        for branch_index, branch in enumerate(block.branches, 1):
            rows.append(
                {
                    "version": 1,
                    "upstream_sha": PIN,
                    "upstream_file": block.source,
                    "block_id": f"pp-{block_index:04d}",
                    "block_start": block.start,
                    "block_end": block.end,
                    "block_depth": block.depth,
                    "branch_ordinal": branch_index,
                    "branch_line": branch.line,
                    "directive": branch.directive,
                    "active_branch_path": branch.path,
                    "translation_unit": unit_id,
                    "translation_target": source_target(units[unit_id], block.source),
                    "mapping_status": "prepared",
                    "translation_status": authority_translation_status(units[unit_id]),
                    "translation_disposition": "required",
                    "translation_behavior": (
                        "preserve-directive-and-every-branch-in-artifact-source"
                        if pathlib.PurePosixPath(block.source).suffix
                        in {".glsl", ".vert", ".frag"}
                        else "mechanically-translate-every-branch-in-source-order"
                    ),
                    "validation_disposition": (
                        "artifact-generation-and-compile"
                        if pathlib.PurePosixPath(block.source).suffix
                        in {".glsl", ".vert", ".frag"}
                        else "compile-and-execute"
                    ),
                    "evidence": f"cpp:{block.source}:{branch.line}",
                }
            )
    return tsv(columns, rows)


def render_includes(
    occurrences: list[IncludeOccurrence], owners: dict[str, str], units: dict[str, dict],
    correspondences: dict[str, tuple[str, str]],
) -> str:
    columns = (
        "version", "upstream_sha", "upstream_file", "include_line", "directive",
        "include_token",
        "include_syntax", "active_branch_path", "resolution_kind", "resolved_source",
        "source_unit", "dependency_unit", "correspondence_owner",
        "correspondence_evidence", "mapping_status", "translation_status",
        "translation_disposition", "evidence",
    )
    rows = []
    for occurrence in occurrences:
        correspondence = correspondences.get(occurrence.dependency_source)
        dependency_unit = owners.get(occurrence.dependency_source, "-")
        if occurrence.resolution_kind == "toolchain-header":
            disposition = "toolchain-provided"
            correspondence_owner = occurrence.dependency_source
            correspondence_evidence = occurrence.dependency_source
        elif correspondence is not None:
            disposition = "reuse-exact-existing-rust-correspondence"
            correspondence_owner, correspondence_evidence = correspondence
            dependency_unit = f"existing-rust:{correspondence_owner}"
        else:
            disposition = "required-source-edge"
            correspondence_owner = "-"
            correspondence_evidence = "-"
        rows.append(
            {
                "version": 1,
                "upstream_sha": PIN,
                "upstream_file": occurrence.source,
                "include_line": occurrence.line,
                "directive": occurrence.directive,
                "include_token": occurrence.token,
                "include_syntax": occurrence.syntax,
                "active_branch_path": occurrence.branch_path,
                "resolution_kind": occurrence.resolution_kind,
                "resolved_source": occurrence.dependency_source,
                "source_unit": owners[occurrence.source],
                "dependency_unit": dependency_unit,
                "correspondence_owner": correspondence_owner,
                "correspondence_evidence": correspondence_evidence,
                "mapping_status": (
                    "existing-complete" if correspondence is not None else "prepared"
                ),
                "translation_status": authority_translation_status(
                    units[owners[occurrence.source]]
                ),
                "translation_disposition": disposition,
                "evidence": f"cpp:{occurrence.source}:{occurrence.line}",
            }
        )
    return tsv(columns, rows)


def strongly_connected_components(graph: dict[str, set[str]]) -> list[list[str]]:
    index = 0
    indices: dict[str, int] = {}
    lowlinks: dict[str, int] = {}
    stack: list[str] = []
    on_stack: set[str] = set()
    components: list[list[str]] = []

    def visit(node: str) -> None:
        nonlocal index
        indices[node] = lowlinks[node] = index
        index += 1
        stack.append(node)
        on_stack.add(node)
        for dependency in sorted(graph.get(node, set())):
            if dependency not in graph:
                continue
            if dependency not in indices:
                visit(dependency)
                lowlinks[node] = min(lowlinks[node], lowlinks[dependency])
            elif dependency in on_stack:
                lowlinks[node] = min(lowlinks[node], indices[dependency])
        if lowlinks[node] == indices[node]:
            component: list[str] = []
            while True:
                member = stack.pop()
                on_stack.remove(member)
                component.append(member)
                if member == node:
                    break
            components.append(sorted(component))

    for node in sorted(graph):
        if node not in indices:
            visit(node)
    return sorted(components, key=lambda component: component[0])


def dependency_analysis(
    occurrences: list[IncludeOccurrence], owners: dict[str, str], units: dict[str, dict],
    correspondences: dict[str, tuple[str, str]],
) -> tuple[str, dict[str, set[str]], dict[str, str]]:
    graph = {unit_id: set() for unit_id in units}
    edge_occurrences: dict[tuple[str, str], list[IncludeOccurrence]] = defaultdict(list)
    for occurrence in occurrences:
        dependency_unit = owners.get(occurrence.dependency_source)
        source_unit = owners[occurrence.source]
        if dependency_unit and dependency_unit != source_unit:
            graph[source_unit].add(dependency_unit)
        edge_occurrences[(occurrence.source, occurrence.dependency_source)].append(
            occurrence
        )

    for unit_id, unit in units.items():
        if "dependencies" in unit:
            raise ValueError(
                f"translation unit {unit_id} uses ambiguous dependencies; "
                "declare source_dependencies and dispatch_prerequisites separately"
            )
        declared_sources = [str(value) for value in unit.get("source_dependencies", [])]
        if declared_sources != sorted(graph[unit_id]):
            raise ValueError(
                f"translation unit {unit_id} source_dependencies drifted: "
                f"expected {sorted(graph[unit_id])}, got {declared_sources}"
            )
        if "dispatch_prerequisites" not in unit:
            raise ValueError(
                f"translation unit {unit_id} is missing dispatch_prerequisites"
            )

    real_components = [component for component in strongly_connected_components(graph) if len(component) > 1]
    assert_exact("real unit SCC count", len(real_components), EXPECTED_REAL_UNIT_SCCS)
    assert_exact(
        "real unit SCC member count",
        sum(len(component) for component in real_components),
        EXPECTED_REAL_UNIT_SCC_MEMBERS,
    )
    component_ids = {
        member: f"unit-scc-{index:02d}"
        for index, component in enumerate(real_components, 1)
        for member in component
    }

    declared = {
        unit_id: set(
            str(value)
            for value in unit.get(
                "dispatch_prerequisites", unit.get("dependencies", [])
            )
        )
        for unit_id, unit in units.items()
    }
    missing_unit_occurrences = [
        occurrence
        for occurrence in occurrences
        if owners.get(occurrence.dependency_source)
        and owners[occurrence.dependency_source] != owners[occurrence.source]
        and owners[occurrence.dependency_source] not in declared[owners[occurrence.source]]
    ]
    missing_unit_edges = {
        (owners[occurrence.source], owners[occurrence.dependency_source])
        for occurrence in missing_unit_occurrences
    }
    assert_exact(
        "missing unit source-edge count",
        len(missing_unit_edges),
        EXPECTED_MISSING_UNIT_EDGE_COUNT,
    )
    assert_exact(
        "missing unit source-edge occurrence count",
        len(missing_unit_occurrences),
        EXPECTED_MISSING_UNIT_OCCURRENCE_COUNT,
    )

    columns = (
        "version", "upstream_sha", "upstream_file", "dependency_source",
        "resolution_kind", "occurrence_count", "occurrence_lines", "include_tokens",
        "source_unit", "dependency_unit", "correspondence_owner",
        "correspondence_evidence", "unit_edge_status", "unit_scc",
        "translation_status",
        "translation_disposition", "evidence",
    )
    rows = []
    for (source, dependency), edge_rows in sorted(edge_occurrences.items()):
        source_unit = owners[source]
        correspondence = correspondences.get(dependency)
        dependency_unit = owners.get(dependency, "external")
        if correspondence is not None:
            dependency_unit = f"existing-rust:{correspondence[0]}"
            edge_status = "existing-rust-correspondence"
        elif dependency_unit == "external":
            edge_status = (
                "toolchain-boundary"
                if dependency.startswith("toolchain:")
                else "global-source-boundary"
            )
        elif dependency_unit == source_unit:
            edge_status = "same-unit-source-dependency"
        elif dependency_unit in declared[source_unit]:
            edge_status = "also-dispatch-prerequisite"
        else:
            edge_status = "source-only-dependency"
        scc = (
            component_ids.get(source_unit, "-")
            if component_ids.get(source_unit) == component_ids.get(dependency_unit)
            else "-"
        )
        rows.append(
            {
                "version": 1,
                "upstream_sha": PIN,
                "upstream_file": source,
                "dependency_source": dependency,
                "resolution_kind": ";".join(sorted({row.resolution_kind for row in edge_rows})),
                "occurrence_count": len(edge_rows),
                "occurrence_lines": ",".join(str(row.line) for row in edge_rows),
                "include_tokens": ";".join(sorted({row.token for row in edge_rows})),
                "source_unit": source_unit,
                "dependency_unit": dependency_unit,
                "correspondence_owner": correspondence[0] if correspondence else "-",
                "correspondence_evidence": correspondence[1] if correspondence else "-",
                "unit_edge_status": edge_status,
                "unit_scc": scc,
                "translation_status": authority_translation_status(units[source_unit]),
                "translation_disposition": (
                    "provided-by-toolchain"
                    if edge_status == "toolchain-boundary"
                    else (
                        "reuse-exact-existing-rust-correspondence"
                        if edge_status == "existing-rust-correspondence"
                        else "preserve-source-dependency"
                    )
                ),
                "evidence": ";".join(
                    f"cpp:{row.source}:{row.line}" for row in edge_rows
                ),
            }
        )
    assert_exact("normalized source-dependency count", len(rows), EXPECTED_NORMALIZED_DEPENDENCY_COUNT)
    return tsv(columns, rows), graph, component_ids


def render_dispatch(
    units: dict[str, dict], source_graph: dict[str, set[str]], component_ids: dict[str, str]
) -> str:
    columns = (
        "version", "upstream_sha", "translation_unit", "dispatch_ordinal",
        "dispatch_prerequisites", "source_dependencies", "source_dependency_scc",
        "ordering_contract", "evidence",
    )
    rows = []
    dispatch_graph: dict[str, set[str]] = {}
    for unit_id, unit in units.items():
        prerequisites = {
            str(value)
            for value in unit.get(
                "dispatch_prerequisites", unit.get("dependencies", [])
            )
        }
        dispatch_graph[unit_id] = prerequisites
        rows.append(
            {
                "version": 1,
                "upstream_sha": PIN,
                "translation_unit": unit_id,
                "dispatch_ordinal": unit.get("dispatch_ordinal", "-"),
                "dispatch_prerequisites": ";".join(sorted(prerequisites)) or "-",
                "source_dependencies": ";".join(sorted(source_graph[unit_id])) or "-",
                "source_dependency_scc": component_ids.get(unit_id, "-"),
                "ordering_contract": "acyclic-dispatch-only",
                "evidence": "manifest:docs/metal-port-manifest.toml",
            }
        )
    cycles = [component for component in strongly_connected_components(dispatch_graph) if len(component) > 1]
    if cycles:
        raise ValueError(f"dispatch prerequisites must be acyclic; got {cycles}")
    ordinal_order = tuple(
        unit_id
        for unit_id, unit in sorted(
            units.items(), key=lambda item: int(item[1].get("dispatch_ordinal", 10_000))
        )
    )
    if ordinal_order != DISPATCH_ORDER:
        raise ValueError("dispatch ordinals do not match the stable global topological order")
    ordinals = {
        unit_id: unit.get("dispatch_ordinal") for unit_id, unit in units.items()
    }
    for unit_id, prerequisites in dispatch_graph.items():
        unit_ordinal = ordinals[unit_id]
        if not isinstance(unit_ordinal, int):
            continue
        for prerequisite in prerequisites:
            prerequisite_ordinal = ordinals.get(prerequisite)
            if isinstance(prerequisite_ordinal, int) and prerequisite_ordinal >= unit_ordinal:
                raise ValueError(
                    f"dispatch prerequisite {prerequisite} ordinal "
                    f"{prerequisite_ordinal} must precede {unit_id} ordinal {unit_ordinal}"
                )
    rows.sort(
        key=lambda row: (
            10_000 if row["dispatch_ordinal"] == "-" else int(row["dispatch_ordinal"]),
            str(row["translation_unit"]),
        )
    )
    return tsv(columns, rows)


MAKE_MATRIX = (
    ("make-minify", (21,), "minify", "all-34-minify-inputs", "batch-minify-once"),
    ("make-draw-combinations", (49,), "$(DRAW_COMBINATIONS_METAL)", "draw-generator", "generate-exact-permutation-include"),
    ("apple-macosx", (52, 62, 71), "rive_pls_macosx_metallib -> $(OUT)/macosx/rive_pls_macosx.metallib -> $(OUT)/rive_pls_macosx.metallib.c", "macosx/metal2.3/min-macos11", "translate-seven-way-apple-artifact"),
    ("apple-iphoneos", (53, 76, 85), "rive_pls_ios_metallib -> $(OUT)/ios/rive_pls_ios.metallib -> $(OUT)/rive_pls_ios.metallib.c", "iphoneos/metal2.2/min-ios13", "translate-seven-way-apple-artifact"),
    ("apple-iphonesimulator", (54, 88, 97), "rive_pls_ios_simulator_metallib -> $(OUT)/ios/rive_pls_ios_simulator.metallib -> $(OUT)/rive_pls_ios_simulator.metallib.c", "iphonesimulator/metal2.2/min-iossim13", "translate-seven-way-apple-artifact"),
    ("apple-xros", (55, 100, 109), "rive_renderer_xros_metallib -> $(OUT)/ios/rive_renderer_xros.metallib -> $(OUT)/rive_renderer_xros.metallib.c", "xros/metal3.1/air64-apple-xros1.0", "translate-seven-way-apple-artifact"),
    ("apple-xrsimulator", (56, 112, 121), "rive_renderer_xros_simulator_metallib -> $(OUT)/ios/rive_renderer_xros_simulator.metallib -> $(OUT)/rive_renderer_xros_simulator.metallib.c", "xrsimulator/metal3.1/air64-apple-xros1.0-simulator", "translate-seven-way-apple-artifact"),
    ("apple-appletvos", (57, 124, 133), "rive_renderer_appletvos_metallib -> $(OUT)/ios/rive_renderer_appletvos.metallib -> $(OUT)/rive_renderer_appletvos.metallib.c", "appletvos/metal3.0/min-tvos16", "translate-seven-way-apple-artifact"),
    ("apple-appletvsimulator", (58, 136, 145), "rive_renderer_appletvsimulator_metallib -> $(OUT)/ios/rive_renderer_appletvsimulator.metallib -> $(OUT)/rive_renderer_appletvsimulator.metallib.c", "appletvsimulator/metal3.0/min-tvossim16", "translate-seven-way-apple-artifact"),
    ("spirv", (411, 412), "spirv;spirv-binary", "SPIR-V/glslangValidator+spirv-opt+header", "preserve-full-source-non-metal-rule"),
    ("wgsl", (465,), "wgsl", "WGSL/Naga-keep-coordinate-space+header", "preserve-full-source-non-metal-rule"),
    ("d3d", (498,), "d3d", "D3D/FXC-vs5+ps5+rootsig1.1", "preserve-full-source-non-metal-rule"),
)

MAKE_SIGNATURES = {
    "make-minify": ("python3 minify.py $(FLAGS) -o $(OUT) $(MINIFY_INPUTS)",),
    "make-draw-combinations": ("python3 metal/generate_draw_combinations.py $(DRAW_COMBINATIONS_METAL)",),
    "apple-macosx": ("xcrun -sdk macosx metal -std=macos-metal2.3", "-mmacosx-version-min=11.0"),
    "apple-iphoneos": ("xcrun -sdk iphoneos metal -std=ios-metal2.2", "-mios-version-min=13"),
    "apple-iphonesimulator": ("xcrun -sdk iphonesimulator metal -std=ios-metal2.2", "-miphonesimulator-version-min=13"),
    "apple-xros": ("xcrun -sdk xros metal -std=metal3.1", "--target=air64-apple-xros1.0"),
    "apple-xrsimulator": ("xcrun -sdk xrsimulator metal -std=metal3.1", "--target=air64-apple-xros1.0-simulator"),
    "apple-appletvos": ("xcrun -sdk appletvos metal -std=metal3.0", "-mappletvos-version-min=16.0"),
    "apple-appletvsimulator": ("xcrun -sdk appletvsimulator metal -std=metal3.0", "-mappletvsimulator-version-min=16.0"),
    "spirv": ("glslangValidator", "spirv-opt", "spirv_binary_to_header.py"),
    "wgsl": ("naga --keep-coordinate-space", "wgsl_to_header.py"),
    "d3d": ("fxc /D VERTEX", "fxc /D FRAGMENT", "/T rootsig_1_1"),
}


def option_family(expression: str) -> str:
    if "human_readable" in expression:
        return "human-readable-output"
    if "ply_path" in expression:
        return "lexer-provider-path"
    if "msvc" in expression:
        return "msvc-header-output"
    if "outdir" in expression:
        return "output-directory"
    return "algorithmic"


def render_build_branches(upstream: pathlib.Path) -> str:
    columns = (
        "version", "upstream_sha", "authority_kind", "upstream_file", "entry_id",
        "line", "branch_kind", "condition_or_target", "option_family",
        "target_family", "translation_unit", "translation_target",
        "translation_disposition", "translation_behavior", "evidence",
    )
    rows: list[dict[str, object]] = []
    makefile = "renderer/src/shaders/Makefile"
    make_text = (upstream / makefile).read_text(encoding="utf-8")
    make_lines = make_text.splitlines()
    for entry_id, lines, target, target_family, behavior in MAKE_MATRIX:
        if any(line > len(make_lines) for line in lines):
            raise ValueError(f"Makefile matrix line is missing: {lines}")
        if any(signature not in make_text for signature in MAKE_SIGNATURES[entry_id]):
            raise ValueError(f"Makefile rule-family signature drifted: {entry_id}")
        rows.append(
            {
                "version": 1, "upstream_sha": PIN, "authority_kind": "make-rule-family",
                "upstream_file": makefile, "entry_id": entry_id,
                "line": ",".join(str(line) for line in lines),
                "branch_kind": "rule-family", "condition_or_target": target,
                "option_family": "-", "target_family": target_family,
                "translation_unit": "metal-shader-source-batch",
                "translation_target": "crates/nuxie-renderer/build.rs",
                "translation_disposition": "required",
                "translation_behavior": behavior,
                "evidence": ";".join(f"cpp:{makefile}:{line}" for line in lines),
            }
        )

    python_sources = (
        ("renderer/src/shaders/minify.py", 36, 6, "minify"),
        ("renderer/src/shaders/metal/generate_draw_combinations.py", 10, 1, "draw-generator"),
    )
    for source, expected_if, expected_ifexp, prefix in python_sources:
        tree = ast.parse((upstream / source).read_text(encoding="utf-8"), filename=source)
        nodes = sorted(
            (node for node in ast.walk(tree) if isinstance(node, (ast.If, ast.IfExp))),
            key=lambda node: (node.lineno, 0 if isinstance(node, ast.If) else 1, node.col_offset),
        )
        assert_exact(
            f"{source} If count", sum(isinstance(node, ast.If) for node in nodes), expected_if
        )
        assert_exact(
            f"{source} IfExp count",
            sum(isinstance(node, ast.IfExp) for node in nodes),
            expected_ifexp,
        )
        for ordinal, node in enumerate(nodes, 1):
            expression = ast.unparse(node.test)
            rows.append(
                {
                    "version": 1, "upstream_sha": PIN, "authority_kind": "python-branch",
                    "upstream_file": source, "entry_id": f"{prefix}-{ordinal:02d}",
                    "line": node.lineno,
                    "branch_kind": "If" if isinstance(node, ast.If) else "IfExp",
                    "condition_or_target": expression,
                    "option_family": option_family(expression) if prefix == "minify" else "algorithmic",
                    "target_family": prefix,
                    "translation_unit": "metal-shader-source-batch",
                    "translation_target": (
                        "crates/nuxie-renderer/src/native_metal/shaders/"
                        if prefix == "minify"
                        else "crates/nuxie-renderer/src/native_metal/draw_combinations.rs;"
                        "crates/nuxie-renderer/src/native_metal/shaders/draw_combinations.metal"
                    ),
                    "translation_disposition": "required",
                    "translation_behavior": "preserve-exact-branch",
                    "evidence": f"cpp:{source}:{node.lineno}",
                }
            )
    minify_families = {
        str(row["option_family"])
        for row in rows
        if row["upstream_file"] == "renderer/src/shaders/minify.py"
        and row["option_family"] != "algorithmic"
    }
    assert_exact("minify option-family count", len(minify_families), 4)
    return tsv(columns, rows)


def assert_exact(label: str, actual: int, expected: int) -> None:
    if actual != expected:
        raise ValueError(f"{label}: expected {expected}, got {actual}")


def build(repo: pathlib.Path, upstream: pathlib.Path) -> dict[pathlib.Path, str]:
    manifest = tomllib.loads((repo / "docs/metal-port-manifest.toml").read_text())
    if manifest.get("upstream_ref") != PIN:
        raise ValueError(f"manifest pin must be {PIN}")
    sources = source_scope(manifest, upstream)
    head = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=upstream,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if head != PIN:
        raise ValueError(f"upstream HEAD must be {PIN}, got {head}")
    dirty_sources = subprocess.run(
        ["git", "diff", "--name-only", PIN, "--", *sources],
        cwd=upstream,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()
    if dirty_sources:
        raise ValueError("pinned campaign sources are dirty: " + ", ".join(dirty_sources))
    owners, units = source_owners(manifest)
    if set(owners) != set(sources):
        raise ValueError(
            f"translation-unit source ownership must equal the {EXPECTED_SOURCE_COUNT}-source campaign scope"
        )
    blocks, _guards, occurrences = collect_authority(manifest, upstream, sources)
    correspondences = existing_rust_correspondences(repo, occurrences)
    dependencies, source_graph, component_ids = dependency_analysis(
        occurrences, owners, units, correspondences
    )
    return {
        PREPROCESSOR_PATH: render_preprocessor(blocks, owners, units),
        INCLUDE_PATH: render_includes(occurrences, owners, units, correspondences),
        SOURCE_DEPENDENCY_PATH: dependencies,
        DISPATCH_PATH: render_dispatch(units, source_graph, component_ids),
        BUILD_BRANCH_PATH: render_build_branches(upstream),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=pathlib.Path, default=pathlib.Path.cwd())
    parser.add_argument("--upstream-root", type=pathlib.Path, required=True)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    args = parser.parse_args()
    repo = args.repo_root.resolve()
    upstream = args.upstream_root.resolve()
    try:
        rendered = build(repo, upstream)
        if args.write:
            for relative, content in rendered.items():
                path = repo / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding="utf-8")
                print(f"wrote {relative} ({len(content.splitlines()) - 1} rows)")
        else:
            drift = []
            for relative, expected in rendered.items():
                path = repo / relative
                actual = path.read_text(encoding="utf-8") if path.is_file() else None
                if actual != expected:
                    drift.append(str(relative))
            if drift:
                raise ValueError("authority ledger drift: " + ", ".join(drift))
            print("Metal port authority ledgers are exhaustive and current")
    except (
        OSError,
        ValueError,
        KeyError,
        subprocess.CalledProcessError,
        tomllib.TOMLDecodeError,
    ) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
