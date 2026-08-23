#!/usr/bin/env python3
"""Freeze per-row source-only renderer repeatability observations."""

from __future__ import annotations

import argparse
import csv
import hashlib
import re
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path


HEADER = (
    "campaign",
    "corpus_entry",
    "primary_backend_id",
    "mode",
    "stream_sha256",
    "run_a_replay_sha256",
    "run_b_replay_sha256",
    "adapter",
    "run_a_png_sha256",
    "run_b_png_sha256",
    "observed_different_pixels",
    "observed_max_channel_delta",
    "frozen_max_different_pixels",
    "frozen_max_channel_delta",
    "candidate_output_observed",
    "status",
)
PRIMARY_BACKENDS = {
    "vulkan": "ffi-vulkan",
    "webgpu": "ffi-dawn",
    "webgl2": "ffi-webgl2",
}
RESULT = re.compile(
    r"^exact (?P<id>.+): byte-exact=(?P<byte>true|false) "
    r"different-pixels=(?P<pixels>[0-9]+) max-channel-delta=(?P<delta>[0-9]+)$"
)


@dataclass(frozen=True)
class Observation:
    different_pixels: int
    max_channel_delta: int


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", type=Path, required=True)
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument(
        "--capture",
        action="append",
        default=[],
        metavar="CAMPAIGN=DIR",
        help="source-only corpus-r output directory for one campaign",
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--check", action="store_true")
    return parser.parse_args()


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def captures(values: list[str], repo: Path) -> dict[str, Path]:
    result: dict[str, Path] = {}
    for value in values:
        campaign, separator, raw_path = value.partition("=")
        if not separator or campaign not in PRIMARY_BACKENDS or not raw_path:
            raise ValueError(f"invalid --capture {value!r}")
        if campaign in result:
            raise ValueError(f"duplicate --capture campaign: {campaign}")
        path = Path(raw_path)
        result[campaign] = path if path.is_absolute() else repo / path
    return result


def read_observations(log: Path) -> dict[str, Observation]:
    observations: dict[str, Observation] = {}
    for line in log.read_text().splitlines():
        match = RESULT.fullmatch(line)
        if match is None:
            continue
        entry = match["id"]
        if entry in observations:
            raise ValueError(f"duplicate repeatability result: {entry}")
        observation = Observation(int(match["pixels"]), int(match["delta"]))
        if (match["byte"] == "true") != (
            observation.different_pixels == 0 and observation.max_channel_delta == 0
        ):
            raise ValueError(f"byte-exact flag disagrees with observation: {entry}")
        observations[entry] = observation
    return observations


def pending_row(campaign: str, entry: dict) -> list[str]:
    return [
        campaign,
        entry["id"],
        PRIMARY_BACKENDS[campaign],
        entry["mode"],
        *(["-"] * 10),
        "false",
        "pending-source-capture",
    ]


def captured_row(
    campaign: str,
    entry: dict,
    root: Path,
    observation: Observation,
) -> list[str]:
    entry_id = entry["id"]
    provenance_path = root / f"{entry_id}.provenance.toml"
    if not provenance_path.is_file():
        raise FileNotFoundError(f"missing repeatability provenance: {provenance_path}")
    provenance = tomllib.loads(provenance_path.read_text())
    expected_backend = PRIMARY_BACKENDS[campaign]
    required_equal = {
        "case_id": entry_id,
        "mode": entry["mode"],
        "reference_backend": expected_backend,
        "candidate_backend": expected_backend,
        "adapter_check": "matched",
    }
    for key, expected in required_equal.items():
        if provenance.get(key) != expected:
            raise ValueError(
                f"repeatability provenance {entry_id} {key}: "
                f"expected {expected!r}, got {provenance.get(key)!r}"
            )
    if provenance["reference_replay_sha256"] != provenance["candidate_replay_sha256"]:
        raise ValueError(f"repeatability used different source binaries: {entry_id}")
    if provenance["reference_adapter"] != provenance["candidate_adapter"]:
        raise ValueError(f"repeatability used different adapters: {entry_id}")
    reference_png = Path(provenance["reference_output"])
    candidate_png = Path(provenance["candidate_output"])
    if not reference_png.is_absolute():
        reference_png = root.parents[2] / reference_png
    if not candidate_png.is_absolute():
        candidate_png = root.parents[2] / candidate_png
    for label, path, expected in (
        ("run A", reference_png, provenance["reference_png_sha256"]),
        ("run B", candidate_png, provenance["candidate_png_sha256"]),
    ):
        if not path.is_file() or sha256(path) != expected:
            raise ValueError(f"{label} PNG identity drift: {entry_id}")
    return [
        campaign,
        entry_id,
        expected_backend,
        entry["mode"],
        provenance["stream_sha256"],
        provenance["reference_replay_sha256"],
        provenance["candidate_replay_sha256"],
        provenance["reference_adapter"],
        provenance["reference_png_sha256"],
        provenance["candidate_png_sha256"],
        str(observation.different_pixels),
        str(observation.max_channel_delta),
        str(observation.different_pixels),
        str(observation.max_channel_delta),
        "false",
        "frozen-source-repeatability",
    ]


def render(repo: Path, corpus_path: Path, capture_roots: dict[str, Path]) -> str:
    corpus = tomllib.loads(corpus_path.read_text())["entry"]
    ids = [entry["id"] for entry in corpus]
    if len(ids) != len(set(ids)):
        raise ValueError("renderer corpus contains duplicate entry ids")
    rows: list[list[str]] = []
    for campaign in PRIMARY_BACKENDS:
        root = capture_roots.get(campaign)
        if root is None:
            rows.extend(pending_row(campaign, entry) for entry in corpus)
            continue
        log = root.with_suffix(".log")
        if not log.is_file():
            raise FileNotFoundError(f"missing repeatability log: {log}")
        observations = read_observations(log)
        if set(observations) != set(ids):
            missing = sorted(set(ids) - set(observations))
            extra = sorted(set(observations) - set(ids))
            raise ValueError(
                f"repeatability result denominator mismatch for {campaign}: "
                f"missing={missing[:5]} extra={extra[:5]}"
            )
        rows.extend(
            captured_row(campaign, entry, root, observations[entry["id"]])
            for entry in corpus
        )
    lines = ["\t".join(HEADER), *("\t".join(row) for row in rows)]
    return "\n".join(lines) + "\n"


def verify_frozen(output: Path, corpus_path: Path) -> int:
    corpus = tomllib.loads(corpus_path.read_text())["entry"]
    expected = {
        (campaign, entry["id"]): (PRIMARY_BACKENDS[campaign], entry["mode"])
        for campaign in PRIMARY_BACKENDS
        for entry in corpus
    }
    with output.open(newline="") as handle:
        frozen = list(csv.DictReader(handle, delimiter="\t"))
    if not frozen or tuple(frozen[0]) != HEADER:
        raise ValueError("repeatability inventory header drift")
    indexed = {(row["campaign"], row["corpus_entry"]): row for row in frozen}
    if len(indexed) != len(frozen) or set(indexed) != set(expected):
        raise ValueError("repeatability inventory denominator drift")
    for key, row in indexed.items():
        backend, mode = expected[key]
        if row["primary_backend_id"] != backend or row["mode"] != mode:
            raise ValueError(f"repeatability corpus identity drift: {key}")
        if row["candidate_output_observed"] != "false":
            raise ValueError(f"candidate output entered source repeatability: {key}")
        if row["status"] == "pending-source-capture":
            evidence = [row[column] for column in HEADER[4:14]]
            if any(value != "-" for value in evidence):
                raise ValueError(f"pending repeatability row carries evidence: {key}")
        elif row["status"] == "frozen-source-repeatability":
            if row["run_a_replay_sha256"] != row["run_b_replay_sha256"]:
                raise ValueError(f"repeatability source binary mismatch: {key}")
            if (
                row["observed_different_pixels"]
                != row["frozen_max_different_pixels"]
                or row["observed_max_channel_delta"]
                != row["frozen_max_channel_delta"]
            ):
                raise ValueError(f"repeatability budget is not source-derived: {key}")
            if not row["adapter"] or row["adapter"] == "-":
                raise ValueError(f"repeatability adapter missing: {key}")
        else:
            raise ValueError(f"invalid repeatability status for {key}: {row['status']}")
    print(f"backend source repeatability inventory clean: {len(frozen)} rows")
    return 0


def main() -> int:
    args = parse_args()
    repo = args.repo_root.resolve()
    corpus = args.corpus if args.corpus.is_absolute() else repo / args.corpus
    output = args.output if args.output.is_absolute() else repo / args.output
    capture_roots = captures(args.capture, repo)
    if args.check and not capture_roots:
        if not output.is_file():
            raise FileNotFoundError(f"missing repeatability inventory: {output}")
        return verify_frozen(output, corpus)
    rendered = render(repo, corpus, capture_roots)
    if args.check:
        if not output.is_file() or output.read_text() != rendered:
            print("backend source repeatability inventory is stale", file=sys.stderr)
            return 1
        print(f"backend source repeatability inventory clean: {len(rendered.splitlines()) - 1} rows")
        return 0
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(rendered)
    print(f"wrote {len(rendered.splitlines()) - 1} repeatability rows to {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
