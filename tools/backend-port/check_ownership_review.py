#!/usr/bin/env python3
"""Validate the global backend ownership, lifetime, and ABI review evidence."""

from __future__ import annotations

import argparse
import csv
import hashlib
import importlib.util
import re
import subprocess
import sys
import tempfile
import tomllib
from dataclasses import dataclass
from pathlib import Path
from types import ModuleType
from typing import Any, Iterable


sys.dont_write_bytecode = True


EXPECTED_UPSTREAM_REF = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"
EXPECTED_WORKSPACE_BASE_REF = "4af6b0ac961191bfd9b755223e7a52e2865ee004"
EXPECTED_PLAN_PATH = "docs/backend-port-ownership-review-plan.toml"
EXPECTED_SCHEMA_PATH = "docs/backend-port-ownership-review-schema.md"
EXPECTED_PLAN_SHA256 = "d1a60cdb9db5cd1ce43a2cbcfb27436d1d9fcd547eb944a3748047222e56cb61"
EXPECTED_PLAN_LOGICAL_LINES = 1635
EXPECTED_PLAN_BYTE_COUNT = 66067
EXPECTED_SCHEMA_SHA256 = "e6c56c055ce756f2fe05d9f7a4cafc41cee742c077a2e16e8cb788099a93f9a9"
EXPECTED_SCHEMA_LOGICAL_LINES = 722
EXPECTED_SCHEMA_BYTE_COUNT = 33739
EXPECTED_RECEIPT_DIRECTORY = "docs/backend-port-ownership-reviews"
EXPECTED_SOURCE_RECEIPT_DIRECTORY = "docs/backend-port-source-reviews"
EXPECTED_MANIFEST_PATHS = {
    "ownership_review_plan": EXPECTED_PLAN_PATH,
    "ownership_review_schema": EXPECTED_SCHEMA_PATH,
    "ownership_review_receipt_directory": EXPECTED_RECEIPT_DIRECTORY,
}
EXPECTED_LAUNCH_FROZEN_PATHS = (
    EXPECTED_PLAN_PATH,
    EXPECTED_SCHEMA_PATH,
    "tools/backend-port/check_ownership_review.py",
    "tools/backend-port/check_source_review.py",
)
EXPECTED_MANIFEST_RELATIVE = "docs/backend-port-campaign.toml"
EXPECTED_MAKEFILE_RELATIVE = "Makefile"
EXPECTED_MAKE_TOOL_ASSIGNMENT = (
    "BACKEND_PORT_OWNERSHIP_REVIEW_TOOL ?= "
    "$(CURDIR)/tools/backend-port/check_ownership_review.py"
)
EXPECTED_MAKE_RECIPES = {
    "backend-port-ownership-review-admission": (
        "PYTHONDONTWRITEBYTECODE=1 python3 "
        '"$(BACKEND_PORT_OWNERSHIP_REVIEW_TOOL)" --repo-root "$(CURDIR)" '
        '--upstream-root "$(RIVE_RUNTIME_DIR)" --manifest '
        '"$(BACKEND_PORT_CAMPAIGN)" --admission'
    ),
    "backend-port-ownership-review-check": (
        "PYTHONDONTWRITEBYTECODE=1 python3 "
        '"$(BACKEND_PORT_OWNERSHIP_REVIEW_TOOL)" --repo-root "$(CURDIR)" '
        '--upstream-root "$(RIVE_RUNTIME_DIR)" --manifest '
        '"$(BACKEND_PORT_CAMPAIGN)"'
    ),
}
COMPLETION_PIN_KEYS = {
    "ownership_review_barrier_ref",
    "ownership_review_receipt_tree_sha256",
    "ownership_review_receipt_logical_lines",
    "ownership_review_receipt_bytes",
    "ownership_review_finding_total",
    "ownership_review_p0_findings",
    "ownership_review_p1_findings",
    "ownership_review_p2_findings",
    "ownership_review_p3_findings",
    "ownership_review_finding_id_sha256",
}
EXPECTED_COMPONENT_COUNT = 115
EXPECTED_UNIT_COUNT = 135
EXPECTED_SOURCE_COUNT = 200
EXPECTED_TARGET_COUNT = 188
EXPECTED_SUPPORT_COUNT = 52
EXPECTED_OVERLAY_COUNT = 9
EXPECTED_LEDGER_COUNTS = {
    "field": 1946,
    "lifecycle": 2431,
    "configuration": 5409,
    "dependency": 924,
}
EXPECTED_LEDGER_DIGESTS = {
    "field": "a6bb31c8bbdd609cb04883282ad19efbbb3b5abbc2e0037c7713f5a3821d6b89",
    "lifecycle": "d7dff40064460ceb27a89eed93d75e6218f8fc21f100cdcbe8ce426cec0de67d",
    "configuration": "8a25deb0e0f46f8579b254cf5b54f7b51ca2c1775d4528c2e133e3dbbb512315",
    "dependency": "200635550831eca73a1f88aff3ec19679e8ec096c99ca1592809be2883841f6c",
}
EXPECTED_SOURCE_RECEIPT_TREE_SHA256 = \
    "5ab7b2271288fd1ee5e3de066b2f0c87c1983c17df0a0376e078605a17f30d5f"
EXPECTED_SOURCE_RECEIPT_BYTES = 1179667
EXPECTED_SOURCE_RECEIPT_LINES = 10228
EXPECTED_SOURCE_BINDING_SHA256 = \
    "2eb802438b2fad3e5cd8612319deb22e5e0f9f444649d86f4cb66aa672f1fc91"
EXPECTED_TARGET_BINDING_SHA256 = \
    "8351cbeed2d03ddc7fecce20939983272ab554769a73ecf85352ef6a470410ef"
EXPECTED_SUPPORT_BINDING_SHA256 = \
    "ac34add6fef74cbba4444fdc342300a86aa70648bac4aa46a3db6d301b5f625e"
EXPECTED_COMPONENT_AUTHORITY_COUNT = 11937
EXPECTED_COMPONENT_AUTHORITY_SHA256 = \
    "78573b0c03e151f95ff85981a98bb56219a0ea678c48107647a0d8e67f83fa35"
EXPECTED_SUPPORT_AUTHORITY_SHA256 = \
    "a7772b03439e94efa03c2a9e523301264d8f9d592b447196e282185a0c01f050"
EXPECTED_UNIT_SEAM_COUNT = 452
EXPECTED_UNIT_SEAM_SHA256 = \
    "287f25e6be86c833591fba6fd5459af27d428db7c203f8390030f3d18dac5da1"
EXPECTED_COMPONENT_DEPENDENCY_COUNT = 413
EXPECTED_SOURCE_OVERLAY_COVERED_CROSS_ROWS = 457
EXPECTED_SOURCE_OVERLAY_OMITTED_CROSS_ROWS = 88
EXPECTED_SOURCE_OVERLAY_OMITTED_CROSS_ROWS_SHA256 = \
    "1f83c99b7490a0570cb54c8050205aa080f373276dfd92cf150f108c54d8bf59"
EXPECTED_OWNERSHIP_OVERLAY_COVERED_CROSS_ROWS = 545
EXPECTED_OWNERSHIP_OVERLAY_OMITTED_CROSS_ROWS = 0
EXPECTED_OVERLAY_COVERED_COMPONENT_PAIRS = 412
EXPECTED_OVERLAY_OMITTED_COMPONENT_PAIR = "component-084->component-083"
EXPECTED_SCC_PARTITION_SHA256 = \
    "6dba6dbdf824e9080d850abe19c87ee42a77fda64c692202de767b4da98df3ea"
EXPECTED_OWNER_FAMILY_MEMBERSHIP_SHA256 = \
    "8d32180cf28d3d074471f1154674cad905d8e9d640437abb9880cc897c2ceefd"
EXPECTED_PROFILE_MEMBERSHIP_SHA256 = \
    "f2d966d1ba14937adcf5165de45fbb8ef36b86a219f60cfa747b11686515107c"
EXPECTED_OVERLAY_AUTHORITIES = (
    (5901, "184c6ca1862d6fbe2db34ec3e566f8100427d5114934ee978d081f8b5f282820"),
    (1008, "cd5a390847a2c1a52bc47e6c4e3648f4f14463e8a9d68102980e5b98a75341c9"),
    (4724, "55a77146963bda10711351a134c134201d467260f793d6c78069520dfa56a17c"),
    (5375, "799a653ed8a7c8c2c1224e50432c0b27c31d8be524712ae1518286b97f667c61"),
    (4304, "0cd6739ab24db8df95a3b78791c4d8f3b9d2a757d8dbe66194efec5f83ba3c9f"),
    (4794, "779e3e076f968c05c3b62533eef4b24d33fb48bc1e9a8a74777de7e370c171a9"),
    (1997, "02c42fb360643da1f750f958c9d46504a2e64cd5ca8742bff467e7e77b7f06ee"),
    (5966, "9b868e165c375e15fb2c451b6e5a52a7501b0417e104db27f45a7fb6f4c742cb"),
    (1301, "7585df6d303bc46453ebb9a72c07e7d1287a8cc2d125e735b033739ba0b7bb38"),
)
EXPECTED_CONTRACT_HASHES = {
    "owner_contract": "b9d6aef8689ef92ac7f50de25c803c4fdf4928e9ac3da632b3536f952b4117a6",
    "field_profile": "7fcc6aa87d7ef650de4875b749faadb9bd52b7d0995e13bbef21aa92f5852e79",
}
EXPECTED_COVERAGE = [
    "field-and-layout",
    "ownership-transfers",
    "provenance-and-aliasing",
    "callbacks-and-threading",
    "synchronization-and-mapping",
    "failure-and-loss",
    "teardown-and-destruction-order",
    "unsafe-ffi-and-abi",
    "configuration-owner-graphs",
]
EXPECTED_OVERLAY_COVERAGE = [*EXPECTED_COVERAGE, "cross-owner-overlays"]
EXPECTED_SEVERITIES = ["P0", "P1", "P2", "P3"]
LEDGER_SPECS = {
    "field": ("field-raw", "field_inventory", "ownership_unit"),
    "lifecycle": ("lifecycle-raw", "lifecycle_inventory", "ownership_unit"),
    "configuration": (
        "configuration-raw",
        "configuration_inventory",
        "ownership_unit",
    ),
    "dependency": ("dependency-raw", "dependency_inventory", "source_unit"),
}

PLAN_KEYS = {
    "schema_version",
    "upstream_ref",
    "workspace_base_ref",
    "review_kind",
    "review_mode",
    "receipt_directory",
    "source_review_receipt_directory",
    "coverage",
    "severity_order",
    "finding_id_rules",
    "structural_sequence",
    "overlay_order",
    "denominator",
    "rules",
    "canonicalization",
    "authority",
    "byte_authority",
    "ledger_authority",
    "contract_authority",
    "launch_contract",
    "completion_contract",
    "wave",
    "overlay",
    "prerequisite_receipt",
}
DENOMINATOR_KEYS = {
    "ownership_units", "components", "component_receipts", "support_receipts",
    "overlay_receipts", "total_receipts", "source_review_prerequisite_receipts",
    "source_review_open_findings", "source_review_p0_findings",
    "source_review_p1_findings", "source_review_p2_findings",
    "source_review_p3_findings", "sources", "targets", "support_artifacts",
    "fields", "lifecycle_events", "configurations", "dependency_edges",
    "owner_contract_families", "field_profiles", "owner_family_memberships",
    "component_profile_memberships", "declared_dependency_unit_edges",
    "intra_scc_dependency_unit_edges", "cross_component_dependency_unit_edges",
    "known_unit_dependency_pairs", "self_unit_dependency_pairs",
    "nonself_unit_dependency_pairs", "unique_cross_component_pairs",
    "cross_component_raw_dependency_rows",
    "source_overlay_covered_cross_component_raw_dependency_rows",
    "source_overlay_omitted_cross_component_raw_dependency_rows",
    "ownership_overlay_covered_cross_component_raw_dependency_rows",
    "ownership_overlay_omitted_cross_component_raw_dependency_rows",
    "overlay_covered_cross_component_pairs", "overlay_omitted_cross_component_pairs",
    "source_logical_lines", "source_bytes",
    "target_logical_lines", "target_bytes", "support_logical_lines",
    "support_bytes", "source_review_receipt_logical_lines",
    "source_review_receipt_bytes",
}
WAVE_PLAN_KEYS = {
    "id", "order_group", "component_count", "unit_count", "source_count",
    "target_count", "field_rows", "lifecycle_rows", "configuration_rows",
    "dependency_rows", "owner_family_memberships", "field_profile_memberships",
    "cross_dependency_unit_seams", "dependency_component_pairs",
    "component_order_sha256", "component_ids",
}
OVERLAY_PLAN_KEYS = {
    "ordinal", "id", "rule", "component_count", "support_count", "field_rows",
    "lifecycle_rows", "configuration_rows", "dependency_rows",
    "owner_family_memberships", "field_profile_memberships",
    "source_review_authority_record_count", "source_review_authority_sha256",
    "ownership_authority_record_count", "ownership_authority_sha256",
    *{
        f"{category}_{suffix}"
        for category in (
            "source_binding", "target_binding", "support_binding",
            "artifact_binding", "external_binding", "generated_binding",
        )
        for suffix in ("count", "sha256")
    },
    *{
        f"{category}_{suffix}"
        for category in ("source", "target", "support", "artifact", "external", "generated")
        for suffix in ("logical_lines", "byte_count")
    },
    "tree_binding_count", "tree_member_count", "tree_logical_lines",
    "tree_byte_count", "tree_binding_sha256",
}
COMPONENT_RECEIPT_KEYS = {
    "schema_version",
    "receipt_kind",
    "component_id",
    "units",
    "owner_families",
    "field_profiles",
    "upstream_ref",
    "workspace_base_ref",
    "role",
    "review_run_id",
    "review_wave",
    "coverage",
    "owner_contract_bindings",
    "field_profile_bindings",
    "dependency_components",
    "dependency_ownership_receipts",
    "field_authority",
    "lifecycle_authority",
    "configuration_authority",
    "dependency_authority",
    "authority_record_count",
    "authority_sha256",
    "authority_keys",
    "source_review_receipts",
    "sources",
    "targets",
    "findings",
    "open_findings",
    "attestation",
}
SUPPORT_RECEIPT_KEYS = {
    "schema_version",
    "receipt_kind",
    "upstream_ref",
    "workspace_base_ref",
    "role",
    "review_run_id",
    "review_wave",
    "coverage",
    "authority_record_count",
    "authority_sha256",
    "authority_keys",
    "source_review_receipts",
    "artifacts",
    "findings",
    "open_findings",
    "attestation",
}
OVERLAY_RECEIPT_KEYS = {
    "schema_version",
    "receipt_kind",
    "upstream_ref",
    "workspace_base_ref",
    "role",
    "review_run_id",
    "review_wave",
    "coverage",
    "source_review_receipts",
    "overlays",
    "findings",
    "open_findings",
}
OVERLAY_RECORD_KEYS = {
    "id",
    "ordinal",
    "authority_record_count",
    "authority_sha256",
    "component_ids",
    "support_paths",
    "source_bindings",
    "target_bindings",
    "support_bindings",
    "artifact_bindings",
    "tree_bindings",
    "external_bindings",
    "generated_bindings",
    "authority_keys",
    "component_receipts",
    "support_receipts",
    "attestation",
}
SOURCE_RECORD_KEYS = {
    "path",
    "sha256",
    "logical_lines",
    "byte_count",
    "citation",
    "disposition",
}
TARGET_RECORD_KEYS = {"path", "sha256", "logical_lines", "byte_count", "citation"}
SUPPORT_RECORD_KEYS = {
    "path",
    "sha256",
    "logical_lines",
    "byte_count",
    "citation",
    "artifact_role",
    "review_overlay",
    "source_authority",
    "disposition",
}
BINDING_KEYS = {"id", "path", "sha256", "byte_count"}
FILE_BINDING_KEYS = {"path", "sha256", "logical_lines", "byte_count"}
TREE_BINDING_KEYS = {
    "path",
    "tree_sha256",
    "file_count",
    "logical_lines",
    "byte_count",
}
AUTHORITY_RECORD_KEYS = {"kind", "record_count", "sha256", "authority_keys"}
FINDING_KEYS = {
    "id",
    "severity",
    "summary",
    "review_domains",
    "citations",
    "authority_keys",
}
OVERLAY_FINDING_KEYS = {*FINDING_KEYS, "overlay_id"}


class OwnershipReviewError(ValueError):
    """An ownership-review evidence or authority contract failed."""


@dataclass(frozen=True)
class Binding:
    id: str
    path: str
    sha256: str
    byte_count: int

    def record(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "path": self.path,
            "sha256": self.sha256,
            "byte_count": self.byte_count,
        }


@dataclass(frozen=True)
class LedgerAuthority:
    kind: str
    keys: tuple[str, ...]
    sha256: str

    @property
    def count(self) -> int:
        return len(self.keys)

    def record(self) -> dict[str, Any]:
        return {
            "kind": self.kind,
            "record_count": self.count,
            "sha256": self.sha256,
            "authority_keys": list(self.keys),
        }


@dataclass(frozen=True)
class ComponentExpectation:
    component_id: str
    owner_families: tuple[str, ...]
    field_profiles: tuple[str, ...]
    owner_contract_bindings: tuple[Binding, ...]
    field_profile_bindings: tuple[Binding, ...]
    dependency_components: tuple[str, ...]
    ledgers: dict[str, LedgerAuthority]
    source_review_binding: Binding
    source_records: tuple[dict[str, Any], ...]
    target_records: tuple[dict[str, Any], ...]
    authority_keys: tuple[str, ...]
    authority_sha256: str


@dataclass(frozen=True)
class ComponentResult:
    component_id: str
    review_wave: str
    finding_ids: tuple[str, ...]
    open_findings: int


@dataclass(frozen=True)
class SupportResult:
    finding_ids: tuple[str, ...]
    open_findings: int


@dataclass(frozen=True)
class OverlayResult:
    finding_ids: tuple[str, ...]
    open_findings: int


@dataclass(frozen=True)
class OwnershipOverlayExpectation:
    id: str
    ordinal: int
    component_ids: tuple[str, ...]
    support_paths: tuple[str, ...]
    source_bindings: tuple[dict[str, Any], ...]
    target_bindings: tuple[dict[str, Any], ...]
    support_bindings: tuple[dict[str, Any], ...]
    artifact_bindings: tuple[dict[str, Any], ...]
    tree_bindings: tuple[dict[str, Any], ...]
    external_bindings: tuple[dict[str, Any], ...]
    generated_bindings: tuple[dict[str, Any], ...]
    authority_keys: tuple[str, ...]
    authority_sha256: str


@dataclass
class OwnershipAuthority:
    source: ModuleType
    source_authority: Any
    repo: Path
    upstream: Path
    plan: dict[str, Any]
    receipt_directory: Path
    families: tuple[dict[str, Any], ...]
    profiles: tuple[dict[str, Any], ...]
    ledgers: dict[str, tuple[dict[str, str], ...]]
    ledger_keys_by_component: dict[str, dict[str, tuple[str, ...]]]
    components: dict[str, ComponentExpectation]
    support_binding: Binding
    support_records: tuple[dict[str, Any], ...]
    support_authority_keys: tuple[str, ...]
    support_authority_sha256: str
    source_overlay_binding: Binding
    overlays: dict[str, OwnershipOverlayExpectation]
    campaign_manifest: dict[str, Any] | None = None
    live_repo: Path | None = None
    replay_checkout: Any = None


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--upstream-root", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--receipt", type=Path)
    mode.add_argument("--admission", action="store_true")
    return parser.parse_args()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise OwnershipReviewError(message)


def require_exact_keys(value: dict[str, Any], expected: set[str], label: str) -> None:
    missing = sorted(expected - set(value))
    extra = sorted(set(value) - expected)
    require(not missing, f"{label} is missing keys: {', '.join(missing)}")
    require(not extra, f"{label} invents keys: {', '.join(extra)}")


def load_source_checker(repo: Path) -> ModuleType:
    path = repo / "tools/backend-port/check_source_review.py"
    require(path.is_file(), f"missing source-review checker: {path}")
    name = "backend_port_check_source_review_for_ownership"
    spec = importlib.util.spec_from_file_location(name, path)
    require(spec is not None and spec.loader is not None,
            "cannot load source-review checker")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def canonical_key(prefix: str, row: dict[str, str]) -> str:
    fields = "\x1f".join(f"{key}={row[key]}" for key in sorted(row))
    return f"{prefix}:{fields}"


def canonical_digest(keys: Iterable[str]) -> tuple[tuple[str, ...], str]:
    ordered = tuple(sorted(set(keys)))
    digest = hashlib.sha256("\n".join(ordered).encode("utf-8")).hexdigest()
    return ordered, digest


def record_key(prefix: str, record: dict[str, Any]) -> str:
    return canonical_key(prefix, {key: str(value) for key, value in record.items()})


def binding_key(prefix: str, binding: Binding) -> str:
    return canonical_key(prefix, {
        "id": binding.id,
        "path": binding.path,
        "sha256": binding.sha256,
        "byte_count": str(binding.byte_count),
    })


def byte_record(source: ModuleType, path: Path, relative: str, scope: str) -> dict[str, Any]:
    return {
        "path": relative,
        "sha256": source.sha256(path),
        "logical_lines": source.logical_lines(path),
        "byte_count": path.stat().st_size,
        "citation": source.full_citation(scope, relative, source.logical_lines(path)),
    }


def file_binding(source: ModuleType, repo: Path, binding_id: str, relative: str) -> Binding:
    path = source.repo_path(repo, relative, f"{binding_id} binding")
    require(path.is_file(), f"missing {binding_id} binding: {relative}")
    return Binding(binding_id, relative, source.sha256(path), path.stat().st_size)


def validate_binding(value: Any, expected: Binding, label: str) -> None:
    require(isinstance(value, dict), f"{label} must be a table")
    require_exact_keys(value, BINDING_KEYS, label)
    require(value == expected.record(), f"{label} drift")


def validate_binding_array(value: Any, expected: Iterable[Binding], label: str) -> None:
    expected_records = [item.record() for item in expected]
    require(isinstance(value, list), f"{label} must be an array")
    for index, item in enumerate(value):
        require(isinstance(item, dict), f"{label} {index} must be a table")
        require_exact_keys(item, BINDING_KEYS, f"{label} {index}")
    require(value == expected_records, f"{label} drift")


def validate_records(value: Any, expected: tuple[dict[str, Any], ...],
                     keys: set[str], label: str) -> None:
    require(isinstance(value, list), f"{label} must be an array")
    for index, item in enumerate(value):
        require(isinstance(item, dict), f"{label} {index} must be a table")
        require_exact_keys(item, keys, f"{label} {index}")
    require(value == list(expected), f"{label} evidence drift")


def load_toml(path: Path, label: str) -> dict[str, Any]:
    require(path.is_file(), f"missing {label}: {path}")
    with path.open("rb") as handle:
        value = tomllib.load(handle)
    require(isinstance(value, dict), f"{label} must be a TOML table")
    return value


def load_rows(path: Path, label: str) -> tuple[dict[str, str], ...]:
    require(path.is_file(), f"missing {label}: {path}")
    with path.open(encoding="utf-8", newline="") as handle:
        rows = tuple(csv.DictReader(handle, delimiter="\t"))
    require(rows, f"{label} is empty")
    return rows


def expected_source_receipt_paths(source_authority: Any) -> tuple[Path, ...]:
    component_paths = [
        source_authority.receipt_directory / f"{component_id}.source-review.toml"
        for component_id in source_authority.components
    ]
    return tuple([
        *component_paths,
        source_authority.receipt_directory / "support.source-review.toml",
        source_authority.receipt_directory / "overlays.source-review.toml",
    ])


def validate_source_prerequisite(source: ModuleType, source_authority: Any) -> None:
    expected = set(expected_source_receipt_paths(source_authority))
    actual = {
        path for path in source_authority.receipt_directory.rglob("*") if path.is_file()
    }
    require(actual == expected,
            f"source-review prerequisite receipt set drift: {len(actual)}/{len(expected)}")
    require(len(expected) == EXPECTED_COMPONENT_COUNT + 2,
            "source-review prerequisite receipt denominator drift")
    for path in expected:
        require(source.git_tracked(source_authority.repo, path),
                f"source-review prerequisite is not tracked: {path}")
    source.validate_component_set(source_authority, list(source_authority.components))
    source.validate_support_receipt(
        source_authority.receipt_directory / "support.source-review.toml",
        source_authority,
    )
    source.validate_overlay_receipt(
        source_authority.receipt_directory / "overlays.source-review.toml",
        source_authority,
    )
    source.require_frozen_scopes(
        source_authority.repo,
        EXPECTED_WORKSPACE_BASE_REF,
        [path.relative_to(source_authority.repo).as_posix() for path in expected],
    )
    hasher = hashlib.sha256()
    total_bytes = 0
    total_lines = 0
    root = source_authority.receipt_directory
    for path in sorted(expected, key=lambda item: item.relative_to(root).as_posix()):
        relative = path.relative_to(root).as_posix()
        payload = path.read_bytes()
        hasher.update(relative.encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(payload)
        hasher.update(b"\0")
        total_bytes += len(payload)
        total_lines += len(payload.splitlines())
    require(hasher.hexdigest() == EXPECTED_SOURCE_RECEIPT_TREE_SHA256,
            "source-review prerequisite tree hash drift")
    require(total_bytes == EXPECTED_SOURCE_RECEIPT_BYTES,
            "source-review prerequisite byte denominator drift")
    require(total_lines == EXPECTED_SOURCE_RECEIPT_LINES,
            "source-review prerequisite line denominator drift")


def load_contracts(source: ModuleType, source_authority: Any) -> tuple[
    tuple[dict[str, Any], ...], tuple[dict[str, Any], ...], str, str
]:
    manifest = source_authority.manifest
    owner_path_value = str(manifest["owner_contracts"])
    profile_path_value = str(manifest["field_profiles"])
    owner_path = source.repo_path(source_authority.repo, owner_path_value,
                                  "owner-contract authority")
    profile_path = source.repo_path(source_authority.repo, profile_path_value,
                                    "field-profile authority")
    require(source.sha256(owner_path) == EXPECTED_CONTRACT_HASHES["owner_contract"],
            "owner-contract bytes drifted from ownership launch authority")
    require(source.sha256(profile_path) == EXPECTED_CONTRACT_HASHES["field_profile"],
            "field-profile bytes drifted from ownership launch authority")
    owner_contract = load_toml(owner_path, "owner-contract authority")
    field_profiles = load_toml(profile_path, "field-profile authority")
    require_exact_keys(
        owner_contract,
        {"schema_version", "upstream_ref", "review_basis", "field_rule",
         "failure_rule", "family"},
        "owner-contract authority",
    )
    require_exact_keys(field_profiles, {"schema_version", "profile"},
                       "field-profile authority")
    require(owner_contract["schema_version"] == 1,
            "owner-contract schema drift")
    require(owner_contract["upstream_ref"] == EXPECTED_UPSTREAM_REF,
            "owner-contract upstream drift")
    require(field_profiles["schema_version"] == 1,
            "field-profile schema drift")
    families_value = owner_contract["family"]
    profiles_value = field_profiles["profile"]
    require(isinstance(families_value, list) and len(families_value) == 8,
            "owner-contract family denominator drift")
    require(isinstance(profiles_value, list) and len(profiles_value) == 4,
            "field-profile denominator drift")
    families: list[dict[str, Any]] = []
    family_ids: set[str] = set()
    family_keys = {
        "id", "campaign", "unit_prefixes", "review_status", "configuration_rule",
        "ownership_rule", "synchronization_rule", "destruction_rule",
    }
    for index, family in enumerate(families_value):
        require(isinstance(family, dict), f"owner family {index} must be a table")
        require_exact_keys(family, family_keys, f"owner family {index}")
        family_id = family["id"]
        require(isinstance(family_id, str) and family_id not in family_ids,
                f"duplicate or invalid owner family: {family_id}")
        require(family["review_status"] == "reviewed",
                f"owner family is not frozen reviewed: {family_id}")
        require(isinstance(family["unit_prefixes"], list)
                and family["unit_prefixes"],
                f"owner family has no unit prefixes: {family_id}")
        family_ids.add(family_id)
        families.append(family)
    profiles: list[dict[str, Any]] = []
    profile_ids: set[str] = set()
    for index, profile in enumerate(profiles_value):
        require(isinstance(profile, dict), f"field profile {index} must be a table")
        required = {"id", "campaign", "defines", "include_roots"}
        allowed = {*required, "stub_headers"}
        require(required <= set(profile) <= allowed,
                f"field profile {index} shape drift")
        profile_id = profile["id"]
        require(isinstance(profile_id, str) and profile_id not in profile_ids,
                f"duplicate or invalid field profile: {profile_id}")
        require(isinstance(profile["defines"], list)
                and isinstance(profile["include_roots"], list),
                f"invalid field profile arrays: {profile_id}")
        profile_ids.add(profile_id)
        profiles.append(profile)
    return tuple(families), tuple(profiles), owner_path_value, profile_path_value


def load_ledgers(source: ModuleType, source_authority: Any,
                 profiles: tuple[dict[str, Any], ...]) -> tuple[
    dict[str, tuple[dict[str, str], ...]],
    dict[str, dict[str, tuple[str, ...]]],
]:
    manifest = source_authority.manifest
    expected_headers = {
        "field": {
            "campaign", "configuration", "source_path", "qualified_type",
            "field_order", "field_name", "declared_type", "declaration_line",
            "ownership_unit", "ownership_review",
        },
        "lifecycle": {
            "campaign", "source_path", "line", "event_class", "matched_token",
            "source_evidence", "ownership_unit", "review_status",
        },
        "configuration": {
            "campaign", "source_path", "line", "occurrence_count",
            "occurrence_lines", "authority_kind", "token", "enclosing_condition",
            "ownership_unit", "review_status",
        },
        "dependency": {
            "campaign", "source_path", "line", "dependency_syntax",
            "dependency_token", "resolution_kind", "resolved_path",
            "resolved_sha256", "source_unit", "dependency_unit",
        },
    }
    ledgers: dict[str, tuple[dict[str, str], ...]] = {}
    by_component: dict[str, dict[str, tuple[str, ...]]] = {
        component_id: {} for component_id in source_authority.components
    }
    unit_to_component = {
        unit_id: unit.component_id for unit_id, unit in source_authority.units.items()
    }
    for kind, (prefix, manifest_key, unit_column) in LEDGER_SPECS.items():
        path = source.repo_path(source_authority.repo, str(manifest[manifest_key]),
                                f"{kind} authority")
        rows = load_rows(path, f"{kind} authority")
        require(set(rows[0]) == expected_headers[kind],
                f"{kind} authority columns drift")
        require(len(rows) == EXPECTED_LEDGER_COUNTS[kind],
                f"{kind} authority denominator drift")
        keys: list[str] = []
        component_keys: dict[str, list[str]] = {
            component_id: [] for component_id in source_authority.components
        }
        for index, row in enumerate(rows):
            unit_id = row[unit_column]
            require(unit_id in source_authority.units,
                    f"{kind} authority invents unit at row {index + 2}: {unit_id}")
            unit = source_authority.units[unit_id]
            require(row["campaign"] == unit.campaign,
                    f"{kind} authority campaign drift at row {index + 2}")
            owner = source_authority.owners.get(row["source_path"])
            require(owner is not None and owner.unit == unit_id,
                    f"{kind} authority source ownership drift at row {index + 2}")
            source_path = source.upstream_path(source_authority.upstream,
                                               row["source_path"], f"{kind} source")
            line_column = "declaration_line" if kind == "field" else "line"
            line = int(row[line_column])
            require((kind == "field" and line == 0)
                    or 1 <= line <= source.logical_lines(source_path),
                    f"{kind} authority line exceeds source at row {index + 2}")
            if kind == "field":
                require(any(profile["id"] == row["configuration"]
                            and profile["campaign"] == row["campaign"]
                            for profile in profiles),
                        f"field authority profile drift at row {index + 2}")
            key = canonical_key(prefix, row)
            keys.append(key)
            component_keys[unit_to_component[unit_id]].append(key)
        ordered, digest = canonical_digest(keys)
        require(len(ordered) == len(rows), f"duplicate {kind} authority rows")
        require(digest == EXPECTED_LEDGER_DIGESTS[kind],
                f"{kind} authority canonical digest drift")
        ledgers[kind] = rows
        for component_id, values in component_keys.items():
            by_component[component_id][kind] = canonical_digest(values)[0]
    return ledgers, by_component


def source_receipt_binding(source: ModuleType, source_authority: Any,
                           binding_id: str, filename: str) -> Binding:
    path = source_authority.receipt_directory / filename
    relative = path.relative_to(source_authority.repo).as_posix()
    return Binding(binding_id, relative, source.sha256(path), path.stat().st_size)


def source_receipt_authority_key(source: ModuleType, source_authority: Any,
                                 binding: Binding) -> str:
    source.repo_path(source_authority.repo, binding.path,
                     "source-review prerequisite binding")
    return binding_key("source-review-receipt-binding", binding)


def derive_contract_membership(
    source_authority: Any,
    families: tuple[dict[str, Any], ...],
    profiles: tuple[dict[str, Any], ...],
) -> tuple[dict[str, tuple[str, ...]], dict[str, tuple[str, ...]]]:
    family_by_component: dict[str, list[str]] = {
        component_id: [] for component_id in source_authority.components
    }
    profile_by_component: dict[str, list[str]] = {
        component_id: [] for component_id in source_authority.components
    }
    family_membership: list[str] = []
    profile_membership: list[str] = []
    matched_units: set[str] = set()
    for unit_id, unit in source_authority.units.items():
        matches = [
            family for family in families
            if family["campaign"] == unit.campaign
            and any(unit_id.startswith(prefix) for prefix in family["unit_prefixes"])
        ]
        require(len(matches) == 1,
                f"ownership unit does not have exactly one owner family: {unit_id}")
        family = matches[0]
        matched_units.add(unit_id)
        component_families = family_by_component[unit.component_id]
        if family["id"] not in component_families:
            component_families.append(family["id"])
        family_membership.append(canonical_key("owner-family", {
            "campaign": unit.campaign,
            "component_id": unit.component_id,
            "family_id": family["id"],
            "ownership_unit": unit_id,
        }))
    require(len(matched_units) == EXPECTED_UNIT_COUNT,
            "owner-family unit denominator drift")
    family_keys, family_digest = canonical_digest(family_membership)
    require(len(family_keys) == EXPECTED_UNIT_COUNT,
            "owner-family membership denominator drift")
    require(family_digest == EXPECTED_OWNER_FAMILY_MEMBERSHIP_SHA256,
            "owner-family membership digest drift")
    for component_id, component in source_authority.components.items():
        campaigns = {source_authority.units[unit_id].campaign for unit_id in component.units}
        for profile in profiles:
            if profile["campaign"] in campaigns:
                profile_by_component[component_id].append(profile["id"])
                profile_membership.append(canonical_key("field-profile", {
                    "campaign": profile["campaign"],
                    "component_id": component_id,
                    "profile_id": profile["id"],
                }))
    profile_keys, profile_digest = canonical_digest(profile_membership)
    require(len(profile_keys) == 61, "field-profile membership denominator drift")
    require(profile_digest == EXPECTED_PROFILE_MEMBERSHIP_SHA256,
            "field-profile membership digest drift")
    declared_family_ids = [family["id"] for family in families]
    for component_id, values in family_by_component.items():
        family_by_component[component_id] = [
            family_id for family_id in declared_family_ids if family_id in values
        ]
    return (
        {key: tuple(value) for key, value in family_by_component.items()},
        {key: tuple(value) for key, value in profile_by_component.items()},
    )


def derive_component_dependencies(source_authority: Any) -> dict[str, tuple[str, ...]]:
    cross_unit_keys: list[str] = []
    dependency_components: dict[str, set[str]] = {
        component_id: set() for component_id in source_authority.components
    }
    declared_edges = 0
    intra_component_edges = 0
    for unit in source_authority.units.values():
        for dependency_unit_id in unit.dependency_units:
            declared_edges += 1
            require(dependency_unit_id in source_authority.units,
                    f"unit dependency invents owner: {unit.unit}->{dependency_unit_id}")
            dependency_unit = source_authority.units[dependency_unit_id]
            if dependency_unit.component_id == unit.component_id:
                intra_component_edges += 1
                continue
            require(dependency_unit.order_group < unit.order_group,
                    "cross-component ownership dependency is not in an earlier wave: "
                    f"{unit.component_id}->{dependency_unit.component_id}")
            dependency_components[unit.component_id].add(dependency_unit.component_id)
            cross_unit_keys.append(canonical_key("dependency-unit-seam", {
                "dependency_component": dependency_unit.component_id,
                "dependency_unit": dependency_unit_id,
                "source_component": unit.component_id,
                "source_unit": unit.unit,
            }))
    require(declared_edges == 511, "declared unit-dependency denominator drift")
    require(intra_component_edges == 59, "intra-SCC unit-dependency denominator drift")
    seam_keys, seam_digest = canonical_digest(cross_unit_keys)
    require(len(seam_keys) == EXPECTED_UNIT_SEAM_COUNT,
            "cross-component unit seam denominator drift")
    require(seam_digest == EXPECTED_UNIT_SEAM_SHA256,
            "cross-component unit seam digest drift")
    pair_count = sum(len(value) for value in dependency_components.values())
    require(pair_count == EXPECTED_COMPONENT_DEPENDENCY_COUNT,
            "component-dependency pair denominator drift")
    return {key: tuple(sorted(value)) for key, value in dependency_components.items()}


def validate_scc_partition(source_authority: Any) -> None:
    keys = [
        canonical_key("scc-partition", {
            "component_id": component.component_id,
            "order_group": str(component.order_group),
            "units": ";".join(sorted(component.units)),
        })
        for component in source_authority.components.values()
    ]
    ordered, digest = canonical_digest(keys)
    require(len(ordered) == EXPECTED_COMPONENT_COUNT,
            "SCC partition denominator drift")
    require(digest == EXPECTED_SCC_PARTITION_SHA256,
            "SCC partition digest drift")


def validate_independent_scc(source_authority: Any,
                             dependency_rows: tuple[dict[str, str], ...]) -> None:
    unit_ids = set(source_authority.units)
    known_pairs = {
        (row["source_unit"], row["dependency_unit"])
        for row in dependency_rows
        if row["source_unit"] in unit_ids and row["dependency_unit"] in unit_ids
    }
    self_pairs = {pair for pair in known_pairs if pair[0] == pair[1]}
    nonself_pairs = known_pairs - self_pairs
    require(len(known_pairs) == 565,
            "raw dependency known-unit pair denominator drift")
    require(len(self_pairs) == 54,
            "raw dependency self-pair denominator drift")
    require(len(nonself_pairs) == 511,
            "raw dependency nonself-pair denominator drift")
    order_pairs = {
        (unit.unit, dependency)
        for unit in source_authority.units.values()
        for dependency in unit.dependency_units
    }
    require(order_pairs == nonself_pairs,
            "SCC order dependency seams disagree with raw dependency authority")

    forward = {unit_id: [] for unit_id in unit_ids}
    reverse = {unit_id: [] for unit_id in unit_ids}
    for source_unit, dependency_unit in nonself_pairs:
        forward[source_unit].append(dependency_unit)
        reverse[dependency_unit].append(source_unit)
    visited: set[str] = set()
    finish_order: list[str] = []

    def finish(unit_id: str) -> None:
        if unit_id in visited:
            return
        visited.add(unit_id)
        for dependency in sorted(forward[unit_id]):
            finish(dependency)
        finish_order.append(unit_id)

    for unit_id in sorted(unit_ids):
        finish(unit_id)
    visited.clear()
    discovered: list[frozenset[str]] = []

    def collect(unit_id: str, members: set[str]) -> None:
        if unit_id in visited:
            return
        visited.add(unit_id)
        members.add(unit_id)
        for dependency in sorted(reverse[unit_id]):
            collect(dependency, members)

    for unit_id in reversed(finish_order):
        if unit_id in visited:
            continue
        members: set[str] = set()
        collect(unit_id, members)
        discovered.append(frozenset(members))
    frozen = {
        frozenset(component.units)
        for component in source_authority.components.values()
    }
    require(set(discovered) == frozen,
            "independent raw-dependency SCC partition drift")
    require(len(discovered) == EXPECTED_COMPONENT_COUNT,
            "independent SCC denominator drift")
    size_counts: dict[int, int] = {}
    for members in discovered:
        size_counts[len(members)] = size_counts.get(len(members), 0) + 1
    require(size_counts == {1: 109, 2: 3, 4: 1, 5: 1, 11: 1},
            "independent SCC size distribution drift")


def build_component_expectations(
    source: ModuleType,
    source_authority: Any,
    families: tuple[dict[str, Any], ...],
    profiles: tuple[dict[str, Any], ...],
    owner_contract_path: str,
    profile_path: str,
    ledger_keys_by_component: dict[str, dict[str, tuple[str, ...]]],
) -> dict[str, ComponentExpectation]:
    family_by_component, profile_by_component = derive_contract_membership(
        source_authority, families, profiles
    )
    dependencies = derive_component_dependencies(source_authority)
    family_by_id = {family["id"]: family for family in families}
    profile_by_id = {profile["id"]: profile for profile in profiles}
    owner_contract_file = source.repo_path(source_authority.repo, owner_contract_path,
                                           "owner-contract authority")
    profile_file = source.repo_path(source_authority.repo, profile_path,
                                    "field-profile authority")
    result: dict[str, ComponentExpectation] = {}
    all_source_keys: list[str] = []
    all_target_keys: list[str] = []
    for component_id, component in source_authority.components.items():
        owner_families = family_by_component[component_id]
        field_profiles = profile_by_component[component_id]
        owner_bindings = tuple(
            Binding(family_id, owner_contract_path, source.sha256(owner_contract_file),
                    owner_contract_file.stat().st_size)
            for family_id in owner_families
        )
        profile_bindings = tuple(
            Binding(profile_id, profile_path, source.sha256(profile_file),
                    profile_file.stat().st_size)
            for profile_id in field_profiles
        )
        expected_owners = sorted(
            source_authority.owners_by_component[component_id],
            key=lambda owner: owner.source_path,
        )
        source_records: list[dict[str, Any]] = []
        target_records: list[dict[str, Any]] = []
        for owner in expected_owners:
            source_path = source.upstream_path(source_authority.upstream, owner.source_path,
                                               "ownership-reviewed source")
            record = byte_record(source, source_path, owner.source_path, "source")
            record["disposition"] = owner.disposition
            source_records.append(record)
            all_source_keys.append(record_key("source-binding", record))
            if owner.translated:
                translation = source_authority.translations[owner.source_path]
                target_path = source.repo_path(source_authority.repo,
                                               translation.target_path,
                                               "ownership-reviewed target")
                target_record = byte_record(source, target_path, translation.target_path,
                                            "target")
                target_records.append(target_record)
                all_target_keys.append(record_key("target-binding", target_record))
        target_records.sort(key=lambda record: record["path"])
        ledgers: dict[str, LedgerAuthority] = {}
        authority_keys: list[str] = [f"component:{component_id}"]
        for kind, (prefix, _, _) in LEDGER_SPECS.items():
            keys = ledger_keys_by_component[component_id][kind]
            digest = canonical_digest(keys)[1]
            ledgers[kind] = LedgerAuthority(prefix, keys, digest)
            authority_keys.extend(keys)
        for unit_id in component.units:
            unit = source_authority.units[unit_id]
            family_id = next(
                family["id"] for family in families
                if family["campaign"] == unit.campaign
                and any(unit_id.startswith(prefix) for prefix in family["unit_prefixes"])
            )
            authority_keys.append(canonical_key("owner-family", {
                "campaign": unit.campaign,
                "component_id": component_id,
                "family_id": family_id,
                "ownership_unit": unit_id,
            }))
        for profile_id in field_profiles:
            profile = profile_by_id[profile_id]
            authority_keys.append(canonical_key("field-profile", {
                "campaign": profile["campaign"],
                "component_id": component_id,
                "profile_id": profile_id,
            }))
        authority_keys.extend(record_key("source-binding", record)
                              for record in source_records)
        authority_keys.extend(record_key("target-binding", record)
                              for record in target_records)
        source_binding = source_receipt_binding(
            source, source_authority, component_id,
            f"{component_id}.source-review.toml",
        )
        authority_keys.append(source_receipt_authority_key(
            source, source_authority, source_binding
        ))
        dependency_ids = dependencies[component_id]
        authority_keys.extend(
            f"component-dependency:{component_id}->{dependency_id}"
            for dependency_id in dependency_ids
        )
        ordered_keys, authority_sha = canonical_digest(authority_keys)
        result[component_id] = ComponentExpectation(
            component_id,
            owner_families,
            field_profiles,
            owner_bindings,
            profile_bindings,
            dependency_ids,
            ledgers,
            source_binding,
            tuple(source_records),
            tuple(target_records),
            ordered_keys,
            authority_sha,
        )
        require(set(owner_families) <= set(family_by_id),
                f"component invents owner family: {component_id}")
        require(set(field_profiles) <= set(profile_by_id),
                f"component invents field profile: {component_id}")
    source_keys, source_digest = canonical_digest(all_source_keys)
    target_keys, target_digest = canonical_digest(all_target_keys)
    require(len(source_keys) == EXPECTED_SOURCE_COUNT,
            "source binding denominator drift")
    require(source_digest == EXPECTED_SOURCE_BINDING_SHA256,
            "source binding digest drift")
    require(len(target_keys) == EXPECTED_TARGET_COUNT,
            "target binding denominator drift")
    require(target_digest == EXPECTED_TARGET_BINDING_SHA256,
            "target binding digest drift")
    all_authority_keys, all_authority_digest = canonical_digest(
        key for expectation in result.values() for key in expectation.authority_keys
    )
    require(len(all_authority_keys) == EXPECTED_COMPONENT_AUTHORITY_COUNT,
            "component static authority denominator drift")
    require(all_authority_digest == EXPECTED_COMPONENT_AUTHORITY_SHA256,
            "component static authority digest drift")
    return result


def build_support_expectation(source: ModuleType, source_authority: Any) -> tuple[
    Binding, tuple[dict[str, Any], ...], tuple[str, ...], str
]:
    records: list[dict[str, Any]] = []
    for artifact in sorted(source_authority.support_artifacts.values(),
                           key=lambda item: item.path):
        path = source.repo_path(source_authority.repo, artifact.path,
                                "ownership-reviewed support artifact")
        record = byte_record(source, path, artifact.path, "support")
        record.update({
            "artifact_role": artifact.artifact_role,
            "review_overlay": artifact.review_overlay,
            "source_authority": artifact.source_authority,
            "disposition": artifact.disposition,
        })
        records.append(record)
    support_keys, support_digest = canonical_digest(
        record_key("support-binding", record) for record in records
    )
    require(len(support_keys) == EXPECTED_SUPPORT_COUNT,
            "support binding denominator drift")
    require(support_digest == EXPECTED_SUPPORT_BINDING_SHA256,
            "support binding digest drift")
    binding = source_receipt_binding(
        source, source_authority, "support", "support.source-review.toml"
    )
    authority_keys, authority_sha = canonical_digest([
        *support_keys,
        source_receipt_authority_key(source, source_authority, binding),
    ])
    require(len(authority_keys) == EXPECTED_SUPPORT_COUNT + 1,
            "support static authority denominator drift")
    require(authority_sha == EXPECTED_SUPPORT_AUTHORITY_SHA256,
            "support static authority digest drift")
    return binding, tuple(records), authority_keys, authority_sha


def physical_file_record(source: ModuleType, path: Path, relative: str) -> dict[str, Any]:
    return {
        "path": relative,
        "sha256": source.sha256(path),
        "logical_lines": source.logical_lines(path),
        "byte_count": path.stat().st_size,
    }


def tree_binding_record(source: ModuleType, source_authority: Any,
                        relative: str) -> dict[str, Any]:
    artifact = source_authority.tree_artifacts[relative]
    root = source.repo_path(source_authority.repo, relative, "ownership overlay tree")
    members = sorted(artifact.members)
    require(members, f"ownership overlay tree is empty: {relative}")
    require(source.tree_digest(root) == artifact.tree_sha256,
            f"ownership overlay tree hash drift: {relative}")
    member_paths = [
        source.repo_path(source_authority.repo, member, "ownership overlay tree member")
        for member in members
    ]
    return {
        "path": relative,
        "tree_sha256": artifact.tree_sha256,
        "file_count": len(member_paths),
        "logical_lines": sum(source.logical_lines(path) for path in member_paths),
        "byte_count": sum(path.stat().st_size for path in member_paths),
    }


def basic_physical_record(record: dict[str, Any]) -> dict[str, Any]:
    return {key: record[key] for key in ("path", "sha256", "logical_lines", "byte_count")}


def component_contract_authority_keys(expectation: ComponentExpectation) -> list[str]:
    prefixes = (
        "field-raw:",
        "lifecycle-raw:",
        "configuration-raw:",
        "dependency-raw:",
        "owner-family:",
        "field-profile:",
    )
    return [key for key in expectation.authority_keys if key.startswith(prefixes)]


def build_overlay_expectations(
    source: ModuleType,
    source_authority: Any,
    components: dict[str, ComponentExpectation],
    support_records: tuple[dict[str, Any], ...],
) -> dict[str, OwnershipOverlayExpectation]:
    support_by_path = {record["path"]: record for record in support_records}
    result: dict[str, OwnershipOverlayExpectation] = {}
    overlay_ids = list(source.EXPECTED_OVERLAY_IDS)
    require(len(overlay_ids) == EXPECTED_OVERLAY_COUNT,
            "ownership overlay denominator drift")
    for ordinal, overlay_id in enumerate(overlay_ids, start=1):
        source_overlay = source_authority.overlays[overlay_id]
        component_ids = tuple(source_overlay.component_ids)
        support_paths = tuple(source_overlay.support_paths)
        source_bindings_by_path: dict[str, dict[str, Any]] = {}
        target_bindings_by_path: dict[str, dict[str, Any]] = {}
        authority_keys: list[str] = []
        for component_id in component_ids:
            component = components[component_id]
            authority_keys.append(f"component:{component_id}")
            authority_keys.extend(component_contract_authority_keys(component))
            for record in component.source_records:
                physical = basic_physical_record(record)
                source_bindings_by_path[physical["path"]] = physical
            for record in component.target_records:
                physical = basic_physical_record(record)
                target_bindings_by_path[physical["path"]] = physical
        source_bindings = tuple(source_bindings_by_path[path]
                                for path in sorted(source_bindings_by_path))
        target_bindings = tuple(target_bindings_by_path[path]
                                for path in sorted(target_bindings_by_path))
        support_bindings = tuple(
            basic_physical_record(support_by_path[path]) for path in sorted(support_paths)
        )
        authority_keys.extend(f"support:{path}" for path in support_paths)

        artifact_bindings = tuple(
            physical_file_record(
                source,
                source.repo_path(source_authority.repo, path,
                                 "ownership overlay artifact"),
                path,
            )
            for path in sorted(source_overlay.artifact_paths)
        )
        tree_bindings = tuple(
            tree_binding_record(source, source_authority, path)
            for path, _ in sorted(source_overlay.tree_bindings)
        )
        external_bindings = tuple(
            physical_file_record(
                source,
                source.upstream_path(source_authority.upstream, path,
                                     "ownership overlay external authority"),
                path,
            )
            for path in sorted(source_overlay.external_paths)
        )
        generated_bindings = tuple(
            physical_file_record(
                source,
                source.upstream_path(source_authority.upstream, path,
                                     "ownership overlay generated authority"),
                path,
            )
            for path in sorted(source_overlay.generated_paths)
        )
        physical_sets = {
            "source": source_bindings,
            "target": target_bindings,
            "support": support_bindings,
            "artifact": artifact_bindings,
            "tree": tree_bindings,
            "external": external_bindings,
            "generated": generated_bindings,
        }
        for category, records in physical_sets.items():
            authority_keys.extend(record_key(f"physical-{category}", record)
                                  for record in records)
        authority_keys.append(canonical_key("source-review-overlay", {
            "authority_record_count": str(source_overlay.authority_record_count),
            "authority_sha256": source_overlay.authority_sha256,
            "id": overlay_id,
        }))
        ordered, digest = canonical_digest(authority_keys)
        expected_count, expected_digest = EXPECTED_OVERLAY_AUTHORITIES[ordinal - 1]
        require(len(ordered) == expected_count,
                f"ownership overlay authority denominator drift: {overlay_id} "
                f"({len(ordered)}/{expected_count})")
        require(digest == expected_digest,
                f"ownership overlay authority digest drift: {overlay_id}")
        result[overlay_id] = OwnershipOverlayExpectation(
            overlay_id,
            ordinal,
            component_ids,
            support_paths,
            source_bindings,
            target_bindings,
            support_bindings,
            artifact_bindings,
            tree_bindings,
            external_bindings,
            generated_bindings,
            ordered,
            digest,
        )
    return result


def canonical_component_receipt_path(authority: OwnershipAuthority,
                                     component_id: str) -> Path:
    return authority.receipt_directory / f"{component_id}.ownership-review.toml"


def ownership_receipt_binding(authority: OwnershipAuthority,
                              binding_id: str, path: Path) -> Binding:
    require(path.is_file(), f"missing prerequisite ownership receipt: {path}")
    require(authority.source.git_tracked(authority.repo, path),
            f"prerequisite ownership receipt is not tracked: {path}")
    relative = path.relative_to(authority.repo).as_posix()
    return Binding(binding_id, relative, authority.source.sha256(path),
                   path.stat().st_size)


def authority_file_lines(authority: OwnershipAuthority) -> dict[tuple[str, str], int]:
    paths = [
        str(authority.source_authority.manifest[key])
        for key in (
            "field_inventory",
            "lifecycle_inventory",
            "configuration_inventory",
            "dependency_inventory",
            "owner_contracts",
            "field_profiles",
        )
    ]
    return {
        ("authority", relative): authority.source.logical_lines(
            authority.source.repo_path(authority.repo, relative,
                                       "ownership finding authority")
        )
        for relative in paths
    }


def parse_citation(citation: str, label: str) -> tuple[str, str, int, int]:
    match = re.fullmatch(
        r"(source|target|support|authority|source-review):(.+):(\d+)-(\d+)",
        citation,
    )
    require(match is not None, f"invalid {label} citation: {citation}")
    scope, path, first_value, last_value = match.groups()
    first, last = int(first_value), int(last_value)
    require(first >= 1 and last >= first, f"invalid {label} citation: {citation}")
    return scope, path, first, last


def validate_findings(
    findings: Any,
    open_findings: Any,
    allowed_lines: dict[tuple[str, str], int],
    allowed_authority_keys: set[str],
    id_pattern: re.Pattern[str],
    label: str,
    overlay_id: str | None = None,
) -> tuple[str, ...]:
    require(isinstance(findings, list), f"{label} findings must be an array")
    finding_ids: list[str] = []
    expected_keys = OVERLAY_FINDING_KEYS if overlay_id is not None else FINDING_KEYS
    for index, finding in enumerate(findings):
        require(isinstance(finding, dict), f"{label} finding {index} must be a table")
        require_exact_keys(finding, expected_keys, f"{label} finding {index}")
        if overlay_id is not None:
            require(finding["overlay_id"] == overlay_id,
                    f"{label} finding overlay drift")
        finding_id = finding["id"]
        require(isinstance(finding_id, str) and id_pattern.fullmatch(finding_id),
                f"{label} has unstable finding ID: {finding_id}")
        require(finding_id not in finding_ids,
                f"{label} duplicates finding ID: {finding_id}")
        require(finding["severity"] in EXPECTED_SEVERITIES,
                f"{label} severity drift: {finding_id}")
        require(isinstance(finding["summary"], str) and finding["summary"].strip(),
                f"{label} finding has no summary: {finding_id}")
        domains = finding["review_domains"]
        require(isinstance(domains, list) and domains,
                f"{label} finding has no review domains: {finding_id}")
        allowed_domains = (EXPECTED_OVERLAY_COVERAGE
                           if overlay_id is not None else EXPECTED_COVERAGE)
        require(len(domains) == len(set(domains))
                and all(domain in allowed_domains for domain in domains),
                f"{label} finding review-domain drift: {finding_id}")
        citations = finding["citations"]
        require(isinstance(citations, list) and citations,
                f"{label} finding has no citations: {finding_id}")
        require(len(citations) == len(set(citations)),
                f"{label} finding duplicates citations: {finding_id}")
        for citation in citations:
            require(isinstance(citation, str),
                    f"{label} finding citation must be a string")
            scope, path, _, last = parse_citation(citation, f"{label} finding")
            require((scope, path) in allowed_lines,
                    f"{label} finding cites outside its authority: {citation}")
            require(last <= allowed_lines[(scope, path)],
                    f"{label} finding citation exceeds file: {citation}")
        finding_authority = finding["authority_keys"]
        require(isinstance(finding_authority, list) and finding_authority,
                f"{label} finding has no authority keys: {finding_id}")
        require(len(finding_authority) == len(set(finding_authority)),
                f"{label} finding duplicates authority keys: {finding_id}")
        require(all(isinstance(key, str) and key in allowed_authority_keys
                    for key in finding_authority),
                f"{label} finding cites an unknown authority key: {finding_id}")
        finding_ids.append(finding_id)
    require(type(open_findings) is int,
            f"{label} open_findings must be an integer")
    require(open_findings == len(findings),
            f"{label} open_findings count drift")
    return tuple(finding_ids)


def validate_component_receipt(path: Path,
                               authority: OwnershipAuthority) -> ComponentResult:
    component_id = path.name.removesuffix(".ownership-review.toml")
    require(component_id in authority.components,
            f"unknown ownership component receipt: {path.name}")
    require(path == canonical_component_receipt_path(authority, component_id),
            f"noncanonical ownership component receipt: {path.name}")
    require(path.is_file() and not path.is_symlink(),
            f"ownership component receipt is not a regular file: {path}")
    require(authority.source.git_tracked(authority.repo, path),
            f"ownership component receipt is not tracked: {path}")
    receipt = load_toml(path, "ownership component receipt")
    require_exact_keys(receipt, COMPONENT_RECEIPT_KEYS,
                       f"ownership receipt {path.name}")
    source_component = authority.source_authority.components[component_id]
    expectation = authority.components[component_id]
    expected_scalars = {
        "schema_version": 1,
        "receipt_kind": "ownership-review-component",
        "component_id": component_id,
        "units": list(source_component.units),
        "owner_families": list(expectation.owner_families),
        "field_profiles": list(expectation.field_profiles),
        "upstream_ref": EXPECTED_UPSTREAM_REF,
        "workspace_base_ref": EXPECTED_WORKSPACE_BASE_REF,
        "role": authority.source_authority.source_reviewer_role,
        "review_wave": source_component.review_wave,
        "coverage": EXPECTED_COVERAGE,
        "dependency_components": list(expectation.dependency_components),
        "authority_record_count": len(expectation.authority_keys),
        "authority_sha256": expectation.authority_sha256,
        "authority_keys": list(expectation.authority_keys),
        "attestation": "reviewed-complete-component-ownership-lifetime-abi-authority",
    }
    for key, expected in expected_scalars.items():
        require(receipt[key] == expected,
                f"ownership receipt {path.name} {key} drift")
    require(authority.source.valid_review_run_id(receipt["review_run_id"]),
            f"ownership receipt {path.name} has invalid review_run_id")
    validate_binding_array(receipt["owner_contract_bindings"],
                           expectation.owner_contract_bindings,
                           f"ownership receipt {path.name} owner contracts")
    validate_binding_array(receipt["field_profile_bindings"],
                           expectation.field_profile_bindings,
                           f"ownership receipt {path.name} field profiles")
    dependency_bindings = tuple(
        ownership_receipt_binding(
            authority,
            dependency_id,
            canonical_component_receipt_path(authority, dependency_id),
        )
        for dependency_id in expectation.dependency_components
    )
    validate_binding_array(receipt["dependency_ownership_receipts"],
                           dependency_bindings,
                           f"ownership receipt {path.name} dependencies")
    for kind in LEDGER_SPECS:
        key = f"{kind}_authority"
        value = receipt[key]
        require(isinstance(value, dict),
                f"ownership receipt {path.name} {key} must be a table")
        require_exact_keys(value, AUTHORITY_RECORD_KEYS,
                           f"ownership receipt {path.name} {key}")
        require(value == expectation.ledgers[kind].record(),
                f"ownership receipt {path.name} {key} drift")
    validate_binding_array(receipt["source_review_receipts"],
                           (expectation.source_review_binding,),
                           f"ownership receipt {path.name} source review")
    validate_records(receipt["sources"], expectation.source_records,
                     SOURCE_RECORD_KEYS, f"ownership receipt {path.name} sources")
    validate_records(receipt["targets"], expectation.target_records,
                     TARGET_RECORD_KEYS, f"ownership receipt {path.name} targets")
    allowed_lines = authority_file_lines(authority)
    allowed_lines.update({
        ("source", record["path"]): record["logical_lines"]
        for record in expectation.source_records
    })
    allowed_lines.update({
        ("target", record["path"]): record["logical_lines"]
        for record in expectation.target_records
    })
    for dependency_id in expectation.dependency_components:
        dependency = authority.components[dependency_id]
        allowed_lines.update({
            ("source", record["path"]): record["logical_lines"]
            for record in dependency.source_records
        })
        allowed_lines.update({
            ("target", record["path"]): record["logical_lines"]
            for record in dependency.target_records
        })
    source_review_path = authority.source.repo_path(
        authority.repo, expectation.source_review_binding.path,
        "ownership component source-review citation",
    )
    allowed_lines[("source-review", expectation.source_review_binding.path)] = \
        authority.source.logical_lines(source_review_path)
    component_number = component_id.removeprefix("component-")
    finding_ids = validate_findings(
        receipt["findings"], receipt["open_findings"], allowed_lines,
        set(expectation.authority_keys),
        re.compile(rf"OR-C{re.escape(component_number)}-(?:0[1-9]|[1-9][0-9]+)"),
        f"ownership receipt {path.name}",
    )
    return ComponentResult(component_id, source_component.review_wave,
                           finding_ids, len(receipt["findings"]))


def validate_support_receipt(path: Path,
                             authority: OwnershipAuthority) -> SupportResult:
    expected_path = authority.receipt_directory / "support.ownership-review.toml"
    require(path == expected_path,
            "ownership support receipt filename must be support.ownership-review.toml")
    require(path.is_file() and not path.is_symlink(),
            f"ownership support receipt is not a regular file: {path}")
    require(authority.source.git_tracked(authority.repo, path),
            f"ownership support receipt is not tracked: {path}")
    receipt = load_toml(path, "ownership support receipt")
    require_exact_keys(receipt, SUPPORT_RECEIPT_KEYS, "ownership support receipt")
    expected_scalars = {
        "schema_version": 1,
        "receipt_kind": "ownership-review-support",
        "upstream_ref": EXPECTED_UPSTREAM_REF,
        "workspace_base_ref": EXPECTED_WORKSPACE_BASE_REF,
        "role": authority.source_authority.source_reviewer_role,
        "review_wave": "support",
        "coverage": EXPECTED_COVERAGE,
        "authority_record_count": len(authority.support_authority_keys),
        "authority_sha256": authority.support_authority_sha256,
        "authority_keys": list(authority.support_authority_keys),
        "attestation": "reviewed-complete-support-ownership-lifetime-abi-authority",
    }
    for key, expected in expected_scalars.items():
        require(receipt[key] == expected,
                f"ownership support receipt {key} drift")
    require(authority.source.valid_review_run_id(receipt["review_run_id"]),
            "ownership support receipt has invalid review_run_id")
    validate_binding_array(receipt["source_review_receipts"],
                           (authority.support_binding,),
                           "ownership support source-review binding")
    validate_records(receipt["artifacts"], authority.support_records,
                     SUPPORT_RECORD_KEYS, "ownership support artifacts")
    allowed_lines = authority_file_lines(authority)
    allowed_lines.update({
        ("support", record["path"]): record["logical_lines"]
        for record in authority.support_records
    })
    source_review_path = authority.source.repo_path(
        authority.repo, authority.support_binding.path,
        "ownership support source-review citation",
    )
    allowed_lines[("source-review", authority.support_binding.path)] = \
        authority.source.logical_lines(source_review_path)
    finding_ids = validate_findings(
        receipt["findings"], receipt["open_findings"], allowed_lines,
        set(authority.support_authority_keys),
        re.compile(r"OR-SUP-(?:0[1-9]|[1-9][0-9]+)"),
        "ownership support receipt",
    )
    return SupportResult(finding_ids, len(receipt["findings"]))


def validate_physical_records(value: Any, expected: tuple[dict[str, Any], ...],
                              keys: set[str], label: str) -> None:
    require(isinstance(value, list), f"{label} must be an array")
    for index, record in enumerate(value):
        require(isinstance(record, dict), f"{label} {index} must be a table")
        require_exact_keys(record, keys, f"{label} {index}")
    require(value == list(expected), f"{label} drift")


def overlay_allowed_lines(authority: OwnershipAuthority,
                          expectation: OwnershipOverlayExpectation) -> dict[
                              tuple[str, str], int
                          ]:
    allowed = authority_file_lines(authority)
    allowed.update({
        ("source", record["path"]): record["logical_lines"]
        for record in expectation.source_bindings
    })
    allowed.update({
        ("target", record["path"]): record["logical_lines"]
        for record in expectation.target_bindings
    })
    allowed.update({
        ("support", record["path"]): record["logical_lines"]
        for record in expectation.support_bindings
    })
    # Repo-side generated/adaptation artifacts remain target evidence; pinned
    # upstream external/generated files remain source evidence.
    allowed.update({
        ("target", record["path"]): record["logical_lines"]
        for record in expectation.artifact_bindings
    })
    for tree in expectation.tree_bindings:
        artifact = authority.source_authority.tree_artifacts[tree["path"]]
        for member in artifact.members:
            member_path = authority.source.repo_path(
                authority.repo, member, "ownership overlay tree citation"
            )
            allowed[("target", member)] = authority.source.logical_lines(member_path)
    allowed.update({
        ("source", record["path"]): record["logical_lines"]
        for record in expectation.external_bindings
    })
    allowed.update({
        ("source", record["path"]): record["logical_lines"]
        for record in expectation.generated_bindings
    })
    source_review_path = authority.source.repo_path(
        authority.repo, authority.source_overlay_binding.path,
        "ownership overlay source-review citation",
    )
    allowed[("source-review", authority.source_overlay_binding.path)] = \
        authority.source.logical_lines(source_review_path)
    return allowed


def validate_overlay_receipt(path: Path,
                             authority: OwnershipAuthority) -> OverlayResult:
    expected_path = authority.receipt_directory / "overlays.ownership-review.toml"
    require(path == expected_path,
            "ownership overlay receipt filename must be overlays.ownership-review.toml")
    require(path.is_file() and not path.is_symlink(),
            f"ownership overlay receipt is not a regular file: {path}")
    require(authority.source.git_tracked(authority.repo, path),
            f"ownership overlay receipt is not tracked: {path}")
    receipt = load_toml(path, "ownership overlay receipt")
    require_exact_keys(receipt, OVERLAY_RECEIPT_KEYS, "ownership overlay receipt")
    expected_scalars = {
        "schema_version": 1,
        "receipt_kind": "ownership-review-overlays",
        "upstream_ref": EXPECTED_UPSTREAM_REF,
        "workspace_base_ref": EXPECTED_WORKSPACE_BASE_REF,
        "role": authority.source_authority.source_reviewer_role,
        "review_wave": "overlays",
        "coverage": EXPECTED_OVERLAY_COVERAGE,
    }
    for key, expected in expected_scalars.items():
        require(receipt[key] == expected,
                f"ownership overlay receipt {key} drift")
    require(authority.source.valid_review_run_id(receipt["review_run_id"]),
            "ownership overlay receipt has invalid review_run_id")
    validate_binding_array(receipt["source_review_receipts"],
                           (authority.source_overlay_binding,),
                           "ownership overlay source-review binding")
    records = receipt["overlays"]
    require(isinstance(records, list),
            "ownership overlay records must be an array")
    expected_ids = list(authority.overlays)
    require([record.get("id") for record in records if isinstance(record, dict)]
            == expected_ids,
            "ownership overlay record order or membership drift")
    for index, record in enumerate(records):
        require(isinstance(record, dict),
                f"ownership overlay record {index} must be a table")
        require_exact_keys(record, OVERLAY_RECORD_KEYS,
                           f"ownership overlay record {index}")
        overlay_id = expected_ids[index]
        expectation = authority.overlays[overlay_id]
        expected_record_scalars = {
            "id": overlay_id,
            "ordinal": expectation.ordinal,
            "authority_record_count": len(expectation.authority_keys),
            "authority_sha256": expectation.authority_sha256,
            "component_ids": list(expectation.component_ids),
            "support_paths": list(expectation.support_paths),
            "authority_keys": list(expectation.authority_keys),
            "attestation": "reviewed-complete-derived-ownership-overlay-authority",
        }
        for key, expected in expected_record_scalars.items():
            require(record[key] == expected,
                    f"ownership overlay {overlay_id} {key} drift")
        for key, expected in (
            ("source_bindings", expectation.source_bindings),
            ("target_bindings", expectation.target_bindings),
            ("support_bindings", expectation.support_bindings),
            ("artifact_bindings", expectation.artifact_bindings),
            ("external_bindings", expectation.external_bindings),
            ("generated_bindings", expectation.generated_bindings),
        ):
            validate_physical_records(record[key], expected, FILE_BINDING_KEYS,
                                      f"ownership overlay {overlay_id} {key}")
        validate_physical_records(record["tree_bindings"],
                                  expectation.tree_bindings, TREE_BINDING_KEYS,
                                  f"ownership overlay {overlay_id} tree_bindings")
        component_bindings = tuple(
            ownership_receipt_binding(
                authority,
                component_id,
                canonical_component_receipt_path(authority, component_id),
            )
            for component_id in expectation.component_ids
        )
        validate_binding_array(record["component_receipts"], component_bindings,
                               f"ownership overlay {overlay_id} components")
        support_bindings: tuple[Binding, ...] = ()
        if expectation.support_paths:
            support_bindings = (
                ownership_receipt_binding(
                    authority,
                    "support",
                    authority.receipt_directory / "support.ownership-review.toml",
                ),
            )
        validate_binding_array(record["support_receipts"], support_bindings,
                               f"ownership overlay {overlay_id} support")

    findings = receipt["findings"]
    require(isinstance(findings, list),
            "ownership overlay findings must be an array")
    require(type(receipt["open_findings"]) is int,
            "ownership overlay open_findings must be an integer")
    require(receipt["open_findings"] == len(findings),
            "ownership overlay open_findings count drift")
    finding_ids: list[str] = []
    for overlay_id, expectation in authority.overlays.items():
        overlay_findings = [
            finding for finding in findings
            if isinstance(finding, dict) and finding.get("overlay_id") == overlay_id
        ]
        ordinal = expectation.ordinal
        ids = validate_findings(
            overlay_findings, len(overlay_findings),
            overlay_allowed_lines(authority, expectation),
            set(expectation.authority_keys),
            re.compile(rf"OR-OVL-{ordinal:02}-(?:0[1-9]|[1-9][0-9]+)"),
            f"ownership overlay {overlay_id}", overlay_id,
        )
        finding_ids.extend(ids)
    require(len(finding_ids) == len(findings),
            "ownership overlay finding has an unknown overlay")
    require(len(finding_ids) == len(set(finding_ids)),
            "ownership overlay finding IDs are not unique")
    return OverlayResult(tuple(finding_ids), len(findings))


def subset_tree_sha256(records: Iterable[dict[str, Any]],
                       resolver: Any) -> str:
    hasher = hashlib.sha256()
    ordered = sorted(records, key=lambda record: str(record["path"]))
    for record in ordered:
        relative = str(record["path"])
        path = resolver(relative)
        require(path.is_file(), f"subset-tree member is missing: {relative}")
        hasher.update(relative.encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(path.read_bytes())
        hasher.update(b"\0")
    return hasher.hexdigest()


def validate_plan_file_binding(source: ModuleType, repo: Path,
                               table: dict[str, Any], prefix: str,
                               label: str) -> None:
    expected_keys = {
        f"{prefix}_path", f"{prefix}_sha256", f"{prefix}_logical_lines",
        f"{prefix}_byte_count",
    }
    require(expected_keys <= set(table), f"{label} binding is incomplete")
    relative = str(table[f"{prefix}_path"])
    path = source.repo_path(repo, relative, label)
    require(path.is_file() and source.git_tracked(repo, path),
            f"{label} is missing or untracked: {relative}")
    require(table[f"{prefix}_sha256"] == source.sha256(path),
            f"{label} hash drift")
    require(table[f"{prefix}_logical_lines"] == source.logical_lines(path),
            f"{label} line-count drift")
    require(table[f"{prefix}_byte_count"] == path.stat().st_size,
            f"{label} byte-count drift")


def derive_overlay_dependency_coverage(authority: OwnershipAuthority) -> dict[str, Any]:
    units = authority.source_authority.units
    overlay_component_sets = [
        set(expectation.component_ids) for expectation in authority.overlays.values()
    ]
    source_overlay_authority_keys = set().union(*(
        set(expectation.authority_keys)
        for expectation in authority.source_authority.overlays.values()
    ))
    cross_rows: list[dict[str, str]] = []
    source_covered_rows: list[dict[str, str]] = []
    source_omitted_rows: list[dict[str, str]] = []
    ownership_overlay_authority_keys = set().union(*(
        set(expectation.authority_keys)
        for expectation in authority.overlays.values()
    ))
    ownership_covered_rows: list[dict[str, str]] = []
    ownership_omitted_rows: list[dict[str, str]] = []
    for row in authority.ledgers["dependency"]:
        dependency_unit = row["dependency_unit"]
        if dependency_unit not in units:
            continue
        source_component = units[row["source_unit"]].component_id
        dependency_component = units[dependency_unit].component_id
        if source_component == dependency_component:
            continue
        cross_rows.append(row)
        raw_key = canonical_key("dependency-raw", row)
        source_destination = (
            source_covered_rows
            if raw_key in source_overlay_authority_keys else source_omitted_rows
        )
        source_destination.append(row)
        ownership_destination = (
            ownership_covered_rows
            if raw_key in ownership_overlay_authority_keys else ownership_omitted_rows
        )
        ownership_destination.append(row)
    source_omitted_keys, source_omitted_digest = canonical_digest(
        canonical_key("dependency-raw", row) for row in source_omitted_rows
    )
    ownership_omitted_keys, ownership_omitted_digest = canonical_digest(
        canonical_key("dependency-raw", row) for row in ownership_omitted_rows
    )
    require(len(cross_rows) == 545,
            "overlay coverage cross-row denominator drift")
    require(len(source_covered_rows) == EXPECTED_SOURCE_OVERLAY_COVERED_CROSS_ROWS,
            "source-review overlay covered cross-row denominator drift")
    require(len(source_omitted_keys) == EXPECTED_SOURCE_OVERLAY_OMITTED_CROSS_ROWS,
            "source-review overlay omitted cross-row denominator drift")
    require(source_omitted_digest
            == EXPECTED_SOURCE_OVERLAY_OMITTED_CROSS_ROWS_SHA256,
            "source-review overlay omitted cross-row digest drift")
    require(len(ownership_covered_rows)
            == EXPECTED_OWNERSHIP_OVERLAY_COVERED_CROSS_ROWS,
            "ownership overlay covered cross-row denominator drift")
    require(len(ownership_omitted_keys)
            == EXPECTED_OWNERSHIP_OVERLAY_OMITTED_CROSS_ROWS,
            "ownership overlay omitted cross-row denominator drift")
    require(ownership_omitted_digest == hashlib.sha256(b"").hexdigest(),
            "ownership overlay omitted cross-row digest drift")
    component_pairs = {
        (component_id, dependency_id)
        for component_id, expectation in authority.components.items()
        for dependency_id in expectation.dependency_components
    }
    covered_pairs = {
        pair for pair in component_pairs
        if any(pair[0] in members and pair[1] in members
               for members in overlay_component_sets)
    }
    omitted_pairs = component_pairs - covered_pairs
    require(len(component_pairs) == EXPECTED_COMPONENT_DEPENDENCY_COUNT,
            "overlay coverage component-pair denominator drift")
    require(len(covered_pairs) == EXPECTED_OVERLAY_COVERED_COMPONENT_PAIRS,
            "overlay covered component-pair denominator drift")
    require(omitted_pairs == {("component-084", "component-083")},
            "overlay omitted component-pair identity drift")
    return {
        "cross_rows": cross_rows,
        "source_covered_rows": source_covered_rows,
        "source_omitted_rows": source_omitted_rows,
        "source_omitted_digest": source_omitted_digest,
        "ownership_covered_rows": ownership_covered_rows,
        "ownership_omitted_rows": ownership_omitted_rows,
        "ownership_omitted_digest": ownership_omitted_digest,
        "covered_pairs": covered_pairs,
        "omitted_pairs": omitted_pairs,
    }


def validate_plan(authority: OwnershipAuthority, plan_path: Path) -> None:
    plan = authority.plan
    source = authority.source
    source_authority = authority.source_authority
    require_exact_keys(plan, PLAN_KEYS, "ownership-review plan")
    expected_scalars = {
        "schema_version": 1,
        "upstream_ref": EXPECTED_UPSTREAM_REF,
        "workspace_base_ref": EXPECTED_WORKSPACE_BASE_REF,
        "review_kind": "global-ownership-lifetime-abi",
        "review_mode": "independent-read-only-scc-waves",
        "receipt_directory": EXPECTED_RECEIPT_DIRECTORY,
        "source_review_receipt_directory": EXPECTED_SOURCE_RECEIPT_DIRECTORY,
        "coverage": EXPECTED_COVERAGE,
        "severity_order": EXPECTED_SEVERITIES,
        "finding_id_rules": {
            "component": "OR-CNNN-<positive-decimal-minimum-two-digits>",
            "support": "OR-SUP-<positive-decimal-minimum-two-digits>",
            "overlay": "OR-OVL-NN-<positive-decimal-minimum-two-digits>",
        },
        "structural_sequence": [
            "g0", "g1", "g2", "g3", "g4", "g5", "g6",
            "support", "overlays", "global",
        ],
        "overlay_order": list(source.EXPECTED_OVERLAY_IDS),
    }
    for key, expected in expected_scalars.items():
        require(plan[key] == expected, f"ownership-review plan {key} drift")
    source.require_ancestor(authority.repo, EXPECTED_WORKSPACE_BASE_REF,
                            "ownership-review plan workspace_base_ref")
    require(plan_path.relative_to(authority.repo).as_posix() == EXPECTED_PLAN_PATH,
            "ownership-review plan path drift")

    source_records = tuple(
        record for expectation in authority.components.values()
        for record in expectation.source_records
    )
    target_records = tuple(
        record for expectation in authority.components.values()
        for record in expectation.target_records
    )
    denominator = plan["denominator"]
    require(isinstance(denominator, dict), "ownership denominator must be a table")
    require_exact_keys(denominator, DENOMINATOR_KEYS, "ownership denominator")
    source_findings: list[dict[str, Any]] = []
    for path in expected_source_receipt_paths(source_authority):
        source_findings.extend(load_toml(path, "source-review prerequisite")["findings"])
    severity_counts = {
        severity: sum(finding["severity"] == severity for finding in source_findings)
        for severity in EXPECTED_SEVERITIES
    }
    overlay_dependency_coverage = derive_overlay_dependency_coverage(authority)
    expected_denominator = {
        "ownership_units": EXPECTED_UNIT_COUNT,
        "components": EXPECTED_COMPONENT_COUNT,
        "component_receipts": EXPECTED_COMPONENT_COUNT,
        "support_receipts": 1,
        "overlay_receipts": 1,
        "total_receipts": EXPECTED_COMPONENT_COUNT + 2,
        "source_review_prerequisite_receipts": EXPECTED_COMPONENT_COUNT + 2,
        "source_review_open_findings": len(source_findings),
        "source_review_p0_findings": severity_counts["P0"],
        "source_review_p1_findings": severity_counts["P1"],
        "source_review_p2_findings": severity_counts["P2"],
        "source_review_p3_findings": severity_counts["P3"],
        "sources": len(source_records),
        "targets": len(target_records),
        "support_artifacts": len(authority.support_records),
        "fields": len(authority.ledgers["field"]),
        "lifecycle_events": len(authority.ledgers["lifecycle"]),
        "configurations": len(authority.ledgers["configuration"]),
        "dependency_edges": len(authority.ledgers["dependency"]),
        "owner_contract_families": len(authority.families),
        "field_profiles": len(authority.profiles),
        "owner_family_memberships": len(source_authority.units),
        "component_profile_memberships": sum(
            len(expectation.field_profiles) for expectation in authority.components.values()
        ),
        "declared_dependency_unit_edges": 511,
        "intra_scc_dependency_unit_edges": 59,
        "cross_component_dependency_unit_edges": EXPECTED_UNIT_SEAM_COUNT,
        "known_unit_dependency_pairs": 565,
        "self_unit_dependency_pairs": 54,
        "nonself_unit_dependency_pairs": 511,
        "unique_cross_component_pairs": EXPECTED_COMPONENT_DEPENDENCY_COUNT,
        "cross_component_raw_dependency_rows": 545,
        "source_overlay_covered_cross_component_raw_dependency_rows":
            len(overlay_dependency_coverage["source_covered_rows"]),
        "source_overlay_omitted_cross_component_raw_dependency_rows":
            len(overlay_dependency_coverage["source_omitted_rows"]),
        "ownership_overlay_covered_cross_component_raw_dependency_rows":
            len(overlay_dependency_coverage["ownership_covered_rows"]),
        "ownership_overlay_omitted_cross_component_raw_dependency_rows":
            len(overlay_dependency_coverage["ownership_omitted_rows"]),
        "overlay_covered_cross_component_pairs":
            len(overlay_dependency_coverage["covered_pairs"]),
        "overlay_omitted_cross_component_pairs":
            len(overlay_dependency_coverage["omitted_pairs"]),
        "source_logical_lines": sum(record["logical_lines"] for record in source_records),
        "source_bytes": sum(record["byte_count"] for record in source_records),
        "target_logical_lines": sum(record["logical_lines"] for record in target_records),
        "target_bytes": sum(record["byte_count"] for record in target_records),
        "support_logical_lines": sum(
            record["logical_lines"] for record in authority.support_records
        ),
        "support_bytes": sum(record["byte_count"] for record in authority.support_records),
        "source_review_receipt_logical_lines": EXPECTED_SOURCE_RECEIPT_LINES,
        "source_review_receipt_bytes": EXPECTED_SOURCE_RECEIPT_BYTES,
    }
    require(denominator == expected_denominator,
            "ownership-review plan denominator drift")

    rules = plan["rules"]
    canonicalization = plan["canonicalization"]
    require(isinstance(rules, dict) and set(rules) == {
        "authority", "read_only", "complete_bytes", "complete_state",
        "ledger_scope", "owner_authority", "dependency_closure", "unsafe_boundary",
        "no_phase_jump", "finding_stability", "structural_red", "skill_exclusion",
        "wave_order", "component_atomicity", "overlay_gate", "two_commit_launch",
        "two_commit_close",
        "product_scope",
    }, "ownership-review plan rule set drift")
    require(all(isinstance(value, str) and value for value in rules.values()),
            "ownership-review plan has an empty rule")
    require(isinstance(canonicalization, dict) and set(canonicalization) == {
        "logical_lines", "tsv_row_key", "authority_set_sha256", "file_binding_key",
        "source_review_receipt_binding_key", "scc_partition_key", "owner_family_key",
        "field_profile_key", "dependency_unit_seam_key", "component_dependency_key",
        "component_order_sha256", "tree_sha256", "record_order",
        "empty_authority_sha256",
    }, "ownership-review canonicalization rule set drift")
    require(canonicalization["empty_authority_sha256"]
            == hashlib.sha256(b"").hexdigest(),
            "ownership-review empty authority digest drift")
    launch_contract = plan["launch_contract"]
    require(isinstance(launch_contract, dict),
            "ownership launch contract must be a table")
    require(launch_contract == {
        "launch_ref_field": "ownership_review_launch_ref",
        "pending_ref": "pending",
        "active_queue": "ownership-review",
        "active_status": "active",
        "manifest_path": EXPECTED_MANIFEST_RELATIVE,
        "frozen_paths": list(EXPECTED_LAUNCH_FROZEN_PATHS),
        "makefile_path": EXPECTED_MAKEFILE_RELATIVE,
        "make_tool_assignment": EXPECTED_MAKE_TOOL_ASSIGNMENT,
        "make_admission_target": "backend-port-ownership-review-admission",
        "make_admission_recipe": EXPECTED_MAKE_RECIPES[
            "backend-port-ownership-review-admission"
        ],
        "make_check_target": "backend-port-ownership-review-check",
        "make_check_recipe": EXPECTED_MAKE_RECIPES[
            "backend-port-ownership-review-check"
        ],
        "activation_receipt_files": 0,
        "admission_live_receipt_directory": "absent-or-empty",
        "frozen_byte_layers": ["launch-commit", "HEAD", "index", "worktree"],
        "make_definition_layers": ["launch-commit", "HEAD", "index", "worktree"],
        "manifest_clean_states": ["activated", "complete"],
    }, "ownership launch contract drift")
    completion_contract = plan["completion_contract"]
    require(isinstance(completion_contract, dict),
            "ownership completion contract must be a table")
    require(completion_contract == {
        "active_queue": "ownership-review",
        "active_status": "active",
        "transition_queue": "correction",
        "complete_status": "complete",
        "barrier_ref_field": "ownership_review_barrier_ref",
        "receipt_tree_sha256_field": "ownership_review_receipt_tree_sha256",
        "receipt_logical_lines_field": "ownership_review_receipt_logical_lines",
        "receipt_bytes_field": "ownership_review_receipt_bytes",
        "finding_total_field": "ownership_review_finding_total",
        "finding_p0_field": "ownership_review_p0_findings",
        "finding_p1_field": "ownership_review_p1_findings",
        "finding_p2_field": "ownership_review_p2_findings",
        "finding_p3_field": "ownership_review_p3_findings",
        "finding_id_sha256_field": "ownership_review_finding_id_sha256",
        "close_distinct_from_activation": True,
        "active_phase_changed_paths":
            "exact-117-canonical-added-ownership-receipts",
    }, "ownership completion contract drift")

    plan_authority = plan["authority"]
    require(isinstance(plan_authority, dict), "plan authority must be a table")
    expected_authority_scalars = {
        "scc_partition_records": EXPECTED_COMPONENT_COUNT,
        "scc_partition_sha256": EXPECTED_SCC_PARTITION_SHA256,
        "owner_family_membership_records": EXPECTED_UNIT_COUNT,
        "owner_family_membership_sha256": EXPECTED_OWNER_FAMILY_MEMBERSHIP_SHA256,
        "field_profile_membership_records": 61,
        "field_profile_membership_sha256": EXPECTED_PROFILE_MEMBERSHIP_SHA256,
        "cross_dependency_unit_seam_records": EXPECTED_UNIT_SEAM_COUNT,
        "cross_dependency_unit_seam_sha256": EXPECTED_UNIT_SEAM_SHA256,
        "cross_component_raw_dependency_rows": 545,
        "cross_component_raw_dependency_sha256": "4bfefd8b3b5a4cd8e5634378df27f12d16bb19d591b167855afdeb284cd08534",
        "source_overlay_omitted_cross_component_raw_dependency_rows":
            EXPECTED_SOURCE_OVERLAY_OMITTED_CROSS_ROWS,
        "source_overlay_omitted_cross_component_raw_dependency_sha256":
            EXPECTED_SOURCE_OVERLAY_OMITTED_CROSS_ROWS_SHA256,
        "ownership_overlay_omitted_cross_component_raw_dependency_rows":
            EXPECTED_OWNERSHIP_OVERLAY_OMITTED_CROSS_ROWS,
        "ownership_overlay_omitted_cross_component_raw_dependency_sha256":
            hashlib.sha256(b"").hexdigest(),
        "overlay_omitted_cross_component_pair":
            EXPECTED_OVERLAY_OMITTED_COMPONENT_PAIR,
        "component_authority_records": EXPECTED_COMPONENT_AUTHORITY_COUNT,
        "component_authority_sha256": EXPECTED_COMPONENT_AUTHORITY_SHA256,
        "support_authority_records": EXPECTED_SUPPORT_COUNT + 1,
        "support_authority_sha256": EXPECTED_SUPPORT_AUTHORITY_SHA256,
        "source_review_finding_id_sha256": hashlib.sha256(
            "\n".join(sorted(str(finding["id"]) for finding in source_findings)).encode()
        ).hexdigest(),
    }
    for key, expected in expected_authority_scalars.items():
        require(plan_authority.get(key) == expected,
                f"ownership plan authority drift: {key}")
    cross_rows = [
        row for row in authority.ledgers["dependency"]
        if row["dependency_unit"] in source_authority.units
        and source_authority.units[row["source_unit"]].component_id
        != source_authority.units[row["dependency_unit"]].component_id
    ]
    cross_keys, cross_digest = canonical_digest(
        canonical_key("dependency-raw", row) for row in cross_rows
    )
    require(len(cross_keys) == plan_authority["cross_component_raw_dependency_rows"]
            and cross_digest == plan_authority["cross_component_raw_dependency_sha256"],
            "cross-component raw dependency authority drift")
    for prefix, label in (
        ("source_ownership", "source ownership authority"),
        ("ownership_order", "ownership order authority"),
        ("source_review_plan", "source-review plan authority"),
        ("source_review_schema", "source-review schema authority"),
        ("source_review_support", "source-review support authority"),
    ):
        validate_plan_file_binding(source, authority.repo, plan_authority, prefix, label)
    expected_authority_keys = set(expected_authority_scalars) | {
        f"{prefix}_{suffix}"
        for prefix in (
            "source_ownership", "ownership_order", "source_review_plan",
            "source_review_schema", "source_review_support",
        )
        for suffix in ("path", "sha256", "logical_lines", "byte_count")
    }
    require_exact_keys(plan_authority, expected_authority_keys, "plan authority")

    validate_plan_byte_authority(authority, source_records, target_records)
    validate_plan_ledger_authority(authority)
    validate_plan_contract_authority(authority)
    validate_plan_waves(authority)
    validate_plan_overlays(authority)
    validate_plan_prerequisites(authority)


def validate_plan_byte_authority(
    authority: OwnershipAuthority,
    source_records: tuple[dict[str, Any], ...],
    target_records: tuple[dict[str, Any], ...],
) -> None:
    table = authority.plan["byte_authority"]
    require(isinstance(table, dict), "byte authority must be a table")
    require_exact_keys(table, {"sources", "targets", "support", "source_review_receipts"},
                       "byte authority")
    categories = {
        "sources": (
            source_records,
            "source-binding",
            EXPECTED_SOURCE_BINDING_SHA256,
            lambda path: authority.source.upstream_path(
                authority.upstream, path, "source byte authority"
            ),
        ),
        "targets": (
            target_records,
            "target-binding",
            EXPECTED_TARGET_BINDING_SHA256,
            lambda path: authority.source.repo_path(
                authority.repo, path, "target byte authority"
            ),
        ),
        "support": (
            authority.support_records,
            "support-binding",
            EXPECTED_SUPPORT_BINDING_SHA256,
            lambda path: authority.source.repo_path(
                authority.repo, path, "support byte authority"
            ),
        ),
    }
    common_keys = {
        "record_count", "logical_lines", "byte_count", "binding_prefix",
        "binding_sha256", "subset_tree_sha256",
    }
    for category, (records, prefix, digest, resolver) in categories.items():
        value = table[category]
        require(isinstance(value, dict), f"{category} byte authority must be a table")
        require_exact_keys(value, common_keys, f"{category} byte authority")
        expected = {
            "record_count": len(records),
            "logical_lines": sum(record["logical_lines"] for record in records),
            "byte_count": sum(record["byte_count"] for record in records),
            "binding_prefix": prefix,
            "binding_sha256": digest,
            "subset_tree_sha256": subset_tree_sha256(records, resolver),
        }
        require(value == expected, f"{category} byte authority drift")
    receipts = table["source_review_receipts"]
    require(isinstance(receipts, dict),
            "source-review receipt byte authority must be a table")
    require_exact_keys(receipts, {
        "record_count", "logical_lines", "byte_count", "binding_prefix",
        "binding_sha256", "tree_sha256",
    }, "source-review receipt byte authority")
    prerequisite_bindings = [
        Binding(str(record["id"]), str(record["path"]), str(record["sha256"]),
                int(record["byte_count"]))
        for record in authority.plan["prerequisite_receipt"]
    ]
    binding_keys, binding_digest = canonical_digest(
        binding_key("source-review-receipt-binding", binding)
        for binding in prerequisite_bindings
    )
    expected_receipts = {
        "record_count": len(prerequisite_bindings),
        "logical_lines": EXPECTED_SOURCE_RECEIPT_LINES,
        "byte_count": EXPECTED_SOURCE_RECEIPT_BYTES,
        "binding_prefix": "source-review-receipt-binding",
        "binding_sha256": binding_digest,
        "tree_sha256": EXPECTED_SOURCE_RECEIPT_TREE_SHA256,
    }
    require(len(binding_keys) == EXPECTED_COMPONENT_COUNT + 2,
            "source-review binding denominator drift")
    require(receipts == expected_receipts,
            "source-review receipt byte authority drift")


def validate_plan_ledger_authority(authority: OwnershipAuthority) -> None:
    table = authority.plan["ledger_authority"]
    require(isinstance(table, dict), "ledger authority must be a table")
    mapping = {
        "fields": "field",
        "lifecycle_events": "lifecycle",
        "configurations": "configuration",
        "dependency_edges": "dependency",
    }
    require_exact_keys(table, set(mapping), "ledger authority")
    expected_keys = {
        "path", "row_count", "logical_lines", "byte_count", "file_sha256",
        "authority_sha256", "typed_prefix",
    }
    for plan_id, kind in mapping.items():
        value = table[plan_id]
        require(isinstance(value, dict), f"{plan_id} ledger authority must be a table")
        require_exact_keys(value, expected_keys, f"{plan_id} ledger authority")
        prefix, manifest_key, _ = LEDGER_SPECS[kind]
        relative = str(authority.source_authority.manifest[manifest_key])
        path = authority.source.repo_path(authority.repo, relative,
                                          f"{plan_id} ledger authority")
        expected = {
            "path": relative,
            "row_count": len(authority.ledgers[kind]),
            "logical_lines": authority.source.logical_lines(path),
            "byte_count": path.stat().st_size,
            "file_sha256": authority.source.sha256(path),
            "authority_sha256": EXPECTED_LEDGER_DIGESTS[kind],
            "typed_prefix": prefix,
        }
        require(value == expected, f"{plan_id} ledger authority drift")


def validate_plan_contract_authority(authority: OwnershipAuthority) -> None:
    table = authority.plan["contract_authority"]
    require(isinstance(table, dict), "contract authority must be a table")
    require_exact_keys(table, {"owner_contracts", "field_profiles"},
                       "contract authority")
    specifications = {
        "owner_contracts": (
            "owner_contracts",
            "family_ids",
            [family["id"] for family in authority.families],
        ),
        "field_profiles": (
            "field_profiles",
            "profile_ids",
            [profile["id"] for profile in authority.profiles],
        ),
    }
    for table_id, (manifest_key, ids_key, ids) in specifications.items():
        value = table[table_id]
        require(isinstance(value, dict), f"{table_id} contract must be a table")
        require_exact_keys(value, {
            "path", "logical_lines", "byte_count", "file_sha256", ids_key,
        }, f"{table_id} contract")
        relative = str(authority.source_authority.manifest[manifest_key])
        path = authority.source.repo_path(authority.repo, relative,
                                          f"{table_id} contract")
        expected = {
            "path": relative,
            "logical_lines": authority.source.logical_lines(path),
            "byte_count": path.stat().st_size,
            "file_sha256": authority.source.sha256(path),
            ids_key: ids,
        }
        require(value == expected, f"{table_id} contract authority drift")


def validate_plan_waves(authority: OwnershipAuthority) -> None:
    waves = authority.plan["wave"]
    require(isinstance(waves, list) and len(waves) == 7,
            "ownership-review plan must define seven waves")
    require([wave.get("id") for wave in waves if isinstance(wave, dict)]
            == [f"g{index}" for index in range(7)],
            "ownership-review wave order drift")
    for order_group, wave in enumerate(waves):
        require(isinstance(wave, dict), f"ownership wave g{order_group} must be a table")
        require_exact_keys(wave, WAVE_PLAN_KEYS, f"ownership wave g{order_group}")
        component_ids = [
            component.component_id
            for component in authority.source_authority.components.values()
            if component.order_group == order_group
        ]
        components = [authority.source_authority.components[item] for item in component_ids]
        expectations = [authority.components[item] for item in component_ids]
        units = [unit_id for component in components for unit_id in component.units]
        sources = [record for item in expectations for record in item.source_records]
        targets = [record for item in expectations for record in item.target_records]
        expected = {
            "id": f"g{order_group}",
            "order_group": order_group,
            "component_count": len(component_ids),
            "unit_count": len(units),
            "source_count": len(sources),
            "target_count": len(targets),
            "field_rows": sum(len(item.ledgers["field"].keys) for item in expectations),
            "lifecycle_rows": sum(
                len(item.ledgers["lifecycle"].keys) for item in expectations
            ),
            "configuration_rows": sum(
                len(item.ledgers["configuration"].keys) for item in expectations
            ),
            "dependency_rows": sum(
                len(item.ledgers["dependency"].keys) for item in expectations
            ),
            "owner_family_memberships": len(units),
            "field_profile_memberships": sum(
                len(item.field_profiles) for item in expectations
            ),
            "cross_dependency_unit_seams": sum(
                1 for unit_id in units
                for dependency in authority.source_authority.units[unit_id].dependency_units
                if authority.source_authority.units[dependency].component_id
                != authority.source_authority.units[unit_id].component_id
            ),
            "dependency_component_pairs": sum(
                len(item.dependency_components) for item in expectations
            ),
            "component_order_sha256": hashlib.sha256(
                "\n".join(component_ids).encode("utf-8")
            ).hexdigest(),
            "component_ids": component_ids,
        }
        require(wave == expected, f"ownership wave g{order_group} authority drift")


def physical_binding_summary(category: str,
                             records: tuple[dict[str, Any], ...]) -> tuple[int, int, int, str]:
    keys, digest = canonical_digest(
        record_key(f"physical-{category}", record) for record in records
    )
    return (
        len(keys),
        sum(int(record["logical_lines"]) for record in records),
        sum(int(record["byte_count"]) for record in records),
        digest,
    )


def validate_plan_overlays(authority: OwnershipAuthority) -> None:
    tables = authority.plan["overlay"]
    require(isinstance(tables, list) and len(tables) == EXPECTED_OVERLAY_COUNT,
            "ownership-review overlay plan denominator drift")
    expected_ids = list(authority.overlays)
    require([table.get("id") for table in tables if isinstance(table, dict)]
            == expected_ids, "ownership-review overlay plan order drift")
    for index, table in enumerate(tables):
        overlay_id = expected_ids[index]
        require(isinstance(table, dict), f"ownership overlay plan {overlay_id} must be a table")
        require_exact_keys(table, OVERLAY_PLAN_KEYS,
                           f"ownership overlay plan {overlay_id}")
        expectation = authority.overlays[overlay_id]
        source_overlay = authority.source_authority.overlays[overlay_id]
        component_expectations = [
            authority.components[component_id] for component_id in expectation.component_ids
        ]
        component_units = [
            unit_id for component_id in expectation.component_ids
            for unit_id in authority.source_authority.components[component_id].units
        ]
        expected = {
            "ordinal": expectation.ordinal,
            "id": overlay_id,
            "rule": table["rule"],
            "component_count": len(expectation.component_ids),
            "support_count": len(expectation.support_paths),
            "field_rows": sum(len(item.ledgers["field"].keys)
                              for item in component_expectations),
            "lifecycle_rows": sum(len(item.ledgers["lifecycle"].keys)
                                  for item in component_expectations),
            "configuration_rows": sum(len(item.ledgers["configuration"].keys)
                                      for item in component_expectations),
            "dependency_rows": sum(len(item.ledgers["dependency"].keys)
                                   for item in component_expectations),
            "owner_family_memberships": len(component_units),
            "field_profile_memberships": sum(
                len(item.field_profiles) for item in component_expectations
            ),
            "source_review_authority_record_count": source_overlay.authority_record_count,
            "source_review_authority_sha256": source_overlay.authority_sha256,
            "ownership_authority_record_count": len(expectation.authority_keys),
            "ownership_authority_sha256": expectation.authority_sha256,
        }
        categories = {
            "source": expectation.source_bindings,
            "target": expectation.target_bindings,
            "support": expectation.support_bindings,
            "artifact": expectation.artifact_bindings,
            "external": expectation.external_bindings,
            "generated": expectation.generated_bindings,
        }
        for category, records in categories.items():
            count, lines, byte_count, digest = physical_binding_summary(category, records)
            expected.update({
                f"{category}_binding_count": count,
                f"{category}_logical_lines": lines,
                f"{category}_byte_count": byte_count,
                f"{category}_binding_sha256": digest,
            })
        tree_count, tree_lines, tree_bytes, tree_digest_value = \
            physical_binding_summary("tree", expectation.tree_bindings)
        expected.update({
            "tree_binding_count": tree_count,
            "tree_member_count": sum(
                int(record["file_count"]) for record in expectation.tree_bindings
            ),
            "tree_logical_lines": tree_lines,
            "tree_byte_count": tree_bytes,
            "tree_binding_sha256": tree_digest_value,
        })
        require(table == expected,
                f"ownership overlay plan authority drift: {overlay_id}")


def validate_plan_prerequisites(authority: OwnershipAuthority) -> None:
    records = authority.plan["prerequisite_receipt"]
    require(isinstance(records, list),
            "ownership prerequisite receipts must be an array")
    for index, record in enumerate(records):
        require(isinstance(record, dict),
                f"ownership prerequisite receipt {index} must be a table")
        require_exact_keys(record, BINDING_KEYS,
                           f"ownership prerequisite receipt {index}")
    expected: list[dict[str, Any]] = []
    for path in sorted(expected_source_receipt_paths(authority.source_authority),
                       key=lambda item: item.relative_to(authority.repo).as_posix()):
        filename = path.name
        if filename == "support.source-review.toml":
            binding_id = "support"
        elif filename == "overlays.source-review.toml":
            binding_id = "overlays"
        else:
            binding_id = filename.removesuffix(".source-review.toml")
        expected.append(Binding(
            binding_id,
            path.relative_to(authority.repo).as_posix(),
            authority.source.sha256(path),
            path.stat().st_size,
        ).record())
    require(records == expected,
            "ownership prerequisite receipt bindings drift")


def git_object_bytes(repo: Path, revision: str, relative: str,
                     label: str) -> bytes:
    result = subprocess.run(
        ["git", "-C", str(repo), "show", f"{revision}:{relative}"],
        capture_output=True,
    )
    require(result.returncode == 0,
            f"{label} is absent from {revision}: {relative}")
    return result.stdout


def require_git_regular_blob(repo: Path, revision: str, relative: str,
                             label: str) -> None:
    result = subprocess.run(
        ["git", "-C", str(repo), "ls-tree", revision, "--", relative],
        capture_output=True,
        text=True,
    )
    require(result.returncode == 0, f"cannot inspect {label}: {relative}")
    records = result.stdout.splitlines()
    require(len(records) == 1, f"{label} is absent or ambiguous: {relative}")
    metadata, separator, path = records[0].partition("\t")
    fields = metadata.split()
    require(separator == "\t" and path == relative and len(fields) == 3
            and fields[0] in {"100644", "100755"} and fields[1] == "blob",
            f"{label} is not a regular Git blob: {relative}")


def git_index_bytes(repo: Path, relative: str, label: str) -> bytes:
    stages = subprocess.run(
        ["git", "-C", str(repo), "ls-files", "--stage", "--", relative],
        capture_output=True,
        text=True,
    )
    require(stages.returncode == 0, f"cannot inspect {label} index: {relative}")
    records = stages.stdout.splitlines()
    require(len(records) == 1, f"{label} index is absent or conflicted: {relative}")
    metadata, separator, path = records[0].partition("\t")
    fields = metadata.split()
    require(separator == "\t" and path == relative and len(fields) == 3
            and fields[0] in {"100644", "100755"} and fields[2] == "0",
            f"{label} index is not a regular stage-zero blob: {relative}")
    result = subprocess.run(
        ["git", "-C", str(repo), "show", f":{relative}"],
        capture_output=True,
    )
    require(result.returncode == 0, f"cannot read {label} index: {relative}")
    return result.stdout


def require_worktree_and_index_clean(repo: Path, relative: str,
                                     label: str) -> None:
    commands = (
        ["git", "-C", str(repo), "diff", "--quiet", "HEAD", "--", relative],
        [
            "git", "-C", str(repo), "diff", "--quiet", "--cached", "HEAD",
            "--", relative,
        ],
    )
    for command in commands:
        result = subprocess.run(command, capture_output=True)
        require(result.returncode == 0,
                f"{label} index or worktree differs from HEAD: {relative}")


def require_full_ancestor(repo: Path, revision: Any, label: str) -> str:
    require(isinstance(revision, str)
            and re.fullmatch(r"[0-9a-f]{40}", revision) is not None,
            f"{label} must be a full Git revision")
    commit = subprocess.run(
        ["git", "-C", str(repo), "cat-file", "-e", f"{revision}^{{commit}}"],
        capture_output=True,
    )
    require(commit.returncode == 0, f"{label} is not a repository commit")
    ancestor = subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", revision, "HEAD"],
        capture_output=True,
    )
    require(ancestor.returncode == 0, f"{label} is not an ancestor of current HEAD")
    return revision


def load_toml_bytes(raw: bytes, label: str) -> dict[str, Any]:
    try:
        value = tomllib.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise OwnershipReviewError(f"cannot parse {label}: {error}") from error
    require(isinstance(value, dict), f"{label} is not a TOML table")
    return value


def git_direct_transition_child(repo: Path, revision: str, label: str) -> str:
    result = subprocess.run(
        [
            "git", "-C", str(repo), "rev-list", "--ancestry-path", "--reverse",
            f"{revision}..HEAD",
        ],
        capture_output=True,
        text=True,
    )
    require(result.returncode == 0, f"cannot resolve {label} transition")
    candidates: list[str] = []
    for commit in result.stdout.splitlines():
        parents = subprocess.run(
            ["git", "-C", str(repo), "show", "-s", "--format=%P", commit],
            capture_output=True,
            text=True,
        )
        require(parents.returncode == 0, f"cannot inspect {label} transition")
        if parents.stdout.strip().split() == [revision]:
            candidates.append(commit)
    require(len(candidates) == 1,
            f"{label} must have exactly one reachable direct transition child")
    return candidates[0]


def require_direct_child(repo: Path, parent: str, child: str, label: str) -> None:
    result = subprocess.run(
        ["git", "-C", str(repo), "show", "-s", "--format=%P", child],
        capture_output=True,
        text=True,
    )
    require(result.returncode == 0, f"cannot inspect {label} parents")
    require(result.stdout.strip().split() == [parent],
            f"{label} must be a single-parent direct child of its barrier")


def require_manifest_only_commit(repo: Path, parent: str, child: str,
                                 label: str) -> None:
    result = subprocess.run(
        ["git", "-C", str(repo), "diff", "--name-only", parent, child],
        capture_output=True,
        text=True,
    )
    require(result.returncode == 0, f"cannot inspect {label} paths")
    require(result.stdout.splitlines() == [EXPECTED_MANIFEST_RELATIVE],
            f"{label} must change only {EXPECTED_MANIFEST_RELATIVE}")


def validate_make_integration(raw: bytes, label: str) -> None:
    try:
        lines = raw.decode("utf-8").splitlines()
    except UnicodeDecodeError as error:
        raise OwnershipReviewError(f"{label} is not UTF-8: {error}") from error
    variable = "BACKEND_PORT_OWNERSHIP_REVIEW_TOOL"
    assignment_pattern = re.compile(
        rf"(?<![A-Za-z0-9_]){variable}\s*(?:\?=|::=|:=|\+=|!=|=)"
    )
    directive_pattern = re.compile(
        rf"^\s*(?:(?:override|export|private)\s+)*"
        rf"(?:define|undefine|unexport|export)\s+{variable}(?:\s|$)"
    )
    assignments = [
        line for line in lines
        if assignment_pattern.search(line) or directive_pattern.match(line)
    ]
    require(assignments == [EXPECTED_MAKE_TOOL_ASSIGNMENT],
            f"{label} ownership checker assignment drift")
    phony_targets = {
        target
        for line in lines if line.startswith(".PHONY:")
        for target in line.removeprefix(".PHONY:").split()
    }
    for target, recipe in EXPECTED_MAKE_RECIPES.items():
        require(target in phony_targets,
                f"{label} ownership target is not phony: {target}")
        target_line = f"{target}:"
        target_pattern = re.compile(
            rf"^(?!\t)\s*(?:[^\s:#=]+\s+)*{re.escape(target)}"
            rf"(?:\s+[^\s:#=]+)*\s*::?"
        )
        target_definitions = [line for line in lines if target_pattern.match(line)]
        require(target_definitions == [target_line],
                f"{label} ownership target drift: {target}")
        index = lines.index(target_line)
        require(index + 1 < len(lines) and lines[index + 1] == f"\t{recipe}",
                f"{label} ownership recipe drift: {target}")
        require(index + 2 == len(lines) or not lines[index + 2].startswith("\t"),
                f"{label} ownership target has an extra recipe: {target}")


def validate_launch_frozen_bytes(repo: Path, launch_ref: str) -> None:
    for relative in EXPECTED_LAUNCH_FROZEN_PATHS:
        current_path = repo / relative
        require(current_path.is_file() and not current_path.is_symlink(),
                f"ownership-review launch authority is missing or untracked: {relative}")
        require_git_regular_blob(
            repo, launch_ref, relative, "ownership-review launch authority"
        )
        require_git_regular_blob(
            repo, "HEAD", relative, "ownership-review HEAD authority"
        )
        index_bytes = git_index_bytes(
            repo, relative, "ownership-review launch authority"
        )
        require_worktree_and_index_clean(
            repo, relative, "ownership-review launch authority"
        )
        head_diff = subprocess.run(
            [
                "git", "-C", str(repo), "diff", "--quiet", launch_ref, "HEAD",
                "--", relative,
            ],
            capture_output=True,
        )
        require(head_diff.returncode == 0,
                f"ownership-review HEAD drifted from launch: {relative}")
        launch_bytes = git_object_bytes(
            repo, launch_ref, relative, "ownership-review launch authority"
        )
        head_bytes = git_object_bytes(
            repo, "HEAD", relative, "ownership-review HEAD authority"
        )
        require(launch_bytes == head_bytes == index_bytes
                == current_path.read_bytes(),
                f"ownership-review launch authority drift: {relative}")
    current_makefile = repo / EXPECTED_MAKEFILE_RELATIVE
    require(current_makefile.is_file() and not current_makefile.is_symlink(),
            "ownership-review Make integration is missing or untracked")
    require_git_regular_blob(
        repo, launch_ref, EXPECTED_MAKEFILE_RELATIVE,
        "ownership-review launch Makefile",
    )
    require_git_regular_blob(
        repo, "HEAD", EXPECTED_MAKEFILE_RELATIVE,
        "ownership-review HEAD Makefile",
    )
    validate_make_integration(current_makefile.read_bytes(), "current Makefile")
    validate_make_integration(
        git_object_bytes(
            repo, launch_ref, EXPECTED_MAKEFILE_RELATIVE,
            "ownership-review launch Makefile",
        ),
        "launch Makefile",
    )
    validate_make_integration(
        git_object_bytes(
            repo, "HEAD", EXPECTED_MAKEFILE_RELATIVE,
            "ownership-review HEAD Makefile",
        ),
        "HEAD Makefile",
    )
    validate_make_integration(
        git_index_bytes(
            repo, EXPECTED_MAKEFILE_RELATIVE,
            "ownership-review Makefile",
        ),
        "index Makefile",
    )


def validate_launch_state(authority: OwnershipAuthority,
                          manifest_path: Path) -> None:
    manifest = authority.source_authority.manifest
    launch_ref = manifest.get("ownership_review_launch_ref")
    require(launch_ref != "pending",
            "ownership-review launch is pending activation")
    require(isinstance(launch_ref, str)
            and re.fullmatch(r"[0-9a-f]{40}", launch_ref) is not None,
            "ownership_review_launch_ref must be a full Git revision")
    authority.source.require_ancestor(
        authority.repo, launch_ref, "ownership-review launch ref"
    )
    require(launch_ref != authority.source.git_head(authority.repo),
            "ownership-review activation commit does not exist")
    require(manifest_path.relative_to(authority.repo).as_posix()
            == EXPECTED_MANIFEST_RELATIVE,
            "ownership-review manifest path drift")

    launch_manifest = load_toml_bytes(
        git_object_bytes(
            authority.repo, launch_ref, EXPECTED_MANIFEST_RELATIVE,
            "ownership-review launch manifest",
        ),
        "ownership-review launch manifest",
    )
    require(launch_manifest.get("active_queue") == "ownership-review"
            and launch_manifest.get("ownership_review_status") == "active",
            "ownership-review launch manifest is not active")
    require(launch_manifest.get("ownership_review_launch_ref") == "pending",
            "ownership-review launch manifest lacks the pending sentinel")
    require(not (set(launch_manifest) & COMPLETION_PIN_KEYS),
            "ownership-review launch manifest contains completion pins")
    for key, expected in EXPECTED_MANIFEST_PATHS.items():
        require(launch_manifest.get(key) == expected,
                f"ownership-review launch manifest field drift: {key}")

    validate_launch_frozen_bytes(authority.repo, launch_ref)

    activation_ref = git_direct_transition_child(
        authority.repo, launch_ref, "ownership-review activation"
    )
    require_direct_child(
        authority.repo, launch_ref, activation_ref, "ownership-review activation"
    )
    require_manifest_only_commit(
        authority.repo, launch_ref, activation_ref, "ownership-review activation"
    )
    activation_manifest = load_toml_bytes(
        git_object_bytes(
            authority.repo, activation_ref, EXPECTED_MANIFEST_RELATIVE,
            "ownership-review activation manifest",
        ),
        "ownership-review activation manifest",
    )
    require(set(activation_manifest) == set(launch_manifest),
            "ownership-review activation manifest key set drift")
    require(activation_manifest.get("ownership_review_launch_ref") == launch_ref,
            "ownership-review activation does not pin its launch commit")
    for key in launch_manifest:
        if key != "ownership_review_launch_ref":
            require(activation_manifest[key] == launch_manifest[key],
                    f"ownership-review activation changes another field: {key}")
    activation_receipts = subprocess.run(
        [
            "git", "-C", str(authority.repo), "ls-tree", "-r", "--name-only",
            activation_ref, "--", EXPECTED_RECEIPT_DIRECTORY,
        ],
        capture_output=True,
        text=True,
    )
    require(activation_receipts.returncode == 0,
            "cannot inspect ownership-review activation receipt tree")
    require(not activation_receipts.stdout.splitlines(),
            "ownership-review activation commit contains preloaded receipts")
    require(manifest.get("ownership_review_launch_ref") == launch_ref,
            "ownership-review launch ref changed after activation")


def expected_ownership_receipt_relatives(
    authority: OwnershipAuthority,
) -> tuple[str, ...]:
    names = [
        f"{component_id}.ownership-review.toml"
        for component_id in authority.components
    ]
    names.extend(["support.ownership-review.toml", "overlays.ownership-review.toml"])
    return tuple(sorted(names))


def expected_ownership_receipt_repo_paths(
    authority: OwnershipAuthority,
) -> tuple[str, ...]:
    return tuple(
        f"{EXPECTED_RECEIPT_DIRECTORY}/{name}"
        for name in expected_ownership_receipt_relatives(authority)
    )


def validate_review_phase_commit_scope(
    authority: OwnershipAuthority,
    repo: Path,
    close_ref: str,
) -> None:
    launch_ref = authority.source_authority.manifest["ownership_review_launch_ref"]
    activation_ref = git_direct_transition_child(
        repo, launch_ref, "ownership-review activation"
    )
    require(close_ref != activation_ref,
            "ownership-review close commit must be distinct from activation")
    authority.source.require_ancestor(
        repo, activation_ref, "ownership-review activation commit"
    )
    close_ancestor = subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", activation_ref,
         close_ref],
        capture_output=True,
    )
    require(close_ancestor.returncode == 0,
            "ownership-review close does not descend from activation")
    changed = subprocess.run(
        [
            "git", "-C", str(repo), "diff", "--name-status", "--no-renames",
            activation_ref, close_ref,
        ],
        capture_output=True,
        text=True,
    )
    require(changed.returncode == 0,
            "cannot inspect ownership-review active-phase commit scope")
    records: list[tuple[str, str]] = []
    for line in changed.stdout.splitlines():
        status, separator, path = line.partition("\t")
        require(separator == "\t",
                "ownership-review active-phase diff is malformed")
        records.append((status, path))
    expected_paths = expected_ownership_receipt_repo_paths(authority)
    require(tuple(sorted(records))
            == tuple(("A", path) for path in expected_paths),
            "ownership-review B-to-C diff is not the exact 117 added receipts")


def receipt_closure(raw_by_name: dict[str, bytes]) -> dict[str, Any]:
    hasher = hashlib.sha256()
    logical_line_count = 0
    byte_count = 0
    finding_ids: list[str] = []
    severity_counts = {severity: 0 for severity in EXPECTED_SEVERITIES}
    for name in sorted(raw_by_name):
        raw = raw_by_name[name]
        hasher.update(name.encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(raw)
        hasher.update(b"\0")
        logical_line_count += len(raw.splitlines())
        byte_count += len(raw)
        receipt = load_toml_bytes(raw, f"ownership barrier receipt {name}")
        findings = receipt.get("findings")
        require(isinstance(findings, list),
                f"ownership barrier receipt findings drift: {name}")
        for finding in findings:
            require(isinstance(finding, dict),
                    f"ownership barrier finding is not a table: {name}")
            finding_id = finding.get("id")
            severity = finding.get("severity")
            require(isinstance(finding_id, str),
                    f"ownership barrier finding ID drift: {name}")
            require(severity in EXPECTED_SEVERITIES,
                    f"ownership barrier finding severity drift: {finding_id}")
            finding_ids.append(finding_id)
            severity_counts[severity] += 1
    require(len(finding_ids) == len(set(finding_ids)),
            "ownership barrier finding IDs are not globally unique")
    finding_id_sha256 = hashlib.sha256(
        "\n".join(sorted(finding_ids)).encode("utf-8")
    ).hexdigest()
    return {
        "tree_sha256": hasher.hexdigest(),
        "logical_lines": logical_line_count,
        "bytes": byte_count,
        "finding_total": len(finding_ids),
        "severity_counts": severity_counts,
        "finding_id_sha256": finding_id_sha256,
    }


def validate_completion_state(
    authority: OwnershipAuthority,
    live_repo: Path | None = None,
    live_manifest: dict[str, Any] | None = None,
) -> None:
    repo = authority.repo if live_repo is None else live_repo
    manifest = (authority.source_authority.manifest
                if live_manifest is None else live_manifest)
    queue = manifest["active_queue"]
    status = manifest["ownership_review_status"]
    present_pins = set(manifest) & COMPLETION_PIN_KEYS
    if queue == "ownership-review":
        require(status == "active", "ownership-review queue requires active status")
        require(not present_pins,
                "active ownership review must not contain completion pins")
        return

    require(status == "complete", "later queue requires complete ownership status")
    require(present_pins == COMPLETION_PIN_KEYS,
            "complete ownership review has an incomplete completion pin set")
    barrier_ref = manifest["ownership_review_barrier_ref"]
    require(isinstance(barrier_ref, str)
            and re.fullmatch(r"[0-9a-f]{40}", barrier_ref) is not None,
            "ownership_review_barrier_ref must be a full Git revision")
    authority.source.require_ancestor(
        repo, barrier_ref, "ownership-review barrier ref"
    )
    require(barrier_ref != authority.source.git_head(repo),
            "ownership-review transition commit does not exist")
    launch_ref = manifest.get("ownership_review_launch_ref")
    require(isinstance(launch_ref, str)
            and re.fullmatch(r"[0-9a-f]{40}", launch_ref) is not None,
            "ownership_review_launch_ref must be a full Git revision")
    validate_launch_frozen_bytes(repo, launch_ref)
    barrier_manifest = load_toml_bytes(
        git_object_bytes(
            repo, barrier_ref, EXPECTED_MANIFEST_RELATIVE,
            "ownership-review barrier manifest",
        ),
        "ownership-review barrier manifest",
    )
    require(barrier_manifest.get("active_queue") == "ownership-review"
            and barrier_manifest.get("ownership_review_status") == "active",
            "ownership-review barrier manifest is not active")
    require(not (set(barrier_manifest) & COMPLETION_PIN_KEYS),
            "ownership-review barrier manifest contains completion pins")
    require(barrier_manifest.get("ownership_review_launch_ref")
            == manifest.get("ownership_review_launch_ref"),
            "ownership-review barrier launch ref drift")
    for key, expected in EXPECTED_MANIFEST_PATHS.items():
        require(barrier_manifest.get(key) == expected,
                f"ownership-review barrier manifest field drift: {key}")
    validate_review_phase_commit_scope(authority, repo, barrier_ref)

    expected_names = expected_ownership_receipt_relatives(authority)
    tree_result = subprocess.run(
        [
            "git", "-C", str(repo), "ls-tree", "-r",
            barrier_ref, "--", EXPECTED_RECEIPT_DIRECTORY,
        ],
        capture_output=True,
        text=True,
    )
    require(tree_result.returncode == 0,
            "cannot enumerate ownership-review barrier receipts")
    prefix = f"{EXPECTED_RECEIPT_DIRECTORY}/"
    barrier_names_list: list[str] = []
    for line in tree_result.stdout.splitlines():
        metadata, separator, relative = line.partition("\t")
        fields = metadata.split()
        require(separator == "\t" and len(fields) == 3
                and fields[0] in {"100644", "100755"} and fields[1] == "blob",
                "ownership-review barrier receipt is not a regular Git blob")
        require(relative.startswith(prefix),
                "ownership-review barrier receipt escaped its directory")
        barrier_names_list.append(relative.removeprefix(prefix))
    barrier_names = tuple(sorted(barrier_names_list))
    require(barrier_names == expected_names,
            "ownership-review barrier receipt set drift")
    barrier_raw = {
        name: git_object_bytes(
            repo, barrier_ref, f"{EXPECTED_RECEIPT_DIRECTORY}/{name}",
            "ownership-review barrier receipt",
        )
        for name in expected_names
    }
    current_receipt_directory = repo / EXPECTED_RECEIPT_DIRECTORY
    current_entries = (list(current_receipt_directory.rglob("*"))
                       if current_receipt_directory.is_dir() else [])
    require(all(path.parent == current_receipt_directory
                and path.is_file() and not path.is_symlink()
                for path in current_entries),
            "current ownership-review receipt directory contains a non-regular entry")
    current_names = tuple(sorted(
        path.relative_to(current_receipt_directory).as_posix()
        for path in current_entries
    ))
    require(current_names == expected_names,
            "current ownership-review receipt set differs from barrier")
    for name, raw in barrier_raw.items():
        live_path = current_receipt_directory / name
        require(live_path.is_file() and not live_path.is_symlink(),
                f"current ownership-review receipt is not a regular file: {name}")
        require(authority.source.git_tracked(repo, live_path),
                f"current ownership-review receipt is not tracked: {name}")
        require(live_path.read_bytes() == raw,
                f"current ownership-review receipt drift from barrier: {name}")
    receipt_diff_commands = (
        ["git", "-C", str(repo), "diff", "--quiet", barrier_ref, "HEAD", "--",
         EXPECTED_RECEIPT_DIRECTORY],
        ["git", "-C", str(repo), "diff", "--quiet", "HEAD", "--",
         EXPECTED_RECEIPT_DIRECTORY],
        ["git", "-C", str(repo), "diff", "--quiet", "--cached", "HEAD", "--",
         EXPECTED_RECEIPT_DIRECTORY],
    )
    for command in receipt_diff_commands:
        clean = subprocess.run(command, capture_output=True)
        require(clean.returncode == 0,
                "current ownership-review receipt Git state drifted from barrier")
    closure = receipt_closure(barrier_raw)
    expected_pins = {
        "ownership_review_receipt_tree_sha256": closure["tree_sha256"],
        "ownership_review_receipt_logical_lines": closure["logical_lines"],
        "ownership_review_receipt_bytes": closure["bytes"],
        "ownership_review_finding_total": closure["finding_total"],
        "ownership_review_p0_findings": closure["severity_counts"]["P0"],
        "ownership_review_p1_findings": closure["severity_counts"]["P1"],
        "ownership_review_p2_findings": closure["severity_counts"]["P2"],
        "ownership_review_p3_findings": closure["severity_counts"]["P3"],
        "ownership_review_finding_id_sha256": closure["finding_id_sha256"],
    }
    for key, expected in expected_pins.items():
        require(type(manifest[key]) is type(expected) and manifest[key] == expected,
                f"ownership-review completion pin drift: {key}")

    transition_ref = git_direct_transition_child(
        repo, barrier_ref, "ownership-review completion"
    )
    require_direct_child(
        repo, barrier_ref, transition_ref, "ownership-review completion"
    )
    require_manifest_only_commit(
        repo, barrier_ref, transition_ref, "ownership-review completion"
    )
    transition_manifest = load_toml_bytes(
        git_object_bytes(
            repo, transition_ref, EXPECTED_MANIFEST_RELATIVE,
            "ownership-review completion manifest",
        ),
        "ownership-review completion manifest",
    )
    require(set(transition_manifest) == set(barrier_manifest) | COMPLETION_PIN_KEYS,
            "ownership-review completion manifest key set drift")
    for key in barrier_manifest:
        if key not in {"active_queue", "ownership_review_status"}:
            require(transition_manifest[key] == barrier_manifest[key],
                    f"ownership-review completion changes another field: {key}")
    require(transition_manifest["active_queue"] == "correction"
            and transition_manifest["ownership_review_status"] == "complete",
            "ownership-review completion transition state drift")
    for key in COMPLETION_PIN_KEYS:
        require(transition_manifest[key] == manifest[key],
                f"ownership-review completion pin changed after transition: {key}")
    require(set(manifest) == set(transition_manifest),
            "current campaign manifest key set drifted after ownership close")
    for key in transition_manifest:
        if key != "active_queue":
            require(manifest[key] == transition_manifest[key],
                    f"current campaign manifest changed closed field: {key}")
    component_results = validate_component_set(authority, list(authority.components))
    support_result = validate_support_receipt(
        authority.receipt_directory / "support.ownership-review.toml", authority
    )
    overlay_result = validate_overlay_receipt(
        authority.receipt_directory / "overlays.ownership-review.toml", authority
    )
    semantic_finding_ids = [
        finding_id
        for result in component_results
        for finding_id in result.finding_ids
    ]
    semantic_finding_ids.extend(support_result.finding_ids)
    semantic_finding_ids.extend(overlay_result.finding_ids)
    require(len(semantic_finding_ids) == len(set(semantic_finding_ids))
            == closure["finding_total"],
            "ownership-review barrier semantic finding closure drift")


def load_active_authority(repo: Path, upstream: Path,
                          manifest_path: Path) -> OwnershipAuthority:
    source = load_source_checker(repo)
    source_authority = source.load_authority(repo, upstream, manifest_path)
    manifest = source_authority.manifest
    require(manifest["source_review_status"] == "complete",
            "ownership review requires complete source review")
    for key, expected in EXPECTED_MANIFEST_PATHS.items():
        require(manifest.get(key) == expected,
                f"campaign ownership field drift: {key}")
    ownership_status = manifest.get("ownership_review_status")
    require(ownership_status in {"active", "complete"},
            "campaign ownership-review status is invalid")
    queue_order = manifest["queue_order"]
    queue = manifest["active_queue"]
    require(queue_order.index(queue) >= queue_order.index("ownership-review"),
            "active queue precedes ownership review")
    if queue == "ownership-review":
        require(ownership_status == "active",
                "ownership-review queue requires active ownership status")
    else:
        require(ownership_status == "complete",
                "later queue requires complete ownership status")
    source.require_ancestor(repo, EXPECTED_WORKSPACE_BASE_REF,
                            "ownership workspace_base_ref")
    validate_source_prerequisite(source, source_authority)
    validate_scc_partition(source_authority)

    plan_path = source.repo_path(repo, str(manifest["ownership_review_plan"]),
                                 "ownership-review plan")
    plan = load_toml(plan_path, "ownership-review plan")
    require(source.git_tracked(repo, plan_path),
            "ownership-review plan is not tracked")
    schema_path = source.repo_path(repo, str(manifest["ownership_review_schema"]),
                                   "ownership-review schema")
    require(schema_path.is_file() and source.git_tracked(repo, schema_path),
            "ownership-review schema is missing or untracked")
    require(source.sha256(plan_path) == EXPECTED_PLAN_SHA256
            and source.logical_lines(plan_path) == EXPECTED_PLAN_LOGICAL_LINES
            and plan_path.stat().st_size == EXPECTED_PLAN_BYTE_COUNT,
            "ownership-review plan bytes drifted from launch authority")
    require(source.sha256(schema_path) == EXPECTED_SCHEMA_SHA256
            and source.logical_lines(schema_path) == EXPECTED_SCHEMA_LOGICAL_LINES
            and schema_path.stat().st_size == EXPECTED_SCHEMA_BYTE_COUNT,
            "ownership-review schema bytes drifted from launch authority")
    families, profiles, owner_contract_path, profile_path = load_contracts(
        source, source_authority
    )
    ledgers, ledger_keys_by_component = load_ledgers(
        source, source_authority, profiles
    )
    validate_independent_scc(source_authority, ledgers["dependency"])
    components = build_component_expectations(
        source,
        source_authority,
        families,
        profiles,
        owner_contract_path,
        profile_path,
        ledger_keys_by_component,
    )
    support_binding, support_records, support_keys, support_sha = \
        build_support_expectation(source, source_authority)
    source_overlay_binding = source_receipt_binding(
        source, source_authority, "overlays", "overlays.source-review.toml"
    )
    overlays = build_overlay_expectations(
        source, source_authority, components, support_records
    )
    receipt_directory = source.repo_path(repo, EXPECTED_RECEIPT_DIRECTORY,
                                         "ownership-review receipt directory")
    authority = OwnershipAuthority(
        source,
        source_authority,
        repo,
        upstream,
        plan,
        receipt_directory,
        families,
        profiles,
        ledgers,
        ledger_keys_by_component,
        components,
        support_binding,
        support_records,
        support_keys,
        support_sha,
        source_overlay_binding,
        overlays,
    )
    validate_plan(authority, plan_path)
    validate_launch_state(authority, manifest_path)
    validate_completion_state(authority)
    return authority


def load_authority(repo: Path, upstream: Path,
                   manifest_path: Path) -> OwnershipAuthority:
    require(manifest_path.is_file() and not manifest_path.is_symlink(),
            f"missing or non-regular campaign manifest: {manifest_path}")
    try:
        manifest_relative = manifest_path.resolve().relative_to(repo).as_posix()
    except ValueError as error:
        raise OwnershipReviewError(
            f"campaign manifest is outside repository: {manifest_path}"
        ) from error
    require(manifest_relative == EXPECTED_MANIFEST_RELATIVE,
            "ownership-review manifest path drift")
    tracked_manifest = subprocess.run(
        [
            "git", "-C", str(repo), "ls-files", "--error-unmatch", "--",
            EXPECTED_MANIFEST_RELATIVE,
        ],
        capture_output=True,
    )
    require(tracked_manifest.returncode == 0,
            "ownership-review campaign manifest is not tracked")
    current_manifest = load_toml(manifest_path, "current campaign manifest")
    launch_ref = current_manifest.get("ownership_review_launch_ref")
    if launch_ref != "pending":
        launch_ref = require_full_ancestor(
            repo, launch_ref, "ownership-review launch ref"
        )
        require_git_regular_blob(
            repo, "HEAD", EXPECTED_MANIFEST_RELATIVE,
            "ownership-review campaign manifest",
        )
        index_manifest_bytes = git_index_bytes(
            repo, EXPECTED_MANIFEST_RELATIVE,
            "ownership-review campaign manifest",
        )
        require_worktree_and_index_clean(
            repo, EXPECTED_MANIFEST_RELATIVE,
            "ownership-review campaign manifest",
        )
        head_manifest_bytes = git_object_bytes(
            repo, "HEAD", EXPECTED_MANIFEST_RELATIVE,
            "ownership-review campaign manifest",
        )
        require(head_manifest_bytes == index_manifest_bytes
                == manifest_path.read_bytes(),
                "ownership-review campaign manifest Git layers drift")
        validate_launch_frozen_bytes(repo, launch_ref)
    status = current_manifest.get("ownership_review_status")
    queue = current_manifest.get("active_queue")
    if status == "active":
        require(queue == "ownership-review",
                "active ownership status requires ownership-review queue")
        authority = load_active_authority(repo, upstream, manifest_path)
        authority.campaign_manifest = current_manifest
        authority.live_repo = repo
        return authority

    require(status == "complete",
            "campaign ownership-review status is invalid")
    require(isinstance(current_manifest.get("queue_order"), list)
            and queue in current_manifest["queue_order"],
            "complete ownership review has an invalid active queue")
    require(current_manifest["queue_order"].index(queue)
            >= current_manifest["queue_order"].index("correction"),
            "complete ownership review precedes correction queue")
    require(current_manifest.get("source_review_status") == "complete",
            "ownership review requires complete source review")
    for key, expected in EXPECTED_MANIFEST_PATHS.items():
        require(current_manifest.get(key) == expected,
                f"campaign ownership field drift: {key}")
    require((set(current_manifest) & COMPLETION_PIN_KEYS) == COMPLETION_PIN_KEYS,
            "complete ownership review has an incomplete completion pin set")
    barrier_ref = require_full_ancestor(
        repo, current_manifest.get("ownership_review_barrier_ref"),
        "ownership-review barrier ref",
    )

    checkout_owner = tempfile.TemporaryDirectory(
        prefix="backend-port-ownership-review-barrier-"
    )
    checkout = Path(checkout_owner.name) / "repo"
    clone = subprocess.run(
        [
            "git", "clone", "--shared", "--no-checkout", "--quiet",
            str(repo), str(checkout),
        ],
        capture_output=True,
        text=True,
    )
    require(clone.returncode == 0,
            f"cannot create ownership barrier replay checkout: {clone.stderr.strip()}")
    checkout_result = subprocess.run(
        ["git", "-C", str(checkout), "checkout", "--detach", "--quiet", barrier_ref],
        capture_output=True,
        text=True,
    )
    require(checkout_result.returncode == 0,
            "cannot check out ownership-review barrier commit")
    validate_launch_frozen_bytes(checkout, launch_ref)
    barrier_authority = load_active_authority(
        checkout, upstream, checkout / EXPECTED_MANIFEST_RELATIVE
    )
    barrier_authority.campaign_manifest = current_manifest
    barrier_authority.live_repo = repo
    barrier_authority.replay_checkout = checkout_owner
    validate_completion_state(barrier_authority, repo, current_manifest)
    return barrier_authority


def validate_component_set(authority: OwnershipAuthority,
                           component_ids: Iterable[str]) -> list[ComponentResult]:
    return [
        validate_component_receipt(
            canonical_component_receipt_path(authority, component_id), authority
        )
        for component_id in component_ids
    ]


def global_check(authority: OwnershipAuthority) -> None:
    require(authority.receipt_directory.is_dir(),
            f"missing ownership-review receipt directory: {authority.receipt_directory}")
    component_paths = {
        canonical_component_receipt_path(authority, component_id)
        for component_id in authority.components
    }
    support_path = authority.receipt_directory / "support.ownership-review.toml"
    overlay_path = authority.receipt_directory / "overlays.ownership-review.toml"
    expected_paths = {*component_paths, support_path, overlay_path}
    actual_entries = list(authority.receipt_directory.rglob("*"))
    require(all(path.parent == authority.receipt_directory
                and path.is_file() and not path.is_symlink()
                for path in actual_entries),
            "ownership-review receipt directory contains a non-regular entry")
    actual_paths = set(actual_entries)
    require(actual_paths == expected_paths,
            f"ownership-review receipt set drift: {len(actual_paths)}/{len(expected_paths)}")
    campaign_manifest = authority.campaign_manifest or authority.source_authority.manifest
    if campaign_manifest["ownership_review_status"] == "active":
        close_ref = authority.source.git_head(authority.repo)
        validate_review_phase_commit_scope(authority, authority.repo, close_ref)
        for staged in (False, True):
            arguments = ["git", "-C", str(authority.repo), "diff", "--quiet"]
            if staged:
                arguments.append("--cached")
            arguments.extend(["HEAD", "--", EXPECTED_RECEIPT_DIRECTORY])
            clean = subprocess.run(arguments, capture_output=True)
            require(clean.returncode == 0,
                    "ownership-review close has uncommitted receipt bytes")
    results = validate_component_set(authority, list(authority.components))
    support = validate_support_receipt(support_path, authority)
    overlays = validate_overlay_receipt(overlay_path, authority)
    component_ids = [result.component_id for result in results]
    finding_ids = [finding_id for result in results for finding_id in result.finding_ids]
    all_finding_ids = [*finding_ids, *support.finding_ids, *overlays.finding_ids]
    require(len(component_ids) == len(set(component_ids)) == EXPECTED_COMPONENT_COUNT,
            "global ownership component overlap or omission")
    require(len(all_finding_ids) == len(set(all_finding_ids)),
            "global ownership finding IDs are not unique")
    open_findings = (sum(result.open_findings for result in results)
                     + support.open_findings + overlays.open_findings)
    audit = "red" if open_findings else "green"
    print(
        "backend ownership-review evidence complete: structure=complete, "
        f"audit={audit}, components={len(results)}/{EXPECTED_COMPONENT_COUNT}, "
        f"units={len(authority.source_authority.units)}/{EXPECTED_UNIT_COUNT}, "
        f"sources={len(authority.source_authority.owners)}/{EXPECTED_SOURCE_COUNT}, "
        f"targets={len(authority.source_authority.translations)}/{EXPECTED_TARGET_COUNT}, "
        f"support={len(authority.support_records)}/{EXPECTED_SUPPORT_COUNT}, "
        f"overlays={len(authority.overlays)}/{EXPECTED_OVERLAY_COUNT}, "
        f"open_findings={open_findings}, "
        f"queue={(authority.campaign_manifest or authority.source_authority.manifest)['active_queue']}"
    )


def partial_check(path: Path, authority: OwnershipAuthority) -> None:
    if not path.is_absolute():
        path = authority.repo / path
    elif authority.live_repo is not None:
        try:
            relative = path.resolve().relative_to(authority.live_repo)
        except ValueError:
            pass
        else:
            path = authority.repo / relative
    path = path.resolve()
    require(path.parent == authority.receipt_directory,
            f"ownership-review receipt is outside its directory: {path}")
    if path.name == "support.ownership-review.toml":
        validate_component_set(authority, list(authority.components))
        result = validate_support_receipt(path, authority)
        print("backend ownership-review support receipt complete: "
              f"artifacts={len(authority.support_records)}, "
              f"open_findings={result.open_findings}")
        return
    if path.name == "overlays.ownership-review.toml":
        validate_component_set(authority, list(authority.components))
        validate_support_receipt(
            authority.receipt_directory / "support.ownership-review.toml", authority
        )
        result = validate_overlay_receipt(path, authority)
        print("backend ownership-review overlay receipt complete: "
              f"overlays={len(authority.overlays)}, "
              f"open_findings={result.open_findings}")
        return
    component_id = path.name.removesuffix(".ownership-review.toml")
    require(component_id in authority.components,
            f"unknown ownership-review component receipt: {path.name}")
    candidate = authority.source_authority.components[component_id]
    prior = [
        component.component_id
        for component in authority.source_authority.components.values()
        if component.order_group < candidate.order_group
    ]
    validate_component_set(authority, prior)
    result = validate_component_receipt(path, authority)
    print("backend ownership-review component receipt complete: "
          f"component={result.component_id}, wave={result.review_wave}, "
          f"units={len(candidate.units)}, "
          f"sources={len(authority.components[component_id].source_records)}, "
          f"targets={len(authority.components[component_id].target_records)}, "
          f"open_findings={result.open_findings}")


def admission_check(authority: OwnershipAuthority) -> None:
    manifest = authority.campaign_manifest or authority.source_authority.manifest
    require(manifest["active_queue"] == "ownership-review",
            "ownership-review admission is only valid in the ownership-review queue")
    require(manifest["ownership_review_status"] == "active",
            "ownership-review admission requires active ownership status")
    receipt_directory = authority.live_repo / EXPECTED_RECEIPT_DIRECTORY
    if receipt_directory.exists() or receipt_directory.is_symlink():
        require(receipt_directory.is_dir() and not receipt_directory.is_symlink(),
                "ownership-review admission receipt path is not a directory")
        require(not any(receipt_directory.iterdir()),
                "ownership-review admission requires an empty live receipt directory")
    print(
        "backend ownership-review admission clean: "
        f"components={len(authority.components)}, "
        f"units={len(authority.source_authority.units)}, "
        f"sources={len(authority.source_authority.owners)}, "
        f"targets={len(authority.source_authority.translations)}, "
        f"support={len(authority.support_records)}, "
        f"overlays={len(authority.overlays)}, "
        f"fields={len(authority.ledgers['field'])}, "
        f"lifecycle={len(authority.ledgers['lifecycle'])}, "
        f"configurations={len(authority.ledgers['configuration'])}, "
        f"dependencies={len(authority.ledgers['dependency'])}, "
        f"source_receipts={EXPECTED_COMPONENT_COUNT + 2}"
    )


def main() -> int:
    args = parse_args()
    repo = args.repo_root.resolve()
    upstream = args.upstream_root.resolve()
    manifest_path = args.manifest if args.manifest.is_absolute() else repo / args.manifest
    authority = load_authority(repo, upstream, manifest_path.resolve())
    if args.admission:
        admission_check(authority)
    elif args.receipt is not None:
        partial_check(args.receipt, authority)
    else:
        global_check(authority)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, KeyError, TypeError, ValueError, csv.Error,
            tomllib.TOMLDecodeError) as error:
        print(f"backend ownership-review failure: {error}", file=sys.stderr)
        raise SystemExit(1)
