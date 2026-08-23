#!/usr/bin/env python3
"""Replay one receipt's byte, citation, and campaign-contract evidence."""

from __future__ import annotations

import argparse
import csv
import importlib.util
import pathlib
import sys
import tomllib


def load_checker(repo_root: pathlib.Path):
    path = repo_root / "tools" / "metal-port" / "check.py"
    spec = importlib.util.spec_from_file_location("metal_port_receipt_check", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load checker {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument("--upstream-root", type=pathlib.Path, required=True)
    parser.add_argument("--manifest", type=pathlib.Path, required=True)
    parser.add_argument("--receipt", type=pathlib.Path, required=True)
    args = parser.parse_args()
    repo_root = args.repo_root.resolve()
    upstream_root = args.upstream_root.resolve()
    manifest_path = args.manifest.resolve()
    receipt_path = args.receipt
    if not receipt_path.is_absolute():
        receipt_path = repo_root / receipt_path
    with manifest_path.open("rb") as source:
        manifest = tomllib.load(source)
    with receipt_path.open("rb") as source:
        receipt = tomllib.load(source)
    unit_id = str(receipt.get("unit", ""))
    units = {
        str(unit.get("id", "")): unit for unit in manifest.get("translation_unit", [])
    }
    unit = units.get(unit_id)
    divergence_row = None
    if unit is None and unit_id.startswith("divergence-"):
        ledger_path = repo_root / str(manifest.get("divergence_ledger", ""))
        try:
            with ledger_path.open(encoding="utf-8", newline="") as source:
                divergence_rows = list(csv.DictReader(source, delimiter="\t"))
        except (OSError, csv.Error) as error:
            print(f"cannot read divergence ledger: {error}", file=sys.stderr)
            return 1
        divergence_id = unit_id.removeprefix("divergence-")
        divergence_row = next(
            (row for row in divergence_rows if row.get("id") == divergence_id), None
        )
        if divergence_row is not None:
            citations = [
                value.strip()
                for value in str(divergence_row.get("evidence", "")).split(";")
                if value.strip()
            ]
            unit = {
                "id": unit_id,
                "base_ref": str(divergence_row.get("upstream_sha", "")),
                "sources": sorted(
                    {
                        value.removeprefix("cpp:").rsplit(":", 1)[0]
                        for value in citations
                        if value.startswith("cpp:")
                    }
                ),
                "rust_targets": [str(divergence_row.get("rust_owner", ""))],
                "artifact_targets": [],
            }
    if unit is None:
        print(f"unknown receipt unit {unit_id}", file=sys.stderr)
        return 1
    kind_to_field = {
        "translation": "translation_receipt",
        "source-review": "source_review_receipt",
        "ownership-review": "ownership_review_receipt",
        "fix": "fix_receipt",
        "compile": "compile_receipt",
        "verification": "verification_receipt",
    }
    field = kind_to_field.get(str(receipt.get("receipt_kind", "")))
    if field is None:
        print("unknown receipt kind", file=sys.stderr)
        return 1
    checker = load_checker(repo_root)
    checker.REPLAY_RECEIPT_COMMANDS = False
    errors: list[str] = []
    checker.validate_receipt_contents(
        receipt_path,
        unit_id,
        field,
        str(unit.get("base_ref", "")),
        errors,
        repo_root=repo_root,
        upstream_root=upstream_root,
        expected_sources=unit.get("sources", []),
        expected_artifacts=[
            *unit.get("rust_targets", []),
            *unit.get("artifact_targets", []),
        ],
        required_cpp_ranges=(
            [
                value.strip()
                for value in str(divergence_row.get("evidence", "")).split(";")
                if value.strip().startswith("cpp:")
            ]
            if divergence_row is not None
            else []
        ),
        required_rust_owner=(
            str(divergence_row.get("rust_owner", ""))
            if divergence_row is not None
            else ""
        ),
        require_scoped_evidence=divergence_row is not None,
    )
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    checked = len(unit.get("sources", [])) + len(unit.get("rust_targets", [])) + len(
        unit.get("artifact_targets", [])
    )
    print(max(checked, 1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
