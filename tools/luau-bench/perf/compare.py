#!/usr/bin/env python3
"""Per-bench nuxie/luaur ratio from two `luau-bench run` result files.

usage: compare.py NUXIE.json LUAUR.json [--bakeoff BAKEOFF.md] [--markdown]

Each bench's value is the minimum of its per-pass medians (with
LUAU_BENCH_CLOCK=instructions these are millions of retired instructions per
run; host load only adds to a count, so the minimum is the least disturbed).
Prints the ratios sorted, the geomean and the median.

With --bakeoff, also derives a ratio to C++ Luau: nuxie/luaur (this run)
times luaur/cpp (wall time, docs/evidence/luau-safe-vm-bakeoff-2026-10-31.md,
section 2). The bake-off has one row per script, so a script split into
several benches here gets that script's ratio for each of them (marked `s`).
This mixes an instruction ratio with a time ratio: it is a derived estimate.

Measurement only (docs/evidence/luau-safe-vm-perf-map-2026-11-11.md).
"""
import argparse
import json
import math
import re
import statistics

ap = argparse.ArgumentParser()
ap.add_argument("nuxie")
ap.add_argument("luaur")
ap.add_argument("--bakeoff")
ap.add_argument("--markdown", action="store_true")
args = ap.parse_args()


def load(path):
    doc = json.load(open(path))
    out = {}
    for bench, entry in doc["benches"].items():
        if entry.get("status") == "ok":
            out[bench] = min(entry["pass_medians_ms"])
    return doc, out


ndoc, nuxie = load(args.nuxie)
ldoc, luaur = load(args.luaur)

cpp_ratio = {}
if args.bakeoff:
    text = open(args.bakeoff).read()
    for line in text.split("\n"):
        m = re.match(r"^((?:micro_tests|tests)__\S+)\s+([\d.]+)\s+([\d.]+)\s+([\d.]+)\s+", line)
        if m:
            cpp, luaur_ms = float(m.group(2)), float(m.group(4))
            if cpp > 0:
                cpp_ratio[m.group(1)] = luaur_ms / cpp

rows = []
for bench in sorted(set(nuxie) & set(luaur)):
    ratio = nuxie[bench] / luaur[bench]
    script = bench.split("/")[0]
    lc = cpp_ratio.get(bench)
    mark = ""
    if lc is None and script in cpp_ratio:
        lc, mark = cpp_ratio[script], "s"
    rows.append((ratio, bench, luaur[bench], nuxie[bench], lc, mark))
rows.sort()

only = sorted(set(nuxie) ^ set(luaur))


def geo(values):
    return math.exp(sum(math.log(v) for v in values) / len(values))


ratios = [r[0] for r in rows]
if args.markdown:
    print("| # | bench | luaur M instr | nuxie M instr | nuxie/luaur | luaur/cpp (bake-off) | nuxie/cpp (derived) |")
    print("| ---: | --- | ---: | ---: | ---: | ---: | ---: |")
    for i, (ratio, bench, l, n, lc, mark) in enumerate(rows, 1):
        lcs = f"{lc:.2f}{mark}" if lc else "-"
        ncs = f"{ratio * lc:.2f}{mark}" if lc else "-"
        print(f"| {i} | `{bench}` | {l:.3f} | {n:.3f} | {ratio:.2f}x | {lcs} | {ncs} |")
else:
    for ratio, bench, l, n, lc, mark in rows:
        ncs = f"{ratio * lc:7.2f}{mark}" if lc else "      -"
        print(f"{ratio:7.2f}x {ncs}  {l:12.3f} {n:12.3f}  {bench}")
print()
print(f"benches compared: {len(rows)}; only in one file: {only or 'none'}")
print(f"nuxie/luaur geomean {geo(ratios):.3f}x, median {statistics.median(ratios):.3f}x, "
      f"min {min(ratios):.2f}x, max {max(ratios):.2f}x")
derived = [r[0] * r[4] for r in rows if r[4]]
exact = [r[0] * r[4] for r in rows if r[4] and not r[5]]
nonerr = [r[0] * r[4] for r in rows if r[4] and "Failure" not in r[1] and "string_lib" not in r[1]]
if derived:
    print(f"derived nuxie/cpp over {len(derived)} benches: geomean {geo(derived):.3f}x, median {statistics.median(derived):.3f}x; "
          f"{len(exact)} with an exact bake-off row: geomean {geo(exact):.3f}x; "
          f"{len(nonerr)} without error-path benches: geomean {geo(nonerr):.3f}x")
for name, doc in (("luaur", ldoc), ("nuxie", ndoc)):
    loads = [(p.get("load_before") or [None])[0] for p in doc.get("passes", [])]
    print(f"{name}: passes {[p.get('seconds') for p in doc.get('passes', [])]} s, 1-min load at pass start {loads}")
