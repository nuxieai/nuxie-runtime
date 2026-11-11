# Safe Luau VM, Phase 2 lane 2: tables and strings (2026-11-11)

Lane 2 of Phase 2 in `docs/luau-safe-vm-plan.md` ("How Phase 2 runs"),
written on `claude/luau-vm-lane2-tables-e68f37`, rebased on `origin/main`
after lane 1 merged (nuxieai/nuxie-runtime#1375).

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
- **Differential corpus: 28 cases × 4 profiles (112 runs), 0
  disagreements.** Two new cases. Five new limit tests.
- **Benches: 74 of 87 now run on nuxie** (lane 1: 59). Count only, no
  tuning (section 5).
- `unsafe` and the forced panic lints stay at 0 in the crate. The vendored
  changes add no `unsafe`.

## 1. Commits

| Commit | What |
| --- | --- |
| `feat(luau-vm): the rest of the table library` | The eleven functions, `Table::clear`/`clone_unfrozen`/`slot_count`, `Vm::check_stack` and `LUAI_MAXCSTACK` for `unpack`. Corpus case `tablelib2.luau`; limit test for `create` and `clone`. |
| `feat(luau-vm): string.gmatch and string.split` | `gmatch` state in a builtin upvalue (`Vm::set_builtin_upval`); builtin upvalues are charged. Corpus case `gmatch-split.luau`; limit test for `split` and the sparse `move`. |
| `test(luau-vm): require calls, clear, move and vararg on nuxie-luau-vm` | Scope file. |
| `feat(luaur): byte-source compile entries` | `Parser::parse_bytes`/`new_bytes`, `compile_bytes`, `Lua::load_bytes`, each with a `NUXIE_PATCH.md` entry. |
| `test(luau-conformance): compile sources as bytes; require literals and pm` | Both harnesses; both scope files. |
| `test(luau-vm): tables and types are permanent non-goals, as on luaur` | Per the coordinator: match `scope.txt`. |
| `fix(luau-vm): review round 1: ...` | Lossy lexeme text, builtin upvalues in the live estimate, no sparse-move key buffer. |
| `fix(luau-vm): review round 2: ...` | Sparse-move buffer checked per collected entry. |
| `fix(luau-vm): review round 3: ...` | Sparse move copies into a charged temporary table. |
| `docs(luau-vm): Phase 2 lane 2 evidence` | This document. |

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
- `getn`, `maxn`, `foreach` (`next` order, the key kept in a stack slot so
  it stays rooted across the call), `foreachi` (`#t` taken once), `freeze`,
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

- `src/heap.rs`: `function_bytes` charges a builtin's upvalues like a Lua
  closure's, and the post-collection live estimate uses the same function
  (review round 1).
- `src/libs/mod.rs`: `Vm::check_stack` with `LUAI_MAXCSTACK` (lane 5's
  `string.unpack` will want it too), and `Vm::set_builtin_upval`. Two
  additions; nothing existing changed.
- `tests/luau-conformance/scope-nuxie.txt`: only lane 2's lines, plus the
  `literals`/`pm` lines (harness work this lane was given). The `strings`
  line is unchanged from main (lane 5's).
- `tests/luau-conformance/scope.txt` (luaur): `literals` and `pm` move to
  required.
- `tools/luau-conformance` (both harnesses) and `tools/luau-vm-diff`
  (`limit_tests`): byte compile and new tests.
- Vendored `luaur-ast`, `luaur-compiler`, `luaur-rt` (section 4).

`vm.rs` and `interp.rs` are untouched.

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

`codex review --base origin/main`, three rounds:

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

The findings went 3, 1, 1, and every one was a memory-budget gap or an
unchecked `str`, the same pattern lane 1 saw. All are fixed and none is
open. Each fix has a limit test or a harness test.

## 7. Final gates

Run once on `d71e8e2bf9`; every command exited 0. The 1-minute load was
15.8 at the start and 19.5 at the end (other lanes' sessions were
building).

- `make luau-vm-diff`: 112 runs, 0 disagreements, every admitted opcode
  reached.
- `make luau-conformance-nuxie`: 19 OK, 35 FAIL, 1 TIMEOUT (closure, lane
  3). That is 19 required (all pass), 34 expected-fail and 2
  permanent-non-goal; no expected-fail passes.
- `make luau-conformance` (luaur): 30 OK; 30 required, 1 expected-fail,
  24 permanent-non-goal.
- `make luau-vm-ratchets`, `make ratchet-unsafe`, `make
  ratchet-panic-lints`: `crates/nuxie-luau-vm` is at 0 in each. The
  vendored `luaur-ast`, `luaur-compiler` and `luaur-rt` are unchanged
  (+0).
- `python3 tools/bazel/generate.py --check` is clean.
- `clippy -p nuxie-luau-vm --lib` is clean.
- Unit tests: `luau-vm-diff` 19 (`limit_tests`, 5 of them new) and
  `luau-conformance` 7 (1 new), all pass.

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
