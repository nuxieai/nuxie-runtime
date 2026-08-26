# Wave C4 final independent adversarial rereview

Status: **ACCEPTED**

Reviewed frozen candidate `adecccd73a9534f5a99669bbd67322c7d79ea386`,
independent rejection `53064ea2f`, and correction `6bb5d7528` against pinned
upstream `4ac7b32798da0482e441ef09304dc3b480ed3ee5`. This receipt changes no
test, ledger row, expectation, owner, fixture, or runtime behavior.

## Verdict

The correction is exact and minimal. Wave C4 now has the required 52-case
denominator and final topology:

- three direct passes;
- 27 structured `cxx-language-only` adapted passes;
- 22 strict pending/unverified rows;
- 30 distinct executable evidence locators.

There are no rejected executable rows and no expected-red rows.

## Correction audit

`tests/unit_tests/runtime/simple_array_test.cpp#1` is no longer executable
evidence. Its sibling-module backing-`Vec` proxy test is absent, and the row is
strict pending/unverified with empty evidence, no note, and no adaptation.
Neither the deleted test symbol nor its private backing-container expressions
remain mapped or present.

The only consequential ledger changes beyond that row are the mechanically
required Span locator shifts caused by deleting the preceding test: Span case
2 moved from line 19 to line 7 and Span case 3 from line 33 to line 21. The
other 49 ledger rows are byte-for-byte identical to the frozen candidate, and
the correction ratchet moved only from 21 to 22 pending rows.

## Full evidence audit

- The 15 executable SIMD rows preserve every pinned value, lane order,
  operation, loop, and assertion. Fourteen are explicit C++ SIMD-language
  adaptations over primitive/array operations. `fast_acos` is direct owner
  evidence and preserves the boundary checks, eight starting lanes, ten
  Newton iterations, per-lane approximation checks, derivative convergence,
  and six known-root checks through the retained production formula.
- The eight unavailable SIMD owners remain strict pending; no swizzle,
  IEEE/generic-vector, ternary, min/max/clamp, reduce, `div255`, or mix proxy is
  substituted.
- All 13 SimpleArray rows are now strict pending because the retained owner
  lacks the complete pinned constructor/allocation/capacity/byte-size/pointer/
  move/failure observables. No backing-storage calculation is accepted as
  owner evidence.
- Span case 1 remains pending. Span cases 2 and 3 preserve the pinned
  container-conversion and iteration streams in their original order.
- Both RefCnt rows preserve the complete null, ownership-count, copy, reset,
  conversion, move, equality, and field assertion streams; only the declared
  C++ inheritance/converting-move representation is adapted.
- All 11 type-conversion rows preserve exact operand widths, boundary values,
  success/overflow results, aliasing order, and products. Their structured
  adaptations identify only the unavailable C++ output-parameter/template
  route.

Every adapted row has structured `kind`, `rationale`, and literal
`inapplicable_observable` fields. Every pending row has empty evidence and no
note or adaptation.

## Gates

- Focused non-incremental evidence sweep: 30 passed, zero failed, zero ignored
  (SIMD 14, type conversions 11, RefCnt/Span/fast-acos 5).
- Strict Wave C4 shard: 52/52 rows resolved as three direct pass, 27 adapted
  pass, and 22 pending/unverified; all 30 evidence locators are distinct and
  resolve.
- Repository correspondence: 157 files / 1,404 pinned cases, green.
- Correspondence-checker unit suite: 24/24 green.
- Pinned upstream HEAD and all five tracked source SHA-256 identities match the
  frozen candidate. The shared upstream checkout contains unrelated untracked
  dependency/build outputs; no tracked pinned source differs.
- JSON parsing, strict pending/adaptation schema, source identity, correction
  diff, whitespace, and forbidden proxy/symbol scans are green.
- Default non-incremental release LLVM IR builds for `nuxie-binary`,
  `nuxie-runtime`, and Vulkan-enabled `nuxie-renderer` are green. Five emitted
  library IR artifacts contain no Wave C4 test, deleted SimpleArray proxy,
  expected-red, or pending-helper symbol.

All relied-on Cargo invocations disabled incremental compilation.
