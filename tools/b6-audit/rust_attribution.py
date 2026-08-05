#!/usr/bin/env python3
"""Require every in-scope Rust source to have a provenance classification."""

from __future__ import annotations

import argparse
import collections
import pathlib
import tomllib


CRATE_ROOTS = (
    "crates/nuxie-audio/src",
    "crates/nuxie-runtime/src",
    "crates/nuxie/src",
    "crates/nuxie-scripting/src",
    "crates/nuxie-binary/src",
    "crates/nuxie-render-api/src",
)
CATEGORY_VALUES = (
    "baseline-adaptation",
    "product-authoring",
    "product-host",
    "product-data",
    "product-trust",
    "mixed-product-host",
    "retained-render",
    "codegen",
    "test-infra",
)
CATEGORIES = set(CATEGORY_VALUES)
ADDITIONS_SCHEMA = "nuxie-rust-additions/v1"
ADDITIONS_SCHEMA_VERSION = 1


def rust_modules(rows: list[dict[str, object]]) -> set[str]:
    return {
        module.strip()
        for row in rows
        for module in str(row.get("rust_module", "")).split(";")
        if module.strip()
    }


def is_test_only_source(path: pathlib.Path) -> bool:
    return (
        path.stem == "tests"
        or path.stem.endswith("_tests")
        or "tests" in path.parts
    )


def check_rust_attribution(
    repo_root: pathlib.Path,
    manifest_path: pathlib.Path,
    additions_path: pathlib.Path,
) -> list[str]:
    manifest = tomllib.loads(manifest_path.read_text())
    additions = tomllib.loads(additions_path.read_text())
    attributed = rust_modules(manifest.get("file", []))
    addition_rows = additions.get("addition", [])
    sources = {
        path.relative_to(repo_root).as_posix()
        for root in CRATE_ROOTS
        for path in (repo_root / root).rglob("*.rs")
        if not is_test_only_source(path.relative_to(repo_root / root))
    }
    errors = []
    if additions.get("schema") != ADDITIONS_SCHEMA:
        errors.append(
            f"invalid rust-additions schema: expected {ADDITIONS_SCHEMA}"
        )
    if additions.get("schema_version") != ADDITIONS_SCHEMA_VERSION:
        errors.append(
            "invalid rust-additions schema_version: "
            f"expected {ADDITIONS_SCHEMA_VERSION}"
        )
    if additions.get("category_values") != list(CATEGORY_VALUES):
        errors.append(
            "invalid rust-additions category_values: expected "
            + ", ".join(CATEGORY_VALUES)
        )
    for row in addition_rows:
        path = str(row.get("path", ""))
        category = str(row.get("category", ""))
        if category not in CATEGORIES:
            errors.append(f"invalid category for {path}: {category}")
        if path not in sources:
            errors.append(f"classified Rust source does not exist: {path}")
    path_counts = collections.Counter(str(row.get("path", "")) for row in addition_rows)
    duplicates = sorted(path for path, count in path_counts.items() if count > 1)
    if duplicates:
        errors.append(f"duplicate classified Rust paths: {', '.join(duplicates)}")
    classified = {str(row.get("path", "")) for row in addition_rows}
    overlap = sorted(attributed & classified)
    if overlap:
        errors.append(
            "Rust paths are both attributed and classified as additions: "
            + ", ".join(overlap)
        )
    unknown = sorted(sources - attributed - classified)
    if unknown:
        errors.append(f"unclassified Rust files: {', '.join(unknown)}")
    return errors


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument("--manifest", type=pathlib.Path, required=True)
    parser.add_argument("--additions", type=pathlib.Path, required=True)
    args = parser.parse_args()

    errors = check_rust_attribution(args.repo_root, args.manifest, args.additions)
    if errors:
        raise SystemExit(
            "Rust attribution check failed: " + "\nRust attribution check failed: ".join(errors)
        )
    print("Rust attribution coverage: every in-scope Rust source is classified")


if __name__ == "__main__":
    main()
