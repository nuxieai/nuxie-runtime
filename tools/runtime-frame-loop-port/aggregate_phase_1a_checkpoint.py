#!/usr/bin/env python3
"""Aggregate Phase 1A only from committed acceptance-receipt snapshots."""

from __future__ import annotations

import collections
import json
import pathlib
import subprocess


ACCEPTED = (
    ("A", "048a0657c33231e3846abfd6d7c247b905e1791b", "wave-a.json", "wave-a-frozen-acceptance.md"),
    ("B2", "26b2c6ebf1725161b1b96833eb6e87d23f3415e2", "wave-b2.json", "wave-b2-final-independent-review.md"),
    ("B3", "896152b0807e102038c1818c4356a9f47b2f649e", "wave-b3.json", "wave-b3-locator-final-independent-review.md"),
    ("B4", "0d41d8903f46a14edf47f1e19300f38000647f89", "wave-b4.json", "wave-b4-final-acceptance.md"),
    ("B5", "96ee6fdb4e927d75c2287b20b636583281487f0b", "wave-b5.json", "wave-b5-case4-final-independent-review.md"),
    ("C1", "1aea14538a93a16b6bc848e95f024e1c014fff5f", "wave-c1.json", "wave-c1-four-row-correction-independent-acceptance.md"),
    ("C2", "d59a3caf56e85f34f8a3ece09e3a5ffc219fd36f", "wave-c2.json", "wave-c2-final-independent-acceptance.md"),
    ("C3 micro", "46f03b586c741d38c595400134516c3c9cdb9d6c", "wave-c3-micro.json", "wave-c3-micro-final-correction-independent-rereview.md"),
    ("C4", "6e0e1ed21b6532d53a6a0c228cf850bbd5add2fc", "wave-c4.json", "wave-c4-final-independent-rereview.md"),
    ("C5", "9ee7a7c87f3b03a6902ce507021b8a7fea2c8480", "wave-c5.json", "wave-c5-correction-independent-rereview.md"),
    ("C6", "162df278d5900e4655f8e2f6ce1dc8bb6f7acda6", "wave-c6.json", "wave-c6-final-independent-rereview.md"),
    ("C7", "1ab4dd63fb99ab793662e1d4ab093d0d3c0c451c", "wave-c7.json", "wave-c7-final-independent-rereview.md"),
    ("C8", "70185181b0fa31a2f9a39b1cdc3865cbd3b1410d", "wave-c8.json", "wave-c8-final-independent-rereview.md"),
    ("C9", "b1a86c2b5d18cbed2cd41ac4815b3eef2622af3f", "wave-c9.json", "wave-c9-margin-final-independent-acceptance.md"),
    ("C11", "c79aa6e8870e132c800219a9cd13e7c1946a9f41", "wave-c11.json", "wave-c11-promise-final-independent-review.md"),
    ("C12 scalar", "cf28c9339511e63edf680af76cad9af61c35b3ac", "wave-c12-scalar.json", "wave-c12-scalar-independent-review.md"),
    ("C12 Silver", "e03eb265c48dbc4a3cdf8a2fad7b32327142215f", "wave-c12-silver.json", "wave-c12-silver-final-independent-acceptance.md"),
    ("C13", "f49121e641986fe0d752d60d2a20340105a9ca6b", "wave-c13.json", "wave-c13-final-independent-rereview.md"),
    ("C14 scroll", "c4b46131cffb7a522227ce645c6ded27c4d646a6", "wave-c14-scroll.json", "wave-c14-scroll-independent-review.md"),
    ("C14 vector", "c034bdb2490341dd445e501538ac80a324c78b34", "wave-c14-vector.json", "wave-c14-vector-independent-review.md"),
    ("C14 velocity", "f571a0f76392d66040c39d23747f3ac447a51599", "wave-c14-velocity.json", "wave-c14-velocity-independent-review.md"),
    ("C14 wake", "e919e85b076ee83be142cd85c9f7039e55ff8ab1", "wave-c14-wake.json", "wave-c14-wake-final-independent-rereview.md"),
    ("C15", "598aec2ea1414f031847bf8664009f9e608b09d7", "wave-c15.json", "wave-c15-final-independent-acceptance.md"),
    ("C16", "bea957785f6e62247951055cebf17be36920094e", "wave-c16.json", "wave-c16-independent-review.md"),
    ("C17", "66332008b734782e776bb9b128c5dec56538ee49", "wave-c17.json", "wave-c17-final-independent-acceptance.md"),
)

PROVISIONAL = (
    (
        "B1",
        "d5fc3870ba6908ca42a38a61dd66d342b19c80db",
        "wave-b1.json",
        "wave-b1-transition-self-acceptance.md",
        "receipt is explicitly self-acceptance and records no durable independent reviewer identity",
    ),
)

BASE = "docs/runtime-test-case-waves"
PIN = "4ac7b32798da0482e441ef09304dc3b480ed3ee5"


def show(repo: pathlib.Path, revision: str, name: str) -> str:
    result = subprocess.run(
        ["git", "show", f"{revision}:{BASE}/{name}"],
        cwd=repo,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout


def main() -> None:
    repo = pathlib.Path(__file__).resolve().parents[2]
    totals: collections.Counter[str] = collections.Counter()
    blockers: collections.Counter[str] = collections.Counter()
    waves: list[dict[str, object]] = []
    ids: set[str] = set()

    for wave, revision, ledger_name, receipt_name in ACCEPTED:
        receipt = show(repo, revision, receipt_name)
        if "ACCEPT" not in receipt.upper():
            raise SystemExit(f"{wave}: receipt does not contain an acceptance verdict")
        ledger = json.loads(show(repo, revision, ledger_name))
        if ledger.get("upstream_ref") != PIN:
            raise SystemExit(f"{wave}: wrong upstream pin")
        rows = ledger.get("cases", [])
        if ledger.get("case_count") != len(rows):
            raise SystemExit(f"{wave}: case_count does not match rows")

        status = collections.Counter(row["status"] for row in rows)
        outcome = collections.Counter(row["outcome"] for row in rows)
        for row in rows:
            case_id = row["id"]
            if case_id in ids:
                raise SystemExit(f"duplicate accepted case: {case_id}")
            ids.add(case_id)
            if row["status"] == "pending":
                blockers[row["upstream"]] += 1

        waves.append(
            {
                "wave": wave,
                "receipt": revision,
                "receipt_path": f"{BASE}/{receipt_name}",
                "ledger": ledger_name,
                "accounted": len(rows),
                "pass": outcome["pass"],
                "expected_red": outcome["expected-red"],
                "not_applicable": outcome["not-applicable"],
                "pending": status["pending"],
            }
        )
        totals["accounted"] += len(rows)
        for key, value in status.items():
            totals[f"status_{key}"] += value
        for key, value in outcome.items():
            totals[f"outcome_{key}"] += value

    provisional_totals: collections.Counter[str] = collections.Counter()
    provisional_waves: list[dict[str, object]] = []
    for wave, revision, ledger_name, receipt_name, reason in PROVISIONAL:
        ledger = json.loads(show(repo, revision, ledger_name))
        rows = ledger.get("cases", [])
        status = collections.Counter(row["status"] for row in rows)
        outcome = collections.Counter(row["outcome"] for row in rows)
        overlap = ids.intersection(row["id"] for row in rows)
        if overlap:
            raise SystemExit(f"provisional case overlaps accepted case: {min(overlap)}")
        provisional_waves.append(
            {
                "wave": wave,
                "receipt": revision,
                "receipt_path": f"{BASE}/{receipt_name}",
                "ledger": ledger_name,
                "reason": reason,
                "accounted": len(rows),
                "pass": outcome["pass"],
                "expected_red": outcome["expected-red"],
                "pending": status["pending"],
            }
        )
        provisional_totals["accounted"] += len(rows)
        provisional_totals["pass"] += outcome["pass"]
        provisional_totals["expected_red"] += outcome["expected-red"]

    executable = totals["outcome_pass"] + totals["outcome_expected-red"]
    executable_adapted = totals["status_adapted"] - totals["outcome_not-applicable"]
    result = {
        "pin": PIN,
        "accepted_waves": waves,
        "totals": {
            "accounted": totals["accounted"],
            "non_pending": totals["accounted"] - totals["status_pending"],
            "executable": executable,
            "pass": totals["outcome_pass"],
            "expected_red": totals["outcome_expected-red"],
            "not_applicable": totals["outcome_not-applicable"],
            "executable_direct": totals["status_direct"],
            "executable_differential": totals["status_differential"],
            "executable_adapted": executable_adapted,
            "all_adapted_including_not_applicable": totals["status_adapted"],
            "pending": totals["status_pending"],
        },
        "provisional_waves": provisional_waves,
        "provisional_totals": dict(provisional_totals),
        "upstream_denominator": 1404,
        "untouched_unaccounted": 1404
        - totals["accounted"]
        - provisional_totals["accounted"],
        "pending_by_upstream_file": [
            {"upstream": upstream, "pending": count}
            for upstream, count in sorted(
                blockers.items(), key=lambda item: (-item[1], item[0])
            )
        ],
    }
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
