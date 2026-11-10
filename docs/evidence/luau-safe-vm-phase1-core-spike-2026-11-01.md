# Safe Luau VM, Phase 1: the core spike (2026-11-01)

Phase 1 of `docs/luau-safe-vm-plan.md`: a throwaway core VM in safe Rust
that settles D1 (value, stack and call-frame representation) and D2 (heap
and handle model), and confirms D3 (error and limits model). The spike is
`spikes/luau-core-spike/` on branch `claude/luau-phase1-core-spike`. It is
its own cargo workspace, outside the repository workspace, has no Bazel
target, and is marked throwaway. Its README fixed the kill criteria before
the first bench run.

**Outcome.**

- **D2 passes.** Typed arenas per object kind, with collection only at
  `&mut Vm` safepoints and a host API that has no unrooted handle type,
  run the reentrancy sequence with no `unsafe` and no liveness check
  anywhere.
- **D3 is confirmed.** Every guest-reachable failure tried is an `Err`,
  and the limits are fields of one struct. Not covered: a memory budget and
  an out-of-memory path.
- **D1's soundness half passes,** but **its performance kill criterion
  fires.** On the five benches the spike is **1.41x the vendored VM**
  (geomean of interleaved minimums; 1.61x by median), measured in the same
  sitting. Projected onto C++ that is **1.65x to 2.1x**. Both limits are
  1.3x.
- Two costed fixes were tried and neither moved the number, so there is
  no identified, costed path down yet. Section 6 names the next cheapest
  experiment.
- The host was heavily loaded throughout, with the 1-minute load average
  between 46 and 800 on 18 cores. Every number below is an interleaved A/B
  with the load recorded, not a quiet-host measurement.

## 1. What was built

About 4,600 lines of `#![forbid(unsafe_code)]` library Rust, plus about
400 lines of bench driver and tests. It loads bytecode from the vendored
compiler (`luaur-compiler` 0.1.8, default options:
optimization 1, debug 1), the same bytes the vendored VM runs, and
executes the five benches end to end. It covers:

- the hot opcode set, plus everything the five merged scripts and
  `bench_support` reach: namecall, upvalues, varargs, generic and numeric
  for loops, FASTCALL variants, closures, SETLIST and table templates;
- tables, with the D4 array-part contract;
- interned strings;
- coroutines;
- a minimal base library: print, assert, error, pcall, type, tostring,
  tonumber, setmetatable, getmetatable, next, pairs, ipairs, rawget,
  select, collectgarbage and os.clock, plus parts of string, table, bit32,
  math and coroutine.

### 1.1 Values (D1)

`Value` is a 16-byte `Copy` enum (a const assert pins the size):

```
Nil | Bool(bool) | Num(f64) | Str(StrId) | Table(TableId) | LFunc(ClosureId)
    | NFunc(NativeId) | Thread(ThreadId) | Light(u64) | UpRef(UpvalId)
```

Every heap reference is a typed `u32` arena index. There is no NaN-boxing
and no pointer. `Value` is crate-private, so host code cannot name it (a
`compile_fail` doctest checks this). Equality is manual: numbers compare as
f64, and references compare by index.

### 1.2 Stack and frames (D1)

- Each thread has one `Vec<Value>` stack and a `Vec<Frame>` call stack.
  `Frame` is 32 bytes: func, base, pc, closure, proto, nresults, nvarargs
  and a boundary flag.
- **The register window is a `&mut [Value; 256]`.** It is derived once on
  frame entry with `try_from` on `stack[base..base + 256]`. The stack
  always holds at least `base + 256` slots.
- Bytecode register operands are `u8`, and `ra + 1` style offsets are
  masked with `& 0xff`, so every register access is an index below 256
  into a 256-element array. The compiler proves these in range and emits
  no bounds check. The register file is the hottest memory in the
  interpreter, and it costs nothing for safety.
- Code and constants are borrowed from an `Rc<Proto>` that is cloned once
  per `execute` entry, not per instruction. They are read with `get`, which
  is one compare against a length per fetch.
- **VM_PROTECT is enforced by the borrow checker.** The inner loop holds
  `&mut` borrows of the register window and the heap. Anything that can
  reallocate the stack, run a collection or re-enter the VM (calls,
  metamethods, closures, varargs, errors, GC steps) cannot be written
  inside the loop, because it would need a second `&mut self`. The loop
  `break`s with a `Slow` action, the borrows end, the action runs with
  `&mut self`, and the loop re-derives the window. C++ relies on reviewers
  to keep `VM_PROTECT` around such code; here a mistake does not compile.

### 1.3 Heap and handles (D2: option (c) with (a)'s invariant)

- **Typed arenas per object kind**, for tables, Lua closures, native
  closures, upvalues, strings and threads. Each arena is a `Vec<T>` plus a
  live flag and a free list. An id is a `u32` index.
- **Arenas never shrink, and a freed slot is reset to an empty object of
  the same kind.** So every id the VM ever handed out is in bounds, and a
  stale id reads an empty table or closure instead of faulting. Memory
  safety does not depend on liveness at all; only semantic correctness
  does.
- **Collection is stop-the-world mark and sweep, only at safepoints.**
  `Vm::safepoint(&mut self)` and `collect_now(&mut self)` take the whole VM
  mutably. Allocation only adds debt. The interpreter reaches a safepoint
  by breaking out of the loop (`Slow::Gc` after NEWTABLE, DUPTABLE, CONCAT,
  closure creation and fastcalls that allocate). Natives reach one after
  they return, with their results on the stack.
- **Liveness is a type-level invariant, not a runtime check.** Since
  collection needs `&mut Vm`, no Rust reference into the heap or the stack
  can be alive across one. A Rust local that holds a `Value` across a
  safepoint is the only way to keep a dangling id, and the rule that
  prevents it is structural:
  - Inside the crate, every value is written to a stack slot or another
    root before any call that can reach a safepoint. The places that hold
    an id across one are listed and rooted explicitly: the interned
    metamethod names, the base library's `next`/`inext` natives, the
    string library table, and the error value while Rust unwinds.
  - Outside the crate there is no unrooted handle type at all. The host API
    (`HostCtx`, `Slot`) is the Lua C API's stack discipline in types. A
    `Slot` is a position in the host function's own frame, which the
    collector scans, so holding one across `HostCtx::call` is sound by
    construction. `HostCtx<'a>` borrows the VM for exactly the host call,
    and a `compile_fail` doctest shows it cannot escape.
- The bounds checks on arena access (`items.get(id)`) are not liveness
  checks. An arena never shrinks, so the check can never catch a dangling
  id; it only exists because safe Rust indexes slices with a check. It
  is one compare per table access and is part of the D1 cost below.

Phase 2 can strengthen the crate-internal half from a rule into a type.
One way is a branded `Epoch` token, gc-arena style: a `Value` read out of
the heap carries the epoch's lifetime, and `safepoint` consumes the epoch,
so holding a read value across a collection stops compiling. The spike
shows the run-time model is sound without it; the brand turns a code
review rule into a compile error.

### 1.4 Tables (D4)

- The array part is ported exactly as D4 requires: `computesizes`,
  `adjustasize` with `aextra`, the `sizearray+1` rehash trigger, SETLIST
  through `resizearray`, and `luaH_getn`'s last-slot fast path and
  Khuong-Morin search. There is no `aboundary` cache.
- The hash part started as open addressing at twice C++'s node count. That
  made `pairs` scan twice as many slots, which cost 2.5x on TableIteration.
  It is now C++'s chained scatter table with Brent's variation (`newkey`,
  `getfreepos`, `lastfree`), using signed `i32` node offsets instead of
  pointers. Iteration therefore walks exactly `sizenode` nodes, as C++
  does.
- String hashing is a Fibonacci hash of the interned id, so iteration order
  differs from C++. D4 allows this, at the cost of the 3 hash-order sites.

### 1.5 Coroutines

A thread's state (stack, frames, open upvalues, status) is swapped between
`Vm::cur` and the thread arena on resume and yield. A resumed thread runs
in a nested `execute`, with the resume counted against the host-call depth
limit. `nny` counts Rust frames on a thread, and a yield across one is an
`Err`, as in C++. Resume and yield move up to 4 values without allocating.

### 1.6 Errors and limits (D3)

- Every guest-reachable failure is an `Err(VmError)`, carrying a message
  and an optional thrown value. The thrown value is rooted while Rust
  unwinds.
- `pcall` catches at the Rust level. A failed call pops the frames it
  pushed and restores the stack top.
- The limits are fields of one `Limits` struct: call depth 20000, host-call
  depth 200, stack slots 1,000,000, meta chain 100.

## 2. Correctness of the bench runs

Before any number was trusted:

- **sha256** checks its own digest with `assert`.
- **tictactoe** prints `Analyzed positions: 44628` 24 times, the same as
  PUC Lua 5.5.0 running the same merged script.
- **sieve** finds 403,200 primes in total (24 runs x 100 loops x 168
  primes). The count came from an instrumented copy, which matches PUC Lua
  5.5.0.
- **TableIteration** sums to 6,060,606,000,000 on an instrumented copy,
  also matching PUC Lua.
- **OOP_field_access** reads a constant field.

The vendored VM's bench tool does not expose script output, so the
cross-check is against PUC Lua, not luaur. These four scripts use no
Luau-only semantics.

## 3. Numbers

### 3.1 Method and its limits

- Same scripts and statistic as `tools/luau-bench`: the merged scripts
  from `tests/luau-bench/merge.py`. Per pass, each script's 20 kept runs
  are reduced to their median, and the host's 1-minute load average is
  recorded before and after the pass.
- Both VMs ran the same five merged scripts and compiled them with the
  same vendored compiler at the same options:
  - the vendored VM through `target/release/luau-bench__bin`, built by
    `make luau-bench` (Bazel, release, fat LTO);
  - the spike through `spike-bench` (cargo, release, fat LTO, one codegen
    unit).
- **The host was never quiet.** Other sessions' Bazel builds, an editor
  test suite and iOS simulators held the 1-minute load average between 46
  and 800 on 18 cores for the whole sitting. A single pass of one VM moved
  2x between rounds, as the process landed on P or E cores.
- So the passes were **interleaved**: one whole-suite pass of the vendored
  VM, then one of the spike, repeated. This ran 8 rounds, then 6 more
  (those 6 inside a three-way A/B with experiment E2, using the same base
  build), for **14 passes per VM**.
- Reported per bench:
  - the **minimum** pass median, the best estimate of uncontended speed.
    The vendored VM's minimum on OOP_field_access (30.04 ms in one run,
    31.77 ms pooled) reproduces its quiet-host bake-off number (30.04 ms)
    almost exactly.
  - the **median** pass median, the `tools/luau-bench` statistic, here
    over 14 passes instead of 5.
- The raw per-pass JSON files (schema 1, with load per pass) are in the
  session scratchpad, not committed.
- `/usr/bin/time -l` instruction counts were tried as a load-independent
  metric. They are not usable here: they reported about 1.9M instructions
  for multi-second runs.

### 3.2 Results

| Bench | Spike min (ms) | Vendored min (ms) | Spike / vendored, min | Spike median (ms) | Vendored median (ms) | Spike / vendored, median | C++ (bake-off, ms) | Vendored, bake-off (ms) | Vendored, `baseline-luaur.json` (ms) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| OOP_field_access | 56.59 | 31.77 | 1.78x | 78.94 | 40.11 | 1.97x | 23.84 | 30.04 | 20.32 |
| TableIteration | 6.42 | 3.97 | 1.62x | 9.10 | 4.44 | 2.05x | 2.97 | 4.12 | 2.98 |
| sha256 | 24.73 | 13.20 | 1.87x | 34.04 | 16.22 | 2.10x | 8.07 | 11.44 | 8.97 |
| sieve | 206.67 | 148.91 | 1.39x | 295.90 | 197.20 | 1.50x | 86.11 | 113.65 | 96.73 |
| tictactoe | 136.44 | 181.93 | **0.75x** | 176.23 | 208.27 | **0.85x** | 62.33 | 142.16 | 126.45 |
| **geomean** | 47.95 | 33.95 | **1.41x** | 66.24 | 41.18 | **1.61x** | 19.83 | 29.64 | 23.14 |

Against the kill criterion:

- **Spike / vendored, same sitting:** 1.41x (min) to 1.61x (median). Over
  1.3x.
- **Spike / C++:** the C++ column comes from a quiet host, so the only
  honest comparison is projected. Scaling the same-sitting ratio by the
  vendored VM's own ratio to C++ gives:
  - 2.11x, using the quiet bake-off (vendored is 1.49x C++ there);
  - 1.65x, using the committed fat-LTO baseline (vendored is 1.17x C++
    there).
- The direct ratio of the spike's minimums to C++ (2.42x) is inflated by
  the load, so it is not a usable number.

### 3.3 Where the time goes

- **tictactoe** is faster than the vendored VM (0.75x). The design is not
  uniformly slow. The vendored VM is a known outlier here: 2.28x C++ in the
  bake-off.
- **OOP_field_access** (1.8x to 2.0x) is a two-instruction loop,
  GETTABLEKS hitting a string key, then FORNLOOP. It measures per-instruction
  overhead and nothing else. Reading the generated code:
  - the dispatch is one shared indirect branch;
  - the code slice's pointer and length are spilled to the stack and
    reloaded on every fetch;
  - the opcode is range-checked before the jump table;
  - GETTABLEKS does three bounds checks: constant, table arena and node
    array;
  - the key's id is spilled and reloaded inside the probe;
  - the path is about 45 instructions, against roughly 15 for C++.
- **sha256** (1.9x to 2.1x) is the same per-instruction tax plus native
  calls. `string.byte` with 4 results and `table.unpack` with 8 leave the
  fastcall path, in C++ too. Each such call breaks out of the loop,
  re-derives the frame and re-enters.
- **TableIteration** (1.6x to 2.1x) was 2.6x before the hash part became
  chained scatter. What remains is FORGLOOP's per-iteration cost: 40-byte
  nodes against C++'s 32, a cursor kept as a `Value`, and the same
  dispatch tax.
- **sieve** (1.4x to 1.5x) is coroutine-bound: resume and yield swap
  `ThreadState` and run a nested `execute`. Moving resume values inline
  instead of in two `Vec`s per resume was done before measuring.

### 3.4 Fixes tried

Fixes found by profiling and kept, all before the measurement above:

- variadic FASTCALL with multret arguments (`bit32.bxor` in sha256 had
  fallen to the native path);
- removing a 4 KiB clear and a `Vec` from every native math call;
- string natives sharing bytes instead of copying them;
- allocation-free resume;
- the chained scatter hash part.

Two further experiments, each measured by interleaved A/B against the base
build and then reverted, because neither helped:

- **E1, cold helpers out of line.** `#[inline(never)]`/`#[cold]` on
  `fastcall`, `index_chain`, `less_than`, `less_equal` and the error
  builders, to cut register pressure in `execute`. OOP minimum 52.88 ms
  before, 54.53 ms after, over 12 rounds. No effect.
- **E2, C++'s predicted node slot for GETTABLEKS and SETTABLEKS.**
  Instructions held as `Cell<u32>` so the slot is patched into C, as
  `VM_PATCH_C` does. OOP minimum 63.44 ms before, 67.09 ms after, over 6
  rounds. The other benches moved within noise (sieve -7%, tictactoe -8%,
  TableIteration +7%, sha256 +8%, all by minimum). No effect: the hash
  probe is not where OOP's time goes. The dispatch and fetch overhead
  around it is.

## 4. The D2 reentrancy test

`spikes/luau-core-spike/tests/reentrancy.rs`,
`newindex_host_callback_collects_and_unwinds`:

1. Lua creates a coroutine that takes a table, stores it in a local of its
   own frame, and yields. The main script then drops its own reference, so
   from this point only the suspended coroutine's stack reaches the table.
2. Lua writes `proxy.a = 1` and `proxy.b = {x = 2}`. `proxy` has a
   `__newindex` whose handler is a host Rust function.
3. The host function calls back into Lua (`churn`). `churn` allocates
   40,000 objects with a 64 KiB collection threshold, so collections run
   at safepoints while the host frame and the `__newindex` dispatch are
   live beneath it. The host then forces one more collection explicitly.
4. The host checks its own argument `Slot`s still hold what they held
   before (the table and the key), then calls Lua again (`store`) with
   them.
5. Everything returns through the host frame, the `__newindex` dispatch
   and the interpreter. The script resumes the coroutine, which returns
   `inner.n + #inner.t.list` and `inner.t.tag` from the table it held.

Asserted:

- the coroutine's table survived intact (`10`, `"held"`);
- the callback's stores arrived;
- the handler ran twice;
- more than 2 collections ran inside the callbacks;
- collections freed tables;
- the live table count afterwards is under 1,000, so the churn garbage was
  actually reclaimed.

**It passes.** It needs no `unsafe` (the crate forbids it) and no liveness
check anywhere, on or off the hot path. The interpreter loop never checks
liveness.

Companion tests in the same file, all passing:

- `error_inside_host_callback_unwinds_to_pcall`: `error({code = 42})` is
  raised three host and dispatch layers down, under a 16 KiB threshold. It
  reaches a Lua `pcall` as the same table, three times in a row, and the
  VM keeps working.
- `yield_across_host_frame_is_an_error`: yielding through a host frame
  returns "attempt to yield across a C-call boundary".
- The D3 limits each come back as an `Err`: `d3_lua_recursion_limit_is_an_err`
  ("stack overflow"), `d3_host_reentry_limit_is_an_err` (Lua to host to Lua
  recursion, "C stack overflow") and `d3_metatable_cycle_is_an_err`
  (`__index` cycle, meta chain limit).
- `d3_runtime_errors_are_errs`: indexing nil, arithmetic on a table, a NaN
  key, a nil key and `error()`.
- `d3_errors_in_library_callbacks_reach_pcall`: errors thrown from a
  `table.sort` comparator and from a `string.gsub` callback reach `pcall`
  with their value, and the half-sorted table is still sortable afterwards.

Writing these tests found a real spike bug. NEWCLOSURE read its CAPTURE
registers before writing the new closure to its own register, so a
`local function f` that calls itself captured nil. C++ writes the closure
first (`setclvalue` before the capture loop, in NEWCLOSURE and DUPCLOSURE).
The spike now does too.

## 5. Panic-lint count (D3)

`cargo clippy --release --lib` with the plan's forced lints (`unwrap_used`,
`expect_used`, `indexing_slicing`, `panic`, `unreachable`,
`arithmetic_side_effects`, `unwrap_in_result`) reports **153 distinct
source sites**. Counting each expansion of one macro separately makes it
167. None is `unwrap`, `expect`, `panic`, `unreachable` or
`unwrap_in_result`.

| Kind | Sites | What they are |
|---|---:|---|
| `indexing_slicing`, register window | 107 | `regs[x]` with `x = operand & 0xff` into `[Value; 256]`. These cannot fail; clippy cannot see the mask. |
| `indexing_slicing`, slicing | 2 | `args[..n]` in one macro (15 expansions) after an `n <= 3` check, and `args[..nparams.min(3)]`. |
| `arithmetic_side_effects` | 44 | `ra + 1`, `b - 1` and similar on `u8`-derived operands, after guards; 3 cost sums in the heap. |

In Phase 2 the 107 window sites collapse to one, with a `Reg(u8)`
newtype and `impl Index<Reg> for Window` (safe, and still check-free after
optimization). The arithmetic sites become `wrapping_*` or move into typed
operand decoders. The ratchet is reachable; the spike did not spend time on
it.

## 6. Verdicts and the next experiment

Proposed status lines, for the coordinator to copy into the plan:

- **D1**, 2026-11-01: Phase 1 spike
  (`docs/evidence/luau-safe-vm-phase1-core-spike-2026-11-01.md`).
  - Soundness half passes. A 16-byte tagged enum, `u32` typed ids, and a
    `&mut [Value; 256]` register window indexed by `u8` operands need no
    generation check and no handle bounds check in the loop. The remaining
    in-loop checks (code fetch, constants, arena and node indexing) are
    slice bounds checks, not liveness checks.
  - **Perf kill criterion fires.** The spike is 1.41x the vendored VM by
    interleaved minimums (1.61x by median), same sitting, on a heavily
    loaded host. That projects to 1.65x to 2.1x C++, against a limit of
    1.3x. Two costed fixes (cold-path outlining, predicted node slots) did
    not move it.
  - Status: **open, failing**. Next is the dispatch-floor kernel (below).
- **D2**, 2026-11-01: **passes**. Option (c) with (a)'s invariant: typed
  arenas that never shrink, stop-the-world collection only at
  `&mut Vm` safepoints, no public unrooted handle (`HostCtx`/`Slot` is the
  C API's stack discipline in types).
  - The reentrancy sequence passes with `#![forbid(unsafe_code)]` and no
    liveness check anywhere: `__newindex`, then host Rust, then Lua, then
    collections at a 64 KiB threshold plus a forced one, then unwinding,
    with a coroutine-held table surviving.
  - Phase 2 adds an epoch brand, so the crate-internal rooting rule becomes
    a compile error.
- **D3**, 2026-11-01: **confirmed** for everything the spike can reach.
  These all return `Err`, and the VM stays usable:
  - recursion, host re-entry and meta-chain limits;
  - errors inside host callbacks, `sort` comparators and `gsub` callbacks;
  - yield across a host frame;
  - NaN and nil keys.

  The limits are fields of one `Limits` struct. The panic-lint count is
  153 distinct sites, none an `unwrap`, `expect` or `panic`. 107 of them
  are register-window indexing, which collapses to one site behind a `Reg`
  newtype. Still unverified: a memory budget with an out-of-memory path,
  and the audio engine's sendable isolated mode.

**What was learned.**

- The heap and handle model is not the problem. Collection, reentrancy and
  coroutines cost nothing on the hot path, and tictactoe, the most mixed
  workload, beats the vendored VM.
- The gap is per-instruction overhead in a safe `match` interpreter over
  raw `u32` bytecode: a shared dispatch branch, spilled slice lengths and
  bounds checks on every fetch, and Rust's layout of the enum `Value`.
- The vendored VM runs the same Rust `match` shape at 1.26x C++ on
  OOP_field_access (bake-off). So Rust itself is not the floor. The
  difference is what safe indexing and this representation add to each
  instruction.

**Next cheapest experiment (one to two days).** A standalone
dispatch-floor kernel: OOP_field_access's two-instruction loop, plus one
call/return pair, in safe Rust, measured in the same sitting against C++'s
2.4 ns per iteration (`23.84 ms / 10M`). Vary one thing at a time:

1. **Pre-decoded typed instructions.** The D7 verifier already has to walk
   every instruction. If it emits `enum Insn { GetTableKs { a: u8, b: u8,
   key: StrId }, ForNLoop { a: u8, jump: i32 }, ... }`, the match is
   exhaustive (no opcode range check), operands need no shifting, and
   constant keys are resolved at load (no constant-array bounds check).
2. **Fetch without a spilled length.** For example, a code buffer padded to
   a power of two and indexed with a mask, the same trick as the register
   window. Verified jump targets keep the masked fetch exact.
3. **Value layout.** Payloads at offset 8 for every variant, so a `Value`
   copy is two 8-byte moves, not a byte plus two unaligned words.

If the kernel reaches 1.3x C++ or better, D1 passes with that
representation, and the Phase 2 VM is built on it. If the best variant is
still above 1.3x, the plan has to choose between a slower safe VM and the
vendored VM's model. That choice belongs to the plan, not to this
document.
