# Safe Luau VM, Phase 2 lane 2: tables and strings (2026-11-11)

Lane 2 of Phase 2 in `docs/luau-safe-vm-plan.md` ("How Phase 2 runs"),
written on `claude/luau-vm-lane2-tables-e68f37`, rebased on `origin/main`
after lane 1 merged (nuxieai/nuxie-runtime#1375), and again after lane 3
(the collector and the memory budget) merged (nuxieai/nuxie-runtime#1380).
Section 3a covers what the second rebase changed.

**Outcome.**

- **The table library is complete.** Every function upstream registers is
  ported from the vendored `ltablib`: `create`, `clear`, `clone`, `move`,
  `find`, `getn`, `maxn`, `foreach`, `foreachi`, `freeze` and `isfrozen`
  are new. `unpack` now refuses past `LUAI_MAXCSTACK` (8000) slots, as
  `lua_checkstack` does.
- **The string library is complete except `pack`, `unpack` and
  `packsize`** (lane 5). `gmatch` and `split` are new.
- **Byte sources.** Both conformance harnesses compile scripts and
  `loadstring` arguments as bytes, through new safe entries in the
  vendored compiler. `literals` and `pm` now pass on luaur, and `pm` on
  nuxie.
- **Conformance on nuxie: 19 of 55 required** (lane 1 left 14). New:
  `calls`, `clear`, `move`, `vararg`, `pm`. On luaur, `literals` and `pm`
  join the required set (30).
- **Differential corpus: 29 cases × 4 profiles (116 runs, with lane 3's
  case), 0 disagreements.** Two new cases. Five new limit tests and six
  budget-suite cases.
- **Benches: 74 of 87 now run on nuxie** (lane 1: 59). Count only, no
  tuning (section 5).
- `unsafe` and the forced panic lints stay at 0 in the crate. The vendored
  changes add no `unsafe`.

## 1. Commits

| Commit | What |
| --- | --- |
| `feat(luau-vm): the rest of the table library` | The eleven functions, `Table::clear`/`clone_unfrozen` (later replaced by lane 3's `Heap::clone_table`)/`slot_count`, `Vm::check_stack` and `LUAI_MAXCSTACK` for `unpack`. Corpus case `tablelib2.luau`; limit test for `create` and `clone`. |
| `feat(luau-vm): string.gmatch and string.split` | `gmatch` state in a builtin upvalue (`Vm::set_builtin_upval`); builtin upvalues are charged. Corpus case `gmatch-split.luau`; limit test for `split` and the sparse `move`. |
| `test(luau-vm): require calls, clear, move and vararg on nuxie-luau-vm` | Scope file. |
| `feat(luaur): byte-source compile entries` | `Parser::parse_bytes`/`new_bytes`, `compile_bytes`, `Lua::load_bytes`, each with a `NUXIE_PATCH.md` entry. |
| `test(luau-conformance): compile sources as bytes; require literals and pm` | Both harnesses; both scope files. |
| `test(luau-vm): tables and types are permanent non-goals, as on luaur` | Per the coordinator: match `scope.txt`. |
| `fix(luau-vm): review round 1: ...` | Lossy lexeme text, builtin upvalues in the live estimate, no sparse-move key buffer. |
| `fix(luau-vm): review round 2: ...` | Sparse-move buffer checked per collected entry. |
| `fix(luau-vm): review round 3: ...` | Sparse move copies into a charged temporary table. |
| `docs(luau-vm): Phase 2 lane 2 evidence` | This document. |
| `fix(luau-vm): put lane 2's builtins on lane 3's budget` | After the rebase onto lane 3 (section 3a). |
| `fix(luau-vm): review round 4: ...` | `foreach` roots the current value across the callback. |

The `gmatch` commit's message says builtin upvalues are charged by this
lane's `function_bytes`; after the rebase that function is gone and lane
3's `function_cost` charges them (section 3a). History was not rewritten.

## 2. What was added

**Tables** (`src/libs/table.rs`, `src/table.rs`). Each function follows its
vendored body, including error text and argument order:

- `create(n [, v])`: `size out of range` for negative `n`; `table overflow`
  past `MAXSIZE` (2^26), before anything is allocated (C++
  `setarrayvector`). The array part is checked against the budget first.
- `clear`: C++ `luaH_clear`, keeping both parts' capacity.
- `clone`: C++ `luaH_clone`, the same parts, contents and metatable, neither
  read-only nor `safeenv`. Refused for a protected metatable. The copy's
  capacity is checked before it is made.
- `move`: C++ `tmove` and `moveelements`, with the destination's array
  resize and the `LuauTableMoveTimeoutFix` sparse path (section 4).
- `find`: equality with `__eq`, scanning from `init` to the first nil.
- `getn`, `maxn`, `foreach` (`next` order; the key and value kept in stack
  slots below the call, as C++ keeps `lua_next`'s pair, so both stay rooted
  across the callback: review round 4), `foreachi` (`#t` taken once), `freeze`,
  `isfrozen`.
- `unpack`: the 8000-slot `lua_checkstack` cap (tables.luau:677 checks
  that 7999 elements unpack and 8000 do not).

**Strings** (`src/libs/string.rs`).

- `gmatch`: an iterator closure over `(s, p, position)`, as C++
  `gmatch_aux`. `^` is not an anchor. The position is the third upvalue,
  replaced through `Vm::set_builtin_upval`, the analogue of C++
  `lua_replace(L, lua_upvalueindex(3))`. `NativeFn::BuiltinWith` keeps its
  `Rc<[Value]>`; the setter swaps in a new slice. The upvalue count never
  changes, so the replacement charges nothing new.
- `split(s [, sep])`: C++ `str_split`, with its default `","`, the
  empty-separator byte split, and no trailing piece for an empty separator.

## 3. Shared files touched, and why

- `src/heap.rs`: no change after the second rebase. Before it, this lane's
  `function_bytes` charged builtin upvalues (review round 1); lane 3's
  `function_cost` does that now, so it was dropped.
- `src/table.rs`: `clear`, `slot_count` and `max_number_key` (the storage
  is private since lane 3, so `maxn` asks the table).
- `src/libs/mod.rs`: `Vm::check_stack` with `LUAI_MAXCSTACK` (lane 5's
  `string.unpack` will want it too), and `Vm::set_builtin_upval`. Two
  additions; nothing existing changed. `set_builtin_upval` writes through
  lane 3's `Heap::function_mut`, so it takes the write barrier.
- `tests/luau-conformance/scope-nuxie.txt`: only lane 2's lines, plus the
  `literals`/`pm` lines (harness work this lane was given). The `strings`
  line is unchanged from main (lane 5's).
- `tests/luau-conformance/scope.txt` (luaur): `literals` and `pm` move to
  required.
- `tools/luau-conformance` (both harnesses) and `tools/luau-vm-diff`
  (`limit_tests`): byte compile and new tests.
- Vendored `luaur-ast`, `luaur-compiler`, `luaur-rt` (section 4).

`vm.rs` and `interp.rs` are untouched.

## 3a. On lane 3's budget

The rebase onto lane 3 conflicted only in `heap.rs` (lane 3's side taken).
Each new builtin now allocates through the rules in `budget.rs`:

| Builtin | How it allocates | Budget-suite entry |
| --- | --- | --- |
| `table.create` | `Vm::new_table(n, 0)`, then `array_mut().fill` (no growth) | case |
| `table.clone` | `Heap::clone_table`, then clears `readonly`/`safeenv` | case |
| `table.move` | `Heap::table_resize_array` and `raw_set`; the sparse path's temporary is a `Vm::new_table`, rooted on the stack | cases `table.move` and `table.move(sparse)` |
| `string.split` | `Vm::new_table`, `intern`, `raw_set` | case |
| `string.gmatch` | `Heap::new_function` with 3 upvalues (charged by `function_cost`); each step replaces an upvalue, same count | case |
| `clear`, `find`, `foreach`, `foreachi`, `freeze`, `getn`, `isfrozen`, `maxn` | nothing (callbacks allocate on their own account) | `NON_ALLOCATING` |

`every_builtin_is_classified` and the `accounting_drift` check pass with
these (`luau-vm-diff` unit tests, section 7).

`scope-nuxie.txt`'s `gc` line now records that the collector parts pass
through gc.luau:225 and the `table.clear`/`table.create` part through
:244; the next blockers are `newproxy` (gc.luau:245, lane 5) and
`coroutine.create` (:322, lane 4). `gc` and `closure` stay out of the
required set (permanent non-goals on luaur).

## 4. Byte sources

The vendored lexer already reads a pointer and a length; only the entry
points took `&str`. The change is small and adds no `unsafe`:

- `luaur-ast`: `Parser::new_bytes` and `Parser::parse_bytes`; `new` and
  `parse` delegate to them.
- `luaur-compiler`: `compile_bytes(&[u8], ...)`; `compile` delegates.
- `luaur-rt`: `Lua::load_bytes`; `Chunk` carries an optional byte source.
  `Lua::load` is unchanged.

Two existing `from_utf8_unchecked` sites could now see non-UTF-8 bytes.
Both now copy lossily: the hot-comment content (`--!...`), and
`Lexeme::to_string`, which builds syntax-error text (review round 1). The
second was already reachable before this lane, through escapes such as
`"\255"` in a broken token. Other `from_utf8_unchecked` sites in the
vendored crates are in printers, bytecode dumps and the C ABI
`luau_compile`, which already took arbitrary bytes. None is on the
harness's compile path, and none was audited further.

Result: `literals` and `pm` pass on luaur. On nuxie, `pm` passes, and
`literals` stops at `coroutine.wrap` (literals.luau:158). With that loop
driven by a callback instead, the rest of `literals` passes on both VMs.

## 5. Conformance scope

Lane 2's scripts (gate run in section 7):

| Script | Result | Blocker | Owner |
| --- | --- | --- | --- |
| calls, clear, move, vararg | **pass (required)** | | |
| pm | **pass (required)** | was the harness's Latin-1 transcoding | |
| sort | expected-fail | `math.random` (sort.luau:64), `os.clock` (sort.luau:67). With both stubbed the script passes. | lane 5 |
| tables | permanent-non-goal, as in `scope.txt` | host `makelud` (tables.luau:684). Also `bit32.extract` (662, lane 5), `coroutine.wrap` (727, lane 4), and the `pairs` order of a constant-key table (797–821, free under D4). With all four stubbed or cut, the rest passes. | harness |
| types | permanent-non-goal, as in `scope.txt` | the analysis frontend's RTTI table (types.luau:44). `table.find` now works. | harness |
| strings | expected-fail, line unchanged | `newproxy` (strings.luau:172). The Rive profile has no `newproxy`, and `scope.txt` files the script as permanent-non-goal; the coordinator dropped it from lane 2. With a stub `newproxy` the rest passes. | lane 5 owns the line |
| literals | expected-fail | `coroutine.wrap` (literals.luau:158) | lane 4 |

The "with stubs" results came from local copies of the scripts: lane-5
functions replaced by Lua stand-ins, and blocks cut. They are evidence of
what blocks a script, not a pass.

**Benches** (count only; `luau-bench run --vm nuxie --runs 1`, release
build of `615a17e653`, one pass in 1123.5 s, 1-minute load 7.0 → 20.1 with
this lane's own builds and reviews running): **74 of 87 run** (lane 1: 59
of 84; the merged suite now has 87). Every one of the 13 failures is
another lane's library:

- `math.random`/`randomseed`: 4;
- `vector.create`: 4;
- `bit32`: 3;
- `coroutine`: 2.

No table or string function is missing from any bench.

## 6. What was not verified

- **`table.move` follows only the flags-on profile.** The vendored VM
  takes the sparse path only under `LuauTableMoveTimeoutFix`, which every
  harness turns on. Under the corpus's flags-off profile the vendored VM
  runs its element loop. Results agree, but a 2^30 range takes minutes
  there, so the huge sparse ranges live in `limit_tests`, not the corpus.
- **The sparse path's insertion order** into the destination follows
  this crate's iteration, not C++'s temporary table. Contents agree; hash
  shape, and so later `pairs` order, may not. `pairs` order is free under
  D4.
- **`table.create` capacity rules** beyond what `clear.luau` and the
  corpus check (`#`, values, overflow) are not compared.
- **The vendored `from_utf8_unchecked` sites** outside the compile path
  (section 4) were not audited.
- **Re-entry depth** for `foreach`/`foreachi` callbacks is the same Rust
  recursion lane 1 documented (bounded by `max_c_calls`), and was not
  measured separately.

## 6a. Review

`codex review --base origin/main`, three rounds before the second rebase
and one after it:

1. Three findings, all fixed: (P1) `Lexeme::to_string` built an unchecked
   `str` from bytes a byte source can reach; (P2) the live estimate after
   a collection dropped builtin upvalues, freeing fictitious budget (the
   test fails without the fix); (P2) the sparse move's destination-key
   buffer was unbudgeted.
2. One finding, fixed: the sparse move reserved the whole source
   capacity, so an empty range could fail under a small budget.
3. One finding, fixed: the buffer was checked by length, but a `Vec`
   grows by doubling, and the storage was never charged. The sparse move
   now copies into a temporary VM table, as C++ does. That table is charged
   like any table and rooted in a stack slot.
4. (After rebasing onto lane 3.) One finding, fixed: `foreach` rooted
   only the key across the callback, so a weak-value entry whose
   callback dropped its argument and collected was cleared; C++ keeps the
   value on the stack too. Diff case in `tablelib2.luau`.

The findings went 3, 1, 1, 1. The first three was a memory-budget gap or an
unchecked `str`, the same pattern lane 1 saw; the fourth a GC root. All are fixed and none is
open. Each fix has a limit test or a harness test.

## 7. Final gates

Run once on `7efec2d465` (the code tip after rebasing onto lane 3 and
review round 4; only this document changes after it). Every command
exited 0. The 1-minute load was 9.7 at the start and 19.7 at the end
(other sessions were building).

- `make luau-vm-diff`: 116 runs, 0 disagreements, every admitted opcode
  reached.
- `make luau-conformance-nuxie`: 55 scripts, 19 OK, 36 FAIL, 0 TIMEOUT.
  That is 19 required (all pass), 34 expected-fail and 2
  permanent-non-goal; no expected-fail passes.
- `make luau-conformance` (luaur): 30 OK; 30 required, 1 expected-fail,
  24 permanent-non-goal.
- `make luau-vm-ratchets`, `make ratchet-unsafe`, `make
  ratchet-panic-lints`: `crates/nuxie-luau-vm` is at 0 in each. The
  vendored `luaur-ast` (1017), `luaur-compiler` (744) and `luaur-rt` (320)
  `unsafe` counts are unchanged (+0).
- `python3 tools/bazel/generate.py --check` is clean.
- `clippy -p nuxie-luau-vm --lib` is clean.
- Unit tests: `luau-vm-diff` 31 (`limit_tests` and the budget suite) and
  `luau-conformance` 7 (1 new), all pass.

Before the second rebase the same list was green on `d71e8e2bf9` (112
runs; `closure` then timed out on nuxie, which lane 3's collector fixed).

## 8. Proposed plan lines

> - 2026-11-11: Lane 2 (tables and strings) on
>   `claude/luau-vm-lane2-tables-e68f37` (evidence
>   `docs/evidence/luau-safe-vm-phase2-lane2-2026-11-11.md`). The table
>   library is complete and the string library is complete except
>   pack/unpack/packsize (lane 5). `scope-nuxie.txt` requires 19 of 55. 74
>   of 87 benches run on nuxie; the other 13 need `math.random`, `vector`,
>   `bit32` or `coroutine`.
> - 2026-11-11: The harnesses compile sources as bytes (`compile_bytes`,
>   `Lua::load_bytes`, no new `unsafe`). `literals` and `pm` are required
>   on luaur; `pm` on nuxie, and `literals` waits on lane 4.
> - 2026-11-11: `sort` waits only on `math.random` and `os.clock` (lane 5).
>   `tables` and `types` are permanent non-goals, as on luaur.
