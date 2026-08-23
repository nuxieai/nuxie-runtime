#!/usr/bin/env python3
"""Fail closed when the native-Metal test inventory changes or selects nothing."""

from __future__ import annotations

import argparse
import hashlib
import pathlib
import re
import subprocess
import sys
import tomllib
from typing import Any


TEST_LINE = re.compile(r"^(.*): test$")
IGNORE = re.compile(
    r'#\[ignore(?:\s*=\s*"([^"]*)")?\]\s*'
    r'(?:#\[[^\]]+\]\s*)*fn\s+([A-Za-z0-9_]+)',
    re.MULTILINE,
)
LIVE_METAL_GUARD = re.compile(r'crate::live_metal_test_unavailable\("([^"]+)"\)')


class CensusError(RuntimeError):
    pass


def names_from_list(output: str) -> list[str]:
    return sorted({match.group(1) for line in output.splitlines() if (match := TEST_LINE.match(line))})


def names_sha256(names: list[str]) -> str:
    payload = "".join(f"{name}\n" for name in sorted(names)).encode()
    return hashlib.sha256(payload).hexdigest()


def source_ignores(repo_root: pathlib.Path, paths: list[str]) -> list[tuple[str, str, str]]:
    rows: list[tuple[str, str, str]] = []
    for relative in sorted(set(paths)):
        text = (repo_root / relative).read_text(encoding="utf-8")
        rows.extend((relative, match.group(2), match.group(1) or "") for match in IGNORE.finditer(text))
    return sorted(rows)


def source_ignore_sha256(rows: list[tuple[str, str, str]]) -> str:
    payload = "".join("\0".join(row) + "\n" for row in rows).encode()
    return hashlib.sha256(payload).hexdigest()


def live_metal_guards(repo_root: pathlib.Path, paths: list[str]) -> list[tuple[str, int, str]]:
    rows: list[tuple[str, int, str]] = []
    for relative in sorted(set(paths)):
        text = (repo_root / relative).read_text(encoding="utf-8")
        rows.extend(
            (relative, index, match.group(1))
            for index, match in enumerate(LIVE_METAL_GUARD.finditer(text), start=1)
        )
    return rows


def live_metal_guard_sha256(rows: list[tuple[str, int, str]]) -> str:
    payload = "".join(f"{path}\0{index}\0{context}\n" for path, index, context in rows).encode()
    return hashlib.sha256(payload).hexdigest()


def validate_canonical_ore_census(
    guard_paths: list[str], required_tests: set[str]
) -> None:
    inert_paths = sorted(
        path
        for path in guard_paths
        if path.startswith("crates/nuxie-ore-metal/src/")
        and not path.startswith("crates/nuxie-ore-metal/src/mechanical_port/")
    )
    if inert_paths:
        raise CensusError(
            f"ORE live-Metal guard paths must name canonical mechanical owners: {inert_paths}"
        )

    inert_names = sorted(
        name
        for name in required_tests
        if name.startswith(("ore-default:", "ore-tools:"))
        and ":mechanical_port::source::" not in name
    )
    if inert_names:
        raise CensusError(
            "ORE required-live tests must name canonical mechanical modules: "
            f"{inert_names}"
        )


def run_list(repo_root: pathlib.Path, cargo_args: list[str], ignored: bool = False) -> list[str]:
    command = ["cargo", "test", "--locked", *cargo_args, "--", "--list"]
    if ignored:
        command.append("--ignored")
    result = subprocess.run(command, cwd=repo_root, text=True, capture_output=True, check=False)
    if result.returncode:
        raise CensusError(
            f"test-list command failed ({' '.join(command)}):\n{result.stdout}{result.stderr}"
        )
    return names_from_list(result.stdout)


def check_harness(repo_root: pathlib.Path, harness: dict[str, Any]) -> list[str]:
    harness_id = str(harness["id"])
    names = run_list(repo_root, list(harness["cargo_args"]))
    ignored = run_list(repo_root, list(harness["cargo_args"]), ignored=True)
    expected_total = int(harness["expected_total"])
    expected_ignored = sorted(str(name) for name in harness.get("expected_ignored", []))
    expected_active = int(harness["expected_active"])

    if not names:
        raise CensusError(f"{harness_id}: command selected zero tests")
    if len(names) != expected_total:
        raise CensusError(f"{harness_id}: expected {expected_total} tests, selected {len(names)}")
    if len(names) - len(ignored) != expected_active:
        raise CensusError(
            f"{harness_id}: expected {expected_active} active tests, selected {len(names) - len(ignored)}"
        )
    if ignored != expected_ignored:
        raise CensusError(
            f"{harness_id}: ignored-name set changed\nexpected={expected_ignored}\nactual={ignored}"
        )
    digest = names_sha256(names)
    if digest != harness["names_sha256"]:
        raise CensusError(
            f"{harness_id}: name-set hash changed; expected {harness['names_sha256']}, got {digest}"
        )
    active_digest = names_sha256(sorted(set(names) - set(ignored)))
    if active_digest != harness["active_names_sha256"]:
        raise CensusError(
            f"{harness_id}: active-name hash changed; expected {harness['active_names_sha256']}, got {active_digest}"
        )
    print(f"{harness_id}: {len(names)} total, {len(names) - len(ignored)} active, {len(ignored)} ignored")
    return names


def check_all_harnesses(
    repo_root: pathlib.Path, harnesses: list[dict[str, Any]]
) -> tuple[set[str], list[str]]:
    """Execute every census lane even when an earlier lane is already red."""

    selected: set[str] = set()
    errors: list[str] = []
    for harness in harnesses:
        try:
            names = check_harness(repo_root, harness)
        except CensusError as error:
            errors.append(str(error))
            continue
        selected.update(f"{harness['id']}:{name}" for name in names)
    return selected, errors


def check_manifest(repo_root: pathlib.Path, manifest_path: pathlib.Path, execute: bool) -> None:
    with manifest_path.open("rb") as source:
        manifest = tomllib.load(source)
    if manifest.get("version") != 1:
        raise CensusError("test census version must be 1")
    harnesses = manifest.get("harness", [])
    if not harnesses:
        raise CensusError("test census has no harnesses")
    ids = [harness.get("id") for harness in harnesses]
    if len(ids) != len(set(ids)):
        raise CensusError("test census harness ids must be unique")
    if any(int(harness.get("expected_total", 0)) <= 0 for harness in harnesses):
        raise CensusError("every census harness must select a nonzero expected test count")

    ignore_paths = [str(path) for path in manifest.get("source_ignore_paths", [])]
    rows = source_ignores(repo_root, ignore_paths)
    if len(rows) != int(manifest["expected_source_ignore_count"]):
        raise CensusError(
            f"expected {manifest['expected_source_ignore_count']} source ignores, found {len(rows)}"
        )
    digest = source_ignore_sha256(rows)
    if digest != manifest["source_ignore_sha256"]:
        raise CensusError(
            f"source ignore names/reasons changed; expected {manifest['source_ignore_sha256']}, got {digest}"
        )

    diagnostics = {
        (str(row["path"]), str(row["name"]), str(row["reason"]))
        for row in manifest.get("diagnostic_ignore", [])
    }
    if not diagnostics.issubset(set(rows)):
        raise CensusError(f"declared external diagnostic ignores are absent: {sorted(diagnostics - set(rows))}")

    guards = live_metal_guards(
        repo_root, [str(path) for path in manifest.get("live_metal_guard_paths", [])]
    )
    if len(guards) != int(manifest["expected_live_metal_guard_count"]):
        raise CensusError(
            f"expected {manifest['expected_live_metal_guard_count']} live-Metal guards, found {len(guards)}"
        )
    guard_digest = live_metal_guard_sha256(guards)
    if guard_digest != manifest["live_metal_guard_sha256"]:
        raise CensusError(
            f"live-Metal guard set changed; expected {manifest['live_metal_guard_sha256']}, got {guard_digest}"
        )

    required_tests = {
        str(name) for name in manifest.get("baseline", {}).get("required_live_metal_test", [])
    }
    if len(required_tests) != int(manifest["expected_live_metal_test_count"]):
        raise CensusError("required-live-Metal test baseline count does not match its exact name set")
    validate_canonical_ore_census(
        [str(path) for path in manifest.get("live_metal_guard_paths", [])],
        required_tests,
    )

    if execute:
        selected, harness_errors = check_all_harnesses(repo_root, harnesses)
        absent = required_tests - selected
        # A failed harness has no trustworthy selected-name set. Report its
        # direct count/compile error first; only grade required-live membership
        # once every lane produced a valid census.
        if absent and not harness_errors:
            harness_errors.append(
                f"required live-Metal tests are not selected: {sorted(absent)}"
            )
        if harness_errors:
            raise CensusError(
                "test harness census mismatches:\n- " + "\n- ".join(harness_errors)
            )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument("--manifest", type=pathlib.Path, required=True)
    parser.add_argument("--execute", action="store_true", help="run every cargo --list census command")
    args = parser.parse_args()
    try:
        check_manifest(args.repo_root.resolve(), args.manifest.resolve(), args.execute)
    except (CensusError, OSError, tomllib.TOMLDecodeError) as error:
        print(f"metal test census failed: {error}", file=sys.stderr)
        return 1
    print("native Metal test census: CLEAN")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
