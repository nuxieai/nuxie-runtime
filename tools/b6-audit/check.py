#!/usr/bin/env python3
"""Mechanical closure checks for the pinned B-6 structural audit."""

from __future__ import annotations

import collections
import pathlib
import tomllib


ROOT = pathlib.Path(__file__).resolve().parents[2]
MANIFEST = ROOT / "file-correspondence-manifest.toml"
# The B-6 structural audit is a frozen snapshot: it was performed at
# d788e8ec and stays anchored there, which is what `audit_upstream_ref`
# records. The live pin (`upstream_ref`) advances independently with each
# upstream sync, so it is deliberately NOT what this gate compares.
EXPECTED_AUDIT_REF = "d788e8ec6e8b598526607d6a1e8818e8b637b60c"
# The census below is a deliberate ratchet: an upstream sync that adds,
# removes, or reclassifies a manifest row is meant to fail this gate until
# the numbers are re-stated here on purpose. Updating it is a one-liner, and
# NOT doing so is what kept CI red for two days after the 4ac7b327 advance:
#
#   python3 -c 'import collections,tomllib,pathlib; \
#     d=tomllib.loads(pathlib.Path("file-correspondence-manifest.toml").read_text()); \
#     r=d["file"]; print(len(r), collections.Counter(x.get("b6_verdict") for x in r))'
#
# Rows the census reports under `None` are unaudited: add them to
# POST_AUDIT_UNAUDITED below (and to the register follow-up that tracks it),
# not to EXPECTED_COUNTS.
EXPECTED_ROW_COUNT = 456
EXPECTED_COUNTS = {
    "ISOMORPHIC": 22,
    "ADAPTED": 204,
    "DIVERGENT": 153,
    "TRACKED-GAP": 30,
    "N/A": 37,
}
# Upstream files that appeared after the audit ref and therefore carry no
# B-6 verdict. Listing them by name (rather than letting the census absorb
# them) keeps the unaudited surface visible: any further verdict-less row
# fails this gate until it is audited or added here deliberately.
#
# This set is tracked debt, not a permanent exemption --- register row #H5
# owns shrinking it. Audit a row into a verdict, delete it here, and restate
# EXPECTED_COUNTS; #H5 closes when this set is empty.
POST_AUDIT_UNAUDITED = {
    "src/animation/keyframe_int.cpp",
    "src/component_origin.cpp",
    "src/core/field_types/core_int_type.cpp",
    "src/data_bind/context/context_value_asset_blob.cpp",
    "src/layout/grid_item_placement.cpp",
    "src/layout/grid_track.cpp",
    "src/layout/layout_participant.cpp",
    "src/layout/layout_sizing_style.cpp",
    "src/viewmodel/runtime/viewmodel_instance_asset_blob_runtime.cpp",
    "src/viewmodel/viewmodel_instance_asset_blob.cpp",
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
        "B6-0301",
        "B6-0323",
        "B6-0324",
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
        "B6-0322",
        "B6-0325",
        "B6-0326",
        "B6-0339",
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
    if data.get("audit_upstream_ref") != EXPECTED_AUDIT_REF:
        fail(
            f"audit_upstream_ref is {data.get('audit_upstream_ref')!r}, "
            f"expected {EXPECTED_AUDIT_REF}"
        )
    if data.get("row_count") != EXPECTED_ROW_COUNT or len(rows) != EXPECTED_ROW_COUNT:
        fail(
            f"expected {EXPECTED_ROW_COUNT} rows, manifest declares "
            f"{data.get('row_count')} and has {len(rows)}"
        )

    unaudited = {row["upstream"] for row in rows if row.get("b6_verdict") is None}
    if unaudited != POST_AUDIT_UNAUDITED:
        missing = sorted(POST_AUDIT_UNAUDITED - unaudited)
        extra = sorted(unaudited - POST_AUDIT_UNAUDITED)
        fail(f"unaudited row set drifted (unexpected: {extra}, now audited: {missing})")

    audited = [row for row in rows if row.get("b6_verdict") is not None]
    ids = [row.get("b6_row_id") for row in audited]
    if len(set(ids)) != len(ids):
        duplicates = sorted(
            row_id for row_id, count in collections.Counter(ids).items() if count > 1
        )
        fail(f"duplicate row ids: {duplicates}")

    counts = collections.Counter(row.get("b6_verdict") for row in audited)
    if counts.get("UNKNOWN", 0):
        fail(f"{counts['UNKNOWN']} UNKNOWN rows remain")
    if dict(counts) != EXPECTED_COUNTS:
        fail(f"verdict census is {dict(counts)}, expected {EXPECTED_COUNTS}")

    by_id = {row["b6_row_id"]: row for row in audited}
    for verdict, row_ids in SECOND_PASS_VERDICTS.items():
        for row_id in sorted(row_ids):
            row = by_id[row_id]
            if row.get("b6_verdict") != verdict:
                fail(f"{row_id} must remain {verdict}, got {row.get('b6_verdict')}")
            if row.get("audit_record") != "docs/b6-audit/SECOND_PASS.md":
                fail(f"{row_id} does not cite SECOND_PASS.md")

    for row in audited:
        if row.get("b6_verdict") != "TRACKED-GAP":
            continue
        note = row.get("note", "")
        if not any(token in note for token in OWNER_TOKENS):
            fail(f"{row['b6_row_id']} TRACKED-GAP note has no register owner")
    print(
        f"B-6 audit closure: {len(audited)} audited rows "
        f"({len(POST_AUDIT_UNAUDITED)} post-audit rows pending), zero UNKNOWN, "
        "exact second-pass dispositions and owners verified"
    )


if __name__ == "__main__":
    main()
