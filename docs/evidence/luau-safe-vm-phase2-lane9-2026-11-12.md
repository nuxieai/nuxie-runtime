# Safe Luau VM, Phase 2 lane 9: the call path (2026-11-12)

Lane 9 of Phase 2 in `docs/luau-safe-vm-plan.md`: build
`docs/luau-safe-vm-call-path.md` in `crates/nuxie-luau-vm`, stage by stage
in its section 7 order, each stage checked against its rows in section 6.
Branch `claude/luau-vm-lane9-call-path-build-0d02f0`, rebased on
`origin/main` `56ea4ff1c0`; not pushed, no PR.

**Outcome.**

- **Built**: frameless plain natives with results in place (4.1), the
  builtin fast table with all 133 bodies (4.2), 32-byte `Copy` frames with
  a pending side stack, one `LibCont` enum, and Lua `CALL`/`RETURN` inside
  the dispatch loop (4.3), and a thread switch that is one `mem::swap` of
  an `Exec` per side (4.4). The generic-host experiment (4.6) is a scratch
  commit, measured and not landed.
- **Withdrawn**: the safepoint `trigger` (4.5) and the per-activation
  `safeenv` cache (4.2), under Levi's no-caches rule of 2026-11-12. Both
  were built and then removed; the safepoint is `main`'s shape and the
  `FASTCALL` handlers read `safeenv` from the environment table. The
  direct reads cost 20 instructions per fast call (the cached compare was
  5) and 8 per safepoint (16 under a memory limit; the one compare was
  about 4) (section 5).
- **Every §6 row moved toward luaur; one is inside its range** (section
  4): `select('#', i)` at 424, the top of 324 to 424. Builtin rows fell
  2.4x to 5.3x (`math.abs(i)` 570 → 127, luaur 68); Lua-call rows 1.8x to
  2.3x (`f()` 779 → 335, luaur 146); `pcall` 1.35x; `co()` 1.5x. Each
  excess is attributed to the per-bytecode floor (dispatch, checked
  access, spills) or to the general native protocol that `pcall` and
  coroutines still use; no row was out for a step the design omits.
- **Suite: 22% fewer instructions** (geomean 0.777 over 89 benches; 4.32x
  to 3.39x luaur over the 87 that `groups.py` assigns). Builtin calls
  4.30x to 2.28x luaur, Lua calls 3.89x to 2.76x, coroutines 4.31x to
  3.11x, and the audio-block benches 4.6x to 2.0x (1,078 and 1,106
  instructions a frame against a predicted 810 to 1,010). No bench got
  dearer (section 7.1).
- **Gates green on the tip** (section 3), including `nuxie-scripting` 40 of
  40 under both settings. No `unsafe`; the panic lints stay at zero.

## 1. Commits

On `origin/main` `56ea4ff1c0`, oldest first. Authored by Levi; no trailer.

| Commit | Stage | Concern |
|---|---|---|
| `9038011615` | | design doc |
| `cdd3144eef` | 2 | vm-diff: level-sensitive observers of a plain builtin, and its memory error |
| `3d787c24b6` | 2 | a frameless native leaves no record until something looks (4.1; results in place, the request check removed) |
| `fee2f6316f` | 2 | the safepoint's fast path is one compare (4.5; reverted by `97527031cf`) |
| `3af2bb84da` | 2 | design §6: the `select('#')` row counts the bytecodes before the call (ruling) |
| `5b1faf15fc` | 2 | vm-diff: the vendored compiler's crash reproducer, kept as `.txt`, not run |
| `64766bddf1` | 2 | an inline stack compare and register moves for plain-native results (ruling) |
| `06032291b8` | 3 | vm-diff: every fast body's types, arities and result counts, `safeenv` on and off |
| `3ad2c89a6a` | 3 | the builtin fast table (4.2) |
| `ad526d493d` | 3 | vm-diff: Rive's `Vector` builtins with no `Vector` global |
| `2eeb1a51d1` | 3 | one fast-call path per `FASTCALL` form |
| `58cd9dde33` | 4 | vm-diff: returns into every kind of caller, call depth at the limit |
| `85094b442f` | 4 | library continuations are one enum (`LibCont`) |
| `0aa30e8f96` | 4 | a frame is 32 bytes of plain data, with a pending side stack |
| `85f0b9ae84` | 4 | Lua calls and returns stay in the dispatch loop |
| `785192b1c8` | 5 | a thread switch is one swap per side (4.4) |
| `a658618170` | rule | `FASTCALL` reads `safeenv` where it lives, no cached copy |
| `97527031cf` | rule | the safepoint evaluates its rule where the inputs live (4.5 withdrawn) |
| `630ea1a26e` | rule | design: no caches; 4.5 withdrawn; §6 ranges raised by the measured reads |
| `8e6c0db81b` | review | fast `table.unpack` holds the configured stack limit |
| `470a593050` | docs | the call-path design as built |
| `d19ede6415` | perf | `tests/luau-bench/instructions-nuxie.json` after the lane |
| `f46d032dc4` | docs | the plan records lane 9 |
| (this commit) | docs | this evidence |

Scratch, not landed: `scratch/luau-vm-lane9-generic-host` `ba330ef24b`
(stage 6, on the pre-rebase stage 5 commit).

## 2. What each stage built

**Stage 1, instrument.** `call_path_micro.py` on the base reproduced the
design's section 1 within 0 to 5 instructions per row.

**Stage 2 (4.1).** A plain native called by `CALL` runs without a frame:
the call stores its callee slot in `Core::frameless`, and every observer
outside the call protocol (error positions, argument-error names,
`debug.info`, tracebacks, `getfenv` levels, the collector's root scan)
goes through `Core::frames_observed`, which first pushes the frame the
pending native stands for. The frame list's methods are visible only to
the call protocol's modules (`crate::protocol`), so a new observer cannot
read it raw. Results are moved in place by register copies; the stack
check is two compares with `grow_stack` cold; the request-slot check is
gone (a plain native cannot request). Slots are `u32`
(`Limits::max_stack_slots` is capped below `u32::MAX`).

**Stage 3 (4.2).** `FAST: [Option<FastEntry>; 256]`, indexed by the
compiler's builtin id: 133 bodies (ids 1 to 132, 243, 245 to 255), each
beside its library function and ported from its `luau_f_*` including where
it bails. `libs/fast.rs` is deleted. One fast-call path per `FASTCALL`
form (the shared arm re-matched the opcode, 25 to 29 instructions a
call). The integer ids (94 to 132) are unreachable under the pinned
compiler flags and are covered by unit tests (`libs/integer.rs`).

**Stage 4 (4.3).** `Frame` is `Copy` with no drop glue, 32 bytes (both
asserted at compile time). The wait state and saved C-call counters live
on a pending side stack, one entry per frame flagged `PENDING`.
`LibCont` replaces the `Continuation` trait and its boxed impls.
`CALL` of a Lua function resolves the callee with one function lookup,
checks the shared prototypes by pointer, enters through an inlined
`enter_lua_fixed` (or `enter_lua` for varargs) and reloads the loop's
locals; plain natives take the stage 2 path; everything else is
`push_call`. `RETURN` pops in place when `Frames::pop_plain` finds a plain
Lua frame returning to a Lua caller with no pending entry; otherwise the
general deliver logic runs.

**Stage 5 (4.4).** The eleven per-thread fields are one `Exec` held
inline in `Core`; a switch is `mem::swap(&mut self.exec, &mut
thread.exec)`, once per side.

**Stage 6 (4.6).** Scratch only: `run_frames<D: Dispatch>` and
`call_frameless<D: Dispatch>` (section 6).

**The no-caches rule (2026-11-12).** `Core::deopt`, `deoptimize` and the
per-activation flag are gone; `Table::safeenv` is a plain field written
where `setfenv`, `getfenv`, the sandbox and the host change it, and read
by the `FASTCALL` handlers. The safepoint `trigger` is reverted. Kept:
`Core::type_names`, the twelve type-name strings interned once at VM
creation and rooted (constant data, like `tm_names`; never invalidated).

## 3. Gates

On `f46d032dc4` (code `8e6c0db81b`; the commits after it are docs and
the instruction table), sequential, one Bazel server, 1-minute load 10.7
at the start and 14.9 at the end (an 18-core laptop shared with other
sessions; nothing is timed against a ceiling). The same battery passed on
`630ea1a26e` before the review fix.

| Gate | Result |
|---|---|
| `make luau-vm-diff` | 225 runs, **0 disagreements**, bytecode versions 7 and 11; the corpus reaches every admitted opcode |
| `make luau-conformance-nuxie` | PASS: 36 required OK, 0 required failing, 4 expected-fail, 15 non-goal |
| `make luau-conformance` | PASS: 30 required OK, 0 required failing |
| `make luau-vm-ratchets` | PASS: `unsafe` in `nuxie-luau-vm` 0 (baseline 0); every panic lint 0 |
| `python3 tools/bazel/generate.py --check` | OK |
| clippy `nuxie-luau-vm`, `luau-bench` | clean |
| tests `nuxie-luau-vm` | 73 passed |
| tests `luau-vm-diff` | 55 passed (at `8e6c0db81b`, with the review test) |
| tests `nuxie-scripting` (default) | **40 / 40** |
| tests `nuxie-scripting --features nuxie-vm` | **40 / 40** |

`nuxie-scripting` ran with `RIVE_RUNTIME_DIR` at the pinned rive-runtime
checkout (`cd5351c3`).

## 4. Predictions and measurements (design section 6)

Per iteration over the empty loop, retired instructions
(`LUAU_BENCH_CLOCK=instructions`, Bazel fat-LTO release), `luau-bench`
micro scripts of 200,000 iterations, median of 2 runs. The empty loop is
the raw count. Base is `fd7da2b187`, the pre-rebase design commit (its VM
crate and bench tool equal `origin/main` `56ea4ff1c0`'s, the micro script
aside); each stage column is the stage's last pre-rebase commit (the
rebase changed no code); "tip" is `8e6c0db81b`, load 29.5 to 32.7. luaur
is the vendored VM at the base.
Ranges are the design's as amended (2026-11-12: the fast-call rows rose by
the measured `safeenv` read, 20; `select`, `table.insert` and the plain
host call by the safepoint's 4).

| Loop body | base | stage 2 | stage 3 | stage 4 | stage 5 | tip | range | luaur | verdict |
|---|---:|---:|---:|---:|---:|---:|---|---:|---|
| empty loop (raw) | 93 | 93 | 93 | 82 | 82 | 87 |  | 38 |  |
| `math.abs(i)` | 570 | 425 | 121 | 114 | 114 | 127 | 65–100 | 68 | out (+27) |
| `math.floor(i)` | 571 | 423 | 121 | 114 | 114 | 127 | 65–100 | 68 | out (+27) |
| `type(i)` | 689 | 544 | 125 | 118 | 118 | 131 | 70–110 | 70 | out (+21) |
| `math.min(i, 5)` | 706 | 561 | 156 | 157 | 157 | 169 | 75–120 | 79 | out (+49) |
| `bit32.band(i, 255)` | 720 | 576 | 158 | 159 | 159 | 171 | 75–120 | 81 | out (+51) |
| `math.sin(i)` | 624 | 478 | 182 | 175 | 175 | 188 | 120–170 | 131 | out (+18) |
| `vector.create(i, i, i)` | 804 | 660 | 186 | 175 | 175 | 187 | 90–140 | 92 | out (+47) |
| `buffer.writef32(b, 0, i)` | 728 | 605 | 313 | 292 | 292 | 308 | 130–190 | 133 | out (+118) |
| `rawget(t, 1)` | 682 | 534 | 272 | 267 | 267 | 281 | 130–210 | 127 | out (+71) |
| `table.insert(t, i)` | 1,057 | 931 | 707 | 691 | 691 | 713 | 274–474 | 176 | out (+239) |
| `select('#', i)` | 569 | 423 | 421 | 410 | 409 | 424 | 324–424 | 238 | in |
| `f()` | 779 | 694 | 699 | 332 | 332 | 335 | 200–300 | 146 | out (+35) |
| `local y = f(i)` | 866 | 782 | 789 | 410 | 410 | 415 | 250–360 | 169 | out (+55) |
| `local a, b = f(i, i)` | 1,085 | 1,005 | 1,011 | 604 | 603 | 614 | 330–460 | 227 | out (+154) |
| `o:m(i)` | 1,058 | 974 | 978 | 589 | 588 | 597 | 400–560 | 215 | out (+37) |
| `pcall(f, i)` | 2,413 | 2,253 | 2,327 | 1,783 | 1,782 | 1,792 | 800–1,200 | 743 | out (+592) |
| `co()` | 3,567 | 3,094 | 3,060 | 2,342 | 2,294 | 2,309 | 900–1,500 | 567 | out (+809) |
| `s = s + i` | 262 | 259 | 259 | 241 | 241 | 247 | unchanged | 81 | unchanged |
| `tostring(i)` | 13,984 | 13,907 | 13,193 | 13,185 | 13,166 | 13,207 | unchanged | 622 | unchanged |
| upvalue call (not in §6) | 952 | 863 | 872 | 494 | 494 | 501 |  | 199 |  |
| vararg call (not in §6) | 1,005 | 901 | 908 | 606 | 606 | 615 |  | 244 |  |

**Verdicts**, under the stop rule (a: a designed step built heavier than
the text, fixed; b: a step the design omits, stop; c: the floor, recorded):

- **Builtin rows (abs to vector): out, (c).** `math.abs(i)` at stage 3 was
  121: `FASTCALL1`'s own dispatch, decode and skip 34 (a skip-only probe
  build); the designed steps about 44 (entry lookup 6, the counter compare
  5, the `CALL` word's bounds check, load and test 14, pointer call 5,
  `ra` 5, `top` and `pc` 9); checked and spilled register traffic about
  23 (the argument `Value` copied through stack spills, the
  `[Value; 3]` built in memory, the loop's spilled locals reloaded); the
  body 19. One (a) was fixed on the way (`2eeb1a51d1`). The direct
  `safeenv` read then replaced the 5-instruction compare with 20.
- **`buffer.writef32`, `rawget`, `table.insert`: out, (c), plus a floor the
  ranges left out.** All three benches keep `b`/`t` as upvalues, so each
  iteration starts with `GETUPVAL` (72 here, 23 on luaur). `table.insert`'s
  remaining cost is the store: `t[i] = i` alone is 309 over `GETUPVAL`
  (luaur 45), which the design keeps ("the store's accounting stays").
- **`select('#', i)`: in, at the edge (424 against 324 to 424; 425 in an
  earlier run of the same code).** Stage 2
  stopped here at 483 against the first range; the range was amended
  (the three bytecodes before the call, 198, were left out), and the
  plain-native path got an inline stack compare and register moves
  (`64766bddf1`). The remaining call protocol is about 160 against "about
  90": the `CALL`'s own dispatch and decode (about 30, not separated from
  the floor in the estimate), callee resolution (tag, function-arena
  lookup, kind match: about 15, not a listed step), and the one-value move
  through spills (about 10 over a register copy).
- **`f()`: out (335 against 200 to 300), (c) after two (a) fixes.** Traced:
  `GETUPVAL` 47 + `CALL` 141 + `RETURN` 84 + `FORNLOOP` 62 + four
  dispatches of 20. Two (a) fixes: `enter_lua_fixed` was out of line (−28)
  and `check_call_depth` was out of line with saturating sums (−29).
  `CALL` 141: interrupt 3, decode 13, callee tag 9, function lookup 9, kind
  6, protos `ptr_eq` 4, proto lookup 10, depth 5, stack need and compare
  14, nil fill 11, caller `savedpc` 10, `u32` clamps 9, push 17, locals
  reload from spill slots 16. `RETURN` 84: decode 11, `ptr_eq` 5,
  `pop_plain` 14, pop 12, result moves 20, reload 18. Each step is in the
  design's list; each costs more than "a few instructions" because of
  checked arena access and `run_frames`' spilled locals.
- **`f(i)`, `f(i, i)`, `o:m(i)`: out, (c).** `f(i, i)` adds four `MOVE`s
  at 51 each (luaur's `MOVE` is about 15), `CALL` 160 and `RETURN` 132 (two
  results through checked `get`/`set`).
- **`pcall(f, i)`: out (1,792 against 800 to 1,200), (c), flagged.** As
  built it is what the row says: two frames, one pending entry, no
  allocation, no `Wait` drop. It runs the general native-request protocol,
  which the design keeps ("the existing deliver logic, cold"): `CALL` 750
  + `RETURN` 825, of which `push_call` ×3 250, `enter_lua` 107,
  `step_native` 220, `finish_native` 220, `deliver` 175, `move_results`
  155, `Frames::pop` 48, the `pcall` body 59, loop glue about 300. The
  range priced that protocol as lean; it is out-of-line calls with large
  prologues and `Result`s through memory.
- **`co()`: out (2,309 against 900 to 1,500), (c), flagged.** `swap_exec`
  4 × 83 = 332 (the 136-byte `Exec` copied as 17 words, plus the thread
  arena lookup): the designed size. The rest is the same general protocol
  in both directions: `finish_native` 529, loop glue 329, `step_native`
  235, `push_call` 234, `deliver` 111, `complete_yield` 101,
  `move_results` 76, `push_from_parked` 32, library bodies 134.
- **Audio block: out (1,078 and 1,106 a frame against 810 to 1,010),
  (c).** Three fast calls a frame, each at its row's measured cost (about
  127 against the 65 to 100 priced), plus the per-bytecode floor the
  derivation took as about 600. 2.0x luaur (530 a frame), down from 4.6x;
  lane 8 had predicted 2 to 2.6x for that lane.
- **Plain host call: out (405 against 134 to 224), (c), plus a floor the
  estimate left out** (section 7.2): the `GETIMPORT` and `LOADN` before
  the call, about 130 with their dispatches, as in the `select` row.

## 5. The cost of reading the source of truth

Measured on the tip's release binary (`objdump` of `run_frames`, and an
lldb single-step trace of one steady-state iteration).

**`safeenv` in the `FASTCALL1` handler: 20 instructions.** The
function-arena lookup for the running closure 7 (the arena's pointer and
length, the 48-byte stride, the bounds compare and select, with the
closure id and the fallback pointer reloaded from spill slots); the
Lua/native kind select for the environment field 4; the environment id 1;
the table-arena lookup 6; the flag byte and its test 2. The cached flag it
replaced was a 5-instruction counter compare. The micro rows moved +12 to
+16, with the empty loop and the non-call rows moving +3 to +10 from
layout in the same build.

**The safepoint's fast path: 8 instructions with no memory limit, 16 with
one.** At the plain-native call site: the bytes in use, the limit's tag
and the threshold loaded (3), the "no limit or collector stopped" test
(`cmp`, `ccmn`, `b.eq`), the threshold compare and branch (2). Under a
limit the pressure point adds 8: the limit and `live_after_cycle` loaded,
a saturating subtract, a halve-and-add with its saturation, the compare
and branch. The one-compare trigger was about 4. The benches set no
limit; `nuxie-scripting` gives every script VM one
(`ScriptExecutionLimits::max_memory_bytes`).

## 6. Stage 6: the generic host (scratch)

`ba330ef24b` makes `run_frames` and `call_frameless` generic over
`D: Dispatch` (two signatures; inner calls coerce back to `&mut dyn`).
`luau-bench` has one host type, so one instance.

| | `dyn` | generic |
|---|---:|---:|
| `luau-bench__bin` bytes | 2,236,752 | 2,236,880 |
| `__text` bytes | 1,528,988 | 1,528,832 |
| `run_frames` bytes | 42,820 | 42,620 |
| empty loop (raw) | 82 | 90 |
| `f()` / `f(i)` / `f(i, i)` | 332 / 410 / 603 | 346 / 428 / 633 (+4.2% to +5.0%) |
| `o:m(i)` / upvalue call / vararg | 588 / 494 / 606 | 594 / 518 / 628 |
| `pcall` / `co()` | 1,782 / 2,294 | 1,796 / 2,312 |
| builtins | | +3 each |
| host call, plain / full / lane 6 (per call) | 392 / 994 / 1,349 | 390 / 1,005 / 1,360 |

Generic moves the call benches by more than the 3% noise floor, in the
wrong direction, and buys nothing on host calls. Per 4.6 the choice goes
back to the coordinator; the evidence says keep `dyn` (decision 3). Each
further host type would add a `run_frames` of about 42.6 KB.

## 7. The suite and host calls

### 7.1 The suite

`make luau-vm-perf-report`, 3 passes each, Bazel fat-LTO release,
`LUAU_BENCH_CLOCK=instructions`. **Before** is `origin/main` `56ea4ff1c0`
(clean tree), 2026-11-12 00:02 UTC, 1-minute load 12.6 to 19.6.
**After** is `8e6c0db81b` (clean tree; the commits after it are docs and
the table), 00:48 UTC, load 26.0 to 12.9, run with
`LUAU_VM_PERF_UPDATE=1`, so `tests/luau-bench/instructions-nuxie.json`
is the after table. luaur is the same harness on the vendored VM at the
base (unchanged by this lane).

`tools/luau-bench/perf/report.py` before against after: **geomean 0.7765
over 89 benches; 81 fewer instructions, none more, 8 inside the noise.**
The largest moves: `AbsSum_math_abs` −62%, the two `AudioBlock` benches
−55% and −56%, `Pcall_call_return` −45%, `Factorial` −42%,
`TableInsertion_table_insert` −39%, `MethodCalls` and the `pcall`/`xpcall`
return benches −24% to −25%.

Groups (`tools/luau-bench/perf/groups.py`, nuxie/luaur geomean; the two
`AudioBlock` benches are not in its grouping and are listed below it):

| Group | Benches | Before | After |
|---|---:|---:|---:|
| builtin calls (`FASTCALL`-able) | 21 | 4.30x | **2.28x** |
| Lua calls and returns | 8 | 3.89x | **2.76x** |
| table get/set by constant key | 7 | 3.53x | 2.90x |
| array access and iteration | 14 | 3.03x | 2.81x |
| arithmetic and loops | 2 | 2.36x | 2.23x |
| string building, interning, number formatting | 18 | 5.26x | 4.91x |
| closures and upvalues | 1 | 2.88x | 2.81x |
| coroutines | 2 | 4.31x | **3.11x** |
| table library bulk ops | 10 | 10.00x | 9.86x |
| error path | 4 | 2.01x | 1.89x |
| **all 87** | 87 | **4.32x** | **3.39x** |
| `AudioBlock_sine` | | 4.58x (79.45 M) | **2.03x** (35.31 M) |
| `AudioBlock_shared_controls` | | 4.41x (80.49 M) | **1.99x** (36.24 M) |

Against design §6's derived suite predictions: builtin calls "toward 2.5x
to 3.2x" reached 2.28x (better); Lua calls "toward 2.2x to 2.8x" reached
2.76x (in); coroutines "toward 2.5x" reached 3.11x (short: the general
native protocol, section 4).

### 7.2 Host calls

The lane 8 evidence's §6.2 table, re-measured on the tip with the same
measurement-only patch to `tools/luau-bench/src/vm.rs` (registers
`__hc_plain` as `HostEntry::Plain` and `__hc_full` as `HostEntry::Full`;
the lane 6 form is `__bench_atomic_load` through `Compat`; applied,
measured and reverted, not committed). 200,000 calls of a one-argument
function returning one number (`s += f(1)`), minus the same loop with
`s += 1`, per call; median of 2 runs; 1-minute load 48.6.

| Kind | lane 8 before | lane 8 after | stage 5 (`dyn`) | tip | luaur |
|---|---:|---:|---:|---:|---:|
| lane 6 API (`Vm::register`, through `Compat`) | 936 | 1,529 | 1,349 | **1,362** | |
| core full (`HostEntry::Full`) | | 1,176 | 994 | **1,006** | |
| core plain (`HostEntry::Plain`, frameless) | | 539 | 392 | **405** | |
| luaur-rt `create_function` | | | | | 1,532 |
| the loop without a call (per iteration, raw) | 163 | 167 | 150 | 157 | 68 |

The plain call against design §6's 134 to 224: one traced iteration is
`GETIMPORT` 70 + `LOADN` 16 + `CALL` 258 + their three dispatches (66),
against the base loop's `ADD` and `FORNLOOP`. `CALL` 258 is the
`run_frames` arm 134 (the plain-native protocol of section 4's `select`
row, plus the host entry kind), `Dispatch::call_plain` 35, the bench's
host body 63 and an out-of-line `Core::to_number` for its argument 26.
The estimate priced the call alone; the bytecodes that load the function
and its argument are the floor. (c). The lane 6 API is now 1,362 against
luaur's 1,532 for the same call through `create_function`; every product
binding still goes through it until the next lane collapses the layers.

## 8. Deviations from the design

- **4.5 withdrawn and 4.2's `safeenv` cache not built** (no-caches rule,
  2026-11-12; section 5). Both were built first (`fee2f6316f`, and the
  stage 3 commit's `Core::deopt`) and removed (`97527031cf`,
  `a658618170`).
- **The frame is 32 bytes, not 28.** `nresults` stayed `i32`; with the
  flags byte the struct pads to 32 either way unless `nresults` shrinks to
  `i16`. Asserted `<= 32`.
- **Two swaps per thread switch.** The §6 row says "one swap each"; 4.4's
  text says once per side. Built per 4.4: each resume and each yield
  swaps the `Exec` out and the target's in.
- **The shared-prototype `ptr_eq`** before an inline call or return (the
  callee's and caller's protos must be the same `Arc`, lane 8's protos by
  index): 4 to 5 instructions per call and return, not in the design's
  step list.
- **The frameless native's frame is materialized on its error path**
  inside `call_frameless`, not as "the catcher's first step": same
  observable frames, and `frameless` is zero whenever control is outside
  the native.
- **The request-slot check went in stage 2** with the rest of the
  plain-native path (the design lists it under 4.3).
- **`running_closure`** reads a frameless builtin's upvalues from its
  callee slot without materializing (`gmatch`).
- **`frames_observed` returns an `Observed` view** that derefs to `Core`;
  the observer-only access is enforced by `crate::protocol` (a `#[path]`
  grouping module) rather than a separate crate.
- **The vm-diff harness charges this VM one call info** for the reference
  running chunks from a C function (`58cd9dde33`), so the deepest
  successful recursion matches (19,995 on both).
- **Plain-native result moves and the inline stack compare**
  (`64766bddf1`) were a ruling during stage 2, not in the first text.
- **The `select('#')` row** was amended during stage 2 (`3af2bb84da`).

## 9. Review

`codex review --base origin/main`, two rounds (the brief allows four;
round 2 was clean).

- **Round 1 (tip `630ea1a26e`): one finding, real, fixed in `8e6c0db81b`.**
  The fast `table.unpack` body checked the room left in the allocated
  stack but not `Limits::max_stack_slots`. A sandboxed thread's stack
  starts at 64 slots, above a lower limit, so under a limit of 50
  unpacking 47 values succeeded on the fast path where the library
  function raises "too many results to unpack". The body now falls back
  unless the call's own requirement (`top + n + 1` slots) fits the limit,
  so the two paths agree at every count. Test:
  `limit_tests::fast_unpack_holds_the_stack_limit_as_the_call_does`
  (`tools/luau-vm-diff`), counts 20 to 60 for `table.unpack(t)` and
  `table.unpack(t, 1, #t)`, fast against `getfenv()`-disabled, sandboxed;
  it fails without the fix at 47 values. No other fast body writes past
  its frame's registers (each writes at most three results, inside the
  frame the call entry already sized within the limit).
- **Round 2 (tip `8e6c0db81b`): no actionable findings.** Codex ran the
  unit tests (128), the differential corpus (225 runs) and conformance
  (36 required) itself.

## 10. Not verified

- **Product frames.** No `nuxie-scripting` scene or audio callback was
  timed; the audio block is the only product-shaped bench.
- **Pacing under a memory limit.** The benches run without one, so the
  16-instruction safepoint path and the fast-body `needs_gc` fallbacks
  under pressure are covered by the budget suite and the collector tests
  for correctness, not measured.
- **The integer fast bodies (ids 94 to 132)** run only in unit tests; the
  pinned compiler flags never emit them.
- **The 4.3 security property** (callee registers nil on entry) is kept by
  construction; no hostile-bytecode test was added.
- **The generic host with more than one host type**: one instance only.
- **Instruction counts only.** No wall-clock or cache-miss numbers; the
  host was loaded throughout (loads beside each table).
