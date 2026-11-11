# Safe Luau VM: measured gap map against luaur (2026-11-11)

A speed map of `crates/nuxie-luau-vm` against the vendored luaur VM
(`vendor/luaur-*-0.1.8`) on the full upstream bench suite (87 benches), with
the gap attributed to code and a ranked list of changes. Measurement and
analysis only: no VM code changed. Branch `claude/luau-vm-perf-map`, based on
`origin/main` `94b0d21fcb` (lanes 1 to 6 merged).

**Outcome.**

- **nuxie/luaur: geomean 4.48x, median 3.84x** over all 87 benches, in
  retired instructions per timed run. Range 1.39x (`table: concat (big)`) to
  210x (`TableMove_same_table`). Lane 1's 4.45x was wall time on 59 benches;
  this is the first number on all 87.
- **nuxie/C++ (derived, not measured): geomean about 3.9x over all 87, 5.3x
  without the error-path benches.** Derived by multiplying this run's
  instruction ratio by the 2026-10-31 bake-off's luaur/C++ wall ratio (§1.3).
- **The gap comes from five design choices more than from tuning** (§4):
  one frame-and-continuation protocol for every native call, even
  `math.abs`; a 128-byte all-purpose frame with calls and returns leaving
  the dispatch loop; panic-freedom realized as checked access at every use
  instead of validation at the boundary; every table access through the
  general metamethod step with memory accounting on each store; and library
  and string code built on one-value-at-a-time operations (plus a weak
  interner hash and a two-pass number formatter).
- **Each has a simpler design that removes the work** (§5.1), ranked by how
  much it simplifies as well as saves: pure builtins as plain functions over
  argument slices; small `Copy` Lua frames with calls kept in the loop; a
  validated `u8`-indexed register window; raw table paths in the
  instruction with accounting at reallocation; range operations and a
  stored-hash string store. None needs `unsafe` or PGO. Local tuning of the
  current design is listed separately (§5.2).
- **The counts are now an instrument, not a gate** (§7): a committed
  per-bench table for all 87 benches and `make luau-vm-perf-report`, which
  prints before/after with the measured run-to-run spread and never fails.

Measured vs inferred: §1 to §4 are measured (counts, sampled profiles,
disassembly), except where a line says *inferred*. §5's expected sizes are
inferred from the measurements and are not verified by patches. §6 lists what
was not measured. §7 is the count table and report.

## 1. Method

### 1.1 Build and harness

- `tools/luau-bench` (`luau-bench run`), unchanged except for an opt-in
  instruction clock (§1.2). Scripts from `tests/luau-bench/merge.py` (72
  scripts, 87 bench ids; the 7 experimental-`class` scripts excluded as
  before).
- Release profile from the workspace: `lto = "fat"`, `codegen-units = 1`,
  `panic = "unwind"`. Built with cargo into a private target dir
  (`target-perf/`, deleted afterwards), with `CARGO_PROFILE_RELEASE_DEBUG=
  line-tables-only` and `CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO=packed` so the
  same binary could be profiled. Line tables do not change codegen. rustc
  1.97.1, Apple M5 Max (18 CPUs), macOS 26.6.2.
- Both VMs in one binary, same compiler front end. luaur runs on a fresh
  coroutine of a sandboxed state; nuxie on the sandboxed main state (as the
  harness has done since lane 1).

### 1.2 The instruction clock

The host was loaded (1-minute load 12 to 26 throughout, from other lanes),
so wall time was not used. With `LUAU_BENCH_CLOCK=instructions`, both VMs
get a host `os.clock` that returns this process's retired instructions / 1e9
(macOS `proc_pid_rusage(RUSAGE_INFO_V4).ri_instructions`, user and kernel;
the same counter `tools/rust-golden-runner` reads, declared in
`tools/luau-bench/src/vm.rs` without the `libc` crate so the tool's
dependencies and the Bazel lockfiles are unchanged). `bench_support.lua` is
unchanged: each bench's 20 kept "ms" values become millions of retired
instructions per timed run, and the reported value is their median.

- 3 whole-suite passes per VM, both VMs at once. Each bench's value is the
  **minimum of its 3 per-pass medians** (load only adds to a count;
  `fat-lto-counts-min-of-3`).
- **Noise is small.** Pass-to-pass spread (max/min − 1) per bench: luaur
  median 0.13%, p90 0.35%, max 2.0%; nuxie median 0.12%, p90 0.67%, max 4.2%
  (both maxima on `Failure_pcall_a_bar`).
- Pass wall times, for the record only: luaur 63.4, 59.6, 73.1 s; nuxie
  338.8, 346.6, 322.5 s (1-minute load at pass start 26.2, 19.0, 17.6 and
  26.2, 12.7, 19.0).
- The two `os.clock` host calls per run cost a few thousand instructions,
  which is noise even on the smallest bench (luaur `TableMove_*`, 1.5 M).
- Raw results: `docs/evidence/luau-safe-vm-perf-map-2026-11-11/instr-{nuxie,luaur}.json`.

### 1.3 The C++ column (derived)

No C++ Luau build was run. The C++ column multiplies this run's nuxie/luaur
instruction ratio by luaur/C++ **wall time** from
`docs/evidence/luau-safe-vm-bakeoff-2026-10-31.md` §2 (quiet host, median of
20). The bake-off has one row per script, so a script split into several
benches here (`StringInterp`, `ToNumberString`, `TableSort`, `string_lib`,
`table_concat`) gets its script's ratio for every bench (marked `s`). This
assumes luaur's instructions-per-cycle equals C++'s, so read it as an
estimate. The bake-off's error-path benches (`Failure_*`, `string_lib`) have
luaur at 0.03x to 0.21x of C++ (Rust `Result` vs C++ `throw`), which pulls
the all-bench geomean down.

| Derived nuxie/C++ | Benches | Geomean | Median |
| --- | ---: | ---: | ---: |
| All | 87 | 3.90x | 3.97x |
| Exact bake-off row (no `s`) | 67 | 3.86x | - |
| Without `Failure_*` and `string_lib` | 79 | 5.25x | - |

### 1.4 Profiles and code

- `samply` at 4 kHz on one pass of one script (nuxie VM), expanded through
  fat-LTO inlining with `llvm-symbolizer` by `tools/luau-bench/perf/prof_ops.py`.
  It prints the innermost source function per sample, inclusive counts,
  overlapping subsystem buckets, and **per-opcode attribution**: for samples
  under `run_frames`, the innermost `interp.rs` line is mapped to its `match`
  arm. `dispatch` is the fetch/decode before the `match`; `frame-entry` is
  `run_frames`'s per-frame setup above the inner loop, which re-runs after
  every call and return.
- Profiles cover the whole script, including untimed setup and the
  `collectgarbage()` between runs, so the GC share is an upper bound for the
  timed region. A script with several benches (`StringInterp`) is profiled as
  one.
- Disassembly of `Vm::run_frames` with `llvm-objdump -l` (36 KB of code,
  11.6k lines with source lines).
- Saved outputs: `docs/evidence/luau-safe-vm-perf-map-2026-11-11/prof-*.txt`.

## 2. Results

### 2.1 Per bench, sorted

Columns: millions of retired instructions per timed run (min of 3 passes);
the ratio; the bake-off's luaur/C++ wall ratio; and the derived nuxie/C++
ratio (§1.3).

| # | bench | luaur M instr | nuxie M instr | nuxie/luaur | luaur/cpp (bake-off) | nuxie/cpp (derived) |
| ---: | --- | ---: | ---: | ---: | ---: | ---: |
| 1 | `micro_tests__test_table_concat/table: concat (big)` | 203.565 | 283.005 | 1.39x | 0.79s | 1.10s |
| 2 | `micro_tests__test_TableCreate_number` | 297.826 | 442.115 | 1.48x | 1.08 | 1.61 |
| 3 | `micro_tests__test_TableCreate_nil` | 234.707 | 355.261 | 1.51x | 1.08 | 1.63 |
| 4 | `micro_tests__test_string_lib/string: reverse/upper/lower (large)` | 25.030 | 41.379 | 1.65x | 0.21s | 0.35s |
| 5 | `micro_tests__test_StringInterp/interp: %f format` | 3863.430 | 6750.496 | 1.75x | 1.08s | 1.88s |
| 6 | `micro_tests__test_OOP_method_access` | 2452.715 | 4373.789 | 1.78x | 1.06 | 1.89 |
| 7 | `micro_tests__test_Failure_xpcall_a_bar` | 42.758 | 78.256 | 1.83x | 0.03 | 0.06 |
| 8 | `micro_tests__test_Failure_xpcall_game_Foo` | 43.932 | 81.452 | 1.85x | 0.03 | 0.06 |
| 9 | `micro_tests__test_Failure_pcall_game_Foo` | 39.182 | 72.646 | 1.85x | 0.03 | 0.05 |
| 10 | `micro_tests__test_Failure_pcall_a_bar` | 37.192 | 69.585 | 1.87x | 0.03 | 0.06 |
| 11 | `micro_tests__test_TableFind_table_find` | 108.121 | 206.933 | 1.91x | 0.98 | 1.87 |
| 12 | `micro_tests__test_LargeTableCtor_hash` | 67.145 | 129.518 | 1.93x | 0.91 | 1.75 |
| 13 | `micro_tests__test_SqrtSum_exponent` | 53.017 | 114.041 | 2.15x | 1.00 | 2.15 |
| 14 | `micro_tests__test_StringInterp/interp: interp string` | 38.008 | 87.014 | 2.29x | 1.08s | 2.47s |
| 15 | `micro_tests__test_AbsSum_and_or` | 149.522 | 351.357 | 2.35x | 1.01 | 2.38 |
| 16 | `micro_tests__test_TableIteration` | 99.507 | 241.660 | 2.43x | 1.39 | 3.37 |
| 17 | `micro_tests__test_Pcall_xpcall_return` | 93.311 | 228.178 | 2.45x | 0.97 | 2.37 |
| 18 | `micro_tests__test_table_concat/table: concat (small)` | 293.467 | 718.801 | 2.45x | 0.79s | 1.94s |
| 19 | `micro_tests__test_StringInterp/interp: %s format` | 788.475 | 1936.920 | 2.46x | 1.08s | 2.65s |
| 20 | `micro_tests__test_LargeTableSum_loop_iter` | 84.355 | 214.779 | 2.55x | 1.26 | 3.22 |
| 21 | `micro_tests__test_TableMarshal_varargs` | 127.200 | 337.089 | 2.65x | 1.25 | 3.32 |
| 22 | `micro_tests__test_string_lib/string: rep (small)` | 894.660 | 2374.565 | 2.65x | 0.21s | 0.57s |
| 23 | `micro_tests__test_LargeTableSum_loop_ipairs` | 91.147 | 254.053 | 2.79x | 1.20 | 3.34 |
| 24 | `tests__tictactoe` | 1417.410 | 4003.701 | 2.82x | 2.28 | 6.44 |
| 25 | `micro_tests__test_TableMarshal_table_pack` | 159.607 | 453.896 | 2.84x | 1.12 | 3.18 |
| 26 | `micro_tests__test_LargeTableSum_loop_index` | 112.081 | 322.536 | 2.88x | 0.90 | 2.57 |
| 27 | `micro_tests__test_Pcall_pcall_return` | 77.635 | 223.619 | 2.88x | 1.09 | 3.14 |
| 28 | `micro_tests__test_LargeTableSum_loop_pairs` | 84.378 | 248.649 | 2.95x | 1.17 | 3.45 |
| 29 | `micro_tests__test_LargeTableCtor_array` | 90.086 | 266.492 | 2.96x | 0.89 | 2.64 |
| 30 | `tests__voxelgen` | 781.618 | 2366.410 | 3.03x | 1.31 | 3.97 |
| 31 | `micro_tests__test_TableMarshal_select` | 201.010 | 623.689 | 3.10x | 1.13 | 3.50 |
| 32 | `micro_tests__test_TableFind_loop_ipairs` | 196.617 | 616.022 | 3.13x | 0.87 | 2.72 |
| 33 | `micro_tests__test_Pcall_pcall_yield` | 343.420 | 1088.863 | 3.17x | 1.12 | 3.55 |
| 34 | `micro_tests__test_SqrtSum_sqrt_getfenv` | 150.574 | 483.255 | 3.21x | 0.83 | 2.68 |
| 35 | `micro_tests__test_SqrtSum_sqrt_roundabout` | 145.568 | 467.677 | 3.21x | 0.89 | 2.88 |
| 36 | `tests__matrixmult` | 272.807 | 899.985 | 3.30x | 1.02 | 3.36 |
| 37 | `micro_tests__test_UpvalueCapture` | 500.127 | 1662.902 | 3.32x | 1.02 | 3.38 |
| 38 | `micro_tests__test_OOP_virtual_constructor` | 94.580 | 314.860 | 3.33x | 1.19 | 3.95 |
| 39 | `tests__life` | 984.065 | 3394.521 | 3.45x | 1.37 | 4.72 |
| 40 | `micro_tests__test_TableInsertion_index_len` | 63.702 | 227.887 | 3.58x | 1.14 | 4.07 |
| 41 | `micro_tests__test_TableCreate_zerofill` | 1275.758 | 4655.738 | 3.65x | 0.89 | 3.26 |
| 42 | `micro_tests__test_OOP_constructor` | 860.520 | 3157.172 | 3.67x | 0.94 | 3.45 |
| 43 | `micro_tests__test_TableSort/table.sort: 12 strings` | 291.281 | 1084.473 | 3.72x | 0.96s | 3.57s |
| 44 | `tests__qsort` | 1012.616 | 3889.524 | 3.84x | 1.18 | 4.52 |
| 45 | `micro_tests__test_OOP_field_access` | 750.377 | 2900.441 | 3.87x | 1.26 | 4.87 |
| 46 | `tests__mesh-normal-scalar` | 349.460 | 1363.665 | 3.90x | 1.26 | 4.90 |
| 47 | `micro_tests__test_Factorial` | 665.105 | 2596.693 | 3.90x | 1.24 | 4.85 |
| 48 | `micro_tests__test_Pcall_call_return` | 21.622 | 87.819 | 4.06x | 1.23 | 5.01 |
| 49 | `micro_tests__test_TableSort/table.sort: 10k numbers` | 623.637 | 2579.176 | 4.14x | 0.96s | 3.97s |
| 50 | `micro_tests__test_TableInsertion_index_cached` | 32.769 | 137.002 | 4.18x | 0.92 | 3.86 |
| 51 | `tests__trig` | 281.017 | 1182.991 | 4.21x | 1.40 | 5.89 |
| 52 | `micro_tests__test_TableSort/table.sort: 100 numbers` | 301.953 | 1275.804 | 4.23x | 0.96s | 4.06s |
| 53 | `tests__sieve` | 1236.346 | 5226.496 | 4.23x | 1.24 | 5.25 |
| 54 | `tests__mesh-normal-vector` | 178.164 | 780.846 | 4.38x | 1.26 | 5.50 |
| 55 | `tests__vector-math` | 128.017 | 574.592 | 4.49x | 1.09 | 4.90 |
| 56 | `micro_tests__test_MethodCalls` | 524.712 | 2365.215 | 4.51x | 0.87 | 3.91 |
| 57 | `tests__chess` | 980.056 | 4553.451 | 4.65x | 1.28 | 5.94 |
| 58 | `micro_tests__test_string_lib/string: reverse/upper/lower (unique)` | 503.257 | 2423.049 | 4.81x | 0.21s | 1.03s |
| 59 | `micro_tests__test_OOP_field_access_random` | 7400.263 | 36093.928 | 4.88x | 1.05 | 5.12 |
| 60 | `micro_tests__test_OOP_method_call` | 2892.844 | 14273.212 | 4.93x | 1.03 | 5.08 |
| 61 | `micro_tests__test_StringInterp/interp: %d format` | 2491.162 | 13081.135 | 5.25x | 1.08s | 5.66s |
| 62 | `micro_tests__test_ToNumberString/tonumber` | 408.157 | 2349.335 | 5.76x | 1.16s | 6.66s |
| 63 | `tests__base64` | 251.480 | 1485.890 | 5.91x | 1.26 | 7.42 |
| 64 | `tests__pcmmix` | 115.658 | 683.630 | 5.91x | 1.16 | 6.86 |
| 65 | `micro_tests__test_TableInsertion_table_insert` | 63.429 | 380.235 | 5.99x | 1.34 | 8.05 |
| 66 | `micro_tests__test_TableMarshal_table_unpack_array` | 381.169 | 2302.727 | 6.04x | 1.19 | 7.17 |
| 67 | `micro_tests__test_TableMarshal_table_unpack_range` | 432.184 | 2649.817 | 6.13x | 1.11 | 6.82 |
| 68 | `micro_tests__test_string_lib/string: rep (large)` | 88.151 | 596.900 | 6.77x | 0.21s | 1.45s |
| 69 | `micro_tests__test_AbsSum_abs` | 138.013 | 966.232 | 7.00x | 1.17 | 8.20 |
| 70 | `micro_tests__test_SqrtSum_sqrt` | 69.009 | 483.301 | 7.00x | 1.15 | 8.10 |
| 71 | `micro_tests__test_AbsSum_math_abs` | 138.013 | 997.402 | 7.23x | 1.15 | 8.33 |
| 72 | `micro_tests__test_SqrtSum_math_sqrt` | 69.008 | 498.727 | 7.23x | 1.11 | 8.02 |
| 73 | `micro_tests__test_VariadicSelect` | 216.410 | 1606.220 | 7.42x | 1.29 | 9.59 |
| 74 | `micro_tests__test_vector_lib` | 683.289 | 5119.227 | 7.49x | 0.92 | 6.93 |
| 75 | `tests__sha256` | 301.474 | 2492.003 | 8.27x | 1.42 | 11.72 |
| 76 | `micro_tests__test_Assert` | 101.013 | 837.342 | 8.29x | 1.40 | 11.57 |
| 77 | `micro_tests__test_StringInterp/interp: tostring` | 1359.316 | 21178.929 | 15.58x | 1.08s | 16.80s |
| 78 | `micro_tests__test_StringInterp/interp: concat` | 1294.589 | 20395.832 | 15.75x | 1.08s | 16.99s |
| 79 | `micro_tests__test_ToNumberString/tostring` | 518.169 | 9153.517 | 17.67x | 1.16s | 20.43s |
| 80 | `micro_tests__test_TableMove_empty_table` | 2.759 | 51.957 | 18.83x | 1.08 | 20.28 |
| 81 | `micro_tests__test_StringInterp/interp: interp number` | 1126.272 | 21347.605 | 18.95x | 1.08s | 20.44s |
| 82 | `micro_tests__test_StringInterp/interp: %* format` | 1078.343 | 21019.004 | 19.49x | 1.08s | 21.02s |
| 83 | `micro_tests__test_TableInsertion_table_insert_index` | 21.012 | 429.990 | 20.46x | 1.21 | 24.66 |
| 84 | `micro_tests__test_ToNumberString/tostring-gc` | 675.948 | 17304.440 | 25.60x | 1.16s | 29.61s |
| 85 | `micro_tests__test_TableRemoval_table_remove` | 15.313 | 420.005 | 27.43x | 1.04 | 28.60 |
| 86 | `micro_tests__test_TableMove_table_create` | 1.508 | 51.516 | 34.16x | 1.29 | 43.92 |
| 87 | `micro_tests__test_TableMove_same_table` | 1.508 | 316.326 | 209.77x | 1.00 | 209.77 |

**Summary.** nuxie/luaur geomean **4.484x**, median **3.841x**, min 1.39x,
max 209.77x, over 87 of 87.

### 2.2 Per operation (measured)

Instructions per loop iteration, from benches whose timed loop is one
operation and a known trip count (count / iterations):

| Bench | Loop body | Iterations | luaur | nuxie | Extra per iteration |
| --- | --- | ---: | ---: | ---: | ---: |
| `AbsSum_math_abs` | `sum = sum + math.abs(i)` | 1,000,001 | 138 | 997 | +859 |
| `SqrtSum_sqrt` | `sum = sum + sqrt(i)` (local) | 500,001 | 138 | 967 | +829 |
| `Assert` | `assert(i)` | 1,000,001 | 101 | 837 | +736 |
| `Factorial` | one `fact(n)` call + return, `n * fact(n-1)` | 2,575,000 calls | 258 | 1008 | +750 |
| `OOP_method_call` | `n:Get()` (NAMECALL, CALL, GETTABLEKS, RETURN) | 10,000,000 | 289 | 1427 | +1138 |
| `LargeTableSum_loop_index` | `sum = sum + t[i]` | 1,000,000 | 112 | 323 | +211 |

A builtin call costs about 830 to 860 instructions more than on luaur, a Lua
call and return about 750 more, an array read in a loop about 210 more.

## 3. Attribution

### 3.1 By what dominates each bench

Each bench is assigned to one group by its hot loop (and its profile where
one was taken); `tools/luau-bench/perf/groups.py`. The last column is the
group's share of the suite's total log-gap (sum of log ratios), so it says
how much of the 4.48x geomean each group carries.

| group | benches | geomean nuxie/luaur | range | share of suite log-gap |
| --- | ---: | ---: | --- | ---: |
| builtin calls (FASTCALL-able: math, assert, bit32, select, vector, unpack) | 21 | 5.20x | 1.91x to 8.29x | 27% |
| string building, interning and number formatting | 18 | 5.51x | 1.39x to 25.60x | 24% |
| table library bulk ops (move, insert/remove at index, create, sort) | 10 | 10.08x | 1.48x to 209.77x | 18% |
| array access and iteration | 14 | 2.95x | 1.93x to 4.18x | 12% |
| Lua calls and returns | 8 | 3.55x | 2.45x to 4.93x | 8% |
| table get/set by constant key (fields, methods, constructors) | 7 | 3.57x | 1.78x to 4.88x | 7% |
| coroutines | 2 | 3.66x | 3.17x to 4.23x | 2% |
| error path (pcall/xpcall of a failing call) | 4 | 1.85x | 1.83x to 1.87x | 2% |
| arithmetic and loops | 2 | 2.25x | 2.15x to 2.35x | 1% |
| closures and upvalues | 1 | 3.32x | 3.32x | 1% |

There is no "GC and allocation" group: no bench is dominated by it, but GC
shows up inside several (§3.2). Even the cheapest groups sit at 2 to 3x,
which is the interpreter-core floor (dispatch, register access, call frame).

### 3.2 Profiles: subsystem buckets

Percent of samples whose expanded stack contains the subsystem (buckets
overlap). `syslib` is samples whose leaf is unsymbolized system-library code
(malloc, free, memmove). Worst-10 benches are covered by the `StringInterp`,
`ToNumberString`, `TableMove`, `TableRemoval` and
`TableInsertion_table_insert_index` scripts; mid-range representatives are
`tictactoe`, `LargeTableSum_loop_index`, `OOP_constructor`, `Factorial` and
`MethodCalls`.

| script | gc | intern | numfmt | native call | lua call | return | tget | tset | coro | syslib |
|---|---|---|---|---|---|---|---|---|---|---|
| test_StringInterp | 19 | 46 | 22 | 67 | 1 | 1 | 0 | 0 | 0 | 28 |
| test_ToNumberString | 12 | 32 | 35 | 94 | 3 | 1 | 0 | 0 | 0 | 31 |
| test_TableMove | 23 | 0 | 0 | 37 | 3 | 2 | 1 | 37 | 0 | 0 |
| test_TableRemoval | 4 | 0 | 0 | 97 | 0 | 0 | 0 | 72 | 0 | 4 |
| test_TableInsertion_table_insert_index | 0 | 0 | 0 | 96 | 0 | 1 | 0 | 72 | 0 | 3 |
| tests__tictactoe | 2 | 0 | 0 | 3 | 2 | 2 | 15 | 12 | 0 | 3 |
| test_LargeTableSum_loop_index | 8 | 0 | 0 | 8 | 0 | 0 | 14 | 36 | 0 | 2 |
| test_OOP_constructor | 15 | 0 | 0 | 20 | 13 | 9 | 4 | 10 | 0 | 15 |
| test_Factorial | 0 | 0 | 0 | 0 | 22 | 19 | 0 | 0 | 0 | 0 |
| test_MethodCalls | 0 | 0 | 0 | 0 | 12 | 9 | 24 | 10 | 0 | 1 |
| test_OOP_method_call | 0 | 0 | 0 | 0 | 13 | 14 | 27 | 0 | 0 | 0 |
| test_AbsSum_math_abs | 0 | 0 | 0 | 62 | 24 | 20 | 0 | 0 | 0 | 1 |
| test_Assert | 0 | 0 | 0 | 59 | 25 | 16 | 0 | 0 | 0 | 2 |
| tests__sha256 | 0 | 0 | 0 | 64 | 16 | 17 | 2 | 1 | 0 | 2 |
| tests__trig | 4 | 0 | 0 | 34 | 7 | 6 | 0 | 17 | 0 | 4 |
| test_VariadicSelect | 0 | 0 | 0 | 36 | 12 | 12 | 0 | 0 | 0 | 0 |
| test_UpvalueCapture | 43 | 0 | 0 | 21 | 0 | 0 | 0 | 14 | 0 | 37 |
| test_Pcall_pcall_yield | 18 | 0 | 0 | 80 | 12 | 9 | 0 | 0 | 17 | 28 |
| sieve | 2 | 0 | 0 | 73 | 11 | 8 | 0 | 0 | 31 | 12 |

### 3.3 Profiles: opcodes

Share of all samples, inclusive of callees, by the `interp.rs` arm they sit
under (top entries; full lists in the `prof-*.txt` files):

| Script | Top opcodes |
| --- | --- |
| `Factorial` | RETURN 28%, CALL 24%, frame-entry 14%, dispatch 13%, GETGLOBAL 9% |
| `OOP_method_call` | NAMECALL 29%, RETURN 20%, CALL 15%, frame-entry 12%, GETTABLEKS 11%, dispatch 7% |
| `MethodCalls` | NAMECALL 23%, GETTABLEKS 14%, CALL 14%, RETURN 13%, SETTABLEKS 11%, frame-entry 11%, dispatch 7% |
| `tictactoe` | dispatch 24%, GETTABLE 18%, frame-entry 16%, SETTABLE 12%, FORGLOOP 6% |
| `LargeTableSum_loop_index` | SETTABLE 38% (setup loop), GETTABLE 18%, frame-entry 13%, dispatch 9% |
| `OOP_constructor` | CALL 30%, DUPTABLE 12%, SETTABLEKS 11%, RETURN 8%, frame-entry 8% |
| `AbsSum_math_abs` | CALL 71% (the builtin), dispatch 11%, frame-entry 6% |
| `Assert` | CALL 70%, dispatch 10%, frame-entry 8% |
| `sha256` | CALL 70% (bit32), frame-entry 8%, dispatch 6% |
| `VariadicSelect` | CALL 44% (`select`), GETVARARGS 26%, dispatch 9% |
| `UpvalueCapture` | NEWCLOSURE 53% (mostly GC mark/sweep under it), CALL 21%, SETTABLE 15% |
| `StringInterp` | CALL 67% (`string.format`, `tostring`), CONCAT 24% |
| `TableMove`, `TableRemoval`, `table_insert_index` | CALL 39% to 97% (the library function), SETTABLE 39% in `TableMove` |
| `sieve` | CALL 65% (coroutine resume/yield), frame-entry 21% |

Across scripts, three handlers are the hottest: **CALL** (with the native
call path under it), **RETURN**, and the **table access arms**
(GETTABLE/GETTABLEKS/NAMECALL/SETTABLE). `frame-entry` (8 to 21%) and
`dispatch` (6 to 24%) are paid by everything.

### 3.4 What the code does (disassembly and source)

Release fat-LTO `run_frames`, read with `llvm-objdump -l`. Each item names
the source and what the machine code shows.

**Dispatch loop** (`interp.rs:347-356`), per instruction:
- `pc`, `code.ptr`, `code.len`, `k.ptr`, `k.len` live in stack slots
  (`[sp,#0x158]`, `[sp,#0x148]`, `[sp,#0x150]`, ...), not registers:
  `run_frames` has a 4.4 KB stack frame and spills them.
- `self.pc = pc` is a store to `Vm` on every instruction (`str x9,
  [x21,#0x730]`), kept so errors can name the line.
- Bounds check of `code.get(pc)`, an opcode range check (`cmp w28,#0x57;
  b.hi`), then a jump table (`ldrh` + `adr` + `add` + `br`). About 14
  instructions before the handler runs; C++ Luau's computed goto is about 5.

**Register access** (`reg!`/`set_reg!` → `Vm::get`/`Vm::set`, `vm.rs:322`):
- `base.saturating_add(i)` compiles to `adds` + `csinv` per access.
- `self.stack.len` and `self.stack.ptr` are reloaded from `Vm` on every
  access (`ldr x9,[x21,#0x590]` / `ldr x11,[x21,#0x588]`), because
  `self` is `&mut` and any store may alias them; then a bounds compare and
  branch. `FORNLOOP` reads three registers and writes one: four reloads,
  four bounds checks, plus a tag test per read (`number()` returns 0.0 on a
  non-number, so `fcsel`).
- In `tictactoe` this shows up as 16% self time in `saturating_add` and 24%
  in `dispatch`.

**Frame entry and calls** (`interp.rs:286-302`, `call.rs`):
- `Frame` is 128 bytes (`add x8,x9,x8,lsl #7`). It carries `wait: Wait`
  (an enum with `Finish`, `Box<dyn Continuation>` and `Outcome` payloads),
  `restore: Option<Saved>`, `proto: Option<Rc<Proto>>`, flags.
- Every call **and every return** goes back to the top of `'frame: loop`,
  which re-reads the top frame, **clones its `Rc<Proto>`** (`ldr; adds;
  str; b.hs` overflow-checked increment, and a decrement at the next
  `continue 'frame`), and re-spills code/k pointers and lengths.
- CALL calls `push_call` out of line (result returned through memory,
  checked against a sentinel), re-reads `frames.last()` to see whether a Lua
  frame was pushed, and for natives calls `step_native` out of line.
  `push_call` itself runs `saved_calls`, `resolve_callable`,
  `heap.function(id)` twice (match, then clone), `check_call_depth`,
  `save_caller_pc`, `ensure_stack`, two `fill_nil`s and a 128-byte push.
- RETURN runs `close_upvals`, pops the frame (moving 128 bytes out),
  `move_results` (`ensure_stack`, `copy_slots`, `fill_nil`), and `deliver`,
  which does `mem::take(&mut caller.wait)` and drops the old `Wait`
  (`drop_glue::<Wait>`: 4 to 16% self time in call-heavy profiles).
- Natives: `step_native` takes and drops the frame's `Wait`, matches the
  function, **clones the `NativeFn`** (an `Rc` increment and decrement for
  host functions), runs it, checks `self.request`, pops, `move_results`,
  `safepoint()`, `deliver`. That whole path is what a `math.abs` pays.

**FASTCALL** (`interp.rs:358-421`): only builtins listed in
`libs/fast.rs` (those whose fast result differs from the library function)
run a fast body; every other builtin falls through to the full CALL. This is
the measured +830 to +860 instructions per builtin call.

**Table access** (`meta.rs:196-271`, `table.rs`):
- GETTABLE, GETTABLEKS, GETTABLEN, SETTABLE, SETTABLEKS, SETTABLEN and the
  NAMECALL fallback all call `get_table_step`/`set_table_step` **out of
  line** (9 and 4 call sites in `run_frames`). They return
  `VmResult<Meta>`, about 88 bytes (`Meta::Call(MetaCall)` holds four
  `Value`s), through memory, and the arm copies the pieces back out.
- No per-instruction slot cache: every constant-key access hashes the key
  and probes (`find_str`); C++ Luau caches the node slot in the
  instruction's C operand.
- `Table::set` (`table.rs:333`) computes `capacity_bytes()` before and after
  every store and calls `capacity_after_new_key`, which repeats the array
  check and does **a second hash lookup** (`find_node`) before the store's
  own lookup; then `budget.settle`. `Heap::table_set` adds a write barrier.
  `set_table_step` itself does a `table.get(key)` first to decide on
  `__newindex`. A store to an existing key is three lookups.
- `array_index` (float→integer exactness check) runs on both the check and
  the store.

**Strings** (`heap.rs:265-291, 619-637`):
- Every string is interned through `HashMap<Rc<[u8]>, u32, FxHasher>`.
  `FxHasher::finish` returns the raw state. The last multiply only carries
  low bits upward, so the hash's low bits (hashbrown's bucket index) depend
  only on the low bytes of each 8-byte word. Strings that differ only in
  their last bytes, such as `"j=1"` .. `"j=1000000"`, land in the same few
  buckets.
- Measured standalone (`tools/luau-bench/perf/fx_intern_probe.rs`, std
  `HashMap`, 200k keys inserted then looked up): `"j=N"` keys take 17,578 ms
  with this hasher and 19 ms with a final `rotate_left(26)` (what
  rustc-hash 2.x does for hashbrown); `"N.000000"` keys 397 ms vs 12 ms;
  `"Nabc"` keys 87 ms vs 9 ms.
- In the bench profiles, hashbrown's group-probe loop (`likely`,
  `vreinterpret_u64_u8`, `trailing_zeros`, `lowest_set_bit`) is 42% of
  `StringInterp` samples and 30% of `ToNumberString`.
- Number to string goes through `core::fmt` (`grisu::format_shortest_opt`,
  `format_inner`, `String` writes) and a re-layout: the `numfmt` bucket is
  22 to 35% of those scripts.

**Table library** (`libs/table.rs:125-150`): `move_elements` (used by
`table.move`, `table.insert(t, pos, v)` and `table.remove(t, pos)`) builds
a `Box<dyn Iterator<Item = i64>>`, then for each element does `get_int`,
an `i64 → f64` key, and `vm.raw_set` (barrier, `Table::set` with its
accounting and second lookup). C++ Luau's `tmove` memmoves the array part
when source and destination are the same table. This is the 210x on
`TableMove_same_table` (luaur: 1.5 M instructions per run, nuxie: 316 M).

**Coroutines** (`thread.rs:165-188`): `switch_to` swaps 11 `Exec` fields
twice per switch (`swap_exec` out, `swap_exec` in), and resume/yield go
through `Request`/`Wait` boxing. `sieve`: coroutine bucket 31%, `swap_chunk`
6%, `drop_glue::<Wait>` 7%.

**Closures and GC**: `NEWCLOSURE` allocates a `Vec<Value>` for the upvalues
(a second allocation besides the arena slot) and `Rc::clone`s the proto.
`UpvalueCapture`: 43% of samples under GC (`mark_value`, `traverse`,
`sweep`) and 37% in system-library leaves (allocator). The profile includes
the untimed `collectgarbage()` calls, so this is an upper bound.

**VARARGS**: `GETVARARGS` copies through `copy_slots`/`fill_nil` with a
checked `get`/`set` per slot; `VariadicSelect` 26%.

## 4. The causes, by the design choice behind them

Each cause below is stated three ways: the measured cost, the structural
choice in the VM that produces it, and the simpler design that would not do
the work at all. Local tuning of the current design comes last (§5.2) on
purpose: the counts in §7 are an instrument for judging redesigns, not a
number to hill-climb toward.

### 4.1 One calling protocol for everything native

- **Cost (measured).** A builtin call is +830 to +860 instructions over luaur
  (§2.2). The 21 builtin-call benches sit at 5.2x and carry 27% of the
  suite's log-gap.
- **Structural choice.** There is one way to run native code: push a frame
  and step it through the resumable-native protocol (`Wait`, `Request`,
  `Continuation`, `Outcome`), which exists for the hard cases (host
  functions that yield, `pcall` continuations, natives that call back into
  Lua). Every builtin is a `fn(&mut Vm, Args) -> VmResult<usize>` that reads
  its arguments from a frame on the VM stack, so even `math.abs` needs the
  frame, the `Wait` take/drop, the `NativeFn` clone, the request check,
  `move_results`, `safepoint` and `deliver`. FASTCALL could skip this, but
  only by duplicating each builtin as a second "fast body" (`libs/fast.rs`
  has four), so it mostly falls through.
- **Simpler design.** Split natives by what they can do. Pure builtins
  (no yield, no call into Lua: almost all of `math`, `bit32`, `vector`,
  `string.byte/len/sub`, `type`, `select`, `rawget`, `assert`'s success
  path) become plain functions over a slice of argument values that write
  their results, e.g. `fn(&[Value], &mut Results) -> Result<usize, Error>`
  (plus `&mut Heap` for those that allocate strings). CALL and FASTCALL
  invoke them directly on the register window: no frame, no protocol, and
  FASTCALL needs no second body because the library function is the fast
  path. Only the natives that really need it keep the frame and
  continuation machinery. This removes the work instead of making it
  cheaper, and deletes the fast-body duplication.

### 4.2 A frame that carries every kind of call, and calls that leave the loop

- **Cost (measured).** A Lua call plus return is +750 instructions over
  luaur; `frame-entry` is 8 to 21% of samples in every profile; RETURN 13
  to 28%, CALL 14 to 24% in call-heavy ones.
- **Structural choice.** Lua frames and native frames share one 128-byte
  `Frame` that also holds the native protocol's `Wait` (boxed
  continuations, metamethod `Finish` state), C-call `restore` counters and
  an owned `Option<Rc<Proto>>`. The interpreter is two nested loops: CALL
  and RETURN leave the dispatch loop (`continue 'frame`) and the outer loop
  re-derives the running state from the frame stack, cloning the proto's
  `Rc` each time. Closures hold both an `Rc<Proto>` and a `ProtoId`.
- **Simpler design.** A Lua frame is a small `Copy` record (base, saved pc,
  proto index, closure, nresults). The dispatch loop keeps the running
  frame in locals and treats a Lua-to-Lua CALL and a RETURN to a Lua caller
  as a local state switch: push or pop the small record and keep
  dispatching. Native-protocol and metamethod-wait state lives in a side
  stack used only by frames that need it. Protos are owned by the heap and
  named by index everywhere, with no `Rc`. This removes the per-call
  `Rc` traffic, the `Wait` drop, the 128-byte moves and the outer-loop
  re-entry, and it makes `call.rs`'s common path one short function.

### 4.3 Checked access everywhere instead of validation at the boundary

- **Cost (measured).** The 2 to 3x floor in the cheapest groups (array 2.95x,
  arithmetic 2.25x, fields 3.57x): every register access is a saturating
  add, two reloads of the stack's pointer and length from `Vm`, a bounds
  branch and a fallback value; `pc` and `code` are spilled; `self.pc` is
  stored on every instruction (§3.4). `tictactoe`: dispatch 24%,
  `saturating_add` 16% self.
- **Structural choice.** Panic-freedom is achieved by making every access
  fallible-with-a-default at the point of use (`stack.get(..).unwrap_or_default()`,
  `saturating_add` on every index), and the stack is a `Vec` reached through
  `&mut self`, so nothing proven about it survives a store.
- **Simpler design.** Prove once, at the edges, and then use types that
  cannot be out of range. The loader's verifier already knows each proto's
  `maxstacksize` and checks register operands; frame entry can guarantee
  that `base + 256 <= stack.len()`. The dispatch loop then works on a
  `&mut [Value; 256]` register window indexed by `u8` operands: no
  arithmetic, no reload, no bounds check, and still no `unsafe` and no
  panic site, because a `u8` index into a 256-array is total. The saved pc
  is written only where something can observe it (calls, errors). This is
  less code than today's checked accessors, not more.

### 4.4 Tables reached through the general metamethod machinery, with accounting on every store

- **Cost (measured).** Table access is out of line (`get/set_table_step`,
  13 call sites), returns about 88 bytes through memory, has no slot cache,
  and a store to an existing key does three lookups and two capacity
  computations. tget 14 to 27%, tset 10 to 36% of samples.
- **Structural choice.** Every table instruction goes through the general
  `luaV_gettable`/`settable` equivalent whose result type can carry a whole
  pending metamethod call, even when the table has no metatable (the common
  case). Memory accounting is done per mutation (capacity before and after
  each `set`) rather than at the places capacity changes.
- **Simpler design.** Tables without a metatable (or with a raw hit) are
  handled entirely in the instruction: a raw lookup and a store. The
  metamethod path is a separate cold call that is entered only on a miss
  with a metatable, and its pending call is a small handle, not 88 bytes.
  The budget is charged where the table actually reallocates (resize,
  rehash), which the table code already knows, so an ordinary store does no
  accounting. Constant-key instructions can cache the node slot they hit
  last (in a per-proto side array), as upstream does in the instruction.

### 4.5 Library code written against one-value-at-a-time VM operations

- **Cost (measured).** `table.move`/`insert(t,i,v)`/`remove(t,i)`: 4 of
  the worst 10 (18x to 210x), 18% of the log-gap.
- **Structural choice.** The table library reaches tables only through
  per-element VM operations (`get_int`, `raw_set`, each with barrier and
  accounting), and `move_elements` picks its direction through a boxed
  iterator.
- **Simpler design.** Tables expose range operations on their array part
  (`move_range`, `insert_shift`, `remove_shift`) with one barrier and one
  accounting step per call; the library functions become a few lines over
  those, and fall back to the per-element path only when a range leaves the
  array part. C++ does the same with `memmove`.

### 4.6 Strings stored twice and hashed weakly; numbers formatted twice

- **Cost (measured).** 18 string benches at 5.5x, 24% of the log-gap; 6 of
  the worst 10 (15x to 26x). Interner probing is 42% of `StringInterp` and
  30% of `ToNumberString` samples; number formatting 22 to 35%.
- **Structural choice.** Every string's bytes live in an `Rc<[u8]>` shared
  between the string arena and the interner's `HashMap` key, and the map
  rehashes from bytes with an unfinalized FxHasher (so keys that differ only
  in their last bytes share buckets: 17.6 s vs 19 ms for 200k `"j=N"` keys,
  §3.4). Numbers become strings through `core::fmt` into a `String`, then a
  re-layout into Luau's format, then the intern.
- **Simpler design.** Store each string's bytes once, in the arena, with its
  hash computed once at creation; the interner is a hash table of string
  ids keyed by that stored hash (comparing bytes through the arena on a
  hash match), so there is no second owner, no `Rc` count, and rehashing
  never rereads bytes. Number to string is one direct shortest-round-trip
  formatter writing Luau's layout into a stack buffer, then one intern.
  The weak hash is also a one-line fix today (§5.2), but the stored-hash
  design removes the class of problem.

### 4.7 Smaller ones

- **Coroutines** (`sieve` 31%, `Pcall_pcall_yield` 17%): the running
  thread's state is swapped field by field into `Vm` and back on every
  switch (11 fields, twice). Simpler: each thread's execution state stays in
  the arena and the VM addresses the current one by index; a switch changes
  the index.
- **Closures** (`UpvalueCapture`, GC up to 43%): each closure allocates a
  separate `Vec` for upvalues besides its arena slot and clones its proto's
  `Rc`. Simpler: upvalues sized once with the closure (`Box<[Value]>`), proto
  by index. Whether the collector itself does more work per allocated byte
  than luaur's was not measured.
- **Varargs and result moves** (`VariadicSelect` GETVARARGS 26%): slot by
  slot through checked `get`/`set`; with the register window (§4.3) these
  become `copy_within` and `fill` on a slice.

## 5. Proposed changes, ranked

### 5.1 Redesigns

Ranked by how much they simplify the VM and how much they save, together.
All are without `unsafe` and without PGO, and behavior-neutral (each keeps
what the differential and conformance gates pin). Savings are **inferred**
from §2 to §4 and not verified by patches.

| # | Redesign | Simplifies | Benches it moves | Rough saving (inferred) |
| ---: | --- | --- | --- | --- |
| 1 | **Pure builtins as plain functions over argument slices**, called directly by CALL and FASTCALL; frame + continuation protocol only for natives that yield or call Lua (§4.1) | Deletes the fast-body duplication; most of the library stops touching frames, `Wait` and `Request` | 21 builtin-call benches (5.2x group: `AbsSum_*`, `SqrtSum_*`, `Assert`, `VariadicSelect`, `vector_lib`, `TableMarshal_*`, `sha256`, `trig`, `base64`, `pcmmix`, `vector-math`, `mesh-normal-vector`, `chess`) | Per builtin call from +850 to well under +100 over luaur; group to about 1.5 to 2x |
| 2 | **Small `Copy` Lua frames; CALL/RETURN between Lua frames stay in the dispatch loop; side stack for native/metamethod wait state; protos by index, no `Rc`** (§4.2) | `call.rs`'s common path becomes one short function; `Frame` loses `Wait`, `restore`, `Option<Rc<Proto>>` | Every bench that calls: `Factorial`, `MethodCalls`, `OOP_*`, `Pcall_*`, `qsort`, and the floor of all others (`frame-entry` 8 to 21%) | Per Lua call+return from +750 to perhaps +150; Lua-call group 3.55x to about 1.5 to 2x |
| 3 | **Validate once, then a `&mut [Value; 256]` register window indexed by `u8`; saved pc written only where observable** (§4.3) | Removes the checked-accessor layer (`get`/`set` fallbacks, saturating index math) from the interpreter | All; most visibly `tictactoe`, `LargeTableSum_*`, `AbsSum_and_or`, `SqrtSum_exponent`, `matrixmult`, `life` | Dispatch plus register traffic from roughly 25 to about 10 instructions per bytecode; 2 to 3x floor toward 1.3 to 1.8x |
| 4 | **Raw table access in the instruction; metamethod path cold and small; budget charged at reallocation, not per store** (§4.4) | One raw path and one cold path instead of a general step on every access; `Table::set` loses its accounting prelude | `OOP_*`, `MethodCalls`, `tictactoe`, `life`, `chess`, `mesh-*`, `LargeTable*`, `TableInsertion_index_*`, `voxelgen`, `TableCreate_*` | Table arms from about 3x to 1.5x of luaur; existing-key stores lose two of three lookups |
| 5 | **Range operations on the table's array part for the table library** (§4.5) | `move_elements` and friends shrink to a few lines | `TableMove_*` (19x to 210x), `TableRemoval` (27x), `table_insert_index` (20x) | To about luaur parity on these five; bulk-ops group from 10x to about 2x |
| 6 | **Strings stored once with a stored hash; interner keyed by id; direct number formatter** (§4.6) | No `Rc<[u8]>` second owner; no `core::fmt` round trip | `StringInterp/*`, `ToNumberString/*`, `string_lib/*`, `table_concat/*`, `%d`/`%f` formats | Probe share (30 to 42%) and formatting share (22 to 35%) to a few percent each |
| 7 | **Thread state addressed by index, not swapped** (§4.7) | `switch_to` becomes an index change | `sieve`, `Pcall_pcall_yield` | Coroutine share (17 to 31%) roughly halved |
| 8 | **Closures with upvalues sized at creation, proto by index; then measure GC work per byte against luaur** (§4.7) | One allocation per closure | `UpvalueCapture`, `OOP_constructor`, `tostring-gc` | Moderate; GC share not separable in these profiles |

1, 2 and 3 are one piece of work in practice: once builtins do not need
frames and Lua calls do not leave the loop, the register window can live
across the whole dispatch loop. Measure after each (§7), since they
interact.

### 5.2 Local tuning of the current design

Worth doing only where a redesign above is far off, or as a stopgap:

- **Interner hash finalizer**: `rotate_left(26)` in `FxHasher::finish`
  (what rustc-hash 2.x does). One line; measured standalone at 900x on
  `"j=N"` keys. Superseded by #6 but cheap enough to take now.
- **`move_elements` without `Box<dyn Iterator>`**: choose `Range` or `Rev`
  with an `if`. Superseded by #5.
- **`Table::set`**: skip `capacity_after_new_key`'s extra lookup when the
  key exists. Superseded by #4.
- **`step_native`**: borrow the `NativeFn` instead of cloning it. Superseded
  by #1 for builtins.
- **Cold arms** (generic FASTCALL, closure capture loops, FORNPREP errors,
  SETLIST, GETVARARGS) into `#[cold]` functions to lower `run_frames`'s
  4.4 KB frame and register pressure. Partly superseded by #2 and #3.
- **GETVARARGS / `move_results`** with `copy_within`/`fill`. Superseded by #3.

**A rough bound (inferred).** Capping the builtin and bulk-op groups and
the >5x string benches at 2x (or 2.5x) of luaur, and leaving every other
bench as measured, gives a suite geomean of 2.43x (2.67x). Going below C++
needs the floor in every group (2 to 3x today) to fall below luaur's own
1.12x of C++, which is what #1 to #4 are about. The measurements here do
not show that this is reachable; they show where the instructions go and
which design choices put them there.

## 6. What was not measured

- **No C++ Luau run.** The C++ column is derived (§1.3) from an older wall
  time ratio and assumes equal IPC.
- **No wall time.** Only retired instructions. Instruction ratios can
  differ from time ratios (cache misses, branch mispredicts, the
  jump-table dispatch); a quiet-host wall run is still owed.
- **No patch was tried.** Every expected size in §5 is inferred. The interner
  hash finding is measured on a standalone `HashMap`, not on the VM.
- **No luaur-side profiles.** The comparison is per bench and per operation
  (§2.2), not per function on both sides.
- **Profiles are per script and include untimed work** (setup, the
  `collectgarbage()` between runs), so GC shares are upper bounds and the
  multi-bench scripts are not split per bench. 19 scripts were profiled,
  not all 72.
- **Product workloads were not measured.** The 2026-11-10 workload
  characterization put bytecode at 21% of script cost in Nuxie's own
  content, with the host boundary at 41% and host bodies at 37%; this map is
  about the bytecode and library part only. The embedding API's boundary
  cost (lane 6) is not covered.
- **Memory and allocation counts** were not measured directly; `syslib`
  leaves are unsymbolized and only suggest allocator time.

## 7. The instruction-count table and `make luau-vm-perf-report`

An instrument, not a gate. Lanes from lane 8 on can see what a change did to
retired instructions per bench, with the noise beside it. Nothing fails on a
regression, and there is no tolerance to aim at: the point is to judge
redesigns (§5.1), not to hill-climb toward a number.

### 7.1 The committed table

`tests/luau-bench/instructions-nuxie.json`: a `luau-bench run` result file
for `nuxie-luau-vm` on all 87 benches. Values are millions of retired
instructions per timed run (each pass: the median of the script's 20 kept
runs). The report uses each bench's minimum over the 3 passes.

| Field | Value |
| --- | --- |
| Commit | `602c73b6cd` (VM code identical to `origin/main` `94b0d21fcb`), clean tree (`dirty_files` 0) |
| Build | `tools/bazel/runtime.py build --release -p luau-bench` (the repo's normal path): `.bazelrc` `release` = fat LTO, `codegen_units = 1`; workspace `panic = "unwind"` |
| Toolchain | rustc 1.94.1 through rules_rust 0.74.0 (`MODULE.bazel`) |
| Host | Apple M5 Max, 18 CPUs, macOS 26.6.2 |
| Clock | `LUAU_BENCH_CLOCK=instructions` (§1.2), user + kernel retired instructions |
| Passes | 3, at 1-minute load 9.7 to 20.1 (other lanes' builds) |
| Taken | 2026-11-11T13:10:55Z |

This table is from the Bazel binary, so it differs from §2's cargo-built
numbers (rustc 1.97.1): over the 87 benches Bazel/cargo is geomean 1.001,
single benches 0.972x (`AbsSum_and_or`) to 1.035x (`Failure_xpcall_a_bar`).
Compare only tables from the same build path and toolchain; the report says
so when they differ.

### 7.2 Run-to-run spread

Spread per bench is max/min − 1 of its per-pass medians in one file.

| Run | Median | p75 | p90 | Max |
| --- | ---: | ---: | ---: | ---: |
| Committed table (Bazel, load 10 to 20) | 0.38% | 1.45% | 2.65% | 4.24% (`StringInterp` `%f format`) |
| §2 nuxie run (cargo, load 13 to 26) | 0.12% | - | 0.67% | 4.19% (`Failure_pcall_a_bar`) |
| Second Bazel run (§7.4, load 10 to 18) | 0.03% | 0.31% | 1.04% | 2.22% |

- **Noise only adds.** In the committed table pass 1 (the slowest, 486 s)
  holds the maximum for 50 of 87 benches; the minimum is spread over passes
  2 and 3. The counter includes kernel instructions (page faults, zeroing
  new pages, interrupts), which grow with host load. That is why the report
  takes the minimum.
- **Pure bytecode is near-deterministic**: `Factorial` 0.13%, `sieve` 0.01%.
- **Allocation-heavy benches are wider**, for the same kernel reason, plus GC
  timing that depends on what the allocator returns: `StringInterp` `%f`,
  `%*`, `%d` and `tostring` (3.1 to 4.2%), `chess` (4.2%),
  `TableMove_same_table` (4.0%), `Pcall_pcall_return` (3.3%), `base64`
  (2.8%), `life` (2.7%), `TableRemoval_table_remove` (2.6%). These are not
  excluded: the report prints each bench's own spread next to its change,
  so they read as noisier, not as exempt.
- **A second floor: codegen.** Two builds of the same source differ by up to
  ±3.5% on single benches (§7.1), and earlier runtime A/B work here saw 2
  to 5% inlining and layout shifts from unrelated changes. A
  single-bench change under about 3% is unproven even when it is beyond the
  run spread; the geomean and groups of benches moving together are the
  signal.

### 7.3 Using it

```bash
make luau-vm-perf-report                        # 3 passes, table vs the committed file
make luau-vm-perf-report LUAU_BENCH_FILTER=Factorial LUAU_VM_PERF_RUNS=1   # quick look at one script
```

It builds `luau-bench` through Bazel, merges the scripts, runs nuxie with the
instruction clock, and prints one row per bench sorted by change: before,
after, change, the larger of the two spreads, and `+`/`-`/`~` (`~` when the
change is inside that spread). A failing bench shows as `missing/failed`.
The exit status is 0 whatever the numbers; only a usage or file error exits
2. It is macOS only (the counter is `proc_pid_rusage`); elsewhere the clock
reads 0 and the report says "zero count". It is not part of
`make luau-vm-ratchets` or CI.

**Runtime**: 1166 s (19.4 min) for the committed 3-pass table at load 10 to
20; 920 s (15.3 min) for the second run at load 10 to 18. Wall time scales with the VM's speed:
the same 3 passes on luaur take about 3.3 min.

### 7.4 Replacing the table

When a change lands that moves the counts (or the toolchain or build
settings change), refresh the table on the commit whose code it measures,
from a clean tree, and commit the table with or right after that change:

```bash
make luau-vm-perf-report LUAU_VM_PERF_UPDATE=1
git add tests/luau-bench/instructions-nuxie.json
```

`LUAU_VM_PERF_UPDATE=1` prints the report first, then copies the run over
the committed file. The file records commit, dirty-file count, build,
toolchain, date, host and each pass's load, so a reader can see what it
measured. Run it on the quietest host available; the minimum of 3 passes
absorbs ordinary load, not a host that is busy for the whole run.

**Second run against the committed table** (same commit, a separate
process 20 minutes later, load 9.8 to 18.4): geomean after/before
**0.9997**; per-bench |change| median 0.014%, p90 0.08%, max 0.17%
(−0.17% to +0.11%). 85 of 87 marked `~`; the 2 marked `-` moved 0.15% and
0.17%, just past their own small spreads. So the minimum of 3 passes
reproduces across processes far more tightly than single passes vary, and
on one build of one commit a change of more than about 0.5% on a bench is
real. Across commits the codegen floor (about 3%, §7.2) dominates.

## 8. Reproducing

```bash
CARGO_TARGET_DIR=target-perf CARGO_PROFILE_RELEASE_DEBUG=line-tables-only \
  CARGO_PROFILE_RELEASE_SPLIT_DEBUGINFO=packed \
  cargo build --release -p luau-bench --bin luau-bench
python3 tests/luau-bench/merge.py --out "$SCRATCH/scripts"
LUAU_BENCH_CLOCK=instructions target-perf/release/luau-bench run --scripts "$SCRATCH/scripts" \
  --vm nuxie --runs 3 --json nuxie.json      # and --vm luaur --json luaur.json
python3 tools/luau-bench/perf/compare.py nuxie.json luaur.json \
  --bakeoff docs/evidence/luau-safe-vm-bakeoff-2026-10-31.md --markdown
python3 tools/luau-bench/perf/groups.py nuxie.json luaur.json
tools/luau-bench/perf/prof.sh "$PWD/target-perf/release/luau-bench" "$SCRATCH/scripts" test_Factorial factorial
rustc -O -o fx tools/luau-bench/perf/fx_intern_probe.rs && ./fx
```

Delete `target-perf/` afterwards. The count table and its report: §7.3.
