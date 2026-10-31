# D4 table layout: what the conformance corpus pins (2026-10-31)

Decision D4 in `docs/luau-safe-vm-plan.md` asks whether a new Luau VM must
copy C++ Luau's array/hash split, its size hints and its rehash policy
exactly, or whether it can choose its own layout.

**Answer.** It has to copy the array part. The sizing policy, the
`table.create` and `table.clear` capacity rules, and `luaH_getn`'s exact
binary search are pinned by 11 assertion sites. The `aboundary` cache is not
pinned. The hash part is free, apart from 3 assertion sites that pin
traversal order through string keys. All 3 sites exist on purpose, for
"backwards compat" with scripts that rely on order. Every other
layout-adjacent assertion in the corpus passes under all nine alternative
layouts tested below.

Reference: luau-lang/luau `3fc82b10` (tag 0.734), `VM/src/ltable.cpp`,
`ltablib.cpp`, `lvmexecute.cpp`. Line numbers in this document refer to
that tree. The vendored engine (`vendor/luaur-vm-0.1.8`) transliterates the
same functions (`computesizes.rs`, `adjustasize.rs`, `lua_h_getn.rs`,
`updateaboundary.rs`, `mainposition.rs`), so the product inherits these
rules exactly today.

## 1. The C++ rules

**Two parts.** A table has an array part (`array[0..sizearray)`, which
holds keys 1..sizearray) and a hash part (`node[0..2^lsizenode)`). Integer
keys outside the array part live in the hash part. `sizearray` is both the
capacity and the logical array length, and a `nil` slot inside it is a hole.

**Boundary invariant.** The header comment at ltable.cpp:19-23 states that
the border that `#` returns must lie inside the array part. In practice this
means `t[sizearray+1]` is nil whenever the last array slot is non-nil. Three
things enforce it:

- `newkey` (ltable.cpp:617-626): writing key `sizearray+1` always triggers a
  rehash, even when the value written is nil.
- `adjustasize` (500-508): after a resize, the array size grows while
  `t[size+1]` is non-nil, or while `size+1` is the key being inserted.
- `luaH_resizearray`, used by SETLIST and `table.move`, runs through
  `adjustasize`.

**Rehash triggers.** A rehash runs in two cases:

1. The key `sizearray+1` is inserted.
2. `getfreepos` cannot find a free node (`newkey`, 628-638). `lastfree` only
   moves downward, and dead entries (nil value, key retained) still occupy
   their nodes until the next rehash. A rehash can therefore fire before
   every node is in use, and *when* it fires depends on node placement.

**Sizing (`rehash`, 522-561).** The rehash proceeds in five steps:

1. Count the non-nil integer keys in each power-of-two slice `(2^(i-1), 2^i]`.
   `numusearray` counts the array part, `numusehash` counts the hash part,
   and `countint` adds the key being inserted.
2. `computesizes` (308-332) picks the largest power of two `n` such that
   more than `n/2` of the slots `1..n` would be in use.
3. `adjustasize` raises that size to restore the boundary invariant.
4. If `adjustasize` added `aextra` slots, the array gets `aextra` more on top
   of that, and `adjustasize` runs again. The comment explains that hash
   nodes are twice the size of array slots.
5. The hash part becomes `2^ceil(log2(totaluse - na))` nodes, where `na` is
   the number of keys that go to the array part. `resize` (449-498) moves
   array elements beyond the new array size into the hash part. It then
   re-inserts the old nodes from the highest index down, through
   `arrayornewkey`.

Both parts are capped at `MAXSIZE = 2^26`. Anything larger raises
"table overflow".

**Size hints.**

- `NEWTABLE` (lvmexecute.cpp:2475) passes the compiler's array count `aux`
  and hash size `1 << (B-1)` to `luaH_new`.
- `DUPTABLE` clones a constant-key template with `luaH_clone` (a memcpy, so
  the copy has the same node order). The template is built in `lvmload` by
  `luaH_new(L, 0, keys)` followed by inserting the keys in source order.
- `table.create(n[, v])` (ltablib.cpp:571-596) allocates an array part of
  exactly `n` slots and fills it with `v` if one is given.
- `SETLIST` resizes the array part only when the list extends past
  `sizearray`.

**Hash placement.**

- Each key type has its own hash: `luaS_hash` for strings, a Murmur finalizer
  for numbers and pointers, and a spatial hash for vectors. The hash is
  reduced mod `2^k` (`hashpow2`, 60).
- Collisions are resolved by Lua's chained scatter table with Brent's
  variation (`newkey`, 628-671). A colliding node is moved out only if it is
  not in its own main position. Free nodes come from `lastfree`, scanning
  downward.
- Keys that are GC objects hash by address. Their traversal order is
  therefore allocator-dependent in C++ itself.

**`next` and `pairs` order (`luaH_next`, 270-292).** Traversal visits the
array part in index order, then the nodes in node-index order, and skips nil
values. `findindex` (242-268) locates the current key through its main
position chain and accepts a dead key, so a key deleted during traversal,
or even one already collected, still continues the traversal. The other
traversal paths walk the same order:

- `lua_rawiter` (lapi.cpp:1501-1530)
- the `FORGLOOP` fast path (lvmexecute.cpp:2722-2770)
- `table.foreach`, `maxn`, `clear` and `clone`

**`#` (`luaH_getn`, 859-898).** Every `#` goes through this function: the
`LENGTH` opcode, `rawlen`, `lua_objlen`, `table.insert`, `table.remove`,
`table.concat`, `table.unpack`, `table.sort`, and the builtin fastcalls.

- The function only reads the array part. The boundary invariant makes the
  hash part irrelevant.
- If the last array slot is non-nil, the result is `sizearray`.
- Otherwise a Khuong-Morin "branchless" binary search runs over the array
  (`base` moves right whenever `base[half]` is non-nil). This finds a border
  near the right end. The classic Lua 5.1 search can return a different
  border.
- A cached `aboundary` (stored negated, in a union with `lastfree`,
  lobject.h:547-548) short-circuits repeated calls via `updateaboundary`.
  This makes `#` history-dependent in principle.

**Library behavior.**

- `table.clear` (937-963) nils every slot but keeps both part sizes.
- `table.insert` and `table.remove` (ltablib.cpp:219-270) raise no range
  errors in Luau. An out-of-range `insert` position is a plain `t[pos] = v`,
  and an out-of-range `remove` returns nothing, so their boundary behavior
  is entirely the behavior of `#`.
- `table.sort` (550-569) sorts `array[0..#t)` in place. It detects a
  comparator that modifies the table by comparing `sizearray` before and
  after each comparator call ("table modified during sorting").

## 2. Method

Each step below is evidence. None of it is an argument from the
specification.

1. **Mutate the reference VM.** I copied the pinned C++ tree to the session
   scratchpad and added environment knobs to `ltable.cpp`, `ltablib.cpp` and
   `lvmexecute.cpp`; appendix A has the patch. With every knob at 0 the
   behavior is upstream's. Each knob replaces one rule with a different
   *valid* rule: `#` still returns a border, iteration still visits every
   key once, and nothing is unsafe.
2. **Run the whole suite under each knob.** That is `Luau.Conformance`: all
   55 `.luau` scripts plus the C++ API tests in `Conformance.test.cpp`, 305
   test cases and 6417 doctest assertions. The baseline passes 305/305.
3. **Make the asserts soft.** To list *every* failing assertion rather than
   the first one per script, a copy of the corpus gets a one-line prefix,
   placed on line 1 so line numbers are unchanged. The prefix shadows
   `assert` with a version that logs `D4SOFT <chunk> <line>` instead of
   raising. A baseline soft run records the noise: asserts that expect a
   failure, and debugger tests that see the extra local. Results below are
   the difference from that baseline.
4. **Neutralize non-assert errors.** Four `table.concat` calls that raised
   errors under one knob were wrapped in `pcall` on the same line, so the
   run could continue past them: tables.luau:383, 385, 387 and 472.
5. **Prototype a safe Rust table.** A scratch crate (section 6) uses a plain
   `Vec` array part, ports the sizing and `#` rules, and uses an
   insertion-ordered hash part with its own growth trigger. It replays the
   pinned assertions.

| Knob | Alternative rule it models |
|---|---|
| `D4_HASH=1` | a different hash function (remix and salt every hash before `mod 2^k`; the `NAMECALL` main-position probe was patched to match) |
| `D4_HASH=2` | same hash, node vector twice as large (different load factor, positions and rehash timing) |
| `D4_GETN=1` | `#` = smallest border (first nil from 1): what a dense `Vec` with "length = first hole" gives |
| `D4_GETN=2` | `#` = largest border over all integer keys, array and hash (a "max key, trailing nils trimmed" design) |
| `D4_GETN=3` | `#` = Lua 5.1 binary search over the array part (`i=0, j=sizearray`) |
| `D4_GETN=4` | upstream `luaH_getn` with the `aboundary` cache disabled |
| `D4_SIZE=1` | `computesizes` disabled: the array part is only the contiguous prefix, with the `aextra` doubling (a `Vec` that grows by doubling) |
| `D4_SIZE=2` | array-size hints ignored (`NEWTABLE aux`, and `table.create(n)` without a fill value): lazy `Vec::new()` instead of `with_capacity` semantics |
| `D4_SIZE=3` | `computesizes` disabled and no doubling: the array part is exactly the contiguous prefix |

## 3. Results per knob

The sites that fail under each knob, compared with the baseline:

| Knob | Failing sites |
|---|---|
| `D4_HASH=1` | basic.luau:249, 250, 309; tables.luau:809, 810, 812, 813, 815, 816 |
| `D4_HASH=2` | basic.luau:309 |
| `D4_GETN=1` | tables.luau:404, 532, 534, 537, 613 |
| `D4_GETN=2` | tables.luau:385, 387, 472, 631 |
| `D4_GETN=3` | tables.luau:534, 537 |
| `D4_GETN=4` | **none** |
| `D4_SIZE=1` | clear.luau:39; tables.luau:613 |
| `D4_SIZE=2` | tables.luau:404, 768; clear.luau:39; pcall.luau:172-176 (via `checkresults`, pcall.luau:6 and 8); Conformance.test.cpp:2810-2854 (`ApiCalls`: `lua_pcall` on OOM via `largealloc`); Conformance.test.cpp:3084 (`ExceptionObject`: `large_allocation_error`) |
| `D4_SIZE=3` | sort.luau:107; tables.luau:613; `Coroutine` did not finish within 600 s (quadratic appends: a performance pathology, not a semantic failure) |

`iter.luau`, `move.luau`, `clear.luau` other than line 39, `sort.luau` other
than line 107, and every C++ API test that walks `lua_next` or `lua_rawiter`
pass under every knob.

## 4. Assertions that pin the C++ layout (class b)

Classes:

- **(a)** accepts any valid border or order.
- **(b)** pins the C++ layout.
- **(c)** pins behavior that does not depend on layout, though it may
  constrain the design.

Every (b) site found:

| Site | Assertion | What it pins | Fails under |
|---|---|---|---|
| tables.luau:404 | `table.create(5); t[5]=5; assert(#t == 5) -- magic` (borders 0 and 5) | `table.create(n)` gives an array part of exactly `n`; last slot non-nil means `#t == sizearray` | GETN=1, SIZE=2 |
| tables.luau:532 | `table.create(10,1); t[5]=nil; #t == 10` (borders 4, 10) | the last-slot fast path | GETN=1 |
| tables.luau:534 | then `t[10]=nil; #t == 9` (borders 4, 9) | the Khuong-Morin search shape; Lua 5.1's search returns 4 | GETN=1, GETN=3 |
| tables.luau:537 | then `t[9]=nil; t[8]=nil; #t == 7` (borders 4, 7) | same | GETN=1, GETN=3 |
| tables.luau:613 | `table.create(5,42); arr[1]=nil; arr.a='a'; #arr == 5`; the comment at 613 admits 0 is also valid | the rehash sizing: `computesizes` gives 4, `adjustasize` gives 5, `aextra` gives 6; then the search over `[nil,42,42,42,42,nil]` | GETN=1, SIZE=1, SIZE=3 |
| tables.luau:385 | `{1,2,3}` plus `a[10]=10` in the hash; `table.concat(a) == "123"` (borders 3, 10) | the border never reaches into hash-part integer keys | GETN=2 |
| tables.luau:387 | same, after `a[-1e9]=42` | same | GETN=2 |
| tables.luau:472 | `t[123]="a"; t[124]="b"; table.concat(t) == ""` (borders 0, 124) | same | GETN=2 |
| tables.luau:631 | `arr[5..2]` filled downward; `#arr == 0` after each write (borders 0, 5) | same | GETN=2 |
| clear.luau:39 | `#t1 == #t2`, where t1 is 1..16 filled then `table.clear`ed and t2 is `table.create(16)`, each with one slot set | `table.clear` keeps the array capacity; filling 1..16 one key at a time yields exactly 16 (power-of-two sizing); `table.create(16)` is exactly 16 | SIZE=1, SIZE=2 |
| sort.luau:107 | a comparator that inserts string keys must make `table.sort` fail | a hash insert during the sort changes `sizearray` (6 becomes 8 through `computesizes`). A design that detects modification some other way (a structural epoch, say) also passes | SIZE=3 |
| basic.luau:249 | `pairs({1, 2, 3, a=5, b=6, c=7})` gives values `"1,2,3,5,7,6"`; comment at 248: "a,b,c results in a,c,b during traversal since index is based on hash & size" | string hash, node count 4, and collision placement | HASH=1 |
| basic.luau:250 | `pairs({1, 2, 3, nil, 4, a=5, b=6, c=7})` gives `"1,2,3,4,5,7,6"` | same | HASH=1 |
| basic.luau:309-323 | biome table traversal equals `"ArcticDunesCanyonsWaterMountainsHillsLavaflowPlainsMarsh"`; comment at 308: "some scripts rely on exact table traversal order" | string hash, node count 16, collision placement | HASH=1, HASH=2 |
| tables.luau:809, 810, 812, 813, 815, 816 (block 797-821) | `for key, val in {foo=1, bar="string", thing=true}` visits thing, bar, foo; comment at 797-798: "tables with constant keys maintain their insertion/iteration order ... important to maintain for backwards compat" | DUPTABLE template layout. The expected order is the *reverse* of insertion order, so an insertion-ordered map (IndexMap) also fails it | HASH=1 |

### Capacity sites: they pin eager allocation, not layout

All of these fail only when `table.create(n)` stops reserving memory. A
`Vec::with_capacity(n)` design, or `try_reserve` with an explicit limit,
passes them.

| Site | Assertion | What it pins |
|---|---|---|
| tables.luau:768 | `pcall(table.create, 1e9) == false` | a size limit (C++: `MAXSIZE` 2^26 raises "table overflow"). A Rust `Vec` must use `try_reserve` against a limit, or it aborts |
| pcall.luau:172-176 | OOM raised by `table.create(1e6)` under the harness memory cap | `table.create` allocates eagerly |
| Conformance.test.cpp:2810-2854 | `lua_pcall` on OOM, through `largealloc` (apicalls.luau:25-27, `table.create(1000000)`) | same |
| Conformance.test.cpp:3084 | `large_allocation_error` raises (exceptions.luau:29) | same |

### Sites that survived every knob (classes a and c)

These are the layout-adjacent sites I read in full. All of them pass under
all nine knobs.

- **(a), unique border.** tables.luau:16-19, 169-173, 298-305, 322, 342-377,
  539-565, 573, 618-623, 639-640, 645-649; clear.luau:23-26, 58 onward;
  basic.luau:287-289 (table-literal `#` checks); move.luau.
- **(a), order-agnostic iteration.** tables.luau:205-206, 225-228, 241-245,
  255-262, 281-296, 308-316; basic.luau:243-245, 256-258; clear.luau:28-31,
  44-55; iter.luau.
- **(c), iteration contract.** Deleting the current key during `pairs` and
  running `collectgarbage()` between steps keeps the traversal valid
  (tables.luau:328-339, 725-753). `next(t)` after deleting earlier keys
  returns the current key (tables.luau:729-730). **Design constraint:**
  deleting a key must not move other entries, so no `swap_remove` and no
  compaction outside a rehash. `next` must also accept a key that is dead
  or already collected.
- **(c), array prefix iterated first in ascending order.** basic.luau:233-240
  (`"123"`, `"1235"`) and 253-255; mlua_table.rs:47-49 and 128-141;
  mlua_function.rs:51-64 (`pairs({...})` concatenated in order). These hold
  for any design where keys `1..#t` live in an array part that is visited
  first. That is the upstream comment's stated reason for the boundary
  invariant.
- **(c), other.** The `table.unpack` result limit (tables.luau:669-678). The
  arity errors of `table.insert` and `table.remove` (tables.luau:757-762).
  `table.find` reaching hash-part integer keys (tables.luau:416).

`tables.luau:22-165` (`querytab`, `int2fb`) sits under `if T then`. The
`Tables` harness (Conformance.test.cpp:1284-1312) does not define `T`, so
that block never runs and pins nothing.

`tables.luau:590-600` intends to check that `table.clone` preserves order,
but `order()` returns the undefined global `v`, so the comparison is
`nil == nil`. It is vacuous today. If upstream fixes it, it becomes a (c)
check that the clone matches its source, not that it matches C++.

### Vendored engine tests

- `vendor/luaur-vm-0.1.8/src/*_tests.rs` has no length or iteration
  assertions. `rung9_tests.rs:424` only registers the `rawlen` fastcall
  pointer.
- In `vendor/luaur-rt-0.1.8/tests`, every `len`, `raw_len`, `pairs` and
  `sequence_values` assertion is (a) or (c):
  - `mlua_table.rs`: 44, 58, 65-69, 85, 103, 114, 161-162, 176-179, 409
  - `mlua_luau.rs`: 221-225
  - `mlua_function.rs`: 51-64

  None is (b).
- `table3 = {1, 2, nil, 4, 5}` (mlua_table.rs:65-69) goes through mlua's own
  "iterate until the first nil", which does not depend on `#`.

## 5. What the product relies on

The `nuxie-scripting` and `nuxie` Rust sources, plus the Lua embedded in
`nux-capi` tests, were searched for `pairs`, `lua_next`, `raw_len`,
`sequence_values`, `lua_objlen` and `#t`.

- **Module initialization order follows hash order.**
  `crates/nuxie-scripting/src/vm/native_registration.rs:96` walks each
  module's string-keyed `deps` table with `pairs()` during the topological
  sort. The order in which modules register and load therefore follows
  hash traversal order.
  `crates/nuxie-scripting/src/vm/native_registration.rs:160-163` also
  reports `missing_dependencies` in that order. The sort follows upstream's
  recursion (see the comment at :91-92). With a free hash part, the load order of scripts
  that have two or more dependencies can change, and it becomes visible if
  a module has side effects when it loads. The fix lives on the product
  side: sort the dependency names before visiting them. That needs no
  decision from the VM.
- **HTTP header order follows hash order.**
  `crates/nuxie-scripting/src/vm/lua_scriptnet.rs:283-292` pushes
  `request.headers` in `pairs` order, so the wire order of headers follows
  hash order. This is observable but harmless (HTTP does not give header
  order a meaning). Sort here too if determinism matters.
- **Which error is reported can follow hash order.**
  - `crates/nuxie-scripting/src/gpu_canvas.rs:3354` (`reject_unknown_fields`)
    names the first unknown field in traversal order.
  - `crates/nuxie-scripting/src/host_commands.rs:486` can report "mixed or
    invalid host table keys" or a different error, depending on which key
    comes first.
  - `crates/nuxie-scripting/src/vm/lua_paint.rs:119` applies fields in
    `pairs` order, but every key maps to an independent setter, so the
    result does not depend on order.
- **Order-independent sites.** These collect into a `BTreeMap` or sort
  first:
  - `crates/nuxie-scripting/src/host_commands.rs:483-536`
  - `crates/nuxie-scripting/src/vm/lua_audio_processors.rs:684-700`
  - `crates/nuxie-scripting/src/vm/luau_audio_processor_engine.rs:1466, 1555`
  - `crates/nuxie-scripting/src/vm/lua_font.rs:98` (tag pairs go to a
    variation list)
- **`#` and `raw_len`.** Product uses expect dense lists:
  - GPU descriptor arrays in `crates/nuxie-scripting/src/gpu_canvas.rs`
  - `crates/nuxie-scripting/src/gpu_canvas_ore/pipeline.rs:118-390`, which
    includes `lua_objlen` at :265
  - `crates/nuxie-scripting/src/gpu_canvas_ore/pass.rs:112-113`
  - `crates/nuxie-scripting/src/vm/renderer.rs:67-68`
  - `crates/nuxie-scripting/src/vm/view_model.rs:2652`

  Nothing depends on which border a holey table returns, unless a script
  passes a holey list. Port the array part (section 7) and that case keeps
  C++ behavior too.
- **Scripted golden streams: not verified.** No `.luau` fixture source in
  the repository iterates with `pairs` in a way that feeds output. The
  editor-authored scripts inside the `.riv` fixtures were not audited.
  Phase 3's scripted golden compare on both VMs is the oracle. A cheap
  earlier check: add the `D4_HASH=1` remix to `mainposition` and the
  string hash probe in the vendored `luaur-vm`, then run the scripted golden
  compare and the product goldens once.

## 6. Prototype: a safe Rust table

The scratch crate (`scratchpad/d4/proto`, `#![forbid(unsafe_code)]`,
about 575 lines with the scenarios) has three pieces:

- **Array part.** A plain `Vec<Value>`. Its length is `sizearray`.
- **Hash part.** An insertion-ordered `Vec<(Key, Value)>` with a `HashMap`
  index. Dead entries keep their slot until the next rehash. The part
  rehashes only when completely full at a power-of-two capacity. That is a
  different trigger from C++, whose `lastfree` scan can fire earlier.
- **Ported functions.** `computesizes`, `numusearray`, `countint`,
  `adjustasize`, `rehash`, `resize`, `resizearray`, and `luaH_getn` without
  the `aboundary` cache.

It replays the 11 border and sizing sites and the 3 hash-order blocks from
section 4:

| Prototype mode | Border and sizing sites (11) | Hash-order sites (3 blocks) |
|---|---|---|
| ported `#` + ported sizing, free hash | **all pass** | all fail: foo,bar,thing instead of thing,bar,foo; `1,2,3,5,6,7`; biomes in source order |
| smallest-border `#` | 404, 532, 613 fail | fail |
| Lua 5.1 binary search | 534 fails | fail |
| prefix + doubling sizing | 613 and clear:39 fail | fail |

The prototype reproduces the C++ mutation results independently. The array
contract carries over unchanged to a `Vec` beside a hash part of a
different kind. The rehash trigger also differed from C++, and no
assertion in the corpus noticed.

## 7. Recommendation

The three decision options:

1. **Must match exactly.** Port the array part and the hash part.
2. **Match `#` semantics only and free the hash.** Port the array part; the
   hash part is free.
3. **Free everything.** Use a scope-file expected-fail list.

I recommend option 2, with the array contract stated precisely. Port the
whole array part, not just "an array part with some binary search":

- `computesizes`, `adjustasize` and `aextra` sizing
- the `newkey` trigger for key `sizearray+1`
- `table.create(n)` reserving exactly `n` slots eagerly, with a
  `try_reserve` limit
- `table.clear` keeping capacity
- SETLIST and `resizearray` going through `adjustasize`
- `luaH_getn`'s last-slot fast path and Khuong-Morin search

The `aboundary` cache is unobservable and is the VM's choice. All of this
is about 150 lines of safe Rust (the prototype has it) with no cost to the
hot path, and it clears all 11 border sites.

Free the hash part: the hash function, the collision scheme, node capacity
and growth. Constraints that remain:

- `next` accepts dead and collected keys.
- Deletion never moves another entry.
- Growth is geometric.
- `table.sort` raises when the comparator adds keys (any structural-change
  detection will do).

That leaves 3 expected failures: basic.luau:249-250, basic.luau:309-323 and
tables.luau:797-821. Waiving them needs the scope file to work at assertion
level, or a patched corpus copy recorded in the scope file. The format in D6
is per script, and waiving all of basic.luau and tables.luau would discard
most of the coverage.

Option 1 is possible in safe Rust: a `Vec<Node>` with `i32` next offsets
plus a port of `luaS_hash` and Brent's placement. It buys only those 3
sites, and it gives up D1's freedom in hash layout. It still cannot match
C++ order for keys that are GC objects, because C++ hashes those by
address.

Option 3 adds 11 expected failures to save code that costs nothing at run
time. It also changes `#` on holey tables in shipped scripts, so it is not
worth it.

Residual risk of option 2: rehash timing in C++ depends on hash-node
placement through `lastfree`. A different hash part can therefore resize
the array part at different moments, and `#` on a *holey* table may then
differ from C++ in programs outside the corpus. It stays a valid border
either way. The differential battery against the vendored VM will show
whether this matters. The prototype hit no case in the corpus.

## 8. Reproduction

All artifacts are in the session scratchpad, under
`/private/tmp/claude-501/-Users-levi-dev-nuxie-runtime--claude-worktrees-peaceful-varahamihira-22fa41/b0f1c1fb-3ed4-401e-b7f4-026880f76f5c/scratchpad/d4/`:

- `luau/`: the pinned tree with the knobs (a git repo whose `base` commit
  is pristine `3fc82b10` minus `.git`). In it, `tests/conformance/` holds
  the soft copy and `tests/conformance_orig/` the original corpus.
- `build/Luau.Conformance`: built with
  `cmake -DCMAKE_BUILD_TYPE=Release -DLUAU_BUILD_TESTS=ON`.
- `tools/run_soft.sh <tag>` runs every knob. `tools/diff_runs.py <tag>`
  produces the comparison with the baseline used in section 3. The final
  runs are `soft5` for all knobs except `D4_SIZE=3`, and `soft6` for
  `D4_SIZE=3` with `--test-case-exclude=Coroutine`.
- `proto/`: the Rust prototype (`cargo run`).

### Appendix A: the mutation patch (against `3fc82b10`, `-U1`)

```diff
--- a/VM/src/ltable.cpp
+++ b/VM/src/ltable.cpp
@@ -59,3 +59,13 @@
-#define hashpow2(t, n) (gnode(t, lmod((n), sizenode(t))))
+#define hashpow2(t, n) (gnode(t, lmod(d4_mix(unsigned(n)), sizenode(t))))
+#include <stdlib.h>
+static int d4_env(const char* name) { const char* v = getenv(name); return v ? atoi(v) : 0; }
+int d4_hash = d4_env("D4_HASH");
+int d4_getn = d4_env("D4_GETN");
+int d4_size = d4_env("D4_SIZE");
@@ computesizes, before "*narray = n;"
+    if (d4_size == 1 || d4_size == 3) { *narray = 0; return 0; }
@@ setnodevector, after "lsize = ceillog2(size);"
+        if (d4_hash == 2) lsize++;
@@ rehash
-        nasize = nadjusted + aextra;
+        nasize = d4_size == 3 ? nadjusted : nadjusted + aextra;
@@ luaH_getn, at entry
+    if (d4_getn == 1) { int k = 0; while (!ttisnil(luaH_getnum(t, k + 1))) k++; return k; }
+    if (d4_getn == 2) { /* last non-nil array index, then any hash int key k with t[k+1] nil, max */ }
+    if (d4_getn == 3) { /* Lua 5.1: if array[j-1] nil, binary search i=0..j=sizearray; else j */ }
-    int boundary = getaboundary(t);
+    int boundary = d4_getn == 4 ? 0 : getaboundary(t);
@@ luaH_getn, after the Khuong-Morin search
-        maybesetaboundary(t, boundary);
+        if (d4_getn != 4) maybesetaboundary(t, boundary);
--- a/VM/src/ltable.h  (appended)
+extern int d4_hash, d4_getn, d4_size;
+static inline unsigned d4_mix(unsigned h)
+{ if (d4_hash == 1) { h ^= h >> 15; h *= 0x2c1b3c6du; h ^= h >> 12; h ^= 0x5bd1e995u; } return h; }
--- a/VM/src/ltablib.cpp
@@ tsort: a GETN=2 border may lie past the array part
+    if (n > t->sizearray) luaH_resizearray(L, t, n);
@@ tcreate, no fill value
-        lua_createtable(L, size, 0);
+        lua_createtable(L, d4_size == 2 ? 0 : size, 0);
--- a/VM/src/lvmexecute.cpp
@@ NAMECALL main-position probe
-                    LuaNode* n = &h->node[tsvalue(kv)->hash & (sizenode(h) - 1)];
+                    LuaNode* n = &h->node[d4_mix(tsvalue(kv)->hash) & (sizenode(h) - 1)];
@@ NEWTABLE
-                sethvalue(L, ra, luaH_new(L, aux, b == 0 ? 0 : (1 << (b - 1))));
+                sethvalue(L, ra, luaH_new(L, d4_size == 2 ? 0 : aux, b == 0 ? 0 : (1 << (b - 1))));
```

The full unabridged patch is
`scratchpad/d4/d4-mutation-u1.patch`. The `D4_GETN=2` and `D4_GETN=3`
bodies are elided above; they are about 20 lines each.
