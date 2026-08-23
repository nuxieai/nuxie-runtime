#!/usr/bin/env python3
"""Generate the Rust counterpart of frozen Wagyu webgpu_cpp.h.

The upstream file is generated C++: enum-class names and an RAII/member-call
facade sit directly over the C ABI from webgpu.h. This generator consumes both
frozen owners, validates their complete public denominators, and emits the same
split in Rust: transparent enum classes, ABI-identical structure aliases, and
owning one-word handle wrappers whose raw member calls cover every C entry
point owned by each C++ object class.
"""

from __future__ import annotations

import argparse
import hashlib
import re
from dataclasses import dataclass
from pathlib import Path


OBJECTS = (
    "Adapter", "BindGroup", "BindGroupLayout", "Buffer", "CommandBuffer",
    "CommandEncoder", "ComputePassEncoder", "ComputePipeline", "Device",
    "Instance", "PipelineLayout", "QuerySet", "Queue", "RenderBundle",
    "RenderBundleEncoder", "RenderPassEncoder", "RenderPipeline", "Sampler",
    "ShaderModule", "Surface", "Texture", "TextureView",
)
BITMASKS = {"BufferUsage", "ColorWriteMask", "MapMode", "ShaderStage", "TextureUsage"}
OWNED_STRUCTS = {
    "AdapterInfo": "wgpuAdapterInfoFreeMembers",
    "SupportedFeatures": "wgpuSupportedFeaturesFreeMembers",
    "SupportedInstanceFeatures": "wgpuSupportedInstanceFeaturesFreeMembers",
    "SupportedWGSLLanguageFeatures": "wgpuSupportedWGSLLanguageFeaturesFreeMembers",
    "SurfaceCapabilities": "wgpuSurfaceCapabilitiesFreeMembers",
}
OWNED_STRUCT_GUARDS = {
    "AdapterInfo": (
        "!self.0.vendor.data.is_null() || !self.0.architecture.data.is_null() || "
        "!self.0.device.data.is_null() || !self.0.description.data.is_null()"
    ),
    "SupportedFeatures": "!self.0.features.is_null()",
    "SupportedInstanceFeatures": "!self.0.features.is_null()",
    "SupportedWGSLLanguageFeatures": "!self.0.features.is_null()",
    "SurfaceCapabilities": (
        "!self.0.formats.is_null() || !self.0.presentModes.is_null() || "
        "!self.0.alphaModes.is_null()"
    ),
}
CONVERTIBLE_STATUS_FUNCTIONS = {
    "wgpuAdapterGetInfo", "wgpuAdapterGetLimits", "wgpuBufferReadMappedRange",
    "wgpuBufferWriteMappedRange", "wgpuDeviceGetAdapterInfo", "wgpuDeviceGetLimits",
    "wgpuSurfaceGetCapabilities", "wgpuSurfacePresent",
}


@dataclass(frozen=True)
class RawFunction:
    name: str
    args: tuple[tuple[str, str], ...]
    result: str | None


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cpp-header", type=Path, required=True)
    parser.add_argument("--c-header", type=Path, required=True)
    parser.add_argument("--raw-binding", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    return parser.parse_args()


def parse_raw_struct_fields(source: str) -> dict[str, dict[str, str]]:
    result: dict[str, dict[str, str]] = {}
    pattern = re.compile(r'pub\(crate\) struct (WGPU\w+) \{\n(.*?)\n\}', re.S)
    for match in pattern.finditer(source):
        fields = {}
        for field, kind in re.findall(
            r'^\s*pub\(crate\) (r#\w+|\w+): (.+),$', match.group(2), re.MULTILINE
        ):
            fields[field.removeprefix("r#")] = kind
        result[match.group(1)] = fields
    return result


def parse_c_initializers(source: str) -> list[tuple[str, str, list[tuple[str, str]]]]:
    lines = source.splitlines()
    result: list[tuple[str, str, list[tuple[str, str]]]] = []
    index = 0
    start_pattern = re.compile(
        r'^#define (WGPU_[A-Z0-9_]+_INIT) '
        r'_wgpu_MAKE_INIT_STRUCT\((WGPU\w+), \{ \\$',
    )
    while index < len(lines):
        match = start_pattern.match(lines[index])
        if not match:
            index += 1
            continue
        block = [lines[index]]
        index += 1
        while index < len(lines):
            block.append(lines[index])
            terminal = not lines[index].rstrip().endswith("\\")
            index += 1
            if terminal:
                break
        starts = [
            offset for offset, line in enumerate(block)
            if re.match(r'^    /\*\.\w+=\*/', line)
        ]
        fields: list[tuple[str, str]] = []
        for offset, start in enumerate(starts):
            end = starts[offset + 1] if offset + 1 < len(starts) else len(block) - 1
            expression = " ".join(
                line.strip().removesuffix("\\").strip() for line in block[start:end]
            )
            field_match = re.match(r'^/\*\.(\w+)=\*/(.*)$', expression)
            assert field_match
            expression = re.sub(r'\s+_wgpu_COMMA\s*$', '', field_match.group(2).strip())
            fields.append((field_match.group(1), expression))
        result.append((match.group(1), match.group(2), fields))
    if len(result) != 86:
        raise ValueError(f"expected 86 C structure initializers, found {len(result)}")
    return result


def translate_initializer_expression(
    expression: str,
    field_type: str,
    macro_types: dict[str, str],
) -> str:
    nested = re.fullmatch(
        r'_wgpu_MAKE_INIT_STRUCT\((WGPU\w+), \{ (.*) \}\)', expression
    )
    if nested:
        nested_fields = re.findall(
            r'/\*\.(\w+)=\*/(.*?)\s+_wgpu_COMMA(?=\s+(?:/\*\.|\}))',
            nested.group(2) + " }",
        )
        raw_fields = {name: kind for name, kind in nested_fields}
        assignments = []
        for field, value in raw_fields.items():
            kind = "*mut WGPUChainedStruct" if field == "next" else "WGPUSType"
            translated = translate_initializer_expression(value.strip(), kind, macro_types)
            assignments.append(f"{field}: {translated}")
        return f"{nested.group(1)} {{ {', '.join(assignments)} }}"
    enum_zero = re.fullmatch(r'_wgpu_ENUM_ZERO_INIT\((WGPU\w+)\)', expression)
    if enum_zero:
        return f"0 as {enum_zero.group(1)}"
    if expression == "_wgpu_STRUCT_ZERO_INIT":
        return f"{field_type}::default()"
    if expression in macro_types:
        return f"{macro_types[expression]}::default()"
    if expression == "NULL":
        if "Callback" in field_type or field_type == "WGPUProc":
            return "None"
        if field_type.startswith("*const"):
            return "std::ptr::null()"
        return "std::ptr::null_mut()"
    if expression == "WGPU_DEPTH_CLEAR_VALUE_UNDEFINED":
        return "f32::NAN"
    return {"0.": "0.0", "0.f": "0.0", "32.f": "32.0"}.get(expression, expression)


def emit_c_defaults(
    initializers: list[tuple[str, str, list[tuple[str, str]]]],
    raw_fields: dict[str, dict[str, str]],
) -> list[str]:
    macro_types = {macro: kind for macro, kind, _ in initializers}
    lines = []
    for _macro, kind, fields in initializers:
        expected = raw_fields.get(kind)
        if expected is None:
            raise ValueError(f"raw binding lacks initializer structure {kind}")
        if [name for name, _ in fields] != list(expected):
            raise ValueError(f"initializer field drift for {kind}")
        lines += [f"impl Default for {kind} {{", "    fn default() -> Self {", "        Self {"]
        for field, expression in fields:
            rust_field = f"r#{field}" if field == "type" else field
            translated = translate_initializer_expression(expression, expected[field], macro_types)
            lines.append(f"            {rust_field}: {translated},")
        lines += ["        }", "    }", "}", ""]
    return lines


def split_top_level(value: str) -> list[str]:
    result: list[str] = []
    depth = 0
    start = 0
    for index, char in enumerate(value):
        if char in "<([":
            depth += 1
        elif char in ">)]":
            depth -= 1
        elif char == "," and depth == 0:
            result.append(value[start:index].strip())
            start = index + 1
    tail = value[start:].strip()
    if tail:
        result.append(tail)
    return result


def parse_raw_functions(source: str) -> list[RawFunction]:
    functions: list[RawFunction] = []
    pattern = re.compile(
        r'^\s*pub\(crate\) fn (wgpu\w+)\((.*)\)(?: -> ([^;]+))?;$', re.MULTILINE
    )
    for match in pattern.finditer(source):
        args: list[tuple[str, str]] = []
        for item in split_top_level(match.group(2)):
            name, kind = item.split(":", 1)
            args.append((name.strip(), kind.strip()))
        functions.append(RawFunction(match.group(1), tuple(args), match.group(3)))
    if len(functions) != 199:
        raise ValueError(f"expected 199 raw WebGPU functions, found {len(functions)}")
    return functions


def parse_enums(source: str) -> list[tuple[str, str, list[tuple[str, str]]]]:
    pattern = re.compile(r'enum class (\w+)\s*:\s*(uint(?:32|64)_t)\s*\{(.*?)\n\};', re.S)
    enums = []
    for match in pattern.finditer(source):
        values = re.findall(r'^\s*(\w+)\s*=\s*(WGPU\w+),$', match.group(3), re.MULTILINE)
        enums.append((match.group(1), match.group(2), values))
    if len(enums) != 58:
        raise ValueError(f"expected 58 enum classes, found {len(enums)}")
    return enums


def parse_abi_pairs(source: str) -> list[tuple[str, str]]:
    pairs: list[tuple[str, str]] = []
    pattern = re.compile(r'static_assert\(sizeof\((\w+)\) == sizeof\((WGPU\w+)\)')
    for pair in pattern.findall(source):
        if pair not in pairs:
            pairs.append(pair)
    if len(pairs) != 156:
        raise ValueError(f"expected 156 unique ABI pairs, found {len(pairs)}")
    return pairs


def cpp_methods(source: str) -> list[tuple[str, str]]:
    result = re.findall(
        r'(?m)^(?!static_assert)(?:[\w:<>,*& ]+?)\s+(\w+)::(\w+)\s*\(', source
    )
    if len(result) != 288:
        raise ValueError(f"expected 288 method definitions, found {len(result)}")
    return result


def object_for_function(function: RawFunction) -> str | None:
    for name in sorted(OBJECTS, key=len, reverse=True):
        prefix = f"wgpu{name}"
        if function.name.startswith(prefix) and function.args:
            if function.args[0][1] == f"WGPU{name}":
                return name
    return None


def emit_enum(name: str, width: str, values: list[tuple[str, str]]) -> list[str]:
    raw = f"WGPU{name}"
    rust_width = "u64" if width == "uint64_t" else "u32"
    lines: list[str] = []
    if name == "SType":
        lines += [f"impl {name} {{"]
    else:
        lines += [
            "#[repr(transparent)]",
            "#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]",
            f"pub(crate) struct {name}(pub(crate) {rust_width});",
            f"impl {name} {{",
        ]
    for cpp_name, c_name in values:
        lines.append(f"    pub(crate) const {cpp_name}: Self = Self({c_name} as {rust_width});")
    lines += ["}", ""]
    if name != "SType":
        lines += [
            f"impl From<{raw}> for {name} {{",
            f"    fn from(value: {raw}) -> Self {{ Self(value as {rust_width}) }}",
            "}",
            f"impl From<{name}> for {raw} {{",
            f"    fn from(value: {name}) -> Self {{ value.0 as {raw} }}",
            "}",
            "",
        ]
    if name in BITMASKS:
        lines += [
            f"impl IsWGPUBitmask for {name} {{",
            f"    type Integral = {rust_width};",
            "    fn fromIntegral(value: Self::Integral) -> Self { Self(value) }",
            "    fn intoIntegral(self) -> Self::Integral { self.0 }",
            "    fn wrappingSubOne(value: Self::Integral) -> Self::Integral { value.wrapping_sub(1) }",
            "}",
            f"impl_wgpu_bitmask_operators!({name});",
            "",
        ]
    return lines


def emit_object(
    name: str, functions: list[RawFunction], enum_names: set[str]
) -> list[str]:
    raw = f"WGPU{name}"
    addref = f"wgpu{name}AddRef"
    release = f"wgpu{name}Release"
    methods = [f for f in functions if object_for_function(f) == name]
    method_names = {f.name for f in methods}
    if addref not in method_names or release not in method_names:
        raise ValueError(f"{name} lacks source AddRef/Release pair")
    lines = [
        "#[repr(transparent)]",
        f"pub(crate) struct {name} {{ handle: {raw} }}",
        f"impl {name} {{",
        f"    pub(crate) const fn Get(&self) -> {raw} {{ self.handle }}",
        f"    pub(crate) unsafe fn Acquire(handle: {raw}) -> Self {{ Self {{ handle }} }}",
        f"    pub(crate) unsafe fn FromBorrowed(handle: {raw}) -> Self {{",
        f"        if !handle.is_null() {{ {addref}(handle); }}",
        "        Self { handle }",
        "    }",
        f"    pub(crate) fn MoveToCHandle(mut self) -> {raw} {{",
        "        let handle = self.handle;",
        "        self.handle = std::ptr::null_mut();",
        "        handle",
        "    }",
    ]
    for function in methods:
        if function.name in {addref, release}:
            continue
        method = function.name[len(f"wgpu{name}"):]
        args = function.args[1:]
        signature = ", ".join(f"{arg}: {kind}" for arg, kind in args)
        if signature:
            signature = ", " + signature
        return_type = function.result
        conversion_prefix = ""
        conversion_suffix = ""
        if return_type and return_type.startswith("WGPU"):
            cpp_return = return_type[4:]
            if cpp_return in OBJECTS:
                return_type = cpp_return
                conversion_prefix = f"{cpp_return}::Acquire("
                conversion_suffix = ")"
            elif cpp_return in enum_names:
                return_type = cpp_return
                conversion_prefix = f"{cpp_return}::from("
                conversion_suffix = ")"
            elif return_type == "WGPUBool":
                return_type = "Bool"
                conversion_prefix = "Bool::from("
                conversion_suffix = ")"
        if function.name in CONVERTIBLE_STATUS_FUNCTIONS:
            return_type = "ConvertibleStatus"
            conversion_prefix = "ConvertibleStatus(Status::from("
            conversion_suffix = "))"
        result = f" -> {return_type}" if return_type else ""
        call_args = ", ".join(["self.handle", *(arg for arg, _ in args)])
        lines += [
            f"    pub(crate) unsafe fn {method}(&self{signature}){result} {{",
            f"        {conversion_prefix}{function.name}({call_args}){conversion_suffix}",
            "    }",
        ]
    lines += ["}", ""]
    lines += [
        f"impl Default for {name} {{",
        "    fn default() -> Self { Self { handle: std::ptr::null_mut() } }",
        "}",
        f"impl Clone for {name} {{",
        "    fn clone(&self) -> Self {",
        f"        unsafe {{ if !self.handle.is_null() {{ {addref}(self.handle); }} }}",
        "        Self { handle: self.handle }",
        "    }",
        "}",
        f"impl Drop for {name} {{",
        "    fn drop(&mut self) {",
        f"        unsafe {{ if !self.handle.is_null() {{ {release}(self.handle); }} }}",
        "    }",
        "}",
        f"impl PartialEq for {name} {{",
        "    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }",
        "}",
        f"impl Eq for {name} {{}}",
        "",
    ]
    return lines


def main() -> None:
    args = arguments()
    cpp = args.cpp_header.read_text()
    c_header = args.c_header.read_text()
    raw = args.raw_binding.read_text()
    functions = parse_raw_functions(raw)
    enums = parse_enums(cpp)
    pairs = parse_abi_pairs(cpp)
    method_defs = cpp_methods(cpp)
    initializers = parse_c_initializers(c_header)
    raw_struct_fields = parse_raw_struct_fields(raw)
    source_lines = len(cpp.splitlines())
    source_bytes = len(cpp.encode())
    if (source_lines, source_bytes) != (5833, 287_261):
        raise ValueError(f"frozen source size drift: {(source_lines, source_bytes)}")

    object_functions = {f.name for f in functions if object_for_function(f)}
    free_functions = [f for f in functions if f.name not in object_functions]
    lines = [
        "//! Generated complete Rust translation of frozen Wagyu webgpu_cpp.h; do not hand edit.",
        "",
        "#![allow(non_snake_case, non_upper_case_globals)]",
        "",
        "use super::webgpu_cpp_chained_struct_decl::{ChainedStruct, ChainedStructOut, SType};",
        "use super::webgpu_decl::*;",
        "use super::webgpu_enum_class_bitmasks_decl::{impl_wgpu_bitmask_operators, IsWGPUBitmask};",
        "",
        'pub(crate) const PINNED_SOURCE: &str = include_str!("source/renderer_src_webgpu_wagyu-port_include_webgpu_webgpu_cpp.h");',
        f"pub(crate) const PINNED_SOURCE_LINE_COUNT: usize = {source_lines};",
        f"pub(crate) const PINNED_SOURCE_BYTE_COUNT: usize = {source_bytes:_};",
        f'pub(crate) const PINNED_SOURCE_SHA256: &str = "{hashlib.sha256(cpp.encode()).hexdigest()}";',
        "",
        "pub(crate) const kArrayLayerCountUndefined: u32 = WGPU_ARRAY_LAYER_COUNT_UNDEFINED;",
        "pub(crate) const kCopyStrideUndefined: u32 = WGPU_COPY_STRIDE_UNDEFINED;",
        "pub(crate) const kDepthClearValueUndefined: f32 = f32::NAN;",
        "pub(crate) const kDepthSliceUndefined: u32 = WGPU_DEPTH_SLICE_UNDEFINED;",
        "pub(crate) const kLimitU32Undefined: u32 = WGPU_LIMIT_U32_UNDEFINED;",
        "pub(crate) const kLimitU64Undefined: u64 = WGPU_LIMIT_U64_UNDEFINED;",
        "pub(crate) const kMipLevelCountUndefined: u32 = WGPU_MIP_LEVEL_COUNT_UNDEFINED;",
        "pub(crate) const kQuerySetIndexUndefined: u32 = WGPU_QUERY_SET_INDEX_UNDEFINED;",
        "pub(crate) const kStrlen: usize = WGPU_STRLEN;",
        "pub(crate) const kWholeMapSize: usize = WGPU_WHOLE_MAP_SIZE;",
        "pub(crate) const kWholeSize: u64 = WGPU_WHOLE_SIZE;",
        "",
    ]
    for name, width, values in enums:
        lines += emit_enum(name, width, values)

    lines += emit_c_defaults(initializers, raw_struct_fields)

    lines += [
        "#[repr(transparent)]",
        "#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]",
        "pub(crate) struct Bool(pub(crate) WGPUBool);",
        "impl Bool { pub(crate) const fn asBool(self) -> bool { self.0 != 0 } }",
        "impl From<bool> for Bool { fn from(value: bool) -> Self { Self(value as WGPUBool) } }",
        "impl From<WGPUBool> for Bool { fn from(value: WGPUBool) -> Self { Self(value) } }",
        "",
        "#[repr(transparent)]",
        "#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]",
        "pub(crate) struct OptionalBool(pub(crate) WGPUOptionalBool);",
        "impl OptionalBool {",
        "    pub(crate) const False: Self = Self(WGPUOptionalBool_False);",
        "    pub(crate) const True: Self = Self(WGPUOptionalBool_True);",
        "    pub(crate) const Undefined: Self = Self(WGPUOptionalBool_Undefined);",
        "    pub(crate) fn intoOption(self) -> Option<bool> {",
        "        (self.0 != WGPUOptionalBool_Undefined).then_some(self.0 != WGPUOptionalBool_False)",
        "    }",
        "}",
        "impl Default for OptionalBool { fn default() -> Self { Self::Undefined } }",
        "impl From<bool> for OptionalBool { fn from(value: bool) -> Self { Self(value as WGPUOptionalBool) } }",
        "impl From<Option<bool>> for OptionalBool {",
        "    fn from(value: Option<bool>) -> Self { value.map(Self::from).unwrap_or(Self::Undefined) }",
        "}",
        "",
        "#[repr(transparent)]",
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]",
        "pub(crate) struct ConvertibleStatus(pub(crate) Status);",
        "impl ConvertibleStatus { pub(crate) fn asBool(self) -> bool { self.0 == Status::Success } }",
        "",
        "pub(crate) type Proc = WGPUProc;",
        "pub(crate) type StringView = WGPUStringView;",
        "",
    ]

    enum_names = {name for name, _, _ in enums}
    for cpp_name, raw_name in pairs:
        if cpp_name in enum_names or cpp_name in OBJECTS or cpp_name in {"ChainedStruct", "SType"}:
            continue
        if cpp_name in OWNED_STRUCTS:
            continue
        lines.append(f"pub(crate) type {cpp_name} = {raw_name};")
    lines.append("")

    for name, free_fn in OWNED_STRUCTS.items():
        raw_name = f"WGPU{name}"
        guard = OWNED_STRUCT_GUARDS[name]
        lines += [
            "#[repr(transparent)]",
            f"pub(crate) struct {name}(pub(crate) {raw_name});",
            f"impl {name} {{",
            f"    pub(crate) fn asRaw(&self) -> &{raw_name} {{ &self.0 }}",
            f"    pub(crate) fn asRawMut(&mut self) -> &mut {raw_name} {{ &mut self.0 }}",
            "}",
            f"impl Default for {name} {{",
            f"    fn default() -> Self {{ Self({raw_name}::default()) }}",
            "}",
            f"impl Drop for {name} {{",
            "    fn drop(&mut self) {",
            f"        if {guard} {{ unsafe {{ {free_fn}(std::ptr::read(&self.0)); }} }}",
            "    }",
            "}",
            "",
        ]

    for name in OBJECTS:
        lines += emit_object(name, functions, enum_names)

    lines += [
        "#[derive(Clone, Copy, Debug, PartialEq, Eq)]",
        "pub(crate) struct SourceSymbol {",
        "    pub(crate) owner: &'static str,",
        "    pub(crate) name: &'static str,",
        "}",
        "",
        "pub(crate) const CPP_METHOD_DEFINITIONS: &[SourceSymbol] = &[",
    ]
    for owner, name in method_defs:
        lines.append(f'    SourceSymbol {{ owner: "{owner}", name: "{name}" }},')
    lines += ["];","", "pub(crate) const RAW_MEMBER_ENTRY_POINTS: &[&str] = &["]
    for name in sorted(object_functions):
        lines.append(f'    "{name}",')
    lines += ["];","", "pub(crate) const RAW_FREE_ENTRY_POINTS: &[&str] = &["]
    for function in free_functions:
        lines.append(f'    "{function.name}",')
    lines += [
        "];",
        "",
        f"pub(crate) const CPP_ENUM_CLASS_COUNT: usize = {len(enums)};",
        f"pub(crate) const CPP_ABI_PAIR_COUNT: usize = {len(pairs)};",
        f"pub(crate) const CPP_METHOD_DEFINITION_COUNT: usize = {len(method_defs)};",
        f"pub(crate) const CPP_TEMPLATE_DECLARATION_COUNT: usize = {len(re.findall(r'(?m)^template\s*<', cpp))};",
        f"pub(crate) const CPP_STATIC_ASSERT_COUNT: usize = {len(re.findall(r'(?m)^static_assert\s*\(', cpp))};",
        f"pub(crate) const C_STRUCTURE_INITIALIZER_COUNT: usize = {len(initializers)};",
        f"pub(crate) const RAW_MEMBER_ENTRY_POINT_COUNT: usize = {len(object_functions)};",
        f"pub(crate) const RAW_FREE_ENTRY_POINT_COUNT: usize = {len(free_functions)};",
        "const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];",
        "",
        "#[cfg(test)]",
        "mod tests {",
        "    use super::*;",
        "    use std::mem::{align_of, size_of};",
        "",
        "    #[test]",
        "    fn complete_generated_denominators_are_locked() {",
        "        assert_eq!(CPP_ENUM_CLASS_COUNT, 58);",
        "        assert_eq!(CPP_ABI_PAIR_COUNT, 156);",
        "        assert_eq!(CPP_METHOD_DEFINITION_COUNT, 288);",
        "        assert_eq!(CPP_TEMPLATE_DECLARATION_COUNT, 43);",
        "        assert_eq!(C_STRUCTURE_INITIALIZER_COUNT, 86);",
        "        assert_eq!(RAW_MEMBER_ENTRY_POINT_COUNT + RAW_FREE_ENTRY_POINT_COUNT, 199);",
        "        assert_eq!(RAW_MEMBER_ENTRY_POINTS.len(), RAW_MEMBER_ENTRY_POINT_COUNT);",
        "        assert_eq!(RAW_FREE_ENTRY_POINTS.len(), RAW_FREE_ENTRY_POINT_COUNT);",
        "    }",
        "",
        "    #[test]",
        "    fn generated_defaults_preserve_nonzero_and_chain_initializers() {",
        "        let binding = WGPUBindGroupEntry::default();",
        "        assert_eq!(binding.size, WGPU_WHOLE_SIZE);",
        "        let sampler = WGPUSamplerDescriptor::default();",
        "        assert_eq!(sampler.lodMaxClamp, 32.0);",
        "        let max_draw = WGPURenderPassMaxDrawCount::default();",
        "        assert_eq!(max_draw.chain.sType, WGPUSType_RenderPassMaxDrawCount);",
        "        assert_eq!(max_draw.maxDrawCount, 50_000_000);",
        "        let depth = WGPURenderPassDepthStencilAttachment::default();",
        "        assert!(depth.depthClearValue.is_nan());",
        "    }",
        "",
        "    #[test]",
        "    fn enum_and_object_abi_is_transparent() {",
        "        assert_eq!(size_of::<TextureFormat>(), size_of::<WGPUTextureFormat>());",
        "        assert_eq!(align_of::<TextureFormat>(), align_of::<WGPUTextureFormat>());",
        "        assert_eq!(size_of::<Device>(), size_of::<WGPUDevice>());",
        "        assert_eq!(align_of::<Device>(), align_of::<WGPUDevice>());",
        "    }",
        "",
        "    #[test]",
        "    fn optional_bool_preserves_three_source_states() {",
        "        assert_eq!(OptionalBool::Undefined.intoOption(), None);",
        "        assert_eq!(OptionalBool::False.intoOption(), Some(false));",
        "        assert_eq!(OptionalBool::True.intoOption(), Some(true));",
        "    }",
        "}",
        "",
    ]
    args.output.write_text("\n".join(lines))


if __name__ == "__main__":
    main()
