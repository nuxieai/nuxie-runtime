#!/usr/bin/env python3
"""Pure source-coverage checks for the ore_types.hpp mechanical translation."""

from __future__ import annotations

import argparse
import re
from dataclasses import dataclass
from pathlib import Path


def without_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.DOTALL)
    return re.sub(r"//.*", "", text)


def braced_blocks(text: str, header: re.Pattern[str]) -> list[tuple[str, str, int]]:
    blocks: list[tuple[str, str, int]] = []
    for match in header.finditer(text):
        open_brace = text.find("{", match.end())
        if open_brace < 0:
            continue
        depth = 1
        cursor = open_brace + 1
        while cursor < len(text) and depth:
            if text[cursor] == "{":
                depth += 1
            elif text[cursor] == "}":
                depth -= 1
            cursor += 1
        if depth:
            raise AssertionError(f"unclosed declaration {match.group(1)}")
        blocks.append((match.group(1), text[open_brace + 1 : cursor - 1], match.start()))
    return blocks


def integer_expression(expression: str) -> int:
    expression = re.sub(
        r"(?<=[0-9A-Fa-f])(?:u(?:8|16|32|64|128|size)|i(?:8|16|32|64|128|size)|[uUlL]+)\b",
        "",
        expression.strip(),
    )
    if re.fullmatch(r"(?:0[xX][0-9A-Fa-f]+|\d+)", expression):
        return int(expression, 0)
    shifted = re.fullmatch(
        r"(0[xX][0-9A-Fa-f]+|\d+)\s*<<\s*(0[xX][0-9A-Fa-f]+|\d+)",
        expression,
    )
    if shifted:
        return int(shifted.group(1), 0) << int(shifted.group(2), 0)
    raise AssertionError(f"unsupported integer expression {expression!r}")


def enum_members(body: str, *, require_explicit: bool) -> list[tuple[str, int]]:
    members: list[tuple[str, int]] = []
    next_value = 0
    for declaration in body.split(","):
        declaration = declaration.strip()
        if not declaration:
            continue
        parts = declaration.split("=", 1)
        name = parts[0].strip()
        if not re.fullmatch(r"[A-Za-z_]\w*", name):
            raise AssertionError(f"malformed enum member {declaration!r}")
        if len(parts) == 2:
            value = integer_expression(parts[1])
        else:
            if require_explicit:
                raise AssertionError(f"Rust enum member {name} has no explicit value")
            value = next_value
        members.append((name, value))
        next_value = value + 1
    return members


def direct_struct_fields(body: str) -> list[str]:
    fields: list[str] = []
    depth = 0
    for line in body.splitlines():
        stripped = line.strip()
        if depth == 0 and stripped.endswith(";") and not stripped.startswith(("static ", "}")):
            declaration = stripped[:-1]
            if "(" not in declaration:
                for part in declaration.split(","):
                    before_initializer = part.split("=", 1)[0]
                    before_array = before_initializer.split("[", 1)[0]
                    identifiers = re.findall(r"[A-Za-z_]\w*", before_array)
                    if identifiers:
                        fields.append(identifiers[-1])
        depth += line.count("{") - line.count("}")
        if depth < 0:
            raise AssertionError("invalid nested struct depth")
    return fields


@dataclass(frozen=True)
class SourceField:
    name: str
    cpp_type: str
    default: str | None
    array_size: str | None


def direct_source_fields(body: str) -> list[SourceField]:
    fields: list[SourceField] = []
    depth = 0
    for line in body.splitlines():
        stripped = line.strip()
        if depth == 0 and stripped.endswith(";") and not stripped.startswith(("static ", "}")):
            declaration = stripped[:-1]
            if "(" not in declaration:
                parts = declaration.split(",")
                inherited_type: str | None = None
                for index, part in enumerate(parts):
                    match = re.fullmatch(
                        r"\s*(?:(?P<type>.+?\s))?(?P<name>[A-Za-z_]\w*)"
                        r"\s*(?:\[(?P<array>[^]]+)\])?"
                        r"\s*(?:=\s*(?P<default>.*))?\s*",
                        part,
                    )
                    if not match:
                        raise AssertionError(f"cannot parse source field {declaration!r}")
                    cpp_type = match.group("type")
                    if index == 0:
                        if cpp_type is None:
                            raise AssertionError(f"source field has no type {declaration!r}")
                        inherited_type = " ".join(cpp_type.split())
                    elif cpp_type is not None:
                        raise AssertionError(f"unexpected repeated field type {declaration!r}")
                    assert inherited_type is not None
                    fields.append(
                        SourceField(
                            name=match.group("name"),
                            cpp_type=inherited_type,
                            default=match.group("default").strip() if match.group("default") else None,
                            array_size=match.group("array").strip() if match.group("array") else None,
                        )
                    )
        depth += line.count("{") - line.count("}")
        if depth < 0:
            raise AssertionError("invalid nested struct depth")
    return fields


def direct_target_fields(body: str) -> list[tuple[str, str]]:
    fields: list[tuple[str, str]] = []
    depth = 0
    for line in body.splitlines():
        stripped = line.strip()
        if depth == 0 and stripped.endswith(",") and ":" in stripped:
            declaration = stripped[:-1].removeprefix("pub ")
            name, rust_type = declaration.split(":", 1)
            normalized_name = name.strip().removeprefix("r#")
            if re.fullmatch(r"[A-Za-z_]\w*", normalized_name):
                fields.append((normalized_name, " ".join(rust_type.split())))
        depth += line.count("{") - line.count("}")
        if depth < 0:
            raise AssertionError("invalid nested struct depth")
    return fields


def one_braced_body(text: str, header: re.Pattern[str], description: str) -> str:
    blocks = braced_blocks(text, header)
    if len(blocks) != 1:
        raise AssertionError(f"expected exactly one {description}, found {len(blocks)}")
    return blocks[0][1]


def default_assignments(target: str, struct_name: str) -> dict[str, str] | None:
    header = re.compile(
        rf"\bimpl(?:<[^>]+>)?\s+Default\s+for\s+({re.escape(struct_name)})(?:<[^>]+>)?"
    )
    blocks = braced_blocks(target, header)
    if not blocks:
        return None
    if len(blocks) != 1:
        raise AssertionError(f"multiple Default impls for {struct_name}")
    default_body = one_braced_body(
        blocks[0][1], re.compile(r"\bfn\s+(default)\s*\("), f"{struct_name}::default"
    )
    initializer = one_braced_body(
        default_body, re.compile(r"\b(Self)\s*(?=\{)"), f"{struct_name} initializer"
    )
    assignments: dict[str, str] = {}
    depth = 0
    for line in initializer.splitlines():
        stripped = line.strip()
        if depth == 0 and stripped.endswith(",") and ":" in stripped:
            name, expression = stripped[:-1].split(":", 1)
            assignments[name.strip().removeprefix("r#")] = " ".join(expression.split())
        depth += line.count("{") - line.count("}")
    return assignments


def normalized_expression(expression: str) -> str:
    return re.sub(r"\s+", "", expression).replace("_", "")


def declaration_sequence(source: str) -> list[tuple[str, str]]:
    pattern = re.compile(
        r"\b(?:(enum\s+class|struct|class)\s+(\w+)"
        r"|((?:static\s+)?constexpr\s+uint(?:8|32)_t)\s+(\w+)"
        r"|(inline\s+(?:uint32_t|ColorWriteMask))\s+(operator[|&]|\w+))"
    )
    declarations: list[tuple[str, str]] = []
    for match in pattern.finditer(source):
        if match.group(1):
            declarations.append((match.group(1), match.group(2)))
        elif match.group(3):
            declarations.append(("const", match.group(4)))
        else:
            declarations.append(("function", match.group(6)))
    return declarations


def target_declaration(kind: str, name: str) -> str:
    if name == "operator|":
        return "impl BitOr for ColorWriteMask"
    if name == "operator&":
        return "impl BitAnd for ColorWriteMask"
    if kind == "class":
        return f"pub type {name}Handle = AnyResourceHandle;"
    if kind == "enum class" and name == "ColorWriteMask":
        return "pub struct ColorWriteMask"
    if kind == "enum class":
        return f"pub enum {name}"
    if kind == "struct":
        return f"pub struct {name}"
    if kind == "const":
        return f"pub const {name}"
    return f"fn {name}"


def struct_body(blocks: dict[str, str], name: str) -> str:
    try:
        return blocks[name]
    except KeyError as error:
        raise AssertionError(f"missing Rust aggregate {name}") from error


COLLAPSED_COUNT_FIELDS: dict[str, set[str]] = {}

REQUIRED_BORROW_DESCRIPTORS = {"BufferDesc"}


def expected_rust_type(
    struct_name: str,
    field: SourceField,
    borrow_fields: dict[str, dict[str, str]],
) -> str | None:
    if field.name in COLLAPSED_COUNT_FIELDS.get(struct_name, set()):
        return None
    if field.name in borrow_fields.get(struct_name, {}):
        return borrow_fields[struct_name][field.name]
    scalar_types = {
        "uint32_t": "u32",
        "uint8_t": "u8",
        "int32_t": "i32",
        "float": "f32",
        "bool": "bool",
    }
    cpp_type = field.cpp_type
    if "*" in cpp_type:
        raise AssertionError(f"unmapped source pointer {struct_name}.{field.name}: {cpp_type}")
    rust_type = scalar_types.get(cpp_type, cpp_type)
    if cpp_type in {"ColorAttachment", "DepthStencilAttachment"}:
        rust_type = f"{cpp_type}<'a>"
    if field.array_size is not None:
        rust_type = f"[{rust_type}; {field.array_size}]"
    return rust_type


def expected_default_expression(field: SourceField, rust_type: str) -> str:
    expression = field.default
    if expression is None:
        plain_rust_type = rust_type.replace("<'a>", "")
        return f"{plain_rust_type}::default()"
    expression = expression.strip()
    if expression == "nullptr":
        if rust_type.startswith("Option<"):
            return "None"
        if rust_type.startswith("&'a ["):
            return "&[]"
        raise AssertionError(f"nonnull Rust borrow retained nullable default for {field.name}")
    if expression == "{}":
        if field.array_size is None:
            raise AssertionError(f"unexpected aggregate initializer for {field.name}")
        element_type = rust_type.removeprefix("[").split(";", 1)[0].replace("<'a>", "")
        return f"[{element_type}::default(); {field.array_size}]"
    if expression.startswith('"'):
        return f"Some({expression})"
    expression = re.sub(r"(?<=\d)f\b", "", expression)
    expression = re.sub(
        r"\b(kVertex|kFragment|kCompute|kNativeSlotAbsent)\b", r"Self::\1", expression
    )
    return expression


def function_body(target: str, name: str) -> str:
    header = re.compile(rf"\b(?:pub\s+)?(?:const\s+)?fn\s+({re.escape(name)})\b")
    return one_braced_body(target, header, f"function {name}")


def assert_exact_function_body(target: str, name: str, expected: str) -> None:
    actual = normalized_expression(function_body(target, name))
    if actual != normalized_expression(expected):
        raise AssertionError(f"helper body differs for {name}: {actual!r}")


def check_translation(source_raw: str, target_raw: str) -> int:
    source = without_comments(source_raw)
    target = without_comments(target_raw)
    raw_module_start = target_raw.index("pub mod raw_abi")
    features_marker = target_raw.index(
        "// Features — runtime capability query", raw_module_start
    )
    raw_module_end = target_raw.rfind("\n}", raw_module_start, features_marker) + 2
    raw_target_raw = target_raw[raw_module_start:raw_module_end]
    safe_target_raw = target_raw[:raw_module_start] + target_raw[raw_module_end:]
    safe_target = without_comments(safe_target_raw)
    checks = 0

    declarations = declaration_sequence(source)
    cursor = -1
    for kind, name in declarations:
        spelling = target_declaration(kind, name)
        cursor = target.find(spelling, cursor + 1)
        if cursor < 0:
            raise AssertionError(f"missing or out-of-order declaration {kind} {name}")
        if kind == "class" and re.search(
            rf"\bpub\s+(?:struct|enum|type)\s+{re.escape(name)}\b", target
        ):
            raise AssertionError(
                f"forward declaration {name} became a second constructible Rust owner"
            )
        checks += 1

    source_constants = re.findall(
        r"\b(?:static\s+)?constexpr\s+(uint(?:8|32)_t)\s+(\w+)\s*=\s*([^;]+);",
        source,
    )
    for cpp_type, name, expression in source_constants:
        match = re.search(rf"\bpub\s+const\s+{re.escape(name)}:\s*(u(?:8|32))\s*=\s*([^;]+);", target)
        if not match:
            raise AssertionError(f"missing exact constant declaration {name}")
        expected_type = {"uint8_t": "u8", "uint32_t": "u32"}[cpp_type]
        if match.group(1) != expected_type or integer_expression(match.group(2)) != integer_expression(expression):
            raise AssertionError(f"constant type/value differs for {name}")
        checks += 1

    source_enums = {
        name: enum_members(body, require_explicit=False)
        for name, body, _ in braced_blocks(source, re.compile(r"\benum\s+class\s+(\w+)"))
    }
    target_enums = {
        name: enum_members(body, require_explicit=True)
        for name, body, _ in braced_blocks(target, re.compile(r"\bpub\s+enum\s+(\w+)"))
        if name in source_enums
    }
    if set(target_enums) != set(source_enums) - {"ColorWriteMask"}:
        raise AssertionError("Rust repr(u8) enum set does not match the source enum set")
    for name, members in target_enums.items():
        if members != source_enums[name]:
            raise AssertionError(f"enum values differ for {name}")
        enum_start = target.find(f"pub enum {name}")
        attributes = target[max(0, enum_start - 160) : enum_start]
        if "#[repr(u8)]" not in attributes:
            raise AssertionError(f"enum {name} is missing repr(u8)")
        checks += len(members)

    mask_impl = braced_blocks(target, re.compile(r"\bimpl\s+(ColorWriteMask)\b"))[0][1]
    target_mask = [
        (name, integer_expression(expression))
        for name, expression in re.findall(
            r"pub\s+const\s+(\w+):\s+Self\s*=\s*Self\(([^)]+)\);", mask_impl
        )
    ]
    if target_mask != source_enums["ColorWriteMask"]:
        raise AssertionError("ColorWriteMask constants differ from the source")
    if "#[repr(transparent)]" not in target[: target.find("pub struct ColorWriteMask")][-160:]:
        raise AssertionError("ColorWriteMask is not an arbitrary-byte transparent mask")
    for spelling in ("pub const fn from_bits", "pub const fn bits"):
        if spelling not in mask_impl:
            raise AssertionError(f"ColorWriteMask is missing {spelling}")
    checks += len(target_mask) + 2

    source_struct_blocks = braced_blocks(source, re.compile(r"\bstruct\s+(\w+)"))
    target_struct_block_list = braced_blocks(
        safe_target_raw, re.compile(r"\bpub\s+struct\s+(\w+)")
    )
    target_structs = {name: body for name, body, _ in target_struct_block_list}
    target_struct_starts = {name: start for name, _, start in target_struct_block_list}
    raw_struct_block_list = braced_blocks(
        raw_target_raw, re.compile(r"\bpub\s+struct\s+(\w+)")
    )
    raw_struct_starts = {name: start for name, _, start in raw_struct_block_list}
    for name, source_body, _ in source_struct_blocks:
        target_body = struct_body(target_structs, name)
        cursor = -1
        fields = direct_struct_fields(source_body)
        for field in fields:
            match = re.search(rf"\b{re.escape(field)}\b", target_body[cursor + 1 :])
            if not match:
                raise AssertionError(f"missing or out-of-order field {name}.{field}")
            cursor += match.end()
            checks += 1
        prefix = safe_target_raw[
            max(0, target_struct_starts[name] - 180) : target_struct_starts[name]
        ]
        if name in raw_struct_starts:
            raw_prefix = raw_target_raw[
                max(0, raw_struct_starts[name] - 180) : raw_struct_starts[name]
            ]
            if "#[repr(C)]" not in raw_prefix:
                raise AssertionError(f"raw ABI aggregate {name} is missing repr(C)")
        elif "#[repr(C)]" not in prefix:
            raise AssertionError(f"aggregate {name} is missing repr(C)")
        derive = re.findall(r"#\[derive\(([^)]*)\)\]", prefix)
        if not derive or not {"Clone", "Copy"}.issubset(set(derive[-1].replace(" ", "").split(","))):
            raise AssertionError(f"aggregate {name} does not preserve trivial-copy semantics")
        checks += 1

    aliases = ("Buffer", "Texture", "TextureView", "Sampler", "ShaderModule", "Pipeline", "BindGroupLayout")
    for name in aliases:
        if f"pub type {name}Handle = AnyResourceHandle;" not in target:
            raise AssertionError(f"resource {name} does not use the canonical opaque handle")
        checks += 1
    if re.search(r"\*(?:const|mut)\b|\bc_(?:char|void)\b", safe_target):
        raise AssertionError("unconstrained raw descriptor pointers remain")
    checks += 1

    borrow_fields = {
        "BufferDesc": {"data": r"Option<&'a [u8]>", "label": r"Option<&'a str>"},
        "TextureDesc": {"label": r"Option<&'a str>"},
        "TextureViewDesc": {"texture": r"Option<&'a TextureHandle>"},
        "TextureDataDesc": {"data": r"Option<&'a [u8]>"},
        "SamplerDesc": {"label": r"Option<&'a str>"},
        "ShaderModuleDesc": {
            "code": r"Option<&'a [u8]>",
            "label": r"Option<&'a str>",
            "hlslSource": r"Option<&'a str>",
            "hlslEntryPoint": r"Option<&'a str>",
            "bindingMapBytes": r"Option<&'a [u8]>",
            "glFixupBytes": r"Option<&'a [u8]>",
        },
        "VertexBufferLayout": {"attributes": r"&'a [VertexAttribute]"},
        "BindGroupLayoutDesc": {"entries": r"&'a [BindGroupLayoutEntry]", "label": r"Option<&'a str>"},
        "PipelineDesc": {
            "vertexModule": r"Option<&'a ShaderModuleHandle>",
            "vertexEntryPoint": r"Option<&'a str>",
            "fragmentModule": r"Option<&'a ShaderModuleHandle>",
            "fragmentEntryPoint": r"Option<&'a str>",
            "vertexBuffers": r"Option<&'a [VertexBufferLayout<'a>]>",
            "bindGroupLayouts": r"Option<&'a [Option<&'a BindGroupLayoutHandle>]>",
            "label": r"Option<&'a str>",
        },
        "ColorAttachment": {"view": r"Option<&'a TextureViewHandle>", "resolveTarget": r"Option<&'a TextureViewHandle>"},
        "DepthStencilAttachment": {"view": r"Option<&'a TextureViewHandle>"},
        "RenderPassDesc": {"label": r"Option<&'a str>"},
        "BindGroupDesc": {
            "layout": r"Option<&'a BindGroupLayoutHandle>",
            "ubos": r"&'a [UBOEntry<'a>]",
            "textures": r"&'a [TexEntry<'a>]",
            "samplers": r"&'a [SampEntry<'a>]",
            "label": r"Option<&'a str>",
        },
        "UBOEntry": {"buffer": r"Option<&'a BufferHandle>"},
        "TexEntry": {"view": r"Option<&'a TextureViewHandle>"},
        "SampEntry": {"sampler": r"Option<&'a SamplerHandle>"},
    }
    for struct_name, fields in borrow_fields.items():
        body = struct_body(target_structs, struct_name)
        for field, field_type in fields.items():
            if not re.search(rf"\b{field}:\s*{re.escape(field_type)}", body):
                raise AssertionError(f"wrong borrow shape for {struct_name}.{field}")
            checks += 1

    target_clean_structs = {
        name: body
        for name, body, _ in braced_blocks(
            safe_target, re.compile(r"\bpub\s+struct\s+(\w+)")
        )
    }
    for struct_name, source_body, _ in source_struct_blocks:
        source_fields = direct_source_fields(source_body)
        target_fields = direct_target_fields(struct_body(target_clean_structs, struct_name))
        expected_fields = [
            (field, expected_rust_type(struct_name, field, borrow_fields)) for field in source_fields
        ]
        retained_fields = [(field, rust_type) for field, rust_type in expected_fields if rust_type]
        if [name for name, _ in target_fields] != [field.name for field, _ in retained_fields]:
            raise AssertionError(f"exact field set/order differs for {struct_name}")
        for (field, expected_type), (target_name, actual_type) in zip(retained_fields, target_fields):
            assert expected_type is not None
            if actual_type != expected_type:
                raise AssertionError(
                    f"field type differs for {struct_name}.{target_name}: "
                    f"expected {expected_type}, found {actual_type}"
                )
            checks += 1

        for field, rust_type in expected_fields:
            if rust_type is None:
                if field.cpp_type != "uint32_t" or normalized_expression(field.default or "") != "0":
                    raise AssertionError(f"collapsed count contract differs for {struct_name}.{field.name}")
                checks += 1

        assignments = default_assignments(target, struct_name)
        if struct_name in REQUIRED_BORROW_DESCRIPTORS:
            if assignments is not None:
                raise AssertionError(f"required-borrow descriptor {struct_name} must not implement Default")
            checks += 1
            continue
        if assignments is None:
            raise AssertionError(f"source-defaultable aggregate {struct_name} has no Default impl")
        if set(assignments) != {field.name for field, _ in retained_fields}:
            raise AssertionError(f"default field set differs for {struct_name}")
        for field, rust_type in retained_fields:
            assert rust_type is not None
            expected = expected_default_expression(field, rust_type)
            actual = assignments[field.name]
            if normalized_expression(actual) != normalized_expression(expected):
                raise AssertionError(
                    f"default differs for {struct_name}.{field.name}: "
                    f"expected {expected}, found {actual}"
                )
            checks += 1

    source_texture_body = one_braced_body(
        source,
        re.compile(r"\binline\s+uint32_t\s+(textureFormatBytesPerTexel)\b"),
        "source textureFormatBytesPerTexel",
    )
    target_texture_body = function_body(target, "textureFormatBytesPerTexel")
    source_texture_cases = [
        (name, integer_expression(value))
        for name, value in re.findall(
            r"case\s+TextureFormat::(\w+)\s*:\s*return\s+([^;]+);", source_texture_body
        )
    ]
    target_texture_cases = [
        (name, integer_expression(value))
        for name, value in re.findall(
            r"TextureFormat::(\w+)\s*=>\s*([^,]+),", target_texture_body
        )
    ]
    if target_texture_cases != source_texture_cases:
        raise AssertionError("textureFormatBytesPerTexel case values differ")
    source_texture_default = re.search(r"default\s*:\s*return\s+([^;]+);", source_texture_body)
    target_texture_default = re.search(r"_\s*=>\s*([^,]+),", target_texture_body)
    if (
        source_texture_default is None
        or target_texture_default is None
        or integer_expression(source_texture_default.group(1))
        != integer_expression(target_texture_default.group(1))
    ):
        raise AssertionError("textureFormatBytesPerTexel default differs")
    if not re.search(
        r"pub\s+const\s+fn\s+textureFormatBytesPerTexel\(fmt:\s*TextureFormat\)\s*->\s*u32",
        target,
    ):
        raise AssertionError("textureFormatBytesPerTexel signature differs")
    checks += len(source_texture_cases) + 2

    source_operator_blocks = {
        name: body
        for name, body, _ in braced_blocks(
            source, re.compile(r"\binline\s+ColorWriteMask\s+(operator[|&])\s*\(")
        )
    }
    target_operator_blocks = {
        trait_name: one_braced_body(
            target,
            re.compile(rf"\bimpl\s+({trait_name})\s+for\s+ColorWriteMask"),
            f"ColorWriteMask {trait_name}",
        )
        for trait_name in ("BitOr", "BitAnd")
    }
    for source_name, trait_name, method_name, operator in (
        ("operator|", "BitOr", "bitor", "|"),
        ("operator&", "BitAnd", "bitand", "&"),
    ):
        expected_source = (
            "return static_cast<ColorWriteMask>(static_cast<uint8_t>(a) "
            f"{operator} static_cast<uint8_t>(b));"
        )
        if normalized_expression(source_operator_blocks[source_name]) != normalized_expression(
            expected_source
        ):
            raise AssertionError(f"unsupported source body for {source_name}")
        expected_target = (
            "type Output = ColorWriteMask;"
            f"fn {method_name}(self, rhs: ColorWriteMask) -> ColorWriteMask {{"
            f"ColorWriteMask(self.0 {operator} rhs.0)"
            "}"
        )
        if normalized_expression(target_operator_blocks[trait_name]) != normalized_expression(
            expected_target
        ):
            raise AssertionError(f"ColorWriteMask operator body differs for {source_name}")
        checks += 1

    exact_helpers = {
        "from_bits": "Self(bits)",
        "bits": "self.0",
        "checked_count": "u32::try_from(len).map_err(|_| DescriptorSizeError)",
        "checked_len": "checked_count(bytes.len())",
        "checked_prefix": (
            "match values { Some(values) => values.get(..count as usize).map(Some).ok_or(DescriptorSizeError), None if count == 0 => Ok(None), None => Err(DescriptorSizeError), }"
        ),
        "uninitialized": (
            "Self { usage, size, data: None, immutable: false, label: None, }"
        ),
        "initialized": (
            "Ok(Self { usage, size: checked_len(data)?, data: Some(data), immutable, label: None, })"
        ),
        "size": "self.size",
        "data_prefix": (
            "match self.data { Some(data) => data.get(..self.size as usize).map(Some).ok_or(DescriptorSizeError), None => Ok(None), }"
        ),
        "immutable": "self.immutable",
        "codeSize": "checked_prefix(self.code, self.codeSize).map(|_| self.codeSize)",
        "hlslSourceSize": "checked_prefix(self.hlslSource.map(str::as_bytes), self.hlslSourceSize).map(|_| self.hlslSourceSize)",
        "bindingMapSize": "checked_prefix(self.bindingMapBytes, self.bindingMapSize).map(|_| self.bindingMapSize)",
        "glFixupSize": "checked_prefix(self.glFixupBytes, self.glFixupSize).map(|_| self.glFixupSize)",
        "attributeCount": "self.attributes.get(..self.attributeCount as usize).map(|_| self.attributeCount).ok_or(DescriptorSizeError)",
        "entryCount": "self.entries.get(..self.entryCount as usize).map(|_| self.entryCount).ok_or(DescriptorSizeError)",
        "vertexBufferCount": (
            "checked_prefix(self.vertexBuffers, self.vertexBufferCount).map(|_| self.vertexBufferCount)"
        ),
        "bindGroupLayoutCount": (
            "checked_prefix(self.bindGroupLayouts, self.bindGroupLayoutCount).map(|_| self.bindGroupLayoutCount)"
        ),
        "uboCount": "self.ubos.get(..self.uboCount as usize).map(|_| self.uboCount).ok_or(DescriptorSizeError)",
        "textureCount": "self.textures.get(..self.textureCount as usize).map(|_| self.textureCount).ok_or(DescriptorSizeError)",
        "samplerCount": "self.samplers.get(..self.samplerCount as usize).map(|_| self.samplerCount).ok_or(DescriptorSizeError)",
    }
    for name, expected_body in exact_helpers.items():
        assert_exact_function_body(target, name, expected_body)
        checks += 1

    return checks


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--target", required=True, type=Path)
    parser.add_argument(
        "--mutation-probe",
        choices=(
            "texture-depth-default",
            "rgba8unorm-byte-size",
            "depth-bias-type",
            "forward-handle-alias",
        ),
    )
    args = parser.parse_args()

    source_raw = args.source.read_text()
    target_raw = args.target.read_text()
    if args.mutation_probe is None:
        print(check_translation(source_raw, target_raw))
        return

    mutations = {
        "texture-depth-default": (
            "depthOrArrayLayers: 1,",
            "depthOrArrayLayers: 2,",
            "default differs for TextureDesc.depthOrArrayLayers",
        ),
        "rgba8unorm-byte-size": (
            "TextureFormat::rgba8unorm => 4,",
            "TextureFormat::rgba8unorm => 3,",
            "textureFormatBytesPerTexel case values differ",
        ),
        "depth-bias-type": (
            "pub depthBias: i32,",
            "pub depthBias: u32,",
            "field type differs for DepthStencilState.depthBias",
        ),
        "forward-handle-alias": (
            "pub type BufferHandle = AnyResourceHandle;",
            "pub struct Buffer;",
            "missing or out-of-order declaration class Buffer",
        ),
    }
    before, after, expected_failure = mutations[args.mutation_probe]
    if target_raw.count(before) != 1:
        raise AssertionError(f"mutation probe anchor is not unique: {before!r}")
    mutated = target_raw.replace(before, after, 1)
    try:
        check_translation(source_raw, mutated)
    except AssertionError as error:
        if expected_failure not in str(error):
            raise AssertionError(
                f"mutation {args.mutation_probe} failed for the wrong reason: {error}"
            ) from error
        print(1)
        return
    raise AssertionError(f"checker accepted hostile mutation {args.mutation_probe}")


if __name__ == "__main__":
    main()
