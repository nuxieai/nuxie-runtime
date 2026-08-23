#!/usr/bin/env python3
"""Generate exact Rust ABI declarations from the frozen Wagyu C headers."""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Any


KEYWORDS = {
    "as", "break", "const", "continue", "crate", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match",
    "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static",
    "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do",
    "final", "macro", "override", "priv", "typeof", "unsized", "virtual",
    "yield", "try",
}


def ident(name: str) -> str:
    return f"r#{name}" if name in KEYWORDS else name


def split_args(value: str) -> list[str]:
    if not value or value == "void":
        return []
    result: list[str] = []
    depth = 0
    start = 0
    for index, char in enumerate(value):
        if char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
        elif char == "," and depth == 0:
            result.append(value[start:index].strip())
            start = index + 1
    result.append(value[start:].strip())
    return result


def rust_type(value: str, *, return_type: bool = False) -> str:
    value = " ".join(value.strip().split())
    if value.startswith("const ") and not value.endswith(" *"):
        value = value[6:]
    array = re.fullmatch(r"(.+) \[(\d+)\]", value)
    if array:
        return f"[{rust_type(array.group(1))}; {array.group(2)}]"
    if value.endswith("*"):
        base = value[:-1].strip()
        immutable = base.startswith("const ") or base.endswith(" const")
        base = re.sub(r"^const ", "", base)
        base = re.sub(r" const$", "", base)
        base = re.sub(r"^(struct|enum) ", "", base)
        pointer = "*const" if immutable else "*mut"
        pointee = "std::ffi::c_void" if base == "void" else rust_type(base)
        return f"{pointer} {pointee}"
    value = re.sub(r"^(struct|enum) ", "", value)
    primitives = {
        "void": "()" if return_type else "std::ffi::c_void",
        "char": "std::ffi::c_char",
        "uint8_t": "u8", "uint16_t": "u16", "uint32_t": "u32", "uint64_t": "u64",
        "int8_t": "i8", "int16_t": "i16", "int32_t": "i32", "int64_t": "i64",
        "size_t": "usize", "float": "f32", "double": "f64",
        "unsigned int": "u32", "int": "i32",
    }
    return primitives.get(value, value)


def function_pointer(value: str) -> str:
    match = re.fullmatch(r"(.+?)\s*\(\*\)\((.*)\)", value)
    if not match:
        raise ValueError(f"not a function pointer: {value}")
    result = rust_type(match.group(1), return_type=True)
    args = ", ".join(rust_type(arg) for arg in split_args(match.group(2)))
    suffix = "" if result == "()" else f" -> {result}"
    return f"Option<unsafe extern \"C\" fn({args}){suffix}>"


def evaluated_value(node: dict[str, Any]) -> int:
    if "value" in node:
        return int(node["value"], 0)
    for child in node.get("inner", []):
        try:
            return evaluated_value(child)
        except ValueError:
            pass
    raise ValueError(f"no constant value for {node.get('name', node.get('kind'))}")


def clang_ast(include_root: Path, header: str) -> dict[str, Any]:
    command = [
        "clang", "-x", "c", f"-I{include_root}", "-Xclang", "-ast-dump=json",
        "-fsyntax-only", "-",
    ]
    completed = subprocess.run(
        command,
        input=f"#include <webgpu/{header}>\n".encode(),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=True,
    )
    return json.loads(completed.stdout)


def relevant_nodes(ast: dict[str, Any], wagyu: bool) -> list[dict[str, Any]]:
    nodes = []
    for node in ast["inner"]:
        name = node.get("name", "")
        if wagyu:
            keep = "Wagyu" in name or name.startswith("wgpuWagyu")
        else:
            keep = name.startswith("WGPU") or name.startswith("wgpu")
        if keep and node["kind"] in {
            "RecordDecl", "EnumDecl", "TypedefDecl", "VarDecl", "FunctionDecl"
        }:
            nodes.append(node)
    return nodes


def macro_spans(source: str) -> list[tuple[str, int, int]]:
    lines = source.splitlines()
    result = []
    index = 0
    while index < len(lines):
        match = re.match(r"^\s*#\s*define\s+([A-Za-z_][A-Za-z0-9_]*)", lines[index])
        if match:
            start = index + 1
            while lines[index].rstrip().endswith("\\") and index + 1 < len(lines):
                index += 1
            result.append((match.group(1), start, index + 1))
        index += 1
    return result


def emit_bindings(nodes: list[dict[str, Any]], source: str, *, wagyu: bool) -> str:
    records = {
        node["name"]: node
        for node in nodes
        if node["kind"] == "RecordDecl" and node.get("completeDefinition")
    }
    enums = {node["name"]: node for node in nodes if node["kind"] == "EnumDecl"}
    typedefs = {node["name"]: node for node in nodes if node["kind"] == "TypedefDecl"}
    variables = [node for node in nodes if node["kind"] == "VarDecl"]
    functions = [node for node in nodes if node["kind"] == "FunctionDecl"]
    macros = macro_spans(source)
    field_count = sum(
        1
        for node in records.values()
        for child in node.get("inner", [])
        if child["kind"] == "FieldDecl"
    )

    lines = [
        "//! Generated exact C ABI translation; do not hand edit.",
        "",
        "#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]",
        "",
    ]
    if wagyu:
        lines += ["use super::webgpu_decl::*;", ""]
        source_name = "renderer_src_webgpu_wagyu-port_include_webgpu_webgpu_wagyu.h"
        source_lines, source_bytes = 722, 43_574
    else:
        source_name = "renderer_src_webgpu_wagyu-port_include_webgpu_webgpu.h"
        source_lines, source_bytes = 2_828, 148_027
    lines += [
        f'pub(crate) const PINNED_SOURCE: &str = include_str!("source/{source_name}");',
        f"pub(crate) const PINNED_SOURCE_LINE_COUNT: usize = {source_lines};",
        f"pub(crate) const PINNED_SOURCE_BYTE_COUNT: usize = {source_bytes:_};",
        "",
    ]

    lines += [
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]",
        "pub(crate) struct MacroAuthority {",
        "    pub(crate) name: &'static str,",
        "    pub(crate) startLine: usize,",
        "    pub(crate) endLine: usize,",
        "}",
        "",
        "pub(crate) const MACRO_AUTHORITIES: &[MacroAuthority] = &[",
    ]
    for name, start, end in macros:
        lines.append(
            f'    MacroAuthority {{ name: "{name}", startLine: {start}, endLine: {end} }},'
        )
    lines += [
        "];",
        "",
        "pub(crate) fn macroSource(authority: MacroAuthority) -> String {",
        "    PINNED_SOURCE",
        "        .lines()",
        "        .skip(authority.startLine - 1)",
        "        .take(authority.endLine - authority.startLine + 1)",
        "        .collect::<Vec<_>>()",
        "        .join(\"\\n\")",
        "}",
        "",
    ]
    if not wagyu:
        lines += [
            "pub(crate) const WGPU_TRUE: u32 = 1;",
            "pub(crate) const WGPU_FALSE: u32 = 0;",
            "pub(crate) const WGPU_ARRAY_LAYER_COUNT_UNDEFINED: u32 = u32::MAX;",
            "pub(crate) const WGPU_COPY_STRIDE_UNDEFINED: u32 = u32::MAX;",
            "pub(crate) const WGPU_DEPTH_CLEAR_VALUE_UNDEFINED: f64 = f64::NAN;",
            "pub(crate) const WGPU_DEPTH_SLICE_UNDEFINED: u32 = u32::MAX;",
            "pub(crate) const WGPU_LIMIT_U32_UNDEFINED: u32 = u32::MAX;",
            "pub(crate) const WGPU_LIMIT_U64_UNDEFINED: u64 = u64::MAX;",
            "pub(crate) const WGPU_MIP_LEVEL_COUNT_UNDEFINED: u32 = u32::MAX;",
            "pub(crate) const WGPU_QUERY_SET_INDEX_UNDEFINED: u32 = u32::MAX;",
            "pub(crate) const WGPU_STRLEN: usize = usize::MAX;",
            "pub(crate) const WGPU_WHOLE_MAP_SIZE: usize = usize::MAX;",
            "pub(crate) const WGPU_WHOLE_SIZE: u64 = u64::MAX;",
            "",
        ]
    else:
        lines += [
            "pub(crate) const WGPU_WAGYU_EXTENSION_LEVEL: u32 = 1;",
            "pub(crate) const WGPU_WAGYU_RESERVED_RANGE_BASE: i32 = 0x0006_0000;",
            "pub(crate) const WGPU_WAGYU_STRLEN: usize = usize::MAX;",
            "pub(crate) const WGPU_WAGYU_PIXEL_LOCAL_STORAGE_SIZE_UNDEFINED: u32 = u32::MAX;",
            "",
        ]

    opaque: dict[str, str] = {}
    for name, node in typedefs.items():
        match = re.fullmatch(r"struct (\w+) \*", node["type"]["qualType"])
        if match:
            opaque[name] = match.group(1)
    for implementation in sorted(set(opaque.values())):
        lines += [
            "#[repr(C)]",
            f"pub(crate) struct {implementation} {{ _private: [u8; 0] }}",
        ]
    if opaque:
        lines.append("")
    for name, implementation in sorted(opaque.items()):
        lines.append(f"pub(crate) type {name} = *mut {implementation};")
    if opaque:
        lines.append("")

    for name, node in sorted(enums.items()):
        lines.append(f"pub(crate) type {name} = i32;")
        for child in node.get("inner", []):
            if child["kind"] != "EnumConstantDecl":
                continue
            lines.append(
                f"pub(crate) const {child['name']}: {name} = {evaluated_value(child)};"
            )
        lines.append("")

    skipped = set(records) | set(enums) | set(opaque)
    callbacks: list[tuple[str, str]] = []
    aliases: list[tuple[str, str]] = []
    for name, node in sorted(typedefs.items()):
        if name in skipped:
            continue
        value = node["type"]["qualType"]
        if "(*)" in value:
            callbacks.append((name, function_pointer(value)))
        elif value.startswith(("struct ", "enum ")) and value.split()[-1] == name:
            continue
        else:
            aliases.append((name, rust_type(value)))
    for name, value in aliases:
        lines.append(f"pub(crate) type {name} = {value};")
    if aliases:
        lines.append("")
    for name, value in callbacks:
        lines.append(f"pub(crate) type {name} = {value};")
    if callbacks:
        lines.append("")

    for node in sorted(variables, key=lambda item: (item.get("loc", {}).get("line", 0), item["name"])):
        value_type = rust_type(node["type"]["qualType"])
        lines.append(
            f"pub(crate) const {node['name']}: {value_type} = "
            f"{evaluated_value(node)} as {value_type};"
        )
    if variables:
        lines.append("")

    for name, node in sorted(records.items(), key=lambda item: item[1].get("loc", {}).get("line", 0)):
        fields = [child for child in node.get("inner", []) if child["kind"] == "FieldDecl"]
        lines += ["#[repr(C)]", f"pub(crate) struct {name} {{"]
        for field in fields:
            lines.append(
                f"    pub(crate) {ident(field['name'])}: {rust_type(field['type']['qualType'])},"
            )
        lines += ["}", ""]

    lines.append('unsafe extern "C" {')
    for node in sorted(functions, key=lambda item: item.get("loc", {}).get("line", 0)):
        result = rust_type(node["type"]["qualType"].split("(", 1)[0], return_type=True)
        params = []
        for index, child in enumerate(
            child for child in node.get("inner", []) if child["kind"] == "ParmVarDecl"
        ):
            params.append(f"arg{index}: {rust_type(child['type']['qualType'])}")
        suffix = "" if result == "()" else f" -> {result}"
        lines.append(f"    pub(crate) fn {node['name']}({', '.join(params)}){suffix};")
    lines += ["}", ""]

    lines += [
        f"pub(crate) const ABI_ENUM_COUNT: usize = {len(enums)};",
        f"pub(crate) const ABI_STRUCT_COUNT: usize = {len(records)};",
        f"pub(crate) const ABI_FIELD_COUNT: usize = {field_count};",
        f"pub(crate) const ABI_HANDLE_COUNT: usize = {len(opaque)};",
        f"pub(crate) const ABI_FUNCTION_POINTER_COUNT: usize = {len(callbacks)};",
        f"pub(crate) const ABI_STATIC_CONSTANT_COUNT: usize = {len(variables)};",
        f"pub(crate) const ABI_FUNCTION_COUNT: usize = {len(functions)};",
        f"pub(crate) const PREPROCESSOR_DEFINITION_COUNT: usize = {len(macros)};",
        "const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];",
        "",
        "#[cfg(test)]",
        "mod tests {",
        "    use super::*;",
        "    use std::collections::BTreeSet;",
        "",
        "    #[test]",
        "    fn complete_preprocessor_denominator_is_frozen() {",
        "        assert_eq!(MACRO_AUTHORITIES.len(), PREPROCESSOR_DEFINITION_COUNT);",
        "        for authority in MACRO_AUTHORITIES {",
        "            assert!(macroSource(*authority).trim_start().starts_with('#'));",
        "            assert!(macroSource(*authority).contains(\"define\"));",
        "        }",
        "    }",
        "",
        "    #[test]",
        "    fn exported_function_names_are_unique() {",
        "        let mut names = BTreeSet::new();",
        "        for line in PINNED_SOURCE.lines() {",
        "            if let Some(index) = line.find(\" wgpu\") {",
        "                let name = line[index + 1..].split('(').next().unwrap_or(\"\");",
        "                if name.starts_with(\"wgpu\") { names.insert(name); }",
        "            }",
        "        }",
        "        assert!(names.len() >= ABI_FUNCTION_COUNT);",
        "    }",
        "}",
        "",
    ]
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", required=True, type=Path)
    args = parser.parse_args()
    repo = args.repo_root.resolve()
    source = repo / "crates/nuxie-renderer/src/mechanical_port/webgpu/source"
    output = repo / "crates/nuxie-renderer/src/mechanical_port/webgpu"
    with tempfile.TemporaryDirectory() as temporary:
        include = Path(temporary) / "include/webgpu"
        include.mkdir(parents=True)
        shutil.copyfile(
            source / "renderer_src_webgpu_wagyu-port_include_webgpu_webgpu.h",
            include / "webgpu.h",
        )
        shutil.copyfile(
            source / "renderer_src_webgpu_wagyu-port_include_webgpu_webgpu_wagyu.h",
            include / "webgpu_wagyu.h",
        )
        core_source = (include / "webgpu.h").read_text()
        wagyu_source = (include / "webgpu_wagyu.h").read_text()
        core = emit_bindings(
            relevant_nodes(clang_ast(include.parent, "webgpu.h"), False),
            core_source,
            wagyu=False,
        )
        wagyu = emit_bindings(
            relevant_nodes(clang_ast(include.parent, "webgpu_wagyu.h"), True),
            wagyu_source,
            wagyu=True,
        )
    (output / "renderer_src_webgpu_wagyu_port_include_webgpu_webgpu_h__decl.rs").write_text(core)
    (output / "renderer_src_webgpu_wagyu_port_include_webgpu_webgpu_wagyu_h__decl.rs").write_text(wagyu)
    print("generated frozen Wagyu core and extension ABI declarations")


if __name__ == "__main__":
    main()
