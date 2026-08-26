# Wave C2 six-row owner correction candidate

Status: **CORRECTED CANDIDATE; PENDING FRESH INDEPENDENT REREVIEW**

Candidate base: `0b3b7ad4b0d42c9743e36a092c44971318239dcc`

Independent rejection: `a734bf214d226bb6f8d7df84728c3c7414f64833`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

This correction changes test evidence and correspondence metadata only. It
does not change production behavior or declare Wave C2 accepted.

## Six-row correction

- `layout_test.cpp#23` selects the styled child through the concrete live
  LayoutComponent parent relation after `Artboard::advance(0)`, proves that
  parent is a non-Artboard LayoutComponent, and reads all four retained child
  bounds. Immutable graph ancestry is no longer used.
- `library_asset_test.cpp#2/#3` preserve the exact live NestedArtboard name,
  position, `artboardId`, source animation/state-machine count and name,
  nested-animation empty name/id/count, and zero FileAsset count.
- `library_asset_test.cpp#6` preserves the exact one-asset File count, live
  source Image count, live `Image.assetId == 0`, and Image-to-ImageAsset owner.
- `library_asset_test.cpp#7` preserves the exact two-asset File count and both
  named live nested Image-to-ImageAsset relations.
- `library_asset_test.cpp#10` binds the exact authored default ViewModel,
  proves the exact nested lib2 Artboard and its single live first Fill owner,
  derives that Fill's SolidColor mutator from the same paint relation, advances
  the root, and asserts the final live `0xff101566` color.

All six ledger locators now point to the corrected crate-owned exact-owner
tests. The prior graph-ancestry layout test and graph-wide library aggregate
are not registered as evidence.

## Exact census

- denominator: 73/73 cases;
- classifications: 62 direct, seven explicit C++-language adaptations, four
  pending;
- outcomes: 53 pass, 16 expected-red, four unverified;
- the 63 executable and four pending rows accepted by the independent review
  retain their semantic classifications and evidence.

The four pending rows remain `layout_test.cpp#10` and
`line_break_test.cpp#7/#11/#12`; no additional owner gap was introduced.

## Gates

- strict pinned identities, ordinals, source lines, exact names,
  classifications, outcomes, adaptations, evidence locators, and ignore
  reasons: 73/73;
- individually selected pass sweep: 53/53 passed;
- individually forced expected-red sweep: 16/16 failed at the recorded seams;
- repository correspondence checker: 157 files / 1,404 cases;
- correspondence-checker unit suite: 24/24;
- scoped formatting, JSON parsing, and diff checks: green;
- default no-feature non-test `nuxie-runtime` LLVM IR contains none of the
  corrected Wave C2 test-owner symbols.

Every relied-on Cargo invocation used `CARGO_INCREMENTAL=0` and
`CARGO_PROFILE_TEST_INCREMENTAL=false`. Fresh independent semantic rereview is
required before acceptance.
