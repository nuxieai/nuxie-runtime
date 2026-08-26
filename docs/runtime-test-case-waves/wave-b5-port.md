# Wave B5 executable test port

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: the 31 cases in runtime test files 46-50, from
`hittest_test.cpp` through `image_decoders_test.cpp`.

Verdict: **PORTED; PENDING INDEPENDENT REVIEW**

## Census

- 31/31 pinned identities match by source path, ordinal, line, and exact case
  name.
- Classification: 29 direct and two `rust-safety` adaptations for stable
  ImageAsset owner identities in place of raw pointer addresses.
- Outcome: 21 pass and 10 executable expected-red.
- Incomplete: zero pending, unverified, raw-C++-anchor, generic-probe, proxy,
  or source-only rows.

## Exact executable coverage

The seven non-Silver hittest cases execute the pinned HitTester geometry or
the live fixture, artboard, state-machine, input, event, and animation owner
flows. Five pass. The nested opaque-artboard flow reaches the exact 301px
boundary assertion and exposes the incorrect parent hit. The early-out flow
proves the four concrete HitComponent owners and reaches the absent retained
`TESTING` counter seam.

All 14 Silver cases execute the exact manifest action streams and compare
fresh Rust SRIV against the pinned files. Eight compare exactly. Six preserve
their first concrete operation-level mismatch as ignored expected-red tests.

The existing IK tests were re-audited against the pinned C++ action and
assertion streams. They preserve the exact fixtures, component types,
dependents, graph-order relations, target values, matrices, 0.0001 matrix
tolerance, and 1,000-iteration loop.

Both ImageAsset tests resolve each named Image referencer to a stable retained
FileAsset identity, prove exact shared/distinct ownership and byte sizes, and
perform the pinned component update and draw. The out-of-band case attaches
the exact `walle-370.png` and `eve-317.png` payloads to their authored asset
identities before drawing.

All five decoder cases read the exact pinned byte lengths. PNG, JPEG, and WebP
pass with exact dimensions and decoded-byte rules. The malformed JPEG
non-Apple branch and malformed PNG Apple branch are executable expected-red
tests at the concrete decoder-result boundary; the non-Apple PNG branch keeps
the pinned null-result assertion.

## Evidence runs

- Direct hittest target: five pass, two ignored, zero failures.
- Decoder target: three pass, two ignored, zero failures.
- IK targets: three pass, zero failures.
- ImageAsset target: two pass, zero failures.
- Silver target: eight pass, six ignored, zero failures.
- Forced-red sweep: each of the 10 ignored tests was selected alone and
  failed at its documented owner, decoder, or SRIV boundary.
- Strict B5 identity/classification/ignore-reason/locator validation: 31/31;
  29 direct, two adapted, 21 pass, 10 expected-red, zero pending.
- Repository correspondence checker: 157 files and 1,404 pinned
  `TEST_CASE`s, green.
- Correspondence checker unit suite: 24/24 green.
- Production artifact audit: no Wave B5 test symbol is present in the
  `nuxie-runtime` rlib.
- Scoped `git diff --check`: green.

An attempted consolidated locator audit also detected a pre-existing,
concurrently shifted Wave A locator in `crates/nuxie/src/lib.rs`; Wave B5's
disjoint shard is strict and green, and this candidate does not edit any
earlier wave ledger.
