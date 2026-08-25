#!/usr/bin/env python3
"""Freeze and verify the pinned runtime implementation denominator.

This deliberately does not decide whether a Rust symbol is equivalent.  It
establishes the smaller units which a later certification ledger must cover.

Handwritten C++ bodies are enumerated at symbol granularity. Generated C++ and
Rust schema authority is frozen separately at byte granularity and the Rust
schema is replayed through nuxie-codegen by the generated-authority gate.
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import os
import pathlib
import re
import subprocess
import tempfile
import tomllib
from typing import Iterable


DENOMINATOR_SCHEMA = "nuxie-runtime-source-symbol-denominator/v2"
DISPOSITIONS_SCHEMA = "nuxie-runtime-source-symbol-dispositions/v1"
ALLOWED_DISPOSITIONS = {
    "exact",
    "adapted",
    "not-applicable",
    "missing",
}
CONTROL_NAMES = {
    "alignas",
    "catch",
    "decltype",
    "for",
    "if",
    "noexcept",
    "requires",
    "sizeof",
    "static_assert",
    "switch",
    "typeid",
    "while",
}
IDENTIFIER = re.compile(r"^[A-Za-z_$][A-Za-z0-9_$]*$")
MULTI_TOKENS = tuple(
    sorted(
        (
            "<=>",
            ">>=",
            "<<=",
            "->*",
            "...",
            "::",
            "->",
            "++",
            "--",
            "&&",
            "||",
            "==",
            "!=",
            "<=",
            ">=",
            "+=",
            "-=",
            "*=",
            "/=",
            "%=",
            "&=",
            "|=",
            "^=",
            "<<",
            ">>",
            ".*",
            "##",
        ),
        key=len,
        reverse=True,
    )
)


@dataclasses.dataclass(frozen=True)
class Token:
    value: str
    start: int
    end: int
    line: int
    kind: str = "punctuation"


@dataclasses.dataclass(frozen=True)
class Definition:
    symbol: str
    signature: str
    line: int
    fingerprint: str
    kind: str = "function"


def _sha256_bytes(contents: bytes) -> str:
    return hashlib.sha256(contents).hexdigest()


def _file_authority(root: pathlib.Path, relative: str) -> dict[str, object]:
    contents = (root / relative).read_bytes()
    return {
        "path": relative,
        "byte_count": len(contents),
        "sha256": _sha256_bytes(contents),
    }


def _authority_set(root: pathlib.Path, relatives: Iterable[str]) -> dict[str, object]:
    files = [_file_authority(root, relative) for relative in sorted(relatives)]
    digest = hashlib.sha256()
    for row in files:
        digest.update(str(row["path"]).encode("utf-8"))
        digest.update(b"\0")
        digest.update(str(row["byte_count"]).encode("ascii"))
        digest.update(b"\0")
        digest.update(str(row["sha256"]).encode("ascii"))
        digest.update(b"\n")
    return {
        "file_count": len(files),
        "byte_count": sum(int(row["byte_count"]) for row in files),
        "corpus_sha256": digest.hexdigest(),
        "files": files,
    }


def _line_end(source: str, start: int) -> int:
    """Return the end of a preprocessor logical line."""
    cursor = start
    while True:
        newline = source.find("\n", cursor)
        if newline < 0:
            return len(source)
        back = newline - 1
        while back >= start and source[back] in " \t\r":
            back -= 1
        if back < start or source[back] != "\\":
            return newline + 1
        cursor = newline + 1


def _quoted_end(source: str, start: int, quote: str) -> int:
    cursor = start + 1
    while cursor < len(source):
        if source[cursor] == "\\":
            cursor += 2
        elif source[cursor] == quote:
            return cursor + 1
        else:
            cursor += 1
    return len(source)


def _raw_string_end(source: str, start: int) -> int | None:
    match = re.match(r'(?:u8|u|U|L)?R"([^ ()\\\t\r\n]{0,16})\(', source[start:])
    if match is None:
        return None
    delimiter = match.group(1)
    terminator = ")" + delimiter + '"'
    end = source.find(terminator, start + match.end())
    return len(source) if end < 0 else end + len(terminator)


def tokenize(source: str) -> list[Token]:
    """Lex enough C++ to locate definitions without preprocessing includes.

    Comments and preprocessor directives are omitted. Literals remain atomic,
    so braces and comment markers inside them cannot affect structure.
    """
    tokens: list[Token] = []
    cursor = 0
    line = 1
    at_logical_line_start = True
    while cursor < len(source):
        char = source[cursor]
        if char in " \t\r\f\v":
            cursor += 1
            continue
        if char == "\n":
            cursor += 1
            line += 1
            at_logical_line_start = True
            continue
        if at_logical_line_start and char == "#":
            end = _line_end(source, cursor)
            line += source.count("\n", cursor, end)
            cursor = end
            at_logical_line_start = True
            continue
        at_logical_line_start = False
        if source.startswith("//", cursor):
            end = source.find("\n", cursor + 2)
            cursor = len(source) if end < 0 else end
            continue
        if source.startswith("/*", cursor):
            end = source.find("*/", cursor + 2)
            end = len(source) if end < 0 else end + 2
            line += source.count("\n", cursor, end)
            cursor = end
            continue
        raw_end = _raw_string_end(source, cursor)
        if raw_end is not None:
            value = source[cursor:raw_end]
            tokens.append(Token(value, cursor, raw_end, line, "literal"))
            line += value.count("\n")
            cursor = raw_end
            continue
        if char in "\"'":
            end = _quoted_end(source, cursor, char)
            value = source[cursor:end]
            tokens.append(Token(value, cursor, end, line, "literal"))
            line += value.count("\n")
            cursor = end
            continue
        identifier = re.match(r"[A-Za-z_$][A-Za-z0-9_$]*", source[cursor:])
        if identifier is not None:
            end = cursor + identifier.end()
            tokens.append(Token(source[cursor:end], cursor, end, line, "identifier"))
            cursor = end
            continue
        number = re.match(
            r"(?:0[xX][0-9A-Fa-f']+|0[bB][01']+|"
            r"(?:[0-9][0-9']*\.?[0-9']*|\.[0-9][0-9']*)"
            r"(?:[eEpP][+-]?[0-9']+)?)(?:[A-Za-z_][A-Za-z0-9_]*)?",
            source[cursor:],
        )
        if number is not None:
            end = cursor + number.end()
            tokens.append(Token(source[cursor:end], cursor, end, line, "number"))
            cursor = end
            continue
        operator = next(
            (value for value in MULTI_TOKENS if source.startswith(value, cursor)),
            char,
        )
        end = cursor + len(operator)
        tokens.append(Token(operator, cursor, end, line))
        cursor = end
    return tokens


def _preprocessor_logical_lines(source: str) -> list[tuple[int, int, int, str]]:
    """Return (start, end, line, text) for every preprocessor logical line."""
    rows: list[tuple[int, int, int, str]] = []
    cursor = 0
    line = 1
    while cursor < len(source):
        physical_end = source.find("\n", cursor)
        physical_end = len(source) if physical_end < 0 else physical_end + 1
        physical = source[cursor:physical_end]
        if re.match(r"^[ \t]*#", physical):
            end = _line_end(source, cursor)
            text = source[cursor:end]
            rows.append((cursor, end, line, text))
            line += text.count("\n")
            cursor = end
            continue
        line += physical.count("\n")
        cursor = physical_end
    return rows


def extract_macro_definitions(source: str) -> list[Definition]:
    """Freeze every macro definition, executable or not, as explicit authority.

    Keeping all definitions is intentionally conservative: deciding whether a
    replacement list becomes executable requires preprocessing every target
    configuration. The later disposition ledger can mark non-behavioral guards
    and constants mechanically while executable macro bodies remain visible.
    """
    definitions: list[Definition] = []
    for start, end, line, text in _preprocessor_logical_lines(source):
        match = re.match(
            r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_$][A-Za-z0-9_$]*)",
            text,
        )
        if match is None:
            continue
        name = match.group(1)
        cursor = match.end()
        parameters = ""
        # Function-like macros require the opening parenthesis to immediately
        # follow the name (C/C++ preprocessing rule).
        if cursor < len(text) and text[cursor] == "(":
            depth = 0
            parameter_end = cursor
            while parameter_end < len(text):
                char = text[parameter_end]
                if char == "(":
                    depth += 1
                elif char == ")":
                    depth -= 1
                    if depth == 0:
                        parameter_end += 1
                        break
                parameter_end += 1
            parameters = re.sub(r"\s+", " ", text[cursor:parameter_end]).strip()
        signature = f"#define {name}{parameters}"
        definitions.append(
            Definition(
                symbol=f"macro {name}",
                signature=signature,
                line=line,
                fingerprint=_sha256_bytes(source[start:end].encode("utf-8")),
                kind="macro-definition",
            )
        )
    return definitions


def executable_macro_names(source: str) -> set[str]:
    """Conservatively identify replacement lists which can inject a body.

    This is not used to decide whether macro definitions enter the denominator
    (all of them do). It only controls the additional invocation census for
    body-generating macros which otherwise have no post-expansion braces in the
    handwritten file.
    """
    names: set[str] = set()
    for _, _, _, text in _preprocessor_logical_lines(source):
        match = re.match(
            r"^[ \t]*#[ \t]*define[ \t]+([A-Za-z_$][A-Za-z0-9_$]*)(?:\([^\n]*?\))?",
            text,
        )
        if match is None:
            continue
        replacement = text[match.end() :]
        if "{" in replacement or re.search(
            r"\b(?:return|do|if|for|while|switch|throw)\b", replacement
        ):
            names.add(match.group(1))
    return names


def extract_macro_invocations(source: str, names: set[str]) -> list[Definition]:
    """Freeze invocations of known body-generating macros.

    Tokenization has already removed macro definitions, comments, strings, and
    inactive-branch directives. Conditional source branches remain visible, so
    every authored invocation receives an occurrence in source order.
    """
    if not names:
        return []
    tokens = tokenize(source)
    definitions: list[Definition] = []
    for index, token in enumerate(tokens[:-1]):
        if token.value not in names or tokens[index + 1].value != "(":
            continue
        depth = 0
        close = None
        for cursor in range(index + 1, len(tokens)):
            if tokens[cursor].value == "(":
                depth += 1
            elif tokens[cursor].value == ")":
                depth -= 1
                if depth == 0:
                    close = cursor
                    break
        if close is None:
            raise ValueError(f"unterminated macro invocation {token.value} at line {token.line}")
        invocation_tokens = tokens[index : close + 1]
        definitions.append(
            Definition(
                symbol=f"macro-invocation {token.value}",
                signature=_canonical_signature(invocation_tokens),
                line=token.line,
                fingerprint=_sha256_bytes(
                    source[token.start : tokens[close].end].encode("utf-8")
                ),
                kind="macro-invocation",
            )
        )
    return definitions


def _brace_pairs(tokens: list[Token]) -> dict[int, int]:
    stack: list[int] = []
    pairs: dict[int, int] = {}
    for index, token in enumerate(tokens):
        if token.value == "{":
            stack.append(index)
        elif token.value == "}":
            if not stack:
                raise ValueError(f"unmatched closing brace at line {token.line}")
            opening = stack.pop()
            pairs[opening] = index
    if stack:
        token = tokens[stack[-1]]
        raise ValueError(f"unmatched opening brace at line {token.line}")
    return pairs


def _paren_pairs(header: list[Token]) -> tuple[dict[int, int], int]:
    stack: list[int] = []
    pairs: dict[int, int] = {}
    for index, token in enumerate(header):
        if token.value == "(":
            stack.append(index)
        elif token.value == ")":
            if not stack:
                return {}, -1
            pairs[stack.pop()] = index
    return pairs, len(stack)


def _qualified_regular_name(header: list[Token], before_open: int) -> tuple[str, int] | None:
    if before_open < 0 or header[before_open].kind != "identifier":
        return None
    if header[before_open].value in CONTROL_NAMES:
        return None
    start = before_open
    if start > 0 and header[start - 1].value == "~":
        start -= 1
    while start >= 2 and header[start - 1].value == "::":
        qualifier_end = start - 2
        qualifier_start = qualifier_end
        if header[qualifier_end].value == ">":
            depth = 1
            qualifier_start -= 1
            while qualifier_start >= 0 and depth:
                if header[qualifier_start].value == ">":
                    depth += 1
                elif header[qualifier_start].value == "<":
                    depth -= 1
                qualifier_start -= 1
            qualifier_start += 1
            if qualifier_start > 0 and header[qualifier_start - 1].kind == "identifier":
                qualifier_start -= 1
        if header[qualifier_start].kind != "identifier":
            break
        start = qualifier_start
    return "".join(token.value for token in header[start : before_open + 1]), start


def _operator_name(header: list[Token], before_open: int) -> tuple[str, int] | None:
    lower = max(0, before_open - 12)
    operator_index = next(
        (
            index
            for index in range(before_open, lower - 1, -1)
            if header[index].value == "operator"
        ),
        None,
    )
    if operator_index is None:
        return None
    start = operator_index
    while start >= 2 and header[start - 1].value == "::":
        if header[start - 2].kind != "identifier":
            break
        start -= 2
    spelling = "".join(token.value for token in header[start : before_open + 1])
    return spelling, start


def _standalone_colon(tokens: Iterable[Token]) -> bool:
    return any(token.value == ":" for token in tokens)


def _has_top_level_assignment(tokens: list[Token]) -> bool:
    angle_depth = 0
    paren_depth = 0
    bracket_depth = 0
    for index, token in enumerate(tokens):
        value = token.value
        if value == "<" and not (
            index > 0 and tokens[index - 1].value == "operator"
        ):
            angle_depth += 1
        elif value == ">" and angle_depth:
            angle_depth -= 1
        elif value == ">>" and angle_depth:
            angle_depth = max(0, angle_depth - 2)
        elif value == "(":
            paren_depth += 1
        elif value == ")" and paren_depth:
            paren_depth -= 1
        elif value == "[":
            bracket_depth += 1
        elif value == "]" and bracket_depth:
            bracket_depth -= 1
        elif value == "=" and angle_depth == paren_depth == bracket_depth == 0:
            return True
    return False


def _function_header(header: list[Token]) -> tuple[str, int, int] | None:
    """Return (symbol, header start, declarator close) for a body header."""
    if not header:
        return None
    pairs, unmatched = _paren_pairs(header)
    if unmatched != 0:
        return None
    top_level_opens: list[int] = []
    depth = 0
    for index, token in enumerate(header):
        if token.value == "(":
            if depth == 0:
                top_level_opens.append(index)
            depth += 1
        elif token.value == ")":
            depth -= 1

    candidates: list[tuple[str, int, int]] = []
    for opening in top_level_opens:
        before = opening - 1
        named = _operator_name(header, before) or _qualified_regular_name(header, before)
        if named is None:
            continue
        symbol, name_start = named
        if _has_top_level_assignment(header[:name_start]):
            continue
        close = pairs[opening]
        tail = header[close + 1 :]
        if (
            tail
            and tail[-1].kind == "identifier"
            and tail[-1].value
            not in {"const", "volatile", "noexcept", "override", "final"}
            and not tail[-1].value.startswith("RIVE_")
            and not any(token.value in {"->", "requires", "noexcept", "throw"} for token in tail)
        ):
            # Function-like types in member initializers (`std::function<int(int)>
            # callback{...}`) are not declarators for the following brace.
            continue
        candidates.append((symbol, name_start, close))
    if not candidates:
        return None

    # A constructor initializer may itself look like a function call. Its
    # declarator is the candidate before the first standalone colon.
    for candidate in candidates:
        symbol, name_start, close = candidate
        tail = header[close + 1 :]
        colon_index = next(
            (index for index, token in enumerate(tail) if token.value == ":"),
            None,
        )
        if colon_index is None:
            continue
        pieces = symbol.replace("~", "").split("::")
        if len(pieces) >= 2 and pieces[-1] == pieces[-2]:
            after_colon = tail[colon_index + 1 :]
            # At `member{...}` this is an initializer brace, not the function
            # body. At the real body the preceding initializer is complete.
            if after_colon and after_colon[-1].value not in (")", "}"):
                return None
            return symbol, name_start, close

    # Annotation and noexcept macro calls precede/follow the true declarator;
    # the last remaining function-like top-level declarator is the useful one.
    return candidates[-1]


def _scope_declaration_keyword(header: list[Token]) -> int | None:
    cursor = 0
    while cursor < len(header):
        if (
            header[cursor].value in {"public", "private", "protected"}
            and cursor + 1 < len(header)
            and header[cursor + 1].value == ":"
        ):
            cursor += 2
            continue
        if header[cursor].value == "template" and cursor + 1 < len(header):
            cursor += 1
            if header[cursor].value != "<":
                return None
            depth = 0
            while cursor < len(header):
                if header[cursor].value == "<":
                    depth += 1
                elif header[cursor].value == ">":
                    depth -= 1
                elif header[cursor].value == ">>":
                    depth -= 2
                cursor += 1
                if depth <= 0:
                    break
            continue
        if header[cursor].value == "typedef":
            cursor += 1
            continue
        break
    return cursor if cursor < len(header) else None


def _scope_kind(header: list[Token]) -> str | None:
    keyword = _scope_declaration_keyword(header)
    if keyword is None:
        return None
    value = header[keyword].value
    if value == "namespace" or value == "extern" or (
        value == "inline"
        and keyword + 1 < len(header)
        and header[keyword + 1].value == "namespace"
    ):
        return "namespace"
    if value in {"class", "struct", "union", "enum"}:
        return "class"
    return None


def _class_scope_name(header: list[Token]) -> str | None:
    """Return the declared class/struct/union name, including specialization."""
    keyword = _scope_declaration_keyword(header)
    if keyword is None or header[keyword].value not in {
        "class",
        "struct",
        "union",
        "enum",
    }:
        return None
    cursor = keyword + 1
    if (
        header[keyword].value == "enum"
        and cursor < len(header)
        and header[cursor].value in {"class", "struct"}
    ):
        cursor += 1
    while cursor < len(header) and header[cursor].kind != "identifier":
        cursor += 1
    if cursor >= len(header):
        return None
    start = cursor
    end = cursor + 1
    if end < len(header) and header[end].value == "<":
        depth = 0
        while end < len(header):
            if header[end].value == "<":
                depth += 1
            elif header[end].value == ">":
                depth -= 1
                if depth == 0:
                    end += 1
                    break
            end += 1
    return "".join(token.value for token in header[start:end])


def _contains_lambda_capture_after(header: list[Token], index: int) -> bool:
    cursor = index
    while cursor < len(header):
        if header[cursor].value != "[":
            cursor += 1
            continue
        # `[[attribute]]` is not a lambda capture.
        if cursor + 1 < len(header) and header[cursor + 1].value == "[":
            cursor += 2
            continue
        depth = 1
        cursor += 1
        while cursor < len(header) and depth:
            if header[cursor].value == "[":
                depth += 1
            elif header[cursor].value == "]":
                depth -= 1
            cursor += 1
        if depth == 0:
            return True
    return False


def _canonical_signature(tokens: list[Token]) -> str:
    return " ".join(token.value for token in tokens)


def extract_definitions(
    source: str,
    *,
    include_inline_class: bool = False,
    macro_statement_names: set[str] | None = None,
    include_lexical_fallbacks: bool = False,
) -> list[Definition]:
    tokens = tokenize(source)
    braces = _brace_pairs(tokens)
    definitions: list[Definition] = []
    macro_statement_names = macro_statement_names or set()

    def walk(start: int, end: int, class_scope: tuple[str, ...] = ()) -> None:
        statement_start = start
        cursor = start
        while cursor < end:
            token = tokens[cursor]
            if (
                token.value in macro_statement_names
                and cursor + 1 < end
                and tokens[cursor + 1].value == "("
            ):
                depth = 0
                invocation_end = None
                for invocation_cursor in range(cursor + 1, end):
                    if tokens[invocation_cursor].value == "(":
                        depth += 1
                    elif tokens[invocation_cursor].value == ")":
                        depth -= 1
                        if depth == 0:
                            invocation_end = invocation_cursor + 1
                            break
                if invocation_end is None:
                    raise ValueError(
                        f"unterminated body-macro invocation {token.value} at line {token.line}"
                    )
                statement_start = invocation_end
                cursor = invocation_end
                continue
            if token.value == ";":
                statement_start = cursor + 1
                cursor += 1
                continue
            if token.value != "{":
                cursor += 1
                continue
            close = braces[cursor]
            header = tokens[statement_start:cursor]
            scope_kind = _scope_kind(header)
            if scope_kind == "namespace":
                walk(cursor + 1, close, class_scope)
                statement_start = close + 1
                cursor = close + 1
                continue
            if scope_kind == "class":
                if include_inline_class:
                    name = _class_scope_name(header)
                    if name is None:
                        name = f"<anonymous-class@{token.line}>"
                    walk(cursor + 1, close, (*class_scope, name))
                statement_start = close + 1
                cursor = close + 1
                continue
            function = _function_header(header)
            if function is not None:
                symbol, name_start, declarator_close = function
                if _contains_lambda_capture_after(header, declarator_close + 1):
                    if include_lexical_fallbacks:
                        definition_start = header[0].start
                        definitions.append(
                            Definition(
                                symbol=f"<lexical-brace@{header[0].line}>",
                                signature="brace-authority "
                                + _canonical_signature(header),
                                line=header[0].line,
                                fingerprint=_sha256_bytes(
                                    source[
                                        definition_start : tokens[close].end
                                    ].encode("utf-8")
                                ),
                                kind="lexical-brace-authority",
                            )
                        )
                    cursor = close + 1
                    continue
                if class_scope and "::" not in symbol and not any(
                    token.value == "friend" for token in header[:name_start]
                ):
                    symbol = "::".join((*class_scope, symbol))
                # Include declaration specifiers in the human-readable
                # signature, but omit constructor initializers.
                signature_tokens = header[: declarator_close + 1]
                definition_start = header[0].start
                definition_end = tokens[close].end
                fingerprint = hashlib.sha256(
                    source[definition_start:definition_end].encode("utf-8")
                ).hexdigest()
                definitions.append(
                    Definition(
                        symbol=symbol,
                        signature=_canonical_signature(signature_tokens),
                        line=header[0].line,
                        fingerprint=fingerprint,
                    )
                )
                statement_start = close + 1
                cursor = close + 1
                continue
            # Aggregate and constructor initializer braces are part of the
            # surrounding statement. The header census retains one explicit
            # lexical fallback row for such a region, so a lambda/default
            # initializer or unsupported declarator can never disappear
            # silently. Auditors disposition non-executable aggregates as
            # not-applicable and investigate any executable fallback.
            if include_lexical_fallbacks:
                definition_start = header[0].start if header else token.start
                definition_line = header[0].line if header else token.line
                definitions.append(
                    Definition(
                        symbol=f"<lexical-brace@{definition_line}>",
                        signature="brace-authority " + _canonical_signature(header),
                        line=definition_line,
                        fingerprint=_sha256_bytes(
                            source[definition_start : tokens[close].end].encode("utf-8")
                        ),
                        kind="lexical-brace-authority",
                    )
                )
            cursor = close + 1

    walk(0, len(tokens))
    return definitions


def _git_head(upstream_root: pathlib.Path) -> str:
    result = subprocess.run(
        ["git", "-C", str(upstream_root), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def _require_clean_tracked_authority(upstream_root: pathlib.Path) -> None:
    result = subprocess.run(
        [
            "git",
            "-C",
            str(upstream_root),
            "diff",
            "--quiet",
            "HEAD",
            "--",
            "src",
            "include/rive",
            "dev/defs",
        ],
        check=False,
    )
    if result.returncode == 1:
        raise ValueError(
            "upstream runtime authority has tracked changes; denominator must use pinned bytes"
        )
    if result.returncode != 0:
        raise ValueError("cannot verify the pinned upstream runtime worktree")


def _owner_row(
    upstream_root: pathlib.Path,
    relative: str,
    definitions: list[Definition],
    authority_kind: str,
) -> dict[str, object]:
    symbols: list[dict[str, object]] = []
    id_counts: dict[str, int] = {}
    for definition in definitions:
        seed = f"{relative}\0{definition.signature}"
        base = hashlib.sha256(seed.encode("utf-8")).hexdigest()[:16]
        occurrence = id_counts.get(base, 0) + 1
        id_counts[base] = occurrence
        symbols.append(
            {
                "id": f"{relative}::{base}:{occurrence}",
                "kind": definition.kind,
                "symbol": definition.symbol,
                "signature": definition.signature,
                "line": definition.line,
                "definition_sha256": definition.fingerprint,
            }
        )
    file_row = _file_authority(upstream_root, relative)
    return {
        "upstream": relative,
        "authority_kind": authority_kind,
        "byte_count": file_row["byte_count"],
        "file_sha256": file_row["sha256"],
        "symbol_count": len(symbols),
        "symbols": symbols,
    }


def build_denominator(
    upstream_root: pathlib.Path, manifest_path: pathlib.Path
) -> dict[str, object]:
    manifest = tomllib.loads(manifest_path.read_text())
    upstream_ref = str(manifest["upstream_ref"])
    actual_ref = _git_head(upstream_root)
    if actual_ref != upstream_ref:
        raise ValueError(
            f"upstream checkout is {actual_ref}, manifest pins {upstream_ref}"
        )
    _require_clean_tracked_authority(upstream_root)
    source_glob = str(manifest["source_glob"])
    exclude_glob = str(manifest["exclude_glob"])
    if source_glob != "src/**/*.cpp" or exclude_glob != "src/generated/**":
        raise ValueError("symbol extractor only supports the current source/exclude globs")

    actual_sources = sorted(
        path.relative_to(upstream_root).as_posix()
        for path in (upstream_root / "src").rglob("*.cpp")
        if not path.relative_to(upstream_root).as_posix().startswith("src/generated/")
    )
    manifest_sources = sorted(str(row["upstream"]) for row in manifest["file"])
    if actual_sources != manifest_sources:
        missing = sorted(set(actual_sources) - set(manifest_sources))
        extra = sorted(set(manifest_sources) - set(actual_sources))
        raise ValueError(f"source owner census differs: missing={missing}, extra={extra}")

    cpp_owners: list[dict[str, object]] = []
    for relative in actual_sources:
        definitions = extract_definitions((upstream_root / relative).read_text())
        cpp_owners.append(_owner_row(upstream_root, relative, definitions, "cpp-source"))

    objective_cpp_sources = sorted(
        path.relative_to(upstream_root).as_posix()
        for path in (upstream_root / "src").rglob("*.mm")
        if "/generated/" not in f"/{path.relative_to(upstream_root).as_posix()}"
    )
    objective_cpp_owners = [
        _owner_row(
            upstream_root,
            relative,
            extract_definitions((upstream_root / relative).read_text()),
            "objective-cpp-source",
        )
        for relative in objective_cpp_sources
    ]

    handwritten_headers = sorted(
        path.relative_to(upstream_root).as_posix()
        for directory in (upstream_root / "include" / "rive", upstream_root / "src")
        for path in directory.rglob("*")
        if path.is_file()
        and path.suffix in {".h", ".hpp"}
        and "/generated/" not in f"/{path.relative_to(upstream_root).as_posix()}"
    )
    executable_macros: set[str] = set()
    for relative in handwritten_headers:
        executable_macros.update(
            executable_macro_names((upstream_root / relative).read_text())
        )
    header_owners: list[dict[str, object]] = []
    for relative in handwritten_headers:
        source = (upstream_root / relative).read_text()
        definitions = extract_definitions(
            source,
            include_inline_class=True,
            macro_statement_names=executable_macros,
            include_lexical_fallbacks=True,
        )
        definitions.extend(extract_macro_definitions(source))
        definitions.extend(extract_macro_invocations(source, executable_macros))
        definitions.sort(key=lambda item: (item.line, item.kind, item.signature))
        header_owners.append(
            _owner_row(upstream_root, relative, definitions, "handwritten-header")
        )

    defs_files = sorted(
        path.relative_to(upstream_root).as_posix()
        for path in (upstream_root / "dev" / "defs").rglob("*")
        if path.is_file()
    )
    cpp_generated_files = sorted(
        path.relative_to(upstream_root).as_posix()
        for directory in (
            upstream_root / "include" / "rive" / "generated",
            upstream_root / "src" / "generated",
        )
        for path in directory.rglob("*")
        if path.is_file()
    )
    repo_root = manifest_path.parent.resolve()
    rust_schema_relative = "crates/nuxie-schema/src/generated/schema.rs"
    codegen_files = ["tools/nuxie-codegen/Cargo.toml"] + sorted(
        path.relative_to(repo_root).as_posix()
        for path in (repo_root / "tools" / "nuxie-codegen" / "src").rglob("*.rs")
    )
    owners = [*cpp_owners, *objective_cpp_owners, *header_owners]
    total = sum(int(owner["symbol_count"]) for owner in owners)
    return {
        "schema": DENOMINATOR_SCHEMA,
        "generator": "tools/source-symbol-correspondence/check.py",
        "definition_scope": (
            "out-of-line C++ definitions, handwritten inline header bodies, all "
            "handwritten header macro definitions, and invocations of body-generating macros"
        ),
        "parser_policy": {
            "conditional_compilation": "all authored branches are counted without preprocessing",
            "macro_definitions": "every handwritten #define is an explicit authority unit",
            "body_macro_invocations": (
                "invocations of macros whose replacement list contains braces or control-flow "
                "tokens are explicit authority units"
            ),
            "generated_code": (
                "excluded from handwritten symbol parsing and frozen by the separate generated "
                "authority byte/codegen gate"
            ),
            "lexical_fallbacks": (
                "every handwritten-header brace region not classified as namespace, class, or "
                "function is retained as a lexical-brace-authority row"
            ),
            "limits": [
                (
                    "the lexer does not expand arbitrary or externally defined macros; every "
                    "local macro definition and every owner file is nevertheless byte-frozen"
                ),
                (
                    "C++ declarations synthesized only after include expansion are governed by "
                    "their macro authority unit rather than assigned invented post-expansion symbols"
                ),
                "conditional branches are intentionally over-counted rather than target-selected",
            ],
        },
        "dispositions_schema": DISPOSITIONS_SCHEMA,
        "disposition_values": sorted(ALLOWED_DISPOSITIONS),
        "upstream_repository": str(manifest["upstream_repository"]),
        "upstream_ref": upstream_ref,
        "source_glob": source_glob,
        "exclude_glob": exclude_glob,
        "objective_cpp_source_glob": "src/**/*.mm",
        "handwritten_header_globs": [
            "include/rive/**/*.{h,hpp}",
            "src/**/*.{h,hpp}",
        ],
        "handwritten_header_exclude_globs": [
            "include/rive/generated/**",
            "src/generated/**",
        ],
        "cpp_owner_count": len(cpp_owners),
        "objective_cpp_owner_count": len(objective_cpp_owners),
        "handwritten_header_owner_count": len(header_owners),
        "owner_count": len(owners),
        "symbol_count": total,
        "owners": owners,
        "generated_authority": {
            "dev_defs": _authority_set(upstream_root, defs_files),
            "cpp_generator_outputs": _authority_set(upstream_root, cpp_generated_files),
            "rust_schema": _file_authority(repo_root, rust_schema_relative),
            "nuxie_codegen": _authority_set(repo_root, codegen_files),
            "schema_replay": {
                "tool": (
                    "cargo run --quiet -p nuxie-codegen -- --defs <dev/defs> --out <temp>; "
                    "rustfmt --edition 2024 <temp>"
                ),
                "expected_output": rust_schema_relative,
            },
        },
    }


def _canonical_json(document: dict[str, object]) -> str:
    return json.dumps(document, indent=2, sort_keys=False) + "\n"


def check_denominator(expected: dict[str, object], path: pathlib.Path) -> list[str]:
    if not path.is_file():
        return [f"symbol denominator is missing: {path}"]
    try:
        actual = json.loads(path.read_text())
    except (json.JSONDecodeError, OSError) as error:
        return [f"cannot read symbol denominator: {error}"]
    if actual.get("schema") != DENOMINATOR_SCHEMA:
        return [f"unexpected denominator schema: {actual.get('schema')!r}"]
    if actual != expected:
        return ["symbol denominator drifted; regenerate the pinned artifact"]
    return []


def verify_generated_schema_replay(
    repo_root: pathlib.Path, upstream_root: pathlib.Path
) -> list[str]:
    expected_path = repo_root / "crates/nuxie-schema/src/generated/schema.rs"
    if not expected_path.is_file():
        return [f"generated Rust schema is missing: {expected_path}"]
    with tempfile.TemporaryDirectory(prefix="nuxie-schema-authority-") as directory:
        replay_path = pathlib.Path(directory) / "schema.rs"
        result = subprocess.run(
            [
                "cargo",
                "run",
                "--quiet",
                "-p",
                "nuxie-codegen",
                "--",
                "--defs",
                str(upstream_root / "dev" / "defs"),
                "--out",
                str(replay_path),
            ],
            cwd=repo_root,
            check=False,
            capture_output=True,
            text=True,
            env={**os.environ, "CARGO_INCREMENTAL": "0"},
        )
        if result.returncode != 0:
            detail = result.stderr.strip().splitlines()
            suffix = f": {detail[-1]}" if detail else ""
            return [f"nuxie-schema codegen replay failed{suffix}"]
        if not replay_path.is_file():
            return ["nuxie-schema codegen replay produced no schema.rs"]
        format_result = subprocess.run(
            ["rustfmt", "--edition", "2024", str(replay_path)],
            cwd=repo_root,
            check=False,
            capture_output=True,
            text=True,
        )
        if format_result.returncode != 0:
            detail = format_result.stderr.strip().splitlines()
            suffix = f": {detail[-1]}" if detail else ""
            return [f"nuxie-schema rustfmt replay failed{suffix}"]
        expected = expected_path.read_bytes()
        actual = replay_path.read_bytes()
        if actual != expected:
            return [
                "nuxie-schema codegen replay differs from checked-in schema.rs "
                f"(expected {_sha256_bytes(expected)}, replay {_sha256_bytes(actual)})"
            ]
    return []


def check_dispositions(
    denominator: dict[str, object], dispositions_path: pathlib.Path
) -> list[str]:
    errors: list[str] = []
    try:
        document = json.loads(dispositions_path.read_text())
    except (json.JSONDecodeError, OSError) as error:
        return [f"cannot read symbol dispositions: {error}"]
    if document.get("schema") != DISPOSITIONS_SCHEMA:
        errors.append(f"unexpected dispositions schema: {document.get('schema')!r}")
    if document.get("upstream_ref") != denominator["upstream_ref"]:
        errors.append("dispositions do not pin the denominator upstream ref")
    expected_owners = {
        str(owner["upstream"]): int(owner["symbol_count"])
        for owner in denominator["owners"]
    }
    seen_owners: set[str] = set()
    for row in document.get("owners", []):
        owner_path = str(row.get("upstream", ""))
        if owner_path in seen_owners:
            errors.append(f"duplicate owner receipt: {owner_path}")
        seen_owners.add(owner_path)
        if not isinstance(row.get("receipt"), str) or not row["receipt"].strip():
            errors.append(f"owner lacks a receipt path: {owner_path}")
        review = row.get("independent_review")
        if (
            not isinstance(review, dict)
            or review.get("status") != "accepted"
            or not isinstance(review.get("reviewer"), str)
            or not review["reviewer"].strip()
        ):
            errors.append(f"owner lacks accepted independent review: {owner_path}")
        if expected_owners.get(owner_path) == 0 and not row.get(
            "no_executable_units_decision"
        ):
            errors.append(
                f"zero-unit owner lacks an explicit no-executable-units decision: {owner_path}"
            )
    missing_owners = sorted(set(expected_owners) - seen_owners)
    unknown_owners = sorted(seen_owners - set(expected_owners))
    if missing_owners:
        errors.append(
            f"{len(missing_owners)} owners lack receipts (first: {missing_owners[0]})"
        )
    if unknown_owners:
        errors.append(f"unknown owner receipts: {', '.join(unknown_owners[:5])}")
    if len(seen_owners) != len(expected_owners):
        errors.append(
            f"owner receipt census expected {len(expected_owners)}, "
            f"observed {len(seen_owners)} unique paths"
        )
    expected_ids = {
        symbol["id"]
        for owner in denominator["owners"]
        for symbol in owner["symbols"]
    }
    seen: set[str] = set()
    for row in document.get("symbols", []):
        symbol_id = str(row.get("id", ""))
        if symbol_id in seen:
            errors.append(f"duplicate symbol disposition: {symbol_id}")
        seen.add(symbol_id)
        disposition = row.get("disposition")
        if disposition not in ALLOWED_DISPOSITIONS:
            errors.append(f"invalid disposition for {symbol_id}: {disposition!r}")
        if disposition in {"exact", "adapted"}:
            rust_owners = row.get("rust_owners")
            if not isinstance(rust_owners, list) or not rust_owners or not all(
                isinstance(owner, str) and owner.strip() for owner in rust_owners
            ):
                errors.append(f"{disposition} symbol lacks concrete Rust owners: {symbol_id}")
            if not isinstance(row.get("receipt"), str) or not row["receipt"].strip():
                errors.append(f"{disposition} symbol lacks a receipt path: {symbol_id}")
            review = row.get("independent_review")
            if (
                not isinstance(review, dict)
                or review.get("status") != "accepted"
                or not isinstance(review.get("reviewer"), str)
                or not review["reviewer"].strip()
            ):
                errors.append(
                    f"{disposition} symbol lacks accepted independent review: {symbol_id}"
                )
            evidence = row.get("evidence")
            has_evidence = isinstance(evidence, list) and bool(evidence) and all(
                isinstance(item, str) and item.strip() for item in evidence
            )
            exemption = row.get("evidence_exemption")
            has_exemption = isinstance(exemption, str) and bool(exemption.strip())
            if not has_evidence and not has_exemption:
                errors.append(
                    f"{disposition} symbol lacks behavioral evidence or exemption: {symbol_id}"
                )
        if disposition == "adapted" and not row.get("adaptation"):
            errors.append(f"adapted symbol lacks a named adaptation: {symbol_id}")
        if disposition == "not-applicable" and not row.get("decision"):
            errors.append(f"not-applicable symbol lacks a governing decision: {symbol_id}")
        if disposition == "missing" and not row.get("tracking"):
            errors.append(f"missing/incorrect symbol lacks tracking: {symbol_id}")
    missing = sorted(expected_ids - seen)
    unknown = sorted(seen - expected_ids)
    if missing:
        errors.append(f"{len(missing)} symbols lack dispositions (first: {missing[0]})")
    if unknown:
        errors.append(f"unknown symbol dispositions: {', '.join(unknown[:5])}")
    if len(seen) != len(expected_ids):
        errors.append(
            f"disposition census expected {len(expected_ids)}, observed {len(seen)} unique ids"
        )
    return errors


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--upstream-root", type=pathlib.Path, required=True)
    parser.add_argument("--manifest", type=pathlib.Path, required=True)
    parser.add_argument("--denominator", type=pathlib.Path, required=True)
    parser.add_argument("--dispositions", type=pathlib.Path)
    parser.add_argument("--verify-generated-authority", action="store_true")
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    expected = build_denominator(args.upstream_root, args.manifest)
    if args.write:
        args.denominator.parent.mkdir(parents=True, exist_ok=True)
        args.denominator.write_text(_canonical_json(expected))
    errors = check_denominator(expected, args.denominator)
    if args.verify_generated_authority:
        errors.extend(
            verify_generated_schema_replay(args.manifest.parent.resolve(), args.upstream_root)
        )
    if args.dispositions is not None:
        errors.extend(check_dispositions(expected, args.dispositions))
    if errors:
        raise SystemExit("Source symbol correspondence failed:\n- " + "\n- ".join(errors))
    print(
        "Source symbol denominator: "
        f"{expected['symbol_count']} authority units across {expected['owner_count']} owners "
        f"({expected['cpp_owner_count']} C++ sources, "
        f"{expected['objective_cpp_owner_count']} Objective-C++ sources, "
        f"{expected['handwritten_header_owner_count']} handwritten headers)"
        + ("; generated authority replayed" if args.verify_generated_authority else "")
        + ("; every definition dispositioned" if args.dispositions else "")
    )


if __name__ == "__main__":
    main()
