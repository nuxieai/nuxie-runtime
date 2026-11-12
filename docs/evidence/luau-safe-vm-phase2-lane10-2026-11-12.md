# Safe Luau VM, Phase 2 lane 10: the register window (2026-11-12)

Lane 10 of Phase 2 in `docs/luau-safe-vm-plan.md`: build
`docs/luau-safe-vm-register-window.md` in `crates/nuxie-luau-vm`, stage by
stage in its section 7 order, each stage checked against its rows in
section 6. Branch `claude/luau-vm-lane10-register-window-8a57e3`, on
`410e8d745b` (`origin/main` has since moved to `92ce3fc840`); not pushed,
no PR.

**Outcome.**

- **Built**: the register window with the 256-slot stack slack, register
  arms, cold slow tails and `exec.pc` written only by general arms (stage
  1); table reads and existing-key stores beside the window, and
  `Table::set` as one lookup (stage 2); `CALL` and plain `RETURN` in the
  window (stage 3); `pcall` of a Lua function entered from `CALL`
  without running the native or the request protocol, completed in place
  when its callee returns (stage 4); `Box<Exec>`, so a thread switch swaps
  two pointers, and `resume`/`wrap` completed in place when their
  coroutine yields (stage 5).
- **Not built as designed**: the `CATCH` frame flag (4.7) and the direct
  resume/yield with an `AWAIT` frame (4.8). The catch already lives in
  `pcall`'s pending continuation; the design's direct yield leaves no frame
  for a later resume to deliver to (section 5).
- **Floor rows: 3 of 10 inside their ranges at the tip (4 after stage
  2), the rest 1 to 9 instructions above**; every row is 1.4x to 4.8x
  below `main` (`t.x = i` 424 → 88, `a, b = b, a` 160 → 75).
- **Call rows: none inside their ranges.** Stage 3 took 3 to 19% off
  the Lua-call rows; stage 5a's `Box<Exec>` put about 25 instructions
  back on every call, so at the tip `f()` is 5% dearer than `main`,
  `f(i, i)` 15% cheaper, `pcall(f, i)` 25% cheaper and `co()` 6%
  cheaper. The excess is
  attributed in section 4: the dispatch head and spilled loop locals of
  one ~8,800-instruction loop function (every row), the general native
  protocol that `pcall` and coroutines still go through, and `Box<Exec>`
  (+25 per call).
- **Suite**: 13% fewer instructions over 89 benches (geomean 0.868; 69
  fewer, 13 more, 7 within noise); 3.39x to 2.95x luaur over the 87
  non-audio benches; the audio block 25% fewer (2.0x to 1.44x and 1.50x
  luaur). Three of the design's eight group ranges hold (builtin calls,
  arithmetic, the audio block); the others are 0.05x to 0.31x above.
- **What got dearer is attributed by stage** (section 7.2): stage 1
  doubles `Pcall_pcall_yield` (a new coroutine per iteration; the
  256-slot slack, inferred) and costs the allocation-heavy string benches
  6 to 10% (not traced); vector arithmetic's cold tail costs
  `vector-math` 16%; `Box<Exec>` costs `xpcall` 5.6%. Stage 2's double
  lookup on a field miss (`OOP_method_access` +17%) is fixed on the tip
  (+2.5%).
- **Gates green on the tip** (section 3), including `nuxie-scripting` 40 of
  40 under both settings. No `unsafe`; the panic lints stay at zero. Three
  `codex review` rounds: the first found two defects (fixed, section 6),
  the second and third (after the `GETTABLEKS` fix) none.

## 1. Commits

| Commit | Stage |
|---|---|
| `65f8f31314` | test: the stack-overflow depth at four limits, pinned on the code before the window |
| `7eeccc9136` | 1: the register window |
| `6cbc5b6d2d` | 2: tables beside the window |
| `9df6df6bbb` | 3: calls and returns in the window |
| `4db8f18637` | 4: `pcall` of a Lua function entered in the loop, with `tests/luau-vm-diff/pcall_frames.luau` |
| `9444770919` | 5a: `Box<Exec>` |
| `6ad6df4639` | 5b: `resume`/`wrap` complete when their coroutine hands back |
| `3b0d5375b5` | review fixes: `pcall` completions in a loop; a swept thread keeps its box |
| `cb0495a859` | `GETTABLEKS` enters the `__index` chain at the metatable after its own miss |
| (the next commit) | the instruction table; the design as built; the plan entry; this record |

## 2. What was built, stage by stage

### Stage 1: the window (4.1 to 4.3, 4.5)

- `Core::ensure_frame(need, base)`: the limit check keeps its
  expression on `need`; physically the stack holds `base + 256` (up to
  `max_stack_slots + 256`). `65f8f31314` pins the overflow depth and
  message for a fixed frame, a vararg frame and a coroutine at limits 300,
  1,000, 4,096 and 50,000; they did not move.
- `run_frames` carries `regs: &mut [Value; 256]`, re-derived after every
  general arm and before every bare `continue` in one. Reads and writes go
  through `get`/`get_mut` on a `u8` index (clippy denies indexing; the
  check folds away).
- Loop locals: `closure`, `base`, `pc`, `code`, `k`; `exec.pc` is written
  by `sync_pc!` at the top of general arms only.
- Register arms: loads, moves, upvalue reads and closed-upvalue writes,
  jumps (the interrupt read by field), conditional jumps on numbers and
  raw-comparable values, number arithmetic, `NOT`, `MINUS`, `LENGTH` of a
  plain table or a string, `FORNPREP`, `FORNLOOP`, `JUMPXEQK*`.
- Slow tails are `#[cold] #[inline(never)]` and re-read their operands
  from the instruction word. The first build passed the operands by
  value, which made the number path read its registers through stack
  memory; re-reading moved `s = s + i` 39 → 36 and three adds 117 → 108.

### Stage 2: tables (4.4)

- Read arms (`GETTABLEKS` via `get_str`, `GETTABLEN`, `GETTABLE`) and
  existing-key store arms through `Heap::table_slot_mut` and
  `Heap::table_str_slot_mut` (barrier, then one lookup; `None` for a
  read-only table or an absent key).
- `Table::set` is one lookup (`slot_mut`), with only a new key reaching
  the sizing and budget path.
- The first build of `t.x = i` measured 136: `table_slot_mut` was out of
  line with the key in memory and used the generic `find_node`. Inlined,
  string-specialised, with the barrier's regray out of line: 85.

### Stage 3: calls and returns (4.6)

`CALL` reads its callee from the window before the sync; `base` changes
only with the running frame, so general arms stopped reloading it; a plain
`RETURN` moves its results through one slice from the caller's result
slot; `close_upvals` is one inline compare with the walk out of line.

### Stage 4: `pcall` (4.7)

- `NativeFn::Pcall(ResumableFn)`: the kind `CALL` recognises; every other
  way in runs it as `Resumable`.
- `CALL` of it with at least one argument and a Lua function to protect:
  `Core::enter_pcall` pushes `pcall`'s native frame waiting on its
  `Protected` continuation, then the callee (`push_call` with
  `protected_mode`). Exactly the frames the native leaves after its
  request; the native and the request protocol do not run.
- Return: `deliver` reads a waiting `Protected` without a handler in place
  and completes `pcall` (`true` in its slot, the results already above
  it), in a loop for consecutive completions.
- `tests/luau-vm-diff/pcall_frames.luau` (four profiles, against luaur):
  results at every count; error values and depths; upvalues closed on the
  unwind; nested `pcall`; `pcall` of a native, a `__call` value, a number
  and `pcall`; `debug.info` levels and a traceback through it; a yield
  inside, then a return, then an error; `isyieldable` inside; the C-call
  counters after 60 caught errors and a depth probe.

### Stage 5: threads (4.8)

- `Core::exec` and `Thread::exec` are `Box<Exec>`; `swap_exec` swaps two
  boxes. A swept thread resets its fields in place and `coroutine.close`
  takes the state out of its box, so neither allocates.
- After a yield, a `resume` or `wrap` waiting on the coroutine completes in
  place (`resume_returned`: the values move once, after `true` for
  `resume`). A coroutine that finishes or fails takes the request path.

## 3. Gates

On the tip, sequential, one Bazel server, load 8.5 to 25 (1-minute), free disk 68 to 72 GiB.

| Gate | Result |
|---|---|
| `runtime.py clippy -p nuxie-luau-vm` | clean |
| `nuxie-luau-vm` tests | 73 passed |
| `luau-vm-diff` tests | 57 passed (one new: nested `pcall` completions) |
| `make luau-vm-diff` | 229 runs, 0 disagreements, bytecode 7 and 11, every admitted opcode reached |
| `make luau-conformance-nuxie` | PASS, 36 of 36 required |
| `make luau-vm-ratchets` | PASS (`unsafe` 0, panic lints PASS) |
| `make luau-conformance` | PASS, 30 of 30 required |
| `tools/bazel/generate.py --check` | OK |
| `runtime.py clippy -p luau-bench` | exit 0; warnings only in vendored crates |
| `nuxie-scripting`, default | 40 of 40 (Bazel cache hits: none of their inputs changed since the run on `3b0d5375b5`) |
| `nuxie-scripting --features nuxie-vm` | 40 of 40 |

`nuxie-scripting` ran with `RIVE_RUNTIME_DIR` at the pinned rive-runtime
checkout (`cd5351c3`, `/Users/levi/dev/worktrees/rive-runtime-pin-cd5351c3`).
The same battery passed after each stage (stage logs: stage 1 load 50 to
67, stage 2 21 to 40, stage 3 13 to 34, stage 4 23 to 47, stage 5a 22 to
35).

## 4. Predictions and measurements (design section 6)

Per iteration over the empty loop, retired instructions
(`LUAU_BENCH_CLOCK=instructions`, Bazel fat-LTO release), micro scripts
of 200,000 iterations, median of 2 runs. The empty loop is raw. "now" is
the design's column (`main` `86ed289f1c`); our base run on `65f8f31314`
matched it within 1 to 2 on every row. Each stage column is measured
right after that stage's commit; "tip" is `cb0495a859`. luaur at the tip run.

### 4.1 Floor rows

| Loop body | now | stage 1 | stage 2 | tip | range | in? | luaur |
|---|---:|---:|---:|---:|---|---|---:|
| empty loop (raw) | 87 | 63 | 56 | 56 | 38 to 55 | +1 | 38 |
| `s = s + i` | 64 | 36 | 36 | 36 | 22 to 36 | in | 33 |
| three adds | 193 | 108 | 108 | 108 | 70 to 105 | +3 | 99 |
| `a, b = b, a` | 160 | 75 | 75 | 75 | 40 to 66 | +9 | 51 |
| `if i < 0 then s = i end` | 123 | 72 | 72 | 71 | 36 to 62 | +9 | 48 |
| `s = i * 2` | 70 | 41 | 41 | 41 | 25 to 40 | +1 | 29 |
| `s = t.x` | 38 | 27 | 27 | 27 | 14 to 25 | +2 | 20 |
| `s = t[1]` | 171 | 182 | 54 | 54 | 35 to 55 | in | 29 |
| `t.x = i` | 424 | 433 | 85 | 88 | 50 to 85 | +3 (in at stage 2) | 41 |
| `s = t[(i % 8) + 1]` | 314 | 269 | 163 | 162 | 110 to 170 | in | 104 |

Attribution, every miss is (c), from the release disassembly of
`run_frames` (`otool`, the jump table decoded per opcode):

- **The dispatch head is 19 instructions, not ~10.** `code`'s pointer and
  length are reloaded from spill slots, the bounds check, the opcode range
  check, three operand decodes, and the jump table's base rematerialised
  every dispatch. `a, b = b, a` is three dispatches of 19 plus `MOVE`'s 6.
- **`k` and `base` are spilled too.** `MULK`, `LOADK` and the
  `GETTABLEKS` arm reload `k`'s pointer and length; every window access
  computes from a reloaded `base`.
- Cause: register allocation of a single match in an ~8,800-instruction
  function. The prototype, whose numbers set these ranges, had a small
  first match and kept the locals in registers. Tried and not kept:
  dropping more locals (no change), a three-way `FORNLOOP` match (56 →
  63, layout).

### 4.2 Call rows

| Loop body | now | stage 3 | stage 4 | stage 5a | stage 5b | tip | range | luaur |
|---|---:|---:|---:|---:|---:|---:|---|---:|
| `f()` | 335 | 325 | 325 | 352 | 352 | 353 | 180 to 260 | 146 |
| `local y = f(i)` | 415 | 379 | 380 | 404 | 403 | 404 | 220 to 320 | 169 |
| `local a, b = f(i, i)` | 614 | 498 | 500 | 523 | 522 | 523 | 290 to 430 | 227 |
| `o:m(i)` | 597 | 569 | 572 | 597 | 596 | 598 | 300 to 440 | 215 |
| `math.abs(i)` | 127 | 118 | 118 | 125 | 125 | 125 | 85 to 115 | 68 |
| `math.min(i, 5)` | 169 | 161 | 162 | 167 | 167 | 167 | 115 to 155 | 79 |
| `select('#', i)` | 424 | 402 | 405 | 415 | 415 | 416 | 250 to 360 | 238 |
| `pcall(f, i)` | 1,792 | 1,765 | 1,316 | 1,321 | 1,319 | 1,347 | 380 to 600 | 742 |
| `co()`, swap only | 2,309 | 2,294 | 2,378 | 2,223 | | | 1,900 to 2,100 | 566 |
| `co()`, direct (not built; in-place completion instead) | 2,309 | | | | 2,121 | 2,174 | 500 to 950 | 566 |

The tip's `pcall` and `co()` rise over stage 5b (+28, +53) is the review
fix that turned `deliver`'s completion into a loop and moved the resume
short-cut to `yield_thread`.

Attribution:

- **`f()`, `f(i)`, `f(i, i)`, `o:m(i)` (c).** A static walk of the
  Lua-fixed `CALL` path in the disassembly (lldb single-stepping ran at
  about 10 ms an instruction, so no dynamic trace) counts about 174
  instructions against the design's ~95: dispatch head 19; interrupt flag,
  `CALLFB` test, argument count and `top` ~28; the function and prototype
  lookups, `Arc::ptr_eq` and `is_vararg` ~28; the depth compare and both
  stack compares ~22; the nil fill ~12; `savedpc` and the frame push with
  its six narrowed fields ~35; the switch, with `code`, `k` and `base`
  stored to their spill slots and the window re-derived, ~25. Every step
  is the design's; the excess is the same register allocation as the
  floor rows, paid twice per call (a switch in and a switch out).
  `GETUPVAL` plus `RETURN` together are ~95.
- **Stage 5a added about 25 to every call row** (`f()` 325 → 352) and 6
  to 7 to the fast builtins: `exec` is behind a pointer that the call path
  reloads at each access, more than the design's "one pointer load".
- **`math.abs`, `math.min`, `select` (c)**: the dispatch head on
  `FASTCALL1`/`FASTCALL2K` and `CALL`, plus the box.
- **`pcall(f, i)` (b and c)**: built smaller than the design (section 5).
  The design's 380 to 600 assumed the C-call counters fit the frame's
  spare field and the callee entered as 4.6. Neither holds: `Saved` is 12
  bytes and a native frame's spare field (its prototype id) is 4, so a
  104-byte pending entry carries them; on a thread that cannot yield,
  `CountedYieldable` puts a second pending entry on the callee, whose
  `RETURN` is then general. `enter_lua_fixed` writes the caller's
  `savedpc` into whatever frame is on top, which for `pcall`'s native
  frame is its argument count; the callee goes through the out-of-line
  `push_call`/`enter_lua` and re-enters the frame loop.
- **`co()` (b)**: section 5. Swap alone 2,223 (range 1,900 to 2,100),
  with the in-place completion 2,174 at the tip.

### 4.3 Groups against luaur

Geomean nuxie/luaur per group, the 87 non-audio benches (`groups.py`
over the committed table before and the regenerated table after; luaur
from one pass at load 20 to 22):

| Group | benches | before | after | design range | in? |
|---|---:|---:|---:|---|---|
| builtin calls | 21 | 2.27x | 1.91x | 1.5x to 2.0x | in |
| Lua calls and returns | 8 | 2.76x | 2.41x | 1.5x to 2.1x | +0.31x |
| table get/set by constant key | 7 | 2.89x | 2.06x | 1.3x to 2.0x | +0.06x |
| array access and iteration | 14 | 2.81x | 2.21x | 1.4x to 2.1x | +0.11x |
| arithmetic and loops | 2 | 2.23x | 1.35x | 1.1x to 1.5x | in |
| string building and formatting | 18 | 4.88x | 4.83x | (not touched) | |
| closures and upvalues | 1 | 2.79x | 2.78x | | |
| coroutines | 2 | 3.08x | 4.25x | | |
| table library bulk ops | 10 | 9.82x | 8.91x | (not touched) | |
| error path | 4 | 1.89x | 1.84x | 1.0x to 1.6x | +0.24x |
| all 87 | 87 | 3.39x | 2.95x | 2.3x to 2.9x | +0.05x |
| audio block, shared controls | 1 | 1.94x | 1.44x | 1.3x to 1.7x | in |
| audio block, sine | 1 | 2.01x | 1.50x | 1.3x to 1.7x | in |

Each miss follows its rows: Lua calls carry the call-row excess
(section 4.2); the table and array groups the floor's dispatch head; the
error path goes through the general native protocol (`pcall` of a native
that fails, and `xpcall`, which this lane did not touch). Coroutines got
dearer: `Pcall_pcall_yield` (section 7.2) outweighs the cheaper switch.

## 5. What in the design did not survive, and what was done instead

1. **`CATCH` frame flag (4.7).** The fact "this frame catches" already
   lives in `pcall`'s pending continuation (`Wait::Native(Protected)`),
   which `find_catcher` scans; a flag on the frame beside it would be a
   second record of the same fact. Built instead: the native's own frame
   shapes, pushed from `CALL` without running the native, and an in-place
   completion on return. `find_catcher`, `finish_catch`, yields,
   observers and the counters are unchanged, which is why the new
   differential case agrees everywhere. The design's counters-in-the-frame
   step does not fit (12 bytes against 4).
2. **Callee entered inline under `pcall` (4.7 step 2).**
   `enter_lua_fixed` would overwrite the native frame's argument count
   with the caller's `savedpc`. Not generalised in this lane; the callee
   uses `push_call`.
3. **Direct resume and yield (4.8).** The design moves the yielded values
   to the resumer, pops the `AWAIT` frame and swaps. But a yield must
   leave behind the frame its resume delivers to: whoever resumes the
   coroutine next (the host API, `coroutine.resume`, a wrap function),
   `complete_yield` pops the coroutine's top frame, the yield's, to put
   the resume values in the yielding `CALL`'s registers. The design's
   path keeps no such frame. Stopped at the pointer swap as the brief
   says, with one smaller step that keeps every shape: after a yield, a
   waiting `resume` or `wrap` completes in place (2,223 → 2,174 at the tip).
   **`Box<Exec>` is net dearer for call-heavy code**: about 140
   instructions off a thread switch, about 25 on every Lua call and 6 on
   every fast builtin. It is one commit (`9444770919`); keeping it is the
   coordinator's call.
4. **"Each thread's stack starts at 256 + `MIN_STACK`" (4.1).** Built
   lazily: a new coroutine starts at 40 slots and grows on its first
   frame, so the memory charge lands where it did (first resume), and
   memory-limit errors fire where they did. The instruction cost of the
   slack per coroutine was not predicted (section 7.2).
5. **Floor and call ranges (section 6).** Set from a prototype whose
   small loop kept its locals in registers; the built loop spills them.
   No design step was skipped for these rows.

## 6. Review

`codex review --base origin/main`, three rounds.

- Round 1, two findings, both real and fixed in `3b0d5375b5`:
  - **P1**: `deliver → pcall_returned → deliver` recursed on the host
    stack; 6,000 nested `pcall(pcall, ...)` levels on a yieldable thread
    (no C-call count bounds them) overflowed a 1 MiB thread where the
    request path completed. Now a loop; `resume_returned` runs only after
    a yield, outside `deliver`. `luau-vm-diff`'s
    `nested_pcall_completions_do_not_recurse_on_the_host_stack` runs 6,001
    levels on a 1 MiB thread; on `6ad6df4639` it aborts with a stack
    overflow (checked).
  - **P2**: the sweep reset a dead thread with `Thread::default()`, which
    allocated a new `Exec` box per collected coroutine (1,000 host
    allocations for 1,000 dead coroutines against 0 before). Reset in
    place now; not covered by a test (no counting allocator in the crate
    tests).
- Round 2: no findings.
- Round 3, on `cb0495a859`: no findings.

## 7. The suite

### 7.1 Before and after

`make luau-vm-perf-report`, 3 passes, Bazel fat-LTO release,
`LUAU_BENCH_CLOCK=instructions`. **Before** is the committed table (lane
9's, `main`). **After** is the tip, run with `LUAU_VM_PERF_UPDATE=1`, so
`tests/luau-bench/instructions-nuxie.json` is the after table. luaur is
the same harness on the vendored VM, one pass at the tip (load
20 to 22).

Geomean after/before **0.868 over 89 benches: 69 fewer instructions,
13 more, 7 within noise** (`report.py`, change against the larger
per-bench spread). The audio block: `AudioBlock_shared_controls` 36.24 M
→ 27.02 M (-25.4%), `AudioBlock_sine` 35.31 M → 26.27 M (-25.6%). The
largest falls are the stage 1 and 2 shapes: arithmetic and field loops,
array reads, fast builtins.

More instructions by at least 3%: `Pcall_pcall_yield` +104%,
`vector-math` +16%, five `StringInterp` benches +6.6 to +9.9%,
`ToNumberString/tostring-gc` +5.9%, `Pcall_xpcall_return` +5.6%. Under 3%:
`OOP_method_access` +2.5% (after its fix), `TableInsertion_index_cached`
+2.2%, `string_lib` reverse/upper/lower (unique) +1.7%, two more under
1%.

### 7.2 Benches that got dearer

A cargo fat-LTO build of `luau-bench` at each lane commit, in a scratch
worktree, one pass over the suspects (millions of instructions; cargo's
and Bazel's absolute counts differ by about 1%):

| Commit | `OOP_method_access` | `Pcall_pcall_yield` | `Pcall_xpcall_return` | `vector-math` | load (1 min) |
|---|---:|---:|---:|---:|---:|
| `65f8f31314` (base) | 4,371 | 969 | 198 | 452 | 14.7 |
| `7eeccc9136` (1) | 4,203 | 1,971 | 194 | 542 | 12.4 |
| `6cbc5b6d2d` (2) | 4,931 | 1,976 | 194 | 536 | 10.9 |
| `9df6df6bbb` (3) | 4,848 | 1,964 | 188 | 515 | 28.1 |
| `4db8f18637` (4) | 4,847 | 1,923 | 194 | 515 | 11.1 |
| `9444770919` (5a) | 4,943 | 1,994 | 200 | 528 | 8.5 |
| `6ad6df4639` (5b) | 5,027 | 2,041 | 205 | 528 | 12.3 |
| `3b0d5375b5` (fixes) | 4,946 | 1,952 | 205 | 528 | 17.5 |

The eight `StringInterp` benches moved by +6 to +10% at stage 1 (for
example 11.84 → 13.00 and 18.74 → 19.99) and stayed there.

- **`Pcall_pcall_yield`, 2.0x, stage 1.** One new coroutine per
  iteration. Stage 1's only change a coroutine sees on creation is the
  stack slack: its first frame grows the stack from 40 slots past 256 (a
  reallocation, 4 KiB more charged, nil-filled), and the collector's
  pacing counts the charge. About 10,000 more instructions per coroutine
  (9,700 → 19,700). The mechanism is inferred from the stage and the
  per-iteration delta, not traced. Decision 1 accepted the slack's memory;
  this per-coroutine instruction cost was not predicted.
- **`StringInterp`, 6 to 10%, stage 1, and `ToNumberString/tostring-gc`,
  6%.** Allocation-heavy (formatting and interning); `StringInterp`
  moved at stage 1 in the bisect; `tostring-gc` was not in the bisect
  and is placed with them by kind only. Not traced.
- **`vector-math`, 16 to 20%, stage 1.** Vector arithmetic is not a
  register-arm case: it takes the cold `arith_tail` (an out-of-line call,
  the operands decoded again, `arith_step`, `store_tail`), where `main`
  called `arith_step` from the arm. A step the design leaves out (b):
  vectors are first-class arithmetic in Luau and in Rive scripts. Not
  fixed in this lane.
- **`OOP_method_access`, +17%, stage 2; fixed.** A method lives in the
  class table, so the object's raw lookup misses; the stage-2 arm looked
  up, missed, and the general path looked up again. On the tip the
  general path enters the `__index` chain at the metatable after the
  arm's miss (`Core::get_table_miss_step`): 4,946 → 4,533 (base 4,421).
- **`Pcall_xpcall_return`, +5.5%, stage 5a**: the box on `xpcall`'s
  general native path.

## 8. Not verified

- No dynamic instruction trace of any row: lldb stepping was too slow;
  attributions are from static disassembly walks and stage-by-stage
  builds.
- The `Pcall_pcall_yield` and `StringInterp` mechanisms (section 7.2) are
  inferred, not traced.
- P2 (no allocation on sweep) has no test.
- No product frame (`nuxie-scripting` scene or audio callback) was timed;
  the `AudioBlock` benches are the stand-in.
- Host load was 9 to 67 (1-minute) across the stages, on an 18-core
  laptop shared with other sessions; instruction counts are robust to it,
  and fat-LTO layout moves single benches by 1 to 3%.
- `origin/main` moved during the lane; nothing was rebased or re-run on
  it.
