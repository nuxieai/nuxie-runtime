#!/usr/bin/env python3
"""Derive backend configuration branches and API capability symbols."""

from __future__ import annotations

import argparse
import csv
import hashlib
import re
import sys
from dataclasses import dataclass
from pathlib import Path


HEADER = (
    "campaign",
    "source_path",
    "line",
    "occurrence_count",
    "occurrence_lines",
    "authority_kind",
    "token",
    "enclosing_condition",
    "ownership_unit",
    "review_status",
)
DIRECTIVE = re.compile(r"^\s*#\s*(if|ifdef|ifndef|elif|else|endif)\b(.*)$")
CAPABILITY = re.compile(
    r"\b(?:VK_[A-Za-z0-9_]+|WGPU[A-Za-z0-9_]+|GL_[A-Za-z0-9_]+|"
    r"RIVE_[A-Za-z0-9_]+|ORE_BACKEND_[A-Za-z0-9_]+|PLS_IMPL_[A-Za-z0-9_]+|"
    r"TARGET_[A-Za-z0-9_]+|USE_WEBGPU_[A-Za-z0-9_]+|"
    r"DISABLE_[A-Za-z0-9_]+|FIXED_FUNCTION_[A-Za-z0-9_]+)\b"
)


@dataclass(frozen=True, order=True)
class Row:
    campaign: str
    source_path: str
    line: int
    occurrence_count: int
    occurrence_lines: str
    authority_kind: str
    token: str
    enclosing_condition: str
    ownership_unit: str
    review_status: str

    def tsv(self) -> str:
        return "\t".join(str(getattr(self, column)) for column in HEADER)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--upstream-root", type=Path, required=True)
    parser.add_argument("--ownership-inventory", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    return parser.parse_args()


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def condition(stack: list[str]) -> str:
    return " && ".join(stack) if stack else "all"


def render(upstream_root: Path, ownership_inventory: Path) -> str:
    with ownership_inventory.open(newline="") as handle:
        sources = list(csv.DictReader(handle, delimiter="\t"))
    rows: list[Row] = []
    symbols: dict[tuple[str, str, str, str], list[tuple[int, str]]] = {}
    for source in sources:
        path = upstream_root / source["source_path"]
        if digest(path) != source["source_sha256"]:
            raise ValueError(f"pinned source drift: {source['source_path']}")
        stack: list[str] = []
        for line_number, line in enumerate(path.read_text(errors="replace").splitlines(), 1):
            match = DIRECTIVE.match(line)
            if match:
                directive = match.group(1)
                expression = " ".join(match.group(2).strip().split()) or "-"
                rows.append(
                    Row(
                        source["campaign"],
                        source["source_path"],
                        line_number,
                        1,
                        str(line_number),
                        f"preprocessor-{directive}",
                        expression,
                        condition(stack),
                        source["ownership_unit"],
                        "derived",
                    )
                )
                if directive in {"if", "ifdef", "ifndef"}:
                    stack.append(f"{directive} {expression}")
                elif directive in {"elif", "else"}:
                    if not stack:
                        raise ValueError(
                            f"orphan #{directive}: {source['source_path']}:{line_number}"
                        )
                    stack[-1] = f"{directive} {expression}"
                elif directive == "endif":
                    if not stack:
                        raise ValueError(f"orphan #endif: {source['source_path']}:{line_number}")
                    stack.pop()
            for token in sorted(set(CAPABILITY.findall(line))):
                key = (
                    source["campaign"],
                    source["source_path"],
                    token,
                    source["ownership_unit"],
                )
                symbols.setdefault(key, []).append((line_number, condition(stack)))
        if stack:
            raise ValueError(f"unterminated preprocessor branch: {source['source_path']}")
    for (campaign, source_path, token, unit), occurrences in symbols.items():
        lines = [line for line, _ in occurrences]
        enclosing = ";".join(sorted(set(value for _, value in occurrences)))
        rows.append(
            Row(
                campaign,
                source_path,
                min(lines),
                len(lines),
                ",".join(str(line) for line in lines),
                "configuration-or-capability-symbol",
                token,
                enclosing,
                unit,
                "review-required",
            )
        )
    rows.sort()
    return "\n".join(("\t".join(HEADER), *(row.tsv() for row in rows))) + "\n"


def repo_path(repo_root: Path, path: Path) -> Path:
    return path if path.is_absolute() else repo_root / path


def main() -> int:
    args = parse_args()
    ownership = repo_path(args.repo_root, args.ownership_inventory)
    output = repo_path(args.repo_root, args.output)
    rendered = render(args.upstream_root, ownership)
    count = len(rendered.splitlines()) - 1
    if args.check:
        if not output.is_file() or output.read_text() != rendered:
            print("backend configuration inventory is stale", file=sys.stderr)
            return 1
        print(f"backend configuration inventory clean: {count} rows")
        return 0
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(rendered)
    print(f"wrote {count} backend configuration rows to {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
