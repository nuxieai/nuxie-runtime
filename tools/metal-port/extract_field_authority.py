#!/usr/bin/env python3
"""Derive renderer field authority from Clang's AST, never from the field ledger."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import pathlib
import re
import shutil
import subprocess
import tempfile
from dataclasses import dataclass


@dataclass(frozen=True)
class OwnerSpec:
    upstream_file: str
    cpp_type: str
    ast_qualified_type: str
    source_sha256: str
    disposition: str = "required"
    exclusion_reason: str = "-"


@dataclass(frozen=True)
class SourceSpec:
    upstream_file: str
    source_sha256: str


def read_owner_specs(path: pathlib.Path) -> dict[str, OwnerSpec]:
    with path.open(encoding="utf-8", newline="") as source:
        rows = list(csv.DictReader(source, delimiter="\t"))
    specs = {
        row["ast_qualified_type"]: OwnerSpec(**row)
        for row in rows
    }
    if len(specs) != len(rows):
        raise RuntimeError("field-owner input repeats an AST-qualified type")
    return specs


def read_source_specs(path: pathlib.Path) -> dict[str, SourceSpec]:
    with path.open(encoding="utf-8", newline="") as source:
        rows = list(csv.DictReader(source, delimiter="\t"))
    specs = {row["upstream_file"]: SourceSpec(**row) for row in rows}
    if len(specs) != len(rows):
        raise RuntimeError("field-source input repeats an upstream file")
    return specs


def _clang_command(upstream_root: pathlib.Path, stub_root: pathlib.Path) -> list[str]:
    clang = shutil.which("xcrun")
    if clang:
        command = [clang, "--sdk", "macosx", "clang++"]
    else:
        clangxx = shutil.which("clang++")
        if not clangxx:
            raise RuntimeError("Clang is required to derive renderer field authority")
        command = [clangxx]
    main = upstream_root / "renderer/src/metal/render_context_metal_impl.mm"
    include_roots = (
        stub_root,
        upstream_root / "include",
        upstream_root / "renderer/include",
        upstream_root / "renderer/src",
        upstream_root / "renderer/src/shaders",
        upstream_root / "decoders/include",
    )
    return command + [
        "-x",
        "objective-c++",
        "-std=c++17",
        "-fsyntax-only",
        "-Xclang",
        "-ast-dump",
        *(argument for root in include_roots for argument in ("-I", str(root))),
        "-DRIVE_CANVAS",
        "-DWITH_RIVE_TOOLS",
        "-DDEBUG",
        "-D__EMSCRIPTEN__",
        str(main),
    ]


def _json_clang_command(upstream_root: pathlib.Path, stub_root: pathlib.Path) -> list[str]:
    command = _clang_command(upstream_root, stub_root)
    command[command.index("-ast-dump")] = "-ast-dump=json"
    return command


def _write_hermetic_stubs(stub_root: pathlib.Path, upstream_root: pathlib.Path) -> None:
    (stub_root / "emscripten.h").write_text(
        "#pragma once\n#define EMSCRIPTEN_KEEPALIVE\n", encoding="utf-8"
    )
    generated = stub_root / "generated/shaders"
    generated.mkdir(parents=True)
    implementation = (upstream_root / "renderer/src/metal/render_context_metal_impl.mm").read_text()
    macros = sorted(set(re.findall(r"\bGLSL_[A-Za-z0-9_]+", implementation)))
    exports = "#pragma once\n" + "".join(
        f'#define {macro} "{macro[5:]}"\n' for macro in macros
    )
    (generated / "color_ramp.glsl.exports.h").write_text(exports, encoding="utf-8")
    (generated / "tessellate.glsl.exports.h").write_text(exports, encoding="utf-8")
    (generated / "rive_pls_macosx.metallib.c").write_text(
        "static const unsigned char rive_pls_macosx_metallib[] = {0};\n"
        "static const unsigned int rive_pls_macosx_metallib_len = 1;\n", encoding="utf-8"
    )


def _line_number(text: str, source_name: str) -> int | None:
    explicit = re.findall(rf"{re.escape(source_name)}:(\d+):", text)
    if explicit:
        return int(explicit[-1])
    line = re.search(r"(?:<|, )line:(\d+)(?::\d+)?", text)
    return int(line.group(1)) if line else None


def _configuration(source_line: str, enclosing_directives: tuple[str, ...]) -> str:
    if "RIVE_DEBUG_CODE" in source_line:
        return "DEBUG"
    joined = " ".join(enclosing_directives)
    for token in ("WITH_RIVE_TOOLS", "RIVE_CANVAS", "__EMSCRIPTEN__"):
        if re.search(rf"\b{re.escape(token)}\b", joined):
            return token
    return "all"


def _directive_context(lines: list[str], line_number: int) -> tuple[str, ...]:
    stack: list[str] = []
    for current, line in enumerate(lines[:line_number], 1):
        stripped = line.strip()
        if re.match(r"#(?:if\s|ifdef\s|ifndef\s)", stripped):
            stack.append(stripped)
        elif re.match(r"#(?:elif\s|else(?:\s|$))", stripped):
            if stack:
                stack[-1] = stripped
        elif re.match(r"#endif(?:\s|$)", stripped):
            if stack:
                stack.pop()
    return tuple(stack)


def _discover_owner_records(
    ast: dict[str, object],
    upstream_root: pathlib.Path,
    source_specs: dict[str, SourceSpec],
) -> dict[str, str]:
    """Return every state-bearing record declared in the pinned source set.

    Clang's text AST abbreviates repeated source paths as relative ``line:``
    locations.  Its JSON AST retains byte offsets, so the source can be
    recovered independently even when ``loc.file`` is omitted.
    """
    source_locations: dict[str, tuple[bytes, dict[int, tuple[int, int]]]] = {}
    for source in source_specs:
        data = (upstream_root / source).read_bytes()
        starts = [0]
        starts.extend(match.end() for match in re.finditer(b"\n", data))
        source_locations[source] = (
            data,
            {
                line: (start, starts[line] if line < len(starts) else len(data) + 1)
                for line, start in enumerate(starts, 1)
            },
        )

    def declared_source(node: dict[str, object]) -> str | None:
        location = node.get("loc", {})
        if not isinstance(location, dict):
            return None
        filename = location.get("file")
        if isinstance(filename, str):
            try:
                relative = str(pathlib.Path(filename).relative_to(upstream_root))
            except ValueError:
                return None
            return relative if relative in source_specs else None
        line = location.get("line")
        offset = location.get("offset")
        token_length = location.get("tokLen")
        if not isinstance(line, int) or not isinstance(offset, int):
            return None
        expected = node.get("name")
        candidates = [
            source
            for source, (data, lines) in source_locations.items()
            if line in lines
            and lines[line][0] <= offset < lines[line][1]
            and isinstance(token_length, int)
            and (
                not isinstance(expected, str)
                or data[offset : offset + token_length].decode(errors="replace") == expected
            )
        ]
        return candidates[0] if len(candidates) == 1 else None

    discovered: dict[str, str] = {}

    def visit(
        node: dict[str, object], parents: tuple[str, ...], source_context: str | None
    ) -> None:
        kind = node.get("kind")
        name = node.get("name")
        next_parents = parents
        next_source_context = source_context
        if kind in {"NamespaceDecl", "ClassTemplateDecl"} and isinstance(name, str):
            next_parents += (name,)
            if kind == "ClassTemplateDecl":
                next_source_context = declared_source(node) or source_context
        elif (
            kind == "CXXRecordDecl"
            and node.get("completeDefinition")
            and node.get("tagUsed") in {"class", "struct", "union"}
            and not (
                isinstance(node.get("definitionData"), dict)
                and node["definitionData"].get("isLambda")
            )
        ):
            record_name = name if isinstance(name, str) else "<anonymous>"
            qualified = "::".join((*parents, record_name))
            children = node.get("inner", [])
            if not isinstance(children, list):
                children = []
            if qualified.startswith("rive::") and any(
                isinstance(child, dict) and child.get("kind") == "FieldDecl"
                for child in children
            ):
                source = declared_source(node) or source_context
                if source is not None:
                    discovered[qualified] = source
            next_parents += (record_name,)
            next_source_context = declared_source(node) or source_context
        children = node.get("inner", [])
        if isinstance(children, list):
            for child in children:
                if isinstance(child, dict):
                    visit(child, next_parents, next_source_context)

    visit(ast, (), None)
    return discovered


def extract(
    upstream_root: pathlib.Path,
    owner_spec_path: pathlib.Path,
    source_spec_path: pathlib.Path,
) -> list[tuple[str, str, str, str, int, str]]:
    specs = read_owner_specs(owner_spec_path)
    source_specs = read_source_specs(source_spec_path)
    sources: dict[str, list[str]] = {}
    for spec in source_specs.values():
        source_path = upstream_root / spec.upstream_file
        digest = hashlib.sha256(source_path.read_bytes()).hexdigest()
        if digest != spec.source_sha256:
            raise RuntimeError(
                f"field-owner input hash drifted for {spec.upstream_file}: {digest}"
            )
        sources.setdefault(
            spec.upstream_file,
            source_path.read_text(encoding="utf-8").splitlines(),
        )

    with tempfile.TemporaryDirectory(prefix="metal-field-ast-") as temporary:
        stub_root = pathlib.Path(temporary)
        _write_hermetic_stubs(stub_root, upstream_root)
        process = subprocess.run(
            _clang_command(upstream_root, stub_root),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        json_process = subprocess.run(
            _json_clang_command(upstream_root, stub_root),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
    if process.returncode:
        raise RuntimeError("Clang field extraction failed:\n" + process.stderr)
    if json_process.returncode:
        raise RuntimeError("Clang owner discovery failed:\n" + json_process.stderr)

    discovered = _discover_owner_records(
        json.loads(json_process.stdout), upstream_root, source_specs
    )
    omitted = sorted(set(discovered) - set(specs))
    invented = sorted(set(specs) - set(discovered))
    if omitted:
        raise RuntimeError("field-owner authority omits Clang records: " + ", ".join(omitted))
    if invented:
        raise RuntimeError("field-owner authority invents Clang records: " + ", ".join(invented))

    # The text AST preserves lexical nesting while remaining small enough to
    # regenerate during every campaign check. Fields are collected only from
    # the 68 independently pinned owner declarations, then deduplicated across
    # template instantiations by source identity.
    stack: list[tuple[int, str, str, str | None, int | None, int | None]] = []
    declarations: dict[tuple[str, str, str], tuple[str, int, str]] = {}
    for ast_line in process.stdout.splitlines():
        match = re.search(
            r"(NamespaceDecl|ClassTemplateDecl|CXXRecordDecl|FieldDecl) ", ast_line
        )
        if match is None:
            continue
        kind = match.group(1)
        depth = match.start() // 2
        while stack and stack[-1][0] >= depth:
            stack.pop()

        if kind == "NamespaceDecl":
            tokens = ast_line.strip().split()
            name = tokens[-2] if tokens[-1] == "nested" else tokens[-1]
            stack.append((depth, kind, name, None, None, None))
            continue
        if kind == "ClassTemplateDecl":
            stack.append((depth, kind, ast_line.strip().split()[-1], None, None, None))
            continue
        if kind == "CXXRecordDecl":
            named = re.search(
                r"\b(?:class|struct|union) ([A-Za-z_]\w*) definition\b", ast_line
            )
            name = named.group(1) if named else None
            if name is None and re.search(r"\bunion definition\b", ast_line):
                name = "<anonymous>"
            if name is None:
                continue
            qualified = "::".join(
                [
                    entry[2]
                    for entry in stack
                    if entry[1]
                    in {"NamespaceDecl", "ClassTemplateDecl", "CXXRecordDecl"}
                ]
                + [name]
            )
            spec = specs.get(qualified)
            begin = _line_number(ast_line.split(",", 1)[0], spec.upstream_file) if spec else None
            relative_lines = re.findall(r"\bline:(\d+)(?::\d+)?", ast_line)
            end = (
                max(map(int, relative_lines))
                if spec and relative_lines
                else (_line_number(ast_line, spec.upstream_file) if spec else None)
            )
            stack.append((depth, kind, name, qualified if spec else None, begin, end))
            continue

        record = next(
            (entry for entry in reversed(stack) if entry[1] == "CXXRecordDecl"),
            None,
        )
        if record is None or record[3] is None:
            continue
        if depth != record[0] + 1:
            continue
        spec = specs[record[3]]
        before_type = ast_line.rsplit(" '", 1)[0]
        candidate = before_type.split()[-1]
        field = (
            candidate
            if re.fullmatch(r"[A-Za-z_]\w*", candidate) and candidate != "implicit"
            else None
        )
        line_number = _line_number(ast_line, spec.upstream_file)
        if field is None:
            anonymous_line = re.search(
                rf"{re.escape(spec.upstream_file)}:(\d+):", ast_line
            )
            if anonymous_line:
                line_number = int(anonymous_line.group(1))
                field = f"<anonymous-union@{line_number}>"
        source_lines = sources[spec.upstream_file]
        token = "union" if field and field.startswith("<anonymous-union@") else field
        if token is None:
            raise RuntimeError(
                f"Clang emitted an unidentifiable field for {spec.cpp_type}: {ast_line}"
            )
        if (
            line_number is None
            or line_number > len(source_lines)
            or re.search(rf"\b{re.escape(token)}\b", source_lines[line_number - 1]) is None
        ):
            start = record[4] or 1
            end = min(record[5] or len(source_lines), len(source_lines))
            hits = [
                number
                for number in range(start, end + 1)
                if re.search(rf"\b{re.escape(token)}\b", source_lines[number - 1])
            ]
            if not hits:
                raise RuntimeError(
                    f"cannot locate Clang field {spec.cpp_type}.{field} in {spec.upstream_file}"
                )
            line_number = hits[-1]
        configuration = _configuration(
            source_lines[line_number - 1],
            _directive_context(source_lines, line_number),
        )
        key = (spec.upstream_file, spec.cpp_type, field)
        previous = declarations.get(key)
        quoted_types = re.findall(r"'([^']*)'", ast_line)
        declared_type = quoted_types[0] if quoted_types else ""
        declared_type = declared_type.replace(str(upstream_root) + "/", "")
        if not declared_type:
            raise RuntimeError(f"Clang omitted type for {spec.cpp_type}.{field}")
        value = (declared_type, line_number, configuration)
        if previous is not None:
            if previous[1:] != value[1:]:
                raise RuntimeError(f"Clang field identity is ambiguous: {key}")
            continue
        declarations[key] = value

    missing_owners = sorted(
        spec.cpp_type
        for qualified, spec in specs.items()
        if not any(key[1] == spec.cpp_type for key in declarations)
    )
    if missing_owners:
        raise RuntimeError("Clang omitted configured field owners: " + ", ".join(missing_owners))
    return sorted(
        (*key, declared_type, line, configuration)
        for key, (declared_type, line, configuration) in declarations.items()
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--upstream-root", type=pathlib.Path, required=True)
    parser.add_argument("--owners", type=pathlib.Path, required=True)
    parser.add_argument("--sources", type=pathlib.Path, required=True)
    arguments = parser.parse_args()
    writer = csv.writer(__import__("sys").stdout, delimiter="\t", lineterminator="\n")
    writer.writerow(
        ["upstream_file", "cpp_type", "cpp_field", "cpp_declared_type", "declaration_line", "configuration"]
    )
    writer.writerows(extract(arguments.upstream_root, arguments.owners, arguments.sources))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
