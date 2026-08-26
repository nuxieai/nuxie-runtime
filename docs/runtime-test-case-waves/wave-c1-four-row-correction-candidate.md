# Wave C1 four-row owner correction candidate

Status: **CORRECTED CANDIDATE; PENDING FRESH INDEPENDENT REVIEW**

Candidate base: `46a213848d758f55936baa43cc6f0cff9b9066ac`

Independent rejection: `8e4fb2d249fac341a63a7b154999649a1ad638c6`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

This correction changes test evidence and correspondence metadata only. It
does not change production runtime behavior or declare Wave C1 accepted.

## Four-row correction

- `in_band_asset_load_test.cpp#1` now calls the distinct no-loader production
  `File::import` flow. It preserves the exact asset metadata and 308-byte
  in-band payload assertions, then fails at the concrete missing no-loader
  fallback decode: the live ImageAsset is absent. It does not reuse the
  rejecting-loader flow from case 3.
- `in_band_asset_load_test.cpp#3` retains its accepted rejecting-loader flow.
  Its ledger `expected_red_reason` now exactly equals the test's `#[ignore]`
  text.
- `layout_participant_test.cpp#2` is pending because Rust retains no callable
  `Solo::activeComponent()` getter.
- `layout_participant_test.cpp#7` is pending because Rust retains no callable
  `Solo::getActiveChildIndex()` getter. The exact write owner exists, but it
  cannot certify either pinned read.

The test-local Solo reconstruction from `activeComponentId` plus
`cpp_local_ids`, and both tests that depended on it, were deleted. No
replacement getter or proxy was introduced.

## Exact census

- denominator: 62/62 cases;
- classifications: 49 direct, seven adapted, six pending;
- outcomes: 38 pass, 18 expected-red, six unverified;
- all 58 rows accepted by the independent review retain their semantic
  evidence, including the four previously honest pending rows.

The six pending owner gaps are grid-line offsets, `Solo::activeComponent()`,
`Solo::getActiveChildIndex()`, two intrinsic Shape-bound cases, and the
two-axis nearest-scroll-snap owner.

## Gates

- strict pinned identities, ordinals, source lines, exact names,
  classifications, outcomes, adaptations, evidence locators, and exact ignore
  reasons: 62/62;
- individually selected pass sweep: 38/38 passed;
- individually forced expected-red sweep: 18/18 failed at the recorded seams;
- repository correspondence checker: 157 files / 1,404 cases;
- correspondence-checker unit suite: 24/24;
- scoped formatting, JSON parsing, and diff checks: green;
- default no-feature non-test `nuxie` LLVM IR contains none of the corrected
  Wave C1 test or removed Solo reconstruction symbols.

Every relied-on Cargo invocation used `CARGO_INCREMENTAL=0` and
`CARGO_PROFILE_TEST_INCREMENTAL=false`. Fresh independent semantic review is
required before acceptance.
