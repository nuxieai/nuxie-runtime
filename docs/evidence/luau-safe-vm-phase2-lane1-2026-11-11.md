# Safe Luau VM, Phase 2 lane 1: the core (2026-11-11)

Lane 1 of Phase 2 in `docs/luau-safe-vm-plan.md` ("How Phase 2 runs"). It adds
the crate, the core VM, the differential harness, and the second VM in the
conformance and bench tools. It was written on
`claude/luau-vm-lane1-core-0b14a3` and is proposed as branch
`levi/luau-vm-lane1`.

**Outcome.**

- **`crates/nuxie-luau-vm`**:
  - `#![forbid(unsafe_code)]`;
  - the seven forced panic lints are denied at the crate root, with no
    `allow` anywhere;
  - it builds through Bazel and is clippy-clean under the repo's
    `-D warnings`.
  - Every failure a script can cause is a returned `Error`.
  - Every limit is a field of `Limits`.
- **Differential corpus: 26 cases × 4 compiler profiles (104 runs), 0 disagreements**
  with the vendored VM, on bytecode versions 7 and 11. The corpus reaches
  every opcode the loader admits (`make luau-vm-diff`).
- **Conformance on the new VM: 14 of 55 upstream scripts pass** and are
  `required` in `tests/luau-conformance/scope-nuxie.txt`
  (`make luau-conformance-nuxie`). They include the four lane-1 targets
  that pass: `assert`, `ifelseexpr`, `constructs` and `locals`. `literals`
  fails for the harness reason it also fails on luaur (Latin-1 source
  transcoded to UTF-8).
- **The new crate is at 0 in both ratchets** (`unsafe` and the forced
  panic lints).
- **Speed** is recorded in section 5 together with the host load. It is
  not a gate.

## 1. Commits

| Commit | What |
| --- | --- |
| `feat(luau-vm): add the safe Luau VM core as crates/nuxie-luau-vm` | The crate and its Bazel registration. The message proposes the name and location. |
| `fix(luau-vm): match the vendored VM on unpack, string errors; expose opcodes` | Four differences the corpus found, plus `format_number` and `Vm::opcodes` for the harness. |
| `test(luau-vm): differential harness and seed corpus (make luau-vm-diff)` | `tools/luau-vm-diff` and `tests/luau-vm-diff/`. |
| `feat(luau-vm): run conformance and bench on nuxie-luau-vm; ratchet it at zero` | `--vm nuxie` in `tools/luau-conformance` and `tools/luau-bench`, the scope file, the make targets and the ratchet baselines. |
| `fix(luau-vm): address review findings in limits, hex literals, patterns and modf` | The `codex review` findings (section 6 lists them) and `review-fixes.luau`. |
| `fix(luau-vm): store NEWCLOSURE's closure before its captures; diff re-entry` | A self-capturing local function captured `nil`. The diff tool runs each VM on its own thread and charges nuxie the reference's extra C-call level. Adds `reentry.luau`. |
| `feat(luau-vm): table.sort` | Upstream `tsort`, because the bench support script calls it. Adds `sort.luau`. |
| `fix(luau-vm): check limits before allocating, not after` | Second review: table growth, gsub output, refused charges, the stack limit below 64 slots. |
| `fix(luau-vm): run modf's fast body where upstream does; round hex once` | Second review: the modf fastcall/library split; subnormal hex rounding. |
| `fix(luau-vm): bound result buffers and loader buffers; collect small budgets` | Third review. |
| `fix(luau-vm): FASTCALL2 targets, clamp NaN bounds, NaN vector keys; exact diff outcomes` | Third review; the diff tool now compares typed, lossless outcomes. |
| `fix(luau-vm): concat budget, host-boundary safepoint, __tostring arity, setfenv "0", math.deg` | Fourth review. |
| `fix(luau-vm): unwind host tostring errors; collect after rejected loads; DUPTABLE budget; exact %q length` | Fifth review. |
| `fix(luau-vm): refuse SETLIST into readonly tables; document re-entry stack needs` | Sixth review. |
| `fix(luau-vm): even-digit ties in num2str; string missing-method error; host-boundary collection` | Seventh review. |
| `fix(luau-vm): budget-check string builders; return empty repeats at once` | Eighth (last) review. |
| `docs(luau-vm): Phase 2 lane 1 evidence` | This document and the scope-file reason for `sort.luau`. |

**Final gates**, run once on `610c527a5d`, all exiting 0:

- `make luau-vm-diff`: 104 runs, 0 disagreements, every admitted opcode
  reached.
- `make luau-conformance-nuxie`: 14 OK, 40 FAIL, 1 TIMEOUT; 14 required
  pass and no expected-fail passes.
- `make luau-conformance` (luaur, unchanged): 28 OK; 28 required pass.
- `make luau-vm-ratchets`: `crates/nuxie-luau-vm` is at 0 `unsafe` and 0
  for each forced panic lint.
- `generate.py --check` is clean.

## 2. Module map

`crates/nuxie-luau-vm/src` is about 7,600 lines. It has no dependencies.

| Module | Contents | Plan decision |
| --- | --- | --- |
| `value.rs` | `Value`: 16 bytes, a tag plus payload (nil, boolean, number, vector, string, table, function, light userdata). Typed `u32` ids per arena (`StrId`, `TableId`, `FnId`, `UpvalId`, `ProtoId`). | D1 |
| `heap.rs` | Typed arenas with free lists and mark bits. String interning (with the `StrId(0)` "not enough memory" reservation). Allocation accounting against `Limits::max_memory_bytes`. A stop-the-world mark-and-sweep `collect(roots)`. | D2 |
| `table.rs` | Tables. The array part follows C++ `luaH_resize`/`rehash`/`computesizes` exactly. The hash part is a chained node array with C++'s main-position and free-pointer scheme. `len` is `luaH_getn`. `next_from` is the iteration cursor. Read-only and `safeenv` flags. | D4 |
| `bytecode/mod.rs` | Opcode numbers, instruction field decoding, `Proto`, line info. | |
| `bytecode/load.rs` | The loader for versions 3..=11 and type versions 1..=3. Rejects 12, 13 and 100, and the userdata, class and integer constants. Resolves imports at load. | D7 |
| `bytecode/verify.rs` | Per-prototype verification on load. Checks register, constant and upvalue indices; jump targets inside the code and never into an aux word; aux words present; `CAPTURE` sequences matching the child's upvalue count; `DUPCLOSURE` not capturing by reference; `FASTCALL` targets; `SETLIST`/`CALL` ranges. Rejects `BREAK`, `NATIVECALL` and the udata and class opcodes. | D7 |
| `vm.rs` | VM state: the stack, frames, open upvalues (sorted by slot), globals, registry, string metatable, metamethod names. Also the call protocol (`call_value`, `enter_lua`, `call_native`, `call_host`, `move_results`), error construction (`runtime_error`, `lib_error`, the luaL_where prefix, type and index errors), unwinding, call-depth and re-entry limits, and the GC safepoint with its root set. | D3, D8 |
| `interp.rs` | The instruction loop. Every admitted opcode, with Lua-to-Lua calls inline (no host recursion). FASTCALL\* is a no-op that falls through to the CALL; the builtins are ordinary functions. Where the vendored fastcall body differs from its library function, the library function follows the fastcall body, since that is what runs under `safeenv` (today only `math.modf`'s signed-zero fraction). | |
| `meta.rs` | Operations with metamethod fallbacks: arithmetic (numbers, vectors, string coercion), comparison, equality, length, concatenation, `get_table`/`set_table` chains, and `tostring` (`luaL_tolstring`). | |
| `num.rs` | `luai_num2str` (shortest round-trip digits in Luau's layout) and `luaO_str2d` (strtod semantics, hex). | |
| `libs/` | `base` (everything in `luaB` except `newproxy`), `string` (len, sub, upper, lower, rep, reverse, byte, char, find, match, gsub, format; a full Lua pattern matcher), `table` (pack, unpack, concat, insert, remove, sort; the global `unpack`), `math` (the `f64` functions; missing `random`, `noise`, `frexp`, `ldexp`, `map`, `lerp` and the newer predicates). | |
| `host.rs` | The embedding API: `Vm::new`/`with_limits`, `register`, `set_global`, `sandbox`, `run`, `collect`, `memory_bytes`. Host functions get a `HostCtx` (args, `tostring_value`, push, `load`). `HostValue` is the copy-out value type. | |
| `limits.rs` | `Limits`: call depth, C-call depth, stack slots, metatable chain, bytecode size, string size, memory. | D3 |

Error text matches the vendored VM, including the Rive `luaL_where`
formatting (`=chunk:line:`, and `:: ` with no Lua frame). Two vendored quirks
are reproduced on purpose because the oracle has them: `string.find` with
`%s` does not count `\v` as space, and a pattern ending in `%` reports
`(ends with '%%')`.

## 3. Seams for lanes 2 to 7

Each seam is described in the code where it lives.

- **Lane 2, tables and strings.**
  - `table.rs`: `table.create`/`clear`/`move`/`find`/`clone`/`getn`/`foreach`/`freeze`/`isfrozen` go into `libs/table.rs`, as functions over `Table`.
  - The read-only flag already exists.
  - Strings are interned byte vectors in `heap.rs`. A short-string/long-string split, if wanted, is local to `Heap::intern` and `str_bytes`.
  - `gmatch` needs a builtin with mutable state; `NativeFn::BuiltinWith` carries immutable upvalues today.
- **Lane 3, collector.**
  - `Heap::collect` is stop-the-world. `Vm::safepoint` is the only caller, at allocation points where every live value is rooted.
  - Incremental marking fits behind the same `safepoint` call, with a gray list and a write barrier at `raw_set`/`upval_set`/`set_table`.
  - Weak tables need `__mode` (removed from `Tm` in lane 1) and a clearing pass after mark.
  - `collectgarbage("step")` currently runs a full cycle.
  - Memory accounting is an estimate: arena slots, string bytes, table array and node capacity, prototype bytes and stack growth. It is not C++'s `GCCOUNT`.
- **Lane 4, coroutines and errors.**
  - `vm.rs` holds one thread's state (stack, frames, open upvalues, `c_calls`). A coroutine is that state moved into a `Thread` arena object, with the stack slots addressed through the thread id.
  - `Vm::interrupt` is the interrupt hook, a no-op today.
  - `pcall` re-enters the interpreter (`call_value`) and so is bounded by `max_c_calls` (200); upstream allows 10,000 nested pcalls (pcall.luau:66). A continuation-style pcall in the loop is lane 4's.
  - The xpcall handler runs at the entry `c_calls` level, not at C++'s deeper one.
- **Lane 5, libraries.**
  - `libs/mod.rs`: each library is a module with an `open` function and builtins of type `BuiltinFn`.
  - Missing: `bit32`, `buffer`, `utf8`, `vector`, `os`, `debug`, `coroutine` (with lane 4), `newproxy`, `string.pack`/`gmatch`, `math.random`/`noise`/`frexp`/`ldexp`.
  - `Proto::linedefined` and local/upvalue names are loaded for `debug`.
  - The integer type is rejected at load; it is one more `Value` variant and arena.
- **Lane 6, embedding.**
  - `host.rs` is the whole API. `HostValue::Object` is a placeholder where userdata and persistent references go.
  - `Vm::opcodes` is doc-hidden tooling, not API.
- **Lane 7, Rive bytecode extensions and raw-hatch retirement.**
  - The admission set is `VERSION_MIN..=VERSION_MAX` in `bytecode/mod.rs`, and the rejected opcodes are listed in `verify.rs`.
  - Rive's extensions are new opcodes or constant kinds admitted there and executed in `interp.rs`.

## 4. Conformance scope

`make luau-conformance-nuxie`: 55 scripts, 14 OK, 40 FAIL, 1 TIMEOUT; 14
required, 41 expected-fail, 0 regressions. The column on the right is the
lane the plan assigns, or the one this run suggests (section 7).

| Script | Result | First blocker | Lane |
| --- | --- | --- | --- |
| apicalls, assert, attrib, constructs, exceptions, explicit_type_instantiations, ifelseexpr, interrupt, iter_fenv, jit_inliner, locals, safeenv, strconv, stringinterp | **pass (required)** | | 1 |
| literals | fail | Latin-1 source transcoded to UTF-8 (also fails on luaur) | harness |
| pm | fail | same transcoding (also fails on luaur) | harness |
| basic, tmerror | fail | Rive luaL_where text (reproduced on purpose) | non-goal |
| calls, vararg | fail | `table.getn` | 2 |
| clear, move, sort, tables, types | fail | `table.clear`/`move`/`clone`/`foreach`/`find` (`sort` needs only `table.clone` now) | 2 |
| closure | timeout | weak tables (closure.luau:30 loops until a weak entry clears) | 3, then 4 |
| gc | fail | `collectgarbage("step")` is a full cycle | 3 |
| coroutine, cyield, ndebug_upvalues | fail | no `coroutine` library | 4 |
| pcall | fail | nested pcall depth 200 vs 10,000 | 4 |
| bitwise, buffers, datetime, debug, errors, math, tpack, utf8, vector, vector_library, native_integer_spills | fail | missing library or integer type | 5 |
| events, iter, strings | fail | `newproxy` | 5 |
| coverage, debugger, native_types, native_userdata, udata_direct, userdata | fail | host callbacks or userdata | 6 |
| classes, integers, integers_regspill, native | fail | outside the Rive profile (permanent non-goals in `scope.txt`) | none |

The brief named five lane-1 targets.

- **Pass:** `assert`, `ifelseexpr`, `constructs` and `locals`.
- **Blocked:** `literals`, by the harness, not the VM. Its embedded program
  `prog` (literals.luau:95) is Latin-1, and both harnesses compile a `&str`.
  The luaur harness has the same `expected-fail`. Fixing it means a
  byte-source compile entry. The vendored compiler takes `&String`, so that
  is a compiler-side change.
- **Also pass, outside the five:** nine scripts the plan assigns to other
  lanes or to no lane: `apicalls`, `attrib`, `exceptions`,
  `explicit_type_instantiations`, `interrupt`, `iter_fenv`, `jit_inliner`,
  `safeenv`, `strconv` and `stringinterp`. Of these, `strconv` and
  `stringinterp` are lane 2's in the plan.

## 5. Numbers

`luau-bench` (the upstream suite merged by `tests/luau-bench/merge.py`), release
build with fat LTO, 3 passes, each bench the median of its 20 kept runs.
Recorded, not gated. Host: aarch64 macOS, 18 CPUs.

| VM | Commit | Benches run | Geomean | Pass times | 1-min load (start → end) |
| --- | --- | --- | --- | --- | --- |
| luaur (vendored) | `b9d5963e72` | 87 of 87 | 7.399 ms | 50.2 s, 49.5 s, 49.9 s | 8.48 → 7.68 → 5.91 → 7.12 |
| nuxie-luau-vm | `32bbcc9781` | 59 of 84 | 31.991 ms | 1053 s, 927 s, 1012 s | 14.86 → 9.88 → 7.70 → 12.61 |

- **On the 59 benches both VMs ran:** the geomean is 7.196 ms on luaur and
  31.991 ms on nuxie, a ratio of **4.45x**. The median per-bench ratio is
  3.61x. The range runs from 1.05x (`TableFind_loop_ipairs`) to 113x (the
  three `StringInterp` number-formatting benches).
- **The 25 failures are all missing libraries:** `table.create`, `clone`,
  `move` and `find`; `math.random`; `vector`; `bit32`; and `coroutine`.
  `chess` is the same gap: it polyfills `table.create` into the
  read-only, sandboxed `table`. None is a VM fault.
- **Host load differs between the runs, so the ratio is indicative only.**
  The nuxie run shared the host with this lane's own Bazel builds and
  `codex review` runs, at a 1-minute load of 7.7 to 14.9 (luaur: 5.9 to
  8.5). The nuxie run also measured `32bbcc9781`. The later commits
  change limits, modf, hex parsing and error paths, not the hot loop.
- **Speed is not a lane 1 goal.** Per the plan it is a budget, not a
  gate. The `StringInterp` outliers point at `num2str` (Rust's `{:e}`
  formatting, then a string re-layout into Luau's form). A Schubfach port
  that writes Luau's layout directly is the obvious first target when
  speed work starts. This was not profiled. Since the bench run, `num2str` also does a
  neighbour round-trip check for Schubfach's tie rule (review round 7),
  which makes it slower still.

## 6. What was not verified

- **Re-entry can exhaust a small thread stack, which aborts the process.**
  This is measured, and the risk is open. Metamethods, `pcall` and host
  callbacks re-enter the interpreter through Rust recursion, bounded only
  by `Limits::max_c_calls`. `reentry.luau` (`__index`, `pcall`, `__call`,
  `__tostring` and `__lt` to the limit) was run under
  `luau-conformance exec-one --stack-mib N`:
  - **Release:** the default 200 levels fit in 1 MiB.
  - **Fastbuild:** about 25 KiB per level. 4 MiB aborts and 6 MiB passes.

  The `Limits` doc states this contract: an embedder sizes the thread or
  lowers the limit. Only the paths in `reentry.luau` were measured.
  `table.sort` comparators and `gsub` callbacks re-enter the same way and
  were not measured. The structural fix, re-entry without host recursion,
  belongs with lane 4's continuation-style `pcall` (section 3).
- **Hash-part iteration order** is expected to differ from C++ (D4). The
  corpus sorts keys, so the difference is never compared.
- **Memory accounting** is an estimate of what the arenas hold, not C++'s
  allocation sizes. No test checks `max_memory_bytes` against the vendored
  VM's `lua_gc(LUA_GCCOUNT)`.
- **Decimal parsing** (`str2d`) uses Rust's `f64::from_str` on the
  validated text, which rounds correctly. It is not compared against
  `strtod` on hostile inputs beyond the corpus and `strconv.luau`. Hex
  parsing now keeps a sticky bit and rounds once at the result's precision
  (unit tests cover subnormals, ties and overflow).
- **Pseudo-addresses** in `tostring` of tables and functions are not C++'s.
  The harnesses never compare them.
- **No fuzzing.** The verifier is tested by the corpus (valid bytecode
  only) and by the bytecode the conformance scripts produce. Malformed
  bytecode is not fuzzed. The loader and verifier return errors by
  construction (no panic lints, no indexing), but nothing exercises them
  with hostile input yet.
- **Bytecode versions 3 to 6, 8, 9 and 10** are admitted but not produced
  by any profile the harness runs (it produces 7 and 11). The editor emits
  6.
- **`make lint-gate`** was not run in full. `clippy -p nuxie-luau-vm --lib`
  was run, and it is what the gate runs for this crate.

## 6a. Review

`codex review --base main` ran eight times. The coordinator made round 8
the last one for lane 1. Each round's findings were fixed
in their own commit (section 1), with a corpus case or a limit test that
reproduces the finding. In order, the rounds found:

1. The table-template preflight, accounting, the string limit, hex
   scaling, position backreferences and modf.
2. Table growth, `gsub` output, refused charges and the stack limit;
   modf's two bodies; subnormal hex.
3. `concat`/`format`/loader buffers, small-budget collection, `FASTCALL2`
   targets, `clamp` NaN, NaN vector keys and lossy diff outcomes.
4. The concat budget, the host-boundary safepoint, `__tostring` arity,
   `setfenv "0"` and `math.deg`.
5. Host `tostring` unwinding, rejected loads, `DUPTABLE` budget and the
   exact `%q` length.
6. `SETLIST` into a read-only table; the re-entry stack. The stack was
   documented and measured rather than fixed (section 6).
7. `num2str` tie-breaking (even last digit, as Schubfach), the missing
   method error on non-table receivers, and collection after `set_global`
   and `register`.
8. Budget checks in `upper`/`lower`/`reverse`/`char`; `string.rep("", n)`
   looping `n` times.

Every finding from every round is fixed. No review finding is left open.
Section 6 lists what is still unverified, and the re-entry stack is
documented rather than fixed.

The findings went 6, 6, 7, 5, 4, 2, 3, 2 per round: down overall, though
not monotonically, and most concerned the memory
budget (D3) under adversarial scripts. The limit tests live in
`tools/luau-vm-diff` (`limit_tests`), because the crate has no compiler of
its own. A round 7 result, if any, is in the hand-over note.

## 7. Plan lines (applied)

The commit "docs(luau): record the lane 1 outcome and revised lane
ownership" applies these to `docs/luau-safe-vm-plan.md`, under Phase 2. It
records the lane 1 outcome, the revised lane ownership and the re-entry
stack risk. The lines this document proposed were:

> - 2026-11-11: Lane 1 landed on `claude/luau-vm-lane1-core-0b14a3`
>   (evidence `docs/evidence/luau-safe-vm-phase2-lane1-2026-11-11.md`).
>   `crates/nuxie-luau-vm` forbids `unsafe` and denies the forced panic
>   lints. The differential corpus (26 cases, 4 compiler profiles, every
>   admitted opcode) agrees with the vendored VM. `scope-nuxie.txt`
>   requires 14 of 55 upstream scripts.
> - 2026-11-11: `literals` and `pm` stay blocked on the harness until the
>   compiler takes byte sources. That is the harness's gap, not a lane's.
> - 2026-11-11: Re-entry recurses on the Rust stack. The default 200
>   levels need 1 MiB in release and 6 MiB unoptimized. Lane 4 makes
>   re-entry stack-safe for every re-entry path, not only `pcall`.

Lane-split changes this run suggested (the plan commit adopts them as
the revised lane ownership):

- **Remove `strconv` and `stringinterp` from lane 2.** They already pass.
- **Give `calls`, `vararg` and `types` to lane 2.** Their first blocker is
  `table.getn` or `table.find`, not coroutines.
- **Split `closure` between lanes 3 and 4.** It needs weak tables (lane 3)
  before it reaches its coroutine half. Either lane 3 owns weak tables and
  `closure` waits for both, or weak-table clearing moves to lane 2. The
  first is more natural, since clearing happens during the collector's
  mark phase.
- **Give the unowned scripts to lane 5.** `debug`, `datetime`, `events`
  and `iter` have no lane in the plan.
- **Land `newproxy` before lane 2 claims `strings`.** `strings` is lane 2's,
  but its first blocker is `newproxy` (lane 5).
- **Add a byte-source compile entry** (harness work) so `literals` and `pm`
  can count. It is small, and it unblocks two scripts on both VMs.
- **Widen lane 4 to all re-entry.** Metamethods, `table.sort`
  comparators, `gsub` callbacks and host callbacks all re-enter through
  Rust recursion. A continuation-style `pcall` alone leaves the stack
  abort (section 6) open.
- **Give lane 3 the memory-budget suite.** Six review rounds found budget
  bypasses one at a time. Lane 3 (the collector) should own one
  accounting point per allocation site. `limit_tests` in
  `tools/luau-vm-diff` is the seed.
- **Lane 2 or 5 adds `table.create`, `clone`, `move` and `find`,
  `math.random` and `bit32` early.** They gate 25 of the 84 benches, and
  `sort.luau` needs only `table.clone`.
