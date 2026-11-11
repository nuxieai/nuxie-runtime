# Safe Luau VM, Phase 2 lane 5: libraries (2026-11-11)

Lane 5 of Phase 2 in `docs/luau-safe-vm-plan.md` ("How Phase 2 runs"). It
adds the standard libraries the new VM was missing. Branch
`claude/luau-vm-lane5-libraries-f204bd`, rebased onto `origin/main`; not
pushed.

**Outcome.**

- **New libraries:** `math` (all of it), `bit32`, `buffer`, `vector` (with
  the vector metatable), `utf8`, `os`, `debug` (`info`, `traceback`), and
  `string.pack` / `packsize` / `unpack`.
- **New value kinds:** the integer type (`Value::Integer`, integer
  constants in bytecode) and buffers (`Value::Buffer`).
- **Conformance on the new VM: 19 of 55 scripts are `required` and pass**
  (lane 1 left 14). The five new ones are `math`, `bitwise`, `tpack`,
  `vector_library` and `native_integer_spills`. Every other lane 5 script
  names its first blocking line in `scope-nuxie.txt` (section 3).
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
`native_integer_spills`.

Still `expected-fail`, each with its first blocking line:

| Script | Blocker | Owner |
| --- | --- | --- |
| `buffers` | `table.create` (buffers.luau:661, again :844). Passes with that call replaced. | lane 2 |
| `datetime` | `string.split` (datetime.luau:52). Passes with that call replaced. | lane 2 (strings) |
| `utf8` | `string.gmatch` (utf8.luau:77, again :227). Passes with both loops replaced. | lane 2 |
| `debug` | `coroutine.create` (debug.luau:19). `debug.info`/`traceback` have no thread forms until threads exist. | lane 4 |

**Permanent non-goals, matching `scope.txt`.** The coordinator ruled
(2026-11-11) that the new VM adds no script-visible surface the product
VM lacks, and that wherever `scope.txt` marks a script
`permanent-non-goal`, `scope-nuxie.txt` matches it. These lines are copied
from `scope.txt` word for word. Corrections to the lane 5 brief:

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
  - `execute` records the running pc in the frame an error leaves.
    Before this, an `xpcall` handler's `debug.traceback` printed the
    erroring frame without its line. The diff showed it.
- `libs/base.rs` (`xpcall`): calls the handler with the erroring frame's
  pc, so the call does not overwrite that frame's saved pc with the
  caller's.
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
| `debug.traceback` | the result | `append` before each growth |
| `debug.info` | the short source string (option `s`, at most 256 bytes) | `Heap::intern`; listed as non-allocating in the budget suite because the size is fixed |
| `string.pack` | the result, padding included | `check_buffer` before each growth |
| `string.unpack` | each `c`, `s`, `z` field | `Heap::intern`; at most 8000 results (`luaL_checkstack`) |

Not allocating: `math.*` (including `random` and `noise`), `bit32.*`,
`vector.*` (vectors are inline values), `buffer.read*`/`write*`/`len`/
`copy`/`fill`, `utf8.offset`/`codepoint`/`len`, `os.time`/`clock`/
`difftime`, `string.packsize`.

## 6. Benches

**66 of 84 benches run on the new VM** (lane 1: 59). This is a count,
not a timing gate. It was one pass (`luau-bench run --vm nuxie --runs 1`
on `8d14124b37`), under 1-minute load 18 to 26 from other lanes'
builds. The geomean it printed, 40.9 ms, is not comparable to lane 1's
number and is not a result.

All 18 failures are other lanes' pieces:

| Missing | Benches | Owner |
| --- | --- | --- |
| `table.create` | 14: the `TableCreate`×3, `TableMove`×3, `TableRemoval`, `TableSort` and `table_concat` micro tests; `matrixmult`, `mesh-normal-scalar`, `mesh-normal-vector`, `trig`; `chess`, whose polyfill assigns into the readonly `table` | lane 2 |
| `table.find` | 1: `TableFind` | lane 2 |
| `table.clone` | 1: `qsort` | lane 2 |
| `coroutine` | 2: `Pcall_pcall_yield`, `sieve` | lane 4 |

Since lane 1, `math.random`, `bit32` and `vector` removed every failure
they caused.

## 7. Final gates

Run on `57d409ed52` (rebased onto `origin/main` `3452bcfe36`), one
after another. The 1-minute host load ran from 5.5 to 16, from other
lanes' builds; none of these gates is timed.

| Gate | Result |
| --- | --- |
| `make luau-vm-diff` | 142 runs, 0 disagreements, 0 accounting drift; reaches every admitted opcode |
| `make luau-conformance-nuxie` | PASS: 19 required pass, 30 expected-fail, 6 permanent-non-goal, 0 panics |
| `make luau-conformance` (luaur) | PASS: 28 required, unchanged |
| `make luau-vm-ratchets` | PASS |
| `make ratchet-unsafe` | PASS (`nuxie-luau-vm` 0) |
| `make ratchet-panic-lints` | PASS (`nuxie-luau-vm` 0) |
| `python3 tools/bazel/generate.py --check` | OK |
| clippy, `nuxie-luau-vm` | clean |
| `nuxie-luau-vm` unit tests | pass |
| `luau-vm-diff` unit tests (limit tests, budget suite, collector tests) | pass |
| `cargo build -p nuxie-luau-vm --target wasm32-unknown-unknown` | builds (the only cargo use: Bazel has no wasm target for the crate) |

**Codex review** (`codex review --base origin/main`), two rounds:

- Round 1, four findings, all fixed:
  - the clocks used `SystemTime`/`Instant`, which panic on
    wasm32-unknown-unknown;
  - `buffer.copy` copied the source into an uncharged temporary;
  - `debug.traceback` copied its message before checking the budget;
  - `utf8.char` checked the budget against a worst-case length.
- Round 2, one finding, fixed: `string.unpack` collected its results
  in an uncharged `Vec`; they are now pushed onto the charged stack.

## 8. Not verified

- **The thread forms** of `debug.info` and `debug.traceback`. No thread
  values exist until lane 4.
- **Local time.** It is UTC by design (section 2).
- **The `int64` library.** The oracle opens it when `LuauIntegerLibrary`
  is on. No conformance script needs it.
- **Scripts behind another lane's blocker.** They were run past that
  line with the blocking call replaced in a scratch copy, not in the
  repo: `buffers`, `datetime` and `utf8` then pass.

## 9. Proposed plan lines

For `docs/luau-safe-vm-plan.md`, under Phase 2:

- 2026-11-11: lane 5 landed `math`, `bit32`, `buffer`, `vector`, `utf8`,
  `os`, `debug` (`info`, `traceback`) and `string.pack`, plus the
  integer and buffer value kinds. 19/55 scripts required. `buffers`,
  `datetime` and `utf8` wait on lane 2 (`table.create`, `string.split`,
  `string.gmatch`); `debug` waits on lane 4 (`coroutine.create`).
- 2026-11-11: `newproxy` stays out (coordinator): `errors`, `events`,
  `iter`, `strings`, `vector` and `native` are permanent non-goals in
  `scope-nuxie.txt`, as in `scope.txt`.
- 2026-11-11: local time on the new VM is UTC (no zone database in safe
  Rust). If products need local time, the host must supply the offset.
