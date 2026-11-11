# Safe Luau VM, Phase 2 lane 5: libraries (2026-11-11)

Lane 5 of Phase 2 in `docs/luau-safe-vm-plan.md` ("How Phase 2 runs"). It
adds the standard libraries the new VM was missing. Branch
`claude/luau-vm-lane5-libraries-f204bd`, rebased onto `origin/main`
`12de40cfbb` (lanes 3, 2 and 4); not pushed. Section 10 records what the
lane 2 and lane 4 rebase changed; the sections before it describe the lane
as a whole.

**Outcome.**

- **New libraries:** `math` (all of it), `bit32`, `buffer`, `vector` (with
  the vector metatable), `utf8`, `os`, `debug` (`info`, `traceback`), and
  `string.pack` / `packsize` / `unpack`.
- **New value kinds:** the integer type (`Value::Integer`, integer
  constants in bytecode) and buffers (`Value::Buffer`).
- **Conformance on the new VM: 31 of 55 scripts are `required` and pass**
  (main had 21). Lane 5 adds ten: `math`, `bitwise`, `tpack`,
  `vector_library`, `native_integer_spills`, and since the lane 2 and
  lane 4 rebase `buffers`, `datetime`, `debug`, `sort` and `utf8`. Every
  lane 5 script is now required or a permanent non-goal (section 3).
- **`debug.info` and `debug.traceback` take a thread** and count levels on
  that thread's frames (section 10).
- **Differential corpus:** 9 new cases, 142 runs in all,
  0 disagreements.
- The crate stays at 0 in both ratchets: no `unsafe`, no panic-lint
  `allow`.
- No new dependencies.

## 1. Commits

| Commit | What |
| --- | --- |
| `feat(luau-vm): finish the math library; run ldexp's fast body` | `frexp`, `ldexp`, `fround`, `map`, `lerp`, `isnan`/`isinf`/`isfinite`, PCG32 `random`/`randomseed`, Perlin `noise`, the constants. Fast bodies move to `libs/fast.rs`. `mathlib.luau`. |
| `feat(luau-vm): the integer type and integer constants` | `Value::Integer`, the loader's integer constant, `hashint` keys, exact `==`. `integer.luau` (flags-on). |
| `feat(luau-vm): the bit32 library` | All 15 functions. `bit32.luau`. |
| `feat(luau-vm): the buffer value kind and library` | `Value::Buffer`, the heap arena, every `lbuflib` function plus `readinteger`/`writeinteger`. `buffer.luau` (flags-on). |
| `feat(luau-vm): the vector library and vector metatable` | The library, the readonly metatable whose `__index` reads x/y/z, fused `magnitude`/`dot` fast bodies. `vectorlib.luau`. |
| `feat(luau-vm): the utf8 library` | `offset`, `codepoint`, `char`, `len`, `codes`, `charpattern`. `utf8lib.luau`. |
| `feat(luau-vm): the os library` | `clock`, `date`, `difftime`, `time`. `oslib.luau`. |
| `fix(luau-vm): keep the erroring frame's pc for xpcall handlers` | A traceback taken in an `xpcall` handler lost the erroring frame's line (section 4). |
| `feat(luau-vm): the debug library (info, traceback)` | `debuglib.luau`. |
| `feat(luau-vm): string.pack, packsize and unpack` | `strpack.luau`. |
| `test(luau-vm): move lane 5 scripts to required` | The scope file. |
| `test(luau-vm): match scope.txt's permanent non-goals in scope-nuxie` | The coordinator's ruling (section 3). |
| `fix(luau-vm): wasm clocks, in-place buffer.copy, budget-first traceback and utf8.char` | Codex review round 1 (section 7). |
| `fix(luau-vm): string.unpack pushes results onto the charged stack` | Codex review round 2 (section 7). |
| `test(luau-vm): classify lane 5 builtins in the budget suite` | Lane 3's classification (section 4). |
| `docs(luau-vm): Phase 2 lane 5 evidence` | This document. |
| `feat(luau-vm): debug.info and traceback take a thread` | After the lane 4 rebase (section 10). `debug_threads.luau`. |
| `test(luau-vm): pcall.luau's traceback handlers in debug_threads` | `pcall.luau:106`/`:111` shapes. |
| `test(luau-conformance): require buffers, datetime, debug, sort and utf8` | The scope file (section 3). |
| `fix(luau-vm): debug names are shared, not copied outside the budget` | Codex review round 3 (section 7). |
| `docs(luau-vm): lane 5 evidence after the lanes 2 to 4 rebase` | This revision. |

`fix(luau-vm): keep the erroring frame's pc for xpcall handlers` was
dropped in the lane 4 rebase: lane 4 runs the `xpcall` handler on the
intact stack, and `debug_threads.luau` shows the handler's traceback and
`debug.info(2, "sl")` agree with the oracle without it (section 10).

## 2. Matching the vendored VM

Each case in `tests/luau-vm-diff` runs on both VMs under four compiler
profiles. Two of the new cases (`integer.luau`, `buffer.luau`) start with
`-- luau-vm-diff: flags-on`, a new directive that skips the `base-O1`
profile: integer literals do not compile with the flags off. The README
documents it.

**Fast bodies that differ from their library function.** Under `safeenv`
a compiled builtin call runs the `luauF_*` fast body, not the library
function. Where the two compute different results, this VM runs both,
each where upstream runs it (`libs/fast.rs`). Each has a diff case that
calls the builtin both ways (directly, and through `pcall`):

| Builtin | Difference | Case |
| --- | --- | --- |
| `math.modf` | Fast body: signed-zero fraction for integral inputs. | `fastcall.luau` (lane 1) |
| `math.ldexp` | Fast body: C `ldexp`. Library: `x * 2^e` through `powi`, which overflows or flushes for \|e\| > 1023. | `mathlib.luau` |
| `vector.magnitude`, `vector.dot` | Fast bodies sum with fused multiply-adds; library bodies do not. | `vectorlib.luau` |

The other fast bodies (`bit32.*`, the rest of `math.*` and `vector.*`,
`buffer.read*`/`write*`) compute what their library functions compute,
so the `CALL` runs the library function.

**Oracle quirks kept on purpose:**

- `rawequal` and table keys compare integers as doubles; the
  interpreter's `==` compares the 64-bit values exactly.
- Integer keys hash with `hashint` (MurmurHash64B, low half).
- `debug.traceback` reads its message as UTF-8. A message that is not
  valid UTF-8 becomes empty; one with an embedded NUL is cut there.
- `string.pack` pushes a nil mark above its arguments, so the first
  argument past the end reports "got nil", not "missing argument".
- An invalid `string.pack` format byte at or above 0x80 is named as the
  Latin-1 character of that byte.

**Deliberate deviations from the oracle:**

- **`vector.angle` with one argument.** The vendored VM reads a null
  vector and crashes. This VM checks argument 2 (`luaL_checkvector`, as
  upstream C++ does) and raises the argument error. Not in the diff
  corpus, since the oracle would crash.
- **Local time is UTC.** Safe Rust has no time-zone database. `os.date`
  without `!` gives the UTC fields, with `%z` = `+0000`, `%Z` = `UTC`
  and `isdst` = `false`. It still gives nil before the epoch, as upstream
  does. `oslib.luau` compares only the `!` forms and fixed timestamps.
- **The PCG32 seed.** Upstream seeds `math.random` from an address; this
  VM seeds from 0. Scripts that call `randomseed` get the same sequence
  on both VMs (`mathlib.luau`).
- **`os.clock`** counts from the first call in the process, not from an
  arbitrary monotonic origin. Only differences between calls mean
  anything on either VM.

## 3. Scope changes (`tests/luau-conformance/scope-nuxie.txt`)

Moved to `required`: `math`, `bitwise`, `tpack`, `vector_library`,
`native_integer_spills`; after the lane 2 and lane 4 rebase also
`buffers`, `datetime` and `utf8` (their lane 2 blockers landed), `debug`
(coroutines landed and the debug functions take a thread) and `sort`
(blocked only by `math.random` and `os.clock`).

Other scripts the coordinator asked to re-check, on `12de40cfbb` plus
this lane:

| Script | Now stops at | Reason line |
| --- | --- | --- |
| `errors` | errors.luau:197, `newproxy` | unchanged; the `debug.traceback` use at :162 passes |
| `pcall` | pcall.luau:80 (reported at :8), D8 | the `debug.traceback` handlers at :106 and :111 now differ only by the D8 prefix (`debug_threads.luau` checks both shapes); lane 5 dropped from the line |
| `tables` | tables.luau:684, host `makelud` | `bit32.extract` (:662) passes; lane 5 dropped from the line |
| `gc` | gc.luau:245, `newproxy` | `newproxy` is a non-goal, not lane 5 work |
| `coroutine` | coroutine.luau:302, D8 | unchanged (lane 4's line) |

**Permanent non-goals, matching `scope.txt`.** The coordinator ruled
(2026-11-11) that the new VM adds no script-visible surface the product
VM lacks, and that wherever `scope.txt` marks a script
`permanent-non-goal`, `scope-nuxie.txt` matches it. These lines are copied
from `scope.txt` word for word. After the lane 4 rebase five more lines
were brought in line with `scope.txt`, which marks them permanent
non-goals: `tmerror` (D8 plus host `debugprotectederror` callbacks),
`debugger`, `native_types`, `integers` and `integers_regspill`. Their
blocking lines are unchanged. Corrections to the lane 5 brief:

| Script | Brief said | Now | First line it stops at on this VM |
| --- | --- | --- | --- |
| `errors` | move to required | permanent-non-goal: no `newproxy` (errors.luau:197) | `string.gmatch`, errors.luau:164 (lane 2) |
| `events` | move to required | permanent-non-goal: no `newproxy` (events.luau:13) | the same line |
| `iter` | move to required | permanent-non-goal: no `newproxy` (iter.luau:70) | the same line |
| `vector` | move to required | permanent-non-goal: host vector metatable helpers (vector.luau:99) | the same line; 1-98 pass |
| `strings` | lane 2's | permanent-non-goal: no `newproxy` (strings.luau:172) | the same line |
| `native` | not in the brief | permanent-non-goal: `is_native()` (native.luau:531) | `table.create`, native.luau:390 (lane 2) |

**`newproxy` was not added.** The vendored Rive base library omits it
(`rive_base_library_omits_print_and_newproxy`,
`vendor/luaur-vm-0.1.8/src/rung9_tests.rs:785`), and the coordinator
confirmed: do not build it.

**The integer type stays.** It is oracle surface, not new surface:
`native_integer_spills.luau` uses integer literals (`1i` from line 457 on)
and is `required` on luaur in `scope.txt`. Without the loader's integer
constant the script does not load (lane 1's reason was "the loader
rejects integer constants"). `bit32` alone is not enough.

## 4. Shared files touched, and why

- `interp.rs`:
  - The `FASTCALL*` block is now a generic dispatch to `libs::fast`. It
    computes each form's arguments and runs a fast body only where one
    differs from its library function.
  - (Before the lane 4 rebase, `execute` and `libs/base.rs`'s `xpcall`
    also kept the erroring frame's pc for the handler. Lane 4's handler
    runs on the intact stack, so that change was dropped.)
- `vm.rs`: `rng_state` (PCG32) and `vector_meta` (the shared vector
  metatable, a GC root).
- `value.rs`, `table.rs`, `meta.rs`: the `Integer` and `Buffer` value
  kinds: type names, hashing, equality, `tostring`.
- `heap.rs`: the buffer arena, its charge-before-allocate constructor,
  mark and sweep, and `intern_buffer_slice`.
- `bytecode/load.rs`: the integer constant (`varint64`).
  `bytecode/mod.rs`: dropped the `dead_code` allow on `linedefined`,
  which `debug.info` now reads.
- `libs/mod.rs`, `host.rs`: module declarations and `open_libs`.
- `tools/luau-vm-diff/src/lib.rs`: the `flags-on` directive.

**Rebased onto lane 3 (nuxieai/nuxie-runtime#1380).** All allocation now
goes through lane 3's `budget.rs`, so the rebase changed four more
places:

- `heap.rs`: the buffer arena is a new arena kind on lane 3's model.
  `new_buffer` charges the budget before it allocates and credits the
  charge back if the arena refuses. Sweep credits each freed buffer.
  The incremental collector whitens buffers, marks them black when
  reached (they hold no references, so they are never gray and need no
  write barrier), clears them from weak tables, and counts them in
  `recount`.
- `gc.rs`: `vector_meta` is a root in `gc_roots`.
- `tools/luau-vm-diff/src/budget_suite.rs`: every lane 5 builtin is
  classified. Each allocating one in section 5 has a case that runs out
  of memory under the suite's 1 MiB budget with no accounting drift.
  `bit32` and `vector` join `math` as whole non-allocating libraries.
  The rest are listed one by one.
- `tools/luau-vm-diff/src/collector_tests.rs`: lane 5's libraries grow
  a fresh VM from 12.5 KB to 22.9 KB. The test
  `memory_pressure_wins_over_an_incremental_step` keeps its margin with
  a budget of 51,000 bytes instead of 40,000. Its loop's garbage is
  still far larger than the budget.

## 5. Allocating builtins

Every allocation below is checked against `Limits` and the memory budget
before it happens.

| Builtin | Allocation | Check |
| --- | --- | --- |
| `buffer.create`, `buffer.fromstring` | the buffer | `Heap::new_buffer` charges first; above 1 GiB raises "memory allocation error: block too big". |
| `buffer.tostring`, `buffer.readstring` | a string | `Heap::intern` / `intern_buffer_slice` |
| `utf8.char` | the result | `check_buffer` before each code point |
| `utf8.codes` | a step closure | `Heap::new_function` |
| `os.date` | the formatted string, or the `*t` table | `append` before each growth; `new_table` / `raw_set` |
| `debug.traceback` | the result | `append` before each growth; names are appended from the interned string, not a copy |
| `debug.info` | the short source string (option `s`, at most 256 bytes); a native's name (option `n`) | `Heap::intern`; listed as non-allocating in the budget suite because both are bounded. A Lua function's name is returned as the interned string it already is, never copied (review round 3) |
| `string.pack` | the result, padding included | `check_buffer` before each growth |
| `string.unpack` | each `c`, `s`, `z` field | `Heap::intern`; at most 8000 results (`luaL_checkstack`) |

Not allocating: `math.*` (including `random` and `noise`), `bit32.*`,
`vector.*` (vectors are inline values), `buffer.read*`/`write*`/`len`/
`copy`/`fill`, `utf8.offset`/`codepoint`/`len`, `os.time`/`clock`/
`difftime`, `string.packsize`.

## 6. Benches

**87 of 87 benches run on the new VM** after the lanes 2 to 4 rebase (on
`3d5176d0ea`). This is a count, not a timing gate: one pass
(`luau-bench run --vm nuxie --runs 1`) under 1-minute load 28.8 from
other lanes' builds. The printed geomean, 38.8 ms, is not a result.

Before the rebase, 66 of 84 ran (one pass on `8d14124b37`, load 18 to
26). All 18 failures then were other lanes' pieces: `table.create`,
`table.find` and `table.clone` (lane 2, 16 benches) and coroutines
(lane 4, `Pcall_pcall_yield` and `sieve`). The suite grew to 87 with
lane 2's merge script; all of them run now.

## 7. Final gates

**Final, on the lanes 2 to 4 rebase.** The full list ran once on
`aceac6ffd5`. After review round 3, the gates its fix touches ran again
on the code tip `3d5176d0ea`: diff, both crates' tests, clippy, both
ratchets and `conformance-nuxie`. The later commits change only this
document. Host 1-minute load was 5 to 13 for the full list and 13 for
the rerun; none of these gates is timed. All exit 0.

| Gate | Result |
| --- | --- |
| `make luau-vm-diff` | 170 runs, 0 disagreements, 0 accounting drift; every admitted opcode reached |
| `make luau-conformance-nuxie` | PASS: 31 required pass, 7 expected-fail, 17 permanent-non-goal, 0 panics |
| `make luau-conformance` (luaur) | PASS: 30 required, unchanged from main |
| `make luau-vm-ratchets` | PASS |
| `make ratchet-unsafe` | PASS (`nuxie-luau-vm` 0) |
| `make ratchet-panic-lints` | PASS (`nuxie-luau-vm` 0) |
| `python3 tools/bazel/generate.py --check` | OK |
| clippy: `nuxie-luau-vm`, `luau-vm-diff`, `luau-conformance` | clean. `luau-vm-diff`'s build prints 5 rustc warnings from `vendor/luaur-common`, not from its own code |
| `nuxie-luau-vm` unit tests | pass |
| `luau-vm-diff` unit tests | 40 pass: limit tests (including lane 4's `reentry_reaches_its_limits_on_a_one_mib_stack`), the budget suite with `every_builtin_is_classified`, and the collector tests |
| `luau-conformance` unit tests | pass |
| `nuxie-scripting` | not run: nothing under `vendor/` or the product crates changed |

**Before the lanes 2 to 4 rebase** (on `57d409ed52`, over lane 3): diff
142 runs, 0 disagreements; 19 required; every other gate above passed.
`cargo build -p nuxie-luau-vm --target wasm32-unknown-unknown` built;
this is the one cargo use, because Bazel has no wasm target for the
crate. It was not repeated after the rebase.

**Codex review** (`codex review --base origin/main`), three rounds:

- Round 1, four findings, all fixed:
  - the clocks used `SystemTime`/`Instant`, which panic on
    wasm32-unknown-unknown;
  - `buffer.copy` copied the source into an uncharged temporary;
  - `debug.traceback` copied its message before checking the budget;
  - `utf8.char` checked the budget against a worst-case length.
- Round 2, one finding, fixed: `string.unpack` collected its results
  in an uncharged `Vec`; they are now pushed onto the charged stack.
- Round 3, on the lanes 2 to 4 rebase, one finding, fixed: `debug.info`
  and `debug.traceback` copied a Lua function's name (any length) into
  an uncharged `Vec`. They now use the interned string. The copy was
  transient and not visible to scripts, so
  `debug_names_are_not_copied_outside_the_budget` pins the behaviour
  around it: under a budget too small for a second copy of a 512 KiB
  name, `info` still returns the name and `traceback` is refused.

## 8. Not verified

- **Local time.** It is UTC by design (section 2).
- **The `int64` library.** The oracle opens it when `LuauIntegerLibrary`
  is on. No conformance script needs it.
- **`newproxy`.** Not built (coordinator). `errors`, `events`, `iter`,
  `strings` and `gc` stop at it.
- **`debug.info`'s `lua_rawcheckstack(L1, 1)` on another thread.** Upstream
  makes room for one slot on the target thread. This VM reads the
  parked thread without touching its stack, so it never charges that
  slot. No script sees the difference.
- **The re-entry limit.** No lane 5 builtin calls back into script
  (section 10), so none was added to
  `reentry_reaches_its_limits_on_a_one_mib_stack`.
- **wasm32 after the rebase.** The wasm build ran before the lanes 2 to
  4 rebase, not after it.
- **The removed name copy.** Section 7 says why no test fails without
  round 3's fix.

## 9. Proposed plan lines

For `docs/luau-safe-vm-plan.md`, under Phase 2:

- 2026-11-11: **lane 5 done**: `math`, `bit32`, `buffer`, `vector`,
  `utf8`, `os`, `debug` (`info`, `traceback`, with thread forms) and
  `string.pack`, plus the integer and buffer value kinds. 31/55 scripts
  required on the new VM; 87/87 benches run. No lane 5 builtin calls
  back into script.
- 2026-11-11: `newproxy` stays out (coordinator): `errors`, `events`,
  `iter`, `strings`, `vector` and `native` are permanent non-goals in
  `scope-nuxie.txt`, as in `scope.txt`.
- 2026-11-11: local time on the new VM is UTC (no zone database in safe
  Rust). If products need local time, the host must supply the offset.

## 10. Lanes 2 and 4 rebase

Rebased onto `12de40cfbb` (lane 2's table and string libraries, then
lane 4's call protocol and coroutines).

- **Conflicts.** Most were unions:
  - module lists;
  - `Value` arms, the hash tag and `pseudo_address` (buffers moved to
    their own tags next to threads);
  - the heap arenas;
  - the budget-suite cases and `NON_ALLOCATING`;
  - the limit tests;
  - the scope file.

  Integer `==` in `meta.rs` now returns lane 4's `Meta::Done`.
- **The xpcall pc fix was dropped.** Lane 4 runs the handler on the
  intact stack at upstream's call level. `debug_threads.luau` shows
  `debug.traceback` and `debug.info(2, "sl")` in a handler agree with
  the oracle, in the main thread and in a coroutine, before and after a
  yield (`pcall.luau:106`/`:111`).
- **Callbacks into script.** None of this lane's builtins calls a
  function or a metamethod, so none needs lane 4's frame path or
  `call_value`:
  - `os.time` reads its fields raw (`lua_rawgetfield`, as upstream);
  - `debug.info(f)` only inspects `f`;
  - `debug.traceback` takes its message as a string or number;
  - `string.pack`'s argument checks take no metamethods.

  The vector metatable's `__index` is a native that the interpreter calls
  as a metamethod on lane 4's frame path.
- **Thread forms of `debug.info` and `debug.traceback`** (C++
  `getthread`):
  - A thread first argument shifts the others by one, and the function
    form is refused after a thread ("function or level expected").
  - Levels count down that thread's own frames. A parked thread's frames
    and pc come from its arena slot; the running thread's come from the
    `Vm` fields.
  - `traceback`'s default level is 0 for another thread and 1 for the
    running one.

  `debug_threads.luau` covers fresh, suspended, normal, finished and
  failed coroutines, the running coroutine passed explicitly, and
  argument errors. Mutation check: reading the running thread instead of
  the argument makes it disagree on all four profiles.
- **Scope** (section 3): `buffers`, `datetime`, `debug`, `sort` and
  `utf8` are required. The reason lines for `tables`, `gc` and `pcall`
  no longer name lane 5. Five lines were brought in line with
  `scope.txt`'s permanent non-goals.
