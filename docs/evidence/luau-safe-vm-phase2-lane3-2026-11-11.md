# Safe Luau VM, Phase 2 lane 3: the collector (2026-11-11)

Lane 3 of Phase 2 in `docs/luau-safe-vm-plan.md` ("How Phase 2 runs"). It
was written on `claude/luau-vm-lane3-collector-3cc6d0`, rebased onto
`origin/main` at `7c49ad91e3` (lane 1 merged).

**Outcome.**

- **Weak tables**: `__mode` with `k`, `v` and `kv`, cleared after the mark
  as C++ Luau does it (`traversetable` + `cleartable`): strings are never
  cleared, keys of a weak-key table keep their values strong (Luau has no
  ephemerons), and the mode is read when the table is traversed.
  `closure.luau` now passes its weak-table loop (line 30) and stops at
  `coroutine.wrap` (line 178, lane 4).
- **Incremental collection** behind the same safepoints: a tri-color mark
  with gray and gray-again lists, a write barrier on every heap store, and
  one atomic step (roots again, weak clearing, sweep). `collectgarbage`
  follows upstream's `lua_gc` (`step`, `count`, `stop`/`restart`,
  `isrunning`, the tuning options) with upstream's pacing defaults.
- **One accounting point**: `budget.rs`. Heap objects are charged before
  they exist and credited when swept; tables grow only through methods
  that take the budget. The count is exact against the cost model, so a
  bypass shows up as drift, which the diff harness and the suites check
  after every run.
- **`gc.luau` is not `required` yet.** Everything in it that is the
  collector's passes (lines 1 to 225: incremental steps, stop/restart,
  weak tables, collection with allocations blocked). Line 226 calls
  `table.clear` (lane 2); after it come `table.create` (lane 2, :235),
  `newproxy` (lane 5, :245) and `coroutine.create` (lane 4, :322). With
  those calls polyfilled and the userdata and coroutine blocks cut, the
  rest, including the 200,000-deep structure, passes (section 4).
- Required scripts stay at **14 of 55**; no regressions.

## 1. Commits

| Commit | What |
| --- | --- |
| `feat(luau-vm): incremental collector, weak tables, one accounting point` | `budget.rs`, `gc.rs`, the heap and table changes, the loader building constants in the arena, `Tm::Mode`, the host API. |
| `feat(luau-conformance): upstream collectgarbage and setblockallocations on nuxie` | The conformance harness drives the real collector. |
| `test(luau-vm): weak-table case; accounting drift and collectgarbage in the diff harness` | `tests/luau-vm-diff/weak.luau`; the drift check after every nuxie run. |
| `test(luau-vm): memory-budget suite and incremental collector tests` | `tools/luau-vm-diff/src/budget_suite.rs`, `collector_tests.rs`. |
| `test(luau-conformance): update the gc and closure blockers on nuxie` | Scope-file reasons for `gc` and `closure` only. |
| `fix(luau-vm): propagate-again phase, weak retraversal, signed step credit` | Review round 1 (section 7). |
| `fix(luau-vm): memory pressure before an incremental step; clear the stack from the live top` | Review round 2 (section 7). |
| `docs(luau-vm): Phase 2 lane 3 evidence` | This document. |

Review-fix commits, if any, are listed in section 7.

## 2. What was built

### 2.1 The collector (`heap.rs`, `gc.rs`)

Plan D2 holds: typed arenas, plain indices, no per-access liveness check,
collection only at safepoints. What changed is *how much* a safepoint
does.

- **Phases.** `Pause` → `Propagate` (incremental) → `Atomic` (one step).
  C++'s propagate and propagate-again are folded into one phase, and the
  sweep runs inside the atomic step instead of incrementally. That is
  deliberate: with the sweep atomic, no object is ever white-and-dead
  while the mutator runs, so the interner can never return a string that
  is about to be swept (C++ needs `changewhite` on a dead string for
  this). The cost is a longer final step, which is unobservable to
  scripts except through `collectgarbage("step")` counts, which the
  pacing below keeps upstream-shaped.
- **Colors.** `Free`, `White`, `Gray`, `Black` per arena slot. New objects
  are white. That is sound because anything they are stored into is white
  or gray (traversed later) or black (barriered), and anything only the
  stack holds is marked when the atomic step rescans the roots.
- **Write barrier** (C++ `luaC_barrierback`): `Heap::table_mut`,
  `table_set`, `table_resize_array`, `function_mut` and `set_upval` turn a
  black object gray and queue it on `gray_again`. It is in the accessors,
  so no store site can forget it. The arenas' raw `get_mut` is private to
  `heap.rs`. The stack is not barriered; it is rescanned.
- **Weak tables.** A table's traversal reads `__mode` from its metatable
  (C++ `gettablemode`: a string, scanned up to a NUL for `k` and `v`),
  skips the weak parts and puts the table on the weak list. After the
  atomic mark, `clear_weak_tables` removes every array slot or node whose
  key or value is an unmarked object; a string met there is marked
  instead (C++ `isobjcleared`). A cleared node keeps its key as a dead
  key, as `t[k] = nil` does. The `s` (shrink) flag is accepted and does
  nothing: shrinking is a memory optimization, invisible to scripts.
- **Dead keys are no longer traced.** Lane 1 traced the key of a node
  whose value is nil; C++ does not (`removeentry`). A dead key can name a
  freed (and reused) slot. That is memory-safe by D2's rule, and only a
  `next(t, k)` with a key that is not in the table, a program error, can
  see the difference (it continues instead of erroring), as in C++ with a
  reused address.
- **The root set** (`gc_roots`): the thread (stack, each frame's closure,
  open upvalues), globals, registry, string metatable, the memory-error
  string, metamethod names, pinned library values, pending host errors,
  the pending namecall name. One behavior changed: inside a native call
  the live stack now ends at the current top, as C++ `traversestack`
  marks `stack .. L->top`. Lane 1 marked up to the highest frame top,
  which kept dead registers of the calling Lua frame alive and made
  gc.luau's weak `kv` table keep an entry held only by a finished loop's
  register (gc.luau:189). Slots above the frames are still cleared.
- **Pacing** follows C++ `luaC_step` and `lua_gc(LUA_GCSTEP)` with goal
  200%, step multiplier 200% and step size 1 KiB, and the work of a step
  is the cost of the objects traversed, as C++ `propagatemark` counts it.
  One difference: C++ `getheaptrigger` starts the next cycle early by an
  allocation rate measured with a clock; here the trigger is the heap goal
  itself, which is C++'s answer when the interval is under a millisecond.
  That keeps collection deterministic. With a memory limit, a safepoint
  also finishes the whole cycle once usage passes half the budget left
  after the last cycle (lane 1's small-budget rule). A step always
  traverses at least one object, so `setstepsize 0` terminates.
- **`collectgarbage`** is not in the vendored base library (the Rive
  fork has none), so the VM's base library still does not register it;
  hosts provide it. The conformance harness's version now matches
  upstream's `lua_collectgarbage`: `step` returns whether a cycle
  finished, `count` is KiB as an integer, the tuning options return the
  previous value. `gcinfo` is unchanged (KiB in use).

### 2.2 One accounting point (`budget.rs`)

`Budget` holds `used`, the external part of it, the limit and a
`blocked` flag. Its module comment is the contract for new builtins:

1. **New heap objects** go through `Heap::new_table`, `clone_table`,
   `intern`, `new_function`, `new_upval`, `new_proto`, which charge the
   object's whole cost before it exists. A new object kind gets an arena,
   a `new_*` that charges and a cost function the sweep credits.
2. **Growing an object**: tables are the only objects that grow, and only
   inside `table.rs`. `Table::array` and `nodes` are private; `set`,
   `resize_array` and `try_clone` take `&mut Budget`, check the predicted
   growth before changing anything and settle the actual size after.
   `array_mut` lends a slice, which cannot grow.
3. **Temporary buffers** a builtin builds and frees before returning:
   `Heap::check_alloc(final_len)` first. They are checked, not charged.
4. **VM buffers that outlive a call** (the value stack): `charge_external`.

`used` equals the cost of every live object plus external buffers,
exactly. `Vm::accounting_drift` (doc-hidden) recomputes the sum from the
arenas. Mutation check: removing the settle in `Table::set` failed four
tests, with drift reported by every table-growing case of the suite.

`setblockallocations(true)` (the conformance harness) sets `blocked`, and
every non-zero charge or check fails. Collection itself never charges
(its gray lists are collector memory, bounded by the heap the budget
already bounds), which `collection_never_allocates` and gc.luau:213 show.

### 2.3 Tests

- **`tests/luau-vm-diff/weak.luau`** (differential, 4 profiles): weak
  keys, values and both; strings kept; no ephemerons; `__mode` removed
  and restored; a non-string mode; `s` alone; a self-referencing weak
  table; clearing during `pairs`; closures as weak keys and values. Same
  output on both VMs.
- **The diff harness** checks the drift after every nuxie run and again
  after a full collection, and reports drift as the case's error, so all
  27 corpus cases are accounting checks too.
- **`budget_suite.rs`**: 30 cases, each opcode or builtin that allocates
  in proportion to its input (NEWTABLE, SETLIST, DUPTABLE, SETTABLE,
  SETTABLEKS, NEWCLOSURE, both CAPTURE kinds, CONCAT, call-stack growth,
  varargs, `{...}`, `tostring`, `error`, `assert`, `rawset`, `unpack`,
  every allocating `string` and `table` function) under a 1 MiB budget.
  Each must end in the memory error (`unpack` in upstream's "too many
  results to unpack", which refuses before the stack grows), with usage
  within the budget, zero drift, and the VM usable afterwards.
  `every_builtin_is_classified` lists every library function at run time
  and fails on one that has no case and is not on the non-allocating
  list.
- **`collector_tests.rs`**: the write barrier for each store kind
  (SETTABLEKS, SETTABLEN, closed upvalue, metatable field, `rawset`,
  `table.insert`, `setfenv`) after partial marks of 1 to 80 steps, with
  the stores made in a callee so only the barrier keeps the objects alive.
  Mutation check: with the table barrier removed, the SETTABLEKS
  assertion fails; with all barriers removed, the closed-upvalue one does.
  Also gc.luau's `dosteps`, a zero step size, a bounded heap under
  automatic steps, and one test per review finding (section 7).

## 3. How to audit a new builtin (for lanes 2, 4, 5 and later)

1. Does it create a heap object? Use a `Heap::new_*` / `Vm::new_table` /
   `intern`. Never build a `Table` with capacity outside `heap.rs`.
2. Does it grow a table? Use `Vm::raw_set`, `Heap::table_set`,
   `Heap::table_resize_array`, `Heap::clone_table`. A new growing table
   operation (lane 2's `table.create` with a size, `table.move` into a
   new range) goes into `table.rs` as a method taking `&mut Budget` that
   checks before it changes anything and settles after.
3. Does it build a temporary buffer whose size the script controls?
   `heap.check_alloc(final_len)` before building it.
4. Does it keep a buffer across calls (a coroutine's stack, a `buffer`
   object)? A buffer object is a new arena kind (rule 1); a thread's stack
   is charged with `charge_external` when it grows.
5. Add a case to `budget_suite.rs` (`CASES`), or list the function in
   `NON_ALLOCATING` with the reason it cannot allocate in proportion to
   its input. `every_builtin_is_classified` fails until one is done.
6. Run `make luau-vm-diff` and the `luau-vm-diff` tests: drift is checked
   on every run.

## 4. Conformance scope

`make luau-conformance-nuxie`: 55 scripts, 14 OK, 41 FAIL, 0 TIMEOUT;
14 required, 41 expected-fail, 0 regressions. Lane 1 had one TIMEOUT
(`closure`); it now fails fast at its coroutine half.

| Script | Before | Now | Next blocker |
| --- | --- | --- | --- |
| `gc` | fail at :98 (`step` was a full cycle) | fail at :226 | `table.clear` (lane 2), then `table.create` :235 (lane 2), `newproxy` :245 (lane 5), `coroutine.create` :322 (lane 4) |
| `closure` | timeout at :30 (weak entry never cleared) | fail at :178 | `coroutine.wrap` (lane 4) |

The rest of `gc.luau` was checked with a scratch copy (not committed):
`table.clear(a)` replaced by a `pairs` loop, `table.create(4000, 1)` by a
loop, the userdata block (:244 to :301) and the coroutine blocks (:313
on) cut. It passes, including blocked allocations at :228 and :238 and
the 200,000-deep chain (`gcinfo` 48,679 KiB before, 8,053 after). The
userdata block asserts that weak entries keyed by `newproxy` userdata go
in two cycles; that needs lane 5's userdata `Value` variant, which the
exhaustive matches in `heap.rs` (`mark_value`, `clear_weak_tables`) force
it to classify.

## 5. Shared files touched, and why

- `vm.rs`: `Tm::Mode` appended (end of the enum and of `TM_NAMES`, so no
  index moves); the safepoint, `full_gc` and roots moved out to `gc.rs`;
  `raw_set`, `new_table` and `ensure_stack` call the budget API; a
  `tm_names()` accessor for the roots.
- `heap.rs`: rewritten (the collector). `Heap::new_table` now takes sizes
  instead of a built `Table`.
- `table.rs`: `array`/`nodes` private with `array()`/`array_mut()`;
  `set`/`resize_array` take `&mut Budget`; `try_clone`, `clear_entries`,
  `capacity_for`; `trace` takes the weak flags and skips dead keys;
  `TableError::Memory`.
- `interp.rs`: DUPTABLE uses `clone_table`, SETLIST and `resize_array`
  go through the heap, `function_mut` for the barrier.
- `libs/base.rs` (`setfenv` uses `function_mut`), `libs/table.rs`
  (`array()`/`array_mut()`), `bytecode/load.rs` (constants built in the
  arena), `host.rs` (GC host API, `GcTune`), `lib.rs` (modules, export).
- `scope-nuxie.txt`: only the `gc` and `closure` lines.

**Merge notes for the other lanes.** Lane 2's in-progress `table.rs` adds
`clear` and `clone_unfrozen` using the fields directly; that still works
inside `table.rs`. Code outside `table.rs` that reads `.array` needs
`.array()`. A new `table.clone` should go through `Heap::clone_table`
(or a heap method built like it) so the copy is charged before it
exists. `Heap::new_table(Table)` is now `new_table(narray, nhash)`.

## 6. What lane 4 must mark

`gc.rs` keeps the running thread's state as a root (`push_thread_roots`:
stack up to the live top, each frame's closure, open upvalues). With
coroutines:

- The running thread and the main thread stay roots, through
  `push_thread_roots` over each one's own stack, frames and open upvalues.
- Every other thread is a heap object behind a new `Value` variant. The
  exhaustive matches in `Heap::mark_value` and `clear_weak_tables` will
  not compile until it is handled: add `GcRef::Thread`, and make its
  traversal do what `push_thread_roots` does for that thread's stack (up
  to its live top: its innermost frame's top if that is a Lua frame,
  else its current top), frames and open upvalues.
- A traversed thread goes back on `gray_again`, never black (C++
  `traversestack` keeps threads gray), because stack stores are not
  barriered. The atomic step then sees every reachable thread's final
  stack.
- An open upvalue names a stack slot of its thread. Open upvalues are
  marked through their thread; closing one goes through `set_upval`,
  which barriers.
- A thread's stack growth is charged with `Heap::charge_external`, and
  credited when the thread is swept (its cost function must include its
  stack, so the drift check stays exact).
- Weak tables with thread keys or values are cleared like any other
  object (the `clear_weak_tables` arm for the new variant).

## 7. Review

`codex review --base origin/main`, three rounds. Every finding was a
real defect and was fixed, each with a collector test that fails
without the fix (mutation-checked).

**Round 1** (fixed in `fix(luau-vm): propagate-again phase, ...`):

1. *A mutator storing into marked tables could stall the cycle.* The
   barrier's re-grayed objects went back on the gray list, so propagation
   never ran dry. Now they go on `gray_again`; the cycle passes through a
   `PropagateAgain` phase once and finishes the rest in the atomic step,
   as C++ does. Test: `sustained_stores_into_marked_tables_do_not_stall_the_cycle`.
2. *Weak tables were not retraversed in the atomic step.* A table
   traversed as weak whose `__mode` was removed before the atomic step
   lost values that were strong by then. The atomic step now re-grays
   every table on the weak list and reads the mode again. Test:
   `a_mode_removed_mid_cycle_keeps_the_values`.
3. *A stopped collector's threshold was read as a step credit.* The
   unsigned conversion made `collectgarbage("step")` while stopped behave
   unlike upstream. Now a signed conversion, so the stopped threshold is
   a debt as in `lua_gc(LUA_GCSTEP)`. Test:
   `a_step_while_stopped_restarts_like_upstream`.

**Round 2** (fixed in `fix(luau-vm): memory pressure before ...`):

1. *Memory pressure lost to a due incremental step.* The safepoint
   checked pressure only when no step was due, so a small budget could
   raise the memory error with reclaimable garbage in place. Pressure is
   now checked first. Test: `memory_pressure_wins_over_an_incremental_step`
   (40,000-byte budget).
2. *Dead registers kept stale ids.* `gc_roots` cleared the stack only
   above every frame's top but marked only to the live top. Registers in
   between held ids the sweep freed, and a later automatic step in Lua
   code (which marks to the frame's top) rooted whatever reused them, so
   weak values were retained. The stack is now cleared from the live top.
   Test: `dead_stack_slots_do_not_root_reused_ids` (34 of 40 weak values
   retained without the fix).

**Round 3**: no findings.

## 8. What was not verified

- **`gc.luau` as a whole**, until lanes 2, 4 and 5 land. The collector
  parts were run (section 4); the userdata and coroutine blocks were not.
- **Step counts versus C++.** `dosteps` passes its three assertions, but
  the number of steps per cycle depends on this VM's cost model, not
  C++'s `sizeof`s. No test compares counts with the vendored VM.
- **Temporary buffers are checked by convention.** A builtin that builds
  a Rust buffer without `check_alloc` would pass the drift check (the
  buffer is not a heap object). The suite's single-call cases hit each
  builtin's check, but a transient over-limit allocation that is then
  refused by the intern charge is not observable from a test.
- **Memory accounting is still an estimate** of real sizes (per-object
  constants, `Vec` capacity not counted beyond length). It is exact
  only against its own model.
- **Speed** was not measured. Collection now starts at the heap goal of
  a small heap (C++'s policy) instead of lane 1's 1 MiB minimum, so
  small heaps collect more often. Not a gate; a bench run belongs with
  the next speed pass.
- **`make lint-gate`** was not run in full; clippy for the crate was.

## 9. Proposed plan lines

> - 2026-11-11: Lane 3 written on `claude/luau-vm-lane3-collector-3cc6d0`
>   (evidence `docs/evidence/luau-safe-vm-phase2-lane3-2026-11-11.md`).
>   Incremental collector (tri-color, barrier in the heap accessors,
>   atomic step with the sweep), weak tables as Luau has them, and
>   `budget.rs` as the one accounting point with an exact count that the
>   diff harness checks after every run. The memory-budget suite covers
>   every allocating opcode and builtin and fails on an unclassified new
>   builtin. `closure` passes its weak-table half; `gc` passes through
>   :225 and becomes `required` once `table.clear`/`create` (lane 2),
>   `newproxy` (lane 5) and coroutines (lane 4) land.
> - 2026-11-11: Every lane adding a library function classifies it in
>   `tools/luau-vm-diff/src/budget_suite.rs` (a case, or non-allocating).
>   Lane 4 adds threads as a `GcRef` kind kept on `gray_again`, per the
>   lane 3 evidence, section 6.
