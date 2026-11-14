# Luau client path, lane E2a: recursion and hangs (2026-11-14)

Design: `docs/luaur-client-path-safe.md` 12.2, "E2a". Inputs: the E1
inventory (`docs/evidence/luaur-client-path-lane-e1-inventory-2026-11-14.md`)
section 3 (budget reset, `buffer_ext`), 4.1 and 4.2. Branch
`claude/luaur-lane-e2a-recursion-d9dac4`, based on `9809e81bad` (`levi/luaur-client-safe`; same tree as
`a2e2271537`, which the base branch was rewritten from). One commit per fix:

| Commit | Fix |
|---|---|
| `aa91c7ee60` | 1. a callback error never wraps another |
| `3e84aeddb9` | 2. iterative dependency order for module registration |
| `6c9d783581` | 3. a nested host entry keeps the running entry's budget |
| `4eac4e0907` | 4. `buffer.stridedcopy` / `buffer.convert` do bounded work |

The four regression tests live in
`crates/nuxie-scripting/src/vm/recursion_and_hang_tests.rs`; the property
tests sit beside the code they check.

## Before-state on the old code

The seven scenario tests were written first and run on `a2e2271537` (the four
fixes absent; the error test then counted layers through the old
`cause: Arc<Error>`). Load average 47 / 30 / 23.

| Test | Result on the old code |
|---|---|
| `relaying_a_structured_error_keeps_one_layer` | FAILED: `left: 3002, right: 1` layers after 3,001 rounds |
| `an_acyclic_require_chain_registers_without_a_cycle_error` | FAILED: `script 0: Some("runtime error: cyclic script module dependency")` |
| `a_failed_dependency_keeps_each_scripts_own_error` | FAILED: `runtime error: cyclic script module dependency` |
| `a_real_require_cycle_keeps_the_cycle_error` | passed (a real cycle; the text is kept) |
| `reentering_the_vm_in_a_loop_reaches_the_safepoint_cap` | FAILED: `re-entry never reached the safepoint cap after 200001 entries` |
| `a_zero_stride_strided_copy_is_bounded` | FAILED: `did not finish within 10 s` |
| `a_zero_stride_convert_is_bounded` | FAILED: `did not finish within 10 s` |

All pass on the branch.

## 1. `CallbackError` depth one (`luau/facade/error.rs`)

`Error::CallbackError { traceback, cause }` now has `cause: CallbackCause`,
a new public enum of the three structured errors (`CallbackDestructed`,
`UserDataDestructed`, `RecursiveMutCallback`). It is not an `Error`, so a
callback error cannot hold another: depth one holds in the types, not by
construction. The error object (`WrappedError`) carries the traceback and the
cause; raising a `CallbackError` puts its own traceback and cause in the
object, and recovering the object rebuilds the same `CallbackError`. So a
relayed callback error keeps its root cause and its traceback instead of
gaining a layer.

What no longer recurses: `Display` (the three causes print through
`CallbackCause`'s `Display`, the same strings), `is_structured` (now folded
into `structured()`, a single match; the method had no other caller and was
removed), the derived `Drop` (`CallbackCause` is `Copy`), `log_error` (the
`CallbackError` arm fell to the `_` arm, which prints `to_string()`, the same
bytes the old recursion reached) and `observe_vm_error` (a cause is never a
`MemoryError`; the arm is gone).

Texts: unchanged. Shown by the existing facade and integration tests, and by
the regression test, which compares the text after one round and after 3,001
rounds and pins it to
`a destructed callback or destructed userdata method was called`.

Host-visible change: a host that matched the nested chain now sees one
layer. Two tests matched it: the facade test
`scope_destruction_errors_stay_structured_across_boundaries` asserted "one
layer per boundary" (the old binding crate's behavior) and now asserts one
layer with the same text; `tests/upstream_module_hooks.rs` matches the cause
as `CallbackCause`. No product code matched past the first layer.

**The starter** (a scoped `xpcall` handler reached through `debug.info` and
called after its scope ended, `resource_limits.rs:224-245`): with depth one
it can only produce one error object whose host reading is
`CallbackError(CallbackDestructed)`. That is no longer a crash, but it lets a
script forge a "destructed callback" error that a host would attribute to
its own scopes. Recommendation (not changed here, the file is outside this
lane's fix list): close it by not handing the script a host scope function.
Build the guarded handler in Luau from one non-scoped host guard, e.g. a
precompiled `function(handler, guard) return function(e) guard(e) return
handler(e) end end`, where `guard` (created once at install) does
`record_protected_memory_error` and `reject_if_tripped`. The handler and the
guard are then upvalues the collector sees, so nothing leaks without a scope,
and what `debug.info` returns stays callable after `xpcall` returns.

## 2. Registration order (`vm/native_registration.rs`)

`sort` (recursive, nested `fn` with eight arguments) is replaced by
`dependency_order(roots, dependencies_of)`: an explicit stack of frames
(script, root, remaining dependencies), marks in a `HashMap`, each script's
dependency table read once per walk. Linear in scripts plus dependencies. A
200,000-script chain orders without recursion (test
`a_long_chain_needs_no_recursion`).

**The order rule of the old code, for every acyclic input it accepted:**
roots are the pending scripts, taken from the highest index down; a script is
placed after a depth-first walk of its dependencies in their Lua-table
`pairs` order (only dependencies that name a script in the file); a placed or
already-placed script is skipped; and each time a script is placed, the next
root is started on top of the walk in progress (the recursive version's tail
call after every placement). The new walk does exactly this
(`Interleave::OnPlacement`), so its order is the old order by construction.

The false cycle came from that interleaving: a later root that reaches a
script still open in an earlier root's walk (including the root itself being
open) was reported as a cycle. A pending chain of three in file order, each
requiring the one before, triggers it: E1's minimal input. On that conflict the new code walks the roots
again one after the other (`Interleave::Never`), where an open script can only
be an ancestor in the same walk, so it reports a cycle only when one exists,
with the old text `cyclic script module dependency`.

Tests: the E1 minimal input and its smaller form, end to end through
`register` (all four register; each failed script keeps its own error), a
real two-script cycle (same text), and a seeded property test
(`matches_the_old_order_and_a_reference_sort_on_random_graphs`, 20,000 random
graphs of 1 to 12 scripts, a quarter of them allowed to cycle) that checks:
the order equals the old recursive `sort` (ported into the test) wherever
that sort succeeded; the order is topological and contains exactly the
scripts reachable from the roots; a cycle is reported iff a colouring DFS
finds one reachable. Of the 20,000: 17,152
ordered, 2,848 real cycles; 3,453 that the old sort rejected with a false
cycle are now ordered.

On the fixture corpus: `scripted-golden-compare` and the `nuxie` tests give
the same results as on main (see Checks). That shows the order is unchanged
on the corpus only to the extent the corpus reaches the retry pass; the
argument that it is unchanged for every accepted input is the construction
above plus the property test.

`attempt` now takes the script's result slot instead of indexing `results`,
and the loops zip `scripts` with `results`.

## 3. Nested host entries (`vm.rs`)

`LuaScriptInstance::reset_execution_budget` and
`ScriptVm::reset_execution_budget` now start a fresh budget only for the
outermost entry. A nested entry still polls the audio processors and returns;
it does not clear the trip flag (`begin_cycle`), the safepoint count, the
per-callback interrupt budget, or `budget_exceeded`.

Shape: no depth counter. The design asked for "a nesting depth"; a counter
held by a guard across each entry's Rust scope gets the wrong answer where
one entry function resets twice in sequence (`instantiate_script` resets
before the generator and again before `init`) or calls another resetting
entry after its Luau returned (`set_input` -> `set_input_core` and the other
`*_core` pairs): the second reset would look nested. What "nested" means is
"a script of this VM is running up the stack", and the VM already answers
that: `script_running(lua)` is `vm.inspect_stack(0).is_some()` through the
existing public `Lua::with_vm` (no change to the VM's embedding API). At the
host's top level the running thread has no frame; inside a host callback,
level 0 is the callback. There is no state to keep in step, so nothing to
restore when a nested entry errors or unwinds, and nothing can panic. A VM
that cannot be reached counts as running (no reset).

Test: `reentering_the_vm_in_a_loop_reaches_the_safepoint_cap`. The smallest
real re-entry in the test support is a host function, installed in the VM,
that calls `ScriptVm::call_global` on the same VM (the same path as
`art:advance()` reaching a scripted drawable of the VM: a host call from a
script that enters the VM through a resetting entry). The script loops on it
forever; it stops with `script cycle exceeds 100000 script safepoints` after
fewer than 100,000 re-entries (before: past 200,001), and the next outermost
`call_global` runs normally. The test also asserts `script_running` is false
at the top level. A real nested-artboard fixture was not built: none with a
same-VM scripted drawable under a scripted `art:advance()` was at hand.

**Host commands are a different defect, not fixed.** `HostCommandHost::
begin_cycle` (`host_commands.rs:232`) clears the queue and `drain_effects`
(`:272`) drains it whatever is active. Their only product caller is the
embedder's effect transaction (`nuxie/src/script_import.rs:850`,
`ScriptEffectTransaction::begin`), opened once per page-player step
(`nuxie-runtime/src/page_player.rs:397`). No script binding drives the page
player, so a script cannot open a nested transaction; it takes an embedder
that steps the page player from inside a script callback. It is a
transaction-nesting defect in the embedder seam, not the budget reset. Fix
shape for its owner: `begin_cycle` while `active` returns a checkpoint at
the current queue length instead of clearing, and only the outermost
transaction drains.

Also not covered: `gpu_canvas.rs:2336-2473` keeps its own per-call interrupt
budget (`MAX_LUAU_INTERRUPTS_PER_CALL`) and refills it per call; a nested GPU
canvas call would refill it the same way. Outside this lane's files (GPU
lane).

## 4. `buffer.stridedcopy` and `buffer.convert` (`vm/buffer_ext.rs`)

Reference: the pinned `src/lua/lua_buffer_ext.cpp` at
`$RIVE_RUNTIME_DIR` (`cd5351c3`). Its checks, their order and their texts are
kept.

**Zero strides, upstream.** `stridedcopy` requires `stride >= elementSize >=
0`, so a zero stride means `elementSize == 0`: C++ then runs `count` (up to
`i32::MAX`) empty `memcpy`s, uninterruptible but cheap. `convert` allows
`srcStride == dstStride == 0` (the `>= span` checks only apply to a positive
stride), and the bounds check then only covers one element: C++ runs `count *
components` scalar conversions of the same scalars, up to `2^31 * components`,
with no interrupt check (seconds to minutes).

**Zero strides, here before.** The same loops, but each element went through
the VM (`write_bytes`), and with source and destination the same buffer every
element first copied the whole buffer (`to_vec`): with a 4 KiB buffer and
`count = i32::MAX` neither call finished in 10 s.

**Here now.** Each call reads the source once. Between two buffers the
destination is written element by element in place (no copy of it); in one
buffer a local copy is worked on and the touched range written back once.
No copy is retained after the call.

- `stridedcopy`: after the checks, `elementSize == 0` returns (every copy is
  empty). Otherwise both strides are at least 1, so the bounds check bounds
  `count` by the buffer lengths. Copies in one buffer read what earlier
  copies wrote, element by element, as the old Rust did.
- `convert`: a nonzero stride on either side bounds `count` by that buffer.
  With both strides zero, every element converts the same scalars. Between
  two buffers a repeat changes nothing, so one conversion is done; in one buffer a repeat can feed the
  next, and it does not always settle (a Python model of the scalar loop on
  200,000 random overlapping cases: 5,878 cycles of period 2 to 91 and more,
  some not settled in 400 repeats). Repeats stop when the destination bytes
  equal a checkpoint taken at power-of-two intervals (Brent); the bytes then
  repeat with that period, so the remaining count is taken modulo it and the
  result is the one `count` repeats give. A repeat that has not come back
  within `2^22` scalar conversions (and never fewer than two repeats, so a
  single very wide element still converts) is an error, `buffer.convert with zero
  strides did not settle within 4194304 conversions`, instead of a hang (the
  only new text, and only where the old code hung).
- The packed same-format fast path stays one block copy (memmove), as the
  old Rust did; C++'s `memcpy` of an overlapping run is undefined. C++'s
  packed f32/f16 SIMD paths are not reproduced, as before.

Tests: the two hang tests; `convert_matches_the_element_loop` (6,000 seeded
cases, all formats, 1 to 4 components, apart and aliased, strides zero or
not, repeat counts up to 3,000) and `strided_copy_matches_the_element_loop`
(4,000 cases, overlapping elements in one buffer) against a direct model of
the element loop; `a_cycling_repeat_resolves_modulo_its_period` (a cycling
in-place conversion at `count = i32::MAX` equals the loop at the same phase);
the 77 existing tests in `tests/buffer_extensions.rs`.

## Panic lints (forced, per file)

`tools/ratchets/panic-lints.sh --baseline <scratch copy> crates/nuxie-scripting`,
diagnostics counted per primary-span file.

| File | Before | After |
|---|---:|---:|
| `src/luau/facade/error.rs` | 0 | 0 |
| `src/vm/native_registration.rs` | 9 (indexing_slicing 9) | 0 |
| `src/vm/buffer_ext.rs` | 36 (arithmetic 21, expect 6, indexing 9) | 0 |
| `src/vm.rs` | 37 (arithmetic 22, expect 2, indexing 13) | 37 |
| crate total | 476 | 431 |

`vm.rs`: none of its 37 sites is in the budget reset this lane owns
(`:1317` `LuaScriptInstance::table`'s `expect`, `:2278` the interrupt
budget's `remaining - 1`, the rest elsewhere); left to their owners. Shapes
rather than patches: `native_registration` zips `scripts` with `results` and
passes the result slot; `buffer_ext` reads and writes elements through
checked ranges (`first_chunk`, `get(at..)`), walks positions with
`iter::successors` (no index arithmetic, and a zero stride is a constant
position instead of a `step_by(0)` panic), and `float_to_half` uses explicit
wrapping operations (checked against the old body on every 251st f32 bit
pattern and the edge values). Both totals were measured on the same base,
so the crate's drop of 45 is these files'.

## Checks

Run through `make` / `tools/bazel/runtime.py` with
`RIVE_RUNTIME_DIR` at the `cd5351c3` pin. Load averages (1/5/15 min) at
start of each.

| Check | Result | Load |
|---|---|---|
| `nuxie-scripting` tests | pass (exit 0) | 22 / 27 / 26 |
| `nuxie` tests | 41 of 42 pass; the one red is `stateful_component_image_bind`, red on main too | 21 / 29 / 28, rerun 25 / 20 / 23 |
| `luau-vm-diff` | 409 runs, 0 disagreements | 21 / 26 / 27 |
| `luau-conformance` | 37 OK / 18 FAIL, required-not-passing 0: PASS (as main) | 25 / 24 / 27 |
| `scripted-golden-compare` | entries 364, exact 348, diverges 16 (as main) | 19 / 23 / 26 |
| `luau-vm-ratchets` | PASS (unsafe 2,219 sites unchanged; panic lints down) | 12 / 20 / 24 |
| `nuxie-scripting` tests after the review fix | pass (exit 0) | 16 |
| panic lints on the final tip | 431, per-file table above unchanged | 16 / 15 / 18 |

The battery ran on the tree before the review fix (which touched only
`buffer_ext.rs`); the scripting tests and lints were rerun on the final tip.
Disk: lowest free 92 GB.

`codex review --base levi/luaur-client-safe`: round 1 found two P2s in
`buffer_ext.rs`, both fixed in the buffer commit: (a) the repeat budget
`2^22 / components` was zero for an element wider than `2^22` components, so
`count = 1` errored (now at least two repeats; test added); (b) aliasing
copied the whole destination per call even between two buffers (a 15 ms case
took 645 ms; now the destination is written in place when apart). Round 2:
no findings.

## Not verified

- No fixture exercises a same-VM nested artboard (`art:advance()` reaching a
  scripted drawable of the calling VM); fix 3 is tested through
  `call_global` from a host callback.
- The registration order on the corpus is checked only through unchanged
  golden and `nuxie` results; how many corpus files reach the retry pass,
  and whether any reached the fallback walk, was not counted.
- The host-command nesting (fix 3's sibling) was read, not reproduced.
- The `2^22` conversion bound for cycling in-place conversions is a chosen
  work cap, not derived; cases whose cycle is longer error where C++ would
  (slowly) finish.

## Where the design was wrong or incomplete

- 12.2 (3) "a nesting depth": a Rust-scoped depth miscounts entries that
  reset twice in sequence or call another resetting entry; the VM's call
  stack is the right signal and needs no state.
- 12.2 (4) "checked against the interrupt budget": a host callback has no
  access to the interrupt budget through the facade, and none is needed once
  the work is bounded by the buffers; the one unbounded case (both strides
  zero, in place) is resolved exactly by cycle detection with a work cap.
- 12.2 (1) says depth one makes `Display`, `is_structured` and `Drop`
  non-recursive; true, and it changes what a host matching the chain sees
  (one layer, not one per boundary), which the old binding crate's test
  pinned. No product code depended on it.
- E1 4.2 framed the false cycle as rare (9,560 of 76,871 random graphs); on
  real files a pending chain of three scripts in file order (each requiring
  the one before) is enough, so the fix can change registration results for
  files that used to fail.
