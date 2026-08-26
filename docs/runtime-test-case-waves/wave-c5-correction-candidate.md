# Wave C5 geometry/path correction candidate

Corrects independent rejection: `54cbd8880`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Corrected census

- Direct pass: 12
- Adapted pass: 1
- Direct expected-red: 2
- Pending: 28
- Total: 43

The 15 executable rows have 15 distinct discoverable locators. The four
demoted rows retain no evidence, note, or adaptation. The 24 previously
pending rows, ten previously accepted passes, and two genuine expected-red
streams are unchanged.

## Seven-row correction

- `mat2d_test.cpp#2` now calls the real `Mat2D::invert()` owner, asserts its
  returned `Option` is present in the pinned position, and stores that exact
  returned inverse. The fallback-to-identity path and extra determinant
  assertion are gone.
- `mat2d_test.cpp#3` keeps its complete numeric stream and is now a structured
  `rust-safety` adaptation. Only independent C++ raw-pointer/count versus Span
  overload dispatch is declared inapplicable; all matrices, points, AABBs, and
  ordered results remain live.
- `raw_path_test.cpp#4` is pending with no locator because the existing test
  replaces production `transformInPlace` with test-local point multiplication.
- `raw_path_test.cpp#7` is pending with no locator because the existing test
  replaces the pinned iterator visits with a local fold and adds assertions
  for three source helper results that were intentionally ignored. Those
  legacy test assertions remain outside denominator evidence; this correction
  does not rewrite the de-mapped test.
- `rectangles_to_contour_test.cpp#1` restores the first
  `contourCount() == 1` assertion immediately after the first contour compute,
  before the pinned size and point assertions.
- `render_test.cpp#1` is pending with no locator. The current wrapper does not
  expose the view-model handle needed to bind the already-created live state
  machine in the pinned position; Artboard-only binding is not accepted.
- `stroke_test.cpp#1` is pending with no locator. Static graph name/type/child
  projections are not accepted as live Artboard lookup and retained paint-type
  authority.

No production behavior, Silver stream, fixture, baseline, expectation, or
expected-red reason changes in this correction.

## Candidate gates

- focused non-incremental 13-pass sweep: green;
- exact Silver sweep: 3 passed / 1 ignored expected-red;
- both expected-red rows independently forced and failed at their documented
  live boundary;
- strict Wave C5 identity/status/outcome/adaptation/pending/locator audit:
  43/43 rows and 15/15 executable locators green;
- repository correspondence checker: 157 files / 1,404 pinned cases green;
- correspondence-checker unit suite: 24/24 green;
- pinned checkout/source identities, JSON parse, scoped rustfmt, and
  `git diff --check`: green;
- non-incremental release Silver library IR contains no `wave_c5_` test symbol
  or expected-red string.

This is a correction candidate only and does not self-accept Wave C5.
