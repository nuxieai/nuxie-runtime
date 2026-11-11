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
- **Five causes carry most of the gap** (§3): builtin calls take the full
  native call path (no FASTCALL bodies); the string interner's hash clusters
  badly and number formatting is slow; `table.move`/`insert`/`remove` copy
  element by element through the accounted `raw_set`; every Lua call and
  return pays a 128-byte frame, an `Rc` clone and three out-of-line calls; and
  table access is out of line, returns an 88-byte result and does budget
  accounting on every store.
- **Each cause has a fix that needs no `unsafe` and no PGO** (§5). The top
  three are small and local and should remove most of the >10x outliers.
  The next three are the interpreter-core work that sets the floor for
  everything else.

Measured vs inferred: §1 to §4 are measured (counts, sampled profiles,
disassembly), except where a line says *inferred*. §5's expected sizes are
inferred from the measurements and are not verified by patches. §6 lists what
was not measured.

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

## 4. The five biggest causes

Ranked by the share of the suite's log-gap they touch (§3.1) and the size
of the per-operation excess (§2.2):

1. **Builtin calls take the full native call path.** No FASTCALL fast
   bodies outside `libs/fast.rs`; each `math.abs`/`sqrt`/`assert`/`bit32.*`/
   `select`/`vector.*` call pushes a 128-byte frame, runs `step_native`
   (`Wait` take/drop, `NativeFn` clone, request check), `move_results`,
   `safepoint`, `deliver`, and re-enters the frame loop. +830 to +860
   instructions per call; 21 benches at 5.2x, 27% of the log-gap.
2. **String interning hash clustering, plus slow number formatting.**
   FxHasher without a finalizer (measured: up to 900x slower lookups on
   suffix-varying keys) and `core::fmt`-based `num2str`. 18 benches at 5.5x,
   including 6 of the worst 10 (15x to 26x); 24% of the log-gap.
3. **Table-library bulk operations go element by element through
   `raw_set`.** `table.move`/`insert(t,i,v)`/`remove(t,i)`: per element a
   `dyn Iterator` step, a key conversion, a barrier and `Table::set`'s
   accounting. 4 of the worst 10 (18x to 210x); 18% of the log-gap.
4. **Call and return machinery.** 128-byte `Frame` with `Wait`/`restore`,
   an `Rc<Proto>` increment and decrement on every frame (re)entry, three
   out-of-line calls per call (`push_call`, `step_native` or the frame
   loop, `deliver`), results through memory. +750 instructions per Lua
   call and return; shared by every bench that calls anything.
5. **Table access out of line, with per-store accounting, on top of
   unoptimized register access.** `get/set_table_step` calls returning ~88
   bytes, no slot cache, three lookups per existing-key store, and every
   register access doing a saturating add, two reloads from `Vm` and a
   bounds check, with `pc`/`code` spilled. Sets the 2 to 3x floor seen even
   in the cheapest groups (array 2.95x, fields 3.57x, arithmetic 2.25x).

## 5. Proposed changes, ranked

All without `unsafe` and without PGO, idiomatic Rust, and behavior-neutral
(each keeps the semantics the differential and conformance gates pin).
Expected sizes are **inferred** from §2 to §4 and not verified by patches;
the measured basis is named for each.

| # | Change | Benches it should move | Rough expected size (inferred) | Basis (measured) |
| ---: | --- | --- | --- | --- |
| 1 | **Finish the interner hash**: `rotate_left(26)` in `FxHasher::finish` (rustc-hash 2.x), or a real finalizer such as one `mix` round. One line, no semantic change (ids are unaffected). | `StringInterp` `%*`, `%d`, `interp number`, `tostring`, `concat`; `ToNumberString` `tostring`, `tostring-gc`, `tonumber`; anything building many similar strings | Those benches from 15x to 26x down by roughly the probe share, 30 to 45% of their instructions; the 900x standalone gap says the probe cost itself goes to near zero | §3.4 standalone probe; 42% / 30% probe share in profiles |
| 2 | **Array-part fast path in `move_elements`**: when the table has no `__newindex`/`__index` concerns (raw ops already) and source and destination ranges are inside the array part, `array.copy_within` (and `fill(Nil)` where needed) with one barrier and no per-element accounting; otherwise a plain loop with a `Range`/`Rev` chosen by `if`, not `Box<dyn Iterator>`. Same for `table.remove`/`insert` at an index. | `TableMove_same_table` (210x), `TableMove_table_create` (34x), `TableMove_empty_table` (19x), `TableRemoval_table_remove` (27x), `TableInsertion_table_insert_index` (20x) | To about luaur parity on these five (C++ memmoves the same case); geomean of the bulk-ops group from 10x to about 2x | §3.4 `move_elements`; 72% `tset` in the profiles |
| 3 | **FASTCALL fast bodies for the common builtins** under `safeenv`: `math.abs/sqrt/floor/ceil/min/max/sin/cos/...`, `assert`, `type`, `select("#")`/`select(n)`, `rawget/rawequal/rawlen`, `bit32.*`, `vector.*`, `string.byte/len/sub`, `buffer` reads. Each writes the result to the CALL's registers and skips the CALL, as upstream does; the fallthrough stays for anything unusual. Keep them in a separate module as plain functions over `Value`. | The 21 builtin-call benches (5.2x group): `AbsSum_*`, `SqrtSum_*`, `Assert`, `VariadicSelect`, `vector_lib`, `TableMarshal_*`, `sha256`, `trig`, `base64`, `pcmmix`, `vector-math`, `mesh-normal-vector`, `chess` (bit32) | Per builtin call from about +850 to well under +100 instructions over luaur; the 7x to 8x benches to roughly 1.5 to 2.5x; group geomean 5.2x to about 2x | §2.2 per-call excess; CALL 60 to 71% in those profiles |
| 4 | **Slim the call path**: (a) split `Frame` into a hot 32 to 48-byte part (base, top, savedpc, nresults, closure, proto index, flags) and a side table for the rare `Wait`/`restore` payloads, so push/pop move little and `deliver` drops nothing on the common path; (b) hold the running proto by index or `&Rc` borrowed for the loop, not a fresh `Rc::clone` per frame entry; (c) an inline Lua-to-Lua fast path in CALL (callee is a Lua function, no `__call`, fixed args) and in RETURN to a Lua caller with `Wait::Idle`, which updates the loop's locals instead of `continue 'frame`; (d) natives: borrow the `NativeFn` instead of cloning it. | Every bench that calls: `Factorial`, `MethodCalls`, `OOP_method_call`, `Pcall_*`, `qsort`, plus the builtin group for whatever #3 does not cover | Per Lua call+return from +750 to perhaps +150 over luaur; Lua-call group 3.55x to about 1.5 to 2x; `frame-entry` (8 to 21%) mostly gone | §3.4 frame entry and calls; RETURN 20 to 28%, CALL 15 to 24%, frame-entry 12 to 14% |
| 5 | **Inline table fast paths**: in GETTABLE/GETTABLEKS/GETTABLEN/NAMECALL, when the object is a table and the raw get hits (or there is no metatable), finish in the arm; call `get_table_step` only on a miss with a metatable. Mirror for SETTABLE*: existing key or in-range array slot with no accounting change takes a direct store plus barrier. Make `get/set_table_step` return a small `Meta` (box or index the rare `MetaCall`) so the slow path stops copying 88 bytes. | `OOP_*`, `MethodCalls`, `tictactoe`, `life`, `chess`, `mesh-*`, `LargeTable*`, `TableInsertion_index_*`, `voxelgen` | Table-access arms from about 3x to 1.5x of luaur; table groups (3.0x and 3.6x) to about 2x before #6/#7 | §3.4 table access; tget 14 to 27%, tset 10 to 36% |
| 6 | **Cheaper `Table::set` accounting**: compute growth only when a key is actually new (the store's own lookup knows), not `capacity_bytes` twice plus `capacity_after_new_key`'s extra lookup on every store. | Every store: `LargeTableSum` setup, `TableCreate_*`, `trig`, `OOP_constructor`, `TableMove` | Existing-key and array stores lose two of three lookups; tset bucket roughly halves | §3.4 `Table::set` |
| 7 | **Register window and dispatch registers**: verify at frame entry that `base + 256 <= stack.len()` (grow the stack so it holds) and take a local `&mut [Value; 256]` view indexed by `u8` registers: no saturating add, no reload, no bounds check, all safe. Arms that need `&mut self` re-borrow. Move cold arms (generic FASTCALL, NEWCLOSURE/DUPCLOSURE capture loops, FORNPREP errors, SETLIST, GETVARARGS) into `#[cold]` functions so `pc`/`code` stay in registers; store `self.pc` only before calls and errors (pass `pc` to the error builder). | All, most visibly `tictactoe` (dispatch 24%, `saturating_add` 16%), `LargeTableSum_*`, `AbsSum_and_or`, `SqrtSum_exponent`, `matrixmult` | Dispatch plus register traffic per instruction from roughly 25 to 10 instructions; the 2 to 3x floor groups toward 1.3 to 1.8x | §3.4 dispatch and register access |
| 8 | **Direct number formatting**: a shortest-round-trip formatter (Schubfach, as lane 1 suggested) writing Luau's layout into a stack buffer, then one intern; no `core::fmt`, no `String`. | `StringInterp` (number cases), `ToNumberString/tostring*`, `%d`/`%f` formats | The 22 to 35% `numfmt` share to a few percent | §3.2 numfmt bucket |
| 9 | **Coroutine switch without swaps**: keep each thread's `Exec` in the arena and have `Vm` work through the current thread's index (or swap one boxed `Exec`, not 11 fields twice); avoid `Request`/`Wait` boxing on the resume/yield fast path. | `sieve`, `Pcall_pcall_yield` | Coroutine bucket (17 to 31%) roughly halved | §3.4 coroutines |
| 10 | **Closure allocation and GC pacing check**: store upvalues inline with the closure (`Box<[Value]>` sized once, or a small-vector) and skip the `Rc::clone` of the proto; then compare GC work per allocated byte against luaur before tuning the collector. | `UpvalueCapture`, `OOP_constructor`, `TableCreate_zerofill`, `tostring-gc` | Inferred moderate; GC share not separable from untimed collections in these profiles | §3.2 gc 15 to 43% (upper bound) |
| 11 | **GETVARARGS / `move_results` with slice ops**: `copy_within` + `fill` on the stack slice instead of per-slot `get`/`set`. | `VariadicSelect`, `TableMarshal_varargs`, every multi-result return | GETVARARGS 26% in `VariadicSelect` to a few percent | §3.3 |

**Order.** 1 to 3 are small, local, and take the outliers out (they cover
the whole worst 10 and the 7x to 8x builtin benches). 4, 5 and 7 are the
interpreter-core work that sets the floor for every bench; they interact
(fewer spills once the frame loop is not re-entered), so measure after each.
6, 8 to 11 are follow-ups.

**A rough ceiling (inferred).** Capping the builtin and bulk-op groups and
the >5x string benches at 2x (or 2.5x) of luaur, and leaving every other
bench as measured, gives a suite geomean of 2.43x (2.67x): 1 to 3 alone
roughly halve the gap. Getting
under 1x of C++ (Levi's target) needs 4, 5 and 7 to take the floor from 2
to 3x to below luaur's 1.12x of C++, which is a per-instruction budget of
about luaur's own: dispatch in registers, no per-access checks, inline
fast paths for calls and tables. The measurements here do not show that it
is reachable; they show where the instructions go.

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

## 7. Reproducing

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

Delete `target-perf/` afterwards.
