# Safe Luau VM, Phase 3 item 1: `pairs` order (2026-11-13)

Builds `docs/luau-safe-vm-pairs-order.md` in `crates/nuxie-luau-vm`: the
hash part iterates in the vendored VM's order for string, number, boolean
and vector keys. Branch `claude/luau-vm-pairs-order-5e30b1` on
`origin/main` `47d8119ab4`; not pushed, no PR.

**Outcome.**

- **The generated order corpus agrees**: 600 tables over 40 cases, 4
  compiler profiles. Before: 542 of 600 tables disagree on every profile
  (2,168 of 2,400 table-profile pairs). After: 0. With the named cases,
  `make luau-vm-diff` is 405 runs, 0 disagreements.
- **`data_bind_blob_test` is exact** under `scripting-nuxie-vm`. The whole
  scripted compare on the safe VM is now the default VM's result: 364
  entries, 348 exact, the same 16 diverges, and one failure, the known
  main red `semantic_text_inference_lt1`.
- **The design did not survive whole** (section 4). The hash function was
  not the only difference: C++'s collector decides placement in two
  places the design did not list, dead keys and the `s`-mode shrink. Both
  are ported. Library registration order (audit item 8) was the third
  difference, as the design allowed.
- **Conformance**: `tables.luau` is required on the safe VM (it was a
  permanent non-goal for its hash-order block). `basic.luau` stays
  expected-fail on Rive `luaL_where` (line 39); its hash-order sites pass.
- **Instruction table**: geomean 1.0022 over 89 benches; four benches moved
  more than 3% (section 7).

## 1. Commits

| Commit | What |
|---|---|
| `81d184a0f5` | `luau-vm-diff`: the generated corpus (`gen_pairs_order_cases.py`, seed 20261113), the per-table counter (`count_pairs_order.py`) and the lane 7 reproduction |
| `ba26da9357` | `StrId` is the arena index and the Luau hash (3.1) |
| `31a924f671` | `hash_value` per key type (3.2) |
| `53f708e7ab` | dead keys, the `s`-mode shrink, `-0` stored as `-0`; the corpus made independent of collector timing |
| `ee8484c018` | library tables in C++ registration order; the audit cases |
| `fbb7d23443` | conformance: `tables.luau` required |
| `c426fbe631` | the Paint definition case (from the cutover-prep session's note) |
| `18352d8009` | review round 1: `kv` tables keep their empty nodes; a refused shrink is skipped |
| `0c4a65e334` | review round 2: an empty shrink needs no budget check |
| `4c13381941` | the instruction table |

## 2. What was built

**The string handle (3.1).** `StrId { index, hash }`, 8 bytes; `Value`
stays 16 (both asserted at compile time). `luau_string_hash` is C++
`luaS_hash` (`lua_s_hash.rs`), a `const fn`; a unit test pins 13 values
printed by the vendored function, both branches (under 32 bytes; 12-byte
blocks). One trap: C++'s `rol(x, s)` macro is `(x >> s) | (x << (32 -
s))`, a right rotation, and the first port used `rotate_left`; the long
strings failed the test until it was `rotate_right`. `Heap::intern`
computes the hash when it creates a string; the interner map stores the
handle, so a hit returns it. `StrObj` holds no copy. The one handle made
without interning is the memory error message (arena slot 0): it is the
constant `MEMORY_ERROR_MESSAGE`, whose hash is computed at compile time,
and `with_limits_bare` checks the first intern produced it. No site needed
a handle from a bare arena index.

**`hash_value` (3.2).** Strings read the handle's hash; numbers are C++
`hashnum` (sign bit masked, the MurmurHash64B finalizer, now shared with
`hashint`); booleans are the boolean (the old hash was `b + 1`); vectors
are `hashvec`. The design's "reducer per key type" did not apply: in this
Luau every reducer is `lmod(h, sizenode)`, a mask (`hashpow2`, and the
same mask inside `hashnum`, `hashint`, `hashvec`, `hashpointer`; there is
no `hashmod`), so `main_position` keeps one mask. Object and light
userdata keys keep the crate's id-based hash; the module comment says why.

**`-0` keys.** C++ stores the key it is given, so `t[-0] = v` iterates as
`-0`. The crate normalized it to `0` (so that its own hash would put both
zeros in one node); `hash_num` masks the sign, so `normalize_key` is gone.

**Dead keys (audit item 4; not in the design).** C++ `traversetable` and
`cleartable` call `removeentry` on an emptied node, which retags a
collectable key (strings included) `LUA_TDEADKEY`. A lookup compares the
tag, so it no longer finds the node; storing the key again then takes a
free node instead of reusing the old one, and the order changes. `next`
still accepts the key as a cursor (`findindex`). `Node` gets `dead: bool`
(it fits in the node's padding; `NODE_BYTES` is unchanged), set by
`Table::trace` and `clear_entries` for collectable keys and cleared when
`insert_node` writes a key; `find_node` and `find_str` skip dead nodes;
`cursor_after` walks the chain without that check. A table weak in both
keys and values is left alone: `traversetable` returns before its node
loop (review round 1). `try_clone` copies the flag, as `luaH_clone` copies
the tag.

**The `s`-mode shrink (audit item 6; not in the design).** C++ `cleartable`
resizes a weak table whose mode contains `s` to its live count when fewer
than 3/8 of the nodes hold a value (`tableresizeprotected` →
`luaH_resizehash`). The crate never shrank. `Table::shrink_hash` ports it;
`clear_weak_tables` reads the mode before taking the table out of its
slot, since a table can be its own metatable. The new nodes are checked
against the budget first, and a refusal skips the shrink, as C++ ignores
the failed allocation (review round 1); an empty shrink allocates nothing
and is never refused (review round 2).

**Library registration order (audit item 8).** `string`, `table`, `math`
and `coroutine` registered in a different order from C++'s `luaL_Reg`
arrays; same keys, different collision outcomes. They now follow the
arrays, with the same size hints: `string.pack`, `packsize` and `unpack`
join the `string` list (they were added by a second call, so the table
was sized for 14 and grew), and `coroutine.resume` is set after the
7-entry register, as `luaopen_coroutine` does. `strpack::open` is gone.
`_G` and the other libraries already agreed.

## 3. The corpus

`tools/luau-vm-diff/gen_pairs_order_cases.py`, seed 20261113, writes
`tests/luau-vm-diff/pairs_order_gen_00.luau` to `_39.luau`: 15 tables each,
600 in all, 2,400 table-profile runs. Each table draws keys of one kind
(strings of length 0 to 200, a quarter of them 32 or longer; numbers:
small and large integers, fractions, negatives, `-0`, `2^53`, `1e300`,
`huge`, subnormals; booleans; vectors with `-0` components) or a mix,
creates the table one of seven ways (empty, a literal, a constant
literal for the `DUPTABLE` template, a hint-sized shape, `table.clone`,
`table.create`, a weak table in `v`, `vs`, `kv` or `kvs` mode), then
applies 3 to 9 steps (inserts, deletes, delete-and-reinsert, integer keys
that move between the parts, `table.clear` and refill, `table.clone`,
`table.move` into the hash part, `table.freeze` round trips, weak
collections), printing the `pairs` order and `next` from each key after
every step, and `next` from each deleted key. In all: 376 deletes, 311
clears, 1,159 clones, 386 moves, 373 freezes, 32 weak tables.

Two rules keep the cases deterministic on both VMs; both are C++
behaviour the safe VM matches, not differences between them:

- **Every delete is followed by `collectgarbage()`.** After the dead-key
  port, where a deleted key goes when it is stored again depends on
  whether a collection traversed the table in between, in C++ as here.
  The two collectors pace differently, so without the collect 17 tables
  disagreed for that reason alone.
- **Weak-table values are made in a callee, and its stack slots
  overwritten before each collection.** C++ marks a running frame's whole
  register window (`ci->top` is `base + maxstacksize`), stale slots
  included, and a callee's frame starts inside the caller's window, so
  C++ kept entries the safe VM collected. Which objects survive is the
  collector's business, not the table's.

`tools/luau-vm-diff/count_pairs_order.py` splits `--verbose` output by
table label and counts disagreeing (table, profile) pairs.

| Corpus | VM | Tables disagreeing (of 600) | Table-profile pairs (of 2,400) |
|---|---|---:|---:|
| first cut (`81d184a0f5`) | `origin/main` | 541 | 2,164 |
| first cut | + the handle and `hash_value` (`31a924f671`) | 13 | 45 |
| weak values in a callee | + `-0` kept | 4 | 11 |
| same, values on a finished coroutine | + dead keys | 17 | 46 |
| final (collect after delete, scrub) | + dead keys | 2 | 8 |
| final | + the `s`-mode shrink (`53f708e7ab`) | 0 | 0 |
| final | `origin/main`, measured again at the end | 542 | 2,168 |
| final | the tip | 0 | 0 |

The 59 tables that agreed before the work are empty or single-key
tables, and tables whose keys all sit in the array part.

Named cases (all agree under all 4 profiles at the tip; all 4 fail on
`origin/main`):

- `pairs_order_lane7.luau`: lane 7's reproduction.
- `pairs_order_dead_key.luau`: a key outside its main position deleted and
  stored again, without and with a collection, in plain, `v` and `kv`
  tables.
- `pairs_order_audit.luau`: one block per audit item (section 5).
- `pairs_order_paint.luau`: Paint definition key sets.

## 4. What did not survive the design

1. **"The fix is the hash function and nothing else in the layout"**
   (design section 2). With the hash ported, 13 tables still disagreed.
   Reduced (stop rule, section 6), the first was 12 lines: integer keys
   12 to 32 in the hash part, two strings, delete one, `collectgarbage()`,
   store it again; the vendored VM put the key in a new node. The
   structural difference was C++'s `removeentry` (section 2). The second
   was the `s`-mode shrink. Both are ports of C++ behaviour, not order
   patches.
2. **"`main_position` must apply the reducer C++ uses for that key
   type"** (3.2). There is one reducer, a mask (section 2).
3. **The host API's capacities (item 7).** `luaur-rt` has no
   `create_table_with_capacity`, so the differential host surface creates
   tables at (0, 0) on both VMs. `Vm::create_table_with_capacity` calls the
   same `Table::new` that `NEWTABLE`'s size hint does, which the corpus and
   item 1 cover; the host path at other capacities has no differential.
4. **Order now depends on collector timing**, as in C++. Product content
   that deletes a string key and stores it again, with a collection
   somewhere between, can still see a different order on the two VMs,
   because their collectors pace differently. No known fixture does that.

## 5. The audit (design 3.3)

Each item has a block in `pairs_order_audit.luau` unless noted.

| Item | Path | Result |
|---|---|---|
| 1 | `NEWTABLE` with a hash hint; `DUPTABLE` (template copied, then extended) | agreed once the hash was ported; no change |
| 2 | `table.create`, `clone` (`luaH_clone` copies nodes and `lastfree`), `clear` (capacity kept, `lastfree` reset), `move` into the hash part, `freeze` | agreed; no change |
| 3 | full hash part: `rehash` sizes, reverse re-insert, array/hash migration, array shrink | agreed; no change. One edge noted, not reachable from the corpus: C++ `resize` re-inserts a shrunk array tail with `newkey`, which can rehash again if the key is `sizearray + 1`; the crate's `array_or_newkey` does not. `adjustasize` keeps that key out of the hash part, so no case reached it |
| 4 | delete, then the same key and a different key | **differed**: dead keys (section 2); `pairs_order_dead_key.luau` |
| 5 | collisions: collider out of its main position moves; in it, the new key takes the free node | agreed; no change (Brent's variation was already C++'s) |
| 6 | weak tables after a collection, `s` mode | **differed**: the shrink (section 2) |
| 7 | host API: create, `raw_set`, host `pairs` | agreed; two host-surface globals (`hosttable`, `hostkeys`) make it a differential; capacities not covered (section 4) |
| 8 | `pairs(_G)` and the library tables | **differed** in insertion order only (section 2); `_G` agreed |

## 6. Checks on the tip

1-minute load averages are in the logs below each run; the host ran other
sessions' builds throughout (load 10 to 150).

| Check | Result |
|---|---|
| `make luau-vm-ratchets` | pass (unsafe count, panic lints, conformance ratchet) |
| `make luau-vm-diff` | 405 runs, 0 disagreements, every admitted opcode reached (before the round 1 and 2 fixes; after them the 176 `pairs_order` runs and codex's own full run, 409, agree) |
| `make luau-conformance-nuxie` | 55 scripts, 37 OK; required not passing 0; PASS (before `tables.luau` was promoted; it was OK) |
| `make luau-conformance` | 55 scripts, 30 OK; PASS |
| `tools/bazel/generate.py --check` | pass |
| `runtime.py test -p nuxie-luau-vm` | 74 passed (76 after the review rounds' unit tests, run with cargo) |
| `runtime.py clippy -p nuxie-luau-vm` | pass |
| `runtime.py test -p luau-vm-diff` | 57 passed |
| `nuxie-scripting`, default and `--features nuxie-vm` | 40 of 40 each |
| scripted golden, `data_bind_blob_test`, `scripting-nuxie-vm` runner | exact (5 segments) |
| scripted golden, whole corpus, `scripting-nuxie-vm` runner | 364 entries, 348 exact, 16 diverges, 1 failure (`semantic_text_inference_lt1`, known red on main) |
| `codex review --base origin/main` | round 1: 2 findings, both fixed; round 2: 1, fixed; round 3: none |

The hash-order conformance sites, run extracted (`basic.luau` 249-250 and
309-324, `tables.luau` 797-821, verbatim, as one script): OK on both VMs.

The golden runner was built with `runtime.py build -p rust-golden-runner
--features scripting-nuxie-vm` as lane 7 did, not through the guarded
`make scripted-rust-golden-runner`: the guard's variants (`ordinary`,
`scripted`) take no feature. The C++ runner and fixtures came from `make
fixtures scripted-golden-runner CPP_CONFIG=release` at the `cd5351c3` pin.
The single-entry compare used `golden-compare --id data_bind_blob_test`
with the flags of `make scripted-golden-compare`.

## 7. Instruction table (report only)

`make luau-vm-perf-report LUAU_VM_PERF_UPDATE=1` at `0c4a65e334`, 3
passes, fat-LTO; the table is committed. Geomean after/before 1.0022 over
89 benches (12 fewer, 20 more, 57 inside the noise). Moved more than 3%:

| Bench | Change | Reason |
|---|---:|---|
| `string: reverse/upper/lower (unique)` | +4.88% | Every result is a new string, and interning now runs `luaS_hash` over its bytes once at creation (3.1's cost, paid where strings are made) |
| `Pcall: call return` | +3.18% | The loop calls a global, so each iteration is a string-keyed lookup in `_G`: the main position now comes from the Luau hash, the walk checks `dead`, and the derived `StrId` equality compares both fields. About 16 instructions per iteration |
| `TableInsertion: table.insert index` | -3.38% | Not from this work's logic: the bench is dominated by `table.insert`'s element shift, whose code moved when `libs/table.rs` was recompiled with the new registration order. Not investigated further |
| `TableRemoval: table.remove` | -3.69% | Same as the line above (100 removes from the front of 25,000 elements) |

`OOP: method access` (+2.94%) is under the line but the same mechanism as
`Pcall: call return`: string-keyed lookups in tables whose layout the new
hash changed.

## 8. Not verified

- Product goldens and `nuxie`'s own tests under `scripting-nuxie-vm`
  (`upstream_scripted_path::path_drawing_examples` and
  `paths_with_opacity_applied`, which the cutover-prep session found
  follow Paint key order). The key sets are a diff case and agree; the
  tests themselves were not run here.
- The host API at non-zero capacities against C++ (section 4, item 3).
- Placement under incremental collection between a delete and a store
  (section 4, item 4): only full collections are compared.
- `make luau-vm-diff`, conformance and the Bazel test and clippy runs
  were not repeated after the two review fixes (`18352d8009`,
  `0c4a65e334`); codex ran the crate tests and the full differential after
  them, and the `pairs_order` cases and new unit tests were rerun by hand.
