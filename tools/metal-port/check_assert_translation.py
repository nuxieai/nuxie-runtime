#!/usr/bin/env python3
"""Check C/C++ ``assert`` preservation across all Metal translation units.

The pinned sources use C ``assert`` with NDEBUG semantics.  A mechanical Rust
translation must therefore use ``debug_assert`` at those sites.  This checker
keeps a lexical assertion fingerprint for every translation unit and rejects
any drift, including the dangerous ``debug_assert!`` -> ``assert!`` mutation.
"""

from __future__ import annotations

import argparse
import csv
import re
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path


PINNED_REF = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"
C_FAMILY_SUFFIXES = {".h", ".hpp", ".c", ".cc", ".cpp", ".cxx", ".m", ".mm"}
SOURCE_ASSERT = re.compile(r"\bassert\s*\(")
RUST_DEBUG_ASSERT = re.compile(r"\bdebug_assert(?:_eq|_ne|_abort)?\s*!")
RUST_ASSERT = re.compile(r"(?<!debug_)\bassert(?:_eq|_ne)?\s*!")
RUST_RAW_STRING = re.compile(r"(?:br|rb|r)(?P<hashes>#{0,255})\"")


class CheckError(RuntimeError):
    pass


@dataclass(frozen=True)
class Counts:
    source: int
    rust_debug: int
    rust_release: int


def _mask_lexical_noise(text: str, *, rust: bool) -> str:
    """Blank comments and literals while preserving token positions/newlines."""

    chars = list(text)
    n = len(chars)
    out = chars.copy()
    i = 0

    def blank(start: int, end: int) -> None:
        for index in range(start, end):
            if out[index] != "\n":
                out[index] = " "

    while i < n:
        if text.startswith("//", i):
            end = text.find("\n", i + 2)
            end = n if end < 0 else end
            blank(i, end)
            i = end
            continue
        if text.startswith("/*", i):
            depth = 1
            end = i + 2
            while end < n and depth:
                if rust and text.startswith("/*", end):
                    depth += 1
                    end += 2
                elif text.startswith("*/", end):
                    depth -= 1
                    end += 2
                else:
                    end += 1
            blank(i, end)
            i = end
            continue

        raw = None
        if rust and text[i] in "br":
            raw = RUST_RAW_STRING.match(text, i)
        if raw is not None:
            hashes = raw.group("hashes")
            terminator = '"' + hashes
            body = raw.end()
            end_at = text.find(terminator, body)
            end = n if end_at < 0 else end_at + len(terminator)
            blank(i, end)
            i = end
            continue

        quote_at = i
        if text.startswith('b"', i) or text.startswith('@"', i):
            quote_at = i + 1
        if text[quote_at] == '"':
            end = quote_at + 1
            while end < n:
                if text[end] == "\\":
                    end += 2
                elif text[end] == '"':
                    end += 1
                    break
                else:
                    end += 1
            blank(i, min(end, n))
            i = min(end, n)
            continue

        # Mask a character literal but not a Rust lifetime such as 'a.
        if text[i] == "'":
            end = i + 1
            if end < n and text[end] == "\\":
                end += 2
            else:
                end += 1
            if end < n and text[end] == "'":
                end += 1
                blank(i, end)
                i = end
                continue
        i += 1
    return "".join(out)


def assertion_counts(source_texts: list[str], rust_texts: list[str]) -> Counts:
    source = sum(
        len(SOURCE_ASSERT.findall(_mask_lexical_noise(text, rust=False)))
        for text in source_texts
    )
    rust_debug = sum(
        len(RUST_DEBUG_ASSERT.findall(_mask_lexical_noise(text, rust=True)))
        for text in rust_texts
    )
    rust_release = sum(
        len(RUST_ASSERT.findall(_mask_lexical_noise(text, rust=True)))
        for text in rust_texts
    )
    return Counts(source, rust_debug, rust_release)


def _load_units(manifest_path: Path) -> tuple[str, list[dict[str, object]]]:
    manifest = tomllib.loads(manifest_path.read_text())
    units = manifest.get("translation_unit", [])
    if len(units) != 41:
        raise CheckError(f"manifest must contain exactly 41 translation units, found {len(units)}")
    ids = [str(unit["id"]) for unit in units]
    if len(set(ids)) != len(ids):
        raise CheckError("manifest translation-unit IDs are not unique")
    return str(manifest.get("upstream_ref", "")), units


def _load_authority(path: Path) -> dict[str, Counts]:
    rows: dict[str, Counts] = {}
    with path.open(newline="") as stream:
        reader = csv.DictReader(stream, delimiter="\t")
        expected = {
            "version",
            "upstream_ref",
            "unit",
            "source_c_asserts",
            "rust_debug_asserts",
            "rust_asserts",
        }
        if set(reader.fieldnames or []) != expected:
            raise CheckError(f"unexpected authority columns: {reader.fieldnames}")
        for row in reader:
            if row["version"] != "1" or row["upstream_ref"] != PINNED_REF:
                raise CheckError(f"invalid authority provenance for {row['unit']}")
            unit = row["unit"]
            if unit in rows:
                raise CheckError(f"duplicate authority row for {unit}")
            rows[unit] = Counts(
                int(row["source_c_asserts"]),
                int(row["rust_debug_asserts"]),
                int(row["rust_asserts"]),
            )
    return rows


MUTATIONS = {
    "rive-types-unreachable": (
        "generic-rive-types",
        'debug_assert!(!true, "unreachable reached");',
        'assert!(!true, "unreachable reached");',
    ),
    "gradient-paint-type": (
        "generic-gradient",
        "debug_assert!(\n            paint_type == gpu::PaintType::linearGradient",
        "assert!(\n            paint_type == gpu::PaintType::linearGradient",
    ),
    "renderer-restore": (
        "generic-rive-renderer",
        "debug_assert!(self.m_renderStateStack.len() > 1);",
        "assert!(self.m_renderStateStack.len() > 1);",
    ),
}


def run_check(
    repo_root: Path,
    upstream_root: Path,
    manifest_path: Path,
    authority_path: Path,
    mutation_probe: str | None = None,
) -> tuple[int, int, int]:
    manifest_ref, units = _load_units(manifest_path)
    if manifest_ref != PINNED_REF:
        raise CheckError(f"manifest pins {manifest_ref}, expected {PINNED_REF}")
    head = subprocess.run(
        ["git", "-C", str(upstream_root), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if head != PINNED_REF:
        raise CheckError(f"upstream checkout is {head}, expected {PINNED_REF}")

    authority = _load_authority(authority_path)
    unit_ids = {str(unit["id"]) for unit in units}
    if set(authority) != unit_ids:
        missing = sorted(unit_ids - set(authority))
        extra = sorted(set(authority) - unit_ids)
        raise CheckError(f"authority/manifest mismatch: missing={missing}, extra={extra}")

    mutated = False
    errors: list[str] = []
    totals = [0, 0, 0]
    for unit in units:
        unit_id = str(unit["id"])
        source_texts = []
        for relative in unit["sources"]:
            path = upstream_root / str(relative)
            if not path.is_file():
                raise CheckError(f"{unit_id}: missing pinned source {relative}")
            if path.suffix.lower() in C_FAMILY_SUFFIXES:
                source_texts.append(path.read_text(errors="surrogateescape"))
        rust_texts = []
        for relative in unit["rust_targets"]:
            path = repo_root / str(relative)
            if not path.is_file():
                raise CheckError(f"{unit_id}: missing Rust target {relative}")
            text = path.read_text(errors="surrogateescape")
            if mutation_probe is not None and MUTATIONS[mutation_probe][0] == unit_id:
                before, after = MUTATIONS[mutation_probe][1:]
                occurrences = text.count(before)
                if occurrences > 1 or (occurrences == 1 and mutated):
                    raise CheckError(
                        f"{mutation_probe}: mutation anchor is not globally unique ({relative})"
                    )
                if occurrences == 1:
                    text = text.replace(before, after, 1)
                    mutated = True
            rust_texts.append(text)
        actual = assertion_counts(source_texts, rust_texts)
        expected = authority[unit_id]
        totals[0] += actual.source
        totals[1] += actual.rust_debug
        totals[2] += actual.rust_release
        if actual != expected:
            errors.append(f"{unit_id}: expected {expected}, found {actual}")

    if mutation_probe is not None:
        if not mutated:
            raise CheckError(f"mutation probe {mutation_probe} did not mutate a target")
        if not errors:
            raise CheckError(f"mutation probe {mutation_probe} was not rejected")
        print(f"C/C++ assert parity mutation probe rejected: {mutation_probe}")
        return tuple(totals)
    if errors:
        raise CheckError("assertion parity drift:\n  " + "\n  ".join(errors))
    print(
        "C/C++ assert parity: CLEAN "
        f"(41 units, {totals[0]} source asserts, {totals[1]} Rust debug assertions, "
        f"{totals[2]} other Rust assertions)"
    )
    return tuple(totals)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--upstream-root", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--authority", type=Path, required=True)
    parser.add_argument("--mutation-probe", choices=sorted(MUTATIONS))
    args = parser.parse_args()
    try:
        run_check(
            args.repo_root.resolve(),
            args.upstream_root.resolve(),
            args.manifest.resolve(),
            args.authority.resolve(),
            args.mutation_probe,
        )
    except (CheckError, OSError, subprocess.CalledProcessError, tomllib.TOMLDecodeError) as error:
        print(f"C/C++ assert parity: RED: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
