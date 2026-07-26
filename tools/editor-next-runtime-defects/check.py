#!/usr/bin/env python3
"""Fail-closed checker for the Editor Next runtime-defect atlas."""

from __future__ import annotations

import argparse
import collections
import hashlib
import json
import pathlib
import re
import subprocess
import sys
import tomllib
from typing import Any, Iterable


SCHEMA = "nuxie.editor-next.runtime-defect-atlas/v2"
CORRECTIONS_SCHEMA = "nuxie.editor-next.runtime-defect-corrections/v1"
FIXTURES_SCHEMA = "nuxie.editor-next.runtime-defect-fixtures/v1"
RUNTIME_IDS = {f"RT-ED-{value:03d}" for value in range(1, 8)}
LOCAL_IDS = {
    *(f"LOC-{value:03d}" for value in range(1, 10)),
    *(f"LOC-{value:03d}" for value in range(11, 20)),
}
BASELINE_IDS = RUNTIME_IDS | LOCAL_IDS
EXPECTED_CORRECTION_IDS = {f"COR-{value:02d}" for value in range(1, 13)}
EXPECTED_CORRECTIONS_SHA256 = (
    "d5e3c41d43db53b925f4c01834deb73c51669e5df5fec1f3db7a28393aab83a7"
)
EXPECTED_CHILDREN = {
    "RT-ED-001": (set(), set(), set()),
    "RT-ED-002": (set(), set(), set()),
    "RT-ED-003": ({"P04-C01", "P19-C03"}, set(), set()),
    "RT-ED-004": (set(), set(), set()),
    "RT-ED-005": ({"P09-C01"}, set(), set()),
    "RT-ED-006": (set(), set(), set()),
    "RT-ED-007": ({"P19-C09"}, set(), set()),
    "LOC-001": (set(), {"P13-C07"}, set()),
    "LOC-002": (
        {"P04-C11", "P09-C03", "P09-C06"},
        {"P09-C01"},
        set(),
    ),
    "LOC-003": (set(), set(), set()),
    "LOC-004": (set(), set(), set()),
    "LOC-005": ({"P09-C05"}, set(), set()),
    "LOC-006": (set(), {"P09-C04"}, set()),
    "LOC-007": ({"P11-C12"}, set(), set()),
    "LOC-008": (set(), {"P08-C06"}, set()),
    "LOC-009": (set(), {"P14-C01"}, set()),
    "LOC-011": (set(), {"P08-C06"}, set()),
    "LOC-012": (set(), {"P19-C08"}, set()),
    "LOC-013": (set(), {"P08-C08"}, set()),
    "LOC-014": (set(), {"P08-C09"}, set()),
    "LOC-015": (set(), {"P18-C01", "P18-C04", "P18-C05", "P18-C07"}, set()),
    "LOC-016": (set(), {"P18-C01", "P18-C04"}, set()),
    "LOC-017": (set(), {"P18-C07"}, set()),
    "LOC-018": (set(), {"P04-C12", "P07-C04"}, set()),
    "LOC-019": (set(), {"P14-C06"}, set()),
}
EXPECTED_LEASE = {
    "refreshed": "2026-07-24",
    "active_wave": "FL-A",
    "branch": "levi/fl-a",
    "reserved_files": {
        "crates/nuxie-graph/src/lib.rs",
        "crates/nuxie-runtime/src/artboard.rs",
        "crates/nuxie-runtime/src/artboard_data_bind.rs",
        "crates/nuxie-runtime/src/components.rs",
        "crates/nuxie-runtime/src/constraints.rs",
        "crates/nuxie-runtime/src/draw.rs",
        "crates/nuxie-runtime/src/focus.rs",
        "crates/nuxie-runtime/src/lib.rs",
        "crates/nuxie-runtime/src/objects.rs",
        "crates/nuxie-runtime/src/retained_data_bind.rs",
        "crates/nuxie-runtime/src/text.rs",
        "docs/runtime-frame-loop-gaps.toml",
    },
    "future_files": {
        "crates/nuxie-runtime/src/animation.rs",
        "crates/nuxie-runtime/src/state_machine.rs",
        "crates/nuxie-runtime/src/state_machine/**",
    },
    "shared_ledgers": {
        "docs/runtime-frame-loop-ownership.toml",
        "docs/runtime-frame-loop-status.md",
        "file-correspondence-manifest.toml",
    },
}
EXPECTED_PROGRAM = {
    "formal_objective": "own-complete-editor-reported-runtime-defect-queue",
    "state_source": "docs/editor-next-runtime-defect-atlas.toml",
    "schedule_source": "docs/editor-next-runtime-defect-status.md",
    "completion_source": "docs/editor-next-runtime-defect-goal.md",
    "port_plan": "docs/editor-next-runtime-defect-port-map.md",
    "porting_law": "docs/PORTING.md",
    "collision_ledger": "docs/runtime-frame-loop-ownership.toml",
    "coordinator_thread": "019f9c97-edcf-76d3-a786-11f443da22d3",
    "editor_consumption_required": False,
    "editor_merge_blocks_program": False,
    "parallel_execution": True,
    "runtime_fix_assignment_requires_tracked_dependency": True,
    "terminal_state": "closed",
}
EXPECTED_INBOX = {
    "canonical_branch": "origin/levi/editor-next-cutover-assembly",
    "runtime_defects_path": "plans/nuxie-editor-next-runtime-defects.md",
    "parity_ledger_path": "plans/nuxie-editor-next-parity-ledger.json",
}
PROGRAM_KEYS = set(EXPECTED_PROGRAM) | {"intake_cycle"}
INBOX_KEYS = set(EXPECTED_INBOX) | {
    "last_consumed_editor_ref",
    "last_consumed_runtime_defects_sha256",
    "last_consumed_parity_ledger_sha256",
    "newest_available_editor_ref",
    "newest_available_runtime_defects_sha256",
    "newest_available_parity_ledger_sha256",
    "unconsumed_records",
    "imported_atlas_count",
}
OWNER_CLASSES = {"runtime", "api", "renderer", "editor", "artifact"}
CLASSIFICATIONS = {
    "unqualified",
    "tracked-gap",
    "structural-mistranslation",
    "local-translation-defect",
    "api-surface-gap",
    "verification-gap",
    "editor-integration-defect",
    "upstream-drift",
    "additive-product-feature",
    "stale-oracle",
    "retracted",
}
STATES = {
    "reported",
    "intake-needs-evidence",
    "reproduced",
    "qualified",
    "mapped",
    "executor-green",
    "orchestrator-verified",
    "handoff-ready",
    "editor-consumed",
    "user-decided",
    "stale-oracle",
    "retracted",
    "closed",
}
TRANSITIONS = {
    "reported": {"intake-needs-evidence", "reproduced"},
    "intake-needs-evidence": {"reproduced", "retracted"},
    "reproduced": {"qualified", "user-decided", "stale-oracle", "retracted"},
    "qualified": {"mapped"},
    "mapped": {"executor-green"},
    "executor-green": {"orchestrator-verified"},
    "orchestrator-verified": {"handoff-ready"},
    "handoff-ready": {"editor-consumed", "closed"},
    "editor-consumed": {"closed"},
    "user-decided": {"closed"},
    "stale-oracle": {"closed"},
    "retracted": {"closed"},
    "closed": set(),
}
RESULT_STATUSES = {"pending", "pass", "fail", "not-applicable"}
FIXTURE_KINDS = {
    "artifact",
    "browser-renderer",
    "cpp-runtime",
    "editor-product",
    "historical",
    "rust-runtime",
    "three-layer",
}
FIXTURE_STATUSES = {"registered", "implemented", "qualified", "historical"}
SHA_RE = re.compile(r"^[0-9a-f]{40}$")
SHA256_RE = re.compile(r"^[0-9a-f]{64}$")
CHILD_RE = re.compile(r"^P\d{2}-C\d{2}$")
LOCAL_ID_RE = re.compile(r"^LOC-\d{3}$")
RUNTIME_ID_RE = re.compile(r"^RT-ED-\d{3}$")
DEFECT_HEADING_RE = re.compile(
    r"^### (?P<id>(?:RT-ED|LOC)-\d{3}) — (?P<title>\S(?:.*\S)?)$"
)
MARKDOWN_HEADING_RE = re.compile(
    r"^(?P<indent>[ \t]*)(?P<marks>#{1,6})(?:[ \t]+(?P<title>.*)|[ \t]*)$"
)
DEFECT_LIKE_HEADING_RE = re.compile(
    r"^[ \t]*#{1,}[ \t]*[`*]*(?:RT[-_ ]?ED|LOC)"
    r"(?=[-_\s\d`*]|$)",
    re.IGNORECASE,
)
ID_SHAPED_TOKEN_PATTERN = (
    r"[`*\[]*[A-Z][A-Z0-9_.:/_-]*\d[A-Z0-9_.:/_-]*"
    r"[`*\]]*(?:\([^)\r\n]*\))?"
)
ARBITRARY_TRACKING_HEADING_RE = re.compile(
    rf"^[ \t]*#{{1,}}[ \t]*{ID_SHAPED_TOKEN_PATTERN}(?=[ \t]|$)",
    re.IGNORECASE,
)
ARBITRARY_TRACKING_TEXT_RE = re.compile(
    rf"^[ \t]*{ID_SHAPED_TOKEN_PATTERN}(?=[ \t]|$)",
    re.IGNORECASE,
)
RECORD_SEPARATOR_HEADING_RE = re.compile(
    r"^[ \t]*#{1,3}(?!#)[ \t]*\S.*"
    r"(?:[ \t]+—(?:[ \t]|$)|[ \t]+-[ \t]+)"
)
RECORD_SEPARATOR_TEXT_RE = re.compile(
    r"^[ \t]*\S.*(?:[ \t]+—(?:[ \t]|$)|[ \t]+-[ \t]+)"
)
DEFECT_LIKE_TEXT_RE = re.compile(
    r"^[ \t]*[`*]*(?:RT[-_ ]?ED|LOC)(?=[-_\s\d`*]|$)",
    re.IGNORECASE,
)
RAW_HTML_TYPE_1_OPEN_RE = re.compile(
    r"^<(?P<tag>script|pre|style|textarea)(?=[ \t>]|$)",
    re.IGNORECASE,
)
RAW_HTML_TYPE_3_OPEN_RE = re.compile(r"^<\?")
RAW_HTML_TYPE_4_OPEN_RE = re.compile(r"^<![A-Z]", re.IGNORECASE)
RAW_HTML_TYPE_5_OPEN_RE = re.compile(r"^<!\[CDATA\[")
RAW_HTML_TYPE_6_OPEN_RE = re.compile(
    r"^</?(?:address|article|aside|base|basefont|blockquote|body|caption|"
    r"center|col|colgroup|dd|details|dialog|dir|div|dl|dt|fieldset|"
    r"figcaption|figure|footer|form|frame|frameset|h[1-6]|head|header|"
    r"hr|html|iframe|legend|li|link|main|menu|menuitem|nav|noframes|ol|"
    r"optgroup|option|p|param|search|section|summary|table|tbody|td|tfoot|"
    r"th|thead|title|tr|track|ul)(?=[ \t]|/?>|$)",
    re.IGNORECASE,
)
RAW_HTML_ATTRIBUTE_NAME = r"[A-Za-z_:][A-Za-z0-9_.:-]*"
RAW_HTML_UNQUOTED_VALUE = r"[^ \t\"'=<>`]+"
RAW_HTML_SINGLE_QUOTED_VALUE = r"'[^']*'"
RAW_HTML_DOUBLE_QUOTED_VALUE = r'"[^"]*"'
RAW_HTML_ATTRIBUTE_VALUE = (
    rf"(?:{RAW_HTML_UNQUOTED_VALUE}|"
    rf"{RAW_HTML_SINGLE_QUOTED_VALUE}|"
    rf"{RAW_HTML_DOUBLE_QUOTED_VALUE})"
)
RAW_HTML_ATTRIBUTE = (
    rf"(?:[ \t]+{RAW_HTML_ATTRIBUTE_NAME}"
    rf"(?:[ \t]*=[ \t]*{RAW_HTML_ATTRIBUTE_VALUE})?)"
)
RAW_HTML_TYPE_7_OPEN_RE = re.compile(
    rf"^(?:<[A-Za-z][A-Za-z0-9-]*{RAW_HTML_ATTRIBUTE}*[ \t]*/?>|"
    rf"</[A-Za-z][A-Za-z0-9-]*[ \t]*>)[ \t]*$"
)
ASCII_BLANK_RE = re.compile(r"^[ \t]*$")
COMMONMARK_SOURCE_LINE_RE = re.compile(
    r"[^\r\n]*(?:\r\n|\r|\n|$)"
)
THEMATIC_BREAK_CONTENT_RE = re.compile(
    r"(?:(?:\*[ \t]*){3,}|(?:-[ \t]*){3,}|(?:_[ \t]*){3,})"
)
BULLET_LIST_MARKER_CONTENT_RE = re.compile(
    r"[-+*](?P<suffix>[ \t].*|)$"
)
ORDERED_LIST_MARKER_CONTENT_RE = re.compile(
    r"(?P<number>\d{1,9})[.)](?P<suffix>[ \t].*|)$"
)
BLOCK_QUOTE_CONTENT_RE = re.compile(r">")
SETEXT_UNDERLINE_RE = re.compile(
    r"^[ \t]{0,3}(?P<marker>=+|-+)[ \t]*$"
)
SETEXT_INELIGIBLE_RE = re.compile(
    r"^[ \t]{0,3}(?:[-*+][ \t]+|\d+[.)][ \t]+|>[ \t]*)"
)
EDITOR_SHA_BULLET_RE = re.compile(
    r"^[-*+][ \t]+(?:\*\*)?Editor[ \t]+SHA(?:\*\*)?"
    r"[ \t]*:[ \t]*`?(?P<sha>[0-9a-f]{40})`?[ \t]*$",
    re.IGNORECASE,
)
RUNTIME_SHA_BULLET_RE = re.compile(
    r"^[-*+][ \t]+(?:\*\*)?Runtime[ \t]+SHA(?:\*\*)?"
    r"[ \t]*:[ \t]*`?(?P<sha>[0-9a-f]{40})`?[ \t]*$",
    re.IGNORECASE,
)
COMMAND_BULLET_RE = re.compile(
    r"^[-*+][ \t]+(?:\*\*)?(?:exact[ \t]+)?command"
    r"(?:\*\*)?[ \t]*:[ \t]*.*`(?P<code>[^`\r\n]*)`[ \t]*$",
    re.IGNORECASE,
)
EVIDENCE_BULLET_RE = re.compile(
    r"^[-*+][ \t]+(?:\*\*)?"
    r"(?:result|evidence|observation|failure|deficiency|classification)"
    r"(?:\*\*)?[ \t]*:[ \t]*\S.*$",
    re.IGNORECASE,
)
DEFECT_ANCHOR_RE = re.compile(
    r'^<a[ \t]+id="(?:rt-ed|loc)-\d{3}"[ \t]*></a>[ \t]*$',
    re.IGNORECASE,
)
FULL_SHA_RE = re.compile(r"(?<![0-9a-f])[0-9a-f]{40}(?![0-9a-f])")
RUNTIME_PIN_SHA_RE = re.compile(
    r"\bruntime[ \t]+pin\b[^0-9a-f]*`?"
    r"(?P<sha>[0-9a-f]{40})(?![0-9a-f])",
    re.IGNORECASE,
)
EDITOR_PROVENANCE_SHA_RE = re.compile(
    r"\b(?:assembly[ \t]+(?:base|checkpoint)|"
    r"editor[ \t]+(?:base|checkpoint|sha)|"
    r"qualification[ \t]+code[ \t]+checkpoint)"
    r"[ \t:;,=-]*`?(?P<sha>[0-9a-f]{40})(?![0-9a-f])",
    re.IGNORECASE,
)
COMMAND_EVIDENCE_LABELS = {
    "command",
    "editor integration command",
    "exact command",
    "exact product command",
    "failed unchanged direct-runtime reproducer",
    "focused compiler command",
    "focused consumer command",
    "focused editor/product command",
    "focused journey comparison command",
    "focused product command",
    "focused product commands",
    "focused product reproduction",
    "focused reproduction",
    "focused workspace comparison command",
    "historical focused commands",
    "historical focused reproduction",
    "independent editor-corpus command",
    "minimal diagnostic command",
    "minimal unchanged reproducer",
    "passed editor integration command",
    "passed editor-integration command",
    "supporting legacy-editor lifecycle command",
    "unchanged assembled command",
    "unchanged failed producthost reproducer",
    "unchanged runtime reproducer",
    "unchanged runtime reproducer command",
}
RESULT_EVIDENCE_LABELS = {
    "actual behavior",
    "actual behavior and impact",
    "actual editor next observation",
    "api-seam evidence",
    "c++ oracle evidence",
    "classification",
    "committed evidence manifest",
    "corrected sdk evidence",
    "current classification",
    "current result",
    "evidence",
    "evidence manifest",
    "exact native deficiency",
    "exact product failure",
    "exact runtime deficiency",
    "exact typed runtime deficiency",
    "failure",
    "first significant observation",
    "focused post-fix journey result",
    "frozen journey evidence",
    "historical observations",
    "historical result",
    "immutable editor evidence",
    "immutable evidence",
    "immutable source evidence",
    "incorrect observation",
    "observation",
    "original observation",
    "previous current-product result",
    "product and direct lifecycle observations",
    "result",
    "sdk evidence",
    "shipped artifact and sdk evidence",
    "workspace classification",
}
DEFECT_MENTION_RE = re.compile(
    r"(?<![A-Z0-9-])(?P<id>(?:RT-ED|LOC)-\d{3})(?![A-Z0-9-])"
)
TICKET_RE = re.compile(r"^F-ED-(?:00|0[1-9]|1[0-4])$")
MINIMUM_FLOORS = {
    "runtime_tests": 414,
    "nuxie_tests": 140,
    "cpp_probe_tests": 721,
    "golden_entries": 317,
    "golden_segments": 647,
    "scripted_entries": 317,
    "scripted_segments": 647,
    "renderer_pixels": 1468,
}
MAXIMUM_CEILINGS = {"maximum_sdk_bytes": 9_437_184}
KNOWN_FLOORS = set(MINIMUM_FLOORS) | set(MAXIMUM_CEILINGS)
ARTIFACT_HASH_KEYS = {"proposal", "runtime_defects", "parity_ledger"}
SOURCE_ARTIFACT_PATHS = {
    "cutover-proposal": "nuxie-editor-next-cutover-proposal.md",
    "runtime-defects": "nuxie-editor-next-runtime-defects.md",
    "parity-ledger": "nuxie-editor-next-parity-ledger.json",
}
STIMULUS_ROOTS = {"repo", "rive", "editor"}
REVISION_KEYS = {
    "original_localization_rust_sha",
    "editor_last_consumed_runtime_sha",
    "investigation_head_sha",
    "merged_repair_sha",
    "consumed_runtime_sha",
    "consumed_superproject_sha",
}
RUNTIME_REVISION_KEYS = {
    "original_localization_rust_sha",
    "editor_last_consumed_runtime_sha",
    "investigation_head_sha",
    "consumed_runtime_sha",
}
IMMUTABLE_REVISION_KEYS = {
    "original_localization_rust_sha",
    "merged_repair_sha",
    "consumed_runtime_sha",
    "consumed_superproject_sha",
}
LANDED_REPAIR_PROVENANCE = {
    "RT-ED-003": {
        "merged_repair_sha": "e72323c808b91d706ba3b745396beaca7accd69a",
        "consumed_runtime_sha": "e72323c808b91d706ba3b745396beaca7accd69a",
        "consumed_superproject_sha": "4da896beb5ec6815f6b01a2433875274a321d06c",
    },
    "RT-ED-005": {
        "merged_repair_sha": "08286481b4e7420768f625f901a944f313b84903",
        "consumed_runtime_sha": "e72323c808b91d706ba3b745396beaca7accd69a",
        "consumed_superproject_sha": "4da896beb5ec6815f6b01a2433875274a321d06c",
    },
    "LOC-009": {
        "merged_repair_sha": "7f1450dc22ca7370eac9dc9f422351c2dfcc07ee",
        "consumed_runtime_sha": "e72323c808b91d706ba3b745396beaca7accd69a",
        "consumed_superproject_sha": "4da896beb5ec6815f6b01a2433875274a321d06c",
    },
    "LOC-019": {
        "merged_repair_sha": "ef9dcedd82265efc0184f4f59d5f6aaab0b56cd9",
        "consumed_runtime_sha": "e72323c808b91d706ba3b745396beaca7accd69a",
        "consumed_superproject_sha": "4da896beb5ec6815f6b01a2433875274a321d06c",
    },
}
EARLY_STATES = {"reported", "intake-needs-evidence", "reproduced"}
NORMAL_PIPELINE_STATES = {
    "reported",
    "intake-needs-evidence",
    "reproduced",
    "qualified",
    "mapped",
    "executor-green",
    "orchestrator-verified",
    "handoff-ready",
    "editor-consumed",
    "closed",
}
QUALIFIED_OR_LATER = NORMAL_PIPELINE_STATES - EARLY_STATES
IMPLEMENTED_FIXTURE_STATES = NORMAL_PIPELINE_STATES - {
    "reported",
    "intake-needs-evidence",
}
QUALIFIED_FIXTURE_STATES = QUALIFIED_OR_LATER


class CheckFailure(Exception):
    """Raised when the atlas is incomplete or inconsistent."""


def read_toml(path: pathlib.Path) -> dict[str, Any]:
    try:
        with path.open("rb") as source:
            return tomllib.load(source)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise CheckFailure(f"cannot read {path}: {error}") from error


def duplicate_values(values: Iterable[str]) -> list[str]:
    counts = collections.Counter(values)
    return sorted(value for value, count in counts.items() if count > 1)


def git_head(path: pathlib.Path) -> str:
    result = subprocess.run(
        ["git", "-C", str(path), "rev-parse", "HEAD"],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        raise CheckFailure(
            f"cannot resolve upstream HEAD at {path}: {result.stderr.strip()}"
        )
    return result.stdout.strip()


def run_git(
    repo: pathlib.Path,
    arguments: list[str],
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", "-C", str(repo), *arguments],
        text=True,
        capture_output=True,
        check=False,
    )


def run_git_bytes(
    repo: pathlib.Path,
    arguments: list[str],
) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(
        ["git", "-C", str(repo), *arguments],
        capture_output=True,
        check=False,
    )


def git_blob(
    repo: pathlib.Path,
    revision: str,
    path: str,
    label: str,
    errors: list[str],
) -> bytes | None:
    result = run_git_bytes(
        repo,
        ["show", f"{revision}:{path}"],
    )
    if result.returncode != 0:
        errors.append(
            f"{label} cannot resolve blob {revision}:{path}"
        )
        return None
    return result.stdout


def gitlink_at(
    editor_repo: pathlib.Path,
    revision: str,
    label: str,
    errors: list[str],
) -> str | None:
    result = run_git(
        editor_repo,
        [
            "rev-parse",
            "--verify",
            f"{revision}:third_party/nuxie-runtime",
        ],
    )
    if result.returncode != 0:
        errors.append(
            f"{label} does not contain third_party/nuxie-runtime"
        )
        return None
    return result.stdout.strip()


def git_ref(repo: pathlib.Path, ref: str, errors: list[str]) -> str | None:
    result = run_git(repo, ["rev-parse", "--verify", f"{ref}^{{commit}}"])
    if result.returncode != 0:
        errors.append(
            f"Editor repository does not contain commit/ref {ref!r}: "
            f"{result.stderr.strip()}"
        )
        return None
    return result.stdout.strip()


def git_commit(
    repo: pathlib.Path,
    revision: str,
    repo_label: str,
    field: str,
    errors: list[str],
) -> str | None:
    result = run_git(repo, ["rev-parse", "--verify", f"{revision}^{{commit}}"])
    if result.returncode != 0:
        errors.append(
            f"{field} revision {revision!r} does not resolve as a commit "
            f"in the {repo_label} repository"
        )
        return None
    return result.stdout.strip()


def git_is_ancestor(
    repo: pathlib.Path,
    ancestor: str,
    descendant: str,
    errors: list[str],
    description: str,
    *,
    repo_label: str = "Editor",
) -> bool:
    result = run_git(repo, ["merge-base", "--is-ancestor", ancestor, descendant])
    if result.returncode == 0:
        return True
    if result.returncode == 1:
        errors.append(description)
    else:
        errors.append(
            f"cannot validate {repo_label} commit ancestry "
            f"{ancestor} -> {descendant}: "
            f"{result.stderr.strip()}"
        )
    return False


def validate_editor_source_blob(
    source_root: pathlib.Path,
    editor_repo: pathlib.Path,
    revision: str,
    repo_path: str,
    label: str,
    errors: list[str],
) -> None:
    path = source_path(source_root, repo_path)
    if not validate_file_beneath_root(
        source_root,
        path,
        label,
        errors,
    ):
        return
    expected = git_blob(
        editor_repo,
        revision,
        repo_path,
        label,
        errors,
    )
    if expected is None:
        return
    try:
        actual = path.read_bytes()
    except OSError as error:
        errors.append(f"cannot read {label} at {path}: {error}")
        return
    if actual != expected:
        errors.append(
            f"{label} bytes do not match the pinned Editor commit blob"
        )


def validate_editor_git_provenance(
    inbox: dict[str, Any],
    source_root: pathlib.Path,
    newest_source_root: pathlib.Path,
    editor_repo_dir: pathlib.Path,
    errors: list[str],
) -> str | None:
    consumed = str(inbox.get("last_consumed_editor_ref", ""))
    newest = str(inbox.get("newest_available_editor_ref", ""))
    canonical_branch = str(inbox.get("canonical_branch", ""))
    consumed_commit = git_ref(editor_repo_dir, consumed, errors)
    newest_commit = git_ref(editor_repo_dir, newest, errors)
    canonical_tip = git_ref(editor_repo_dir, canonical_branch, errors)

    if consumed_commit is not None:
        try:
            source_head = git_head(source_root)
        except CheckFailure as error:
            errors.append(str(error))
        else:
            if source_head != consumed_commit:
                errors.append(
                    f"consumed Editor source checkout is {source_head}; "
                    f"inbox pins {consumed_commit}"
                )
    if newest_commit is not None:
        try:
            newest_head = git_head(newest_source_root)
        except CheckFailure as error:
            errors.append(str(error))
        else:
            if newest_head != newest_commit:
                errors.append(
                    f"newest Editor source checkout is {newest_head}; "
                    f"inbox pins {newest_commit}"
                )
    if consumed_commit is not None and newest_commit is not None:
        git_is_ancestor(
            editor_repo_dir,
            consumed_commit,
            newest_commit,
            errors,
            "consumed Editor checkpoint is not an ancestor of the newest "
            "available checkpoint",
        )
    if newest_commit is not None and canonical_tip is not None:
        git_is_ancestor(
            editor_repo_dir,
            newest_commit,
            canonical_tip,
            errors,
            "newest available Editor checkpoint is not an ancestor of the "
            "canonical branch tip",
        )
    runtime_defects_path = str(inbox.get("runtime_defects_path", ""))
    parity_ledger_path = str(inbox.get("parity_ledger_path", ""))
    if runtime_defects_path and parity_ledger_path:
        consumed_sources = (
            (
                f"plans/{SOURCE_ARTIFACT_PATHS['cutover-proposal']}",
                "cutover-proposal",
            ),
            (runtime_defects_path, "runtime-defects"),
            (parity_ledger_path, "parity-ledger"),
        )
        if consumed_commit is not None:
            for repo_path, artifact_id in consumed_sources:
                validate_editor_source_blob(
                    source_root,
                    editor_repo_dir,
                    consumed_commit,
                    repo_path,
                    f"consumed Editor source {artifact_id}",
                    errors,
                )
        if newest_commit is not None:
            for repo_path, artifact_id in (
                (runtime_defects_path, "runtime-defects"),
                (parity_ledger_path, "parity-ledger"),
            ):
                validate_editor_source_blob(
                    newest_source_root,
                    editor_repo_dir,
                    newest_commit,
                    repo_path,
                    f"newest Editor source {artifact_id}",
                    errors,
                )
    return canonical_tip


def path_has_symlink_component(
    root: pathlib.Path,
    path: pathlib.Path,
) -> bool:
    try:
        relative = path.relative_to(root)
    except ValueError:
        return False
    current = root
    if current.is_symlink():
        return True
    for part in relative.parts:
        current /= part
        if current.is_symlink():
            return True
    return False


def validate_file_beneath_root(
    root: pathlib.Path,
    path: pathlib.Path,
    label: str,
    errors: list[str],
) -> bool:
    if path_has_symlink_component(root, path):
        errors.append(f"{label} contains a symlink component")
        return False
    try:
        root_resolved = root.resolve()
        path_resolved = path.resolve()
        path_resolved.relative_to(root_resolved)
    except (OSError, ValueError):
        errors.append(f"{label} escapes declared root {root}")
        return False
    if not path.is_file():
        errors.append(f"{label} does not exist at {path}")
        return False
    return True


def source_path(root: pathlib.Path, canonical_path: str) -> pathlib.Path:
    relative = pathlib.PurePosixPath(canonical_path)
    if relative.parts and root.name == relative.parts[0]:
        relative = pathlib.PurePosixPath(*relative.parts[1:])
    return root.joinpath(*relative.parts)


def strip_html_comments(
    line: str,
    in_comment: bool,
) -> tuple[str, bool]:
    visible: list[str] = []
    cursor = 0
    while cursor < len(line):
        if in_comment:
            close = line.find("-->", cursor)
            if close < 0:
                return "".join(visible), True
            cursor = close + 3
            in_comment = False
            continue
        opening = line.find("<!--", cursor)
        if opening < 0:
            visible.append(line[cursor:])
            break
        visible.append(line[cursor:opening])
        cursor = opening + 4
        in_comment = True
    return "".join(visible), in_comment


def commonmark_indentation(line: str) -> tuple[int, int]:
    columns = 0
    index = 0
    while index < len(line):
        character = line[index]
        if character == " ":
            columns += 1
        elif character == "\t":
            columns += 4 - (columns % 4)
        else:
            break
        index += 1
    return columns, index


def commonmark_indentation_columns(line: str) -> int:
    return commonmark_indentation(line)[0]


def commonmark_fence_marker(
    line: str,
) -> tuple[str, int, str] | None:
    indentation, marker_start = commonmark_indentation(line)
    if indentation > 3 or marker_start == len(line):
        return None
    marker_character = line[marker_start]
    if marker_character not in {"`", "~"}:
        return None
    marker_end = marker_start
    while (
        marker_end < len(line)
        and line[marker_end] == marker_character
    ):
        marker_end += 1
    marker_length = marker_end - marker_start
    if marker_length < 3:
        return None
    remainder = line[marker_end:]
    if marker_character == "`" and "`" in remainder:
        return None
    return marker_character, marker_length, remainder


def commonmark_raw_html_block_start(
    line: str,
) -> tuple[re.Pattern[str] | None, bool, int] | None:
    indentation, content_start = commonmark_indentation(line)
    if indentation > 3:
        return None
    content = line[content_start:]

    type_1 = RAW_HTML_TYPE_1_OPEN_RE.match(content)
    if type_1 is not None:
        return (
            re.compile(
                r"</(?:pre|script|style|textarea)>",
                re.IGNORECASE,
            ),
            False,
            1,
        )
    if RAW_HTML_TYPE_3_OPEN_RE.match(content):
        return re.compile(r"\?>"), False, 3
    if RAW_HTML_TYPE_4_OPEN_RE.match(content):
        return re.compile(r">"), False, 4
    if RAW_HTML_TYPE_5_OPEN_RE.match(content):
        return re.compile(r"\]\]>"), False, 5
    if RAW_HTML_TYPE_6_OPEN_RE.match(content):
        return None, True, 6
    if RAW_HTML_TYPE_7_OPEN_RE.fullmatch(content):
        return None, True, 7
    return None


def commonmark_source_lines(
    content: str,
) -> Iterable[tuple[int, str, str]]:
    for match in COMMONMARK_SOURCE_LINE_RE.finditer(content):
        raw_line = match.group()
        if not raw_line:
            continue
        if raw_line.endswith("\r\n"):
            line = raw_line[:-2]
        elif raw_line.endswith(("\r", "\n")):
            line = raw_line[:-1]
        else:
            line = raw_line
        yield match.start(), raw_line, line


def commonmark_link_title_status(text: str) -> str:
    if not text or text[0] not in {'"', "'", "("}:
        return "invalid"
    closing = ")" if text[0] == "(" else text[0]
    cursor = 1
    while cursor < len(text):
        character = text[cursor]
        if character == "\\":
            cursor += 2
            continue
        if character == closing:
            return (
                "complete"
                if not text[cursor + 1 :].strip(" \t\r\n")
                else "invalid"
            )
        cursor += 1
    if re.search(r"\n[ \t]*\n", text):
        return "invalid"
    return "incomplete"


def commonmark_link_reference_definition_status(
    text: str,
) -> tuple[str, bool]:
    if not text.startswith("["):
        return "invalid", False
    cursor = 1
    while cursor < len(text):
        character = text[cursor]
        if character == "\\":
            cursor += 2
            continue
        if character == "[":
            return "invalid", False
        if character == "]":
            if cursor + 1 >= len(text) or text[cursor + 1] != ":":
                return "invalid", False
            label = text[1:cursor]
            if (
                not label.strip(" \t\r\n")
                or len(label) > 999
                or re.search(r"\n[ \t]*\n", label)
            ):
                return "invalid", False
            suffix = text[cursor + 2 :]
            if re.match(r"^[ \t\r\n]*\n[ \t]*\n", suffix):
                return "invalid", False
            suffix = suffix.lstrip(" \t\r\n")
            if not suffix:
                return "incomplete", False
            destination_end = 0
            if suffix.startswith("<"):
                destination_cursor = 1
                while destination_cursor < len(suffix):
                    destination_character = suffix[destination_cursor]
                    if destination_character == "\\":
                        destination_cursor += 2
                        continue
                    if destination_character == ">":
                        destination_end = destination_cursor + 1
                        break
                    if destination_character in "\r\n":
                        return "invalid", False
                    destination_cursor += 1
                if not destination_end:
                    return "incomplete", False
            else:
                destination_cursor = 0
                parenthesis_depth = 0
                while destination_cursor < len(suffix):
                    destination_character = suffix[destination_cursor]
                    if destination_character == "\\":
                        destination_cursor += 2
                        continue
                    if (
                        destination_character in " \t\r\n"
                        and parenthesis_depth == 0
                    ):
                        break
                    if destination_character == "(":
                        parenthesis_depth += 1
                    elif destination_character == ")":
                        if parenthesis_depth == 0:
                            return "invalid", False
                        parenthesis_depth -= 1
                    destination_cursor += 1
                if destination_cursor == 0:
                    return "invalid", False
                if parenthesis_depth:
                    return "incomplete", False
                destination_end = destination_cursor
            title = suffix[destination_end:].strip(" \t\r\n")
            if not title:
                return "complete", True
            title_status = commonmark_link_title_status(title)
            if title_status == "complete":
                return "complete", False
            if title_status == "incomplete":
                return "incomplete", False
            return "invalid", False
        cursor += 1
    if (
        len(text) - 1 > 999
        or re.search(r"\n[ \t]*\n", text[1:])
    ):
        return "invalid", False
    return "incomplete", False


def commonmark_nonparagraph_block_line(
    line: str,
    paragraph_open: bool,
) -> bool:
    indentation, content_start = commonmark_indentation(line)
    if indentation > 3:
        return False
    content = line[content_start:]
    if THEMATIC_BREAK_CONTENT_RE.fullmatch(content):
        return True
    if BLOCK_QUOTE_CONTENT_RE.match(content):
        return True
    bullet = BULLET_LIST_MARKER_CONTENT_RE.fullmatch(content)
    if bullet is not None:
        if not paragraph_open:
            return True
        return bool(bullet.group("suffix").strip(" \t"))
    ordered = ORDERED_LIST_MARKER_CONTENT_RE.fullmatch(content)
    if ordered is not None:
        if not paragraph_open:
            return True
        return (
            int(ordered.group("number")) == 1
            and bool(ordered.group("suffix").strip(" \t"))
        )
    return False


def commonmark_paragraph_interrupting_block(line: str) -> bool:
    fence = commonmark_fence_marker(line)
    if fence is not None:
        return True
    raw_html = commonmark_raw_html_block_start(line)
    if raw_html is not None and raw_html[2] != 7:
        return True
    return bool(
        MARKDOWN_HEADING_RE.fullmatch(line)
        or SETEXT_UNDERLINE_RE.fullmatch(line)
        or commonmark_nonparagraph_block_line(line, True)
    )


def markdown_visible_lines(
    content: str,
) -> list[tuple[int, str, str]]:
    lines: list[tuple[int, str, str]] = []
    fence_character: str | None = None
    fence_length = 0
    raw_html_end_pattern: re.Pattern[str] | None = None
    raw_html_until_blank = False
    in_html_comment = False
    paragraph_open = False
    link_reference_candidate: str | None = None
    link_reference_candidate_line_indexes: list[int] = []
    link_reference_title_allowed = False
    link_reference_title_candidate: str | None = None
    link_reference_title_candidate_line_indexes: list[int] = []
    for offset, _, line in commonmark_source_lines(content):
        if fence_character is not None:
            close = commonmark_fence_marker(line)
            if (
                close is not None
                and close[0] == fence_character
                and close[1] >= fence_length
                and not close[2].strip(" \t")
            ):
                fence_character = None
                fence_length = 0
            lines.append((offset, line, ""))
            continue
        if raw_html_end_pattern is not None:
            if raw_html_end_pattern.search(line):
                raw_html_end_pattern = None
            paragraph_open = False
            lines.append((offset, line, ""))
            continue
        if raw_html_until_blank:
            if ASCII_BLANK_RE.fullmatch(line):
                raw_html_until_blank = False
            paragraph_open = False
            lines.append((offset, line, ""))
            continue
        indentation, content_start = commonmark_indentation(line)
        content_at_block_start = line[content_start:]
        if (
            indentation <= 3
            and content_at_block_start.startswith("<!--")
        ):
            link_reference_candidate = None
            link_reference_candidate_line_indexes.clear()
            link_reference_title_allowed = False
            link_reference_title_candidate = None
            link_reference_title_candidate_line_indexes.clear()
            in_html_comment = (
                "-->" not in content_at_block_start[4:]
            )
            paragraph_open = False
            lines.append((offset, line, ""))
            continue
        was_in_html_comment = in_html_comment
        visible, in_html_comment = strip_html_comments(
            line,
            in_html_comment,
        )
        if was_in_html_comment:
            link_reference_candidate = None
            link_reference_candidate_line_indexes.clear()
            link_reference_title_allowed = False
            link_reference_title_candidate = None
            link_reference_title_candidate_line_indexes.clear()
            paragraph_open = False
            lines.append((offset, line, ""))
            continue
        if (
            link_reference_candidate is not None
            or link_reference_title_candidate is not None
        ) and commonmark_paragraph_interrupting_block(visible):
            link_reference_candidate = None
            link_reference_candidate_line_indexes.clear()
            link_reference_title_candidate = None
            link_reference_title_candidate_line_indexes.clear()
            paragraph_open = True
        if link_reference_title_candidate is not None:
            if ASCII_BLANK_RE.fullmatch(visible):
                link_reference_title_candidate = None
                link_reference_title_candidate_line_indexes.clear()
                paragraph_open = False
            else:
                combined_title = (
                    f"{link_reference_title_candidate}\n{visible}"
                )
                title_status = commonmark_link_title_status(
                    combined_title
                )
                if title_status == "complete":
                    for line_index in (
                        link_reference_title_candidate_line_indexes
                    ):
                        line_offset, raw, _ = lines[line_index]
                        lines[line_index] = (line_offset, raw, "")
                    link_reference_title_candidate = None
                    link_reference_title_candidate_line_indexes.clear()
                    paragraph_open = False
                    lines.append((offset, line, ""))
                    continue
                if title_status == "incomplete":
                    link_reference_title_candidate = combined_title
                    paragraph_open = False
                    lines.append((offset, line, visible))
                    link_reference_title_candidate_line_indexes.append(
                        len(lines) - 1
                    )
                    continue
                link_reference_title_candidate = None
                link_reference_title_candidate_line_indexes.clear()
                paragraph_open = True
        if link_reference_title_allowed:
            title_indentation, title_start = commonmark_indentation(
                visible
            )
            title_status = "invalid"
            if (
                title_indentation <= 3
                and not ASCII_BLANK_RE.fullmatch(visible)
            ):
                title_status = commonmark_link_title_status(
                    visible[title_start:]
                )
            link_reference_title_allowed = False
            if title_status == "complete":
                paragraph_open = False
                lines.append((offset, line, ""))
                continue
            if title_status == "incomplete":
                link_reference_title_candidate = visible[title_start:]
                paragraph_open = False
                lines.append((offset, line, visible))
                link_reference_title_candidate_line_indexes = [
                    len(lines) - 1
                ]
                continue
        if link_reference_candidate is not None:
            if ASCII_BLANK_RE.fullmatch(visible):
                link_reference_candidate = None
                link_reference_candidate_line_indexes.clear()
                paragraph_open = False
            else:
                combined_candidate = (
                    f"{link_reference_candidate}\n{visible}"
                )
                candidate_status, title_allowed = (
                    commonmark_link_reference_definition_status(
                        combined_candidate
                    )
                )
                if candidate_status == "complete":
                    for line_index in (
                        link_reference_candidate_line_indexes
                    ):
                        line_offset, raw, _ = lines[line_index]
                        lines[line_index] = (line_offset, raw, "")
                    link_reference_candidate = None
                    link_reference_candidate_line_indexes.clear()
                    link_reference_title_allowed = title_allowed
                    paragraph_open = False
                    lines.append((offset, line, ""))
                    continue
                if candidate_status == "incomplete":
                    link_reference_candidate = combined_candidate
                    paragraph_open = True
                    lines.append((offset, line, visible))
                    link_reference_candidate_line_indexes.append(
                        len(lines) - 1
                    )
                    continue
                link_reference_candidate = None
                link_reference_candidate_line_indexes.clear()
                paragraph_open = True
        raw_html = commonmark_raw_html_block_start(visible)
        if raw_html is not None and not (
            raw_html[2] == 7 and paragraph_open
        ):
            end_pattern, until_blank, _ = raw_html
            if end_pattern is not None and end_pattern.search(visible) is None:
                raw_html_end_pattern = end_pattern
            raw_html_until_blank = until_blank
            paragraph_open = False
            lines.append((offset, line, ""))
            continue
        fence = commonmark_fence_marker(visible)
        if fence is not None:
            fence_character = fence[0]
            fence_length = fence[1]
            paragraph_open = False
            lines.append((offset, line, ""))
            continue
        if commonmark_indentation_columns(visible) >= 4:
            lines.append((offset, line, ""))
            continue
        indentation, content_start = commonmark_indentation(visible)
        link_reference_status = "invalid"
        title_allowed = False
        if not paragraph_open and indentation <= 3:
            link_reference_status, title_allowed = (
                commonmark_link_reference_definition_status(
                    visible[content_start:]
                )
            )
        if link_reference_status == "complete":
            link_reference_title_allowed = title_allowed
            paragraph_open = False
            lines.append((offset, line, ""))
            continue
        elif link_reference_status == "incomplete":
            link_reference_candidate = visible[content_start:]
            lines.append((offset, line, visible))
            link_reference_candidate_line_indexes = [len(lines) - 1]
            paragraph_open = True
            continue
        elif ASCII_BLANK_RE.fullmatch(visible):
            paragraph_open = False
        elif (
            MARKDOWN_HEADING_RE.fullmatch(visible) is not None
            or SETEXT_UNDERLINE_RE.fullmatch(visible) is not None
            or commonmark_nonparagraph_block_line(
                visible,
                paragraph_open,
            )
        ):
            paragraph_open = False
        else:
            paragraph_open = True
        lines.append((offset, line, visible))
    return lines


def parse_defect_sections_text(
    content: str,
    label: str,
    errors: list[str],
) -> dict[str, str]:
    sections: dict[str, str] = {}
    boundaries_by_offset: dict[int, tuple[int, str, str, int]] = {}
    visible_lines = markdown_visible_lines(content)
    for index, (offset, raw_line, visible) in enumerate(visible_lines):
        heading = MARKDOWN_HEADING_RE.fullmatch(visible)
        canonical = (
            DEFECT_HEADING_RE.fullmatch(raw_line)
            if raw_line == visible
            else None
        )
        numbered_top_level_heading = (
            heading is not None
            and len(heading.group("marks")) <= 3
            and any(
                character.isdigit()
                for character in (heading.group("title") or "")
            )
        )
        if (
            DEFECT_LIKE_HEADING_RE.match(visible)
            or ARBITRARY_TRACKING_HEADING_RE.match(visible)
            or RECORD_SEPARATOR_HEADING_RE.match(visible)
            or numbered_top_level_heading
        ) and canonical is None:
            errors.append(
                f"{label} runtime-defect inbox has noncanonical "
                f"defect-like heading {raw_line!r}; expected "
                "'### LOC-020 — Title' or '### RT-ED-008 — Title' form"
            )
        if heading is not None and len(heading.group("marks")) <= 3:
            boundaries_by_offset[offset] = (
                offset,
                raw_line,
                visible,
                len(heading.group("marks")),
            )
        setext = SETEXT_UNDERLINE_RE.fullmatch(visible)
        if setext is None or index == 0:
            continue
        (
            previous_offset,
            previous_raw,
            previous_visible,
        ) = visible_lines[index - 1]
        if (
            not previous_visible.strip()
            or MARKDOWN_HEADING_RE.fullmatch(previous_visible) is not None
            or SETEXT_INELIGIBLE_RE.match(previous_visible) is not None
        ):
            continue
        if (
            DEFECT_LIKE_TEXT_RE.match(previous_visible)
            or ARBITRARY_TRACKING_TEXT_RE.match(previous_visible)
            or RECORD_SEPARATOR_TEXT_RE.match(previous_visible)
            or any(character.isdigit() for character in previous_visible)
        ):
            errors.append(
                f"{label} runtime-defect inbox has noncanonical "
                f"defect-like heading {previous_raw!r}; expected "
                "'### LOC-020 — Title' or '### RT-ED-008 — Title' form"
            )
        level = 1 if setext.group("marker").startswith("=") else 2
        boundaries_by_offset[previous_offset] = (
            previous_offset,
            previous_raw,
            previous_visible,
            level,
        )

    boundaries = sorted(boundaries_by_offset.values())
    for index, (start, raw_heading, _, _) in enumerate(boundaries):
        match = DEFECT_HEADING_RE.fullmatch(raw_heading)
        if match is None:
            continue
        defect_id = match.group("id")
        end = (
            boundaries[index + 1][0]
            if index + 1 < len(boundaries)
            else len(content)
        )
        if defect_id in sections:
            errors.append(
                f"{label} runtime-defect inbox repeats heading {defect_id}"
            )
            continue
        sections[defect_id] = content[start:end]
    return sections


def parse_defect_sections(
    path: pathlib.Path,
    label: str,
    errors: list[str],
) -> dict[str, str]:
    try:
        content = path.read_text()
    except OSError as error:
        errors.append(f"cannot read {label} runtime-defect inbox at {path}: {error}")
        return {}
    return parse_defect_sections_text(content, label, errors)


def validate_future_source_record(
    defect_id: str,
    state: str,
    section: str,
    errors: list[str],
) -> None:
    if state == "intake-needs-evidence":
        return
    lines = [
        visible
        for _, _, visible in markdown_visible_lines(section)
    ]
    bullet_blocks: list[tuple[str, str]] = []
    current_block: list[str] = []

    def append_current_block() -> None:
        if not current_block:
            return
        first = re.sub(r"^[-*+][ \t]+", "", current_block[0])
        if ":" not in first:
            return
        raw_label, first_value = first.split(":", 1)
        label = raw_label.strip().strip("*").strip()
        body = " ".join(
            [first_value.strip(), *(line.strip() for line in current_block[1:])]
        ).strip()
        bullet_blocks.append((label.casefold(), body))

    for line in lines:
        if re.match(r"^[-*+][ \t]+", line):
            append_current_block()
            current_block = [line]
        elif current_block and (not line or line[0].isspace()):
            current_block.append(line)
        else:
            append_current_block()
            current_block = []
    append_current_block()

    editor_only_labels = {
        "editor sha",
        "exact editor checkpoint",
        "exact editor provenance",
        "exact editor base and branch",
        "editor intake checkpoint",
    }
    runtime_only_labels = {
        "runtime sha",
        "runtime pin",
        "exact runtime pin",
        "exact runtime and c++ reference pins",
    }
    combined_labels = {"exact editor/runtime checkpoint"}

    def editor_labeled_body_has_sha(label: str, body: str) -> bool:
        shas = set(FULL_SHA_RE.findall(body))
        if not shas:
            return False
        if label == "exact editor provenance":
            return EDITOR_PROVENANCE_SHA_RE.search(body) is not None
        return True

    def has_editor_sha() -> bool:
        return any(
            (
                label in editor_only_labels
                and editor_labeled_body_has_sha(label, block)
            )
            or (
                label in combined_labels
                and len(set(FULL_SHA_RE.findall(block))) >= 2
            )
            for label, block in bullet_blocks
        )

    def has_runtime_sha() -> bool:
        return any(
            (
                label in runtime_only_labels
                and FULL_SHA_RE.search(block) is not None
            )
            or (
                label in combined_labels
                and len(set(FULL_SHA_RE.findall(block))) >= 2
            )
            for label, block in bullet_blocks
        )

    def has_command() -> bool:
        return any(
            label in COMMAND_EVIDENCE_LABELS
            and any(code.strip() for code in re.findall(r"`([^`\r\n]+)`", body))
            for label, body in bullet_blocks
        )

    def block_has_evidence() -> bool:
        return any(
            label in RESULT_EVIDENCE_LABELS and bool(body)
            for label, body in bullet_blocks
        )

    missing: list[str] = []
    if not (
        any(EDITOR_SHA_BULLET_RE.fullmatch(line) for line in lines)
        or has_editor_sha()
    ):
        missing.append("a separately labeled full Editor SHA")
    if not (
        any(RUNTIME_SHA_BULLET_RE.fullmatch(line) for line in lines)
        or has_runtime_sha()
    ):
        missing.append("a separately labeled full Runtime SHA")
    command_matches = (
        COMMAND_BULLET_RE.fullmatch(line)
        for line in lines
    )
    if not (
        any(
            match is not None and bool(match.group("code").strip())
            for match in command_matches
        )
        or has_command()
    ):
        missing.append("a labeled command bullet with nonempty inline code")
    if not (
        any(EVIDENCE_BULLET_RE.fullmatch(line) for line in lines)
        or block_has_evidence()
    ):
        missing.append(
            "a labeled result/evidence/observation/failure/deficiency/"
            "classification bullet"
        )
    if missing:
        errors.append(
            f"{defect_id} committed inbox source record lacks "
            f"{', '.join(missing)}; state must be intake-needs-evidence"
        )


def normalized_source_record(section: str) -> str:
    """Remove canonical heading anchors that carry no defect evidence."""
    visible_lines = markdown_visible_lines(section)
    normalized_parts: list[str] = []
    for index, (offset, raw, visible) in enumerate(visible_lines):
        end = (
            visible_lines[index + 1][0]
            if index + 1 < len(visible_lines)
            else len(section)
        )
        if raw == visible and DEFECT_ANCHOR_RE.fullmatch(visible) is not None:
            continue
        normalized_parts.append(section[offset:end])
    normalized = "".join(normalized_parts)
    return normalized.rstrip() + "\n"


def parse_ledger_children(
    path: pathlib.Path,
    errors: list[str],
) -> tuple[dict[str, set[str]], dict[str, str]]:
    try:
        ledger = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        errors.append(f"cannot read parity ledger at {path}: {error}")
        return {}, {}
    if not isinstance(ledger, dict) or not isinstance(ledger.get("rows"), list):
        errors.append("parity ledger rows must be a list")
        return {}, {}

    formal: dict[str, set[str]] = collections.defaultdict(set)
    assertions: dict[str, str] = {}
    for parent in ledger["rows"]:
        if not isinstance(parent, dict):
            errors.append("parity ledger contains a non-object row")
            continue
        children = parent.get("children", [])
        if not isinstance(children, list):
            errors.append(
                f"parity ledger row {parent.get('id', '<missing>')} children "
                "must be a list"
            )
            continue
        for child in children:
            if not isinstance(child, dict):
                errors.append("parity ledger contains a non-object child")
                continue
            child_id = str(child.get("id", ""))
            if CHILD_RE.fullmatch(child_id) is None:
                errors.append(f"parity ledger has invalid child id {child_id!r}")
                continue
            if child_id in assertions:
                errors.append(f"parity ledger repeats child id {child_id}")
                continue
            assertion = child.get("assertion")
            if not isinstance(assertion, str):
                errors.append(f"parity ledger child {child_id} has no assertion")
                assertion = ""
            assertions[child_id] = assertion
            dependency_ids: list[str] = []
            for field in ("runtimeDependencies", "runtimeDefects"):
                dependencies = child.get(field, [])
                if not isinstance(dependencies, list):
                    errors.append(
                        f"parity ledger child {child_id} {field} must be a list"
                    )
                    continue
                for dependency in dependencies:
                    if not isinstance(dependency, dict):
                        errors.append(
                            f"parity ledger child {child_id} has a non-object "
                            f"entry in {field}"
                        )
                        continue
                    dependency_id = str(dependency.get("id", ""))
                    if not dependency_id:
                        errors.append(
                            f"parity ledger child {child_id} has an empty "
                            f"id in {field}"
                        )
                        continue
                    dependency_ids.append(dependency_id)
                    formal[dependency_id].add(child_id)
            duplicates = duplicate_values(dependency_ids)
            if duplicates:
                errors.append(
                    f"parity ledger child {child_id} repeats structured "
                    f"runtime links: {', '.join(duplicates)}"
                )
    return dict(formal), assertions


def validate_source_record_contract(
    rows: list[dict[str, Any]],
    consumed_sections: dict[str, str],
    formal_by_dependency: dict[str, set[str]],
    assertions: dict[str, str],
    errors: list[str],
) -> None:
    atlas_ids = {str(row.get("id", "")) for row in rows}
    consumed_ids = set(consumed_sections)
    missing = sorted(consumed_ids - atlas_ids)
    fabricated = sorted(atlas_ids - consumed_ids)
    if missing:
        errors.append(
            "atlas is missing consumed Editor inbox records: "
            + ", ".join(missing)
        )
    if fabricated:
        errors.append(
            "atlas has no exact consumed Editor inbox heading for: "
            + ", ".join(fabricated)
        )

    for dependency_id in sorted(formal_by_dependency):
        if dependency_id not in consumed_ids:
            errors.append(
                f"parity ledger runtime dependency {dependency_id} has no "
                "consumed Editor inbox record"
            )
    for row in rows:
        defect_id = str(row.get("id", ""))
        if (
            defect_id not in BASELINE_IDS
            and (
                LOCAL_ID_RE.fullmatch(defect_id)
                or RUNTIME_ID_RE.fullmatch(defect_id)
            )
        ):
            section = consumed_sections.get(defect_id)
            if section is not None:
                validate_future_source_record(
                    defect_id,
                    str(row.get("state", "")),
                    section,
                    errors,
                )
        actual_formal = {
            str(value) for value in row.get("formal_children", [])
        }
        expected_formal = formal_by_dependency.get(defect_id, set())
        if actual_formal != expected_formal:
            missing_children = ", ".join(
                sorted(expected_formal - actual_formal)
            ) or "none"
            extra_children = ", ".join(
                sorted(actual_formal - expected_formal)
            ) or "none"
            errors.append(
                f"{defect_id} formal_children do not match parity-ledger "
                "runtimeDependencies/runtimeDefects; "
                f"missing: {missing_children}; "
                f"extra: {extra_children}"
            )
        for child_id in row.get("candidate_children", []):
            child_id = str(child_id)
            assertion = assertions.get(child_id)
            if assertion is None:
                errors.append(
                    f"{defect_id} candidate child {child_id} is absent from "
                    "the parity ledger"
                )
                continue
            mentions = {
                match.group("id")
                for match in DEFECT_MENTION_RE.finditer(assertion)
            }
            if defect_id not in mentions:
                errors.append(
                    f"{defect_id} candidate child {child_id} assertion does "
                    f"not contain the exact defect id {defect_id}"
                )


def validate_newest_inbox(
    inbox: dict[str, Any],
    consumed_sections: dict[str, str],
    newest_source_root: pathlib.Path,
    errors: list[str],
) -> None:
    newest_defects_path = source_path(
        newest_source_root,
        str(inbox.get("runtime_defects_path", "")),
    )
    newest_ledger_path = source_path(
        newest_source_root,
        str(inbox.get("parity_ledger_path", "")),
    )
    safe_paths: dict[pathlib.Path, bool] = {}
    for path, field in (
        (newest_defects_path, "newest_available_runtime_defects_sha256"),
        (newest_ledger_path, "newest_available_parity_ledger_sha256"),
    ):
        safe = validate_file_beneath_root(
            newest_source_root,
            path,
            f"newest Editor source {path.name}",
            errors,
        )
        safe_paths[path] = safe
        if not safe:
            continue
        try:
            actual = hashlib.sha256(path.read_bytes()).hexdigest()
        except OSError as error:
            errors.append(f"cannot hash newest Editor source {path}: {error}")
            continue
        if actual != inbox.get(field):
            errors.append(
                f"inbox {field} is {inbox.get(field)!r}; newest Editor "
                f"source hash is {actual}"
            )

    newest_sections = (
        parse_defect_sections(
            newest_defects_path,
            "newest",
            errors,
        )
        if safe_paths.get(newest_defects_path, False)
        else {}
    )
    consumed_ids = set(consumed_sections)
    newest_ids = set(newest_sections)
    invalid_future_ids = sorted(
        defect_id
        for defect_id in newest_ids - BASELINE_IDS
        if (
            (
                defect_id.startswith("LOC-")
                and (
                    LOCAL_ID_RE.fullmatch(defect_id) is None
                    or int(defect_id.removeprefix("LOC-")) < 20
                )
            )
            or (
                defect_id.startswith("RT-ED-")
                and (
                    RUNTIME_ID_RE.fullmatch(defect_id) is None
                    or int(defect_id.removeprefix("RT-ED-")) < 8
                )
            )
        )
    )
    if invalid_future_ids:
        errors.append(
            "newest Editor inbox has invalid future defect ids: "
            + ", ".join(invalid_future_ids)
        )
    deleted = sorted(consumed_ids - newest_ids)
    if deleted:
        errors.append(
            "newest Editor inbox deletes consumed records: "
            + ", ".join(deleted)
        )
    changed_records = {
        defect_id
        for defect_id, section in newest_sections.items()
        if normalized_source_record(consumed_sections.get(defect_id, ""))
        != normalized_source_record(section)
    }
    expected_unconsumed = inbox.get("unconsumed_records")
    if expected_unconsumed != len(changed_records):
        errors.append(
            f"inbox unconsumed_records is {expected_unconsumed}; exact "
            "new/changed canonical record count is "
            f"{len(changed_records)}"
        )


def validate_prior_id_ratchet(
    repo_root: pathlib.Path,
    current_rows: list[dict[str, Any]],
    errors: list[str],
) -> dict[str, Any] | None:
    result = run_git(
        repo_root,
        ["show", "origin/main:docs/editor-next-runtime-defect-atlas.toml"],
    )
    if result.returncode != 0:
        errors.append(
            "cannot read prior origin/main runtime-defect atlas: "
            + result.stderr.strip()
        )
        return None
    try:
        prior = tomllib.loads(result.stdout)
    except tomllib.TOMLDecodeError as error:
        errors.append(f"cannot parse prior origin/main runtime-defect atlas: {error}")
        return None
    prior_rows = {
        str(row.get("id", "")): row
        for row in prior.get("defect", [])
        if isinstance(row, dict)
    }
    current_by_id = {
        str(row.get("id", "")): row
        for row in current_rows
        if isinstance(row, dict)
    }
    prior_ids = set(prior_rows)
    current_ids = set(current_by_id)
    deleted = sorted(prior_ids - current_ids)
    if deleted:
        errors.append(
            "atlas deletes previously accepted origin/main defect ids: "
            + ", ".join(deleted)
        )
    for defect_id in sorted(prior_ids & current_ids):
        prior_row = prior_rows[defect_id]
        current_row = current_by_id[defect_id]
        prior_history = prior_row.get("history", [])
        current_history = current_row.get("history", [])
        if not isinstance(prior_history, list) or not isinstance(
            current_history, list
        ):
            errors.append(
                f"{defect_id} cannot ratchet origin/main history because "
                "one history is not a list"
            )
            continue
        if current_history[: len(prior_history)] != prior_history:
            errors.append(
                f"{defect_id} origin/main history is not an exact prefix "
                "of current history"
            )
        prior_state = str(prior_row.get("state", ""))
        current_state = str(current_row.get("state", ""))
        if len(current_history) < len(prior_history):
            errors.append(
                f"{defect_id} current state {current_state!r} regresses "
                f"origin/main state {prior_state!r} by trimming history"
            )
        elif (
            current_history[: len(prior_history)] == prior_history
            and len(current_history) == len(prior_history)
            and current_state != prior_state
        ):
            errors.append(
                f"{defect_id} current state {current_state!r} regresses "
                f"origin/main state {prior_state!r} without appending history"
            )
        prior_revisions = prior_row.get("revisions")
        current_revisions = current_row.get("revisions")
        if not isinstance(prior_revisions, dict) or not isinstance(
            current_revisions, dict
        ):
            continue
        for field in sorted(IMMUTABLE_REVISION_KEYS):
            prior_revision = prior_revisions.get(field)
            if (
                isinstance(prior_revision, str)
                and SHA_RE.fullmatch(prior_revision)
                and current_revisions.get(field) != prior_revision
            ):
                errors.append(
                    f"{defect_id} immutable revision {field} changed "
                    f"from {prior_revision!r} to "
                    f"{current_revisions.get(field)!r}"
                )
    return prior


def validate_consumed_intake_delta(
    prior_atlas: dict[str, Any] | None,
    current_rows: list[dict[str, Any]],
    current_sections: dict[str, str],
    editor_repo: pathlib.Path,
    errors: list[str],
) -> None:
    if (
        not isinstance(prior_atlas, dict)
        or prior_atlas.get("schema") != SCHEMA
        or prior_atlas.get("version") != 2
    ):
        return
    prior_inbox = prior_atlas.get("inbox")
    if not isinstance(prior_inbox, dict):
        errors.append("prior v2 atlas has no inbox table")
        return
    prior_ref = str(prior_inbox.get("last_consumed_editor_ref", ""))
    prior_path = str(prior_inbox.get("runtime_defects_path", ""))
    if SHA_RE.fullmatch(prior_ref) is None:
        errors.append(
            "prior v2 inbox last_consumed_editor_ref is not a full SHA"
        )
        return
    if prior_path != EXPECTED_INBOX["runtime_defects_path"]:
        errors.append(
            f"prior v2 inbox runtime_defects_path is {prior_path!r}; "
            f"expected {EXPECTED_INBOX['runtime_defects_path']!r}"
        )
        return
    prior_blob = git_blob(
        editor_repo,
        prior_ref,
        prior_path,
        "prior v2 consumed runtime-defect inbox",
        errors,
    )
    if prior_blob is None:
        return
    try:
        prior_content = prior_blob.decode("utf-8")
    except UnicodeDecodeError as error:
        errors.append(
            "prior v2 consumed runtime-defect inbox is not UTF-8: "
            f"{error}"
        )
        return
    prior_sections = parse_defect_sections_text(
        prior_content,
        "prior v2 consumed",
        errors,
    )
    rows_by_id = {
        str(row.get("id", "")): row
        for row in current_rows
        if isinstance(row, dict)
    }
    changed_records = sorted(
        defect_id
        for defect_id, section in current_sections.items()
        if normalized_source_record(prior_sections.get(defect_id, ""))
        != normalized_source_record(section)
    )
    for defect_id in changed_records:
        row = rows_by_id.get(defect_id)
        if row is None:
            continue
        validate_future_source_record(
            defect_id,
            str(row.get("state", "")),
            current_sections[defect_id],
            errors,
        )


def literal_revision(revisions: Any, field: str) -> str | None:
    if not isinstance(revisions, dict):
        return None
    value = revisions.get(field)
    if isinstance(value, str) and SHA_RE.fullmatch(value):
        return value
    return None


def validate_landed_repair_provenance(
    rows: list[dict[str, Any]],
    errors: list[str],
) -> None:
    by_id = {
        str(row.get("id", "")): row
        for row in rows
        if isinstance(row, dict)
    }
    for defect_id, expected in LANDED_REPAIR_PROVENANCE.items():
        row = by_id.get(defect_id)
        if row is None:
            continue
        revisions = row.get("revisions")
        for field, expected_revision in expected.items():
            actual = (
                revisions.get(field)
                if isinstance(revisions, dict)
                else None
            )
            if actual != expected_revision:
                errors.append(
                    f"{defect_id} revisions.{field} is {actual!r}; "
                    f"landed provenance ratchet requires {expected_revision!r}"
                )


def validate_revision_provenance(
    atlas: dict[str, Any],
    rows: list[dict[str, Any]],
    runtime_repo: pathlib.Path,
    editor_repo: pathlib.Path,
    errors: list[str],
) -> None:
    current_runtime = str(atlas.get("editor_consumed_runtime_ref", ""))
    top_runtime_revisions = {
        field: str(atlas.get(field, ""))
        for field in ("editor_consumed_runtime_ref", "investigation_base_ref")
    }
    resolved_runtime: dict[tuple[str, str], str] = {}
    for field, revision in top_runtime_revisions.items():
        if SHA_RE.fullmatch(revision):
            resolved = git_commit(
                runtime_repo,
                revision,
                "runtime",
                f"atlas.{field}",
                errors,
            )
            if resolved is not None:
                resolved_runtime[("atlas", field)] = resolved
    source_snapshot_ref = str(atlas.get("source_snapshot_ref", ""))
    resolved_source_snapshot: str | None = None
    if SHA_RE.fullmatch(source_snapshot_ref):
        resolved_source_snapshot = git_commit(
            editor_repo,
            source_snapshot_ref,
            "Editor",
            "atlas.source_snapshot_ref",
            errors,
        )
    resolved_current_runtime = resolved_runtime.get(
        ("atlas", "editor_consumed_runtime_ref")
    )
    if (
        resolved_source_snapshot is not None
        and resolved_current_runtime is not None
    ):
        snapshot_gitlink = gitlink_at(
            editor_repo,
            resolved_source_snapshot,
            "atlas.source_snapshot_ref",
            errors,
        )
        if (
            snapshot_gitlink is not None
            and snapshot_gitlink != resolved_current_runtime
        ):
            errors.append(
                "atlas.editor_consumed_runtime_ref does not match "
                "source_snapshot_ref runtime gitlink: "
                f"{resolved_current_runtime} != {snapshot_gitlink}"
            )

    for row in rows:
        defect_id = str(row.get("id", ""))
        owner_class = str(row.get("owner_class", ""))
        revisions = row.get("revisions")
        if not isinstance(revisions, dict):
            continue
        for field in RUNTIME_REVISION_KEYS:
            revision = literal_revision(revisions, field)
            if revision is None:
                continue
            resolved = git_commit(
                runtime_repo,
                revision,
                "runtime",
                f"{defect_id} revisions.{field}",
                errors,
            )
            if resolved is not None:
                resolved_runtime[(defect_id, field)] = resolved

        merged_repair = literal_revision(revisions, "merged_repair_sha")
        merged_repo = editor_repo if owner_class == "editor" else runtime_repo
        merged_repo_label = "Editor" if owner_class == "editor" else "runtime"
        resolved_merged: str | None = None
        if merged_repair is not None:
            resolved_merged = git_commit(
                merged_repo,
                merged_repair,
                merged_repo_label,
                f"{defect_id} revisions.merged_repair_sha",
                errors,
            )

        consumed_superproject = literal_revision(
            revisions, "consumed_superproject_sha"
        )
        resolved_superproject: str | None = None
        if consumed_superproject is not None:
            resolved_superproject = git_commit(
                editor_repo,
                consumed_superproject,
                "Editor",
                f"{defect_id} revisions.consumed_superproject_sha",
                errors,
            )

        consumed_runtime = literal_revision(revisions, "consumed_runtime_sha")
        resolved_consumed_runtime = resolved_runtime.get(
            (defect_id, "consumed_runtime_sha")
        )
        if (
            owner_class != "editor"
            and resolved_merged is not None
            and resolved_consumed_runtime is not None
        ):
            git_is_ancestor(
                runtime_repo,
                resolved_merged,
                resolved_consumed_runtime,
                errors,
                f"{defect_id} merged runtime repair is not an ancestor "
                "of its consumed runtime",
                repo_label="runtime",
            )
        if (
            owner_class == "editor"
            and resolved_merged is not None
            and resolved_superproject is not None
        ):
            git_is_ancestor(
                editor_repo,
                resolved_merged,
                resolved_superproject,
                errors,
                f"{defect_id} merged Editor repair is not an ancestor "
                "of its consumed superproject",
            )
        if (
            consumed_runtime is not None
            and resolved_consumed_runtime is not None
            and resolved_superproject is not None
        ):
            gitlink = gitlink_at(
                editor_repo,
                resolved_superproject,
                f"{defect_id} consumed superproject",
                errors,
            )
            if (
                gitlink is not None
                and gitlink != resolved_consumed_runtime
            ):
                errors.append(
                    f"{defect_id} consumed superproject runtime gitlink is "
                    f"{gitlink}; revisions.consumed_runtime_sha "
                    f"is {resolved_consumed_runtime}"
                )

    resolved_current = resolved_runtime.get(
        ("atlas", "editor_consumed_runtime_ref")
    )
    if resolved_current is None or current_runtime != resolved_current:
        return
    for row in rows:
        defect_id = str(row.get("id", ""))
        for field in (
            "editor_last_consumed_runtime_sha",
            "consumed_runtime_sha",
        ):
            revision = resolved_runtime.get((defect_id, field))
            if revision is None:
                continue
            git_is_ancestor(
                runtime_repo,
                revision,
                resolved_current,
                errors,
                f"{defect_id} revisions.{field} is not an ancestor of "
                "atlas.editor_consumed_runtime_ref",
                repo_label="runtime",
            )


def validate_closed_inbox(
    inbox: Any,
    canonical_branch_tip: str | None,
    errors: list[str],
) -> None:
    if not isinstance(inbox, dict) or inbox.get("unconsumed_records") != 0:
        errors.append("--require-closed requires unconsumed_records = 0")
    if canonical_branch_tip is None:
        errors.append(
            "--require-closed cannot resolve the canonical Editor branch tip"
        )
    elif not isinstance(inbox, dict) or (
        inbox.get("newest_available_editor_ref") != canonical_branch_tip
    ):
        errors.append(
            "--require-closed requires newest_available_editor_ref to "
            "equal the canonical Editor branch tip"
        )


def validate_result(
    defect_id: str,
    layer: str,
    result: Any,
    state: str,
    errors: list[str],
) -> None:
    if not isinstance(result, dict):
        errors.append(f"{defect_id} has no {layer}_result table")
        return
    status = str(result.get("status", ""))
    if status not in RESULT_STATUSES:
        errors.append(f"{defect_id} {layer}_result has invalid status {status!r}")
        return
    if status in {"pending", "not-applicable"}:
        if not str(result.get("reason", "")).strip():
            errors.append(f"{defect_id} {layer}_result {status} has no reason")
    else:
        if not str(result.get("command", "")).strip():
            errors.append(f"{defect_id} {layer}_result {status} has no command")
        if not str(result.get("evidence", "")).strip():
            errors.append(f"{defect_id} {layer}_result {status} has no evidence")
    if state not in EARLY_STATES and status == "pending":
        errors.append(
            f"{defect_id} is {state} but {layer}_result is still pending"
        )


def validate_history(
    defect_id: str,
    state: str,
    history: Any,
    errors: list[str],
    *,
    owner_class: str | None = None,
) -> None:
    if not isinstance(history, list) or not history:
        errors.append(f"{defect_id} has no state history")
        return
    history_states = [str(row.get("state", "")) for row in history]
    if history_states[0] != "reported":
        errors.append(f"{defect_id} history must begin at reported")
    if history_states[-1] != state:
        errors.append(
            f"{defect_id} state {state!r} does not match history tail "
            f"{history_states[-1]!r}"
        )
    for row in history:
        row_state = str(row.get("state", ""))
        if row_state not in STATES:
            errors.append(f"{defect_id} history has invalid state {row_state!r}")
        if not str(row.get("actor", "")).strip():
            errors.append(f"{defect_id} history state {row_state} has no actor")
        if not str(row.get("evidence", "")).strip():
            errors.append(f"{defect_id} history state {row_state} has no evidence")
        if (
            row_state == "orchestrator-verified"
            and row.get("actor") != "independent-orchestrator"
        ):
            errors.append(
                f"{defect_id} orchestrator-verified was not promoted by "
                "independent-orchestrator"
            )
    for previous, current in zip(history_states, history_states[1:]):
        editor_or_artifact_handoff = (
            previous == "qualified"
            and current == "handoff-ready"
            and owner_class in {"editor", "artifact"}
        )
        if (
            current not in TRANSITIONS.get(previous, set())
            and not editor_or_artifact_handoff
        ):
            errors.append(
                f"{defect_id} has illegal state transition {previous} -> {current}"
            )


def validate_children(
    defect_id: str,
    field: str,
    values: Any,
    errors: list[str],
) -> set[str]:
    if not isinstance(values, list):
        errors.append(f"{defect_id} {field} is not a list")
        return set()
    normalized = [str(value) for value in values]
    duplicates = duplicate_values(normalized)
    if duplicates:
        errors.append(
            f"{defect_id} {field} contains duplicates: {', '.join(duplicates)}"
        )
    for value in normalized:
        if CHILD_RE.fullmatch(value) is None:
            errors.append(f"{defect_id} {field} has invalid child {value!r}")
    return set(normalized)


def validate_corrections(
    corrections: dict[str, Any],
    upstream_ref: str,
    pin_content: bool,
    errors: list[str],
) -> int:
    if corrections.get("schema") != CORRECTIONS_SCHEMA:
        errors.append(f"corrections schema must be {CORRECTIONS_SCHEMA}")
    if corrections.get("version") != 1:
        errors.append("corrections version must be 1")
    if corrections.get("source_pin") != upstream_ref:
        errors.append("corrections and atlas pin different upstream refs")
    rows = list(corrections.get("correction", []))
    ids = [str(row.get("id", "")) for row in rows]
    duplicates = duplicate_values(ids)
    if duplicates:
        errors.append(f"duplicate correction ids: {', '.join(duplicates)}")
    actual_ids = set(ids)
    if actual_ids != EXPECTED_CORRECTION_IDS:
        missing = ", ".join(sorted(EXPECTED_CORRECTION_IDS - actual_ids)) or "none"
        extra = ", ".join(sorted(actual_ids - EXPECTED_CORRECTION_IDS)) or "none"
        errors.append(
            "correction ids must be exactly COR-01..COR-12; "
            f"missing: {missing}; extra: {extra}"
        )
    for row in rows:
        correction_id = str(row.get("id", ""))
        if not re.fullmatch(r"COR-\d{2}", correction_id):
            errors.append(f"invalid correction id {correction_id!r}")
        if row.get("status") not in {"open", "resolved", "versioned"}:
            errors.append(f"{correction_id} has invalid correction status")
        if not str(row.get("description", "")).strip():
            errors.append(f"{correction_id} has no description")
        if not str(row.get("resolution", "")).strip():
            errors.append(f"{correction_id} has no resolution")
    canonical_parts = [
        str(row.get(field, ""))
        for row in rows
        for field in ("id", "status", "description", "resolution")
    ]
    content_digest = hashlib.sha256(
        "\0".join(canonical_parts).encode("utf-8")
    ).hexdigest()
    if pin_content and content_digest != EXPECTED_CORRECTIONS_SHA256:
        errors.append(
            f"correction content digest is {content_digest}; "
            f"expected {EXPECTED_CORRECTIONS_SHA256}"
        )
    expected = corrections.get("expected_corrections")
    if expected != len(rows):
        errors.append(
            f"correction count ratchet says {expected}, actual is {len(rows)}"
        )
    return len(rows)


def fixture_digest(row: dict[str, Any]) -> str:
    parts = [
        str(row.get(field, ""))
        for field in ("id", "defect_id", "kind", "status", "driver")
    ]
    for artifact in row.get("stimulus_files", []):
        if isinstance(artifact, dict):
            parts.extend(
                str(artifact.get(field, ""))
                for field in ("root", "path", "sha256")
            )
    canonical = "\0".join(parts)
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def validate_stimulus_files(
    fixture_id: str,
    status: str,
    artifacts: Any,
    roots: dict[str, pathlib.Path | None],
    verify_files: bool,
    errors: list[str],
    *,
    git_sources: dict[str, tuple[pathlib.Path, str, str]] | None = None,
) -> None:
    if not isinstance(artifacts, list):
        errors.append(f"fixture {fixture_id} stimulus_files must be a list")
        return
    if status == "qualified" and not artifacts:
        errors.append(
            f"fixture {fixture_id} is qualified but has no hashed stimulus files"
        )
    seen: set[tuple[str, str]] = set()
    for artifact in artifacts:
        if not isinstance(artifact, dict):
            errors.append(f"fixture {fixture_id} has a non-table stimulus file")
            continue
        if set(artifact) != {"root", "path", "sha256"}:
            errors.append(
                f"fixture {fixture_id} stimulus file keys must be exactly "
                "root, path, sha256"
            )
            continue
        root_name = str(artifact.get("root", ""))
        relative_text = str(artifact.get("path", ""))
        expected_hash = str(artifact.get("sha256", ""))
        if root_name not in STIMULUS_ROOTS:
            errors.append(
                f"fixture {fixture_id} has unknown stimulus root {root_name!r}"
            )
            continue
        relative = pathlib.PurePosixPath(relative_text)
        if (
            not relative_text
            or relative.is_absolute()
            or ".." in relative.parts
        ):
            errors.append(
                f"fixture {fixture_id} has unsafe stimulus path {relative_text!r}"
            )
            continue
        key = (root_name, relative_text)
        if key in seen:
            errors.append(
                f"fixture {fixture_id} repeats stimulus file "
                f"{root_name}:{relative_text}"
            )
        seen.add(key)
        if SHA256_RE.fullmatch(expected_hash) is None:
            errors.append(
                f"fixture {fixture_id} stimulus {root_name}:{relative_text} "
                "has invalid sha256"
            )
            continue
        if not verify_files:
            continue
        root = roots.get(root_name)
        if root is None:
            errors.append(
                f"fixture {fixture_id} cannot resolve stimulus root {root_name}"
            )
            continue
        path = root / pathlib.Path(relative_text)
        if not validate_file_beneath_root(
            root,
            path,
            f"fixture {fixture_id} stimulus {root_name}:{relative_text}",
            errors,
        ):
            continue
        actual_bytes = path.read_bytes()
        actual_hash = hashlib.sha256(actual_bytes).hexdigest()
        if actual_hash != expected_hash:
            errors.append(
                f"fixture {fixture_id} stimulus {root_name}:{relative_text} "
                f"hash is {actual_hash}; registry records {expected_hash}"
            )
        git_source = (git_sources or {}).get(root_name)
        if git_source is None:
            errors.append(
                f"fixture {fixture_id} stimulus {root_name}:{relative_text} "
                "has no pinned Git source"
            )
            continue
        git_repo, revision, pin_name = git_source
        expected_bytes = git_blob(
            git_repo,
            revision,
            relative_text,
            (
                f"fixture {fixture_id} stimulus "
                f"{root_name}:{relative_text}"
            ),
            errors,
        )
        if expected_bytes is not None and actual_bytes != expected_bytes:
            errors.append(
                f"fixture {fixture_id} stimulus {root_name}:{relative_text} "
                f"does not match pinned {pin_name} Git blob"
            )


def validate_cpp_probe_provenance(
    cpp_probe: pathlib.Path,
    repo_root: pathlib.Path,
    upstream_ref: str,
    errors: list[str],
) -> None:
    stamp_path = pathlib.Path(f"{cpp_probe}.provenance")
    if not stamp_path.is_file():
        errors.append(f"C++ probe provenance stamp is missing at {stamp_path}")
        return
    fields: dict[str, str] = {}
    for line in stamp_path.read_text().splitlines():
        key, separator, value = line.partition("=")
        if not separator or not key or key in fields:
            errors.append(f"C++ probe provenance stamp has invalid line {line!r}")
            continue
        fields[key] = value
    required = {
        "upstream_ref",
        "compiler",
        "flags",
        "source",
        "source_sha256",
        "executable_sha256",
    }
    if set(fields) != required:
        errors.append(
            "C++ probe provenance keys must be exactly "
            + ", ".join(sorted(required))
        )
        return
    if fields["upstream_ref"] != upstream_ref:
        errors.append(
            f"C++ probe provenance pins {fields['upstream_ref']}; "
            f"atlas pins {upstream_ref}"
        )
    if fields["flags"] != "-std=c++20 -Wall -Wextra -Werror":
        errors.append("C++ probe provenance records unexpected compiler flags")
    expected_source = "tools/editor-next-runtime-defects/cpp_probe/registry.cpp"
    if fields["source"] != expected_source:
        errors.append(
            f"C++ probe provenance source is {fields['source']!r}; "
            f"expected {expected_source!r}"
        )
    source_path = repo_root / expected_source
    if source_path.is_file():
        actual_source_hash = hashlib.sha256(source_path.read_bytes()).hexdigest()
        if fields["source_sha256"] != actual_source_hash:
            errors.append(
                "C++ probe provenance source hash does not match registry.cpp"
            )
    else:
        errors.append(f"C++ probe source does not exist at {source_path}")
    actual_executable_hash = hashlib.sha256(cpp_probe.read_bytes()).hexdigest()
    if fields["executable_sha256"] != actual_executable_hash:
        errors.append(
            "C++ probe provenance executable hash does not match the executable"
        )


def run_cpp_probe(
    cpp_probe: pathlib.Path,
    repo_root: pathlib.Path,
    upstream_ref: str,
    verify_provenance: bool,
    errors: list[str],
) -> set[str]:
    if not cpp_probe.is_file():
        errors.append(f"C++ probe executable does not exist at {cpp_probe}")
        return set()
    if verify_provenance:
        validate_cpp_probe_provenance(
            cpp_probe,
            repo_root,
            upstream_ref,
            errors,
        )
    result = subprocess.run(
        [str(cpp_probe), "--list"],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        errors.append(
            f"C++ probe --list failed ({result.returncode}): "
            f"{result.stderr.strip()}"
        )
        return set()
    values = [line.strip() for line in result.stdout.splitlines() if line.strip()]
    duplicates = duplicate_values(values)
    if duplicates:
        errors.append(
            f"C++ probe --list returned duplicates: {', '.join(duplicates)}"
        )
    return set(values)


def validate_fixtures(
    fixtures: dict[str, Any],
    upstream_ref: str,
    atlas_fixtures: dict[str, str],
    atlas_defect_ids: set[str],
    atlas_states: dict[str, str],
    cpp_probe: pathlib.Path | None,
    repo_root: pathlib.Path,
    stimulus_roots: dict[str, pathlib.Path | None],
    stimulus_git_sources: dict[str, tuple[pathlib.Path, str, str]],
    verify_stimulus_files: bool,
    errors: list[str],
) -> tuple[int, dict[str, dict[str, Any]]]:
    if fixtures.get("schema") != FIXTURES_SCHEMA:
        errors.append(f"fixtures schema must be {FIXTURES_SCHEMA}")
    if fixtures.get("version") != 1:
        errors.append("fixtures version must be 1")
    if fixtures.get("upstream_ref") != upstream_ref:
        errors.append("fixtures and atlas pin different upstream refs")

    rows = list(fixtures.get("fixture", []))
    ids = [str(row.get("id", "")) for row in rows]
    defect_ids = [str(row.get("defect_id", "")) for row in rows]
    duplicate_ids = duplicate_values(ids)
    if duplicate_ids:
        errors.append(f"duplicate fixture registry ids: {', '.join(duplicate_ids)}")
    duplicate_defects = duplicate_values(defect_ids)
    if duplicate_defects:
        errors.append(
            "duplicate fixture registry defect ids: "
            f"{', '.join(duplicate_defects)}"
        )

    registry_fixtures: dict[str, str] = {}
    fixture_rows: dict[str, dict[str, Any]] = {}
    expected_cpp_probe_ids: set[str] = set()
    for row in rows:
        fixture_id = str(row.get("id", ""))
        defect_id = str(row.get("defect_id", ""))
        if not fixture_id:
            errors.append("fixture registry row has an empty id")
        elif fixture_id not in registry_fixtures:
            registry_fixtures[fixture_id] = defect_id
            fixture_rows[fixture_id] = row
        if defect_id not in atlas_defect_ids:
            errors.append(
                f"fixture {fixture_id or '<empty>'} has invalid defect_id "
                f"{defect_id!r}"
            )
        if row.get("kind") not in FIXTURE_KINDS:
            errors.append(f"fixture {fixture_id} has invalid kind")
        status = str(row.get("status", ""))
        if status not in FIXTURE_STATUSES:
            errors.append(f"fixture {fixture_id} has invalid status")
        driver = str(row.get("driver", "")).strip()
        if not driver:
            errors.append(f"fixture {fixture_id} has no driver")
        if driver == "cpp_probe/registry.cpp":
            expected_cpp_probe_ids.add(fixture_id)
        if status in {"implemented", "qualified"} and (
            driver.startswith("pending:") or driver.startswith("evidence-only:")
        ):
            errors.append(
                f"fixture {fixture_id} is {status} but uses non-executable "
                f"driver {driver!r}"
            )
        validate_stimulus_files(
            fixture_id,
            status,
            row.get("stimulus_files", []),
            stimulus_roots,
            verify_stimulus_files,
            errors,
            git_sources=stimulus_git_sources,
        )
        atlas_state = atlas_states.get(defect_id, "")
        if status == "implemented" and atlas_state not in IMPLEMENTED_FIXTURE_STATES:
            errors.append(
                f"fixture {fixture_id} is implemented but atlas row {defect_id} "
                f"is {atlas_state or '<missing>'}"
            )
        if status == "qualified" and atlas_state not in QUALIFIED_FIXTURE_STATES:
            errors.append(
                f"fixture {fixture_id} is qualified but atlas row {defect_id} "
                f"is {atlas_state or '<missing>'}"
            )

    registered_ids = set(registry_fixtures)
    atlas_ids = set(atlas_fixtures)
    missing = sorted(atlas_ids - registered_ids)
    extra = sorted(registered_ids - atlas_ids)
    if missing:
        errors.append(
            f"atlas fixture ids missing from registry: {', '.join(missing)}"
        )
    if extra:
        errors.append(
            f"fixture registry has ids absent from atlas: {', '.join(extra)}"
        )
    for fixture_id in sorted(atlas_ids & registered_ids):
        expected_defect = atlas_fixtures[fixture_id]
        actual_defect = registry_fixtures[fixture_id]
        if actual_defect != expected_defect:
            errors.append(
                f"fixture {fixture_id} maps to {actual_defect}; "
                f"atlas assigns it to {expected_defect}"
            )

    expected = fixtures.get("expected_fixtures")
    if expected != len(rows):
        errors.append(
            f"fixture count ratchet says {expected}, actual is {len(rows)}"
        )
    if cpp_probe is not None:
        actual_cpp_probe_ids = run_cpp_probe(
            cpp_probe,
            repo_root,
            upstream_ref,
            verify_stimulus_files,
            errors,
        )
        if actual_cpp_probe_ids != expected_cpp_probe_ids:
            missing = ", ".join(
                sorted(expected_cpp_probe_ids - actual_cpp_probe_ids)
            ) or "none"
            extra = ", ".join(
                sorted(actual_cpp_probe_ids - expected_cpp_probe_ids)
            ) or "none"
            errors.append(
                "C++ probe registry must exactly match fixtures driven by "
                "cpp_probe/registry.cpp; "
                f"missing: {missing}; extra: {extra}"
            )
    return len(rows), fixture_rows


def validate_pending_value(
    defect_id: str,
    field: str,
    value: Any,
    state: str,
    errors: list[str],
    *,
    pending_allowed_late: bool = False,
) -> None:
    if isinstance(value, str):
        normalized = value.strip()
        pending = normalized.lower().startswith("pending")
        if not normalized:
            errors.append(f"{defect_id} closure field {field} is empty")
        elif pending:
            pending_reason = normalized.partition(":")[2].strip()
            if not normalized.lower().startswith("pending:") or not pending_reason:
                errors.append(
                    f"{defect_id} closure field {field} has no pending reason"
                )
            elif state not in EARLY_STATES and not pending_allowed_late:
                errors.append(
                    f"{defect_id} is {state} but closure field {field} is pending"
                )
        return
    if isinstance(value, dict) and value.get("status") == "pending":
        if not str(value.get("reason", "")).strip():
            errors.append(f"{defect_id} closure field {field} has no pending reason")
        if state not in EARLY_STATES and not pending_allowed_late:
            errors.append(
                f"{defect_id} is {state} but closure field {field} is pending"
            )
        return
    if value is None:
        errors.append(f"{defect_id} has no closure field {field}")
    else:
        errors.append(f"{defect_id} closure field {field} must be a string")


def validate_revision(
    defect_id: str,
    field: str,
    value: Any,
    state: str,
    no_repair_path: bool,
    errors: list[str],
    *,
    editor_consumed_path: bool = False,
    handoff_ready_path: bool = False,
) -> None:
    pending_allowed_late = (
        field == "merged_repair_sha"
        and state
        not in {"orchestrator-verified", "handoff-ready", "editor-consumed", "closed"}
    ) or (
        field == "consumed_runtime_sha"
        and state != "editor-consumed"
        and (
            state != "closed"
            or (handoff_ready_path and not editor_consumed_path)
        )
    ) or (
        field == "consumed_superproject_sha"
        and state != "editor-consumed"
        and (
            state != "closed"
            or (handoff_ready_path and not editor_consumed_path)
        )
    )
    if no_repair_path and (
        field == "merged_repair_sha"
        or (
            field in {"consumed_runtime_sha", "consumed_superproject_sha"}
            and not editor_consumed_path
        )
    ):
        pending_allowed_late = True
    if isinstance(value, str) and SHA_RE.fullmatch(value):
        return
    if isinstance(value, dict) and value.get("status") == "pending":
        validate_pending_value(
            defect_id,
            f"revisions.{field}",
            value,
            state,
            errors,
            pending_allowed_late=pending_allowed_late,
        )
        return
    errors.append(
        f"{defect_id} revisions.{field} must be a full SHA or pending with a reason"
    )


def validate_verification(
    defect_id: str,
    field: str,
    value: Any,
    state: str,
    errors: list[str],
) -> None:
    if not isinstance(value, dict):
        errors.append(f"{defect_id} {field} must be a table")
        return
    status = str(value.get("status", ""))
    if status == "pending":
        pending_allowed_late = (
            field == "executor_verification"
            and state in {"qualified", "mapped"}
        ) or (
            field == "orchestrator_verification"
            and state in {"qualified", "mapped", "executor-green"}
        )
        validate_pending_value(
            defect_id,
            field,
            value,
            state,
            errors,
            pending_allowed_late=pending_allowed_late,
        )
    elif status == "pass":
        if not str(value.get("command", "")).strip():
            errors.append(f"{defect_id} {field} pass has no command")
        if not str(value.get("evidence", "")).strip():
            errors.append(f"{defect_id} {field} pass has no evidence")
        if field == "orchestrator_verification" and (
            value.get("actor") != "independent-orchestrator"
        ):
            errors.append(
                f"{defect_id} orchestrator_verification pass lacks independent actor"
            )
    elif status == "not-applicable":
        if not str(value.get("reason", "")).strip():
            errors.append(f"{defect_id} {field} not-applicable has no reason")
    else:
        errors.append(f"{defect_id} {field} has invalid status {status!r}")


def validate_history_verifications(
    defect_id: str,
    history: Any,
    executor_verification: Any,
    orchestrator_verification: Any,
    errors: list[str],
) -> None:
    history_states = {
        str(row.get("state", ""))
        for row in history
        if isinstance(row, dict)
    } if isinstance(history, list) else set()
    executor_status = (
        executor_verification.get("status")
        if isinstance(executor_verification, dict)
        else None
    )
    orchestrator_status = (
        orchestrator_verification.get("status")
        if isinstance(orchestrator_verification, dict)
        else None
    )
    if "executor-green" in history_states and executor_status != "pass":
        errors.append(
            f"{defect_id} history contains executor-green but "
            "executor_verification is not pass"
        )
    if (
        "orchestrator-verified" in history_states
        and orchestrator_status != "pass"
    ):
        errors.append(
            f"{defect_id} history contains orchestrator-verified but "
            "orchestrator_verification is not pass"
        )


def validate_closure_schema(
    row: dict[str, Any],
    fixture_row: dict[str, Any] | None,
    source_artifact_hashes: dict[str, str],
    errors: list[str],
) -> None:
    defect_id = str(row.get("id", ""))
    state = str(row.get("state", ""))
    for field in (
        "source_class",
        "preliminary_disposition",
        "rust_stimulus",
        "cpp_stimulus",
        "rust_owner",
        "displaced_mechanism",
        "owning_ledger",
        "adaptation_rule",
        "decision_row",
    ):
        validate_pending_value(defect_id, field, row.get(field), state, errors)

    hashes = row.get("artifact_hashes")
    if not isinstance(hashes, dict):
        errors.append(f"{defect_id} artifact_hashes must be a table")
    else:
        if set(hashes) != ARTIFACT_HASH_KEYS:
            errors.append(
                f"{defect_id} artifact_hashes keys must be exactly "
                f"{', '.join(sorted(ARTIFACT_HASH_KEYS))}"
            )
        for field in ARTIFACT_HASH_KEYS:
            if SHA256_RE.fullmatch(str(hashes.get(field, ""))) is None:
                errors.append(
                    f"{defect_id} artifact_hashes.{field} must be a SHA256"
                )
            elif hashes.get(field) != source_artifact_hashes.get(field):
                errors.append(
                    f"{defect_id} artifact_hashes.{field} does not match "
                    "the pinned source artifact"
                )

    revisions = row.get("revisions")
    if not isinstance(revisions, dict):
        errors.append(f"{defect_id} revisions must be a table")
    else:
        if set(revisions) != REVISION_KEYS:
            errors.append(
                f"{defect_id} revisions keys must be exactly "
                f"{', '.join(sorted(REVISION_KEYS))}"
            )
        history_states = {
            history.get("state")
            for history in row.get("history", [])
            if isinstance(history, dict)
        }
        no_repair_path = (
            any(
                history.get("state") in {"stale-oracle", "retracted"}
                for history in row.get("history", [])
                if isinstance(history, dict)
            )
            or row.get("classification")
            in {"additive-product-feature", "editor-integration-defect"}
            or row.get("owner_class") == "artifact"
        )
        for field in REVISION_KEYS:
            validate_revision(
                defect_id,
                field,
                revisions.get(field),
                state,
                no_repair_path,
                errors,
                editor_consumed_path="editor-consumed" in history_states,
                handoff_ready_path="handoff-ready" in history_states,
            )

    for field in ("source_files", "source_members", "lifecycle_phases"):
        values = row.get(field)
        if not isinstance(values, list) or not values:
            errors.append(f"{defect_id} {field} must be a nonempty list")
        elif any(not str(value).strip() for value in values):
            errors.append(f"{defect_id} {field} contains an empty value")
        else:
            for value in values:
                validate_pending_value(defect_id, field, value, state, errors)

    for field in ("dependencies", "target_tests"):
        values = row.get(field)
        if not isinstance(values, list):
            errors.append(f"{defect_id} {field} must be a list")
        elif any(not str(value).strip() for value in values):
            errors.append(f"{defect_id} {field} contains an empty value")

    renderer_row = row.get("owner_class") == "renderer" or (
        fixture_row is not None
        and fixture_row.get("kind") == "browser-renderer"
    )
    floors = row.get("required_floors")
    if not isinstance(floors, list) or not floors:
        errors.append(f"{defect_id} required_floors must be a nonempty list")
    else:
        unknown = sorted({str(value) for value in floors} - KNOWN_FLOORS)
        if unknown:
            errors.append(
                f"{defect_id} required_floors contains unknown floors: "
                f"{', '.join(unknown)}"
            )
        if (
            renderer_row
            and state in QUALIFIED_OR_LATER
            and "renderer_pixels" not in floors
        ):
            errors.append(
                f"{defect_id} is qualified renderer work but omits "
                "the renderer_pixels floor"
            )

    renderer = row.get("renderer_provenance")
    if not isinstance(renderer, dict):
        errors.append(f"{defect_id} renderer_provenance must be a table")
    else:
        status = str(renderer.get("status", ""))
        if (
            renderer_row
            and state in QUALIFIED_OR_LATER
            and status != "complete"
        ):
            errors.append(
                f"{defect_id} is qualified renderer work but renderer "
                "provenance is not complete"
            )
        if status in {"pending", "not-applicable"}:
            if not str(renderer.get("reason", "")).strip():
                errors.append(
                    f"{defect_id} renderer_provenance {status} has no reason"
                )
            if status == "pending" and state not in EARLY_STATES:
                errors.append(
                    f"{defect_id} is {state} but renderer_provenance is pending"
                )
        elif status == "complete":
            for field in (
                "backend",
                "dawn_revision",
                "mode",
                "feature_flags",
                "surface",
                "reference_executable",
                "reference_stamp_sha256",
                "command",
                "evidence",
            ):
                if not str(renderer.get(field, "")).strip():
                    errors.append(
                        f"{defect_id} renderer_provenance complete has no {field}"
                    )
            reference_stamp = str(renderer.get("reference_stamp_sha256", ""))
            if (
                reference_stamp
                and SHA256_RE.fullmatch(reference_stamp) is None
            ):
                errors.append(
                    f"{defect_id} renderer_provenance reference stamp "
                    "must be a SHA256"
                )
        else:
            errors.append(
                f"{defect_id} renderer_provenance has invalid status {status!r}"
            )

    validate_verification(
        defect_id, "executor_verification", row.get("executor_verification"), state, errors
    )
    validate_verification(
        defect_id,
        "orchestrator_verification",
        row.get("orchestrator_verification"),
        state,
        errors,
    )
    validate_history_verifications(
        defect_id,
        row.get("history"),
        row.get("executor_verification"),
        row.get("orchestrator_verification"),
        errors,
    )

    reproduction = str(row.get("reproduction_sha256", ""))
    if SHA256_RE.fullmatch(reproduction) is None:
        errors.append(f"{defect_id} reproduction_sha256 must be a SHA256")
    elif fixture_row is None:
        errors.append(f"{defect_id} cannot resolve fixture for reproduction hash")
    else:
        expected = fixture_digest(fixture_row)
        if reproduction != expected:
            errors.append(
                f"{defect_id} reproduction_sha256 is {reproduction}; "
                f"fixture digest is {expected}"
            )


def validate_artifacts(
    artifacts: list[dict[str, Any]],
    source_root: pathlib.Path | None,
    errors: list[str],
) -> None:
    ids = [str(row.get("id", "")) for row in artifacts]
    duplicates = duplicate_values(ids)
    if duplicates:
        errors.append(f"duplicate artifact ids: {', '.join(duplicates)}")
    paths = [str(row.get("path", "")) for row in artifacts]
    duplicate_paths = duplicate_values(paths)
    if duplicate_paths:
        errors.append(
            "duplicate source artifact paths: "
            + ", ".join(duplicate_paths)
        )
    if len(artifacts) != 3:
        errors.append(f"expected 3 source artifacts, found {len(artifacts)}")
    for row in artifacts:
        artifact_id = str(row.get("id", ""))
        relative = str(row.get("path", ""))
        digest = str(row.get("sha256", ""))
        if not artifact_id or not relative:
            errors.append("source artifact has an empty id or path")
            continue
        path_valid = True
        relative_path = pathlib.PurePosixPath(relative)
        if relative_path.is_absolute():
            errors.append(
                f"source artifact {artifact_id} path must be a relative filename"
            )
            path_valid = False
        if ".." in relative_path.parts:
            errors.append(
                f"source artifact {artifact_id} path contains traversal"
            )
            path_valid = False
        expected_path = SOURCE_ARTIFACT_PATHS.get(artifact_id)
        if expected_path is not None and relative != expected_path:
            errors.append(
                f"source artifact {artifact_id} path must be "
                f"{expected_path!r}"
            )
            path_valid = False
        if SHA256_RE.fullmatch(digest) is None:
            errors.append(f"artifact {artifact_id} has invalid sha256")
            continue
        if source_root is None or not path_valid:
            continue
        path = source_root / relative
        if not validate_file_beneath_root(
            source_root,
            path,
            f"source artifact {artifact_id}",
            errors,
        ):
            continue
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != digest:
            errors.append(
                f"artifact {artifact_id} hash is {actual}, atlas records {digest}"
            )


def validate_program(program: Any, errors: list[str]) -> None:
    if not isinstance(program, dict):
        errors.append("atlas has no program table")
        return
    if set(program) != PROGRAM_KEYS:
        errors.append(
            "program keys must be exactly "
            + ", ".join(sorted(PROGRAM_KEYS))
        )
    for field, expected in EXPECTED_PROGRAM.items():
        actual = program.get(field)
        if actual != expected:
            errors.append(
                f"program {field} is {actual!r}; expected {expected!r}"
            )
    intake_cycle = program.get("intake_cycle")
    if not isinstance(intake_cycle, int) or isinstance(intake_cycle, bool):
        errors.append("program intake_cycle must be an integer")
    elif intake_cycle < 1:
        errors.append("program intake_cycle must be at least 1")


def validate_inbox(
    inbox: Any,
    source_snapshot_ref: str,
    source_artifact_hashes: dict[str, str],
    defect_count: int,
    errors: list[str],
) -> None:
    if not isinstance(inbox, dict):
        errors.append("atlas has no inbox table")
        return
    if set(inbox) != INBOX_KEYS:
        errors.append(
            "inbox keys must be exactly "
            + ", ".join(sorted(INBOX_KEYS))
        )
    for field, expected in EXPECTED_INBOX.items():
        actual = inbox.get(field)
        if actual != expected:
            errors.append(
                f"inbox {field} is {actual!r}; expected {expected!r}"
            )

    ref_fields = (
        "last_consumed_editor_ref",
        "newest_available_editor_ref",
    )
    hash_fields = (
        "last_consumed_runtime_defects_sha256",
        "last_consumed_parity_ledger_sha256",
        "newest_available_runtime_defects_sha256",
        "newest_available_parity_ledger_sha256",
    )
    for field in ref_fields:
        if SHA_RE.fullmatch(str(inbox.get(field, ""))) is None:
            errors.append(f"inbox {field} must be a full 40-hex SHA")
    for field in hash_fields:
        if SHA256_RE.fullmatch(str(inbox.get(field, ""))) is None:
            errors.append(f"inbox {field} must be a SHA256")

    if inbox.get("last_consumed_editor_ref") != source_snapshot_ref:
        errors.append(
            "inbox last_consumed_editor_ref does not match "
            "source_snapshot_ref"
        )
    for inbox_field, artifact_field in (
        ("last_consumed_runtime_defects_sha256", "runtime_defects"),
        ("last_consumed_parity_ledger_sha256", "parity_ledger"),
    ):
        if inbox.get(inbox_field) != source_artifact_hashes.get(artifact_field):
            errors.append(
                f"inbox {inbox_field} does not match the pinned "
                f"{artifact_field} source artifact"
            )

    imported = inbox.get("imported_atlas_count")
    if not isinstance(imported, int) or isinstance(imported, bool):
        errors.append("inbox imported_atlas_count must be an integer")
    elif imported != defect_count:
        errors.append(
            f"inbox imported_atlas_count is {imported}; "
            f"atlas has {defect_count} defect rows"
        )

    unconsumed = inbox.get("unconsumed_records")
    if not isinstance(unconsumed, int) or isinstance(unconsumed, bool):
        errors.append("inbox unconsumed_records must be an integer")
        return
    if unconsumed < 0:
        errors.append("inbox unconsumed_records must be nonnegative")
        return
    consumed_checkpoint = (
        inbox.get("last_consumed_editor_ref"),
        inbox.get("last_consumed_runtime_defects_sha256"),
        inbox.get("last_consumed_parity_ledger_sha256"),
    )
    newest_checkpoint = (
        inbox.get("newest_available_editor_ref"),
        inbox.get("newest_available_runtime_defects_sha256"),
        inbox.get("newest_available_parity_ledger_sha256"),
    )
    if consumed_checkpoint == newest_checkpoint and unconsumed != 0:
        errors.append(
            "inbox checkpoints are identical but unconsumed_records is not 0"
        )
    elif consumed_checkpoint != newest_checkpoint and unconsumed == 0:
        errors.append(
            "inbox checkpoints differ but unconsumed_records is 0"
        )


def validate_floors(floors: Any, errors: list[str]) -> None:
    if not isinstance(floors, dict):
        errors.append("atlas has no floors table")
        return
    expected_fields = set(MINIMUM_FLOORS) | set(MAXIMUM_CEILINGS)
    extra_fields = sorted(set(floors) - expected_fields)
    if extra_fields:
        errors.append(f"unknown floor fields: {', '.join(extra_fields)}")
    for field, minimum in MINIMUM_FLOORS.items():
        actual = floors.get(field)
        if not isinstance(actual, int):
            errors.append(f"floor {field} must be an integer")
        elif actual < minimum:
            errors.append(f"floor {field} is {actual}; minimum is {minimum}")
    for field, maximum in MAXIMUM_CEILINGS.items():
        actual = floors.get(field)
        if not isinstance(actual, int):
            errors.append(f"ceiling {field} must be an integer")
        elif actual > maximum:
            errors.append(f"ceiling {field} is {actual}; maximum is {maximum}")


def check(
    *,
    repo_root: pathlib.Path,
    atlas_path: pathlib.Path,
    corrections_path: pathlib.Path,
    fixtures_path: pathlib.Path,
    source_root: pathlib.Path | None,
    newest_source_root: pathlib.Path | None,
    editor_repo_dir: pathlib.Path | None,
    expected_upstream_ref: str,
    rive_runtime_dir: pathlib.Path | None,
    cpp_probe: pathlib.Path | None,
    require_closed: bool,
    validate_source_snapshot_git: bool,
) -> str:
    atlas = read_toml(atlas_path)
    corrections = read_toml(corrections_path)
    fixtures = read_toml(fixtures_path)
    errors: list[str] = []

    if atlas.get("schema") != SCHEMA:
        errors.append(f"atlas schema must be {SCHEMA}")
    if atlas.get("version") != 2:
        errors.append("atlas version must be 2")
    upstream_ref = str(atlas.get("upstream_ref", ""))
    if SHA_RE.fullmatch(upstream_ref) is None:
        errors.append("atlas upstream_ref must be a full 40-hex SHA")
    if upstream_ref != expected_upstream_ref:
        errors.append(
            f"atlas pins {upstream_ref}; expected {expected_upstream_ref}"
        )
    if rive_runtime_dir is not None:
        actual = git_head(rive_runtime_dir)
        if actual != upstream_ref:
            errors.append(
                f"upstream checkout is {actual}; atlas pins {upstream_ref}"
            )
    for field in ("editor_consumed_runtime_ref", "investigation_base_ref"):
        if SHA_RE.fullmatch(str(atlas.get(field, ""))) is None:
            errors.append(f"atlas {field} must be a full 40-hex SHA")
    source_snapshot_status = str(atlas.get("source_snapshot_status", ""))
    source_snapshot_ref = str(atlas.get("source_snapshot_ref", ""))
    if source_snapshot_status == "landed":
        if SHA_RE.fullmatch(source_snapshot_ref) is None:
            errors.append("landed snapshot must have a full source_snapshot_ref")
        elif validate_source_snapshot_git and source_root is not None:
            actual_source_ref = git_head(source_root)
            if actual_source_ref != source_snapshot_ref:
                errors.append(
                    f"Editor source checkout is {actual_source_ref}; "
                    f"atlas pins {source_snapshot_ref}"
                )
    elif source_snapshot_status == "pending-editor-commit":
        if source_snapshot_ref:
            errors.append(
                "pending-editor-commit snapshot must have an empty "
                "source_snapshot_ref"
            )
    else:
        errors.append(
            "source_snapshot_status must be landed or pending-editor-commit"
        )
    declared_corrections = (
        repo_root / str(atlas.get("corrections_file", ""))
    ).resolve()
    if declared_corrections != corrections_path:
        errors.append(
            f"atlas corrections_file resolves to {declared_corrections}, "
            f"but checker received {corrections_path}"
        )
    declared_fixtures = (
        repo_root / str(atlas.get("fixtures_file", ""))
    ).resolve()
    if declared_fixtures != fixtures_path:
        errors.append(
            f"atlas fixtures_file resolves to {declared_fixtures}, "
            f"but checker received {fixtures_path}"
        )

    correction_count = validate_corrections(
        corrections,
        upstream_ref,
        validate_source_snapshot_git,
        errors,
    )
    artifact_rows = list(atlas.get("artifact", []))
    validate_artifacts(artifact_rows, source_root, errors)
    artifact_id_to_field = {
        "cutover-proposal": "proposal",
        "runtime-defects": "runtime_defects",
        "parity-ledger": "parity_ledger",
    }
    source_artifact_hashes = {
        artifact_id_to_field[str(row.get("id", ""))]: str(row.get("sha256", ""))
        for row in artifact_rows
        if str(row.get("id", "")) in artifact_id_to_field
    }
    if set(source_artifact_hashes) != ARTIFACT_HASH_KEYS:
        errors.append(
            "source artifact ids must be exactly cutover-proposal, "
            "runtime-defects, parity-ledger"
        )
    validate_program(atlas.get("program"), errors)
    validate_floors(atlas.get("floors"), errors)

    reserved_ids = set(str(value) for value in atlas.get("reserved_ids", []))
    if reserved_ids != {"LOC-010"}:
        errors.append("reserved_ids must contain only LOC-010")

    lease = atlas.get("lease")
    if not isinstance(lease, dict):
        errors.append("atlas has no lease table")
        lease = {}
    for field in ("refreshed", "active_wave", "branch"):
        if lease.get(field) != EXPECTED_LEASE[field]:
            errors.append(
                f"lease {field} is {lease.get(field)!r}; "
                f"expected {EXPECTED_LEASE[field]!r}"
            )
    for field in ("reserved_files", "future_files", "shared_ledgers"):
        actual_paths = {str(value) for value in lease.get(field, [])}
        expected_paths = EXPECTED_LEASE[field]
        if actual_paths != expected_paths:
            missing = ", ".join(sorted(expected_paths - actual_paths)) or "none"
            extra = ", ".join(sorted(actual_paths - expected_paths)) or "none"
            errors.append(
                f"lease {field} differs from the pinned coordination contract; "
                f"missing: {missing}; extra: {extra}"
            )

    rows = list(atlas.get("defect", []))
    ids = [str(row.get("id", "")) for row in rows]
    duplicates = duplicate_values(ids)
    if duplicates:
        errors.append(f"duplicate defect ids: {', '.join(duplicates)}")
    actual_ids = set(ids)
    missing = sorted(BASELINE_IDS - actual_ids)
    extra: list[str] = []
    for defect_id in actual_ids - BASELINE_IDS:
        local_match = LOCAL_ID_RE.fullmatch(defect_id)
        runtime_match = RUNTIME_ID_RE.fullmatch(defect_id)
        valid_future_local = (
            local_match is not None
            and defect_id not in reserved_ids
            and int(defect_id.removeprefix("LOC-")) >= 20
        )
        valid_future_runtime = (
            runtime_match is not None
            and int(defect_id.removeprefix("RT-ED-")) >= 8
        )
        if not (valid_future_local or valid_future_runtime):
            extra.append(defect_id)
    extra.sort()
    if missing:
        errors.append(f"atlas is missing defect ids: {', '.join(missing)}")
    if extra:
        errors.append(f"atlas has unexpected defect ids: {', '.join(extra)}")
    if atlas.get("expected_defects") != len(rows):
        errors.append(
            f"defect count ratchet says {atlas.get('expected_defects')}, "
            f"actual is {len(rows)}"
        )
    validate_inbox(
        atlas.get("inbox"),
        source_snapshot_ref,
        source_artifact_hashes,
        len(rows),
        errors,
    )
    inbox = atlas.get("inbox")
    canonical_branch_tip: str | None = None
    prior_atlas: dict[str, Any] | None = None
    if (
        validate_source_snapshot_git
        and isinstance(inbox, dict)
        and source_root is not None
        and newest_source_root is not None
        and editor_repo_dir is not None
    ):
        canonical_branch_tip = validate_editor_git_provenance(
            inbox,
            source_root,
            newest_source_root,
            editor_repo_dir,
            errors,
        )
        prior_atlas = validate_prior_id_ratchet(repo_root, rows, errors)
        validate_revision_provenance(
            atlas,
            rows,
            repo_root,
            editor_repo_dir,
            errors,
        )
        validate_landed_repair_provenance(rows, errors)

    consumed_sections: dict[str, str] = {}
    formal_by_dependency: dict[str, set[str]] = {}
    ledger_assertions: dict[str, str] = {}
    if isinstance(inbox, dict) and source_root is not None:
        consumed_defects_path = source_path(
            source_root,
            str(inbox.get("runtime_defects_path", "")),
        )
        consumed_ledger_path = source_path(
            source_root,
            str(inbox.get("parity_ledger_path", "")),
        )
        defects_safe = validate_file_beneath_root(
            source_root,
            consumed_defects_path,
            f"consumed Editor source {consumed_defects_path.name}",
            errors,
        )
        ledger_safe = validate_file_beneath_root(
            source_root,
            consumed_ledger_path,
            f"consumed Editor source {consumed_ledger_path.name}",
            errors,
        )
        if defects_safe:
            consumed_sections = parse_defect_sections(
                consumed_defects_path,
                "consumed",
                errors,
            )
        if ledger_safe:
            formal_by_dependency, ledger_assertions = parse_ledger_children(
                consumed_ledger_path,
                errors,
            )
        validate_source_record_contract(
            rows,
            consumed_sections,
            formal_by_dependency,
            ledger_assertions,
            errors,
        )
        if validate_source_snapshot_git and editor_repo_dir is not None:
            validate_consumed_intake_delta(
                prior_atlas,
                rows,
                consumed_sections,
                editor_repo_dir,
                errors,
            )
        if newest_source_root is not None:
            validate_newest_inbox(
                inbox,
                consumed_sections,
                newest_source_root,
                errors,
            )

    fixture_ids: list[str] = []
    atlas_fixtures: dict[str, str] = {}
    atlas_states = {
        str(row.get("id", "")): str(row.get("state", "")) for row in rows
    }
    formal_children: set[str] = set()
    candidate_children: set[str] = set()
    disputed_children: set[str] = set()
    state_counts: collections.Counter[str] = collections.Counter()
    reserved_paths = {
        str(value)
        for field in ("reserved_files", "future_files", "shared_ledgers")
        for value in lease.get(field, [])
    }

    for row in rows:
        defect_id = str(row.get("id", ""))
        state = str(row.get("state", ""))
        state_counts[state] += 1
        if state not in STATES:
            errors.append(f"{defect_id} has invalid state {state!r}")
        if row.get("owner_class") not in OWNER_CLASSES:
            errors.append(f"{defect_id} has invalid owner_class")
        if row.get("classification") not in CLASSIFICATIONS:
            errors.append(f"{defect_id} has invalid classification")
        ticket = str(row.get("ticket", ""))
        if TICKET_RE.fullmatch(ticket) is None:
            errors.append(f"{defect_id} has invalid ticket {ticket!r}")
        if not str(row.get("title", "")).strip():
            errors.append(f"{defect_id} has no title")

        fixture_id = str(row.get("fixture_id", ""))
        if not fixture_id:
            errors.append(f"{defect_id} has no fixture_id")
        elif fixture_id not in atlas_fixtures:
            atlas_fixtures[fixture_id] = defect_id
        fixture_ids.append(fixture_id)

        validate_history(
            defect_id,
            state,
            row.get("history"),
            errors,
            owner_class=str(row.get("owner_class", "")),
        )
        for layer in ("cpp", "rust", "editor"):
            validate_result(
                defect_id,
                layer,
                row.get(f"{layer}_result"),
                state,
                errors,
            )

        touch = {str(value) for value in row.get("touch", [])}
        declared_dont_touch = {
            str(value) for value in row.get("dont_touch", [])
        }
        dont_touch = (
            reserved_paths
            if declared_dont_touch == {"@active-fl-lease"}
            else declared_dont_touch
        )
        overlap = sorted(touch & dont_touch)
        if overlap:
            errors.append(
                f"{defect_id} TOUCH and DON'T TOUCH overlap: {', '.join(overlap)}"
            )
        if not reserved_paths.issubset(dont_touch):
            missing_locks = sorted(reserved_paths - dont_touch)
            errors.append(
                f"{defect_id} omits active lease locks: {', '.join(missing_locks)}"
            )

        row_formal = validate_children(
            defect_id, "formal_children", row.get("formal_children"), errors
        )
        row_candidate = validate_children(
            defect_id, "candidate_children", row.get("candidate_children"), errors
        )
        row_disputed = validate_children(
            defect_id, "disputed_children", row.get("disputed_children"), errors
        )
        formal_children |= row_formal
        candidate_children |= row_candidate
        disputed_children |= row_disputed
        expected_children = EXPECTED_CHILDREN.get(defect_id)
        if expected_children is not None and (
            row_formal,
            row_candidate,
            row_disputed,
        ) != expected_children:
            errors.append(
                f"{defect_id} child mapping differs from the pinned exact map"
            )

    fixture_duplicates = duplicate_values(fixture_ids)
    if fixture_duplicates:
        errors.append(f"duplicate fixture ids: {', '.join(fixture_duplicates)}")
    stimulus_git_sources = {
        "repo": (
            repo_root,
            str(atlas.get("investigation_base_ref", "")),
            "investigation_base_ref",
        ),
    }
    if editor_repo_dir is not None:
        stimulus_git_sources["editor"] = (
            editor_repo_dir,
            source_snapshot_ref,
            "source_snapshot_ref/last_consumed_editor_ref",
        )
    if rive_runtime_dir is not None:
        stimulus_git_sources["rive"] = (
            rive_runtime_dir,
            upstream_ref,
            "upstream_ref",
        )
    fixture_count, fixture_rows = validate_fixtures(
        fixtures,
        upstream_ref,
        atlas_fixtures,
        actual_ids,
        atlas_states,
        cpp_probe,
        repo_root,
        {
            "repo": repo_root,
            "rive": rive_runtime_dir,
            "editor": source_root.parent if source_root is not None else None,
        },
        stimulus_git_sources,
        validate_source_snapshot_git,
        errors,
    )
    for row in rows:
        fixture_id = str(row.get("fixture_id", ""))
        validate_closure_schema(
            row,
            fixture_rows.get(fixture_id),
            source_artifact_hashes,
            errors,
        )
    if atlas.get("expected_formal_children") != len(formal_children):
        errors.append(
            "formal-child count ratchet says "
            f"{atlas.get('expected_formal_children')}, actual is "
            f"{len(formal_children)}"
        )
    if atlas.get("expected_candidate_children") != len(candidate_children):
        errors.append(
            "candidate-child count ratchet says "
            f"{atlas.get('expected_candidate_children')}, actual is "
            f"{len(candidate_children)}"
        )
    union_children = formal_children | candidate_children
    if atlas.get("expected_union_children") != len(union_children):
        errors.append(
            "union-child count ratchet says "
            f"{atlas.get('expected_union_children')}, actual is "
            f"{len(union_children)}"
        )
    overlap_children = formal_children & candidate_children
    expected_overlap = {
        str(value) for value in atlas.get("expected_overlap_children", [])
    }
    if expected_overlap != overlap_children:
        expected_text = ", ".join(sorted(expected_overlap)) or "empty"
        actual_text = ", ".join(sorted(overlap_children)) or "empty"
        errors.append(
            f"child-overlap ratchet names {expected_text}, "
            f"actual overlap is {actual_text}"
        )
    if not disputed_children.issubset(formal_children | candidate_children):
        errors.append("disputed children must also be formal or candidate children")
    if require_closed:
        if validate_source_snapshot_git:
            validate_closed_inbox(inbox, canonical_branch_tip, errors)
        elif not isinstance(inbox, dict) or inbox.get("unconsumed_records") != 0:
            errors.append("--require-closed requires unconsumed_records = 0")
        open_rows = sorted(
            str(row.get("id", ""))
            for row in rows
            if row.get("state") != "closed"
        )
        if open_rows:
            errors.append(f"rows remain open: {', '.join(open_rows)}")

    if errors:
        raise CheckFailure("\n".join(f"- {error}" for error in errors))
    counts = ",".join(f"{key}:{state_counts[key]}" for key in sorted(state_counts))
    return (
        f"editor-next-runtime-defects: defects={len(rows)} "
        f"corrections={correction_count} fixtures={fixture_count} "
        f"intake_cycle={atlas['program']['intake_cycle']} "
        f"imported={atlas['inbox']['imported_atlas_count']} "
        f"unconsumed={atlas['inbox']['unconsumed_records']} "
        f"states={counts} "
        f"formal_children={len(formal_children)} "
        f"candidate_children={len(candidate_children)} "
        f"union_children={len(union_children)}"
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=pathlib.Path, required=True)
    parser.add_argument("--atlas", type=pathlib.Path, required=True)
    parser.add_argument("--corrections", type=pathlib.Path, required=True)
    parser.add_argument("--fixtures", type=pathlib.Path, required=True)
    parser.add_argument("--source-root", type=pathlib.Path)
    parser.add_argument("--newest-source-root", type=pathlib.Path)
    parser.add_argument("--editor-repo-dir", type=pathlib.Path)
    parser.add_argument("--rive-runtime-dir", type=pathlib.Path)
    parser.add_argument("--cpp-probe", type=pathlib.Path)
    parser.add_argument(
        "--test-mode",
        action="store_true",
        help="permit omitted production provenance inputs for isolated unit fixtures",
    )
    parser.add_argument("--expected-upstream-ref", required=True)
    parser.add_argument("--require-closed", action="store_true")
    return parser.parse_args()


def lexical_absolute(path: pathlib.Path) -> pathlib.Path:
    if path.is_absolute():
        return path
    return pathlib.Path.cwd() / path


def main() -> int:
    args = parse_args()
    canonical_atlas = (
        args.repo_root.resolve()
        / "docs"
        / "editor-next-runtime-defect-atlas.toml"
    )
    if args.test_mode and args.atlas.resolve() == canonical_atlas:
        print(
            "editor-next-runtime-defect-check failed:\n"
            "- --test-mode cannot validate the repository atlas",
            file=sys.stderr,
        )
        return 2
    if not args.test_mode:
        missing = [
            flag
            for flag, value in (
                ("--source-root", args.source_root),
                ("--newest-source-root", args.newest_source_root),
                ("--editor-repo-dir", args.editor_repo_dir),
                ("--rive-runtime-dir", args.rive_runtime_dir),
                ("--cpp-probe", args.cpp_probe),
            )
            if value is None
        ]
        if missing:
            print(
                "editor-next-runtime-defect-check failed:\n"
                "- production mode requires provenance inputs: "
                + ", ".join(missing),
                file=sys.stderr,
            )
            return 2
    try:
        summary = check(
            repo_root=lexical_absolute(args.repo_root),
            atlas_path=args.atlas.resolve(),
            corrections_path=args.corrections.resolve(),
            fixtures_path=args.fixtures.resolve(),
            source_root=(
                lexical_absolute(args.source_root)
                if args.source_root
                else None
            ),
            newest_source_root=(
                lexical_absolute(args.newest_source_root)
                if args.newest_source_root
                else None
            ),
            editor_repo_dir=(
                lexical_absolute(args.editor_repo_dir)
                if args.editor_repo_dir
                else None
            ),
            expected_upstream_ref=args.expected_upstream_ref,
            rive_runtime_dir=(
                lexical_absolute(args.rive_runtime_dir)
                if args.rive_runtime_dir
                else None
            ),
            cpp_probe=args.cpp_probe.resolve() if args.cpp_probe else None,
            require_closed=args.require_closed,
            validate_source_snapshot_git=not args.test_mode,
        )
    except CheckFailure as error:
        print(f"editor-next-runtime-defect-check failed:\n{error}", file=sys.stderr)
        return 1
    print(summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
