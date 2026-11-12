# Safe Luau VM: `pairs` order matches the engine we ship (Phase 3, item 1)

Status: designed 2026-11-13, not built. Reopens plan decision D4's "the hash
part is free" for one reason: authored product content depends on the order
(`data_bind_blob_test`, lane 7 evidence section 9).

## 1. The problem

`pairs` over a table with string keys yields a different order on
`nuxie-luau-vm` than on the vendored VM and C++ Luau. A product fixture's
editor script shows the first attribute `pairs` yields from an XML node, so
the label reads `Fill 1` instead of `1`. This is the one known behavior
difference in product content, and it blocks the cutover.

Reproduction (lane 7): eight string keys inserted in a fixed order, then
`pairs`; 4 of 4 compiler profiles disagree with the vendored VM.

## 2. What is already the same

`table.rs` is already C++'s table in structure:

- the array part, exactly (D4);
- the hash part as a chained scatter table with Brent's variation, `lastfree`
  walking down from the end, signed `next` offsets, dead keys kept in place;
- `rehash` and `resize`, including the reverse walk over the old nodes;
- `next_from` walking the array part and then the nodes in index order.

The only thing that differs is where a key's chain starts: `hash_value`.
Today a string hashes its interned id, a number goes through splitmix64, a
boolean is `b + 1`, a vector is a splitmix64 fold.

So the fix is the hash function and nothing else in the layout.

## 3. The design

### 3.1 A string's handle carries its Luau hash

`StrId(u32)` becomes a two-field handle: the arena index and the string's
Luau hash (`luaS_hash` over its bytes, ported from
`vendor/luaur-vm-0.1.8/src/functions/lua_s_hash.rs`; no seed).

- `Heap::intern` computes the hash once, when it creates the string, and
  returns it in the handle. An interned hit returns the same handle.
- `hash_str` is a field read: no multiply, no memory access.
- Equality stays the index: two handles with the same index have the same
  hash, so the derived `PartialEq` and `Hash` remain correct. If any map is
  keyed by `StrId`, hashing both fields is fine.
- The handle grows from 4 to 8 bytes. `Value` stays 16 bytes; check with the
  existing size assertion, and add one if there is none.

This is not a cache: the hash is a fixed property of an immutable string,
set at creation and never invalidated, and the handle is the only place it
lives (`StrObj` does not get a second copy). If some site needs a handle
from a bare arena index (a sweep, a debug path), report it; do not add a
second store without saying so.

Why the handle and not `StrObj`: the table has no access to the heap's
string store, and a lookup that had to load the string to find its hash
would add a memory access to every string-keyed get and set.

### 3.2 `hash_value` matches C++ for value-typed keys

Port, from the vendored VM's table code (`hashstr`, `hashnum`, `hashboolean`,
`hashvec`, `hashpointer`, `mainposition` and the two reducers `hashpow2` and
`hashmod`):

| Key | Main position |
|---|---|
| string | the handle's hash, reduced as C++ `hashpow2` |
| number | C++ `hashnum` (sign bit masked, the MurmurHash64B finalizer), `hashpow2` |
| integer (Rive) | already C++ `hashint`; confirm the reducer |
| boolean | C++ `hashboolean` |
| vector | C++ `hashvec` |
| table, function, thread, buffer, userdata, light userdata | unchanged |

`main_position` must apply the reducer C++ uses for that key type, not one
mask for all. Object keys hash by address in C++, so their order is not
reproducible and no content can depend on it; they keep this crate's
id-based hash. Say so in the module comment.

### 3.3 Everything else that decides node placement

The hash alone gives the same order only if every path that sizes or fills
a hash part does what C++ does. Audit each against the vendored VM and add
a differential case for each; fix what differs:

1. `NEWTABLE` with a hash size hint, and `DUPTABLE` (the template's node
   layout is copied, not rebuilt).
2. `table.create`, `table.clone`, `table.clear` (capacity kept, `lastfree`
   reset), `table.move`, `table.freeze`.
3. Insert into a full hash part: `rehash` sizes, the reverse re-insert, a
   key that migrates between the array part and the hash part.
4. Delete, then insert the same key and a different key (dead-key reuse).
5. Collision handling: the colliding node is not in its main position and
   moves; it is and the new key takes the free node.
6. Weak tables after a collection, including the shrink in `s` mode.
7. The host API: `create_table` with capacities, `raw_set`, `next`.
8. Environment and library tables as the sandbox builds them, since
   `pairs(_G)` and `pairs(string)` are observable. Match if the insertion
   order is the only difference; report if it is anything more.

## 4. How it is checked

1. **A generated order corpus in `tools/luau-vm-diff`.** A seeded generator
   emits scripts that apply random sequences of the operations in 3.3 to
   tables keyed by strings (lengths 0 to 200, so both branches of the string
   hash run), numbers (integers, fractions, negatives, large, `-0`),
   booleans and vectors, mixed; after each step the script prints the full
   `pairs` order and `next` from each key. At least 2,000 tables across the
   four compiler profiles, 0 disagreements with the vendored VM. The seed is
   fixed and committed, so the corpus is a fixed set, not a fuzzer.
2. **The lane 7 reproduction** as a named case.
3. **Conformance.** The three hash-order sites D4 filed (`basic.luau`
   249–250 and 309–323, `tables.luau` 797–821) now pass at the assertion
   level. Promote `tables.luau` to required if nothing else blocks it;
   update `scope-nuxie.txt` and its reasons either way.
4. **The product fixture.** `data_bind_blob_test` is exact under
   `scripting-nuxie-vm` in the scripted golden compare.
5. **The standing gates**: ratchets, `make luau-vm-diff`,
   `make luau-conformance-nuxie`, `make luau-conformance`, crate tests,
   clippy, `nuxie-scripting` 40 of 40 under both settings.

Speed is parked. Report the suite's instruction table before and after and
explain anything that moves by more than 3%; do not optimize.

## 5. Not in scope

- The interner's own hasher. It may be left as it is.
- Object-key order.
- Any change to the array part.
- The other Phase 3 items (goldens, size, the boundary checker).

## 6. Stop rule

If the generated corpus still disagrees after 3.2 and the audit in 3.3, do
not start patching orders case by case. Reduce one disagreement to its
smallest script, find the structural difference in `table.rs` against the
vendored `ltable`, and report it before going on.
