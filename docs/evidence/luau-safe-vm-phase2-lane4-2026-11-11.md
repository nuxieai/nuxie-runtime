# Safe Luau VM, Phase 2 lane 4: coroutines and re-entry (2026-11-11)

Lane 4 of Phase 2 in `docs/luau-safe-vm-plan.md` ("How Phase 2 runs"):
coroutines, errors, and stack-safe re-entry. Written on
`claude/luau-vm-lane4-coroutines-8ca998`, rebased onto `origin/main`
`3452bcfe36` (lane 3, the incremental collector). Not pushed.

**Outcome.**

- **The lane 1 open risk is closed.** These run as VM frames with
  continuations, not Rust recursion:
  - metamethods;
  - `pcall` and `xpcall`;
  - `tostring`;
  - `table.sort` comparators;
  - `string.gsub` callbacks;
  - the `__iter` and generic-for iterator calls.

  The paths that still recurse from Rust go through `Vm::call_value`:
  - host callbacks that call back into the VM;
  - `string.format`'s `%*` calling `__tostring`;
  - `HostCtx::tostring_value`.

  They are bounded by the new `Limits::max_reentry` (16).
  `reentry_reaches_its_limits_on_a_one_mib_stack` shows every limit is
  reached without overflow on a 1 MiB thread in the unoptimized build.
- **Coroutines.**
  - Threads are heap arena objects addressed by `ThreadId`
    (`Value::Thread`).
  - The full `coroutine` library is implemented, with upstream's error
    text: `create`, `resume`, `running`, `status`, `wrap`, `yield`,
    `isyieldable`, `close`.
  - Yields cross `pcall`/`xpcall` and the other frames upstream allows
    (the `CountedYieldable` ones).
  - Metamethods, sort and gsub are not yieldable, as upstream: `attempt to
    yield across metamethod/C-call boundary`.
- **The collector knows threads.** This follows lane 3's evidence,
  section 6:
  - Threads are a `GcRef` kind and are re-queued on `gray_again`.
  - Stacks are external buffers, charged and credited exactly.
  - `accounting_drift` now recounts the stacks themselves.
- **Errors.**
  - The `xpcall` handler runs on the intact stack, at upstream's call
    level.
  - `error in error handling` comes at 225 C calls, or when the call-info
    overflow headroom runs out.
  - `pcall.luau:66`'s 10,000 nested pcall results match.
- **Host API.**
  - `Vm::start` and `Vm::resume` return `RunState::{Finished, Yielded}`,
    so a host can run a main chunk that yields.
  - `Vm::run` is unchanged for callers and stays non-yieldable.
- **Conformance: 15 of 55 required** (lane 1 had 14).
  - `ndebug_upvalues` moved to required.
  - `coroutine`, `pcall`, `closure` and `gc` pass whole in patched copies
    (section 4).
  - None of them can pass unpatched while D8 holds.
- **Differential corpus: 120 runs, 0 disagreements, 0 accounting drift.**
  This adds `coroutines.luau`, `coroutine_fenv.luau` and
  `coroutine_overflow.luau`, and extends `reentry.luau`.

## 1. Commits (on `3452bcfe36`)

| Commit | What |
| --- | --- |
| `feat(luau-vm): every call a frame; coroutines as arena threads` | New: `call.rs`, `thread.rs`, `libs/sort.rs`, `libs/coroutine.rs`. Changed: the interpreter, metamethods, base library, gsub and host API, and threads in the collector (`GcRef::Thread`). One commit, because the call protocol, the meta step functions and the thread switch only compile together. |
| `test(luau-vm): coroutine and re-entry diff cases; 1 MiB limit test` | `coroutines.luau`, `reentry.luau`, the limit test, and the nuxie conformance harness running on `Vm::start`. |
| `test(luau-conformance): require ndebug_upvalues; lane 4 blockers` | Scope file. |
| `fix(luau-vm): move coroutine transfer values (lua_xmove)` | Review round 1. |
| `fix(luau-vm): per-thread globals (setfenv(0) in a coroutine)` | Review round 2. |
| `fix(luau-vm): a coroutine whose results do not fit stays suspended` | Review round 3. |
| `fix(luau-vm): charge the main thread's first stack; recount stacks in the drift check` | Integration with lane 3. |
| `test(luau-vm): coroutine budget cases; parked-thread stores survive a cycle` | Integration with lane 3. |
| `test(luau-conformance): lane 4 scope after lane 3; match scope.txt's non-goals` | Scope file. |
| `docs(luau-vm): Phase 2 lane 4 evidence` | This document. |

**Final gates.** All were run on the rebased tip and all exit 0. Host
load average at the start was 20.4 / 13.6 / 11.7, with other lanes
building.

- `make luau-vm-diff`:
  - 120 runs, 0 disagreements;
  - every admitted opcode reached;
  - drift checked on every run.
- `make luau-conformance-nuxie`:
  - 15 OK, 40 FAIL, 0 TIMEOUT;
  - 15 required (all pass), 36 expected-fail, 4 permanent-non-goal.
- `make luau-conformance` (luaur): 28 required, all pass.
- `make luau-vm-ratchets`, `make ratchet-unsafe` and
  `make ratchet-panic-lints`: PASS. The crate is still at 0 in both
  ratchets.
- `python3 tools/bazel/generate.py --check`: clean.
- Clippy for `nuxie-luau-vm`, `luau-vm-diff` and `luau-conformance`: no
  warnings in the crate or the tools.
- Unit tests:
  - `nuxie-luau-vm`: 4;
  - `luau-vm-diff`: 28, including the 1 MiB re-entry test, the budget
    suite and the collector tests;
  - `luau-conformance`: 6.

## 2. Design

**Every call is a frame.** The protocol lives in `call.rs`.

- A native that needs to call a function returns:
  - a `Request`: `Call`, `Yield` or `Resume`;
  - a `Continuation`: a trait object that holds no `Value`, so the
    collector never traces it.
- The driver (`execute_from` → `run_frames`) pushes the callee. When the
  callee returns, the driver hands its results to the continuation.
- A frame's `Wait` says what the frame is waiting for: `Idle`, `Fresh`,
  `Op(Finish)`, `Native`, `Ready` or `Yield`.
- Instruction-level metamethods:
  - The `meta.rs` step functions return `Meta::Done(v)` or
    `Meta::Call(MetaCall{.., post})`.
  - The interpreter pushes the call with a `Finish{kind, post, pc}`.
  - `finish_op` then completes the instruction. The kinds are store,
    branch, concat continuation, namecall, import, iterator prep and
    generic-for.
- `table.sort` is upstream's introsort written as a state machine
  (`Sorter`). Its comparison order is identical to C++.
- `gsub` resumes from a `Gsub` state.

**C-call accounting** follows upstream.

- Each thread has its own `c_calls` and `base_c_calls`. A thread can yield
  iff `c_calls <= base_c_calls`.
- `CallMode` covers the four ways upstream enters a call: `Plain`,
  `Checked`, `Counted` and `CountedYieldable`.
- Saved counters carry an epoch. After a yield and resume, a restore uses
  the resumed level, not a stale one.

**Call-info limit.**

- C++ counts its base CallInfo, so the check is `frames + 1 >= max_calls`
  → `stack overflow`.
- The first overflow sets `ci_overflow`. That grants headroom up to
  `max + max/8` before `error in error handling`, so an `xpcall` handler
  can run at overflow.
- A catch or unwind clears `ci_overflow` once the depth is back under the
  limit.

**Threads are checked out.**

- The running thread's state lives in `Vm` fields: stack, top, frames,
  open upvalues, pc, the call counters, the epoch, `ci_overflow` and
  `globals` (C++ `L->gt`).
- `switch_to` swaps these with the thread's arena slot using `mem::swap`.
  The interpreter's hot paths therefore never index the thread arena.
- Open upvalues are `Upval::Open(ThreadId, slot)`, so a closure created in
  one coroutine reads the right stack from another.
- Transfers move values, as `lua_xmove` does: arguments leave the resumer,
  and yielded values leave the coroutine.

**What a thread roots.** `Thread::trace` marks:
- its stack up to the live top (the innermost frame's top if that is a
  Lua frame, else `top`, the same rule gc.rs applies to the running
  thread);
- every frame's closure;
- its open upvalues;
- its resumer;
- the error a dead thread keeps for `coroutine.close`;
- its globals.

Two more roots:
- The running thread is rooted through the `Vm` fields (`gc_roots`).
- Its id and the main thread's id are roots, so their arena slots stay
  allocated.

Marking an open upvalue marks its thread, so a reachable closure keeps
the thread whose stack its value lives in.

**Collector integration** (lane 3's evidence, section 6):
- A traversed thread goes back on `gray_again`, never black, because
  stack stores are not barriered. In the atomic step it turns black, so
  `propagate_all` terminates.
- A thread costs `THREAD_BYTES` plus its stack. The stack is an external
  buffer (budget.rs, rule 4):
  - charged when the thread is created and by `ensure_stack` as it grows;
  - credited by `coroutine.close` (`reset_thread`) and by the sweep
    (`Budget::credit_external`).
- `accounting_drift` now also compares the external count with the stacks
  recounted from their real lengths: the running thread's plus every
  parked one's. This stricter check found that the main thread's initial
  64 slots were never charged. That was a gap on main, and it is fixed
  here.
- Weak tables clear threads like any other object.

## 3. Measurements

Stack cost of one Rust re-entry level was measured in fastbuild
(unoptimized), on aarch64 macOS, by bisecting the thread size at which a
probe overflowed:
- **46,480 bytes per host-callback level;**
- **49,056 bytes per `string.format %*` level.**

With 1 MiB and the frames above the VM, 16 levels fit with margin, hence
`max_reentry = 16`. No timing claims are made.

## 4. Scope (`tests/luau-conformance/scope-nuxie.txt`)

`closure`, `gc`, `pcall` and `cyield` are `permanent-non-goal` here, as in
`scope.txt` (Rive luaL_where, or host hooks). Their reason lines give the
nuxie blockers. "Patched copy" means:
- the other lanes' lines are polyfilled or skipped;
- the D8 strings are rewritten;
- line numbers are kept.

The patched copies are scratch and not committed.

- **`ndebug_upvalues`:** required (the harness resumes it on yield).
- **`coroutine`** (expected-fail):
  - Unpatched, it first fails at :286. Nuxie's `table.unpack` lacks the
    `LUAI_MAXCSTACK` check (lane 2, :276-286).
  - The patched copy passes. Patches:
    - `table.create` (lane 2, :314);
    - the unpack block;
    - D8 at :302, :392 and :411.
  - Lane 3's weak tables now pass :205, which needed a skip before the
    rebase.
- **`pcall`:** the patched copy passes. Patches:
  - `table.move` (lane 2, :39);
  - D8 at :80 and :120/:121;
  - `debug.traceback` (lane 5, :106/:111);
  - host `cxxthrow`/`resumeerror` (lane 6, :132, :144/:146);
  - the `table.create(1e6)` memory-limit blocks (:172-176, :198-200).
- **`closure`:**
  - Unpatched, it fails at :210: the coroutine calls `table.getn`
    (lane 2, :198).
  - With `getn` polyfilled and the D8 line :293 skipped, it passes whole.
- **`gc`:** the patched copy passes whole, including the three coroutine
  blocks (:313, :359, :384). Patches:
  - `table.clear` and `table.create` polyfilled (lane 2);
  - the `newproxy` blocks :244-298 and :333-356 skipped (lane 5).
- **`cyield`:** needs host C yield helpers (`singleYield` and others),
  which need a host continuation API (lane 6).
- **`errors`:** needs `debug.traceback` (lane 5).

## 5. Shared files touched

- `vm.rs`:
  - frame fields: `nargs`, `thread_base`, `restore`, `wait`;
  - the checked-out thread fields;
  - `unwind(depth, func, Saved)`;
  - `call_tm_res` removed;
  - thread-aware upvalue access;
  - the initial stack charge.
- `meta.rs`: the plain wrappers (`get_table`, `set_table`, `arith`,
  `less_than`, `less_equal`, `equal`, `length`, `concat`, `call_bin_tm`,
  `truth`) are replaced by `*_step` plus `meta_now(step)`.
- `heap.rs`:
  - the `threads` arena and `GcRef::Thread`;
  - `new_thread`, `thread`, `thread_mut`, `thread_stack_bytes`,
    `credit_external`;
  - the sweep and the weak-table arm;
  - `Upval::Open(ThreadId, slot)`.
- `budget.rs`: `credit_external`.
- `gc.rs`:
  - thread ids in `gc_roots`;
  - `accounting_drift` recounts stacks.
- `value.rs`, `table.rs`: `ThreadId`, `Value::Thread`, its hash.
- `interp.rs`: the instruction loop is `run_frames`; CALL and RETURN go
  through the protocol.
- `host.rs`:
  - `RunState`, `start`, `resume`;
  - `run` rebuilt on shared helpers.
- `limits.rs`: `max_reentry`.
- Libraries:
  - `libs/base.rs`: pcall, xpcall, tostring;
  - `libs/string.rs`: gsub;
  - `libs/table.rs`: sort moved out to `libs/sort.rs`.
- Tools:
  - `tools/luau-conformance/src/harness_nuxie.rs`;
  - `tools/luau-vm-diff`: corpus, limit test, `budget_suite.rs` (the
    coroutine cases and the `NON_ALLOCATING` entries),
    `collector_tests.rs`.

## 6. Review

`codex review --base origin/main`, three rounds. Each round's finding was
reproduced as a differential case before the fix. Without the fix, each
case fails on all four profiles.

1. **Transfers copied values instead of moving them.** 4,500 values in and
   out failed with `too many results to resume`. Fixed with `lua_xmove`
   semantics; the case is in `coroutines.luau`.
2. **`setfenv(0, t)` in a coroutine leaked into its resumer.** Globals are
   now per thread; the case is `coroutine_fenv.luau`. That case lives in
   its own file: appending it to `coroutines.luau` makes the vendored
   luaur compiler crash (`EXC_BAD_ACCESS` in `DenseHashTable::find`).
   That crash is a compiler bug outside this VM and is not fixed here.
3. **A coroutine whose results did not fit its resumer was marked dead.**
   C++ keeps the results on it and reports it as suspended; the next
   resume calls the last result. Fixed; the case is
   `coroutine_overflow.luau`, with values built by Lua recursion so lane
   2's unpack limit does not decide it.

Collector integration was mutation-checked:
- Turning a traversed thread black fails
  `stores_into_a_parked_coroutine_survive_the_cycle`.
- Dropping the stack credit on sweep fails that test and the budget suite.

## 7. Not verified

- **`ndebug_upvalues`:** upstream's C++ `lua_getupvalue` check is not run,
  because the harness has no debug API (lane 6). The script passes
  without it.
- **Re-entry depth:** Rust re-entry stops at 16 levels, where C++ allows
  about 200 C calls. Scripts that legitimately nest host callbacks deeper
  than 16 now get `C stack overflow` earlier than upstream. No
  conformance script does.
- **Patched copies:** the `coroutine`, `pcall`, `closure` and `gc` passes
  are on patched copies, not the committed corpus.
- **`coroutine.wrap` and the memory error:** when a memory error passes
  through `wrap`, its kind is not checked against upstream. `wrap_error`
  prefixes a string error and raises a runtime error.
- **Shrinking:** thread stacks never shrink. C++ shrinks them in the
  collector, which matters only for memory, and the drift check covers
  the accounting.

## 8. Proposed plan lines

- 2026-11-11: **lane 4 done** (coroutines; every call a frame). The lane
  1 re-entry risk is closed: Rust recursion remains only in `call_value`,
  bounded by `Limits::max_reentry` (16) and proven on a 1 MiB
  unoptimized stack. Threads are collector objects (`GcRef::Thread`,
  re-queued gray) with exact stack accounting. 15 scripts are required.
- Lane 4 leftovers:
  - `cyield` → lane 6 (host continuation API);
  - `coroutine`, `pcall`, `closure` and `gc` wait on lane 2
    (`table.create`, `move`, `getn`, `clear`, and `LUAI_MAXCSTACK` in
    unpack), on lane 5 (`debug.traceback`, `newproxy`), and on D8.
- Follow-up: the vendored luaur compiler crashes on
  `tests/luau-vm-diff/coroutines.luau` with the fenv case appended.
