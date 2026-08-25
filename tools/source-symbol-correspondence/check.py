#!/usr/bin/env python3
"""Freeze and verify the out-of-line C++ definition denominator.

This deliberately does not decide whether a Rust symbol is equivalent.  It
establishes the smaller units which a later certification ledger must cover.
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import pathlib
import re
import subprocess
import tomllib
from typing import Iterable


DENOMINATOR_SCHEMA = "nuxie-runtime-source-symbol-denominator/v1"
DISPOSITIONS_SCHEMA = "nuxie-runtime-source-symbol-dispositions/v1"
ALLOWED_DISPOSITIONS = {
    "mechanically-equivalent",
    "equivalent-under-adaptation",
    "deliberately-unsupported",
    "missing-or-incorrect",
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
        if any(token.value == "=" for token in header[:name_start]):
            continue
        close = pairs[opening]
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


def _scope_kind(header: list[Token]) -> str | None:
    values = [token.value for token in header]
    if "namespace" in values or (values and values[0] == "extern"):
        return "namespace"
    if any(value in {"class", "struct", "union", "enum"} for value in values):
        return "class"
    return None


def _canonical_signature(tokens: list[Token]) -> str:
    return " ".join(token.value for token in tokens)


def extract_definitions(source: str) -> list[Definition]:
    tokens = tokenize(source)
    braces = _brace_pairs(tokens)
    definitions: list[Definition] = []

    def walk(start: int, end: int) -> None:
        statement_start = start
        cursor = start
        while cursor < end:
            token = tokens[cursor]
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
                walk(cursor + 1, close)
                statement_start = close + 1
                cursor = close + 1
                continue
            if scope_kind == "class":
                cursor = close + 1
                continue
            function = _function_header(header)
            if function is not None:
                symbol, _, declarator_close = function
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
            # surrounding statement. Skip their contents without resetting it.
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


def _require_clean_tracked_sources(upstream_root: pathlib.Path) -> None:
    result = subprocess.run(
        ["git", "-C", str(upstream_root), "diff", "--quiet", "HEAD", "--", "src"],
        check=False,
    )
    if result.returncode == 1:
        raise ValueError("upstream src has tracked changes; denominator must use pinned bytes")
    if result.returncode != 0:
        raise ValueError("cannot verify the pinned upstream src worktree")


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
    _require_clean_tracked_sources(upstream_root)
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

    owners: list[dict[str, object]] = []
    total = 0
    for relative in actual_sources:
        definitions = extract_definitions((upstream_root / relative).read_text())
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
                    "symbol": definition.symbol,
                    "signature": definition.signature,
                    "line": definition.line,
                    "definition_sha256": definition.fingerprint,
                }
            )
        total += len(symbols)
        owners.append({"upstream": relative, "symbol_count": len(symbols), "symbols": symbols})
    return {
        "schema": DENOMINATOR_SCHEMA,
        "generator": "tools/source-symbol-correspondence/check.py",
        "definition_scope": "out-of-line function and method definitions in applicable C++ owners",
        "dispositions_schema": DISPOSITIONS_SCHEMA,
        "disposition_values": sorted(ALLOWED_DISPOSITIONS),
        "upstream_repository": str(manifest["upstream_repository"]),
        "upstream_ref": upstream_ref,
        "source_glob": source_glob,
        "exclude_glob": exclude_glob,
        "owner_count": len(owners),
        "symbol_count": total,
        "owners": owners,
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
        if disposition == "equivalent-under-adaptation" and not row.get("adaptation"):
            errors.append(f"adapted symbol lacks a named adaptation: {symbol_id}")
        if disposition == "deliberately-unsupported" and not row.get("decision"):
            errors.append(f"unsupported symbol lacks a named decision: {symbol_id}")
        if disposition == "missing-or-incorrect" and not row.get("tracking"):
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
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    expected = build_denominator(args.upstream_root, args.manifest)
    if args.write:
        args.denominator.parent.mkdir(parents=True, exist_ok=True)
        args.denominator.write_text(_canonical_json(expected))
    errors = check_denominator(expected, args.denominator)
    if args.dispositions is not None:
        errors.extend(check_dispositions(expected, args.dispositions))
    if errors:
        raise SystemExit("Source symbol correspondence failed:\n- " + "\n- ".join(errors))
    print(
        "Source symbol denominator: "
        f"{expected['symbol_count']} definitions across {expected['owner_count']} owners"
        + ("; every definition dispositioned" if args.dispositions else "")
    )


if __name__ == "__main__":
    main()
