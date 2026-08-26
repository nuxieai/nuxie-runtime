# Wave C4 independent adversarial review

Status: **REJECTED; SIMPLEARRAY CASE 1 USES BACKING-VEC PROXIES**

Reviewed frozen candidate
`adecccd73a9534f5a99669bbd67322c7d79ea386` against pinned upstream
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. This is a receipt-only
review. It changes no candidate test, ledger row, expectation, owner, fixture,
or runtime behavior and does not accept Wave C4.

## Verdict

The candidate has the exact 52-case denominator: SIMD 23, SimpleArray 13,
Span three, RefCnt two, and type conversions 11. Its ledger is mechanically
well formed as four direct passes, 27 structured `cxx-language-only` adapted
passes, and 21 pending/unverified rows. All 31 declared executable locators
run green.

One declared direct row is not admissible owner evidence. The exact semantic
result is therefore 30 accepted executable rows, one rejected executable row,
and 21 honest pending rows. No expected-red or hidden red is involved.

## Blocking finding: SimpleArray case 1

Row `tests/unit_tests/runtime/simple_array_test.cpp#1`, `array initializes as
expected`, is declared direct. The pinned body asserts, in order:

1. `array.empty()`;
2. `array.size() == 0`;
3. `array.size_bytes() == 0`;
4. `array.begin() == array.end()`.

The Rust evidence at
`crates/nuxie-renderer/src/mechanical_port/source/src/renderer_cpp/wave_c4_core_utility_tests.rs:4`
preserves that order. Its first two assertions call the retained owner's
`is_empty()` and `len()` methods. Its last two assertions, however, are:

```rust
assert_eq!(array.values.len() * core::mem::size_of::<i32>(), 0);
assert_eq!(
    array.values.as_ptr_range().start,
    array.values.as_ptr_range().end
);
```

Those are backing-`Vec` size and pointer-range proxies, not observations of
the retained `SimpleArray` owner. The retained owner at
`crates/nuxie-renderer/src/mechanical_port/source/src/renderer_cpp.rs:61`
stores a private `values: Vec<T>` and exposes `new`, `add`, `len`, `is_empty`,
`back`, iteration, and indexing. It exposes no `size_bytes`, `begin`, or `end`
observable. Module privacy lets this sibling test inspect the backing field;
it does not turn backing-container calculations into direct owner authority.
This is exactly the proxy form excluded by the frozen Wave C4 contract.

The narrow correction is to classify SimpleArray case 1 as honest
pending/unverified with empty evidence, no note, no adaptation, and no locator,
unless a separately authorized retained owner later exposes all four pinned
observables directly. Do not replace the current expressions with another
capacity, length, allocation, or pointer proxy. Preserve every other pinned
row and its existing order.

## Audited remainder

- The 16 accepted SIMD rows preserve their case-local primitive streams and
  distinct assertion order. The scalar `fast_acos` evidence is owner-local.
  The unavailable swizzle/assignment, generic IEEE, ternary, special
  min/max/clamp, reduce, `div255`, and mix/precision owners remain honest
  pending; no local helper algorithm substitutes for them.
- SimpleArray cases 2-13 remain honest pending because the retained owner does
  not expose their constructor, allocator, capacity, byte-size, pointer,
  move, overflow, OOM, or null-allocation observables. No executable evidence
  is claimed for them.
- Span case 1 remains honestly pending. Cases 2 and 3 retain their literal
  container-conversion action order and iteration assertion stream through
  the mechanical owner.
- Both RefCnt rows preserve the null, retain/release, copy, reset, converting
  construction, converting move, equality, field, and count stream. The
  inheritance/converting-move representation is limited to the declared C++
  language adaptation and calls the retained `rcp` owner.
- All 11 type-conversion rows preserve operand widths, boundary values,
  success/overflow disposition, aliasing order, and exact result streams via
  Rust primitive checked/overflowing multiplication. No candidate-local
  multiplication helper is present.

All 31 executable evidence locations are distinct. Every one of the 21
pending rows has empty evidence and contains no note, adaptation, or locator.

## Gates

- Focused non-incremental evidence sweep: 31 passed, zero failed, zero ignored.
- Strict Wave C4 shard: 52 cases; four direct, 27 adapted, 21 pending; 31 pass
  and 21 unverified; all evidence locators resolved.
- Repository correspondence: 157 files / 1,404 pinned cases, green.
- Correspondence-checker unit suite: 24/24 green.
- Pinned checkout SHA and all five source SHA-256 identities: exact; pinned
  source worktree clean.
- Candidate JSON parsing, strict pending/adaptation shape, distinct evidence
  locations, candidate-scoped source scan, and diff whitespace checks: green.
- Default release builds for `nuxie-binary`, `nuxie-runtime`, and
  `nuxie-renderer` are green; their LLVM IR contains no Wave C4 test,
  expected-red, or pending-helper symbol.

Every relied-on Cargo invocation disabled incremental compilation.
