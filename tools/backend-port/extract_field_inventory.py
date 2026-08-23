#!/usr/bin/env python3
"""Derive all state-bearing backend fields from Clang ASTs."""

from __future__ import annotations

import argparse
import bisect
import csv
import hashlib
import json
import subprocess
import sys
import tempfile
import tomllib
from dataclasses import dataclass
from pathlib import Path


HEADER = (
    "campaign",
    "configuration",
    "source_path",
    "qualified_type",
    "field_order",
    "field_name",
    "declared_type",
    "declaration_line",
    "ownership_unit",
    "ownership_review",
)


@dataclass(frozen=True, order=True)
class Field:
    campaign: str
    configuration: str
    source_path: str
    qualified_type: str
    field_order: int
    field_name: str
    declared_type: str
    declaration_line: int
    ownership_unit: str
    ownership_review: str

    def tsv(self) -> str:
        return "\t".join(str(getattr(self, column)) for column in HEADER)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--upstream-root", type=Path, required=True)
    parser.add_argument("--ownership-inventory", type=Path, required=True)
    parser.add_argument("--profiles", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    return parser.parse_args()


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def repo_path(repo_root: Path, path: Path) -> Path:
    return path if path.is_absolute() else repo_root / path


def ownership_rows(path: Path) -> list[dict[str, str]]:
    with path.open(newline="") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def source_from_location(
    location: object,
    upstream_root: Path,
    allowed: set[str],
    inherited: str | None,
) -> str | None:
    if not isinstance(location, dict):
        return inherited
    filename = location.get("file")
    if isinstance(filename, str):
        try:
            relative = Path(filename).resolve().relative_to(upstream_root).as_posix()
        except ValueError:
            return None
        return relative if relative in allowed else None
    return inherited


def line_for(node: dict, source: str, starts: dict[str, list[int]]) -> int:
    location = node.get("loc")
    if isinstance(location, dict):
        line = location.get("line")
        if isinstance(line, int):
            return line
        offset = location.get("offset")
        if isinstance(offset, int):
            return bisect.bisect_right(starts[source], offset)
    return 0


def review_class(declared_type: str) -> str:
    compact = declared_type.replace(" ", "")
    if "std::function" in compact or "Callback" in compact:
        return "callback-or-async-review-required"
    if compact.startswith("Vk") or "WGPU" in compact or compact.startswith("GLuint"):
        return "native-handle-review-required"
    if "*" in compact or "&" in compact or "span<" in compact:
        return "borrow-or-pointer-review-required"
    if any(token in compact for token in ("unique_ptr<", "shared_ptr<", "rcp<", "Ref<")):
        return "retained-owner-review-required"
    if any(token in compact for token in ("vector<", "array<", "unordered_map<", "map<")):
        return "container-owner-review-required"
    return "value-lifetime-review-required"


def extract_fields(
    ast: dict,
    profile: dict,
    upstream_root: Path,
    source_units: dict[str, str],
    starts: dict[str, list[int]],
) -> list[Field]:
    campaign = profile["campaign"]
    allowed = set(source_units)
    fields: list[Field] = []

    def visit(node: dict, parents: tuple[str, ...], inherited_source: str | None) -> None:
        source = source_from_location(node.get("loc"), upstream_root, allowed, inherited_source)
        kind = node.get("kind")
        name = node.get("name")
        next_parents = parents
        if kind in {"NamespaceDecl", "ClassTemplateDecl"} and isinstance(name, str):
            next_parents = (*parents, name)
        elif kind == "CXXRecordDecl" and node.get("completeDefinition"):
            record_line = line_for(node, source, starts) if source else 0
            record_name = name if isinstance(name, str) and name else f"<anonymous@{record_line}>"
            qualified = "::".join((*parents, record_name))
            children = node.get("inner")
            if not isinstance(children, list):
                children = []
            if source in allowed:
                order = 0
                for child in children:
                    if not isinstance(child, dict) or child.get("kind") != "FieldDecl":
                        continue
                    order += 1
                    field_name = child.get("name")
                    field_line = line_for(child, source, starts)
                    if not isinstance(field_name, str) or not field_name:
                        field_name = f"<anonymous@{field_line}>"
                    type_info = child.get("type")
                    declared_type = "<unknown>"
                    if isinstance(type_info, dict):
                        candidate = type_info.get("qualType")
                        if isinstance(candidate, str):
                            declared_type = candidate
                    fields.append(
                        Field(
                            campaign,
                            profile["id"],
                            source,
                            qualified,
                            order,
                            field_name,
                            declared_type,
                            field_line,
                            source_units[source],
                            review_class(declared_type),
                        )
                    )
            next_parents = (*parents, record_name)
        children = node.get("inner")
        if isinstance(children, list):
            for child in children:
                if isinstance(child, dict):
                    visit(child, next_parents, source)

    visit(ast, (), None)
    return fields


def compile_profile(
    profile: dict,
    upstream_root: Path,
    rows: list[dict[str, str]],
) -> list[Field]:
    campaign_rows = [row for row in rows if row["campaign"] == profile["campaign"]]
    headers = [row for row in campaign_rows if row["source_role"] == "declaration"]
    source_units = {row["source_path"]: row["ownership_unit"] for row in headers}
    starts: dict[str, list[int]] = {}
    for row in headers:
        path = upstream_root / row["source_path"]
        if digest(path) != row["source_sha256"]:
            raise ValueError(f"pinned source drift: {row['source_path']}")
        data = path.read_bytes()
        starts[row["source_path"]] = [0, *(index + 1 for index, byte in enumerate(data) if byte == 10)]

    with tempfile.TemporaryDirectory(prefix=f"backend-fields-{profile['id']}-") as temporary:
        temp = Path(temporary)
        for stub in profile.get("stub_headers", []):
            stub_path = temp / stub
            stub_path.parent.mkdir(parents=True, exist_ok=True)
            stub_path.write_text("#pragma once\n")
        translation_unit = temp / "all_headers.cpp"
        translation_unit.write_text(
            "".join(f'#include "{row["source_path"]}"\n' for row in headers)
        )
        command = [
            "xcrun",
            "--sdk",
            "macosx",
            "clang++",
            "-x",
            "c++",
            "-std=c++17",
            "-fsyntax-only",
            "-Xclang",
            "-ast-dump=json",
            *(f"-D{define}" for define in profile["defines"]),
            "-I",
            str(temp),
            *(
                argument
                for root in profile["include_roots"]
                for argument in ("-I", str(upstream_root / root))
            ),
            str(translation_unit),
        ]
        process = subprocess.run(command, capture_output=True, text=True)
        if process.returncode:
            raise RuntimeError(
                f"Clang field extraction failed for {profile['id']}:\n{process.stderr}"
            )
        return extract_fields(
            json.loads(process.stdout), profile, upstream_root, source_units, starts
        )


def render(upstream_root: Path, ownership: Path, profiles_path: Path) -> str:
    rows = ownership_rows(ownership)
    profiles = tomllib.loads(profiles_path.read_text())["profile"]
    fields = [
        field
        for profile in profiles
        for field in compile_profile(profile, upstream_root, rows)
    ]
    if not fields:
        raise ValueError("Clang discovered no backend fields")
    fields.sort()
    return "\n".join(("\t".join(HEADER), *(field.tsv() for field in fields))) + "\n"


def main() -> int:
    args = parse_args()
    ownership = repo_path(args.repo_root, args.ownership_inventory)
    profiles = repo_path(args.repo_root, args.profiles)
    output = repo_path(args.repo_root, args.output)
    rendered = render(args.upstream_root.resolve(), ownership, profiles)
    count = len(rendered.splitlines()) - 1
    if args.check:
        if not output.is_file() or output.read_text() != rendered:
            print("backend field inventory is stale", file=sys.stderr)
            return 1
        print(f"backend field inventory clean: {count} fields")
        return 0
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(rendered)
    print(f"wrote {count} backend field rows to {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
