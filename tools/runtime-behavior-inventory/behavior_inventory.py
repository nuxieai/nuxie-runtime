#!/usr/bin/env python3
"""Discover and gate behavior-bearing C++ members and Rust items."""

from __future__ import annotations

import argparse
import bisect
import collections
import hashlib
import json
import pathlib
import re
import subprocess
import sys
import tomllib
import unicodedata
from typing import Any

SCHEMA = "nuxie-runtime-behavior-inventory/v1"
CPP_SUFFIXES = {".cpp", ".mm", ".h", ".hpp"}
UPSTREAM_SOURCE_ROOTS = ("src", "include/rive")
CPP_OWNER_ALIASES = {
    # Apple builds compile the same owners through ObjC++ wrappers; the text
    # owner adds CoreText behavior beside the portable HarfBuzz implementation.
    "src/lua/renderer/lua_gpu_apple.mm": "src/lua/renderer/lua_gpu.cpp",
    "src/lua/lua_scripted_context_apple.mm": "src/lua/lua_scripted_context.cpp",
    "src/text/font_hb_apple.mm": "src/text/font_hb.cpp",
}
RUST_CRATES = (
    "nuxie",
    "nuxie-audio",
    "nuxie-binary",
    "nuxie-graph",
    "nuxie-image-codec",
    "nuxie-project-data",
    "nuxie-render-api",
    "nuxie-render-stream",
    "nuxie-renderer",
    "nuxie-renderer-ffi",
    "nuxie-runtime",
    "nuxie-schema",
    "nuxie-scripting",
    "nux-capi",
    "nux-apple-product-extension",
)
ADAPTATION_CATEGORIES = {
    "baseline-adaptation",
    "platform-adapter",
    "retained-render",
}
MAX_CFG_ATOMS = 16
GENERATED_RUST_FILES = {
    "crates/nuxie-schema/src/generated/schema.rs": "tools/nuxie-codegen",
}
RUST_GENERATOR_OUTPUTS = {
    "crates/nux-apple-product-extension/build.rs": [
        "rustc-env:NUX_APPLE_PRODUCT_EXTENSION_BUILD_PROVENANCE",
    ],
    "crates/nux-capi/build.rs": [
        "rustc-env:NUX_RUNTIME_SOURCE_REVISION",
        "rustc-env:NUX_CAPI_BUILD_PROVENANCE",
        "OUT_DIR/nux_capi.generated.h",
        "crates/nux-capi/include/nux_capi.generated.h",
    ],
    "crates/nuxie-renderer-ffi/build.rs": [
        "native-link:nuxie_renderer_ffi",
    ],
    "crates/nuxie-runtime/build.rs": [
        "OUT_DIR/runtime_objects.rs",
    ],
    "crates/nuxie-scripting/build.rs": [
        "OUT_DIR/promise-library.luau-bytecode",
        "OUT_DIR/data-value-metatable.luau-bytecode",
        "OUT_DIR/property-metatable.luau-bytecode",
    ],
}
NAMED_ADAPTATION_PATH_RULES = {
    "crates/nuxie-audio/src/source.rs": (
        "decoded-pcm-resampler",
        None,
        ["src/audio/audio_reader.cpp", "src/audio/audio_source.cpp"],
        [
            "change-frame-clock",
            "change-playback-lifecycle",
            "exceed-two-frame-tolerance",
        ],
        ["docs/PORTING.md:D17", "crates/nuxie-audio/src/source.rs"],
    ),
    "crates/nuxie-runtime/src/profiler.rs": (
        "profile-capture-backend",
        {
            "impl Default for SystemProfileCapture",
            "impl SystemProfileCapture",
            "impl ProfileCapture for SystemProfileCapture",
        },
        ["src/profiler/profiler.cpp"],
        ["change-rive-profile-wire", "change-capture-lifecycle"],
        ["docs/PORTING.md:D16", "crates/nuxie-runtime/src/profiler.rs"],
    ),
    "crates/nuxie-runtime/src/layout/artboard_component_list_override.rs": (
        "layout-engine",
        None,
        ["src/layout/artboard_component_list_override.cpp"],
        ["change-membership", "skip-layout-invalidation"],
        ["docs/PORTING.md:D3", "docs/runtime-drawing-ownership.toml"],
    ),
    "crates/nuxie-scripting/src/gpu_canvas.rs": (
        "lua-gpu-wgpu-adapter",
        None,
        ["src/lua/renderer/lua_gpu.cpp"],
        ["drop-gpu-name", "share-occurrence-module", "substitute-cpu-rendering"],
        [
            "docs/PORTING.md:D18",
            "crates/nuxie-scripting/tests/async_shader_instantiation.rs",
        ],
    ),
    "crates/nuxie-renderer/src/gpu_canvas.rs": (
        "lua-gpu-wgpu-adapter",
        None,
        ["src/lua/renderer/lua_gpu.cpp"],
        ["drop-gpu-name", "share-occurrence-module", "substitute-cpu-rendering"],
        ["docs/PORTING.md:D18", "crates/nuxie/tests/imported_gpu_canvas.rs"],
    ),
    "crates/nuxie-renderer/src/gpu_canvas_shader.rs": (
        "lua-gpu-wgpu-adapter",
        None,
        ["src/lua/renderer/lua_gpu.cpp"],
        ["drop-gpu-name", "share-occurrence-module", "substitute-cpu-rendering"],
        ["docs/PORTING.md:D18", "crates/nuxie/tests/imported_gpu_canvas.rs"],
    ),
    "crates/nuxie-render-api/src/lib.rs": (
        "lua-gpu-wgpu-adapter",
        re.compile(r"GpuCanvas"),
        ["src/lua/renderer/lua_gpu.cpp"],
        ["drop-gpu-name", "share-occurrence-module", "substitute-cpu-rendering"],
        ["docs/PORTING.md:D18", "crates/nuxie/tests/imported_gpu_canvas.rs"],
    ),
    "crates/nuxie-audio/src/lib.rs": (
        "audio-host-backend",
        None,
        ["src/audio/audio_engine.cpp"],
        ["change-authoritative-mixer", "change-audio-lifecycle"],
        ["docs/PORTING.md:D17", "crates/nuxie-audio/src/audio_engine.rs"],
    ),
    "crates/nuxie-render-api/src/serializing.rs": (
        "retained-render-command-serialization",
        None,
        ["src/command_queue.cpp"],
        ["change-command-order", "change-command-payload"],
        [
            "docs/PORTING.md:A1",
            "crates/nuxie-render-api/src/serializing.rs",
        ],
    ),
    "crates/nuxie-runtime/src/focus.rs": (
        "host-focus-bridge",
        None,
        ["src/input/focus_manager.cpp"],
        ["change-focus-order", "skip-focus-notification"],
        ["docs/PORTING.md:A2", "crates/nuxie-runtime/src/focus.rs"],
    ),
    "crates/nuxie-runtime/src/external_data_converter.rs": (
        "external-data-converter-host",
        None,
        ["src/data_bind/converters/data_converter.cpp"],
        ["change-conversion-order", "change-data-bind-dirt"],
        [
            "docs/PORTING.md:A3",
            "crates/nuxie-runtime/src/external_data_converter.rs",
        ],
    ),
    "crates/nuxie-scripting/src/envelope.rs": (
        "authenticated-script-envelope",
        None,
        ["src/assets/script_asset.cpp"],
        ["bypass-authentication", "change-script-payload"],
        ["docs/PORTING.md:AF-5", "crates/nuxie-scripting/src/envelope.rs"],
    ),
    "crates/nuxie-scripting/src/host_commands.rs": (
        "script-host-command-bridge",
        None,
        ["src/command_server.cpp"],
        ["change-command-order", "change-command-payload"],
        [
            "docs/PORTING.md:A5",
            "crates/nuxie-scripting/src/host_commands.rs",
        ],
    ),
    "crates/nuxie-scripting/src/lib.rs": (
        "script-runtime-crate-boundary",
        None,
        ["src/lua/lua_state.cpp"],
        ["change-script-lifecycle", "bypass-resource-limits"],
        ["docs/PORTING.md:AF-1", "crates/nuxie-scripting/src/lib.rs"],
    ),
    "crates/nuxie-scripting/src/vm/bytecode.rs": (
        "validated-luau-bytecode",
        None,
        ["src/assets/script_asset.cpp"],
        ["load-unvalidated-bytecode", "change-bytecode-ownership"],
        [
            "docs/PORTING.md:A7",
            "crates/nuxie-scripting/src/vm/bytecode.rs",
        ],
    ),
    "crates/nuxie-scripting/src/vm/resource_limits.rs": (
        "terminal-script-resource-limits",
        None,
        ["src/lua/lua_state.cpp"],
        ["retry-terminal-error", "continue-script-traversal"],
        [
            "docs/PORTING.md:A8",
            "crates/nuxie-scripting/src/vm/resource_limits.rs",
        ],
    ),
}
NAMED_EXTENSION_RULES = {
    (
        "crates/nuxie-runtime/src/artboard.rs",
        "impl RuntimeSemanticGeometryAuthority",
        "new",
    ): (
        "semantic-geometry-cache-authority",
        ["src/artboard.cpp", "src/shapes/clipping_shape.cpp"],
        ["skip-baseline-update", "skip-baseline-draw"],
        [
            "docs/PORTING.md:X1",
            "crates/nuxie-runtime/tests/semantic_geometry_revision.rs",
        ],
    ),
    (
        "crates/nuxie-runtime/src/artboard.rs",
        "impl RuntimeSemanticGeometryAuthority",
        "require_coverage",
    ): (
        "semantic-geometry-cache-authority",
        ["src/artboard.cpp", "src/shapes/clipping_shape.cpp"],
        ["skip-baseline-update", "skip-baseline-draw"],
        [
            "docs/PORTING.md:X1",
            "crates/nuxie-runtime/tests/semantic_geometry_revision.rs",
        ],
    ),
    (
        "crates/nuxie-runtime/src/artboard.rs",
        "impl ArtboardInstance",
        "adopt_semantic_geometry_authority",
    ): (
        "semantic-geometry-cache-authority",
        ["src/artboard.cpp", "src/shapes/clipping_shape.cpp"],
        ["skip-baseline-update", "skip-baseline-draw"],
        [
            "docs/PORTING.md:X1",
            "crates/nuxie-runtime/tests/semantic_geometry_revision.rs",
        ],
    ),
    (
        "crates/nuxie-runtime/src/artboard.rs",
        "impl ArtboardInstance",
        "mark_semantic_geometry_changed",
    ): (
        "semantic-geometry-cache-authority",
        ["src/artboard.cpp", "src/shapes/clipping_shape.cpp"],
        ["skip-baseline-update", "skip-baseline-draw"],
        [
            "docs/PORTING.md:X1",
            "crates/nuxie-runtime/tests/semantic_geometry_revision.rs",
        ],
    ),
    (
        "crates/nuxie-runtime/src/artboard.rs",
        "impl ArtboardInstance",
        "try_semantic_geometry_revision",
    ): (
        "semantic-geometry-cache-authority",
        ["src/artboard.cpp", "src/shapes/clipping_shape.cpp"],
        ["skip-baseline-update", "skip-baseline-draw"],
        [
            "docs/PORTING.md:X1",
            "crates/nuxie-runtime/tests/semantic_geometry_revision.rs",
        ],
    ),
    (
        "crates/nuxie-runtime/src/artboard.rs",
        "impl ArtboardInstance",
        "set_script_input_for_global_occurrences_if_changed",
    ): (
        "scripted-global-occurrence-broadcast",
        ["src/artboard.cpp", "src/scripted/scripted_object.cpp"],
        ["skip-script-update", "coalesce-baseline-setter", "change-lua-assignment"],
        ["docs/PORTING.md:X2", "crates/nuxie-runtime/src/artboard/tests.rs"],
    ),
    (
        "crates/nuxie-scripting/src/gpu_canvas.rs",
        "impl GpuCanvasBytecodeProgram",
        "set_number_input",
    ): (
        "direct-gpu-bytecode-input-projection",
        ["src/lua/renderer/lua_gpu.cpp", "src/scripted/scripted_object.cpp"],
        ["synthesize-component-dirt", "change-scalar-conversion"],
        ["docs/PORTING.md:X3", "crates/nuxie-scripting/src/gpu_canvas.rs"],
    ),
    (
        "crates/nuxie-scripting/src/gpu_canvas.rs",
        "impl GpuCanvasBytecodeProgram",
        "set_boolean_input",
    ): (
        "direct-gpu-bytecode-input-projection",
        ["src/lua/renderer/lua_gpu.cpp", "src/scripted/scripted_object.cpp"],
        ["synthesize-component-dirt", "change-scalar-conversion"],
        ["docs/PORTING.md:X3", "crates/nuxie-scripting/src/gpu_canvas.rs"],
    ),
    (
        "crates/nuxie-scripting/src/gpu_canvas.rs",
        "impl GpuCanvasBytecodeProgram",
        "set_string_input",
    ): (
        "direct-gpu-bytecode-input-projection",
        ["src/lua/renderer/lua_gpu.cpp", "src/scripted/scripted_object.cpp"],
        ["synthesize-component-dirt", "change-scalar-conversion"],
        ["docs/PORTING.md:X3", "crates/nuxie-scripting/src/gpu_canvas.rs"],
    ),
}
CONTROL_HEADS = {
    "if",
    "for",
    "while",
    "switch",
    "catch",
    "else",
    "do",
    "try",
    "match",
    "loop",
    "case",
}

CPP_CONTROL_HEADS = CONTROL_HEADS - {"loop", "match"}
CPP_BEHAVIORAL_MACRO_CONTROL = CPP_CONTROL_HEADS | {
    "assert",
    "unreachable",
    "return",
    "co_return",
    "co_yield",
    "co_await",
    "break",
    "continue",
    "goto",
    "throw",
    "default",
    "new",
    "delete",
}


def sha256_text(value: str) -> str:
    return hashlib.sha256(value.encode()).hexdigest()


def normalize_space(value: str) -> str:
    return " ".join(value.split())


def owner_family(path: str) -> str:
    parts = pathlib.PurePosixPath(path).parts
    if path.startswith("src/") and len(parts) > 2:
        return parts[1]
    if path.startswith("include/rive/") and len(parts) > 3:
        return parts[2]
    if path.startswith("crates/") and len(parts) > 1:
        return parts[1]
    return parts[0] if parts else "unknown"


def cpp_logical_prefix(
    source: str, start: int, limit: int = 300
) -> tuple[str, list[int]]:
    """Return phase-2 C++ characters and their physical source positions."""
    text = []
    positions = []
    index = start
    while index < len(source) and len(text) < limit:
        if source.startswith("\\\r\n", index):
            index += 3
            continue
        if source.startswith("\\\n", index):
            index += 2
            continue
        text.append(source[index])
        positions.append(index)
        index += 1
    return "".join(text), positions


def cpp_spliced_text(source: str) -> tuple[str, list[int]]:
    """Remove C++ phase-2 line splices and map logical bytes to physical bytes."""
    literal_view = mask_noncode(
        source,
        cpp_line_splices=True,
        preserve_literals=True,
    )
    text = []
    positions = []
    index = 0
    raw_terminator = ""
    while index < len(source):
        if raw_terminator:
            if source.startswith(raw_terminator, index):
                for offset in range(len(raw_terminator)):
                    text.append(source[index + offset])
                    positions.append(index + offset)
                index += len(raw_terminator)
                raw_terminator = ""
                continue
            text.append(source[index])
            positions.append(index)
            index += 1
            continue
        if source[index] == "R" and literal_view[index] == "R":
            logical, logical_positions = cpp_logical_prefix(source, index)
            match = re.match(r'R"([^ ()\\\t\r\n]{0,16})\(', logical)
            if match:
                for char, position in zip(
                    match.group(0), logical_positions[: len(match.group(0))]
                ):
                    text.append(char)
                    positions.append(position)
                index = logical_positions[len(match.group(0)) - 1] + 1
                raw_terminator = ")" + match.group(1) + '"'
                continue
        if source.startswith("\\\r\n", index):
            index += 3
            continue
        if source.startswith("\\\n", index):
            index += 2
            continue
        text.append(source[index])
        positions.append(index)
        index += 1
    return "".join(text), positions


def mask_noncode(
    source: str,
    *,
    nested_block_comments: bool = False,
    cpp_line_splices: bool = False,
    preserve_literals: bool = False,
    preserve_line_comments: bool = False,
) -> str:
    """Mask comments and literals while preserving byte positions and newlines."""
    chars = list(source)
    index = 0
    state = "code"
    quote = ""
    raw_terminator = ""
    block_comment_depth = 0
    while index < len(chars):
        char = chars[index]
        nxt = chars[index + 1] if index + 1 < len(chars) else ""
        if state == "code":
            comment_next = index + 1
            if cpp_line_splices and char == "/":
                while source.startswith("\\\n", comment_next) or source.startswith(
                    "\\\r\n", comment_next
                ):
                    chars[comment_next] = " "
                    if source.startswith("\\\r\n", comment_next):
                        chars[comment_next + 1] = " "
                        comment_next += 3
                    else:
                        comment_next += 2
            logical_next = chars[comment_next] if comment_next < len(chars) else ""
            if char == "/" and logical_next == "/":
                if not preserve_line_comments:
                    chars[index] = chars[comment_next] = " "
                index = comment_next + 1
                state = "line-comment"
                continue
            if char == "/" and logical_next == "*":
                chars[index] = chars[comment_next] = " "
                index = comment_next + 1
                block_comment_depth = 1
                state = "block-comment"
                continue
            if char == "'" and not re.match(r"'(?:\\.|[^\\'\n])'", source[index:]):
                # Rust lifetimes (`'a`, `'static`) are code, not character
                # literals. A character literal always has its closing quote
                # in this compact form; raw/multibyte content is handled by
                # the normal double-quoted path below.
                index += 1
                continue
            if not nested_block_comments and char == "R":
                raw_source = source[index:]
                raw_positions = list(range(index, min(len(source), index + 300)))
                if cpp_line_splices:
                    raw_source, raw_positions = cpp_logical_prefix(source, index)
                match = re.match(r'R"([^ ()\\\t\r\n]{0,16})\(', raw_source)
                if match:
                    raw_terminator = ")" + match.group(1) + '"'
                    opening_positions = raw_positions[: len(match.group(0))]
                    if not preserve_literals:
                        for position in opening_positions:
                            chars[position] = " "
                    index = opening_positions[-1] + 1
                    state = "raw-literal"
                    continue
            if char in {'"', "'"}:
                quote = char
                if not preserve_literals:
                    chars[index] = " "
                index += 1
                state = "literal"
                continue
            if char == "r" and nxt in {'"', "#"}:
                match = re.match(r'r(#{0,255})"', source[index:])
                if match:
                    raw_terminator = '"' + match.group(1)
                    if not preserve_literals:
                        for offset in range(len(match.group(0))):
                            chars[index + offset] = " "
                    index += len(match.group(0))
                    state = "raw-literal"
                    continue
        elif state == "line-comment":
            if cpp_line_splices and (
                source.startswith("\\\n", index) or source.startswith("\\\r\n", index)
            ):
                # C++ translation phase 2 removes the splice before comments
                # are recognized, so the physical next line remains inside
                # this line comment. Preserve its newline for source mapping.
                chars[index] = " "
                if source.startswith("\\\r\n", index):
                    chars[index + 1] = " "
                    index += 3
                else:
                    index += 2
                continue
            if char == "\n":
                state = "code"
            elif not preserve_line_comments:
                chars[index] = " "
        elif state == "block-comment":
            if nested_block_comments and char == "/" and nxt == "*":
                chars[index] = chars[index + 1] = " "
                index += 2
                block_comment_depth += 1
                continue
            comment_close = index + 1
            if cpp_line_splices and char == "*":
                while source.startswith("\\\n", comment_close) or source.startswith(
                    "\\\r\n", comment_close
                ):
                    chars[comment_close] = " "
                    if source.startswith("\\\r\n", comment_close):
                        chars[comment_close + 1] = " "
                        comment_close += 3
                    else:
                        comment_close += 2
            logical_close = chars[comment_close] if comment_close < len(chars) else ""
            if char == "*" and logical_close == "/":
                chars[index] = chars[comment_close] = " "
                index = comment_close + 1
                block_comment_depth -= 1
                if block_comment_depth == 0:
                    state = "code"
                continue
            if char != "\n":
                chars[index] = " "
        elif state == "literal":
            if char == "\\":
                if not preserve_literals:
                    chars[index] = " "
                if index + 1 < len(chars):
                    if not preserve_literals and chars[index + 1] != "\n":
                        chars[index + 1] = " "
                    index += 2
                    continue
            if char == quote:
                state = "code"
            if not preserve_literals and char != "\n":
                chars[index] = " "
        elif state == "raw-literal":
            terminator_source = ""
            terminator_positions: list[int] = []
            if char == raw_terminator[0]:
                terminator_source = source[index:]
                terminator_positions = list(
                    range(index, min(len(source), index + len(raw_terminator) + 8))
                )
                # Phase-2 transformations are reverted for raw-string
                # contents, so a splice-looking sequence cannot create the
                # closing delimiter once the raw payload has begun.
            if terminator_source.startswith(raw_terminator):
                closing_positions = terminator_positions[: len(raw_terminator)]
                if not preserve_literals:
                    for position in closing_positions:
                        chars[position] = " "
                index = closing_positions[-1] + 1
                state = "code"
                continue
            if not preserve_literals and char != "\n":
                chars[index] = " "
        index += 1
    return "".join(chars)


def mask_cpp_preprocessor(source: str, masked: str) -> str:
    """Mask complete C++ directives before structural member discovery."""
    chars = list(masked)
    offset = 0
    continuation = False
    masked_lines = masked.splitlines(keepends=True)
    for line, masked_line in zip(source.splitlines(keepends=True), masked_lines):
        directive = continuation or masked_line.lstrip().startswith("#")
        if directive:
            for index, char in enumerate(line):
                if char not in "\r\n":
                    chars[offset + index] = " "
            continuation = line.rstrip("\r\n").rstrip().endswith("\\")
        else:
            continuation = False
        offset += len(line)
    return "".join(chars)


def mask_cpp_source(source: str) -> str:
    return mask_cpp_preprocessor(source, mask_noncode(source, cpp_line_splices=True))


def matching_brace(masked: str, opening: int) -> int | None:
    depth = 0
    for index in range(opening, len(masked)):
        if masked[index] == "{":
            depth += 1
        elif masked[index] == "}":
            depth -= 1
            if depth == 0:
                return index
    return None


def line_number(source: str, index: int) -> int:
    return source.count("\n", 0, index) + 1


def normalized_projected_ranges(
    source: str,
    ranges: list[tuple[int, int]],
    start: int = 0,
    end: int | None = None,
) -> list[tuple[int, int]]:
    """Extend erased test constructs through their otherwise inert line trivia."""
    end = len(source) if end is None else end
    normalized = []
    cursor = start
    for first, last in sorted(ranges):
        first = max(first, start)
        last = min(last, end - 1)
        if first > last or last < cursor:
            continue
        first = max(first, cursor)
        line_start = source.rfind("\n", start, first) + 1
        if not source[line_start:first].strip():
            first = line_start
            line_end = source.find("\n", last + 1, end)
            trailing_end = end if line_end < 0 else line_end
            if not source[last + 1 : trailing_end].strip():
                last = end - 1 if line_end < 0 else line_end
        else:
            while last >= first and source[last] in " \t":
                last -= 1
            while first > cursor and source[first - 1] in " \t":
                first -= 1
        normalized.append((first, last))
        cursor = last + 1
    return normalized


def source_line_indexes(
    source: str, projected_ranges: list[tuple[int, int]]
) -> tuple[list[int], list[int]]:
    """Index original and projection-removed newlines for logarithmic lookup."""
    newlines = [index for index, char in enumerate(source) if char == "\n"]
    removed_newlines = []
    for first, last in normalized_projected_ranges(source, projected_ranges):
        start = bisect.bisect_left(newlines, first)
        end = bisect.bisect_right(newlines, last)
        removed_newlines.extend(newlines[start:end])
    return newlines, removed_newlines


def indexed_line_number(
    newlines: list[int], removed_newlines: list[int], index: int
) -> tuple[int, int]:
    original = bisect.bisect_left(newlines, index) + 1
    projected = original - bisect.bisect_left(removed_newlines, index)
    return original, projected


def source_contexts(
    masked: str, pattern: re.Pattern[str]
) -> list[tuple[int, int, str]]:
    contexts = []
    for match in pattern.finditer(masked):
        opening = masked.find("{", match.start(), match.end() + 1)
        if opening == -1:
            continue
        closing = matching_brace(masked, opening)
        if closing is not None:
            contexts.append((opening, closing, normalize_space(match.group(1))))
    return contexts


def qualify_contexts(
    contexts: list[tuple[int, int, str]], separator: str = "::"
) -> list[tuple[int, int, str]]:
    """Include lexical parents so sibling namespace/module items stay distinct."""
    qualified = []
    for opening, closing, context in contexts:
        parents = [
            (start, name)
            for start, end, name in contexts
            if start < opening < closing < end
        ]
        parents.sort()
        qualified.append(
            (
                opening,
                closing,
                separator.join([*(name for _, name in parents), context]),
            )
        )
    return qualified


def innermost_context(
    contexts: list[tuple[int, int, str]], position: int, fallback: str
) -> str:
    matches = [
        (end - start, context)
        for start, end, context in contexts
        if start < position < end
    ]
    return min(matches)[1] if matches else fallback


def qualify_cpp_name(context: str, name: str) -> str:
    if not context or name.startswith(f"{context}::"):
        return name
    leaf = context.rsplit("::", 1)[-1]
    if name.startswith(f"{leaf}::"):
        parent = context.rsplit("::", 1)[0] if "::" in context else ""
        return f"{parent}::{name}" if parent else name
    return f"{context}::{name}"


def behavior_kinds(name: str, signature: str, body: str) -> list[str]:
    haystack = f"{name} {signature} {body}"
    lower = haystack.lower()
    kinds: set[str] = set()
    if name.lower().startswith(("set", "update")) or re.search(
        r"\bm_[A-Za-z0-9_]+\s*=", body
    ):
        kinds.add("setter")
    if any(
        word in lower
        for word in ("callback", "notify", "listener", "onchange", "changed")
    ):
        kinds.add("callback")
    if " override" in f" {signature}" or "virtual " in signature:
        kinds.add("virtual-override")
    if re.search(r"\bif\s*\([^)]*(?:==|!=)[^)]*\)\s*(?:\{|return)", body, re.S):
        kinds.add("mutation-guard")
    if re.search(
        r"(?:addDirt|publishDirt|Dirt::|mark_[a-z_]*dirty|publish_dirt|mark[A-Za-z]*Changed|invalidate)",
        body,
    ):
        kinds.add("dirt-publication")
    if re.search(
        r"depend(?:ent|ency)|addDependent|removeDependent|add_dependency",
        haystack,
        re.I,
    ):
        kinds.add("dependency-operation")
    if re.search(
        r"clone|reinit|teardown|dispose|destroy|onAdded|onRemoved|reset|drop",
        name,
        re.I,
    ):
        kinds.add("lifecycle")
    if re.search(
        r"unique_ptr|shared_ptr|make_unique|make_shared|std::move|\bRc\b|\bArc\b|\bBox\b|\.clone\(\)",
        body,
    ):
        kinds.add("ownership")
    if re.search(
        r"\b(for|while|loop)\b|\.sort(?:_by)?\(|lower_bound|upper_bound", body
    ):
        kinds.add("ordering-loop")
    if re.search(
        r"static_cast|as\s+u(?:8|16|32|64|size)|clamp|round|floor|ceil|trunc|wrapping_|saturating_",
        body,
    ):
        kinds.add("scalar-edge")
    return sorted(kinds or {"behavior"})


def cpp_signature_before(masked: str, opening: int) -> tuple[str, str, int] | None:
    window_start = max(0, opening - 1600)
    window = masked[window_start:opening]
    delimiter = -1
    parentheses = 0
    brackets = 0
    for index, char in enumerate(window):
        if char == "(":
            parentheses += 1
        elif char == ")" and parentheses:
            parentheses -= 1
        elif char == "[":
            brackets += 1
        elif char == "]" and brackets:
            brackets -= 1
        elif char in ";{}" and parentheses == 0 and brackets == 0:
            # Braced default arguments and braced constructor arguments are
            # part of the declaration, not declaration boundaries.
            delimiter = index
    fragment_start = window_start + delimiter + 1
    while fragment_start < opening and masked[fragment_start].isspace():
        fragment_start += 1
    signature_fragment = masked[fragment_start:opening]
    signature_lines = []
    preprocessor_continuation = False
    for line in signature_fragment.splitlines():
        if preprocessor_continuation:
            preprocessor_continuation = line.rstrip().endswith("\\")
            continue
        if line.lstrip().startswith("#"):
            preprocessor_continuation = line.rstrip().endswith("\\")
            continue
        if line.strip():
            signature_lines.append(line)
    signature = normalize_space("\n".join(signature_lines))
    while True:
        # Strip balanced leading annotation invocations independent of line
        # breaks. Annotation arguments may themselves contain braced values or
        # lambdas. Retain a sole all-uppercase constructor such as `AABB()`.
        annotation = re.match(r"[A-Z_][A-Z0-9_]*\s*\(", signature)
        if annotation is None:
            break
        stack = [")"]
        cursor = annotation.end()
        pairs = {"(": ")", "[": "]", "{": "}"}
        while cursor < len(signature) and stack:
            char = signature[cursor]
            if char in pairs:
                stack.append(pairs[char])
            elif char == stack[-1]:
                stack.pop()
            cursor += 1
        if stack:
            break
        remainder = signature[cursor:].lstrip()
        if remainder.startswith(
            (
                ":",
                "const ",
                "noexcept",
                "override",
                "final",
                "requires ",
                "->",
                "&",
                "=",
            )
        ) or not re.search(r"[~A-Za-z_][A-Za-z0-9_:~]*\s*\(", remainder):
            break
        signature = remainder
    if not signature or "(" not in signature or ")" not in signature:
        return None
    if signature.startswith(("#", "using ", "typedef ")):
        return None
    operator_match = re.search(
        r"\boperator\s*(\[\]|\(\)|(?:new|delete)(?:\[\])?|[^\w\s(]+|[A-Za-z_][A-Za-z0-9_:]*(?:\s*<[^(){};]+>)?(?:\s*[*&])?)\s*\(",
        signature,
    )
    if operator_match:
        operator_target = normalize_space(operator_match.group(1))
        separator = " " if re.match(r"[A-Za-z_]", operator_target) else ""
        name = "operator" + separator + operator_target
    else:
        prefix = signature[: signature.find("(")].strip()
        if not prefix:
            return None
        if "operator[]" not in signature and re.search(r"\[[^]]*\]\s*$", prefix):
            return None
        head = prefix.split()[-1]
        if head in CPP_CONTROL_HEADS or head.endswith(("=", "]")):
            return None
        name_match = re.search(r"([~A-Za-z_][A-Za-z0-9_:~]*)\s*$", prefix)
        if not name_match:
            return None
        name = name_match.group(1)
    return signature, name, fragment_start


def cpp_is_lambda_body(masked: str, opening: int) -> bool:
    """Return whether an opening brace starts a lambda, not a member body."""
    window = masked[max(0, opening - 1600) : opening]
    if re.search(r"\boperator\s*\[\]\s*\([^{};]*\)\s*$", window):
        return False
    match = re.search(
        r"\[[^]]*\]\s*(?:\([^{};]*\))?"
        r"(?:\s*(?:mutable|consteval|constexpr|noexcept(?:\([^)]*\))?))*"
        r"(?:\s*->\s*[^{};]+)?\s*$",
        window,
    )
    if match is None:
        return False
    prefix = window[: match.start()].rstrip()
    return (
        not prefix
        or prefix[-1] in "=([{,;:"
        or bool(re.search(r"\b(?:return|co_return)\s*$", prefix))
    )


def cpp_logical_directives(
    source: str, *, preserve_literals: bool = False
) -> list[tuple[int, int, int, int, str]]:
    """Return comment-masked logical preprocessing directives with source spans."""
    spliced, positions = cpp_spliced_text(source)
    masked = mask_noncode(spliced, preserve_literals=preserve_literals)
    discovery_masked = mask_noncode(spliced)
    lexical_view = mask_noncode(spliced, preserve_literals=True)
    directives = []
    line_start = 0
    while line_start < len(spliced):
        line_end = spliced.find("\n", line_start)
        line_end = len(spliced) if line_end == -1 else line_end + 1
        if not discovery_masked[line_start:line_end].lstrip().startswith("#"):
            line_start = line_end
            continue
        cursor = line_start
        quote = ""
        raw_terminator = ""
        while cursor < len(spliced):
            if raw_terminator:
                if spliced.startswith(raw_terminator, cursor):
                    cursor += len(raw_terminator)
                    raw_terminator = ""
                else:
                    cursor += 1
                continue
            char = spliced[cursor]
            if quote:
                if char == "\\" and cursor + 1 < len(spliced):
                    cursor += 2
                    continue
                cursor += 1
                if char == quote:
                    quote = ""
                continue
            if lexical_view[cursor] == char and char in {'"', "'"}:
                quote = char
                cursor += 1
                continue
            if char == "R" and lexical_view[cursor] == "R":
                match = re.match(r'R"([^ ()\\\t\r\n]{0,16})\(', spliced[cursor:])
                if match:
                    raw_terminator = ")" + match.group(1) + '"'
                    cursor += len(match.group(0))
                    continue
            cursor += 1
            if char == "\n":
                break
        logical_end = cursor
        physical_start = positions[line_start] if positions else 0
        physical_end = (
            positions[logical_end - 1] + 1 if logical_end and positions else len(source)
        )
        directives.append(
            (
                line_number(source, physical_start),
                line_number(source, max(physical_start, physical_end - 1)),
                physical_start,
                physical_end,
                masked[line_start:logical_end],
            )
        )
        line_start = logical_end
    return directives


def cpp_condition_context(
    source: str,
    position: int,
    directives: list[tuple[int, int, int, int, str]] | None = None,
) -> str:
    stack: list[list[str]] = []
    if directives is None:
        directives = cpp_logical_directives(source, preserve_literals=True)
    for _, _, start, _, logical in directives:
        if start >= position:
            break
        stripped = logical.strip()
        if re.match(r"#\s*(?:if|ifdef|ifndef)\b", stripped):
            stack.append([normalize_cpp_condition(stripped)])
        elif re.match(r"#\s*elif\b", stripped) and stack:
            stack[-1].append(normalize_cpp_condition(stripped))
        elif re.match(r"#\s*else\b", stripped) and stack:
            stack[-1].append("#else")
        elif re.match(r"#\s*endif\b", stripped) and stack:
            stack.pop()
    return " && ".join(" -> ".join(branches) for branches in stack)


def normalize_cpp_condition(value: str) -> str:
    """Collapse C++ condition trivia without altering literal token contents."""
    chars = []
    index = 0
    pending_space = False
    while index < len(value):
        if value[index].isspace():
            pending_space = True
            index += 1
            continue
        if pending_space and chars:
            chars.append(" ")
        pending_space = False
        if value[index] in {'"', "'"}:
            quote = value[index]
            chars.append(quote)
            index += 1
            while index < len(value):
                char = value[index]
                chars.append(char)
                index += 1
                if char == "\\" and index < len(value):
                    chars.append(value[index])
                    index += 1
                elif char == quote:
                    break
            continue
        if value[index] == "R":
            match = re.match(r'R"([^ ()\\\t\r\n]{0,16})\(', value[index:])
            if match:
                terminator = ")" + match.group(1) + '"'
                end = value.find(terminator, index + len(match.group(0)))
                end = len(value) if end == -1 else end + len(terminator)
                chars.extend(value[index:end])
                index = end
                continue
        chars.append(value[index])
        index += 1
    return "".join(chars)


def _cpp_members_spliced(path: str, source: str) -> list[dict[str, Any]]:
    masked = mask_cpp_source(source)
    condition_directives = cpp_logical_directives(source, preserve_literals=True)
    contexts = cpp_source_contexts(masked)
    scope_openings = {opening for opening, _, _ in contexts}
    nested_delimiter_openings = set()
    parentheses = 0
    brackets = 0
    for index, char in enumerate(masked):
        if char == "(":
            parentheses += 1
        elif char == ")" and parentheses:
            parentheses -= 1
        elif char == "[":
            brackets += 1
        elif char == "]" and brackets:
            brackets -= 1
        elif char == "{" and (parentheses or brackets):
            # Braced defaults, initializer arguments, and lambda bodies inside
            # a parameter list are declaration syntax, never member bodies.
            nested_delimiter_openings.add(index)
    candidates: list[tuple[int, int, int, str, str]] = []
    consumed_openings: set[int] = set()
    covered_bodies: list[tuple[int, int]] = []
    covered_initializers: list[tuple[int, int]] = []
    for opening, char in enumerate(masked):
        if (
            char != "{"
            or opening in scope_openings
            or opening in nested_delimiter_openings
            or cpp_is_lambda_body(masked, opening)
            or opening in consumed_openings
            or any(start < opening < end for start, end in covered_bodies)
            or any(start < opening <= end for start, end in covered_initializers)
        ):
            continue
        parsed = cpp_signature_before(masked, opening)
        if parsed is None:
            continue
        closing = matching_brace(masked, opening)
        if closing is None:
            continue
        signature, name, signature_start = parsed
        content_start = opening
        constructor_opening = cpp_constructor_body_opening(masked, opening, signature)
        if constructor_opening is not None:
            # Constructor initializer expressions are behavior: they establish
            # ownership and lifecycle state before the body executes. Keep the
            # stable identity initializer-independent, but bind the complete
            # declaration and initializer list into the member content proof.
            content_start = signature_start
            consumed_openings.add(opening)
            covered_initializers.append((opening, constructor_opening))
            opening = constructor_opening
            signature = normalize_space(
                re.split(r"\)\s*:", signature, maxsplit=1)[0] + ")"
            )
            closing = matching_brace(masked, opening)
            if closing is None:
                continue
        elif re.search(r"\)\s*:\s*", signature):
            # An initializer whose braces are nested inside parentheses was
            # skipped as declaration syntax, so this candidate is already the
            # real constructor body. Bind the initializer list into content.
            content_start = signature_start
            signature = normalize_space(
                re.split(r"\)\s*:", signature, maxsplit=1)[0] + ")"
            )
        context = innermost_context(contexts, opening, "")
        name = qualify_cpp_name(context, name)
        candidates.append((opening, content_start, closing, signature, name))
        covered_bodies.append((opening, closing))
    records = []
    for opening, content_start, closing, signature, name in candidates:
        body = source[content_start : closing + 1]
        condition = cpp_condition_context(source, opening, condition_directives)
        stable_signature = signature + (f" [{condition}]" if condition else "")
        start = line_number(source, content_start)
        end = line_number(source, closing)
        records.append(
            {
                "id": f"cpp:{path}:{name}@{sha256_text(stable_signature)[:16]}",
                "path": path,
                "name": name,
                "owner_family": owner_family(path),
                "start_line": start,
                "end_line": end,
                "_logical_start": content_start,
                "_logical_end": closing,
                "signature_sha256": sha256_text(signature),
                "content_sha256": sha256_text(source[content_start : closing + 1]),
                "behavior_kinds": behavior_kinds(name, signature, body),
            }
        )
    return records


def cpp_members(path: str, source: str) -> list[dict[str, Any]]:
    """Inventory members from the phase-2 C++ token stream with physical spans."""
    spliced, positions = cpp_spliced_text(source)
    records = _cpp_members_spliced(path, spliced)
    for record in records:
        logical_start = record.pop("_logical_start")
        logical_end = record.pop("_logical_end")
        if positions:
            record["start_line"] = line_number(
                source, positions[min(logical_start, len(positions) - 1)]
            )
            record["end_line"] = line_number(
                source, positions[min(logical_end, len(positions) - 1)]
            )
    return records


def cpp_file_classification(
    path: str, members: list[dict[str, Any]], macros: list[dict[str, Any]] | None = None
) -> str:
    if "/generated/" in f"/{path}":
        return "generated"
    if pathlib.PurePosixPath(path).suffix in {".cpp", ".mm"}:
        return "implementation"
    return "behavioral-header" if members or macros else "declaration-only"


def cpp_behavioral_macros(source: str) -> list[dict[str, Any]]:
    """Record behavior-bearing logical macro definitions without fake members."""
    records = []
    condition_directives = cpp_logical_directives(source, preserve_literals=True)
    for start_line, end_line, start, end, logical_masked in cpp_logical_directives(
        source
    ):
        definition = re.match(
            r"\s*#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)" r"(\([^\n]*?\))?[ \t]*(.*)",
            logical_masked,
            re.S,
        )
        if definition is not None:
            function_like = definition.group(2) is not None
            replacement = normalize_space(definition.group(3))
            # Every non-empty function-like macro can affect behavior: calls,
            # comparisons, mutation, allocation, and scalar expressions are
            # just as significant as control-flow expansions. For object-like
            # macros, retain the explicit behavioral spellings.
            object_behavior = bool(
                re.search(
                    r"\b(?:" + "|".join(sorted(CPP_BEHAVIORAL_MACRO_CONTROL)) + r")\b"
                    r"|\+\+|--|[{};=]|[A-Za-z_][A-Za-z0-9_:]*\s*\(",
                    replacement,
                )
            )
            if replacement and (function_like or object_behavior):
                condition = cpp_condition_context(source, start, condition_directives)
                stable = definition.group(1) + (f" [{condition}]" if condition else "")
                records.append(
                    {
                        "id": f"macro:{definition.group(1)}@{sha256_text(stable)[:16]}",
                        "name": definition.group(1),
                        "start_line": start_line,
                        "end_line": end_line,
                        "content_sha256": sha256_text(source[start:end]),
                    }
                )
    return records


def cpp_virtual_declarations(source: str) -> set[str]:
    spliced, _ = cpp_spliced_text(source)
    masked = mask_cpp_source(spliced)
    contexts = source_contexts(
        masked,
        re.compile(
            r"(?m)^[ \t]*(?:(?:template\s*<[^;{]+>\s*)?)(?:class|struct)\s+([A-Za-z_][A-Za-z0-9_]*(?:\s*<[^;{]+>)?)[^;{]*\{"
        ),
    )
    declarations = set()
    pattern = re.compile(
        r"([~A-Za-z_][A-Za-z0-9_]*)\s*\([^;{}]*\)[^;{}]*\boverride\s*;"
    )
    for match in pattern.finditer(masked):
        context = innermost_context(contexts, match.start(), "")
        if context:
            declarations.add(f"{context}::{match.group(1)}")
    return declarations


def cpp_virtual_key(name: str) -> str:
    parts = name.split("::")
    return "::".join(parts[-2:]) if len(parts) >= 2 else name


def cpp_constructor_body_opening(
    masked: str, initializer_opening: int, signature: str
) -> int | None:
    if not re.search(r"\)\s*:\s*[^{}]*$", signature):
        return None
    window_start = max(0, initializer_opening - 1600)
    window = masked[window_start:initializer_opening]
    colon_match = list(re.finditer(r"\)\s*:\s*", window))
    if not colon_match:
        return None
    cursor = window_start + colon_match[-1].end()
    while cursor < len(masked):
        while cursor < len(masked) and masked[cursor].isspace():
            cursor += 1
        while cursor < len(masked) and masked[cursor] not in "({":
            cursor += 1
        if cursor >= len(masked):
            return None
        delimiter = masked[cursor]
        if delimiter == "(":
            depth = 1
            cursor += 1
            while cursor < len(masked) and depth:
                depth += masked[cursor] == "("
                depth -= masked[cursor] == ")"
                cursor += 1
        else:
            closing = matching_brace(masked, cursor)
            if closing is None:
                return None
            cursor = closing + 1
        while cursor < len(masked) and masked[cursor].isspace():
            cursor += 1
        if cursor < len(masked) and masked[cursor] == ",":
            cursor += 1
            continue
        return cursor if cursor < len(masked) and masked[cursor] == "{" else None
    return None


RUST_FN_HEAD = re.compile(
    r"(?<![A-Za-z0-9_])(?:#\[[^\n]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?"
    r"(?:(?:async|const|unsafe)\s+)*(?:extern\s+)?"
    r"fn\s+(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\s*"
)
RUST_CONTEXT = re.compile(
    r"(?m)^[ \t]*((?:#\s*\[[^]]+\]\s*)*(?:(?:unsafe\s+)?(?:impl|trait)\b[^;{]+|(?:pub(?:\([^)]*\))?\s+)?mod\s+(?:r#)?[A-Za-z_][A-Za-z0-9_]*\s*))\{"
)


class RustFunctionMatch:
    def __init__(self, start: int, end: int, name: str) -> None:
        self._start = start
        self._end = end
        self._name = name

    def start(self) -> int:
        return self._start

    def end(self) -> int:
        return self._end

    def group(self, index: int) -> str:
        if index != 1:
            raise IndexError(index)
        return self._name


def rust_function_matches(masked: str) -> list[RustFunctionMatch]:
    """Discover Rust function heads with balanced generic parameter syntax."""
    matches = []
    opening_to_closing = {"<": ">", "(": ")", "[": "]", "{": "}"}
    for head in RUST_FN_HEAD.finditer(masked):
        cursor = head.end()
        if cursor < len(masked) and masked[cursor] == "<":
            stack = [">"]
            cursor += 1
            while cursor < len(masked) and stack:
                char = masked[cursor]
                previous = masked[cursor - 1] if cursor else ""
                if char in "([{":
                    stack.append(opening_to_closing[char])
                elif char == "<" and "}" not in stack:
                    stack.append(">")
                elif char == ">" and previous == "-":
                    pass
                elif char == stack[-1]:
                    stack.pop()
                cursor += 1
            if stack:
                continue
            while cursor < len(masked) and masked[cursor].isspace():
                cursor += 1
        if cursor >= len(masked) or masked[cursor] != "(":
            continue
        matches.append(RustFunctionMatch(head.start(), cursor + 1, head.group(1)))
    return matches


def cpp_source_contexts(masked: str) -> list[tuple[int, int, str]]:
    contexts = source_contexts(
        masked,
        re.compile(
            r"(?m)^[ \t]*(?:(?:template\s*<[^;{]*>\s*)?)(?:class|struct|union)\s+(?:alignas\s*\([^)]*\)\s*)?([A-Za-z_][A-Za-z0-9_]*(?:\s*<(?:[^<>{};]|<[^<>{};]*>)*>)?)\s*(?:final\s*)?(?:\{|[^;{]*:[^;{]*\{)"
        ),
    )
    contexts.extend(
        source_contexts(
            masked,
            re.compile(
                r"(?m)^[ \t]*(?:inline\s+)?namespace\s+([A-Za-z_][A-Za-z0-9_:]*)\s*\{"
            ),
        )
    )
    return qualify_contexts(contexts)


def rust_identity_text(source: str, masked: str, start: int, end: int) -> str:
    """Normalize code while retaining cfg literals that distinguish variants."""
    chars = list(masked[start:end])
    cfg_start = re.compile(r"#\s*\[\s*cfg(?:_attr)?\s*\(")
    for match in cfg_start.finditer(masked, start, end):
        opening = masked.find("[", match.start(), match.end())
        depth = 1
        cursor = opening + 1
        while cursor < end and depth:
            depth += masked[cursor] == "["
            depth -= masked[cursor] == "]"
            cursor += 1
        if depth == 0:
            local_start = match.start() - start
            local_end = cursor - start
            chars[local_start:local_end] = source[match.start() : cursor]
    for match in re.finditer(r"\bextern\b", masked[start:end]):
        literal_start = match.end()
        while literal_start < len(chars):
            absolute = start + literal_start
            if source[absolute].isspace():
                literal_start += 1
                continue
            if source.startswith("/*", absolute):
                depth = 1
                comment_end = absolute + 2
                while comment_end < end and depth:
                    if source.startswith("/*", comment_end):
                        depth += 1
                        comment_end += 2
                    elif source.startswith("*/", comment_end):
                        depth -= 1
                        comment_end += 2
                    else:
                        comment_end += 1
                if depth:
                    break
                literal_start = comment_end - start
                continue
            if source.startswith("//", absolute):
                comment_end = source.find("\n", absolute + 2, end)
                if comment_end == -1:
                    break
                literal_start = comment_end + 1 - start
                continue
            break
        abi_source = source[start + literal_start : end]
        literal_length = rust_string_token_length(abi_source)
        if literal_length is not None:
            literal_end = literal_start + literal_length
            chars[literal_start:literal_end] = source[
                start + literal_start : start + literal_end
            ]
    return normalize_space("".join(chars))


def rust_context_identity(source: str, masked: str, start: int, end: int) -> str:
    """Build context identity while normalizing only a raw module-name token."""
    context_source = source[start:end]
    context_masked = masked[start:end]
    raw_name = re.search(r"\bmod\s+(r#)(?=[A-Za-z_])", context_masked)
    if raw_name is not None:
        prefix_start, prefix_end = raw_name.span(1)
        context_source = context_source[:prefix_start] + context_source[prefix_end:]
        context_masked = mask_noncode(context_source, nested_block_comments=True)
    return rust_identity_text(context_source, context_masked, 0, len(context_source))


def rust_string_token_length(source: str) -> int | None:
    """Return the byte length of one ordinary or raw Rust string token."""
    if source.startswith('"'):
        cursor = 1
        while cursor < len(source):
            if source[cursor] == "\\":
                cursor += 2
                continue
            if source[cursor] == '"':
                return cursor + 1
            cursor += 1
        return None
    raw_opening = re.match(r'r(?P<hashes>#{0,255})"', source)
    if raw_opening is None:
        return None
    terminator = '"' + raw_opening.group("hashes")
    closing = source.find(terminator, raw_opening.end())
    return closing + len(terminator) if closing != -1 else None


def rust_literal_token_length(source: str) -> int | None:
    """Return one Rust literal token length, including an attached suffix."""
    length = rust_string_token_length(source)
    if length is None and source[:1] in {"b", "c"}:
        suffix_length = rust_string_token_length(source[1:])
        if suffix_length is not None:
            length = suffix_length + 1
    if length is None:
        match = re.match(
            r"(?:b)?'(?:\\.|[^'\\\n])+'|"
            r"(?:0x[0-9A-Fa-f](?:_?[0-9A-Fa-f]|_)*|"
            r"0o[0-7](?:_?[0-7]|_)*|0b[01](?:_?[01]|_)*|"
            r"[0-9](?:_?[0-9]|_)*(?:\.(?!\.)(?:[0-9](?:_?[0-9])*)?)?"
            r"(?:[eE][+-]?[0-9](?:_?[0-9])*)?)",
            source,
        )
        length = match.end() if match is not None else None
    if length is None:
        return None
    suffix = next(
        (
            token
            for token in rust_identifier_tokens(source[length:])
            if token[0] == 0 and not source[length:].startswith("r#")
        ),
        None,
    )
    return length + suffix[1] if suffix is not None else length


def rust_string_value(token: str) -> str:
    """Decode one validated ordinary or raw Rust string token."""
    raw_opening = re.match(r'r(?P<hashes>#{0,255})"', token)
    if raw_opening is not None:
        terminator = '"' + raw_opening.group("hashes")
        if not token.endswith(terminator):
            raise ValueError("unterminated raw Rust string")
        return token[raw_opening.end() : -len(terminator)]
    if len(token) < 2 or token[0] != '"' or token[-1] != '"':
        raise ValueError("invalid ordinary Rust string")
    value: list[str] = []
    cursor = 1
    simple = {
        "n": "\n",
        "r": "\r",
        "t": "\t",
        "0": "\0",
        "\\": "\\",
        '"': '"',
        "'": "'",
    }
    while cursor < len(token) - 1:
        char = token[cursor]
        if char != "\\":
            value.append(char)
            cursor += 1
            continue
        cursor += 1
        if cursor >= len(token) - 1:
            raise ValueError("unterminated Rust string escape")
        escape = token[cursor]
        if escape in simple:
            value.append(simple[escape])
            cursor += 1
        elif escape == "x":
            digits = token[cursor + 1 : cursor + 3]
            if len(digits) != 2 or not re.fullmatch(r"[0-9A-Fa-f]{2}", digits):
                raise ValueError("invalid Rust hex escape")
            value.append(chr(int(digits, 16)))
            cursor += 3
        elif escape == "u" and token.startswith("u{", cursor):
            closing = token.find("}", cursor + 2)
            digits = token[cursor + 2 : closing] if closing != -1 else ""
            if not digits or not re.fullmatch(r"[0-9A-Fa-f_]+", digits):
                raise ValueError("invalid Rust unicode escape")
            value.append(chr(int(digits.replace("_", ""), 16)))
            cursor = closing + 1
        elif escape == "\n":
            cursor += 1
            while cursor < len(token) - 1 and token[cursor] in " \t\r\n":
                cursor += 1
        else:
            raise ValueError(f"unsupported Rust string escape: \\{escape}")
    return "".join(value)


def rust_projected_identity(
    source: str,
    ranges: list[tuple[int, int]],
    start: int,
    end: int,
) -> str:
    projected = project_source_ranges(source, ranges, start, end)
    projected_masked = mask_noncode(projected, nested_block_comments=True)
    raw_name = re.search(r"\bfn\s+(r#)(?=[A-Za-z_])", projected_masked)
    if raw_name is not None:
        prefix_start, prefix_end = raw_name.span(1)
        projected = projected[:prefix_start] + projected[prefix_end:]
        projected_masked = mask_noncode(projected, nested_block_comments=True)
    return rust_identity_text(projected, projected_masked, 0, len(projected))


def rust_source_contexts(
    masked: str, source: str, test_ranges: list[tuple[int, int]]
) -> list[tuple[int, int, str]]:
    contexts = []
    for match in RUST_CONTEXT.finditer(masked):
        opening = masked.find("{", match.start(), match.end() + 1)
        closing = matching_brace(masked, opening)
        if opening != -1 and closing is not None:
            contexts.append(
                (
                    opening,
                    closing,
                    rust_context_identity(source, masked, match.start(1), match.end(1)),
                )
            )
    for match in rust_function_matches(masked):
        cursor = match.end()
        depth = 1
        while cursor < len(masked) and depth:
            depth += masked[cursor] == "("
            depth -= masked[cursor] == ")"
            cursor += 1
        opening = rust_body_opening(masked, cursor) if depth == 0 else None
        closing = matching_brace(masked, opening) if opening is not None else None
        if opening is None or closing is None:
            continue
        item_start = rust_leading_attribute_start(masked, match.start())
        signature = rust_projected_identity(source, test_ranges, item_start, opening)
        contexts.append(
            (
                opening,
                closing,
                f"fn {match.group(1)}@{sha256_text(signature)[:16]}",
            )
        )
    return qualify_contexts(contexts)


def rust_leading_attribute_start(masked: str, item_start: int) -> int:
    """Include contiguous attributes even when their bodies span lines."""
    cursor = item_start
    while True:
        end = cursor
        while end > 0 and masked[end - 1].isspace():
            end -= 1
        if end == 0 or masked[end - 1] != "]":
            return cursor
        depth = 1
        opening = end - 2
        while opening >= 0 and depth:
            depth += masked[opening] == "]"
            depth -= masked[opening] == "["
            opening -= 1
        opening += 1
        hash_position = opening - 1
        if depth or hash_position < 0 or masked[hash_position] != "#":
            return cursor
        cursor = hash_position


def rust_body_opening(masked: str, cursor: int) -> int | None:
    """Find a Rust function body without mistaking type punctuation for `;`."""
    paren_depth = 0
    bracket_depth = 0
    angle_depth = 0
    while cursor < len(masked):
        char = masked[cursor]
        previous = masked[cursor - 1] if cursor else ""
        if char == "(":
            paren_depth += 1
        elif char == ")" and paren_depth:
            paren_depth -= 1
        elif char == "[":
            bracket_depth += 1
        elif char == "]" and bracket_depth:
            bracket_depth -= 1
        elif char == "<" and previous != "-":
            angle_depth += 1
        elif char == ">" and angle_depth:
            angle_depth -= 1
        elif char == "{" and (paren_depth or bracket_depth or angle_depth):
            closing = matching_brace(masked, cursor)
            if closing is None:
                return None
            cursor = closing + 1
            continue
        elif not (paren_depth or bracket_depth or angle_depth):
            if char == ";":
                return None
            if char == "{":
                return cursor
        cursor += 1
    return None


def generated_line_ranges(path: str, source: str) -> list[tuple[int, int]]:
    lines = source.splitlines()
    if path in GENERATED_RUST_FILES:
        return [(1, max(1, len(lines)))]
    comment_lines = mask_noncode(
        source,
        nested_block_comments=True,
        preserve_line_comments=True,
    ).splitlines()
    ranges = []
    start = None
    for number, line in enumerate(comment_lines, 1):
        marker = re.fullmatch(
            r"\s*//\s*@generated-region\s+(begin|end)(?:\s+[^\s].*)?\s*", line
        )
        if marker is None:
            continue
        if marker.group(1) == "begin":
            if start is not None:
                raise ValueError(f"nested generated region in {path}:{number}")
            start = number
        else:
            if start is None:
                raise ValueError(f"unmatched generated region end in {path}:{number}")
            ranges.append((start, number))
            start = None
    if start is not None:
        raise ValueError(f"unterminated generated region in {path}:{start}")
    return ranges


def rust_test_ranges(masked: str, source: str) -> list[tuple[int, int]]:
    ranges = []
    function_signatures = []
    for function in rust_function_matches(masked):
        cursor = function.end()
        depth = 1
        while cursor < len(masked) and depth:
            depth += masked[cursor] == "("
            depth -= masked[cursor] == ")"
            cursor += 1
        body = rust_body_opening(masked, cursor) if depth == 0 else None
        if body is not None:
            function_signatures.append((function.start(), body))
    attributes_pattern = re.compile(r"(?ms)(?:#\s*\[[^]]+\]\s*)+")
    for match in attributes_pattern.finditer(masked):
        attributes = source[match.start() : match.end()]
        if not attributes_require_test(attributes):
            continue
        signature_context = next(
            (
                (function_start, body)
                for function_start, body in function_signatures
                if function_start < match.start() < body
            ),
            None,
        )
        signature_boundary = signature_context[1] if signature_context else None
        signature_closer = (
            rust_enclosing_signature_closer(masked, signature_context[0], match.start())
            if signature_context
            else None
        )
        if signature_closer is None:
            signature_closer = rust_enclosing_generic_closer(masked, match.start())
        if signature_closer is None:
            signature_closer = rust_enclosing_closure_closer(masked, match.start())
        closing = rust_attributed_construct_end(
            masked, match.end(), signature_boundary, signature_closer
        )
        if closing is not None:
            ranges.append((match.start(), closing))
    return ranges


def rust_enclosing_signature_closer(
    masked: str, signature_start: int, attribute_start: int
) -> str | None:
    """Return the delimiter enclosing an attribute within a function signature."""
    stack: list[str] = []
    cursor = signature_start
    while cursor < attribute_start:
        char = masked[cursor]
        previous = masked[cursor - 1] if cursor else ""
        if char == "(":
            stack.append(")")
        elif char == "[":
            stack.append("]")
        elif char == "<" and (previous.isalnum() or previous in "_:>"):
            stack.append(">")
        elif stack and char == stack[-1] and not (char == ">" and previous == "-"):
            stack.pop()
        cursor += 1
    return stack[-1] if stack and stack[-1] in {">", ")"} else None


def rust_enclosing_generic_closer(masked: str, attribute_start: int) -> str | None:
    """Detect an attribute within a generic parameter list on any declaration."""
    angle_depth = 0
    delimiter_depth = {"}": 0, ")": 0, "]": 0}
    opening_to_closing = {"{": "}", "(": ")", "[": "]"}
    cursor = attribute_start - 1
    opening = None
    while cursor >= 0:
        char = masked[cursor]
        if char in delimiter_depth:
            delimiter_depth[char] += 1
        elif char in opening_to_closing:
            closing = opening_to_closing[char]
            if delimiter_depth[closing]:
                delimiter_depth[closing] -= 1
            elif char == "{":
                break
        elif char == ";" and not any(delimiter_depth.values()):
            break
        elif (
            char == ">"
            and masked[cursor - 1 : cursor] != "-"
            and not any(delimiter_depth.values())
        ):
            angle_depth += 1
        elif char == "<":
            if any(delimiter_depth.values()):
                cursor -= 1
                continue
            if angle_depth:
                angle_depth -= 1
            else:
                opening = cursor
                break
        cursor -= 1
    if opening is None:
        return None
    stack = [">"]
    opening_to_closing = {"{": "}", "(": ")", "[": "]"}
    cursor = opening + 1
    while cursor < len(masked):
        char = masked[cursor]
        previous = masked[cursor - 1] if cursor else ""
        if char in opening_to_closing:
            stack.append(opening_to_closing[char])
        elif char == "<" and "}" not in stack:
            stack.append(">")
        elif char == ">" and previous == "-":
            pass
        elif stack and char == stack[-1]:
            stack.pop()
            if not stack:
                return ">" if cursor > attribute_start else None
        elif char == ";" and stack == [">"]:
            return None
        cursor += 1
    return None


def rust_enclosing_closure_closer(masked: str, attribute_start: int) -> str | None:
    """Detect an attribute within a closure's pipe-delimited parameter list."""
    delimiter_depth = {"}": 0, ")": 0, "]": 0}
    opening_to_closing = {"{": "}", "(": ")", "[": "]"}
    opening = attribute_start - 1
    while opening >= 0:
        char = masked[opening]
        if char in delimiter_depth:
            delimiter_depth[char] += 1
        elif char in opening_to_closing:
            closing_delimiter = opening_to_closing[char]
            if delimiter_depth[closing_delimiter]:
                delimiter_depth[closing_delimiter] -= 1
            elif char == "{":
                return None
        elif not any(delimiter_depth.values()):
            if char == "|":
                break
            if char == ";":
                return None
        opening -= 1
    if opening < 0 or masked[opening] != "|":
        return None
    delimiter_stack: list[str] = []
    cursor = attribute_start
    while cursor < len(masked):
        char = masked[cursor]
        if char in opening_to_closing:
            delimiter_stack.append(opening_to_closing[char])
        elif delimiter_stack and char == delimiter_stack[-1]:
            delimiter_stack.pop()
        elif not delimiter_stack:
            if char == "|":
                return "|"
            if char in ";}":
                return None
        cursor += 1
    return None


def rust_attributed_construct_end(
    masked: str,
    cursor: int,
    signature_boundary: int | None = None,
    signature_closer: str | None = None,
) -> int | None:
    """Find the source extent controlled by a Rust item/statement attribute."""
    paren_depth = 0
    bracket_depth = 0
    angle_depth = 0
    while cursor < len(masked):
        char = masked[cursor]
        previous = masked[cursor - 1] if cursor else ""
        if char == "(":
            paren_depth += 1
        elif char == ")":
            if paren_depth:
                paren_depth -= 1
            elif not (bracket_depth or angle_depth):
                return cursor - 1
        elif char == "[":
            bracket_depth += 1
        elif char == "]":
            if bracket_depth:
                bracket_depth -= 1
            elif not (paren_depth or angle_depth):
                return cursor - 1
        elif char == "<" and (previous.isalnum() or previous in "_:>"):
            closing_angle = masked.find(">", cursor + 1)
            next_construct = min(
                (
                    position
                    for position in (
                        masked.find("{", cursor + 1),
                        masked.find(";", cursor + 1),
                    )
                    if position != -1
                ),
                default=len(masked),
            )
            if closing_angle != -1 and closing_angle < next_construct:
                angle_depth += 1
        elif char == ">" and previous != "-":
            if angle_depth:
                angle_depth -= 1
            elif signature_closer == ">" and not (paren_depth or bracket_depth):
                return cursor - 1
        elif (
            char == "|"
            and signature_closer == "|"
            and not (paren_depth or bracket_depth or angle_depth)
        ):
            return cursor - 1
        elif not (paren_depth or bracket_depth or angle_depth):
            if char in ";,":
                return cursor
            if char == "}":
                # A final attributed struct field or enum variant ends at its
                # containing declaration, not at a later production item.
                return cursor - 1
            if char == "{":
                if cursor == signature_boundary:
                    # A cfg attribute within a function signature may govern
                    # its final parameter, return component, or where-clause.
                    # The enclosing production body is never part of it.
                    return cursor - 1
                closing = matching_brace(masked, cursor)
                if closing is None:
                    return None
                after = closing + 1
                while after < len(masked) and masked[after].isspace():
                    after += 1
                if masked.startswith("else", after):
                    cursor = after + len("else")
                    continue
                if after < len(masked) and (
                    masked[after] in ".?([+-*/%&|^=<>"
                    or re.match(r"as\b", masked[after:])
                ):
                    cursor = after
                    continue
                return (
                    after if after < len(masked) and masked[after] in ";," else closing
                )
        cursor += 1
    return None


def project_source_ranges(
    source: str, ranges: list[tuple[int, int]], start: int = 0, end: int | None = None
) -> str:
    end = len(source) if end is None else end
    projected = []
    cursor = start
    for first, last in normalized_projected_ranges(source, ranges, start, end):
        projected.append(source[cursor:first])
        cursor = last + 1
    projected.append(source[cursor:end])
    return "".join(projected)


def rust_shipped_source(source: str) -> str:
    """Remove test-required constructs from the shipped-source backstop."""
    masked = mask_noncode(source, nested_block_comments=True)
    return project_source_ranges(source, rust_test_ranges(masked, source))


def split_cfg_arguments(expression: str) -> list[str]:
    masked = mask_noncode(expression, nested_block_comments=True)
    arguments = []
    start = 0
    depth = 0
    for index, char in enumerate(masked):
        if char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
        elif char == "," and depth == 0:
            arguments.append(expression[start:index].strip())
            start = index + 1
    arguments.append(expression[start:].strip())
    return [argument for argument in arguments if argument]


def rust_without_comments(source: str) -> str:
    """Replace Rust comments with trivia while retaining literal cfg atom values."""
    chars = list(source)
    index = 0
    while index < len(source):
        if source.startswith("//", index):
            end = source.find("\n", index + 2)
            end = len(source) if end == -1 else end
            chars[index:end] = " " * (end - index)
            index = end
            continue
        if source.startswith("/*", index):
            start = index
            depth = 1
            index += 2
            while index < len(source) and depth:
                if source.startswith("/*", index):
                    depth += 1
                    index += 2
                elif source.startswith("*/", index):
                    depth -= 1
                    index += 2
                else:
                    index += 1
            if depth:
                raise ValueError("unterminated Rust block comment in cfg expression")
            for offset in range(start, index):
                if chars[offset] != "\n":
                    chars[offset] = " "
            continue
        token_length = rust_string_token_length(source[index:])
        if token_length is not None:
            index += token_length
            continue
        index += 1
    return "".join(chars)


def canonical_cfg_atom(atom: str) -> str:
    """Normalize cfg token trivia and equivalent Rust string spellings."""
    atom = rust_without_comments(atom)
    masked = mask_noncode(atom, nested_block_comments=True)
    equals = [index for index, char in enumerate(masked) if char == "="]
    if len(equals) == 1:
        index = equals[0]
        key = normalize_space(atom[:index])
        token = atom[index + 1 :].strip()
        token_length = rust_string_token_length(token)
        if token_length == len(token):
            return f"{key}={json.dumps(rust_string_value(token), ensure_ascii=False)}"
    parts = []
    cursor = 0
    while cursor < len(atom):
        token_length = rust_string_token_length(atom[cursor:])
        if token_length is not None:
            parts.append(atom[cursor : cursor + token_length])
            cursor += token_length
            continue
        next_literal = cursor + 1
        while next_literal < len(atom):
            if rust_string_token_length(atom[next_literal:]) is not None:
                break
            next_literal += 1
        code = normalize_space(atom[cursor:next_literal])
        parts.append(re.sub(r"\s*=\s*", "=", code))
        cursor = next_literal
    return "".join(parts).strip()


def parse_cfg_expression(expression: str) -> tuple[str, object]:
    expression = rust_without_comments(expression).strip()
    masked = mask_noncode(expression, nested_block_comments=True)
    call = re.fullmatch(r"(all|any|not)\s*\((.*)\)", masked, re.S)
    if not call:
        return ("atom", canonical_cfg_atom(expression))
    operator = call.group(1)
    opening = masked.find("(", call.start())
    body = expression[opening + 1 : masked.rfind(")")]
    return (
        operator,
        tuple(parse_cfg_expression(argument) for argument in split_cfg_arguments(body)),
    )


def cfg_expression_atoms(expression: tuple[str, object]) -> set[str]:
    operator, value = expression
    if operator == "atom":
        return set() if value == "test" else {str(value)}
    return {atom for argument in value for atom in cfg_expression_atoms(argument)}


def evaluate_cfg_expression(
    expression: tuple[str, object], assignments: dict[str, bool]
) -> bool:
    operator, value = expression
    if operator == "atom":
        return False if value == "test" else assignments[str(value)]
    arguments = [evaluate_cfg_expression(argument, assignments) for argument in value]
    if operator == "all":
        return all(arguments)
    if operator == "any":
        return any(arguments)
    return not arguments[0] if arguments else True


def cfg_values_without_test(expression: str) -> set[bool]:
    """Possible predicate values with `test = false` and other atoms consistent."""
    parsed = parse_cfg_expression(expression)
    atoms = sorted(cfg_expression_atoms(parsed))
    if len(atoms) > MAX_CFG_ATOMS:
        raise ValueError(
            "cfg expression has too many distinct atoms for bounded evaluation: "
            f"{len(atoms)} > {MAX_CFG_ATOMS}"
        )
    possible = set()
    for bits in range(1 << len(atoms)):
        assignments = {
            atom: bool(bits & (1 << index)) for index, atom in enumerate(atoms)
        }
        possible.add(evaluate_cfg_expression(parsed, assignments))
        if possible == {False, True}:
            break
    return possible


def cfg_requires_test(expression: str) -> bool:
    """Return true only when a cfg cannot be active with `test = false`."""
    return True not in cfg_values_without_test(expression)


def rust_cfg_meta_constraints(
    meta: str, inherited_predicate: str | None = None
) -> list[str]:
    """Return cfg constraints applied by one real attribute meta item."""
    masked = mask_noncode(meta, nested_block_comments=True)
    stripped = masked.strip()
    direct_expression = None
    if stripped == "test":
        direct_expression = "test"
    else:
        cfg_match = re.fullmatch(r"cfg\s*\((.*)\)\s*", stripped, re.S)
        if cfg_match:
            opening = masked.find("(")
            closing = masked.rfind(")")
            direct_expression = meta[opening + 1 : closing]
    if direct_expression is not None:
        return [
            (
                f"any(not({inherited_predicate}),{direct_expression})"
                if inherited_predicate
                else direct_expression
            )
        ]

    cfg_attr_match = re.fullmatch(r"cfg_attr\s*\((.*)\)\s*", stripped, re.S)
    if cfg_attr_match is None:
        return []
    opening = masked.find("(")
    closing = masked.rfind(")")
    arguments = rust_meta_arguments(meta[opening + 1 : closing])
    if len(arguments) < 2:
        raise ValueError("malformed cfg_attr attribute")
    predicate = arguments[0]
    combined = (
        f"all({inherited_predicate},{predicate})" if inherited_predicate else predicate
    )
    return [
        constraint
        for nested_meta in arguments[1:]
        for constraint in rust_cfg_meta_constraints(nested_meta, combined)
    ]


def attributes_require_test(attributes: str) -> bool:
    expressions = []
    masked = mask_noncode(attributes, nested_block_comments=True)
    attribute_start = re.compile(r"#\s*\[")
    cursor = 0
    while match := attribute_start.search(masked, cursor):
        opening_bracket = masked.find("[", match.start(), match.end())
        bracket_depth = 1
        end = opening_bracket + 1
        while end < len(masked) and bracket_depth:
            bracket_depth += masked[end] == "["
            bracket_depth -= masked[end] == "]"
            end += 1
        if bracket_depth:
            break
        expressions.extend(
            rust_cfg_meta_constraints(attributes[opening_bracket + 1 : end - 1])
        )
        cursor = end
    if not expressions:
        return False
    return cfg_requires_test(f"all({','.join(expressions)})")


def rust_items(path: str, source: str) -> list[dict[str, Any]]:
    masked = mask_noncode(source, nested_block_comments=True)
    test_ranges = rust_test_ranges(masked, source)
    newlines, removed_newlines = source_line_indexes(source, test_ranges)
    contexts = rust_source_contexts(masked, source, test_ranges)
    generated = generated_line_ranges(path, source)
    records = []
    for match in rust_function_matches(masked):
        item_start = rust_leading_attribute_start(masked, match.start())
        if any(start <= item_start <= end for start, end in test_ranges):
            continue
        governing_attributes = source[item_start : match.start()]
        if attributes_require_test(governing_attributes):
            continue
        name = match.group(1)
        context = innermost_context(contexts, match.start(), "module")
        signature_end = match.end()
        depth = 1
        cursor = signature_end
        while cursor < len(masked) and depth:
            if masked[cursor] == "(":
                depth += 1
            elif masked[cursor] == ")":
                depth -= 1
            cursor += 1
        if depth:
            continue
        opening = rust_body_opening(masked, cursor)
        if opening is None:
            continue
        closing = matching_brace(masked, opening)
        if closing is None:
            continue
        signature = rust_projected_identity(source, test_ranges, item_start, opening)
        source_start, start = indexed_line_number(
            newlines, removed_newlines, item_start
        )
        source_end, end = indexed_line_number(newlines, removed_newlines, closing)
        body = project_source_ranges(source, test_ranges, opening, closing + 1)
        overlaps = [
            (first, last)
            for first, last in generated
            if not (source_end < first or source_start > last)
        ]
        if overlaps and not any(
            first <= source_start and source_end <= last for first, last in overlaps
        ):
            raise ValueError(
                f"generated region partially overlaps Rust item {path}:{start}-{end}"
            )
        region = "generated" if overlaps else "handwritten"
        signature_hash = sha256_text(signature)
        records.append(
            {
                "id": f"rust:{path}::{context}::{name}@{signature_hash[:16]}",
                "path": path,
                "name": name,
                "context": context,
                "owner_family": owner_family(path),
                "start_line": start,
                "end_line": end,
                "region": region,
                "signature_sha256": sha256_text(signature),
                "content_sha256": sha256_text(
                    project_source_ranges(source, test_ranges, item_start, closing + 1)
                ),
                "behavior_kinds": behavior_kinds(name, signature, body),
            }
        )
    return records


def split_modules(value: object) -> list[str]:
    return [part.strip() for part in str(value or "").split(";") if part.strip()]


def load_rust_provenance(repo_root: pathlib.Path) -> dict[str, dict[str, Any]]:
    correspondence = tomllib.loads(
        (repo_root / "file-correspondence-manifest.toml").read_text()
    )
    additions = tomllib.loads((repo_root / "rust-additions.toml").read_text())
    by_path: dict[str, dict[str, Any]] = {}
    for row in correspondence.get("file", []):
        upstream = str(row.get("upstream", ""))
        for path in split_modules(row.get("rust_module")):
            record = by_path.setdefault(
                path,
                {"owners": set(), "adapted": False, "evidence": set()},
            )
            if upstream:
                record["owners"].add(upstream)
            if str(row.get("b6_verdict", "")) == "ADAPTED":
                record["adapted"] = True
            evidence = str(row.get("audit_record") or row.get("note") or "")
            if evidence:
                record["evidence"].add(evidence)
    for row in additions.get("addition", []):
        path = str(row.get("path", ""))
        record = by_path.setdefault(
            path,
            {"owners": set(), "adapted": False, "evidence": set()},
        )
        category = str(row.get("category", ""))
        record["addition_category"] = category
        if category in ADAPTATION_CATEGORIES:
            record["adapted"] = True
        record["evidence"].add(f"rust-additions.toml:{category}")
    return by_path


def load_cpp_correspondence(repo_root: pathlib.Path) -> dict[str, dict[str, Any]]:
    manifest = tomllib.loads(
        (repo_root / "file-correspondence-manifest.toml").read_text()
    )
    return {
        str(row.get("upstream", "")): row
        for row in manifest.get("file", [])
        if row.get("upstream")
    }


def workspace_shipped_crates(repo_root: pathlib.Path) -> set[str]:
    workspace = tomllib.loads((repo_root / "Cargo.toml").read_text())
    return {
        pathlib.PurePosixPath(member).name
        for member in workspace.get("workspace", {}).get("members", [])
        if str(member).startswith("crates/")
    }


def header_cpp_owner(path: str) -> str:
    pure = pathlib.PurePosixPath(path)
    if path.startswith("include/rive/"):
        relative = pathlib.PurePosixPath(*pure.parts[2:])
        return (pathlib.PurePosixPath("src") / relative).with_suffix(".cpp").as_posix()
    if path.startswith("src/"):
        return pure.with_suffix(".cpp").as_posix()
    return ""


def attach_cpp_owner_policies(
    repo_root: pathlib.Path,
    files: list[dict[str, Any]],
    members: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    rows = load_cpp_correspondence(repo_root)
    policies: dict[str, dict[str, Any]] = {}
    by_path: dict[str, str] = {}
    for file_record in files:
        path = str(file_record["path"])
        classification = str(file_record["classification"])
        candidate_owner = CPP_OWNER_ALIASES.get(
            path, path if path.endswith(".cpp") else header_cpp_owner(path)
        )
        row = rows.get(candidate_owner)
        if classification == "generated":
            disposition = "generated"
        elif row is None and classification == "behavioral-header":
            disposition = "header-only-unmapped"
            candidate_owner = path
        elif row is None and classification == "declaration-only":
            disposition = "declaration-only"
            candidate_owner = path
        elif row is None:
            disposition = "unmapped"
            candidate_owner = path
        else:
            disposition = {
                "faithful": "mapped",
                "divergent-by-decision": "adapted",
                "partial": "tracked-gap",
                "pending": "tracked-gap",
            }.get(str(row.get("status", "")), "unmapped")
        policy = {
            "cpp_owner": candidate_owner,
            "disposition": disposition,
            "rust_modules": split_modules(row.get("rust_module")) if row else [],
            "evidence": sorted(
                value
                for value in (
                    str(row.get("b6_row_id", "")) if row else "",
                    str(row.get("audit_record", "")) if row else "",
                )
                if value
            ),
        }
        policy_id = (
            "cpp-owner:"
            + sha256_text(json.dumps(policy, sort_keys=True, separators=(",", ":")))[
                :16
            ]
        )
        policies[policy_id] = {"id": policy_id, **policy}
        by_path[path] = policy_id
        file_record["owner_policy"] = policy_id
    for member in members:
        member["owner_policy"] = by_path[str(member["path"])]
        disposition = policies[member["owner_policy"]]["disposition"]
        member["correspondence"] = (
            "unmapped" if disposition == "header-only-unmapped" else disposition
        )
    return [policies[key] for key in sorted(policies)]


def enrich_rust_item(item: dict[str, Any], provenance: dict[str, Any] | None) -> None:
    provenance = provenance or {}
    owners = sorted(provenance.get("owners", set()))
    category = str(provenance.get("addition_category", ""))
    extension = NAMED_EXTENSION_RULES.get(
        (item["path"], item.get("context", "module"), item["name"])
    )
    if extension:
        name, extension_owners, forbidden, evidence = extension
        item["provenance"] = "extension"
        item["extension"] = name
        item["baseline_cpp_owners"] = extension_owners
        item["allowed_call_direction"] = "rust-host-to-cpp-shaped-item"
        item["forbidden_baseline_effects"] = forbidden
        item["evidence"] = evidence
        return
    adaptation = NAMED_ADAPTATION_PATH_RULES.get(item["path"])
    matcher = adaptation[1] if adaptation else None
    item_scope = f"{item.get('context', 'module')}::{item['name']}"
    if adaptation and (
        matcher is None
        or (isinstance(matcher, set) and item.get("context") in matcher)
        or (isinstance(matcher, re.Pattern) and matcher.search(item_scope))
    ):
        name, _, adaptation_owners, forbidden, evidence = adaptation
        item["provenance"] = "adaptation"
        item["adaptation"] = name
        item["baseline_cpp_owners"] = adaptation_owners
        item["allowed_call_direction"] = "cpp-owner-to-rust-item"
        item["forbidden_baseline_effects"] = forbidden
        item["evidence"] = evidence
        return
    if item["region"] == "generated":
        item["provenance"] = "generated"
    elif owners:
        item["provenance"] = "baseline-port"
    elif provenance:
        item["provenance"] = "host-support"
    else:
        item["provenance"] = "unmapped"
    item["baseline_cpp_owners"] = owners


def compact_seam_policies(items: list[dict[str, Any]]) -> list[dict[str, Any]]:
    discovered_selectors = {
        (item["path"], item.get("context", "module"), item["name"]) for item in items
    }
    stale_extensions = sorted(set(NAMED_EXTENSION_RULES) - discovered_selectors)
    if stale_extensions:
        raise ValueError(
            "named extension selectors do not resolve to shipped Rust items: "
            f"{stale_extensions}"
        )
    policies: dict[str, dict[str, Any]] = {}

    def register(policy: dict[str, Any]) -> str:
        policy_id = (
            "seam:"
            + sha256_text(json.dumps(policy, sort_keys=True, separators=(",", ":")))[
                :16
            ]
        )
        policies[policy_id] = {"id": policy_id, **policy}
        return policy_id

    # A reviewed adaptation can intentionally live on a facade/module file
    # with no function body of its own. Keep that configured seam in the
    # authoritative snapshot anyway, so changing its owner, forbidden effects,
    # or evidence cannot bypass the inventory merely because there is no item
    # to carry the compacted policy reference.
    for rust_path, (
        name,
        selector,
        owners,
        forbidden,
        evidence,
    ) in NAMED_ADAPTATION_PATH_RULES.items():
        register(
            {
                "provenance": "adaptation",
                "adaptation": name,
                "extension": None,
                "rust_paths": [rust_path],
                "item_selector": adaptation_selector_label(selector),
                "baseline_cpp_owners": owners,
                "allowed_call_direction": "cpp-owner-to-rust-item",
                "forbidden_baseline_effects": forbidden,
                "evidence": evidence,
            }
        )
    for item in items:
        if item.get("provenance") not in {"adaptation", "extension"}:
            continue
        if item["provenance"] == "adaptation":
            configured_selector = NAMED_ADAPTATION_PATH_RULES[item["path"]][1]
            selector_label = adaptation_selector_label(configured_selector)
        else:
            selector_label = item.get("context", "module") + "::" + item["name"]
        policy = {
            "provenance": item["provenance"],
            "adaptation": item.get("adaptation"),
            "extension": item.get("extension"),
            "rust_paths": [item["path"]],
            "item_selector": selector_label,
            "baseline_cpp_owners": item.get("baseline_cpp_owners", []),
            "allowed_call_direction": item.get("allowed_call_direction"),
            "forbidden_baseline_effects": item.get("forbidden_baseline_effects", []),
            "evidence": item.get("evidence", []),
        }
        policy_id = register(policy)
        item["seam_policy"] = policy_id
        for key in (
            "extension",
            "adaptation",
            "baseline_cpp_owners",
            "allowed_call_direction",
            "forbidden_baseline_effects",
            "evidence",
        ):
            item.pop(key, None)
    return [policies[key] for key in sorted(policies)]


def compact_source_records(records: list[dict[str, Any]]) -> None:
    for record in records:
        for key in (
            "path",
            "name",
            "context",
            "owner_family",
        ):
            record.pop(key, None)


def adaptation_selector_label(selector: object) -> str:
    if selector is None:
        return "all-items-or-module"
    if isinstance(selector, set):
        return "contexts:" + ";".join(sorted(str(value) for value in selector))
    if isinstance(selector, re.Pattern):
        return "regex:" + selector.pattern
    raise TypeError(f"unsupported adaptation selector: {selector!r}")


def approve_host_support(
    inventory: dict[str, Any], expected: dict[str, Any] | None, approve_all: bool
) -> None:
    approved_ids = {
        str(item.get("id", ""))
        for item in (expected or {}).get("rust_items", [])
        if item.get("provenance") == "host-support"
    }
    for item in inventory.get("rust_items", []):
        if item.get("provenance") == "unmapped" and (
            approve_all or item.get("id") in approved_ids
        ):
            item["provenance"] = "host-support"


def approve_header_gaps(
    inventory: dict[str, Any], expected: dict[str, Any] | None, approve_all: bool
) -> None:
    approved_ids = {
        str(member.get("id", ""))
        for member in (expected or {}).get("cpp_members", [])
        if member.get("correspondence") == "reviewed-gap"
    }
    for member in inventory.get("cpp_members", []):
        if member.get("correspondence") == "unmapped" and (
            approve_all or member.get("id") in approved_ids
        ):
            member["correspondence"] = "reviewed-gap"
            member["gap_evidence"] = "UNIV-1976-initial-header-inventory"


def record_family(record: dict[str, Any]) -> str:
    if record.get("owner_family"):
        return str(record["owner_family"])
    stable_id = str(record.get("id", ""))
    if stable_id.startswith("cpp:"):
        return owner_family(stable_id.removeprefix("cpp:").split(":", 1)[0])
    if stable_id.startswith("rust:"):
        return owner_family(stable_id.removeprefix("rust:").split("::", 1)[0])
    return "unknown"


def discover_cpp(
    upstream_root: pathlib.Path,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    files = sorted(
        path
        for root in (upstream_root / "src", upstream_root / "include" / "rive")
        for path in root.rglob("*")
        if path.is_file() and path.suffix in CPP_SUFFIXES
    )
    file_records = []
    member_records = []
    virtual_declarations: set[str] = set()
    for path in files:
        relative = path.relative_to(upstream_root).as_posix()
        source = path.read_text(errors="surrogateescape")
        if path.suffix in {".h", ".hpp"}:
            virtual_declarations.update(cpp_virtual_declarations(source))
        members = cpp_members(relative, source)
        macros = cpp_behavioral_macros(source)
        file_records.append(
            {
                "path": relative,
                "owner_family": owner_family(relative),
                "classification": cpp_file_classification(relative, members, macros),
                "sha256": sha256_text(source),
                "member_count": len(members),
                "behavioral_macro_count": len(macros),
                "behavioral_macros": macros,
            }
        )
        member_records.extend(members)
    for member in member_records:
        if cpp_virtual_key(str(member["name"])) in virtual_declarations:
            member["behavior_kinds"] = sorted(
                set(member["behavior_kinds"]) | {"virtual-override"}
            )
    return file_records, member_records


def rust_skip_trivia(source: str, cursor: int) -> int | None:
    """Skip Rust whitespace and nested comments, returning the next token."""
    while cursor < len(source):
        if source[cursor].isspace():
            cursor += 1
        elif source.startswith("//", cursor):
            newline = source.find("\n", cursor + 2)
            if newline == -1:
                return None
            cursor = newline + 1
        elif source.startswith("/*", cursor):
            depth = 1
            cursor += 2
            while cursor < len(source) and depth:
                if source.startswith("/*", cursor):
                    depth += 1
                    cursor += 2
                elif source.startswith("*/", cursor):
                    depth -= 1
                    cursor += 2
                else:
                    cursor += 1
            if depth:
                return None
        else:
            return cursor
    return cursor


def rust_path_assignment(source: str) -> str | None:
    """Return a decoded path assignment while accepting Rust comment trivia."""
    masked = mask_noncode(source, nested_block_comments=True)
    match = re.search(r"\bpath\s*=", masked)
    if match is None:
        return None
    cursor = match.end()
    cursor = rust_skip_trivia(source, cursor)
    if cursor is None:
        return None
    length = rust_string_token_length(source[cursor:])
    if length is None:
        return None
    return rust_string_value(source[cursor : cursor + length])


def rust_direct_path_assignment(source: str) -> str | None:
    masked = mask_noncode(source, nested_block_comments=True)
    return rust_path_assignment(source) if re.match(r"\s*path\s*=", masked) else None


def rust_path_attribute(attributes: str) -> str | None:
    masked = mask_noncode(attributes, nested_block_comments=True)
    match = re.search(r"#\s*\[\s*path\s*=", masked)
    return rust_path_assignment(attributes[match.start() :]) if match else None


def rust_meta_arguments(source: str) -> list[str]:
    masked = mask_noncode(source, nested_block_comments=True)
    arguments = []
    start = 0
    depths = {"(": 0, "[": 0, "{": 0}
    closing = {")": "(", "]": "[", "}": "{"}
    for index, char in enumerate(masked):
        if char in depths:
            depths[char] += 1
        elif char in closing:
            opener = closing[char]
            depths[opener] -= 1
        elif char == "," and not any(depths.values()):
            arguments.append(source[start:index])
            start = index + 1
    arguments.append(source[start:])
    return [argument.strip() for argument in arguments if argument.strip()]


def rust_static_include_path(source: str, cursor: int) -> str | None:
    """Resolve a repository include path from literals or literal-only concat!."""
    cursor = rust_skip_trivia(source, cursor)
    if cursor is None:
        return None
    token_length = rust_string_token_length(source[cursor:])
    if token_length is not None:
        return rust_string_value(source[cursor : cursor + token_length])
    masked = mask_noncode(source, nested_block_comments=True)
    concat = re.match(r"concat\s*!\s*([({\[])", masked[cursor:])
    if concat is None:
        return None
    opening = cursor + concat.end() - 1
    pairs = {"(": ")", "{": "}", "[": "]"}
    closing = pairs[masked[opening]]
    depth = 1
    end = opening + 1
    while end < len(masked) and depth:
        depth += masked[end] == masked[opening]
        depth -= masked[end] == closing
        end += 1
    if depth:
        return None
    values = []
    for argument in rust_meta_arguments(source[opening + 1 : end - 1]):
        start = rust_skip_trivia(argument, 0)
        if start is None:
            return None
        length = rust_string_token_length(argument[start:])
        if length is None:
            return None
        trailing = argument[start + length :]
        if mask_noncode(trailing, nested_block_comments=True).strip():
            return None
        values.append(rust_string_value(argument[start : start + length]))
    return "".join(values)


def rust_generated_include_output(source: str, cursor: int) -> str | None:
    """Return the normalized output named by a supported OUT_DIR include."""
    cursor = rust_skip_trivia(source, cursor)
    if cursor is None:
        return None
    remaining = source[cursor:]
    masked = mask_noncode(remaining, nested_block_comments=True)
    concat = re.match(r"concat\s*!\s*([({\[])", masked)
    if concat is None:
        return None
    opening = concat.end() - 1
    pairs = {"(": ")", "{": "}", "[": "]"}
    closing = pairs[masked[opening]]
    depth = 1
    end = opening + 1
    while end < len(masked) and depth:
        depth += masked[end] == masked[opening]
        depth -= masked[end] == closing
        end += 1
    if depth:
        return None
    arguments = rust_meta_arguments(remaining[opening + 1 : end - 1])
    if len(arguments) < 2:
        return None
    env_argument = arguments[0]
    env_masked = mask_noncode(env_argument, nested_block_comments=True)
    env_match = re.match(r"\s*env\s*!\s*([({\[])", env_masked)
    if env_match is None:
        return None
    env_opening = env_match.end() - 1
    env_closing = pairs[env_match.group(1)]
    env_end = len(env_masked.rstrip())
    if env_end <= env_opening or env_masked[env_end - 1] != env_closing:
        return None
    env_arguments = rust_meta_arguments(env_argument[env_opening + 1 : env_end - 1])
    if not 1 <= len(env_arguments) <= 2:
        return None
    decoded_env_arguments = []
    for argument in env_arguments:
        start = rust_skip_trivia(argument, 0)
        if start is None:
            return None
        length = rust_string_token_length(argument[start:])
        if (
            length is None
            or mask_noncode(
                argument[start + length :], nested_block_comments=True
            ).strip()
        ):
            return None
        decoded_env_arguments.append(
            rust_string_value(argument[start : start + length])
        )
    if decoded_env_arguments[0] != "OUT_DIR":
        return None
    suffixes = []
    for argument in arguments[1:]:
        start = rust_skip_trivia(argument, 0)
        if start is None:
            return None
        length = rust_string_token_length(argument[start:])
        if (
            length is None
            or mask_noncode(
                argument[start + length :], nested_block_comments=True
            ).strip()
        ):
            return None
        suffixes.append(rust_string_value(argument[start : start + length]))
    suffix = "".join(suffixes)
    if not suffix.startswith("/"):
        return None
    relative = pathlib.PurePosixPath(suffix.removeprefix("/"))
    if not relative.parts or ".." in relative.parts:
        return None
    return f"OUT_DIR/{relative.as_posix()}"


def rust_cfg_attr_paths(
    attributes: str, inherited_predicate: str | None = None
) -> list[tuple[str, str]]:
    """Return (predicate, path) alternatives declared by cfg_attr."""
    masked = mask_noncode(attributes, nested_block_comments=True)
    results = []
    cursor = 0
    pattern = re.compile(r"(?:#\s*\[\s*)?cfg_attr\s*\(")
    while match := pattern.search(masked, cursor):
        opening = masked.find("(", match.start(), match.end())
        depth = 1
        comma = None
        end = opening + 1
        while end < len(masked) and depth:
            char = masked[end]
            if char == "(":
                depth += 1
            elif char == ")":
                depth -= 1
            elif char == "," and depth == 1 and comma is None:
                comma = end
            end += 1
        if depth or comma is None:
            raise ValueError("malformed cfg_attr path attribute")
        predicate = attributes[opening + 1 : comma]
        combined = (
            f"all({inherited_predicate},{predicate})"
            if inherited_predicate
            else predicate
        )
        for meta in rust_meta_arguments(attributes[comma + 1 : end - 1]):
            value = rust_direct_path_assignment(meta)
            if value is not None:
                results.append((combined, value))
            else:
                results.extend(rust_cfg_attr_paths(meta, combined))
        cursor = end
    return results


def rust_inline_module_contexts(masked: str) -> list[tuple[int, int, str]]:
    pattern = re.compile(
        r"(?m)(?:#\s*\[[^]]+\]\s*)*(?:pub(?:\([^)]*\))?\s+)?"
        r"mod\s+(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\s*\{"
    )
    contexts = []
    for match in pattern.finditer(masked):
        opening = masked.rfind("{", match.start(), match.end())
        closing = matching_brace(masked, opening)
        if closing is not None:
            contexts.append((opening, closing, match.group(1)))
    return contexts


def rust_crate_roots(
    repo_root: pathlib.Path, files: list[pathlib.Path]
) -> set[pathlib.Path]:
    """Return conventional and manifest-declared Cargo target roots in scope."""
    candidates = {path.resolve() for path in files}
    roots: set[pathlib.Path] = set()
    manifests = list((repo_root / "crates").glob("*/Cargo.toml"))
    if not manifests:
        return {
            path.resolve()
            for path in files
            if path.name in {"lib.rs", "main.rs"} and path.parent.name == "src"
        }
    roots.update(rust_generator_paths(repo_root) & candidates)
    for manifest_path in manifests:
        manifest = tomllib.loads(manifest_path.read_text())
        package = manifest.get("package", {})
        src_dir = (manifest_path.parent / "src").resolve()
        package = package if isinstance(package, dict) else {}
        lib = manifest.get("lib")
        if isinstance(lib, dict):
            lib_path = lib.get("path", "src/lib.rs")
            if isinstance(lib_path, str):
                resolved = (manifest_path.parent / lib_path).resolve()
                if resolved in candidates:
                    roots.add(resolved)
        elif package.get("autolib", True) and src_dir / "lib.rs" in candidates:
            roots.add(src_dir / "lib.rs")
        auto_bins = package.get("autobins", True)
        bin_dir = (manifest_path.parent / "src/bin").resolve()
        if auto_bins:
            conventional_main = src_dir / "main.rs"
            if conventional_main in candidates:
                roots.add(conventional_main)
            roots.update(
                path.resolve()
                for path in files
                if path.parent.resolve() == bin_dir
                or (path.name == "main.rs" and path.parent.parent.resolve() == bin_dir)
            )
        bins = manifest.get("bin", [])
        targets = bins if isinstance(bins, list) else []
        for target in (target for target in targets if isinstance(target, dict)):
            relative = target.get("path")
            if not isinstance(relative, str):
                name = target.get("name")
                if not isinstance(name, str):
                    raise ValueError(
                        f"bin target has neither path nor name: {manifest_path}"
                    )
                flat = bin_dir / f"{name}.rs"
                nested = bin_dir / name / "main.rs"
                resolved = flat if flat in candidates else nested
            else:
                resolved = (manifest_path.parent / relative).resolve()
            if resolved in candidates:
                roots.add(resolved)
    return roots


def rust_generator_paths(
    repo_root: pathlib.Path,
    crate_names: tuple[str, ...] = RUST_CRATES,
) -> set[pathlib.Path]:
    """Return Cargo's effective build-script target for every crate in scope."""
    generators: set[pathlib.Path] = set()
    for crate in crate_names:
        crate_root = repo_root / "crates" / crate
        manifest_path = crate_root / "Cargo.toml"
        if not manifest_path.is_file():
            continue
        manifest = tomllib.loads(manifest_path.read_text())
        package = manifest.get("package", {})
        package = package if isinstance(package, dict) else {}
        configured = package.get("build")
        if configured is False:
            continue
        if configured is None:
            generator = crate_root / "build.rs"
            if not generator.is_file():
                continue
        elif isinstance(configured, str):
            generator = crate_root / configured
        else:
            raise ValueError(
                f"unsupported Cargo package.build value in {manifest_path}: "
                f"{configured!r}"
            )
        if not generator.is_file():
            raise ValueError(
                f"Cargo build script does not exist: {generator} from {manifest_path}"
            )
        generators.add(generator.resolve())
    return generators


def rust_identifier_tokens(source: str) -> list[tuple[int, int, str]]:
    """Return Rust-like identifiers, including raw and Unicode XID spellings."""
    tokens = []
    cursor = 0
    while cursor < len(source):
        start = cursor
        identifier_start = cursor + 2 if source.startswith("r#", cursor) else cursor
        first = source[identifier_start : identifier_start + 1]
        if not first or not first.isidentifier():
            cursor += 1
            continue
        end = identifier_start + 1
        while end < len(source) and source[identifier_start : end + 1].isidentifier():
            end += 1
        tokens.append((start, end, source[identifier_start:end]))
        cursor = end
    return tokens


def rust_macro_definition_specs(
    masked: str,
    source: str | None = None,
) -> list[tuple[int, str, bool, tuple, int]]:
    """Return local macro definitions with direct behavior and callees."""
    source = masked if source is None else source
    if len(source) != len(masked):
        raise ValueError("Rust macro source and structural mask lengths differ")
    definitions = []
    definition = re.compile(r"\bmacro_rules\s*!\s*")
    for match in definition.finditer(masked):
        suffix = masked[match.end() :]
        name = next(
            (
                token
                for token in rust_identifier_tokens(suffix)
                if not suffix[: token[0]].strip()
            ),
            None,
        )
        if name is None:
            continue
        _name_start, relative_name_end, normalized_name = name
        name_end = match.end() + relative_name_end
        opening = masked.find("{", name_end)
        if opening == -1 or masked[name_end:opening].strip():
            continue
        closing = matching_brace(masked, opening)
        if closing is not None:
            body = masked[opening + 1 : closing]
            source_body = source[opening + 1 : closing]
            arms = rust_macro_arms(body, source_body)
            definitions.append(
                (
                    match.start(),
                    normalized_name,
                    bool(re.search(r"\bmod\s+\$[^;]*;", body)),
                    arms,
                    closing + 1,
                )
            )
    return definitions


def rust_macro_arms(body: str, source_body: str | None = None) -> tuple:
    """Return supported identifier-pattern macro arms and expansion effects."""
    source_body = body if source_body is None else source_body
    if len(source_body) != len(body):
        raise ValueError("Rust macro arm source and structural mask lengths differ")
    pairs = {"(": ")", "[": "]", "{": "}"}
    identifier_fragments = {
        "expr",
        "expr_2021",
        "ident",
        "meta",
        "pat",
        "pat_param",
        "path",
        "stmt",
        "tt",
        "ty",
    }

    def matcher_tokens(text: str) -> tuple:
        identifiers = {
            start: (end, value) for start, end, value in rust_identifier_tokens(text)
        }
        tokens = []
        cursor = 0
        while cursor < len(text):
            if text[cursor] == "$":
                name_start = cursor + 1
                while name_start < len(text) and text[name_start].isspace():
                    name_start += 1
                name_token = identifiers.get(name_start)
                if name_token is not None:
                    name_end, name = name_token
                    name = unicodedata.normalize("NFC", name)
                    colon = name_end
                    while colon < len(text) and text[colon].isspace():
                        colon += 1
                    fragment_start = colon + 1
                    while (
                        colon < len(text)
                        and text[colon] == ":"
                        and fragment_start < len(text)
                        and text[fragment_start].isspace()
                    ):
                        fragment_start += 1
                    fragment_token = identifiers.get(fragment_start)
                    if colon < len(text) and text[colon] == ":" and fragment_token:
                        fragment_end, fragment = fragment_token
                        fragment = unicodedata.normalize("NFC", fragment)
                        tokens.append(
                            (
                                (
                                    "variable"
                                    if fragment in identifier_fragments
                                    else f"unsupported:{fragment}"
                                ),
                                name,
                            )
                        )
                        cursor = fragment_end
                        continue
            identifier = identifiers.get(cursor)
            if identifier is not None:
                end, value = identifier
                value = unicodedata.normalize("NFC", value)
                tokens.append(("literal", value))
                cursor = end
                continue
            cursor += 1
        return tuple(tokens)

    def repetition_constraints(pattern: str, source_pattern: str) -> tuple | None:
        """Return the fixed and repeated portions of the first repetition."""
        for match in re.finditer(r"\$", pattern):
            opening = rust_skip_trivia(pattern, match.end())
            if (
                opening is None
                or opening >= len(pattern)
                or pattern[opening] not in pairs
            ):
                continue
            stack = [pairs[pattern[opening]]]
            cursor = opening + 1
            while cursor < len(pattern) and stack:
                character = pattern[cursor]
                if character in pairs:
                    stack.append(pairs[character])
                elif character == stack[-1]:
                    stack.pop()
                cursor += 1
            if stack:
                return ("", "", "", "", "*")
            source_separator = rust_skip_trivia(source_pattern, cursor)
            source_separator = (
                len(source_pattern) if source_separator is None else source_separator
            )
            separator_start = source_separator
            literal_length = rust_literal_token_length(
                source_pattern[source_separator:]
            )
            lifetime = None
            if literal_length is None and source_pattern.startswith(
                "'", source_separator
            ):
                lifetime = next(
                    (
                        token
                        for token in rust_identifier_tokens(
                            source_pattern[source_separator + 1 :]
                        )
                        if token[0] == 0
                    ),
                    None,
                )
            if literal_length is not None:
                quantifier = source_separator + literal_length
            elif lifetime is not None:
                quantifier = source_separator + 1 + lifetime[1]
            else:
                skipped = rust_skip_trivia(pattern, cursor)
                quantifier = len(pattern) if skipped is None else skipped
                separator_start = quantifier
            compound_separator = next(
                (
                    punctuation
                    for punctuation in (
                        "<<=",
                        ">>=",
                        "...",
                        "..=",
                        "&&",
                        "||",
                        "<<",
                        ">>",
                        "+=",
                        "-=",
                        "*=",
                        "/=",
                        "%=",
                        "^=",
                        "&=",
                        "|=",
                        "==",
                        "!=",
                        ">=",
                        "<=",
                        "->",
                        "=>",
                        "..",
                        "::",
                    )
                    if pattern.startswith(punctuation, quantifier)
                ),
                None,
            )
            if literal_length is not None or lifetime is not None:
                skipped = rust_skip_trivia(pattern, quantifier)
                quantifier = len(pattern) if skipped is None else skipped
            elif compound_separator is not None:
                quantifier += len(compound_separator)
                skipped = rust_skip_trivia(pattern, quantifier)
                quantifier = len(pattern) if skipped is None else skipped
            elif quantifier < len(pattern) and pattern[quantifier] not in "*+?":
                separator_identifiers = rust_identifier_tokens(pattern[quantifier:])
                identifier = next(
                    (token for token in separator_identifiers if token[0] == 0), None
                )
                if identifier is not None:
                    quantifier += identifier[1]
                else:
                    quantifier += 1
                skipped = rust_skip_trivia(pattern, quantifier)
                quantifier = len(pattern) if skipped is None else skipped
            if quantifier < len(pattern) and pattern[quantifier] in "*+?":
                return (
                    source_pattern[: match.start()],
                    source_pattern[quantifier + 1 :],
                    source_pattern[opening + 1 : cursor - 1],
                    source_pattern[separator_start:quantifier].strip(),
                    pattern[quantifier],
                )
        return None

    def delimited_end(start: int) -> int | None:
        stack = [pairs[body[start]]]
        cursor = start + 1
        while cursor < len(body) and stack:
            character = body[cursor]
            if character in pairs:
                stack.append(pairs[character])
            elif character == stack[-1]:
                stack.pop()
            cursor += 1
        return cursor if not stack else None

    arms = []
    cursor = 0
    while cursor < len(body):
        while cursor < len(body) and (body[cursor].isspace() or body[cursor] in ",;"):
            cursor += 1
        if cursor >= len(body) or body[cursor] not in pairs:
            break
        pattern_end = delimited_end(cursor)
        if pattern_end is None:
            break
        pattern = body[cursor + 1 : pattern_end - 1]
        source_pattern = source_body[cursor + 1 : pattern_end - 1]
        cursor = pattern_end
        while cursor < len(body) and body[cursor].isspace():
            cursor += 1
        if not body.startswith("=>", cursor):
            break
        cursor += 2
        while cursor < len(body) and body[cursor].isspace():
            cursor += 1
        if cursor >= len(body) or body[cursor] not in pairs:
            break
        expansion_start = cursor
        expansion_end = delimited_end(cursor)
        if expansion_end is None:
            break
        expansion = body[cursor + 1 : expansion_end - 1]
        source_expansion = source_body[expansion_start + 1 : expansion_end - 1]
        cursor = expansion_end
        pattern_tokens = list(matcher_tokens(pattern))
        pattern_tokens.insert(0, ("matcher:raw", source_pattern))
        fixed_constraints = repetition_constraints(pattern, source_pattern)
        if fixed_constraints is not None:
            pattern_tokens.insert(0, ("unsupported:repetition", fixed_constraints))
        top_level_expansion = list(expansion)
        owned_end = -1
        expansion_invocations = rust_identifier_macro_invocation_specs(
            expansion, source_expansion
        )
        for (
            position,
            end,
            _callee,
            _identifiers,
            _arguments,
        ) in expansion_invocations:
            if position < owned_end:
                continue
            owned_end = end
            top_level_expansion[position:end] = " " * (end - position)
        top_level_expansion = "".join(top_level_expansion)
        direct_targets = []
        identifiers = rust_identifier_tokens(top_level_expansion)
        identifiers_by_start = {
            start: (end, value) for start, end, value in identifiers
        }
        for module_start, module_end, keyword in identifiers:
            if keyword != "mod":
                continue
            target_start = module_end
            while (
                target_start < len(top_level_expansion)
                and top_level_expansion[target_start].isspace()
            ):
                target_start += 1
            variable = (
                target_start < len(top_level_expansion)
                and top_level_expansion[target_start] == "$"
            )
            if variable:
                target_start += 1
                while (
                    target_start < len(top_level_expansion)
                    and top_level_expansion[target_start].isspace()
                ):
                    target_start += 1
            target = identifiers_by_start.get(target_start)
            if target is None:
                continue
            target_end, target_name = target
            target_name = unicodedata.normalize("NFC", target_name)
            semicolon = target_end
            while (
                semicolon < len(top_level_expansion)
                and top_level_expansion[semicolon].isspace()
            ):
                semicolon += 1
            if (
                semicolon >= len(top_level_expansion)
                or top_level_expansion[semicolon] != ";"
            ):
                continue
            attribute_match = re.search(
                r"((?:#\s*\[[^]]+\]\s*)*)$",
                top_level_expansion[:module_start],
                re.S,
            )
            requires_test = bool(
                attribute_match and attributes_require_test(attribute_match.group(1))
            )
            path_targets = ((None, False),)
            if attribute_match:
                source_attributes = source_expansion[
                    attribute_match.start(1) : attribute_match.end(1)
                ]
                explicit_path = rust_path_attribute(source_attributes)
                cfg_attr_paths = rust_cfg_attr_paths(source_attributes)
                if explicit_path is not None and cfg_attr_paths:
                    raise ValueError(
                        "macro-expanded module has both path and cfg_attr(path)"
                    )
                if len(cfg_attr_paths) > 1:
                    raise ValueError(
                        "multiple macro-expanded cfg_attr(path) alternatives"
                    )
                if explicit_path is not None:
                    path_targets = ((explicit_path, False),)
                elif cfg_attr_paths:
                    predicate, alternate_path = cfg_attr_paths[0]
                    path_targets = (
                        (None, cfg_requires_test(f"not({predicate})")),
                        (alternate_path, cfg_requires_test(predicate)),
                    )
            direct_targets.extend(
                (
                    "variable" if variable else "literal",
                    target_name,
                    requires_test or path_requires_test,
                    explicit_path,
                )
                for explicit_path, path_requires_test in path_targets
            )
        delegated = []
        outer_end = -1
        for (
            position,
            end,
            callee,
            _identifiers,
            arguments,
        ) in expansion_invocations:
            if position < outer_end:
                continue
            outer_end = end
            attribute_match = re.search(
                r"((?:#\s*\[[^]]+\]\s*)*)$",
                expansion[:position],
                re.S,
            )
            requires_test = bool(
                attribute_match and attributes_require_test(attribute_match.group(1))
            )
            argument_tokens = []
            masked_arguments = mask_noncode(arguments, nested_block_comments=True)
            argument_identifiers = rust_identifier_tokens(masked_arguments)
            identifiers_by_start = {
                start: (end, value) for start, end, value in argument_identifiers
            }
            argument_cursor = 0
            while argument_cursor < len(masked_arguments):
                if masked_arguments[argument_cursor].isspace():
                    argument_cursor += 1
                    continue
                variable = masked_arguments[argument_cursor] == "$"
                if variable:
                    argument_cursor += 1
                    while (
                        argument_cursor < len(masked_arguments)
                        and masked_arguments[argument_cursor].isspace()
                    ):
                        argument_cursor += 1
                identifier = identifiers_by_start.get(argument_cursor)
                if identifier is None:
                    argument_cursor += 1
                    continue
                identifier_end, identifier_name = identifier
                identifier_name = unicodedata.normalize("NFC", identifier_name)
                argument_tokens.append(
                    ("variable" if variable else "literal", identifier_name)
                )
                argument_cursor = identifier_end
            delegated.append((callee, tuple(argument_tokens), arguments, requires_test))
        arms.append((tuple(pattern_tokens), tuple(direct_targets), tuple(delegated)))
    return tuple(arms)


def rust_macro_arm_modules(
    arms: tuple,
    identifiers: list[str],
    environment: dict[str, tuple[bool, tuple]],
    visiting: frozenset[str],
    raw_arguments: str | None = None,
) -> list[tuple[str, bool]] | None:
    """Resolve a supported macro invocation to concrete module identifiers."""
    identifier_fragments = {
        "expr",
        "expr_2021",
        "ident",
        "meta",
        "pat",
        "pat_param",
        "path",
        "stmt",
        "tt",
        "ty",
    }

    def skip_trivia(text: str, cursor: int) -> int:
        """Skip Rust whitespace and comments, including nested block comments."""
        while cursor < len(text):
            if text[cursor].isspace():
                cursor += 1
                continue
            if text.startswith("//", cursor):
                newline = text.find("\n", cursor + 2)
                cursor = len(text) if newline == -1 else newline + 1
                continue
            if text.startswith("/*", cursor):
                depth = 1
                cursor += 2
                while cursor < len(text) and depth:
                    if text.startswith("/*", cursor):
                        depth += 1
                        cursor += 2
                    elif text.startswith("*/", cursor):
                        depth -= 1
                        cursor += 2
                    else:
                        cursor += 1
                if depth:
                    raise ValueError("unclosed Rust block comment in macro tokens")
                continue
            break
        return cursor

    joint_punctuation = tuple(
        sorted(
            {
                "<<=",
                ">>=",
                "...",
                "..=",
                "&&",
                "||",
                "<<",
                ">>",
                "+=",
                "-=",
                "*=",
                "/=",
                "%=",
                "^=",
                "&=",
                "|=",
                "==",
                "!=",
                ">=",
                "<=",
                "->",
                "=>",
                "..",
                "::",
            },
            key=len,
            reverse=True,
        )
    )

    def punctuation_token(text: str, cursor: int) -> str:
        """Return the longest joint Rust punctuation token at ``cursor``."""
        return next(
            (token for token in joint_punctuation if text.startswith(token, cursor)),
            text[cursor],
        )

    def lexical_tokens(text: str) -> list[tuple[str, str]]:
        """Return NFC Rust tokens needed for macro matching, ignoring trivia."""
        identifiers_by_start = {
            start: (end, text[start:end], value)
            for start, end, value in rust_identifier_tokens(text)
        }
        tokens = []
        cursor = 0
        while cursor < len(text):
            cursor = skip_trivia(text, cursor)
            if cursor >= len(text):
                break
            literal_length = rust_literal_token_length(text[cursor:])
            if literal_length is not None:
                tokens.append(("literal", text[cursor : cursor + literal_length]))
                cursor += literal_length
                continue
            identifier_token = identifiers_by_start.get(cursor)
            if identifier_token is not None:
                end, raw_value, _value = identifier_token
                prefix = "r#" if raw_value.startswith("r#") else ""
                value = raw_value.removeprefix("r#")
                tokens.append(
                    ("identifier", prefix + unicodedata.normalize("NFC", value))
                )
                cursor = end
                continue
            punctuation = punctuation_token(text, cursor)
            tokens.append(("punctuation", punctuation))
            cursor += len(punctuation)
        return tokens

    def structural_matcher_tokens(text: str) -> list[tuple[str, str, str]]:
        """Return variable, literal identifier, and punctuation matcher tokens."""
        identifiers_by_start = {
            start: (end, text[start:end], value)
            for start, end, value in rust_identifier_tokens(text)
        }
        tokens = []
        cursor = 0
        while cursor < len(text):
            cursor = skip_trivia(text, cursor)
            if cursor >= len(text):
                break
            literal_length = rust_literal_token_length(text[cursor:])
            if literal_length is not None:
                tokens.append(("literal", text[cursor : cursor + literal_length], ""))
                cursor += literal_length
                continue
            if text[cursor] == "$":
                name_start = skip_trivia(text, cursor + 1)
                name_token = identifiers_by_start.get(name_start)
                if name_token is not None:
                    name_end, _raw_name, name = name_token
                    name = unicodedata.normalize("NFC", name)
                    colon = skip_trivia(text, name_end)
                    if colon < len(text) and text[colon] == ":":
                        fragment_start = skip_trivia(text, colon + 1)
                        fragment_token = identifiers_by_start.get(fragment_start)
                        if fragment_token is not None:
                            fragment_end, _raw_fragment, fragment = fragment_token
                            fragment = unicodedata.normalize("NFC", fragment)
                            tokens.append(("variable", name, fragment))
                            cursor = fragment_end
                            continue
            identifier_token = identifiers_by_start.get(cursor)
            if identifier_token is not None:
                end, raw_value, _value = identifier_token
                prefix = "r#" if raw_value.startswith("r#") else ""
                value = raw_value.removeprefix("r#")
                tokens.append(
                    ("identifier", prefix + unicodedata.normalize("NFC", value), "")
                )
                cursor = end
                continue
            punctuation = punctuation_token(text, cursor)
            tokens.append(("punctuation", punctuation, ""))
            cursor += len(punctuation)
        return tokens

    def match_fixed_pattern(pattern: str, raw: str) -> tuple[bool, dict[str, str]]:
        """Match the supported one-token fragment subset in source order."""
        pattern_parts = structural_matcher_tokens(pattern)
        raw_parts = lexical_tokens(raw)

        def single_token_tree(text: str) -> str | None:
            start = skip_trivia(text, 0)

            def token_if_trailing_trivia(end: int) -> str | None:
                if skip_trivia(text, end) == len(text):
                    return text[start:end]
                return None

            source = text[start:]
            literal_length = rust_literal_token_length(source)
            if literal_length is not None:
                token = token_if_trailing_trivia(start + literal_length)
                if token is not None:
                    return token
            if source.startswith("'"):
                lifetime_tokens = rust_identifier_tokens(source[1:])
                if len(lifetime_tokens) == 1 and lifetime_tokens[0][0] == 0:
                    token = token_if_trailing_trivia(start + 1 + lifetime_tokens[0][1])
                    if token is not None:
                        return token
            for punctuation in {
                "&&",
                "||",
                "<<",
                ">>",
                "+=",
                "-=",
                "*=",
                "/=",
                "%=",
                "^=",
                "&=",
                "|=",
                "<<=",
                ">>=",
                "==",
                "!=",
                ">=",
                "<=",
                "->",
                "=>",
                "..",
                "..=",
                "...",
                "::",
            }:
                if source.startswith(punctuation):
                    token = token_if_trailing_trivia(start + len(punctuation))
                    if token is not None:
                        return token
            literal = re.match(
                r"(?:b)?'(?:\\.|[^'\\\n])+'|"
                r"(?:0[xob])?[0-9A-Fa-f](?:_?[0-9A-Fa-f]|_)*"
                r"(?:\.(?:[0-9](?:_?[0-9])*)?)?"
                r"(?:[eE][+-]?[0-9](?:_?[0-9])*)?"
                r"(?:[iu](?:8|16|32|64|128|size)|f(?:32|64))?",
                source,
            )
            if literal is not None:
                token = token_if_trailing_trivia(start + literal.end())
                if token is not None:
                    return token
            if len(lexical_tokens(text)) == 1:
                token_kind, token_value = lexical_tokens(text)[0]
                if token_kind == "identifier":
                    identifier = next(
                        token
                        for token in rust_identifier_tokens(source)
                        if token[0] == 0 and token[2] == token_value.removeprefix("r#")
                    )
                    return token_if_trailing_trivia(start + identifier[1])
                return token_if_trailing_trivia(start + 1)
            pairs = {"(": ")", "[": "]", "{": "}"}
            if not source or source[0] not in pairs:
                return None
            masked = mask_noncode(text, nested_block_comments=True)
            stack = [pairs[source[0]]]
            for index in range(start + 1, len(masked)):
                character = masked[index]
                if character in pairs:
                    stack.append(pairs[character])
                elif stack and character == stack[-1]:
                    stack.pop()
                    if not stack:
                        return token_if_trailing_trivia(index + 1)
            return None

        single_tt = None
        if (
            len(pattern_parts) == 1
            and pattern_parts[0][0] == "variable"
            and pattern_parts[0][2] == "tt"
        ):
            single_tt = single_token_tree(raw)
        if single_tt is not None:
            return True, {pattern_parts[0][1]: single_tt}

        def fixed_token_matches(
            matcher_part: tuple[str, str, str], raw_part: tuple[str, str]
        ) -> bool:
            kind, value, _fragment = matcher_part
            raw_kind, raw_value = raw_part
            return kind == "variable" or (kind == raw_kind and value == raw_value)

        first_variable = next(
            (
                index
                for index, part in enumerate(pattern_parts)
                if part[0] == "variable"
            ),
            len(pattern_parts),
        )
        last_variable = next(
            (
                index
                for index in range(len(pattern_parts) - 1, -1, -1)
                if pattern_parts[index][0] == "variable"
            ),
            -1,
        )
        fixed_prefix = pattern_parts[:first_variable]
        fixed_suffix = pattern_parts[last_variable + 1 :] if last_variable >= 0 else []
        if len(raw_parts) < len(fixed_prefix) + len(fixed_suffix):
            return False, {}
        if any(
            not fixed_token_matches(matcher_part, raw_part)
            for matcher_part, raw_part in zip(
                fixed_prefix, raw_parts[: len(fixed_prefix)], strict=True
            )
        ):
            return False, {}
        if fixed_suffix and any(
            not fixed_token_matches(matcher_part, raw_part)
            for matcher_part, raw_part in zip(
                fixed_suffix, raw_parts[-len(fixed_suffix) :], strict=True
            )
        ):
            return False, {}
        if len(pattern_parts) != len(raw_parts):
            if any(
                kind == "variable" and fragment not in {"ident", "tt"}
                for kind, _value, fragment in pattern_parts
            ):
                fragment = next(
                    fragment
                    for kind, _value, fragment in pattern_parts
                    if kind == "variable" and fragment not in {"ident", "tt"}
                )
                raise ValueError(
                    "unsupported Rust macro fragment controls raw arm selection"
                    + (f": {fragment}" if fragment == "vis" else "")
                )
            return False, {}
        bindings = {}
        for (kind, value, fragment), (raw_kind, raw_value) in zip(
            pattern_parts, raw_parts, strict=True
        ):
            if kind == "variable":
                if fragment not in identifier_fragments:
                    raise ValueError(
                        "unsupported Rust macro fragment controls arm selection: "
                        f"{fragment}"
                    )
                if fragment != "tt" and raw_kind != "identifier":
                    return False, {}
                bindings[value] = raw_value
            elif kind != raw_kind or value != raw_value:
                return False, {}
        return True, bindings

    def repetition_matches(descriptor: tuple, raw: str) -> bool | None:
        prefix, suffix, repeated_text, separator, quantifier = descriptor
        nested_structure = mask_noncode(
            repeated_text + suffix, nested_block_comments=True
        )
        if re.search(r"\$\s*[({\[]", nested_structure):
            return None
        repeated_parts = structural_matcher_tokens(repeated_text)
        if len(repeated_parts) != 1 or repeated_parts[0][0] != "variable":
            return None
        _kind, _name, fragment = repeated_parts[0]
        prefix_parts = structural_matcher_tokens(prefix)
        suffix_parts = structural_matcher_tokens(suffix)
        separator_parts = lexical_tokens(separator)
        raw_parts = lexical_tokens(raw)
        complex_edge = any(
            kind == "variable" and edge_fragment not in {"ident", "tt"}
            for kind, _value, edge_fragment in (*prefix_parts, *suffix_parts)
        )
        if complex_edge:
            leading_fixed = []
            for part in prefix_parts:
                if part[0] == "variable":
                    break
                leading_fixed.append(part)
            trailing_fixed = []
            for part in reversed(suffix_parts):
                if part[0] == "variable":
                    break
                trailing_fixed.append(part)
            trailing_fixed.reverse()
            if len(raw_parts) < len(leading_fixed) + len(trailing_fixed):
                return False
            if any(
                (kind, value) != raw_part
                for (kind, value, _fragment), raw_part in zip(
                    leading_fixed, raw_parts[: len(leading_fixed)], strict=True
                )
            ):
                return False
            if trailing_fixed and any(
                (kind, value) != raw_part
                for (kind, value, _fragment), raw_part in zip(
                    trailing_fixed,
                    raw_parts[-len(trailing_fixed) :],
                    strict=True,
                )
            ):
                return False
            return None
        if (
            fragment == "tt"
            and not prefix_parts
            and not suffix_parts
            and not separator_parts
        ):
            if quantifier == "+":
                return bool(raw_parts)
            if quantifier == "?":
                if not raw_parts:
                    return True
                return True if len(raw_parts) == 1 else None
            return True
        if fragment != "ident":
            return None
        fixed_count = len(prefix_parts) + len(suffix_parts)
        if len(raw_parts) < fixed_count:
            return False
        prefix_raw = raw_parts[: len(prefix_parts)]
        suffix_raw = (
            raw_parts[len(raw_parts) - len(suffix_parts) :] if suffix_parts else []
        )
        prefix_raw_text = " ".join(value for _kind, value in prefix_raw)
        suffix_raw_text = " ".join(value for _kind, value in suffix_raw)
        prefix_match, _prefix_bindings = match_fixed_pattern(prefix, prefix_raw_text)
        suffix_match, _suffix_bindings = match_fixed_pattern(suffix, suffix_raw_text)
        if not prefix_match or not suffix_match:
            return False
        middle = raw_parts[
            len(prefix_parts) : (
                len(raw_parts) - len(suffix_parts) if suffix_parts else None
            )
        ]
        if not middle:
            return quantifier in "*?"
        repeated_count = 0
        cursor = 0
        while cursor < len(middle):
            if middle[cursor][0] != "identifier":
                return False
            repeated_count += 1
            cursor += 1
            if cursor == len(middle):
                break
            if (
                not separator_parts
                or middle[cursor : cursor + len(separator_parts)] != separator_parts
            ):
                return False
            cursor += len(separator_parts)
        return quantifier != "?" or repeated_count <= 1

    def substitute_bindings(text: str, bindings: dict[str, str]) -> str:
        """Substitute macro metavariables using normalized Rust identifiers."""
        replacements = []
        cursor = 0
        masked = mask_noncode(text, nested_block_comments=True)
        identifiers = rust_identifier_tokens(masked)
        identifiers_by_start = {
            start: (end, unicodedata.normalize("NFC", value))
            for start, end, value in identifiers
        }
        while cursor < len(masked):
            if masked[cursor] != "$":
                cursor += 1
                continue
            name_start = cursor + 1
            while name_start < len(masked) and masked[name_start].isspace():
                name_start += 1
            identifier = identifiers_by_start.get(name_start)
            if identifier is None:
                cursor += 1
                continue
            name_end, name = identifier
            if name in bindings:
                replacements.append((cursor, name_end, bindings[name]))
            cursor = name_end
        for start, end, replacement in reversed(replacements):
            text = text[:start] + replacement + text[end:]
        return text

    if not arms:
        return None
    invocation = raw_arguments if raw_arguments is not None else " ".join(identifiers)
    for pattern, direct_targets, delegated in arms:
        raw_pattern = next(
            (value for kind, value in pattern if kind == "matcher:raw"), ""
        )
        repetition = next(
            (value for kind, value in pattern if kind == "unsupported:repetition"),
            None,
        )
        if repetition is not None:
            matches = repetition_matches(repetition, invocation)
            if matches is None:
                raise ValueError(
                    "unsupported Rust macro repetition controls arm selection"
                )
            if not matches:
                continue
            if not direct_targets and not delegated:
                return []
            raise ValueError(
                "unsupported Rust macro fragment controls arm selection: repetition"
            )
        unsupported = next(
            (
                kind.removeprefix("unsupported:")
                for kind, _value in pattern
                if kind.startswith("unsupported:")
            ),
            None,
        )
        if unsupported is not None:
            variable_pattern = re.compile(
                rf"\$[A-Za-z_][A-Za-z0-9_]*\s*:\s*{re.escape(unsupported)}"
            )
            remainder = variable_pattern.sub("", raw_pattern, count=1)
            if unsupported == "vis":
                matches, _bindings = match_fixed_pattern(remainder, invocation)
                if matches:
                    raise ValueError(
                        "unsupported Rust macro fragment controls raw arm selection: vis"
                    )
            elif unsupported == "block":
                stripped = invocation.strip()
                if not remainder.strip() and stripped[:1] in "({[":
                    raise ValueError(
                        "unsupported Rust macro fragment controls raw arm selection"
                    )
            elif raw_arguments is None and len(identifiers) == sum(
                kind in {"variable", "literal"} for kind, _value in pattern
            ):
                raise ValueError(
                    "unsupported Rust macro fragment controls arm selection: "
                    f"{unsupported}"
                )
        matched, bindings = match_fixed_pattern(raw_pattern, invocation)
        if not matched:
            continue
        if matched:
            identifier_bindings = {
                name: tokens[0][1].removeprefix("r#")
                for name, value in bindings.items()
                if len(tokens := lexical_tokens(value)) == 1
                and tokens[0][0] == "identifier"
            }
            non_identifier_bindings = {
                name for name in bindings if name not in identifier_bindings
            }
            if non_identifier_bindings and (
                any(
                    kind == "variable" and value in non_identifier_bindings
                    for kind, value, _requires_test, _explicit_path in direct_targets
                )
                or delegated
            ):
                raise ValueError(
                    "unsupported Rust tt fragment controls module generation"
                )
            modules = [
                (
                    (
                        f"@path:{explicit_path}"
                        if explicit_path is not None
                        else (
                            identifier_bindings[value] if kind == "variable" else value
                        )
                    ),
                    requires_test,
                )
                for kind, value, requires_test, explicit_path in direct_targets
                if kind == "literal" or value in bindings
            ]
            for (
                callee,
                argument_tokens,
                delegated_arguments,
                requires_test,
            ) in delegated:
                concrete_arguments = [
                    identifier_bindings[value] if kind == "variable" else value
                    for kind, value in argument_tokens
                    if kind == "literal" or value in bindings
                ]
                concrete_raw_arguments = substitute_bindings(
                    delegated_arguments, bindings
                )
                modules.extend(
                    (module, requires_test or module_requires_test)
                    for module, module_requires_test in rust_resolve_macro_modules(
                        callee,
                        concrete_arguments,
                        environment,
                        visiting,
                        concrete_raw_arguments,
                    )
                )
            return modules
    return None


def rust_resolve_macro_modules(
    name: str,
    identifiers: list[str],
    environment: dict[str, tuple[bool, tuple]],
    visiting: frozenset[str] = frozenset(),
    raw_arguments: str | None = None,
) -> list[tuple[str, bool]]:
    """Resolve one supported macro invocation through delegated wrappers."""
    definition = environment.get(name)
    if definition is None:
        return []
    if name in visiting:
        raise ValueError(f"cyclic module-generating macro expansion: {name}!")
    direct, arms = definition
    if not direct and not any(
        direct_targets or delegated for _pattern, direct_targets, delegated in arms
    ):
        return []
    try:
        modules = rust_macro_arm_modules(
            arms, identifiers, environment, visiting | {name}, raw_arguments
        )
    except ValueError as error:
        raise ValueError(f"{error} in {name}!") from error
    if modules is not None:
        return modules
    return [(identifier, False) for identifier in identifiers] if direct else []


def rust_macro_definitions(masked: str) -> list[tuple[int, str, bool]]:
    """Return directly module-generating local macro definitions."""
    return [
        (position, name, direct)
        for position, name, direct, _callees, _closing in (
            rust_macro_definition_specs(masked)
        )
    ]


def rust_macro_definition_ranges(masked: str) -> list[tuple[int, int]]:
    """Return complete local ``macro_rules!`` definition ranges."""
    return [
        (position, closing)
        for position, _name, _direct, _callees, closing in (
            rust_macro_definition_specs(masked)
        )
    ]


def rust_module_macro_definitions(masked: str) -> set[str]:
    """Return macros in ``masked`` whose expansion can declare ``mod $x``."""
    return {
        name
        for _position, name, generates in rust_macro_definitions(masked)
        if generates
    }


def rust_identifier_macro_invocation_specs(
    masked: str,
    source: str | None = None,
) -> list[tuple[int, int, str, list[str], str]]:
    """Return structurally matched macro names, identifiers, and arguments."""
    source = masked if source is None else source
    if len(source) != len(masked):
        raise ValueError("Rust macro invocation source and mask lengths differ")
    invocations = []
    pairs = {"(": ")", "[": "]", "{": "}"}
    for invocation_start, invocation_end, name in rust_identifier_tokens(masked):
        cursor = invocation_end
        while cursor < len(masked) and masked[cursor].isspace():
            cursor += 1
        if cursor >= len(masked) or masked[cursor] != "!":
            continue
        cursor += 1
        while cursor < len(masked) and masked[cursor].isspace():
            cursor += 1
        if cursor >= len(masked) or masked[cursor] not in pairs:
            continue
        opening = cursor
        closing_char = pairs[masked[opening]]
        depth = 1
        cursor = opening + 1
        while cursor < len(masked) and depth:
            depth += masked[cursor] == masked[opening]
            depth -= masked[cursor] == closing_char
            cursor += 1
        if depth:
            raise ValueError(f"unclosed Rust macro invocation: {name}!")
        masked_arguments = masked[opening + 1 : cursor - 1]
        arguments = source[opening + 1 : cursor - 1]
        identifiers = [token[2] for token in rust_identifier_tokens(masked_arguments)]
        invocations.append((invocation_start, cursor, name, identifiers, arguments))
    return invocations


def rust_identifier_macro_invocations(
    masked: str,
) -> list[tuple[int, str, list[str]]]:
    """Return macro names and identifier arguments from structurally valid calls."""
    return [
        (position, name, identifiers)
        for position, _end, name, identifiers, _arguments in (
            rust_identifier_macro_invocation_specs(masked)
        )
    ]


def rust_macro_module_invocations(
    masked: str, definitions: set[str] | None = None
) -> list[tuple[int, str]]:
    """Return module names passed to visible module-generating macros."""
    definitions = definitions or rust_module_macro_definitions(masked)
    return [
        (invocation_start, identifier)
        for invocation_start, name, identifiers in rust_identifier_macro_invocations(
            masked
        )
        if name in definitions
        for identifier in identifiers
    ]


def external_test_module_paths(
    repo_root: pathlib.Path, files: list[pathlib.Path]
) -> set[pathlib.Path]:
    """Expand ``files`` through modules and return non-production candidates."""
    candidates = {path.resolve() for path in files}
    crate_roots = {
        manifest.parent.resolve()
        for manifest in (repo_root / "crates").glob("*/Cargo.toml")
    }
    generator_outputs_by_crate = {}
    for crate_root in crate_roots:
        outputs = set()
        for generator in rust_generator_paths(
            repo_root, crate_names=(crate_root.name,)
        ):
            relative_generator = generator.relative_to(repo_root.resolve()).as_posix()
            outputs.update(RUST_GENERATOR_OUTPUTS.get(relative_generator, []))
        generator_outputs_by_crate[crate_root] = outputs
    declaration = re.compile(
        r"(?ms)((?:#\s*\[[^]]+\]\s*)*)(?:pub(?:\([^)]*\))?\s+)?mod\s+"
        r"(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\s*;"
    )
    include_declaration = re.compile(
        r"(?ms)((?:#\s*\[[^]]+\]\s*)*)(?P<macro>include)\s*!\s*([({\[])"
    )
    parsed: dict[
        pathlib.Path,
        tuple[str, str, list[tuple[int, int, str]], list[tuple[int, int]]],
    ] = {}
    macro_definition_events: dict[
        pathlib.Path, list[tuple[int, str, bool, frozenset[str], int]]
    ] = {}
    macro_imports: dict[pathlib.Path, list[tuple[int, pathlib.Path]]] = {}
    macro_includes: dict[pathlib.Path, list[tuple[int, pathlib.Path]]] = {}
    macro_exports: dict[pathlib.Path, dict[str, tuple[bool, tuple]]] = {}

    def add_candidate(path: pathlib.Path, *, require_rust_suffix: bool = True) -> bool:
        resolved = path.resolve()
        if resolved in candidates:
            return True
        if (
            (require_rust_suffix and resolved.suffix != ".rs")
            or not resolved.is_file()
            or not any(resolved.is_relative_to(root) for root in crate_roots)
        ):
            return False
        candidates.add(resolved)
        files.append(resolved)
        parse_candidate(resolved)
        return True

    def parse_candidate(owner: pathlib.Path) -> None:
        source = owner.read_text()
        masked = mask_noncode(source, nested_block_comments=True)
        parsed[owner.resolve()] = (
            source,
            masked,
            rust_inline_module_contexts(masked),
            rust_test_ranges(masked, source),
        )
        test_ranges = parsed[owner.resolve()][3]
        macro_definition_events[owner.resolve()] = [
            definition
            for definition in rust_macro_definition_specs(masked, source)
            if not any(start <= definition[0] < end for start, end in test_ranges)
        ]

    for owner in list(files):
        parse_candidate(owner)
    owner_states: dict[pathlib.Path, set[tuple[pathlib.Path, bool]]] = {
        path.resolve(): {(path.parent, False)}
        for path in rust_crate_roots(repo_root, files)
    }

    def lexical_scope(owner_resolved: pathlib.Path, position: int) -> tuple[int, int]:
        masked = parsed[owner_resolved][1]
        stack = []
        containing = [(-1, len(masked))]
        for cursor, character in enumerate(masked):
            if character == "{":
                stack.append(cursor)
            elif character == "}" and stack:
                opening = stack.pop()
                if opening < position < cursor:
                    containing.append((opening, cursor))
        return max(containing, key=lambda scope: scope[0])

    def add_macro_modules(
        owner_resolved: pathlib.Path,
        base: pathlib.Path,
        inherited_requires_test: bool,
        imported_definitions: dict[str, tuple[bool, tuple]],
    ) -> bool:
        source, masked, inline_modules, test_ranges = parsed[owner_resolved]
        changed = False
        local_definitions = macro_definition_stream(
            owner_resolved, include_test_definitions=inherited_requires_test
        )
        definition_ranges = rust_macro_definition_ranges(masked)
        for (
            invocation_start,
            _invocation_end,
            name,
            identifiers,
            raw_arguments,
        ) in rust_identifier_macro_invocation_specs(masked, source):
            if any(start <= invocation_start < end for start, end in definition_ranges):
                continue
            environment = macro_environment_at(
                owner_resolved, invocation_start, imported_definitions
            )
            modules = rust_resolve_macro_modules(
                name, identifiers, environment, raw_arguments=raw_arguments
            )
            if not modules:
                continue
            local_requires_test = any(
                start <= invocation_start < end for start, end in test_ranges
            )
            ancestors = sorted(
                (
                    context
                    for context in inline_modules
                    if context[0] < invocation_start < context[1]
                ),
                key=lambda context: context[0],
            )
            declaration_base = base
            explicit_base = base if ancestors else owner_resolved.parent
            for _opening, _closing, ancestor_name in ancestors:
                declaration_base /= ancestor_name
                explicit_base /= ancestor_name
            for module, expansion_requires_test in modules:
                explicit_path = (
                    module.removeprefix("@path:")
                    if module.startswith("@path:")
                    else None
                )
                targets = (
                    ((explicit_base / explicit_path, True),)
                    if explicit_path is not None
                    else (
                        (declaration_base / f"{module}.rs", False),
                        (declaration_base / module / "mod.rs", False),
                    )
                )
                for target, target_is_explicit in targets:
                    resolved = target.resolve()
                    if add_candidate(resolved):
                        state = (
                            (
                                resolved.parent
                                if target_is_explicit
                                else declaration_base / module
                            ),
                            inherited_requires_test
                            or local_requires_test
                            or expansion_requires_test,
                        )
                        states = owner_states.setdefault(resolved, set())
                        if state not in states:
                            states.add(state)
                            changed = True
        return changed

    def macro_definition_stream(
        owner_resolved: pathlib.Path,
        *,
        include_test_definitions: bool = False,
        exports: dict[pathlib.Path, dict[str, tuple[bool, tuple]]] | None = None,
    ) -> list[tuple[int, str, bool, tuple, int, int]]:
        exports = macro_exports if exports is None else exports
        raw_definitions = (
            rust_macro_definition_specs(
                parsed[owner_resolved][1], parsed[owner_resolved][0]
            )
            if include_test_definitions
            else list(macro_definition_events[owner_resolved])
        )
        definitions = [
            (
                position,
                name,
                direct,
                callees,
                *lexical_scope(owner_resolved, position),
            )
            for position, name, direct, callees, _closing in raw_definitions
        ]
        for include_position, included in macro_includes.get(owner_resolved, []):
            scope_start, scope_end = lexical_scope(owner_resolved, include_position)
            definitions.extend(
                (
                    include_position,
                    name,
                    generates,
                    arms,
                    scope_start,
                    scope_end,
                )
                for name, (generates, arms) in exports.get(included, {}).items()
            )
        for import_position, imported in macro_imports.get(owner_resolved, []):
            scope_start, scope_end = lexical_scope(owner_resolved, import_position)
            definitions.extend(
                (
                    import_position,
                    name,
                    generates,
                    arms,
                    scope_start,
                    scope_end,
                )
                for name, (generates, arms) in exports.get(imported, {}).items()
            )
        resolved = []
        for position, name, direct, arms, scope_start, scope_end in sorted(
            definitions, key=lambda definition: definition[0]
        ):
            resolved.append((position, name, direct, arms, scope_start, scope_end))
        return resolved

    def macro_environment_at(
        owner_resolved: pathlib.Path,
        position: int,
        imported_definitions: dict[str, tuple[bool, tuple]],
    ) -> dict[str, tuple[bool, tuple]]:
        environment = dict(imported_definitions)
        for (
            definition_position,
            name,
            direct,
            arms,
            scope_start,
            scope_end,
        ) in macro_definition_stream(owner_resolved):
            if definition_position >= position:
                break
            if not scope_start < position < scope_end:
                continue
            environment[name] = (direct, arms)
        return environment

    processed: set[tuple[pathlib.Path, pathlib.Path, bool]] = set()
    while True:
        pending = [
            (owner, base, inherited_requires_test)
            for owner, states in owner_states.items()
            for base, inherited_requires_test in states
            if (owner, base, inherited_requires_test) not in processed
        ]
        if not pending:
            macro_exports = {owner: {} for owner in macro_definition_events}
            seen_exports = set()
            while True:
                fingerprint = tuple(
                    (owner, tuple(sorted(exports.items())))
                    for owner, exports in sorted(macro_exports.items())
                )
                if fingerprint in seen_exports:
                    raise ValueError("macro export visibility does not converge")
                seen_exports.add(fingerprint)
                updated_exports = {}
                for owner in macro_exports:
                    exported = {}
                    for (
                        _position,
                        name,
                        generates,
                        arms,
                        scope_start,
                        scope_end,
                    ) in macro_definition_stream(owner, exports=macro_exports):
                        if (scope_start, scope_end) != (
                            -1,
                            len(parsed[owner][1]),
                        ):
                            continue
                        exported[name] = (generates, arms)
                    updated_exports[owner] = exported
                if updated_exports == macro_exports:
                    break
                macro_exports = updated_exports
            include_imports = {owner: {} for owner in macro_definition_events}
            imports_changed = True
            while imports_changed:
                imports_changed = False
                for owner, inclusions in macro_includes.items():
                    imported = include_imports[owner]
                    for position, included in inclusions:
                        environment = macro_environment_at(owner, position, imported)
                        before = dict(include_imports[included])
                        include_imports[included].update(environment)
                        imports_changed |= include_imports[included] != before
            changed = False
            for owner_resolved, states in list(owner_states.items()):
                for base, inherited_requires_test in list(states):
                    changed |= add_macro_modules(
                        owner_resolved,
                        base,
                        inherited_requires_test,
                        include_imports[owner_resolved],
                    )
            if changed:
                continue
            break
        for owner_resolved, base, inherited_requires_test in pending:
            processed.add((owner_resolved, base, inherited_requires_test))
            source, masked, inline_modules, test_ranges = parsed[owner_resolved]
            add_macro_modules(
                owner_resolved,
                base,
                inherited_requires_test,
                {},
            )
            for match in include_declaration.finditer(masked):
                macro_start = match.start("macro")
                if source[max(0, macro_start - 2) : macro_start] == "r#" or (
                    macro_start
                    and (
                        source[macro_start - 1] == "$"
                        or ("a" + source[macro_start - 1]).isidentifier()
                    )
                ):
                    continue
                attributes = source[match.start(1) : match.end(1)]
                local_requires_test = attributes_require_test(attributes) or any(
                    start <= match.start() < end for start, end in test_ranges
                )
                if inherited_requires_test or local_requires_test:
                    continue
                include_path = rust_static_include_path(source, match.end())
                if include_path is None:
                    generated_output = rust_generated_include_output(
                        source, match.end()
                    )
                    if generated_output is not None:
                        crate_root = next(
                            (
                                root
                                for root in crate_roots
                                if owner_resolved.is_relative_to(root)
                            ),
                            None,
                        )
                        if generated_output not in generator_outputs_by_crate.get(
                            crate_root, set()
                        ):
                            raise ValueError(
                                "generated include output is not declared for its "
                                f"Cargo build script: {generated_output} from "
                                f"{owner_resolved}"
                            )
                        continue
                    raise ValueError(
                        f"unresolved repository include in {owner_resolved}: "
                        f"line {line_number(source, match.start())}"
                    )
                included = (owner_resolved.parent / include_path).resolve()
                if not add_candidate(included, require_rust_suffix=False):
                    raise ValueError(
                        f"repository include is outside shipped Rust source scope: "
                        f"{included} from {owner_resolved}"
                    )
                owner_states.setdefault(included, set()).add(
                    (included.parent, inherited_requires_test or local_requires_test)
                )
                macro_includes.setdefault(owner_resolved, []).append(
                    (match.start(), included)
                )
            for match in declaration.finditer(masked):
                if any(
                    start <= match.start() < end
                    for start, end in rust_macro_definition_ranges(masked)
                ):
                    continue
                attributes = source[match.start(1) : match.end(1)]
                ancestors = sorted(
                    (
                        context
                        for context in inline_modules
                        if context[0] < match.start() < context[1]
                    ),
                    key=lambda context: context[0],
                )
                declaration_base = base
                explicit_base = base if ancestors else owner_resolved.parent
                for _opening, _closing, name in ancestors:
                    declaration_base /= name
                    explicit_base /= name
                module = match.group(2)
                explicit_path = rust_path_attribute(attributes)
                cfg_attr_paths = rust_cfg_attr_paths(attributes)
                if explicit_path is not None and cfg_attr_paths:
                    raise ValueError(
                        "module has both path and cfg_attr(path) attributes"
                    )
                if len(cfg_attr_paths) > 1:
                    raise ValueError(
                        "multiple cfg_attr(path) alternatives are unsupported"
                    )
                if explicit_path is not None:
                    targets = ((explicit_base / explicit_path, False, True),)
                else:
                    default_targets = (
                        (declaration_base / f"{module}.rs", False, False),
                        (declaration_base / module / "mod.rs", False, False),
                    )
                    if cfg_attr_paths:
                        predicate, alternate_path = cfg_attr_paths[0]
                        default_requires_test = cfg_requires_test(f"not({predicate})")
                        targets = tuple(
                            (target, default_requires_test, False)
                            for target, _requires_test, _explicit in default_targets
                        ) + (
                            (
                                explicit_base / alternate_path,
                                cfg_requires_test(predicate),
                                True,
                            ),
                        )
                    else:
                        targets = default_targets
                for target, path_requires_test, target_is_explicit in targets:
                    resolved = target.resolve()
                    if add_candidate(resolved):
                        if target_is_explicit:
                            child_base = resolved.parent
                        else:
                            child_base = declaration_base / module
                        local_requires_test = attributes_require_test(
                            attributes
                        ) or any(
                            start <= match.start() < end for start, end in test_ranges
                        )
                        if (
                            not inherited_requires_test
                            and not local_requires_test
                            and not path_requires_test
                            and re.search(
                                r"#\s*\[\s*macro_use(?:\s*\([^]]*\))?\s*\]",
                                mask_noncode(attributes, nested_block_comments=True),
                            )
                        ):
                            macro_imports.setdefault(owner_resolved, []).append(
                                (match.start(), resolved)
                            )
                        owner_states.setdefault(resolved, set()).add(
                            (
                                child_base,
                                inherited_requires_test
                                or local_requires_test
                                or path_requires_test,
                            )
                        )

    production_reachable = {
        owner
        for owner, states in owner_states.items()
        if any(not requires_test for _base, requires_test in states)
    }
    return candidates - production_reachable


def rust_source_candidates(
    repo_root: pathlib.Path,
    crate_names: tuple[str, ...] = RUST_CRATES,
) -> list[pathlib.Path]:
    """Return Rust sources that production Cargo reachability may select."""
    sources = {
        path
        for crate in crate_names
        for path in (repo_root / "crates" / crate / "src").rglob("*.rs")
        if path.is_file()
    }
    sources.update(rust_generator_paths(repo_root, crate_names))
    all_crate_sources = [
        path
        for crate in crate_names
        for path in (repo_root / "crates" / crate).rglob("*.rs")
        if path.is_file()
    ]
    sources.update(rust_crate_roots(repo_root, all_crate_sources))
    return sorted(sources)


def discover_rust(
    repo_root: pathlib.Path,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    provenance = load_rust_provenance(repo_root)
    candidates = rust_source_candidates(repo_root)
    external_test_modules = external_test_module_paths(repo_root, candidates)
    files = [path for path in candidates if path.resolve() not in external_test_modules]
    file_records = []
    item_records = []
    for path in files:
        relative = path.relative_to(repo_root).as_posix()
        source = path.read_text()
        items = rust_items(relative, source)
        generator_outputs = RUST_GENERATOR_OUTPUTS.get(relative)
        item_provenance = provenance.get(relative)
        if generator_outputs is not None:
            item_provenance = {
                "owners": set(),
                "adapted": False,
                "evidence": {relative},
                "addition_category": "runtime-generator",
            }
        for item in items:
            enrich_rust_item(item, item_provenance)
            if generator_outputs is not None:
                item["generator_outputs"] = generator_outputs
        file_record = {
            "path": relative,
            "owner_family": owner_family(relative),
            "sha256": sha256_text(rust_shipped_source(source)),
            "item_count": len(items),
            "generated_item_count": sum(
                item["region"] == "generated" for item in items
            ),
            "handwritten_item_count": sum(
                item["region"] == "handwritten" for item in items
            ),
        }
        if relative in GENERATED_RUST_FILES:
            file_record["generated_by"] = GENERATED_RUST_FILES[relative]
        if generator_outputs is not None:
            file_record["generator_outputs"] = generator_outputs
        file_records.append(file_record)
        item_records.extend(items)
    return file_records, item_records


def build_inventory(
    repo_root: pathlib.Path, upstream_root: pathlib.Path, upstream_ref: str
) -> dict[str, Any]:
    cpp_files, cpp_member_records = discover_cpp(upstream_root)
    rust_files, rust_item_records = discover_rust(repo_root)
    cpp_owner_policies = attach_cpp_owner_policies(
        repo_root, cpp_files, cpp_member_records
    )
    seam_policies = compact_seam_policies(rust_item_records)
    compact_source_records(cpp_member_records)
    compact_source_records(rust_item_records)
    return {
        "schema": SCHEMA,
        "upstream_ref": upstream_ref,
        "scope": {
            "cpp": ["src/**/*.{cpp,mm,h,hpp}", "include/rive/**/*.{h,hpp}"],
            "rust_crates": list(RUST_CRATES),
            "rust_generators": sorted(RUST_GENERATOR_OUTPUTS),
        },
        "summary": {
            "cpp_files": len(cpp_files),
            "cpp_members": len(cpp_member_records),
            "rust_files": len(rust_files),
            "rust_items": len(rust_item_records),
            "seam_policies": len(seam_policies),
            "cpp_owner_policies": len(cpp_owner_policies),
        },
        "cpp_files": cpp_files,
        "cpp_members": cpp_member_records,
        "cpp_owner_policies": cpp_owner_policies,
        "rust_files": rust_files,
        "rust_items": rust_item_records,
        "seam_policies": seam_policies,
    }


def adaptation_policy_coverage_errors(
    additions: dict[str, Any],
    manifest: dict[str, Any],
    named_paths: set[str] | None = None,
) -> list[str]:
    named_paths = named_paths or set(NAMED_ADAPTATION_PATH_RULES)
    addition_approvals = {
        str(row.get("path", ""))
        for row in additions.get("addition", [])
        if row.get("category") in ADAPTATION_CATEGORIES
    }
    manifest_approvals = {
        path
        for row in manifest.get("file", [])
        if row.get("status") == "divergent-by-decision"
        or row.get("b6_verdict") == "ADAPTED"
        for path in split_modules(row.get("rust_module"))
    }
    missing = addition_approvals - named_paths
    stale = named_paths - addition_approvals - manifest_approvals
    if not (missing or stale):
        return []
    return [
        "[seams] approved adaptation policy coverage mismatch: "
        f"missing={sorted(missing)}, stale={sorted(stale)}"
    ]


def validate_configuration(repo_root: pathlib.Path) -> list[str]:
    errors = []
    shipped = workspace_shipped_crates(repo_root)
    declared = set(RUST_CRATES)
    if declared != shipped:
        errors.append(
            "[scope] shipped Rust crate coverage mismatch: "
            f"missing={sorted(shipped - declared)}, extra={sorted(declared - shipped)}"
        )
    generator_paths = {
        path.relative_to(repo_root).as_posix()
        for path in rust_generator_paths(repo_root)
    }
    declared_generators = set(RUST_GENERATOR_OUTPUTS)
    if generator_paths != declared_generators:
        errors.append(
            "[generated] runtime generator coverage mismatch: "
            f"missing={sorted(generator_paths - declared_generators)}, "
            f"stale={sorted(declared_generators - generator_paths)}"
        )
    for path, outputs in RUST_GENERATOR_OUTPUTS.items():
        if not outputs or len(outputs) != len(set(outputs)):
            errors.append(f"[generated] invalid runtime generator outputs: {path}")
    additions = tomllib.loads((repo_root / "rust-additions.toml").read_text())
    manifest = tomllib.loads(
        (repo_root / "file-correspondence-manifest.toml").read_text()
    )
    errors.extend(adaptation_policy_coverage_errors(additions, manifest))
    for path, generator in GENERATED_RUST_FILES.items():
        source_path = repo_root / path
        if not source_path.is_file():
            errors.append(
                f"[generated] declared generated Rust source does not exist: {path}"
            )
        generator_path = repo_root / generator
        if not generator_path.exists():
            errors.append(f"[generated] declared generator does not exist: {generator}")
    evidence_references = {
        evidence
        for rule in (
            *NAMED_ADAPTATION_PATH_RULES.values(),
            *NAMED_EXTENSION_RULES.values(),
        )
        for evidence in rule[-1]
    }
    for evidence in sorted(evidence_references):
        path_text, separator, selector = evidence.partition(":")
        evidence_path = repo_root / path_text
        if not evidence_path.exists():
            errors.append(f"[seams] required evidence does not exist: {evidence}")
        elif separator and selector not in evidence_path.read_text():
            errors.append(
                f"[seams] required evidence selector does not exist: {evidence}"
            )
    return errors


def validate_inventory(inventory: dict[str, Any]) -> list[str]:
    errors = []
    if inventory.get("schema") not in {None, SCHEMA}:
        errors.append(
            f"[inventory] invalid schema: expected {SCHEMA}, found {inventory.get('schema')}"
        )
    summary = inventory.get("summary")
    if isinstance(summary, dict):
        for summary_key, record_key in (
            ("cpp_files", "cpp_files"),
            ("cpp_members", "cpp_members"),
            ("cpp_owner_policies", "cpp_owner_policies"),
            ("rust_files", "rust_files"),
            ("rust_items", "rust_items"),
            ("seam_policies", "seam_policies"),
        ):
            if summary.get(summary_key) != len(inventory.get(record_key, [])):
                errors.append(
                    f"[inventory] summary mismatch for {summary_key}: "
                    f"{summary.get(summary_key)} != {len(inventory.get(record_key, []))}"
                )
    for key in ("cpp_files", "rust_files"):
        paths = [str(record.get("path", "")) for record in inventory.get(key, [])]
        duplicates = sorted(
            path
            for path, count in collections.Counter(paths).items()
            if not path or count > 1
        )
        for path in duplicates:
            errors.append(f"[{owner_family(path)}] duplicate {key} path: {path!r}")
    scope = inventory.get("scope")
    if isinstance(scope, dict) and "rust_generators" in scope:
        declared_generators = scope.get("rust_generators")
        generator_files = {
            str(record.get("path", "")): record.get("generator_outputs")
            for record in inventory.get("rust_files", [])
            if "generator_outputs" in record
        }
        if not isinstance(declared_generators, list) or set(declared_generators) != set(
            generator_files
        ):
            errors.append("[generated] runtime generator file coverage mismatch")
        for path, outputs in generator_files.items():
            if (
                not isinstance(outputs, list)
                or not outputs
                or len(outputs) != len(set(outputs))
            ):
                errors.append(f"[generated] malformed generator outputs: {path}")
    for file_record in inventory.get("cpp_files", []):
        if (
            "behavioral_macros" not in file_record
            and "behavioral_macro_count" not in file_record
        ):
            continue
        path = str(file_record.get("path", ""))
        family = owner_family(path)
        macros = file_record.get("behavioral_macros")
        if not isinstance(macros, list):
            errors.append(f"[{family}] malformed behavioral macros: {path}")
            continue
        if file_record.get("behavioral_macro_count") != len(macros):
            errors.append(f"[{family}] behavioral macro count mismatch: {path}")
        macro_ids = [
            str(record.get("id", "")) for record in macros if isinstance(record, dict)
        ]
        duplicate_macro_ids = sorted(
            stable_id
            for stable_id, count in collections.Counter(macro_ids).items()
            if not stable_id or count > 1
        )
        for stable_id in duplicate_macro_ids:
            errors.append(
                f"[{family}] duplicate behavioral macro id: {stable_id!r} in {path}"
            )
        for macro in macros:
            if not isinstance(macro, dict):
                errors.append(f"[{family}] malformed behavioral macro record: {path}")
                continue
            start_line = macro.get("start_line")
            end_line = macro.get("end_line")
            digest = str(macro.get("content_sha256", ""))
            if (
                not str(macro.get("id", ""))
                or not str(macro.get("name", ""))
                or not isinstance(start_line, int)
                or not isinstance(end_line, int)
                or start_line < 1
                or end_line < start_line
                or re.fullmatch(r"[0-9a-f]{64}", digest) is None
            ):
                errors.append(f"[{family}] malformed behavioral macro record: {path}")
        if macros and file_record.get("classification") == "declaration-only":
            errors.append(
                f"[{family}] behavioral macros classified declaration-only: {path}"
            )
    for key in ("cpp_members", "rust_items"):
        records = inventory.get(key, [])
        counts = collections.Counter(str(record.get("id", "")) for record in records)
        for stable_id, count in sorted(counts.items()):
            if not stable_id or count > 1:
                family = next(
                    (
                        record_family(record)
                        for record in records
                        if str(record.get("id", "")) == stable_id
                    ),
                    "unknown",
                )
                errors.append(f"[{family}] duplicate {key} id: {stable_id!r}")
    cpp_paths = {
        str(record.get("path", "")) for record in inventory.get("cpp_files", [])
    }
    if not cpp_paths:
        cpp_paths = {
            stable_id.removeprefix("cpp:").split(":", 1)[0]
            for stable_id in (
                str(record.get("id", "")) for record in inventory.get("cpp_members", [])
            )
        }
    owner_policies = {
        str(policy.get("id", "")): policy
        for policy in inventory.get("cpp_owner_policies", [])
    }
    if len(owner_policies) != len(inventory.get("cpp_owner_policies", [])):
        errors.append("[cpp-owners] duplicate C++ owner policy id")
    for member in inventory.get("cpp_members", []):
        family = record_family(member)
        stable_id = str(member.get("id", ""))
        policy = owner_policies.get(str(member.get("owner_policy", "")))
        if policy is None:
            errors.append(f"[{family}] unmapped C++ member: {stable_id}")
            continue
        disposition = str(policy.get("disposition", ""))
        if disposition == "unmapped" or disposition not in {
            "mapped",
            "adapted",
            "tracked-gap",
            "generated",
            "header-only-unmapped",
            "declaration-only",
        }:
            errors.append(f"[{family}] unmapped C++ member: {stable_id}")
        if member.get("correspondence") == "unmapped":
            errors.append(f"[{family}] unmapped C++ member: {stable_id}")
        if member.get("correspondence") == "reviewed-gap" and not member.get(
            "gap_evidence"
        ):
            errors.append(
                f"[{family}] reviewed C++ member gap lacks evidence: {stable_id}"
            )
        if disposition in {"mapped", "adapted"} and not policy.get("rust_modules"):
            errors.append(
                f"[{family}] C++ member owner lacks Rust module correspondence: {stable_id}"
            )
    policies = {
        str(policy.get("id", "")): policy
        for policy in inventory.get("seam_policies", [])
    }
    if len(policies) != len(inventory.get("seam_policies", [])):
        errors.append("[seams] duplicate seam policy id")
    rust_paths = {
        str(record.get("path", "")) for record in inventory.get("rust_files", [])
    }
    for policy_id, policy in policies.items():
        provenance = str(policy.get("provenance", ""))
        binding_paths = policy.get("rust_paths", [])
        label = str(policy.get("adaptation") or policy.get("extension") or policy_id)
        if provenance not in {"adaptation", "extension"}:
            errors.append(f"[seams] invalid seam provenance for {label}")
        if (
            not isinstance(binding_paths, list)
            or not binding_paths
            or any(path not in rust_paths for path in binding_paths)
        ):
            errors.append(f"[seams] seam {label} lacks an exact Rust path binding")
        if not policy.get("item_selector"):
            errors.append(f"[seams] seam {label} lacks an item/module selector")
        owners = policy.get("baseline_cpp_owners", [])
        if not owners or any(owner not in cpp_paths for owner in owners):
            errors.append(f"[seams] seam {label} lacks an exact baseline C++ owner")
        if policy.get("allowed_call_direction") not in {
            "cpp-owner-to-rust-item",
            "rust-host-to-cpp-shaped-item",
        }:
            errors.append(f"[seams] seam {label} lacks an allowed call direction")
        if not policy.get("forbidden_baseline_effects"):
            errors.append(f"[seams] seam {label} lacks forbidden baseline effects")
        if not policy.get("evidence"):
            errors.append(f"[seams] seam {label} lacks required evidence")
    for item in inventory.get("rust_items", []):
        if item.get("provenance") not in {
            "adaptation",
            "baseline-port",
            "extension",
            "generated",
            "host-support",
            "unmapped",
        }:
            errors.append(
                f"[{record_family(item)}] invalid Rust provenance: "
                f"{item.get('provenance')} for {item.get('id', '')}"
            )
            continue
        if item.get("provenance") == "unmapped":
            errors.append(
                f"[{record_family(item)}] unmapped Rust item: {item.get('id', '')}"
            )
            continue
        if item.get("provenance") not in {"adaptation", "extension"}:
            continue
        family = record_family(item)
        stable_id = item.get("id", "")
        policy = policies.get(str(item.get("seam_policy", "")), item)
        if "seam_policy" in item and str(item["seam_policy"]) not in policies:
            errors.append(f"[{family}] missing seam policy for {stable_id}")
        elif "seam_policy" not in item:
            owners = policy.get("baseline_cpp_owners", [])
            if not owners or any(owner not in cpp_paths for owner in owners):
                errors.append(
                    f"[{family}] {item.get('provenance')} {stable_id} lacks an exact baseline C++ owner"
                )
            if policy.get("allowed_call_direction") not in {
                "cpp-owner-to-rust-item",
                "rust-host-to-cpp-shaped-item",
            }:
                errors.append(
                    f"[{family}] adaptation {stable_id} lacks an allowed call direction"
                )
            if not policy.get("forbidden_baseline_effects"):
                errors.append(
                    f"[{family}] adaptation {stable_id} lacks forbidden baseline effects"
                )
            if not policy.get("evidence"):
                errors.append(
                    f"[{family}] adaptation {stable_id} lacks required evidence"
                )
    return errors


def indexed(records: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    return {str(record.get("id", "")): record for record in records}


def inventory_differences(
    expected: dict[str, Any], actual: dict[str, Any]
) -> list[str]:
    errors = []
    for key in ("schema", "scope", "summary"):
        if expected.get(key) != actual.get(key):
            errors.append(
                f"[inventory] changed {key}: {expected.get(key)!r} -> {actual.get(key)!r}"
            )
    for key, noun in (("cpp_members", "C++ member"), ("rust_items", "Rust item")):
        old = indexed(expected.get(key, []))
        new = indexed(actual.get(key, []))
        for stable_id in sorted(new.keys() - old.keys()):
            family = record_family(new[stable_id])
            errors.append(f"[{family}] new {noun}: {stable_id}")
        for stable_id in sorted(old.keys() - new.keys()):
            family = record_family(old[stable_id])
            errors.append(f"[{family}] removed {noun}: {stable_id}")
        for stable_id in sorted(old.keys() & new.keys()):
            if old[stable_id] != new[stable_id]:
                family = record_family(new[stable_id])
                errors.append(f"[{family}] changed {noun}: {stable_id}")
    old_cpp_files = {record["path"]: record for record in expected.get("cpp_files", [])}
    new_cpp_files = {record["path"]: record for record in actual.get("cpp_files", [])}
    old_rust_files = {
        record["path"]: record for record in expected.get("rust_files", [])
    }
    new_rust_files = {record["path"]: record for record in actual.get("rust_files", [])}
    for label, old, new in (
        ("C++ source", old_cpp_files, new_cpp_files),
        ("Rust source", old_rust_files, new_rust_files),
    ):
        for path in sorted(new.keys() - old.keys()):
            errors.append(f"[{owner_family(path)}] new {label}: {path}")
        for path in sorted(old.keys() - new.keys()):
            errors.append(f"[{owner_family(path)}] removed {label}: {path}")
        for path in sorted(old.keys() & new.keys()):
            if old[path] != new[path]:
                errors.append(f"[{owner_family(path)}] changed {label}: {path}")
    if expected.get("upstream_ref") != actual.get("upstream_ref"):
        errors.append(
            "[upstream] changed inventory pin: "
            f"{expected.get('upstream_ref')} -> {actual.get('upstream_ref')}"
        )
    if expected.get("seam_policies", []) != actual.get("seam_policies", []):
        errors.append("[seams] changed adaptation/extension policy inventory")
    if expected.get("cpp_owner_policies", []) != actual.get("cpp_owner_policies", []):
        errors.append("[cpp-owners] changed member owner correspondence inventory")
    return errors


def check_snapshot(path: pathlib.Path, actual: dict[str, Any]) -> list[str]:
    if not path.exists():
        return [f"inventory snapshot does not exist: {path}"]
    expected = json.loads(path.read_text())
    return (
        validate_inventory(expected)
        + validate_inventory(actual)
        + inventory_differences(expected, actual)
    )


def render_json(inventory: dict[str, Any]) -> str:
    # One record per line keeps a 20k-record source inventory reviewable in
    # ordinary Git diffs without sacrificing named fields or JSON tooling.
    lines = ["{"]
    keys = sorted(inventory)
    for key_index, key in enumerate(keys):
        value = inventory[key]
        comma = "," if key_index + 1 < len(keys) else ""
        encoded_key = json.dumps(key)
        if isinstance(value, list):
            lines.append(f"  {encoded_key}: [")
            for record_index, record in enumerate(value):
                record_comma = "," if record_index + 1 < len(value) else ""
                lines.append(
                    "    "
                    + json.dumps(record, sort_keys=True, separators=(",", ":"))
                    + record_comma
                )
            lines.append(f"  ]{comma}")
        else:
            lines.append(
                f"  {encoded_key}: "
                + json.dumps(value, sort_keys=True, separators=(",", ":"))
                + comma
            )
    lines.append("}")
    return "\n".join(lines) + "\n"


def resolve_upstream_ref(repo_root: pathlib.Path) -> str:
    return str(
        tomllib.loads((repo_root / "port-manifest.toml").read_text())["upstream_ref"]
    )


def git_head(root: pathlib.Path) -> str:
    return subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def git_worktree_clean(root: pathlib.Path) -> bool:
    changed = subprocess.run(
        [
            "git",
            "status",
            "--porcelain=v1",
            "--untracked-files=all",
            "--",
            *UPSTREAM_SOURCE_ROOTS,
        ],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    ignored = subprocess.run(
        [
            "git",
            "ls-files",
            "--others",
            "--ignored",
            "--exclude-standard",
            "-z",
            "--",
            *UPSTREAM_SOURCE_ROOTS,
        ],
        cwd=root,
        check=True,
        capture_output=True,
    ).stdout.split(b"\0")
    ignored_inventory_sources = any(
        pathlib.PurePosixPath(path.decode(errors="surrogateescape")).suffix
        in CPP_SUFFIXES
        for path in ignored
        if path
    )
    return not changed and not ignored_inventory_sources


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument("--upstream-root", type=pathlib.Path, required=True)
    parser.add_argument("--snapshot", type=pathlib.Path, required=True)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    repo_root = args.repo_root.resolve()
    upstream_root = args.upstream_root.resolve()
    upstream_ref = resolve_upstream_ref(repo_root)
    configuration_errors = validate_configuration(repo_root)
    if configuration_errors:
        print("runtime behavior inventory configuration invalid:", file=sys.stderr)
        print("\n".join(configuration_errors), file=sys.stderr)
        return 1
    upstream_head = git_head(upstream_root)
    if upstream_head != upstream_ref:
        print(
            "runtime behavior inventory upstream mismatch: "
            f"expected {upstream_ref}, found {upstream_head}",
            file=sys.stderr,
        )
        return 1
    if not git_worktree_clean(upstream_root):
        print(
            "runtime behavior inventory upstream worktree is dirty; "
            "commit, stash, or remove local changes before generating evidence",
            file=sys.stderr,
        )
        return 1
    inventory = build_inventory(repo_root, upstream_root, upstream_ref)
    expected = json.loads(args.snapshot.read_text()) if args.snapshot.exists() else None
    approve_host_support(inventory, expected, args.write)
    approve_header_gaps(inventory, expected, args.write)
    validation = validate_inventory(inventory)
    if validation:
        print("runtime behavior inventory validation failed:", file=sys.stderr)
        print("\n".join(validation), file=sys.stderr)
        return 1
    if args.write:
        args.snapshot.write_text(render_json(inventory))
        print(
            "runtime behavior inventory written: "
            f"{inventory['summary']['cpp_members']} C++ members, "
            f"{inventory['summary']['rust_items']} Rust items"
        )
        return 0
    errors = check_snapshot(args.snapshot, inventory)
    if errors:
        print("runtime behavior inventory is stale:", file=sys.stderr)
        print("\n".join(errors[:200]), file=sys.stderr)
        if len(errors) > 200:
            print(f"... {len(errors) - 200} more differences", file=sys.stderr)
        return 1
    print(
        "runtime behavior inventory current: "
        f"{inventory['summary']['cpp_members']} C++ members, "
        f"{inventory['summary']['rust_items']} Rust items"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
