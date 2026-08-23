#!/usr/bin/env python3
"""Freeze exact generated shader artifacts and the toolchain that produced them."""

from __future__ import annotations

import argparse
import csv
import hashlib
import os
import shlex
import shutil
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path


HEADER = (
    "stage",
    "artifact_path",
    "artifact_sha256",
    "retention",
    "direct_include_count",
)
VARIABLE_STAGES = {
    "MINIFY_EXPORT_OUTPUTS": "minify-export",
    "MINIFY_GLSL_OUTPUTS": "minify-source",
    "MINIFY_HPP_OUTPUTS": "minify-header",
    "SPIRV_OUTPUTS_BINARY": "spirv-binary",
    "SPIRV_OUTPUTS_HEADERS": "spirv-header",
    "WGSL_OUTPUTS": "wgsl-source",
    "WGSL_HEADER_OUTPUTS": "wgsl-header",
}


@dataclass(frozen=True, order=True)
class Artifact:
    stage: str
    artifact_path: str
    artifact_sha256: str
    retention: str
    direct_include_count: int

    def tsv(self) -> str:
        return "\t".join(str(getattr(self, column)) for column in HEADER)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--upstream-root", type=Path, required=True)
    parser.add_argument("--toolchain", type=Path, required=True)
    parser.add_argument("--dependencies", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    return parser.parse_args()


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_digest(root: Path) -> str:
    result = hashlib.sha256()
    files = sorted(
        path
        for path in root.rglob("*")
        if path.is_file() and "__pycache__" not in path.parts
    )
    for path in files:
        result.update(path.relative_to(root).as_posix().encode())
        result.update(b"\0")
        result.update(bytes.fromhex(digest(path)))
    return result.hexdigest()


def verify_toolchain(toolchain: dict, upstream_root: Path) -> None:
    ply = upstream_root / toolchain["ply_source"]
    actual_ply = tree_digest(ply)
    if actual_ply != toolchain["ply_tree_sha256"]:
        raise ValueError(f"PLY source tree drift: {actual_ply}")
    for tool in toolchain["tool"]:
        captured = Path(tool["captured_path"])
        if captured.is_file() and digest(captured) == tool["captured_sha256"]:
            executable = captured
        else:
            resolved = shutil.which(tool["command"])
            if resolved is None:
                raise FileNotFoundError(f"missing tool {tool['id']}: {tool['command']}")
            executable = Path(resolved)
        version = subprocess.run(
            [str(executable), *tool["version_args"]],
            check=True,
            capture_output=True,
            text=True,
        )
        combined = version.stdout + version.stderr
        if tool["version_contains"] not in combined:
            raise ValueError(f"tool version drift for {tool['id']}: {combined.strip()}")


def expected_outputs(shader_dir: Path, toolchain: dict) -> dict[str, str]:
    ply_parent = shader_dir.parents[1] / "dependencies" / "dabeaz_ply_3.11"
    make_flags = toolchain["make_flags_template"].format(ply_parent=ply_parent)
    process = subprocess.run(
        ["make", "-pn", f"FLAGS={make_flags}"],
        cwd=shader_dir,
        check=True,
        capture_output=True,
        text=True,
        env={**os.environ, "LC_ALL": "C"},
    )
    outputs: dict[str, str] = {}
    for line in process.stdout.splitlines():
        for variable, stage in VARIABLE_STAGES.items():
            prefixes = (f"{variable} =", f"{variable} :=", f"{variable} +=")
            prefix = next((value for value in prefixes if line.startswith(value)), None)
            if prefix is None:
                continue
            for value in shlex.split(line[len(prefix) :].strip()):
                previous = outputs.setdefault(value, stage)
                if previous != stage:
                    raise ValueError(f"generated output crosses stages: {value}")
    missing_variables = set(VARIABLE_STAGES.values()) - set(outputs.values())
    if missing_variables:
        raise ValueError(f"make database omitted generated stages: {sorted(missing_variables)}")
    return outputs


def direct_includes(path: Path) -> dict[str, int]:
    counts: dict[str, int] = {}
    with path.open(newline="") as handle:
        for row in csv.DictReader(handle, delimiter="\t"):
            if row["resolution_kind"] != "generated-from-owned-source":
                continue
            token = row["dependency_token"]
            if token.startswith("generated/shaders/"):
                artifact = "out/generated/" + token[len("generated/shaders/") :]
            else:
                artifact = "out/generated/" + token
            counts[artifact] = counts.get(artifact, 0) + 1
    return counts


def render(upstream_root: Path, toolchain: dict, dependencies: Path) -> str:
    shader_dir = upstream_root / toolchain["shader_directory"]
    outputs = expected_outputs(shader_dir, toolchain)
    include_counts = direct_includes(dependencies)
    missing_denominator = sorted(set(include_counts) - set(outputs))
    if missing_denominator:
        raise ValueError(
            "generated includes absent from Make denominator: " + ", ".join(missing_denominator)
        )
    artifacts: list[Artifact] = []
    for relative, stage in outputs.items():
        artifact = shader_dir / relative
        if artifact.is_file():
            retention = "retained"
            sha = digest(artifact)
        elif stage == "wgsl-source" and artifact.with_suffix(".hpp").is_file():
            retention = "ephemeral-final-header-retained"
            sha = "-"
        else:
            raise FileNotFoundError(f"missing generated artifact: {relative}")
        artifacts.append(
            Artifact(stage, relative, sha, retention, include_counts.get(relative, 0))
        )
    artifacts.sort()
    return "\n".join(("\t".join(HEADER), *(artifact.tsv() for artifact in artifacts))) + "\n"


def repo_path(repo_root: Path, path: Path) -> Path:
    return path if path.is_absolute() else repo_root / path


def main() -> int:
    args = parse_args()
    toolchain_path = repo_path(args.repo_root, args.toolchain)
    dependencies = repo_path(args.repo_root, args.dependencies)
    output = repo_path(args.repo_root, args.output)
    toolchain = tomllib.loads(toolchain_path.read_text())
    verify_toolchain(toolchain, args.upstream_root)
    rendered = render(args.upstream_root, toolchain, dependencies)
    row_count = len(rendered.splitlines()) - 1
    if args.check:
        if not output.is_file() or output.read_text() != rendered:
            print("backend generated artifact inventory is stale", file=sys.stderr)
            return 1
        print(f"backend generated artifact inventory clean: {row_count} outputs")
        return 0
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(rendered)
    print(f"wrote {row_count} generated artifact rows to {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
