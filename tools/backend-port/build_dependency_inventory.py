#!/usr/bin/env python3
"""Derive include/import closure and ownership-unit order from pinned sources."""

from __future__ import annotations

import argparse
import csv
import hashlib
import re
import sys
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path


EDGE_HEADER = (
    "campaign",
    "source_path",
    "line",
    "dependency_syntax",
    "dependency_token",
    "resolution_kind",
    "resolved_path",
    "resolved_sha256",
    "source_unit",
    "dependency_unit",
)
UNIT_HEADER = (
    "order_group",
    "component_id",
    "campaign",
    "ownership_unit",
    "dependency_units",
    "source_count",
)


@dataclass(frozen=True, order=True)
class Edge:
    campaign: str
    source_path: str
    line: int
    dependency_syntax: str
    dependency_token: str
    resolution_kind: str
    resolved_path: str
    resolved_sha256: str
    source_unit: str
    dependency_unit: str

    def tsv(self) -> str:
        return "\t".join(str(getattr(self, column)) for column in EDGE_HEADER)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--upstream-root", type=Path, required=True)
    parser.add_argument("--ownership-inventory", type=Path, required=True)
    parser.add_argument("--edges-output", type=Path, required=True)
    parser.add_argument("--units-output", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    return parser.parse_args()


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_ownership(path: Path) -> list[dict[str, str]]:
    with path.open(newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    if not rows:
        raise ValueError("ownership inventory is empty")
    if tuple(rows[0]) != (
        "campaign",
        "source_path",
        "source_sha256",
        "ownership_unit",
        "source_role",
        "port_disposition",
        "target_path",
        "mapping_status",
        "translation_status",
    ):
        raise ValueError("unexpected ownership inventory columns")
    return rows


CPP_DEPENDENCY = re.compile(
    r'^\s*#\s*(?P<kind>include|import)\s*(?P<open>[<"])(?P<token>[^>"]+)[>"]'
)
PYTHON_IMPORT = re.compile(
    r"^\s*(?:from\s+(?P<from>[A-Za-z_][\w.]*)\s+import|import\s+(?P<import>[A-Za-z_][\w.]*))"
)
JS_IMPORT = re.compile(
    r"(?:^\s*import(?:.+?from\s*)?[\"'](?P<import>[^\"']+)[\"']|require\([\"'](?P<require>[^\"']+)[\"']\))"
)
MAKE_INCLUDE = re.compile(r"^\s*-?include\s+(?P<token>[^#\s]+)")
SHELL_SOURCE = re.compile(r"^\s*(?:source|\.)\s+(?P<token>[^\s#]+)")
LUA_REQUIRE = re.compile(r"require\s*\(?\s*[\"'](?P<token>[^\"']+)[\"']")


def dependencies(path: Path) -> list[tuple[int, str, str, bool]]:
    try:
        lines = path.read_text(errors="strict").splitlines()
    except UnicodeDecodeError:
        return []
    found: list[tuple[int, str, str, bool]] = []
    for line_number, line in enumerate(lines, 1):
        cpp = CPP_DEPENDENCY.match(line)
        if cpp:
            found.append(
                (
                    line_number,
                    f"cpp-{cpp.group('kind')}",
                    cpp.group("token"),
                    cpp.group("open") == '"',
                )
            )
            continue
        suffix = path.suffix.lower()
        if suffix == ".py":
            match = PYTHON_IMPORT.match(line)
            if match:
                found.append(
                    (line_number, "python-import", match.group("from") or match.group("import"), False)
                )
        elif suffix == ".js":
            for match in JS_IMPORT.finditer(line):
                found.append(
                    (line_number, "javascript-import", match.group("import") or match.group("require"), True)
                )
        elif path.name == "Makefile":
            match = MAKE_INCLUDE.match(line)
            if match:
                found.append((line_number, "make-include", match.group("token"), True))
        elif suffix == ".sh":
            match = SHELL_SOURCE.match(line)
            if match:
                found.append((line_number, "shell-source", match.group("token"), True))
        elif suffix == ".lua":
            for match in LUA_REQUIRE.finditer(line):
                found.append((line_number, "lua-require", match.group("token"), False))
    return found


def resolve_file(upstream_root: Path, source_path: str, token: str) -> Path | None:
    source = upstream_root / source_path
    candidates = (
        source.parent / token,
        upstream_root / token,
        upstream_root / "include" / token,
        upstream_root / "renderer/include" / token,
        upstream_root / "renderer/src" / token,
        upstream_root / "renderer/src/shaders" / token,
        upstream_root / "renderer/glad" / token,
        upstream_root / "renderer/src/webgpu/wagyu-port/include" / token,
        upstream_root / "decoders/include" / token,
    )
    for candidate in candidates:
        try:
            resolved = candidate.resolve()
            resolved.relative_to(upstream_root)
        except (OSError, ValueError):
            continue
        if resolved.is_file():
            return resolved
    return None


def unresolved_kind(syntax: str, quoted: bool, token: str) -> str:
    if syntax in {"python-import", "lua-require"}:
        return "external-tool-module"
    if not quoted:
        return "external-sdk-or-system"
    generated_markers = (
        "generated",
        "_shaders.",
        ".generated.",
        ".minified.",
        "spirv.hpp",
        "wgsl.hpp",
        "astc_footprints.hpp",
    )
    if any(marker in token.lower() for marker in generated_markers):
        return "generated-output"
    if "$" in token or "$(" in token:
        return "build-expanded"
    return "unresolved-quoted"


def generated_source_path(token: str, source_rows: dict[str, dict[str, str]]) -> str | None:
    shader_root = "renderer/src/shaders/"
    if ".minified." in token and "/" not in token:
        candidate = shader_root + token.replace(".minified.", ".")
        return candidate if candidate in source_rows else None
    prefix = "generated/shaders/"
    if not token.startswith(prefix):
        return None
    relative = token[len(prefix) :]
    if relative.endswith(".exports.h"):
        candidate = shader_root + relative[: -len(".exports.h")]
        return candidate if candidate in source_rows else None
    if relative.endswith(".hpp") and not relative.startswith(("spirv/", "wgsl/")):
        candidate = shader_root + relative[: -len(".hpp")]
        return candidate if candidate in source_rows else None
    if relative.startswith(("spirv/", "wgsl/")):
        output_name = relative.split("/", 1)[1]
        candidates = [
            source_path
            for source_path in source_rows
            if source_path.startswith(shader_root + "spirv/")
            and output_name.startswith(Path(source_path).stem + ".")
        ]
        if candidates:
            return max(candidates, key=lambda value: len(Path(value).stem))
    return None


def strongly_connected_components(graph: dict[str, set[str]]) -> list[list[str]]:
    index = 0
    stack: list[str] = []
    on_stack: set[str] = set()
    indices: dict[str, int] = {}
    lowlinks: dict[str, int] = {}
    components: list[list[str]] = []

    def visit(node: str) -> None:
        nonlocal index
        indices[node] = index
        lowlinks[node] = index
        index += 1
        stack.append(node)
        on_stack.add(node)
        for dependency in sorted(graph[node]):
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
    return components


def order_components(graph: dict[str, set[str]], components: list[list[str]]) -> dict[int, int]:
    component_for = {
        member: component_index
        for component_index, component in enumerate(components)
        for member in component
    }
    dependencies: dict[int, set[int]] = {index: set() for index in range(len(components))}
    for owner, owner_dependencies in graph.items():
        owner_component = component_for[owner]
        for dependency in owner_dependencies:
            dependency_component = component_for[dependency]
            if dependency_component != owner_component:
                dependencies[owner_component].add(dependency_component)
    order: dict[int, int] = {}
    remaining = set(dependencies)
    group = 0
    while remaining:
        ready = sorted(index for index in remaining if not (dependencies[index] & remaining))
        if not ready:
            raise ValueError("component graph unexpectedly cyclic")
        for index in ready:
            order[index] = group
            remaining.remove(index)
        group += 1
    return order


def render(
    upstream_root: Path, ownership_path: Path
) -> tuple[str, str, dict[str, int]]:
    ownership = load_ownership(ownership_path)
    source_rows = {row["source_path"]: row for row in ownership}
    units = {row["ownership_unit"] for row in ownership}
    unit_campaigns: dict[str, set[str]] = defaultdict(set)
    unit_sources: dict[str, set[str]] = defaultdict(set)
    for row in ownership:
        unit_campaigns[row["ownership_unit"]].add(row["campaign"])
        unit_sources[row["ownership_unit"]].add(row["source_path"])

    edges: list[Edge] = []
    graph: dict[str, set[str]] = {unit: set() for unit in units}
    resolution_counts: dict[str, int] = defaultdict(int)
    for row in ownership:
        source_path = row["source_path"]
        source_file = upstream_root / source_path
        if digest(source_file) != row["source_sha256"]:
            raise ValueError(f"pinned source drift: {source_path}")
        for line, syntax, token, quoted in dependencies(source_file):
            resolved = resolve_file(upstream_root, source_path, token)
            resolved_path = "-"
            resolved_sha = "-"
            dependency_unit = "-"
            if resolved is None:
                kind = unresolved_kind(syntax, quoted, token)
                if kind == "generated-output":
                    generated_source = generated_source_path(token, source_rows)
                    if generated_source is not None:
                        dependency_row = source_rows[generated_source]
                        kind = "generated-from-owned-source"
                        resolved_path = generated_source
                        resolved_sha = dependency_row["source_sha256"]
                        dependency_unit = dependency_row["ownership_unit"]
                        if dependency_unit != row["ownership_unit"]:
                            graph[row["ownership_unit"]].add(dependency_unit)
            else:
                resolved_path = resolved.relative_to(upstream_root).as_posix()
                resolved_sha = digest(resolved)
                dependency_row = source_rows.get(resolved_path)
                if dependency_row is None:
                    kind = "pinned-source-external"
                else:
                    kind = "owned-source"
                    dependency_unit = dependency_row["ownership_unit"]
                    if dependency_unit != row["ownership_unit"]:
                        graph[row["ownership_unit"]].add(dependency_unit)
            resolution_counts[kind] += 1
            edges.append(
                Edge(
                    campaign=row["campaign"],
                    source_path=source_path,
                    line=line,
                    dependency_syntax=syntax,
                    dependency_token=token,
                    resolution_kind=kind,
                    resolved_path=resolved_path,
                    resolved_sha256=resolved_sha,
                    source_unit=row["ownership_unit"],
                    dependency_unit=dependency_unit,
                )
            )
    edges.sort()
    edge_text = "\n".join(("\t".join(EDGE_HEADER), *(edge.tsv() for edge in edges))) + "\n"

    components = strongly_connected_components(graph)
    component_for = {
        member: component_index
        for component_index, component in enumerate(components)
        for member in component
    }
    component_order = order_components(graph, components)
    unit_lines: list[str] = []
    for unit in sorted(units, key=lambda value: (component_order[component_for[value]], value)):
        component_index = component_for[unit]
        campaigns = sorted(unit_campaigns[unit])
        if len(campaigns) != 1:
            raise ValueError(f"ownership unit crosses campaigns: {unit}: {campaigns}")
        unit_lines.append(
            "\t".join(
                (
                    str(component_order[component_index]),
                    f"component-{component_index:03d}",
                    campaigns[0],
                    unit,
                    ";".join(sorted(graph[unit])),
                    str(len(unit_sources[unit])),
                )
            )
        )
    unit_text = "\n".join(("\t".join(UNIT_HEADER), *unit_lines)) + "\n"
    return edge_text, unit_text, dict(resolution_counts)


def resolve_repo_path(repo_root: Path, path: Path) -> Path:
    return path if path.is_absolute() else repo_root / path


def main() -> int:
    args = parse_args()
    ownership = resolve_repo_path(args.repo_root, args.ownership_inventory)
    edges_output = resolve_repo_path(args.repo_root, args.edges_output)
    units_output = resolve_repo_path(args.repo_root, args.units_output)
    edge_text, unit_text, counts = render(args.upstream_root.resolve(), ownership)
    if args.check:
        stale = []
        for path, expected in ((edges_output, edge_text), (units_output, unit_text)):
            if not path.is_file() or path.read_text() != expected:
                stale.append(str(path))
        if stale:
            print(f"backend dependency inventory is stale: {', '.join(stale)}", file=sys.stderr)
            return 1
        print(
            "backend dependency inventory clean: "
            f"{len(edge_text.splitlines()) - 1} edges, "
            f"{len(unit_text.splitlines()) - 1} units; "
            + ", ".join(f"{key}={counts[key]}" for key in sorted(counts))
        )
        return 0
    edges_output.parent.mkdir(parents=True, exist_ok=True)
    units_output.parent.mkdir(parents=True, exist_ok=True)
    edges_output.write_text(edge_text)
    units_output.write_text(unit_text)
    print(
        f"wrote {len(edge_text.splitlines()) - 1} dependency edges and "
        f"{len(unit_text.splitlines()) - 1} ownership units"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
