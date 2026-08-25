# Wave B2 executable test port

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: the 45 cases in runtime test files 8-14, from
`decode_ktx2_test.cpp` through `file_test.cpp`.

Verdict: **PORTED; PENDING INDEPENDENT REVIEW**

## Census

- 45/45 pinned identities match by path, ordinal, source line, and exact case
  name.
- Proof mechanism: 28 direct and 17 `cxx-language-only` adaptations.
- Outcome: 31 pass and 14 executable expected-red.
- Incomplete: zero pending, unverified, source-anchor-only, or raw-C++-string
  rows.

The enum adaptations retain Rust's native integer bitmask owner instead of
recreating C++ enum-class and operator-overload syntax. All runtime-observable
value checks execute, including the pinned `Flags64` branch that accidentally
calls `decr`.

The 14 expected-red cases are the 11 complete KTX2 stream/action ports that
reach the absent production KTX2/BC7 decoder owner, the absent
`File::stripAssets` owner, the discarded ScriptAsset verification state, and
the deterministic-mode retained-stream difference at frame 0, operation 25.

## New executable coverage

The previously generic draw-order corpus citation is replaced by a direct
test. It imports `draw_rule_cycle.riv`, finds `Blue` as a `Shape`, proves the
single animation, and performs the pinned ten one-second advance/apply/draw
iterations.

The deterministic-mode row now has a dedicated ignored-red Silver test. It
runs the manifest's complete pinned view-model, pointer, advance, frame, and
draw sequence before comparing the generated stream with the pinned `.sriv`.

All other B2 rows point to executable ports that predate and are independent
of the rejected raw-string Wave B file. No row cites
`upstream_wave_b_expected_red.rs`.

## Evidence runs

- Strict shard identity and locator check: 45/45; every locator resolves at
  its declared line and symbol, and every red reason exactly matches its
  test's `#[ignore]` reason.
- Passing evidence: 31/31. This includes 17 enum tests; eight exact
  tools-feature `cpp_probe` tests; the three `nuxie` File facade tests; direct
  distance and draw-order tests; and the internal elastic numeric owner test.
- Expected-red sweep: 14/14. Every forced invocation selected exactly one
  test and failed at its documented owner or retained-stream boundary.
- Repository correspondence: 157 files and 1,404 pinned `TEST_CASE`s, green.
- Correspondence checker unit suite: 24/24 green.
- `git diff --check`: green for the three B2 implementation artifacts and
  this receipt.

This port changes no production runtime source and does not promote the main
1,404-case ledger.
