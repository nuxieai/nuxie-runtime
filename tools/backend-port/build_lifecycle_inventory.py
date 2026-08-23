#!/usr/bin/env python3
"""Derive ownership, synchronization, failure, and teardown evidence lines."""

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
    "event_class",
    "matched_token",
    "source_evidence",
    "ownership_unit",
    "review_status",
)
PATTERNS = {
    "construction-or-allocation": re.compile(
        r"\b(?:new|Make|make_unique|make_shared|create|Create|allocate|Allocate|acquire|Acquire)[A-Za-z0-9_]*\b"
    ),
    "destruction-or-release": re.compile(
        r"(?:~[A-Za-z_][A-Za-z0-9_]*\s*\(|\b(?:delete|destroy|Destroy|release|Release|free|Free|reset|Reset)[A-Za-z0-9_]*\b)"
    ),
    "mapping-or-host-visibility": re.compile(
        r"\b(?:map|Map|unmap|Unmap|flushMapped|invalidateMapped)[A-Za-z0-9_]*\b"
    ),
    "synchronization-or-submission": re.compile(
        r"\b(?:wait|Wait|fence|Fence|semaphore|Semaphore|barrier|Barrier|submit|Submit|flush|Flush|finish|Finish)[A-Za-z0-9_]*\b"
    ),
    "callback-or-async": re.compile(
        r"\b(?:callback|Callback|Async|async|deviceLost|DeviceLost|uncapturedError|UncapturedError)[A-Za-z0-9_]*\b"
    ),
    "thread-or-lock": re.compile(
        r"\b(?:thread|Thread|mutex|Mutex|lock_guard|unique_lock|atomic|Atomic)[A-Za-z0-9_]*\b"
    ),
    "failure-or-loss": re.compile(
        r"\b(?:assert|RIVE_[A-Z_]*ASSERT|CHECK|Error|error|Failed|FAILED|failure|Failure|lost|Lost|OutOfMemory)[A-Za-z0-9_]*\b"
    ),
}


@dataclass(frozen=True, order=True)
class Row:
    campaign: str
    source_path: str
    line: int
    event_class: str
    matched_token: str
    source_evidence: str
    ownership_unit: str
    review_status: str = "review-required"

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


def render(upstream_root: Path, ownership_path: Path) -> str:
    with ownership_path.open(newline="") as handle:
        sources = list(csv.DictReader(handle, delimiter="\t"))
    rows: list[Row] = []
    for source in sources:
        if source["source_role"] not in {
            "declaration",
            "implementation",
            "platform-implementation",
            "compatibility-build-input",
        }:
            continue
        path = upstream_root / source["source_path"]
        if digest(path) != source["source_sha256"]:
            raise ValueError(f"pinned source drift: {source['source_path']}")
        for line_number, raw_line in enumerate(path.read_text(errors="replace").splitlines(), 1):
            evidence = " ".join(raw_line.strip().split()).replace("\t", " ")
            if not evidence or evidence.startswith("//"):
                continue
            for event_class, pattern in PATTERNS.items():
                tokens = sorted(set(match.group(0) for match in pattern.finditer(raw_line)))
                for token in tokens:
                    rows.append(
                        Row(
                            source["campaign"],
                            source["source_path"],
                            line_number,
                            event_class,
                            token,
                            evidence,
                            source["ownership_unit"],
                            "authority-recorded"
                            if source["port_disposition"]
                            in {
                                "dependency-authority",
                                "evidence-only",
                                "source-exclusion-non-webgl2-build",
                            }
                            else "review-required",
                        )
                    )
    rows.sort()
    if not rows:
        raise ValueError("lifecycle extraction found no evidence")
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
            print("backend lifecycle inventory is stale", file=sys.stderr)
            return 1
        print(f"backend lifecycle inventory clean: {count} evidence rows")
        return 0
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(rendered)
    print(f"wrote {count} backend lifecycle evidence rows to {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
