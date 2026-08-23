#!/usr/bin/env python3
"""Pure semantic coverage for the ore RSTB container mechanical translation."""

from __future__ import annotations

import argparse
import re
from pathlib import Path


def without_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.DOTALL)
    return re.sub(r"//.*", "", text)


def compact(text: str) -> str:
    return re.sub(r"\s+", "", without_comments(text))


def braced_body(text: str, header: re.Pattern[str], description: str) -> str:
    matches = list(header.finditer(text))
    if len(matches) != 1:
        raise AssertionError(f"expected exactly one {description}, found {len(matches)}")
    match = matches[0]
    open_brace = text.find("{", match.end())
    if open_brace < 0:
        raise AssertionError(f"missing body for {description}")
    depth = 1
    cursor = open_brace + 1
    while cursor < len(text) and depth:
        if text[cursor] == "{":
            depth += 1
        elif text[cursor] == "}":
            depth -= 1
        cursor += 1
    if depth:
        raise AssertionError(f"unclosed body for {description}")
    return text[open_brace + 1 : cursor - 1]


def function_body(text: str, name: str) -> str:
    return braced_body(
        text,
        re.compile(rf"\b(?:pub\s+)?fn\s+({re.escape(name)})(?:\s*<[^>]+>)?\s*\("),
        name,
    )


def assert_body(target: str, name: str, expected: str) -> None:
    actual = compact(function_body(target, name))
    if actual != compact(expected):
        raise AssertionError(f"wire/helper/parser body differs for {name}")


def struct_body(text: str, name: str) -> str:
    return braced_body(
        text,
        re.compile(rf"\b(?:pub\s+)?struct\s+({re.escape(name)})\b(?:\s*<[^>]+>)?"),
        f"struct {name}",
    )


def rust_fields(body: str) -> list[tuple[str, str]]:
    fields: list[tuple[str, str]] = []
    for line in without_comments(body).splitlines():
        stripped = line.strip()
        if not stripped.endswith(",") or ":" not in stripped:
            continue
        name, field_type = stripped[:-1].removeprefix("pub ").split(":", 1)
        fields.append((name.strip(), " ".join(field_type.split())))
    return fields


def check_translation(source_raw: str, target_raw: str) -> int:
    source = without_comments(source_raw)
    target = without_comments(target_raw)

    logical_lines = source_raw.splitlines()
    if len(logical_lines) != 224 or logical_lines[-1] != "} // namespace rive":
        raise AssertionError("source logical-line coverage must be exactly 1-224")
    checks = len(logical_lines)

    source_declarations = [
        "struct RstbEntry",
        "putU16",
        "putU32",
        "putStr",
        "buildWholeModuleContainer",
        "buildPerEntryContainer",
        "struct RstbEntryView",
        "getU16",
        "getU32",
        "getStr",
        "parseWholeModuleContainer",
        "parsePerEntryContainer",
    ]
    target_spellings = {
        "struct RstbEntry": "pub struct RstbEntry",
        "struct RstbEntryView": "pub struct RstbEntryView",
    }
    source_cursor = target_cursor = -1
    for declaration in source_declarations:
        source_spelling = declaration if declaration.startswith("struct ") else declaration
        source_cursor = source.find(source_spelling, source_cursor + 1)
        target_spelling = target_spellings.get(declaration, declaration)
        target_cursor = target.find(target_spelling, target_cursor + 1)
        if source_cursor < 0 or target_cursor < 0:
            raise AssertionError(f"missing or out-of-order declaration {declaration}")
        checks += 1

    expected_fields = {
        "RstbEntry": [
            ("stage", "u8"),
            ("logical", "RstbString"),
            ("physical", "RstbString"),
            ("source", "Vec<u8>"),
        ],
        "RstbEntryView": [
            ("stage", "u8"),
            ("logical", "RstbString"),
            ("physical", "RstbString"),
            ("source", "Option<&'a [u8]>"),
            ("sourceSize", "u32"),
        ],
    }
    expected_source_fields = {
        "RstbEntry": [
            "uint8_t stage = 0;",
            "std::string logical;",
            "std::string physical;",
            "std::vector<uint8_t> source;",
        ],
        "RstbEntryView": [
            "uint8_t stage = 0;",
            "std::string logical;",
            "std::string physical;",
            "const uint8_t* source = nullptr;",
            "uint32_t sourceSize = 0;",
        ],
    }
    for name, fields in expected_fields.items():
        source_body = compact(struct_body(source, name))
        source_cursor = -1
        for declaration in expected_source_fields[name]:
            source_cursor = source_body.find(compact(declaration), source_cursor + 1)
            if source_cursor < 0:
                raise AssertionError(f"source field/default differs for {name}")
            checks += 1
        if rust_fields(struct_body(target_raw, name)) != fields:
            raise AssertionError(f"target field type/order differs for {name}")
        checks += len(fields)

    for name, required_traits in {
        "RstbString": {"Clone", "Debug", "Default", "PartialEq", "Eq"},
        "RstbEntry": {"Clone", "Debug", "Default", "PartialEq", "Eq"},
    }.items():
        struct_start = target_raw.find(f"pub struct {name}")
        derives = re.findall(r"#\[derive\(([^)]*)\)\]", target_raw[max(0, struct_start - 180) : struct_start])
        actual_traits = set(derives[-1].replace(" ", "").split(",")) if derives else set()
        if not required_traits <= actual_traits:
            raise AssertionError(f"{name} does not preserve zero/empty owned defaults")
        checks += 1

    assert_body(target, "as_bytes", "&self.0")
    checks += 1
    from_impls = {
        r"impl\s+From<&str>\s+for\s+(RstbString)": (
            "fn from(value: &str) -> Self { Self(value.as_bytes().to_vec()) }"
        ),
        r"impl\s+From<String>\s+for\s+(RstbString)": (
            "fn from(value: String) -> Self { Self(value.into_bytes()) }"
        ),
        r"impl\s+From<Vec<u8>>\s+for\s+(RstbString)": (
            "fn from(value: Vec<u8>) -> Self { Self(value) }"
        ),
        r"impl\s+From<&\[u8\]>\s+for\s+(RstbString)": (
            "fn from(value: &[u8]) -> Self { Self(value.to_vec()) }"
        ),
    }
    for pattern, expected in from_impls.items():
        body = braced_body(target, re.compile(pattern), "RstbString conversion")
        if compact(body) != compact(expected):
            raise AssertionError("RstbString byte-preserving conversion differs")
        checks += 1

    view_default = braced_body(
        target,
        re.compile(r"\bimpl\s+Default\s+for\s+(RstbEntryView)(?:<[^>]+>)?"),
        "RstbEntryView Default",
    )
    expected_view_default = """
        fn default() -> Self {
            Self {
                stage: 0,
                logical: RstbString::default(),
                physical: RstbString::default(),
                source: None,
                sourceSize: 0,
            }
        }
    """
    if compact(view_default) != compact(expected_view_default):
        raise AssertionError("RstbEntryView defaults differ")
    checks += 6

    required_signatures = (
        "structRstbString(Vec<u8>);",
        "fnputU16(b:&mutVec<u8>,v:u16)",
        "fnputU32(b:&mutVec<u8>,v:u32)",
        "fnputStr(b:&mutVec<u8>,s:&RstbString)",
        "pubfnbuildWholeModuleContainer(entries:&[RstbEntry],src:Option<&[u8]>,srcLen:u32,)->Vec<u8>",
        "pubfnbuildPerEntryContainer(entries:&[RstbEntry])->Vec<u8>",
        "fngetU16(p:&mut&[u8],v:&mutu16)->bool",
        "fngetU32(p:&mut&[u8],v:&mutu32)->bool",
        "fngetStr(p:&mut&[u8],s:&mutRstbString)->bool",
        "pubfnparseWholeModuleContainer<'a>(blob:Option<&'a[u8]>,size:u32,out:&mutVec<RstbEntryView<'a>>,src:Option<&mutOption<&'a[u8]>>,srcLen:Option<&mutu32>,)->bool",
        "pubfnparsePerEntryContainer<'a>(blob:Option<&'a[u8]>,size:u32,out:&mutVec<RstbEntryView<'a>>,)->bool",
    )
    compact_target = compact(target_raw)
    for signature in required_signatures:
        if signature not in compact_target:
            raise AssertionError(f"required output/signature shape differs: {signature}")
        checks += 1
    if re.search(
        r"\bfn\s+putStr\s*\([^)]*\)\s*->", without_comments(target_raw)
    ):
        raise AssertionError("required output/signature shape differs: putStr must return void")

    expected_bodies = {
        "putU16": """
            b.push((v & 0xFF) as u8);
            b.push(((v >> 8) & 0xFF) as u8);
        """,
        "putU32": """
            b.push((v & 0xFF) as u8);
            b.push(((v >> 8) & 0xFF) as u8);
            b.push(((v >> 16) & 0xFF) as u8);
            b.push(((v >> 24) & 0xFF) as u8);
        """,
        "putStr": """
            Self::putU16(b, s.as_bytes().len() as u16);
            b.extend_from_slice(s.as_bytes());
        """,
        "buildWholeModuleContainer": """
            if entries.len() > 255 { return Vec::new(); }
            let mut b = Vec::new();
            b.push(entries.len() as u8);
            for e in entries {
                b.push(e.stage);
                rstb_detail::putStr(&mut b, &e.logical);
                rstb_detail::putStr(&mut b, &e.physical);
            }
            rstb_detail::putU32(&mut b, srcLen);
            if srcLen > 0 {
                let Some(src) = src else { return Vec::new(); };
                let srcLen = srcLen as usize;
                let Some(src) = src.get(..srcLen) else { return Vec::new(); };
                b.extend_from_slice(src);
            }
            b
        """,
        "buildPerEntryContainer": """
            if entries.len() > 255 { return Vec::new(); }
            let mut b = Vec::new();
            b.push(entries.len() as u8);
            for e in entries {
                b.push(e.stage);
                rstb_detail::putStr(&mut b, &e.logical);
                rstb_detail::putStr(&mut b, &e.physical);
                rstb_detail::putU32(&mut b, e.source.len() as u32);
                b.extend_from_slice(&e.source);
            }
            b
        """,
        "getU16": """
            if p.len() < 2 { return false; }
            *v = u16::from_le_bytes([p[0], p[1]]);
            *p = &p[2..];
            true
        """,
        "getU32": """
            if p.len() < 4 { return false; }
            *v = u32::from_le_bytes([p[0], p[1], p[2], p[3]]);
            *p = &p[4..];
            true
        """,
        "getStr": """
            let mut n = 0;
            if !Self::getU16(p, &mut n) { return false; }
            let n = usize::from(n);
            if p.len() < n { return false; }
            *s = RstbString::from(&p[..n]);
            *p = &p[n..];
            true
        """,
        "parseWholeModuleContainer": """
            let (Some(blob), Some(src), Some(srcLen)) = (blob, src, srcLen) else { return false; };
            if size == 0 || blob.is_empty() { return false; }
            let size = size as usize;
            let Some(blob) = blob.get(..size) else { return false; };
            let mut p = blob;
            let count = p[0];
            p = &p[1..];
            out.clear();
            out.reserve(usize::from(count));
            for _ in 0..count {
                if p.is_empty() { return false; }
                let mut e = RstbEntryView::default();
                e.stage = p[0];
                p = &p[1..];
                if e.stage > 2 { return false; }
                if !rstb_detail::getStr(&mut p, &mut e.logical) || !rstb_detail::getStr(&mut p, &mut e.physical) { return false; }
                out.push(e);
            }
            let mut n = 0;
            if !rstb_detail::getU32(&mut p, &mut n) { return false; }
            let Some(source) = p.get(..n as usize) else { return false; };
            *src = Some(source);
            *srcLen = n;
            true
        """,
        "parsePerEntryContainer": """
            let Some(blob) = blob else { return false; };
            if size == 0 || blob.is_empty() { return false; }
            let size = size as usize;
            let Some(blob) = blob.get(..size) else { return false; };
            let mut p = blob;
            let count = p[0];
            p = &p[1..];
            out.clear();
            out.reserve(usize::from(count));
            for _ in 0..count {
                if p.is_empty() { return false; }
                let mut e = RstbEntryView::default();
                e.stage = p[0];
                p = &p[1..];
                if e.stage > 2 { return false; }
                if !rstb_detail::getStr(&mut p, &mut e.logical) || !rstb_detail::getStr(&mut p, &mut e.physical) { return false; }
                let mut n = 0;
                if !rstb_detail::getU32(&mut p, &mut n) { return false; }
                let Some(source) = p.get(..n as usize) else { return false; };
                e.source = Some(source);
                e.sourceSize = n;
                p = &p[n as usize..];
                out.push(e);
            }
            true
        """,
    }
    for name, body in expected_bodies.items():
        assert_body(target, name, body)
        checks += 1

    source_put_u32 = compact(
        braced_body(source, re.compile(r"\binline\s+void\s+(putU32)\s*\("), "source putU32")
    )
    for shift in ("v&0xFF", "(v>>8)&0xFF", "(v>>16)&0xFF", "(v>>24)&0xFF"):
        if shift not in source_put_u32:
            raise AssertionError(f"source putU32 wire byte missing: {shift}")
        checks += 1
    source_stage_bounds = re.findall(r"e\.stage\s*>\s*(\d+)", source)
    if source_stage_bounds != ["2", "2"]:
        raise AssertionError("source parser stage bounds differ")
    checks += len(source_stage_bounds)
    source_count_bounds = re.findall(r"entries\.size\(\)\s*>\s*(\d+)", source)
    if source_count_bounds != ["255", "255"]:
        raise AssertionError("source entry-count bounds differ")
    checks += len(source_count_bounds)

    return checks


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--target", required=True, type=Path)
    parser.add_argument(
        "--mutation-probe",
        choices=(
            "put-u32-high-byte",
            "stage-upper-bound",
            "src-len-publication",
            "put-str-bool",
        ),
    )
    args = parser.parse_args()

    source_raw = args.source.read_text()
    target_raw = args.target.read_text()
    if args.mutation_probe is None:
        print(check_translation(source_raw, target_raw))
        return

    mutations = {
        "put-u32-high-byte": (
            "b.push(((v >> 24) & 0xFF) as u8);",
            "b.push(((v >> 16) & 0xFF) as u8);",
            "wire/helper/parser body differs for putU32",
        ),
        "stage-upper-bound": (
            "if e.stage > 2 {",
            "if e.stage > 3 {",
            "wire/helper/parser body differs for parseWholeModuleContainer",
        ),
        "src-len-publication": (
            "*srcLen = n;",
            "*srcLen = 0;",
            "wire/helper/parser body differs for parseWholeModuleContainer",
        ),
        "put-str-bool": (
            "fn putStr(b: &mut Vec<u8>, s: &RstbString) {",
            "fn putStr(b: &mut Vec<u8>, s: &RstbString) -> bool {",
            "required output/signature shape differs",
        ),
    }
    before, after, expected_failure = mutations[args.mutation_probe]
    expected_anchor_count = 2 if args.mutation_probe == "stage-upper-bound" else 1
    if target_raw.count(before) != expected_anchor_count:
        raise AssertionError(f"mutation anchor count differs for {args.mutation_probe}")
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
