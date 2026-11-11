#!/usr/bin/env python3
"""Inline-aware function and opcode attribution for a samply profile of luau-bench.

usage: prof_ops.py PROFILE.json.gz BINARY_NAME DWARF INTERP_RS [--top N]

Every sampled address of BINARY_NAME is expanded through inlining with
llvm-symbolizer (fat-LTO code is attributed to the source functions inlined
into it). Prints:

  * self: the innermost source function of each sample;
  * inclusive: every source function on the expanded stack;
  * opcodes: for samples under the interpreter loop, the innermost frame in
    INTERP_RS, mapped to the `match` arm (opcode) whose line range holds it.
    "dispatch" is the fetch/decode before the `match`; "frame-entry" is the
    per-frame setup in `run_frames` above the inner loop.

Measurement only (docs/evidence/luau-safe-vm-perf-map-2026-11-11.md).
"""
import argparse
import collections
import gzip
import json
import re
import subprocess

ap = argparse.ArgumentParser()
ap.add_argument("profile")
ap.add_argument("binary_name")
ap.add_argument("dwarf")
ap.add_argument("interp_rs")
ap.add_argument("--top", type=int, default=30)
ap.add_argument("--symbolizer", default="/opt/homebrew/opt/llvm/bin/llvm-symbolizer")
args = ap.parse_args()

# ---- interp.rs line -> opcode arm ------------------------------------------
src = open(args.interp_rs).read().split("\n")
loop_line = next(i + 1 for i, l in enumerate(src) if "match insn_op(insn) {" in l)
frame_line = next(i + 1 for i, l in enumerate(src) if "'frame: loop {" in l)
end_line = next(i + 1 for i, l in enumerate(src) if '_ => return Err(self.runtime_error("invalid opcode"))' in l)
arm_indent = None
line_op = {}
current = None
pending = []
for n in range(loop_line + 1, end_line + 1):
    text = src[n - 1]
    stripped = text.lstrip()
    indent = len(text) - len(stripped)
    starts_arm = re.match(r"(o @ \()?\|? ?op::[A-Z0-9_]+", stripped)
    if arm_indent is None and starts_arm:
        arm_indent = indent
    if indent == arm_indent and (starts_arm or stripped.startswith("|")):
        if not pending:
            pending = []
        pending += re.findall(r"op::([A-Z0-9_]+)", stripped)
        if "=>" in stripped:
            current = "|".join(dict.fromkeys(pending))
            pending = []
    elif indent == arm_indent and stripped.startswith("_ =>"):
        current = "invalid"
    line_op[n] = current or "dispatch"


def op_for(line):
    if line in line_op:
        return line_op[line]
    if line == 0:
        return "no-line (compiler-merged code)"
    if frame_line <= line <= loop_line:
        return "dispatch" if line >= loop_line - 10 else "frame-entry"
    return f"interp.rs:{line}"


# ---- profile ----------------------------------------------------------------
p = json.load(gzip.open(args.profile))
lib_index = next(i for i, lib in enumerate(p["libs"]) if lib["name"] == args.binary_name)
t = p["threads"][0]
frames, stacks, res, funcs = t["frameTable"], t["stackTable"], t["resourceTable"], t["funcTable"]
strings = t.get("stringArray") or t.get("stringTable") or p.get("shared", {}).get("stringArray", [])
samples = t["samples"]


def frame_lib(f):
    resource = funcs["resource"][frames["func"][f]]
    return res["lib"][resource] if resource is not None and resource >= 0 else None


def walk(stack):
    out = []
    while stack is not None:
        out.append(stacks["frame"][stack])
        stack = stacks["prefix"][stack]
    return out


sample_stacks = [walk(s) if s is not None else [] for s in samples["stack"]]
weights = samples.get("weight") or [1] * len(sample_stacks)
queries = set()
for st in sample_stacks:
    for depth, f in enumerate(st):
        if frame_lib(f) == lib_index:
            queries.add(frames["address"][f] - (0 if depth == 0 else 1))
queries = sorted(queries)
out = subprocess.run(
    [args.symbolizer, "--inlining", "--functions=linkage", "--demangle", "--obj", args.dwarf],
    input="\n".join(hex(0x100000000 + a) for a in queries), capture_output=True, text=True,
).stdout
chains = {}
for address, block in zip(queries, out.strip("\n").split("\n\n")):
    lines = block.split("\n")
    chains[address] = [(lines[i], lines[i + 1] if i + 1 < len(lines) else "") for i in range(0, len(lines), 2)]

HASH = re.compile(r"::h[0-9a-f]{16}$")


def short(name):
    return HASH.sub("", name.replace("nuxie_luau_vm::", ""))


def other_name(f):
    name = funcs["name"][frames["func"][f]]
    return strings[name] if isinstance(name, int) and name < len(strings) else str(name)


# Subsystem buckets: a sample counts once per bucket whose pattern matches
# any frame of its expanded stack (buckets overlap; they are not a partition).
BUCKETS = [
    ("gc (luac_step/full_gc/collect)", re.compile(r"luac_step|full_gc|::collect\b|Vm>::collect|gc::")),
    ("string intern (Heap::intern)", re.compile(r"Heap>::intern\b|Heap>::intern$")),
    ("number formatting (num2str/fmt)", re.compile(r"num2str|flt2dec|core::fmt::write|format_inner")),
    ("native call path (push_native/step_native)", re.compile(r"push_native|step_native")),
    ("lua call path (push_call/enter_lua)", re.compile(r"push_call|enter_lua")),
    ("return path (deliver/move_results)", re.compile(r"Vm>::deliver|move_results")),
    ("table get (get_table_step)", re.compile(r"get_table_step")),
    ("table set (set_table_step/raw_set)", re.compile(r"set_table_step|Vm>::raw_set")),
    ("coroutine switch (resume/yield/switch_to)", re.compile(r"resume_thread|yield_thread|switch_to")),
    ("allocation (malloc/free/realloc)", re.compile(r"malloc|free|realloc|__rust_alloc|__rust_dealloc|_szone|nanov2")),
]
# Leaf-only: system-library code samply left unsymbolized (malloc, free,
# memmove, ...): counted on the innermost frame only.
SYSTEM_LEAF = re.compile(r"^0x[0-9a-f]+$")
bucket_counts = collections.Counter()
self_counts = collections.Counter()
incl_counts = collections.Counter()
op_counts = collections.Counter()
op_self = collections.defaultdict(collections.Counter)
total = 0
interp_total = 0
for st, weight in zip(sample_stacks, weights):
    expanded = []  # (function, location), innermost first
    for depth, f in enumerate(st):
        if frame_lib(f) == lib_index:
            a = frames["address"][f] - (0 if depth == 0 else 1)
            expanded.extend((short(fn), loc) for fn, loc in chains.get(a, [("??", "")]))
        else:
            expanded.append((other_name(f), ""))
    if not expanded:
        continue
    total += weight
    self_counts[expanded[0][0]] += weight
    names = {fn for fn, _ in expanded}
    for n in names:
        incl_counts[n] += weight
    for label, pattern in BUCKETS:
        if any(pattern.search(n) for n in names):
            bucket_counts[label] += weight
    if SYSTEM_LEAF.match(expanded[0][0]):
        bucket_counts["system library leaf (unsymbolized: malloc/free/memmove)"] += weight
    for fn, loc in expanded:
        if loc.split(":")[0].endswith("/interp.rs") and "run_frames" in fn:
            op = op_for(int(loc.split(":")[1]))
            op_counts[op] += weight
            op_self[op][expanded[0][0]] += weight
            interp_total += weight
            break


def pct(c, d):
    return c / max(d, 1) * 100


print(f"samples: {total}; under run_frames: {interp_total} ({pct(interp_total, total):.1f}%)")
print("== buckets (inclusive, overlapping)")
for label in [b[0] for b in BUCKETS] + ["system library leaf (unsymbolized: malloc/free/memmove)"]:
    print(f"{bucket_counts[label]:7d} {pct(bucket_counts[label], total):5.1f}%  {label}")
print("== self (innermost source function)")
for n, c in self_counts.most_common(args.top):
    print(f"{c:7d} {pct(c, total):5.1f}%  {n[:150]}")
print("== inclusive")
for n, c in incl_counts.most_common(args.top):
    print(f"{c:7d} {pct(c, total):5.1f}%  {n[:150]}")
print("== opcode (inclusive of callees; % of all samples), top self functions under it")
for op, c in op_counts.most_common(args.top):
    tops = ", ".join(f"{n.split('::')[-1] if '::' in n else n} {pct(k, c):.0f}%" for n, k in op_self[op].most_common(3))
    print(f"{c:7d} {pct(c, total):5.1f}%  {op:<28} {tops[:150]}")
