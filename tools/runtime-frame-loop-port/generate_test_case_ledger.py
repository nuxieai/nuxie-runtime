#!/usr/bin/env python3
"""Generate the truthful all-pending case ledger from the pinned C++ source."""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import sys
import tomllib

from check_test_correspondence import (
    CASE_ADAPTATION_KINDS,
    CASE_EVIDENCE_KINDS,
    CASE_OUTCOMES,
    CASE_STATUSES,
    SOURCE_GLOBS,
    CheckFailure,
    git_output,
    pinned_sources,
    upstream_case_census,
)


def parse_args(argv: list[str]) -> argparse.Namespace:
    default_root = pathlib.Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=pathlib.Path, default=default_root)
    parser.add_argument(
        "--rive-runtime-dir",
        type=pathlib.Path,
        default=pathlib.Path(
            os.environ.get("RIVE_RUNTIME_DIR", "/Users/levi/dev/oss/rive-runtime")
        ),
    )
    parser.add_argument("--manifest", type=pathlib.Path)
    parser.add_argument("--output", type=pathlib.Path)
    parser.add_argument(
        "--force",
        action="store_true",
        help="replace an existing ledger (this discards all case proof)",
    )
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(sys.argv[1:] if argv is None else argv)
    manifest_path = args.manifest or args.repo_root / "test-correspondence-manifest.toml"
    try:
        with manifest_path.open("rb") as source:
            manifest = tomllib.load(source)
    except (OSError, tomllib.TOMLDecodeError) as error:
        print(f"case-ledger generation failed: {error}", file=sys.stderr)
        return 1
    ref = manifest.get("upstream_ref")
    ledger_name = manifest.get("case_ledger")
    if not isinstance(ref, str) or not isinstance(ledger_name, str):
        print(
            "case-ledger generation failed: manifest needs upstream_ref and case_ledger",
            file=sys.stderr,
        )
        return 1
    output = args.output or args.repo_root / ledger_name
    if output.exists() and not args.force:
        print(
            f"case-ledger generation refused to overwrite {output}; pass --force only "
            "when intentionally resetting every case to pending",
            file=sys.stderr,
        )
        return 1
    try:
        actual_ref = git_output(args.rive_runtime_dir, "rev-parse", "HEAD").strip()
        if actual_ref != ref:
            raise CheckFailure(
                f"upstream pin mismatch: manifest={ref}, checkout={actual_ref}"
            )
        cases = upstream_case_census(pinned_sources(args.rive_runtime_dir, ref))
    except CheckFailure as error:
        print(f"case-ledger generation failed: {error}", file=sys.stderr)
        return 1
    document = {
        "schema": "nuxie-test-case-correspondence/v1",
        "schema_version": 1,
        "upstream_ref": ref,
        "source_globs": list(SOURCE_GLOBS),
        "case_count": len(cases),
        "status_values": list(CASE_STATUSES),
        "outcome_values": list(CASE_OUTCOMES),
        "evidence_kinds": list(CASE_EVIDENCE_KINDS),
        "adaptation_kinds": list(CASE_ADAPTATION_KINDS),
        "ratchet": {"max_pending": len(cases)},
        "cases": [
            {
                "id": case.case_id,
                "upstream": case.upstream,
                "ordinal": case.ordinal,
                "line": case.line,
                "name": case.name,
                "status": "pending",
                "outcome": "unverified",
                "evidence": [],
            }
            for case in cases
        ],
    }
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(document, indent=2, ensure_ascii=False) + "\n")
    print(f"wrote {len(cases)} pending cases to {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
