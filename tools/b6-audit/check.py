#!/usr/bin/env python3
"""Mechanical closure checks for the pinned B-6 structural audit."""

from __future__ import annotations

import collections
import pathlib
import tomllib


ROOT = pathlib.Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "file-correspondence-manifest.toml"
EXPECTED_REF = "d788e8ec6e8b598526607d6a1e8818e8b637b60c"
EXPECTED_COUNTS = {
    "ISOMORPHIC": 19,
    "ADAPTED": 192,
    "DIVERGENT": 157,
    "TRACKED-GAP": 30,
    "N/A": 49,
}
SECOND_PASS_VERDICTS = {
    "TRACKED-GAP": {
        "B6-0027",
        "B6-0046",
        "B6-0047",
        "B6-0049",
        "B6-0050",
        "B6-0051",
        "B6-0052",
        "B6-0098",
        "B6-0103",
        "B6-0127",
        "B6-0134",
        "B6-0139",
        "B6-0140",
        "B6-0246",
        "B6-0260",
        "B6-0267",
        "B6-0270",
        "B6-0301",
        "B6-0323",
        "B6-0324",
        "B6-0339",
        "B6-0378",
        "B6-0384",
        "B6-0388",
        "B6-0389",
        "B6-0391",
        "B6-0392",
        "B6-0398",
        "B6-0401",
        "B6-0406",
    },
    "ADAPTED": {
        "B6-0067",
        "B6-0104",
        "B6-0106",
        "B6-0249",
        "B6-0255",
        "B6-0321",
        "B6-0340",
        "B6-0341",
        "B6-0370",
        "B6-0375",
    },
    "DIVERGENT": {
        "B6-0209",
        "B6-0238",
        "B6-0240",
        "B6-0315",
        "B6-0316",
        "B6-0317",
        "B6-0318",
        "B6-0322",
        "B6-0325",
        "B6-0326",
        "B6-0355",
    },
    "N/A": {"B6-0295"},
}
OWNER_TOKENS = (
    "RB-",
    "F1",
    "F2",
    "F4",
    "F5",
    "F7",
    "F9",
    "F10",
    "F13",
    "A1",
    "A2",
    "C1",
)


def fail(message: str) -> None:
    raise SystemExit(f"B-6 audit check failed: {message}")


def main() -> None:
    data = tomllib.loads(MANIFEST.read_text())
    rows = data.get("file", [])
    if data.get("upstream_ref") != EXPECTED_REF:
        fail(f"upstream_ref is {data.get('upstream_ref')!r}, expected {EXPECTED_REF}")
    if data.get("row_count") != 447 or len(rows) != 447:
        fail(f"expected 447 rows, manifest declares {data.get('row_count')} and has {len(rows)}")

    ids = [row.get("b6_row_id") for row in rows]
    if len(set(ids)) != len(ids):
        duplicates = sorted(
            row_id for row_id, count in collections.Counter(ids).items() if count > 1
        )
        fail(f"duplicate row ids: {duplicates}")

    counts = collections.Counter(row.get("b6_verdict") for row in rows)
    if counts.get("UNKNOWN", 0):
        fail(f"{counts['UNKNOWN']} UNKNOWN rows remain")
    if dict(counts) != EXPECTED_COUNTS:
        fail(f"verdict census is {dict(counts)}, expected {EXPECTED_COUNTS}")

    by_id = {row["b6_row_id"]: row for row in rows}
    for verdict, row_ids in SECOND_PASS_VERDICTS.items():
        for row_id in sorted(row_ids):
            row = by_id[row_id]
            if row.get("b6_verdict") != verdict:
                fail(f"{row_id} must remain {verdict}, got {row.get('b6_verdict')}")
            if row.get("audit_record") != "docs/b6-audit/SECOND_PASS.md":
                fail(f"{row_id} does not cite SECOND_PASS.md")

    for row in rows:
        if row.get("b6_verdict") != "TRACKED-GAP":
            continue
        note = row.get("note", "")
        if not any(token in note for token in OWNER_TOKENS):
            fail(f"{row['b6_row_id']} TRACKED-GAP note has no register owner")
    print(
        "B-6 audit closure: 447 rows, zero UNKNOWN, exact second-pass "
        "dispositions and owners verified"
    )


if __name__ == "__main__":
    main()
