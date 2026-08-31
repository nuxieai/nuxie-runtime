#!/usr/bin/env python3
"""Derive the complete Metal-compatible corpus without changing row policy.

The source manifest remains the single authority for IDs, streams, frames, and
predeclared tolerances. This tool only removes non-Metal MSAA rows and proves
the expected denominator before publishing the derived manifest.
"""

from __future__ import annotations

import argparse
from pathlib import Path
import tomllib


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--expected", type=int, required=True)
    args = parser.parse_args()

    source = args.input.read_text(encoding="utf-8")
    blocks = source.split("[[entry]]")
    selected: list[str] = []
    selected_ids: set[str] = set()
    for body in blocks[1:]:
        block = "[[entry]]" + body
        entry = tomllib.loads(block)["entry"][0]
        if entry["mode"] != "clockwise-atomic":
            continue
        entry_id = entry["id"]
        if entry_id in selected_ids:
            raise SystemExit(f"duplicate clockwise-atomic corpus ID: {entry_id}")
        selected_ids.add(entry_id)
        selected.append(block.strip())

    if len(selected) != args.expected:
        raise SystemExit(
            f"clockwise-atomic corpus has {len(selected)} rows; expected {args.expected}"
        )

    output = (
        "# Generated from corpus-r.toml by derive_clockwise_atomic_manifest.py.\n"
        "# Do not edit: source rows and tolerances remain authoritative.\n\n"
        + "\n\n".join(selected)
        + "\n"
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(output, encoding="utf-8")
    print(f"derived clockwise-atomic corpus rows={len(selected)} output={args.output}")


if __name__ == "__main__":
    main()
