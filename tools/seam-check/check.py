#!/usr/bin/env python3
"""Ratchet parity/replay packages against product and authoring imports.

Nuxie-only architecture guard; no pinned C++ behavior or correspondence row.
See docs/seam-contract.md for the contract and the two grandfathered internal
debt families that this stage-one checker prevents from spreading.
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys
import tomllib
from collections.abc import Iterator


PROTECTED_PACKAGES = (
    "crates/nuxie-audio",
    "crates/nuxie-binary",
    "crates/nuxie-graph",
    "crates/nuxie-image-codec",
    "crates/nuxie-render-api",
    "crates/nuxie-render-stream",
    "crates/nuxie-renderer",
    "crates/nuxie-renderer-ffi",
    "crates/nuxie-runtime",
    "crates/nuxie-schema",
    "crates/nuxie-scripting",
    "tools/golden-compare",
    "tools/perf-compare",
    "tools/pixel-compare",
    "tools/renderer-fuzz-replay",
    "tools/renderer-replay",
    "tools/rust-golden-runner",
    "tools/silver-corpus",
)

FORBIDDEN_DEPENDENCIES = {
    "nuxie",
    "nux-container",
    "nuxie-authoring",
    "nuxie-flow",
    "nuxie-product",
    "nuxie-product-scripting",
    "nuxie-project-data",
}
FORBIDDEN_DEPENDENCY_PREFIXES = (
    "nuxie-authoring-",
    "nuxie-flow-",
    "nuxie-product-",
    "nuxie-project-",
)

# These are file-level ratchet exceptions, not compliant dependencies. A new
# file containing either marker family fails. Deleting entries is allowed and
# should happen as the migration in docs/seam-contract.md proceeds.
INTERNAL_DEBT_FILES = {
    "project-data": {
        "crates/nuxie-runtime/src/project_data_converter.rs",
        "crates/nuxie-runtime/src/lib.rs",
        "crates/nuxie-runtime/src/data_bind/context/context_value.rs",
        "crates/nuxie-runtime/src/data_bind/converters/data_converter_number_to_list.rs",
        "crates/nuxie-runtime/src/data_bind/data_bind_context.rs",
    },
    "product-host-commands": {
        "crates/nuxie-scripting/src/vm.rs",
        "crates/nuxie-scripting/src/vm/host_commands.rs",
        "crates/nuxie-scripting/src/vm/resource_limits.rs",
    },
}
INTERNAL_DEBT_MARKERS = {
    "project-data": re.compile(r"\bProjectData|\bproject_data_converter\b"),
    "product-host-commands": re.compile(
        r"\bhost_commands\b|\bHost(?:Command|Value|CycleCheckpoint|EffectCheckpoint)\b"
    ),
}

EXPLICIT_PRODUCT_PATH = re.compile(
    r"\b(?:nuxie::(?:flow_session|scene)|"
    r"nuxie_(?:authoring|flow|product(?:_scripting)?|project_data)|nux_container)::"
)
LOCAL_PRODUCT_MODULE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:mod|use)\s+"
    r"(?:(?:crate|self|super)::)*(?:authoring|flow_session)(?:::|\s*;)"
)


def normalized_package_name(name: str) -> str:
    return name.strip().replace("_", "-")


def is_forbidden_dependency(name: str) -> bool:
    normalized = normalized_package_name(name)
    return normalized in FORBIDDEN_DEPENDENCIES or normalized.startswith(
        FORBIDDEN_DEPENDENCY_PREFIXES
    )


def dependency_tables(
    value: object, path: tuple[str, ...] = ()
) -> Iterator[tuple[tuple[str, ...], dict[str, object]]]:
    if not isinstance(value, dict):
        return
    for key, child in value.items():
        child_path = (*path, str(key))
        if str(key) in {"dependencies", "dev-dependencies", "build-dependencies"}:
            if isinstance(child, dict):
                yield child_path, child
            continue
        yield from dependency_tables(child, child_path)


def dependency_package(dependency_name: str, specification: object) -> str:
    if isinstance(specification, dict):
        package = specification.get("package")
        if isinstance(package, str):
            return package
    return dependency_name


def code_before_line_comment(line: str) -> str:
    # Import/module statements do not need literal contents. Removing ordinary
    # quoted strings as well as line comments avoids treating resource-code
    # assertions as imports without pretending to be a complete Rust parser.
    code = line.split("//", 1)[0]
    return re.sub(r'"(?:\\.|[^"\\])*"', '""', code)


def check_repository(repo_root: pathlib.Path) -> tuple[list[str], dict[str, set[str]]]:
    errors: list[str] = []
    observed_debt = {family: set() for family in INTERNAL_DEBT_FILES}

    for package in PROTECTED_PACKAGES:
        package_root = repo_root / package
        manifest_path = package_root / "Cargo.toml"
        if not manifest_path.exists():
            continue
        try:
            manifest = tomllib.loads(manifest_path.read_text())
        except (OSError, tomllib.TOMLDecodeError) as error:
            errors.append(f"{package}/Cargo.toml: cannot parse manifest: {error}")
            continue

        for table_path, dependencies in dependency_tables(manifest):
            for dependency_name, specification in dependencies.items():
                resolved_name = dependency_package(dependency_name, specification)
                if is_forbidden_dependency(dependency_name) or is_forbidden_dependency(
                    resolved_name
                ):
                    table = ".".join(table_path)
                    errors.append(
                        f"{package}/Cargo.toml: protected package imports product "
                        f"dependency {dependency_name!r} (package {resolved_name!r}) "
                        f"through [{table}]"
                    )

        source_root = package_root / "src"
        if not source_root.exists():
            continue
        for source_path in sorted(source_root.rglob("*.rs")):
            relative = source_path.relative_to(repo_root).as_posix()
            try:
                lines = source_path.read_text().splitlines()
            except OSError as error:
                errors.append(f"{relative}: cannot read source: {error}")
                continue
            for line_number, line in enumerate(lines, 1):
                code = code_before_line_comment(line)
                if EXPLICIT_PRODUCT_PATH.search(code) or LOCAL_PRODUCT_MODULE.search(code):
                    errors.append(
                        f"{relative}:{line_number}: protected source imports a "
                        f"product/authoring module: {code.strip()}"
                    )
                for family, marker in INTERNAL_DEBT_MARKERS.items():
                    if not marker.search(code):
                        continue
                    observed_debt[family].add(relative)
                    if relative not in INTERNAL_DEBT_FILES[family]:
                        errors.append(
                            f"{relative}:{line_number}: {family} seam debt spread "
                            "outside its grandfathered files"
                        )

    return errors, observed_debt


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--repo-root",
        type=pathlib.Path,
        default=pathlib.Path(__file__).resolve().parents[2],
    )
    args = parser.parse_args()
    repo_root = args.repo_root.resolve()

    errors, observed_debt = check_repository(repo_root)
    if errors:
        for error in errors:
            print(f"seam check failed: {error}", file=sys.stderr)
        return 1

    debt_summary = ", ".join(
        f"{family}={len(paths)} grandfathered file(s)"
        for family, paths in sorted(observed_debt.items())
    )
    print(f"seam check passed; internal migration debt: {debt_summary}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
