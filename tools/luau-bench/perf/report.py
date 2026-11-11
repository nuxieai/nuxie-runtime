#!/usr/bin/env python3
"""Before/after table of nuxie-luau-vm retired instructions per bench.

usage: report.py BASELINE.json RESULTS.json

Both files are `luau-bench run` results taken with
LUAU_BENCH_CLOCK=instructions (values are millions of retired instructions
per timed run). Each bench's value is the minimum of its per-pass medians;
its spread is (max / min - 1) of those per-pass medians, the run-to-run
noise seen in that file.

For each bench the table prints before, after, the change, and the larger of
the two spreads. A change no bigger than that spread is marked `~` (inside
the noise this measurement has shown); a larger one `+` (more instructions)
or `-` (fewer). This is an instrument, not a gate: the exit status is 0
whatever the numbers say (2 only for a usage or file error). See
docs/evidence/luau-safe-vm-perf-map-2026-11-11.md §7.
"""
import json
import math
import sys


def load(path):
    doc = json.load(open(path))
    benches = {}
    for bench, entry in doc.get("benches", {}).items():
        passes = entry.get("pass_medians_ms") or []
        if entry.get("status") == "ok" and passes:
            low = min(passes)
            benches[bench] = (low, max(passes) / low - 1 if low > 0 else 0.0)
    meta = doc.get("meta", {})
    if meta.get("clock") != "instructions":
        print(f"report: {path}: meta.clock is {meta.get('clock')!r}, not 'instructions'; "
              "these may be milliseconds", file=sys.stderr)
    return doc, benches


def main():
    if len(sys.argv) != 3:
        print(__doc__, file=sys.stderr)
        return 2
    try:
        bdoc, before = load(sys.argv[1])
        adoc, after = load(sys.argv[2])
    except (OSError, ValueError) as error:
        print(f"report: {error}", file=sys.stderr)
        return 2

    rows = []
    for bench in sorted(set(before) | set(after)):
        if bench not in before or bench not in after or before[bench][0] <= 0:
            # Missing on one side, or a zero count (no instruction counter on
            # this OS: the clock reads 0 off macOS).
            rows.append((0.0, bench, before.get(bench), after.get(bench)))
            continue
        rows.append((after[bench][0] / before[bench][0], bench, before[bench], after[bench]))
    rows.sort(key=lambda row: (row[0] == 0.0, -row[0]))

    width = max(len(bench) for _, bench, _, _ in rows)
    print(f"{'bench':<{width}} {'before M':>12} {'after M':>12} {'change':>9} {'spread':>7}")
    changed = {"+": 0, "-": 0, "~": 0}
    logs = []
    for ratio, bench, b, a in rows:
        if b is None or a is None:
            print(f"{bench:<{width}} {'-' if b is None else f'{b[0]:.3f}':>12} "
                  f"{'-' if a is None else f'{a[0]:.3f}':>12}   {'new' if b is None else 'missing/failed'}")
            continue
        if b[0] <= 0:
            print(f"{bench:<{width}} {b[0]:12.3f} {a[0]:12.3f}   zero count (no instruction counter?)")
            continue
        change = ratio - 1
        spread = max(b[1], a[1])
        mark = "~" if abs(change) <= spread else ("+" if change > 0 else "-")
        changed[mark] += 1
        logs.append(math.log(ratio))
        print(f"{bench:<{width}} {b[0]:12.3f} {a[0]:12.3f} {change * 100:+8.2f}% {spread * 100:6.2f}% {mark}")

    print()
    if logs:
        print(f"geomean after/before {math.exp(sum(logs) / len(logs)):.4f} over {len(logs)} benches; "
              f"{changed['-']} fewer instructions, {changed['+']} more, {changed['~']} inside the noise "
              "(change <= the larger per-bench spread of the two files)")
    for label, doc in (("before", bdoc), ("after", adoc)):
        meta = doc.get("meta", {})
        print(f"{label}: commit {meta.get('commit', '?')}, build {meta.get('build', '?')}, "
              f"{doc.get('runs', '?')} pass(es), date {meta.get('date', '?')}")
    if (bdoc.get("meta", {}).get("build"), bdoc.get("meta", {}).get("toolchain")) != (
            adoc.get("meta", {}).get("build"), adoc.get("meta", {}).get("toolchain")):
        print("note: the two files come from different builds or toolchains; codegen alone moved single "
              "benches by -2.8% to +3.5% between two builds of identical source")
    print("note: unrelated code changes also shift inlining and layout; read single-bench changes under "
          "about 3% as unproven even when they exceed the run spread")
    print("report only: this never fails on a change (docs/evidence/luau-safe-vm-perf-map-2026-11-11.md §7)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
