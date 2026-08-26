# Wave C2 final independent acceptance

Status: **ACCEPT**

Candidate: `a94115084bef0539d32aec7f8964a7ddf2b817e3`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Correction target: rejection receipt
`a734bf214d226bb6f8d7df84728c3c7414f64833`.

Scope: all 73 cases in `layout_stack_test.cpp` through
`listener_align_target_test.cpp`, with fresh semantic adjudication of the six
previous rejects and a full identity/locator regression check.

This was a review-only adjudication. It did not change candidate tests,
ledgers, fixtures, manifests, or production behavior. The `implement` and
`tdd` skills were explicitly excluded.

## Verdict and census

- accepted executable evidence: **69 cases**;
- outcomes: 53 pass / 16 executable expected-red;
- truthful unavailable-owner rows: **four pending**;
- rejected or unreviewed: **zero**.

The four pending rows remain `layout_test.cpp#10` and
`line_break_test.cpp#7/#11/#12`. Their evidence remains empty and their notes
still identify the unavailable retained Text-align or Paragraph/run/direction
owners. The correction did not substitute aggregate or synthetic evidence for
those gaps.

## Six-row rereview

1. `layout_test.cpp#23` now selects the styled child from the live
   `ArtboardInstance` component collection, proves its live retained parent is
   a non-Artboard LayoutComponent, then reads the four retained child bounds.
   Forced execution fails on the concrete live result
   `(15, 25, 160, 140)` versus the pinned `(10, 20, 160, 140)` expectation.
2. `library_asset_test.cpp#2` now preserves the exact live NestedArtboard
   name, position, `artboardId == 1`, one source animation, zero File assets,
   one nested animation, its empty name and ID zero, and the source animation
   name.
3. `library_asset_test.cpp#3` preserves the corresponding exact state-machine
   count/name, nested state-machine empty name/ID/count, Artboard identity and
   position, and zero File assets.
4. `library_asset_test.cpp#6` now asserts one File asset, exactly one source
   Image, live `assetId == 0`, and the concrete Image-to-ImageAsset owner.
5. `library_asset_test.cpp#7` now asserts exactly two File assets and retains
   both named live nested Image-to-ImageAsset relations.
6. `library_asset_test.cpp#10` is now registered to the corrected crate-owned
   test. That test constructs and binds the authored default ViewModel,
   retains the exact lib2 occurrence and its first live Fill/SolidColor owner,
   advances the root, rechecks that owner relation, and asserts the exact live
   `0xff101566` color.

The correction commit contains only the C2 test module, removal of the
superseded integration red, the C2 ledger locator updates, and its candidate
receipt. The test module is included only under `cfg(test)`; no production
behavior was changed.

## Gates

- Strict pinned identities, ordinals, source lines, names, classifications,
  outcomes, ignore markers, and evidence locators: 73/73 green.
- Corrected library-owner group: six selected tests / six passed, including
  all five corrected pass rows.
- Focused Wave C2 non-incremental suite: green; 33 unit passes / four ignored,
  14 layout integration passes, two linear-animation integration passes / one
  ignored, and two listener-alignment passes.
- All 16 expected-red rows were forced individually with incremental
  compilation disabled; 16/16 selected exactly one test and failed.
- Repository correspondence checker: 157 files / 1,404 pinned Catch cases,
  green.
- Correspondence-checker unit suite: 24/24 green.
- Candidate diff and receipt diff checks: green.

Existing user and other-lane workspace changes were preserved and are not
part of this receipt.
