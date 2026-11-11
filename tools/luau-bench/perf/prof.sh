#!/bin/bash
# usage: prof.sh <luau-bench binary> <scripts dir> <filter> <out-name> [vm]
# Samples one luau-bench script (one pass) with samply at 4 kHz on the given
# VM (default nuxie) and prints prof_ops.py's function and opcode attribution.
# The binary needs line tables and a packed dSYM:
#   CARGO_PROFILE_RELEASE_DEBUG=line-tables-only CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO=packed \
#     cargo build --release -p luau-bench --bin luau-bench
# Measurement only (docs/evidence/luau-safe-vm-perf-map-2026-11-11.md).
set -eu
bin="$1"; scripts="$2"; filter="$3"; name="$4"; vm="${5:-nuxie}"
here="$(cd "$(dirname "$0")" && pwd)"
out="${PROF_OUT:-$(dirname "$bin")/prof}"
mkdir -p "$out"
dsym="$bin.dSYM/Contents/Resources/DWARF/$(basename "$bin")"
[ -f "$dsym" ] || dsym="$(ls "$(dirname "$bin")"/deps/luau_bench-*.dSYM/Contents/Resources/DWARF/* | head -1)"
samply record --save-only --unstable-presymbolicate -r 4000 -o "$out/$name.json.gz" -- \
  "$bin" run --scripts "$scripts" --vm "$vm" --filter "$filter" --runs 1 > "$out/$name.run.txt" 2>&1
python3 "$here/prof_ops.py" "$out/$name.json.gz" "$(basename "$bin")" "$dsym" \
  "$here/../../../crates/nuxie-luau-vm/src/interp.rs" --top "${TOP:-25}" | tee "$out/$name.txt"
