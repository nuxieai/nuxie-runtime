#!/usr/bin/env python3
"""Reject wgpu indirect execution from the production renderer source tree."""

from __future__ import annotations

import argparse
import pathlib
import re
import sys


# With VALIDATION_INDIRECT_CALL disabled, malformed indirect arguments have
# undefined behavior and wgpu's validation path would compile internal WGSL.
# Keep this list aligned with the public RenderPass, RenderBundleEncoder, and
# ComputePass indirect entry points in the pinned wgpu API.
FORBIDDEN_IDENTIFIERS = frozenset(
    {
        "dispatch_workgroups_indirect",
        "draw_indirect",
        "draw_indexed_indirect",
        "draw_mesh_tasks_indirect",
        "multi_draw_indirect",
        "multi_draw_indirect_count",
        "multi_draw_indexed_indirect",
        "multi_draw_indexed_indirect_count",
        "multi_draw_mesh_tasks_indirect",
        "multi_draw_mesh_tasks_indirect_count",
    }
)

# Renderer-owned built-ins must enter through shader_catalog::create so the
# committed Apple artifacts and their translation keys remain exhaustive.
# Authored GPU-canvas source is deliberately dynamic and has its own trust and
# reflection path (UNIV-2073), so those two creation sites stay explicit.
EXPECTED_SHADER_MODULE_CREATION_SITES = {
    pathlib.PurePosixPath("gpu_canvas_shader.rs"): 2,
    pathlib.PurePosixPath("shader_catalog.rs"): 1,
}

RAW_LITERAL_PREFIX = re.compile(r'(?:br|cr|r)(?P<hashes>#{0,255})"')


def _blank_literal_or_comment(source: str) -> str:
    """Blank Rust comments and literals while preserving offsets and newlines."""

    chars = list(source)
    length = len(source)
    index = 0

    def blank(start: int, end: int) -> None:
        for position in range(start, end):
            if chars[position] != "\n":
                chars[position] = " "

    while index < length:
        if source.startswith("//", index):
            end = source.find("\n", index + 2)
            if end == -1:
                end = length
            blank(index, end)
            index = end
            continue

        if source.startswith("/*", index):
            start = index
            index += 2
            depth = 1
            while index < length and depth:
                if source.startswith("/*", index):
                    depth += 1
                    index += 2
                elif source.startswith("*/", index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
            blank(start, index)
            continue

        raw = (
            RAW_LITERAL_PREFIX.match(source, index)
            if source[index] in {"b", "c", "r"}
            else None
        )
        if raw:
            start = index
            terminator = '"' + raw.group("hashes")
            body = index + raw.end()
            end = source.find(terminator, body)
            index = length if end == -1 else end + len(terminator)
            blank(start, index)
            continue

        prefix_length = 1 if source[index : index + 2] in {'b"', 'c"'} else 0
        if source[index + prefix_length : index + prefix_length + 1] == '"':
            start = index
            index += prefix_length + 1
            escaped = False
            while index < length:
                char = source[index]
                index += 1
                if char == '"' and not escaped:
                    break
                escaped = char == "\\" and not escaped
                if char != "\\":
                    escaped = False
            blank(start, index)
            continue

        # A Rust character literal closes before the next newline. A lifetime
        # such as `'shader` does not, so leave it available to the identifier
        # scanner.
        if source[index] == "'":
            end = index + 1
            escaped = False
            while end < length and source[end] != "\n":
                char = source[end]
                end += 1
                if char == "'" and not escaped:
                    blank(index, end)
                    index = end
                    break
                escaped = char == "\\" and not escaped
                if char != "\\":
                    escaped = False
            else:
                index += 1
            continue

        index += 1

    return "".join(chars)


def findings(source_root: pathlib.Path) -> list[tuple[pathlib.Path, int, str]]:
    result: list[tuple[pathlib.Path, int, str]] = []
    for path in sorted(source_root.rglob("*.rs")):
        source = path.read_text(encoding="utf-8")
        code = _blank_literal_or_comment(source)
        for match in re.finditer(r"\b[A-Za-z_][A-Za-z0-9_]*\b", code):
            identifier = match.group(0)
            if identifier in FORBIDDEN_IDENTIFIERS:
                line = source.count("\n", 0, match.start()) + 1
                result.append((path, line, identifier))
    return result


def shader_module_creation_sites(source_root: pathlib.Path) -> dict[pathlib.PurePosixPath, int]:
    result: dict[pathlib.PurePosixPath, int] = {}
    for path in sorted(source_root.rglob("*.rs")):
        code = _blank_literal_or_comment(path.read_text(encoding="utf-8"))
        count = len(re.findall(r"\bcreate_shader_module\b", code))
        if count:
            result[path.relative_to(source_root)] = count
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "source_root",
        nargs="?",
        type=pathlib.Path,
        default=pathlib.Path("crates/nuxie-renderer/src"),
    )
    args = parser.parse_args(argv)
    source_root = args.source_root.resolve()
    if not source_root.is_dir():
        parser.error(f"renderer source root is not a directory: {source_root}")

    violations = findings(source_root)
    if violations:
        for path, line, identifier in violations:
            print(
                f"{path}:{line}: forbidden indirect renderer API `{identifier}`; "
                "the Apple instance intentionally disables wgpu indirect validation",
                file=sys.stderr,
            )
        return 1


    creation_sites = shader_module_creation_sites(source_root)
    if creation_sites != EXPECTED_SHADER_MODULE_CREATION_SITES:
        print(
            "unexpected direct create_shader_module sites; built-ins must use "
            "shader_catalog::create",
            file=sys.stderr,
        )
        print(f"expected: {EXPECTED_SHADER_MODULE_CREATION_SITES}", file=sys.stderr)
        print(f"actual:   {creation_sites}", file=sys.stderr)
        return 1

    print(
        "renderer shader boundary: catalog-only built-ins; direct draw/dispatch only"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
