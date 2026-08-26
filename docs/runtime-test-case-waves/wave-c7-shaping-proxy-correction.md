# Wave C7 shaping-proxy correction candidate

Corrected candidate: `fa9fc4841c9d9c202bc890adfd07d61325dc5a6d`

Independent rejection: `d6c1813f520e608ea02b1027c04c553eff18bcd7`

Pinned upstream: `rive-runtime` `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Correction

The correction applies the rejection literally and changes no production behavior:

- RawTextInput cases 9, 11, 14, and 16 are demoted from `adapted` / `pass` to strict `pending` / `unverified` rows with empty evidence and no note or adaptation.
- The four rejected owner-local tests are deleted entirely. They are not replaced by Unicode-category movement/deletion, a shaping model, a static projection, or another adaptation.
- The pending ratchet is raised from 48 to the corrected honest census of 52.
- Only the mechanically shifted locators for the three surviving tests in the same source file are refreshed: case 10 at line 935, case 15 at line 977, and case 17 at line 1004.
- The other 54 rows and tests are frozen.

## Corrected topology

The corrected Wave C7 candidate contains **six passes, zero expected-red, and 52 pending across 58 exact identities**:

- Three direct passes: raw case 17 and text cases 2 and 3.
- Three adapted passes: raw case 1 (`cxx-language-only`) and raw cases 10 and 15 (`rust-safety`).
- Fifty-two strict pending rows with no claimed evidence.

## Validation

- Focused non-incremental execution must discover and pass exactly the four surviving `wave_c7_` owner tests plus the two distinct live-Artboard text-query tests.
- Strict Wave C7 identity, name, source line, status, outcome, adaptation, and locator validation must accept all 58 rows with direct 3, adapted 3, pending 52; pass 6, unverified 52.
- Global correspondence must remain 157 files / 1,404 pinned cases and the checker unit suite must remain 24/24.
- The five pinned source hashes, JSON, forbidden proxy-symbol scan, scoped formatting, `git diff --check`, default release LLVM IR containment, and exact-path staging must pass before commit.

The standalone historical-floor invocation reports the expected correction mismatch `case max_pending 52 regressed from historical 48`: the rejected candidate committed an invalid semantic floor of 48 before independent review demoted the four false passes. That result is reported explicitly, not represented as a green ratchet gate. The isolated corrected-shard content validator still enforces `max_pending == 52` and validates all identities, classifications, outcomes, adaptations, and live locators independently of that invalid historical floor.

This is a correction candidate only and does not self-accept Wave C7.
