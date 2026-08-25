# Wave B2 semantic correction

Corrected candidate: `bbf1ce429e87deafb6cfb89610d29ddf2b66039f`

Independent rejection receipt: `943755fa4`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **CORRECTED; PENDING FRESH INDEPENDENT REVIEW**

## Correction scope

This correction addresses all 20 rows rejected by the independent semantic
review while preserving the 25 accepted rows except for evidence-locator
refreshes caused by the corrected test bodies.

- All 17 `enums_test.cpp` adaptations now use a faithful test-only
  `std::mt19937_64` implementation seeded with `0xf934929`. The exact pinned
  basic inputs, random sample counts, per-type generator resets, and the
  accidental `Flags64` `decr` branch are retained. Typed Rust flag wrappers
  are compared against the corresponding integral oracle; the prior raw
  integer identities and xorshift substitute are no longer evidence.
- `elastic_easing_test.cpp#2` retains exact equality for the two actual
  amplitude checks and uses the pinned Catch `Approx` expected-magnitude-scaled
  epsilon rule for the three easing checks.
- `file_test.cpp#6` observes `graphOrder` on the retained Artboard runtime
  component owner, asserts it is zero, and preserves all five relative owner
  ordering checks.
- `file_test.cpp#9` finds every pinned `PointsPath`, performs the production
  PATH-dirt action on each retained owner between the two update passes, and
  verifies that the dirt reached each owner.

The correction changes executable tests and evidence only. The edit in
`animation.rs` is confined to its `#[cfg(test)]` module; no production runtime
behavior is modified.

## Census

- identities: 45/45 pinned cases;
- proof mechanism: 28 direct and 17 `cxx-language-only` adapted;
- declared outcomes: 31 pass and 14 executable expected-red;
- corrected rows: 20/20;
- accepted rows changed semantically: 0/25.

## Validation

- All 31 passing evidence rows executed successfully: 17 enum tests, eight
  exact tools-feature `cpp_probe` tests, three exact `nuxie` File tests, the
  distance and draw-order integration tests, and the internal elastic owner
  test.
- All 14 expected-red rows were forced individually. Each selected exactly one
  test and failed at its documented concrete boundary: 11 absent KTX2/BC7
  decoder owners, absent `File::stripAssets`, absent retained ScriptAsset
  verification state, and the deterministic stream difference at frame 0,
  operation 25.
- Strict pinned identity, source-line/name, evidence-locator, executable-test,
  exact ignore-reason, and declared-census validation: 45/45 green.
- Repository correspondence checker: 157 files and 1,404 pinned `TEST_CASE`s,
  green. The independent main case ledger remains unchanged and pending.
- Correspondence checker unit suite: 24/24 green.
- Scoped `git diff --check` and JSON parsing: green.

This receipt records a corrected candidate, not acceptance. A fresh independent
semantic review must adjudicate the 45 rows.
