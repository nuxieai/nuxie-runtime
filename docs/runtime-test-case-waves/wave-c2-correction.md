# Wave C2 owner-evidence correction candidate

Status: **CORRECTED CANDIDATE; PENDING FRESH INDEPENDENT REVIEW**

This corrects Wave C2 candidate `a183596b2d36a9a043832e575d1fa3f223450e43`
against independent rejection receipt
`d8aa0506adbec39e4eb2f37d3cb2c69cef0f619b`. It changes tests and evidence
only and does not declare Wave C2 accepted.

## Exact census

- denominator: 73/73 pinned upstream cases;
- classifications: 62 direct, seven C++-language adaptations, four pending;
- outcomes: 53 pass, 16 expected-red, four unverified;
- all 28 rejected rows were re-adjudicated; the other 45 rows retain their
  accepted semantics.

The four pending rows are `layout_test.cpp#10` and
`line_break_test.cpp#7/#11/#12`. The Rust runtime does not retain the exact
Text align or Paragraph/run/base-direction owners needed to address those
assertions. Their prior narrowed substitutes were removed rather than counted
as executable parity evidence.

## Corrections

- Layout evidence now reads retained `ArtboardInstance::layout_bounds`, live
  transforms, the exact display owner, and the private retained Text bounds.
  The Text-align row remains pending because a retained owner is absent.
- Library evidence asserts the exact named nested animation/state-machine,
  image-to-asset, nested-artboard, paint-container, and event owners at the
  pinned lifecycle points.
- Line-break evidence retains the complete run-break assertions where the
  production annotation owner exists. Three missing Paragraph/run-owner rows
  are pending; their obsolete aggregate-line proxies were deleted.
- Linear-animation timing calls the production owner. Case 5 preserves the
  two-object import lifecycle and sibling global/object identity as an explicit
  C++-language adaptation.
- Listener evidence compares exact imported action identity, owner status,
  untouched owner collections, and the precise dropped-object error. Alignment
  preserves the pinned apply versus non-apply advance sequence.

## Gates

- strict pinned identities, ordinals, source lines, classifications, outcomes,
  evidence locators, adaptations, and ignore reasons: 73/73;
- focused pass rows: 53/53;
- revised individual forced-red sweep: 16/16 failed at their recorded seams;
- repository correspondence checker: 157 files / 1,404 cases;
- correspondence-checker unit suite: 24/24;
- scoped formatting, JSON parsing, and diff checks: green;
- default no-feature non-test LLVM IR: no Wave C2 test-owner symbols.

Every relied-on Cargo gate used `CARGO_INCREMENTAL=0`. This receipt is a
correction-candidate record only; fresh independent semantic review is still
required.
