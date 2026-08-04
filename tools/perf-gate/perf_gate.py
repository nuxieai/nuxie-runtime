#!/usr/bin/env python3
"""Validate and operate the checked-in hot-loop performance ratchet."""

from __future__ import annotations

import argparse
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Any


MANIFEST_SCHEMA = "nuxie-perf-corpus-v1"
REQUIRED_DIVERSITY = {
    "text-heavy",
    "list-virtualization",
    "nested-artboards",
    "scripted",
    "layout-heavy",
}


@dataclass(frozen=True)
class PerfFile:
    id: str
    file_bytes: int
    categories: tuple[str, ...]
    note: str


@dataclass(frozen=True)
class PerfManifest:
    source: str
    minimum_files: int
    files: tuple[PerfFile, ...]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    for name, help_text in (
        ("check-manifest", "validate the perf corpus against corpus.toml"),
        ("ids", "print the validated comma-separated perf corpus ids"),
    ):
        command = subparsers.add_parser(name, help=help_text)
        command.add_argument("--manifest", type=Path, default=Path("perf-corpus.toml"))
        command.add_argument("--corpus", type=Path, default=Path("corpus.toml"))
        command.add_argument("--rive-runtime-dir", type=Path)

    options = parser.parse_args(argv)
    try:
        manifest = load_manifest(options.manifest)
        corpus = load_toml(options.corpus)
        validate_manifest(
            manifest,
            corpus,
            corpus_path=options.corpus,
            rive_runtime_dir=options.rive_runtime_dir,
        )
    except (OSError, ValueError, tomllib.TOMLDecodeError) as error:
        print(f"perf-gate error: {error}", file=sys.stderr)
        return 1

    if options.command == "ids":
        print(",".join(file.id for file in manifest.files))
    else:
        categories = sorted(
            {category for file in manifest.files for category in file.categories}
        )
        print(
            f"perf-corpus ok files={len(manifest.files)} "
            f"categories={','.join(categories)} source={manifest.source}"
        )
    return 0


def load_toml(path: Path) -> dict[str, Any]:
    with path.open("rb") as source:
        return tomllib.load(source)


def load_manifest(path: Path) -> PerfManifest:
    data = load_toml(path)
    if data.get("schema") != MANIFEST_SCHEMA:
        raise ValueError(
            f"{path}: schema must be {MANIFEST_SCHEMA!r}, got {data.get('schema')!r}"
        )
    source = require_nonempty_string(data, "source", path)
    minimum_files = data.get("minimum_files")
    if not isinstance(minimum_files, int) or isinstance(minimum_files, bool):
        raise ValueError(f"{path}: minimum_files must be an integer")
    if minimum_files < 20:
        raise ValueError(f"{path}: minimum_files must be at least 20")

    raw_files = data.get("file")
    if not isinstance(raw_files, list):
        raise ValueError(f"{path}: expected one or more [[file]] entries")
    files = []
    for index, raw_file in enumerate(raw_files, start=1):
        context = Path(f"{path} [[file]] #{index}")
        if not isinstance(raw_file, dict):
            raise ValueError(f"{context}: entry must be a table")
        file_id = require_nonempty_string(raw_file, "id", context)
        file_bytes = raw_file.get("bytes")
        if (
            not isinstance(file_bytes, int)
            or isinstance(file_bytes, bool)
            or file_bytes <= 0
        ):
            raise ValueError(f"{context}: bytes must be a positive integer")
        raw_categories = raw_file.get("categories")
        if not isinstance(raw_categories, list) or not raw_categories:
            raise ValueError(f"{context}: categories must be a non-empty string array")
        categories = tuple(raw_categories)
        if any(not isinstance(category, str) or not category for category in categories):
            raise ValueError(f"{context}: categories must be a non-empty string array")
        if len(categories) != len(set(categories)):
            raise ValueError(f"{context}: categories must not contain duplicates")
        note = require_nonempty_string(raw_file, "note", context)
        files.append(PerfFile(file_id, file_bytes, categories, note))
    return PerfManifest(source, minimum_files, tuple(files))


def require_nonempty_string(data: dict[str, Any], key: str, context: Path) -> str:
    value = data.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{context}: {key} must be a non-empty string")
    return value


def validate_manifest(
    manifest: PerfManifest,
    corpus: dict[str, Any],
    *,
    corpus_path: Path,
    rive_runtime_dir: Path | None,
) -> None:
    if manifest.source != corpus_path.name:
        raise ValueError(
            f"manifest source {manifest.source!r} does not match {corpus_path.name!r}"
        )
    if len(manifest.files) < manifest.minimum_files:
        raise ValueError(
            f"manifest has {len(manifest.files)} files; minimum is {manifest.minimum_files}"
        )
    ids = [file.id for file in manifest.files]
    if len(ids) != len(set(ids)):
        duplicates = sorted({file_id for file_id in ids if ids.count(file_id) > 1})
        raise ValueError(f"manifest contains duplicate ids: {','.join(duplicates)}")

    selected_categories = {
        category for file in manifest.files for category in file.categories
    }
    missing_categories = sorted(REQUIRED_DIVERSITY - selected_categories)
    if missing_categories:
        raise ValueError(
            "manifest is missing required diversity categories: "
            + ",".join(missing_categories)
        )

    corpus_files = corpus.get("file")
    if not isinstance(corpus_files, list):
        raise ValueError(f"{corpus_path}: expected [[file]] entries")
    corpus_by_id = {
        entry.get("id"): entry
        for entry in corpus_files
        if isinstance(entry, dict) and isinstance(entry.get("id"), str)
    }
    for file in manifest.files:
        source = corpus_by_id.get(file.id)
        if source is None:
            raise ValueError(f"manifest id {file.id!r} is absent from {corpus_path}")
        if source.get("status") != "exact":
            raise ValueError(
                f"manifest id {file.id!r} must remain exact in {corpus_path}"
            )
        if source.get("input_script") is not None:
            raise ValueError(
                f"manifest id {file.id!r} has input_script; the perf method requires none"
            )
        if rive_runtime_dir is not None:
            source_path = source.get("path")
            if not isinstance(source_path, str) or not source_path:
                raise ValueError(f"corpus id {file.id!r} has no source path")
            actual_bytes = (rive_runtime_dir / source_path).stat().st_size
            if actual_bytes != file.file_bytes:
                raise ValueError(
                    f"manifest id {file.id!r} records {file.file_bytes} bytes; "
                    f"{source_path} has {actual_bytes}"
                )


if __name__ == "__main__":
    raise SystemExit(main())
