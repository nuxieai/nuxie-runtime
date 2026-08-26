# Wave C1 four-row correction independent acceptance

Status: **ACCEPT**

Candidate: `44b17733cdad02689855241c19e448c9a9ff1897`

Prior rejection: `8e4fb2d249fac341a63a7b154999649a1ad638c6`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: the four rejected rows from the prior receipt, followed by a regression
check of all 62 Wave C1 identities and evidence locators. This review did not
change author code or production behavior and did not use the `implement` or
`tdd` skills.

## Verdict census

- All **62** rows are accepted: 38 executable pass, 18 executable expected-red,
  and six honest pending; 49 are direct and seven are approved adaptations.
- `in_band_asset_load_test.cpp#1` now executes the genuinely distinct
  no-loader high-level `File::import` path. It proves the pinned metadata and
  308-byte embedded payload before failing at the real missing retained
  fallback-decoded `ImageAsset` owner.
- `in_band_asset_load_test.cpp#3` retains its live-owner `Some(4) != Some(308)`
  failure, and its ledger reason now exactly matches its `#[ignore]` reason.
- `layout_participant_test.cpp#2` and `#7` no longer use a test-local Solo
  getter reconstruction. The proxy helper and proxy tests are gone, and both
  rows are honestly pending because the exact getters are not callable.

The six pending rows are limited to owners that do not exist as callable Rust
seams: grid-line offsets; `Solo::activeComponent`; `Solo::getActiveChildIndex`;
two Shape intrinsic-bounds cases; and two-axis nearest-snap offset. None is
represented by a proxy, fresh Taffy recomputation, helper bypass, test-local
algorithm, or synthetic red.

## Mechanical and execution gates

- Strict validation of all 62 pinned identities, ordinals, source lines,
  names, classifications, outcomes, adaptations, pending shapes, and all 56
  evidence path/line/symbol locators: green. Every Rust expected-red ledger
  reason exactly matches its `#[ignore]` reason.
- Focused normal execution for the changed unit targets: 19 passed, four
  ignored, zero failed.
- The corrected no-loader case was forced individually and failed only after
  its exact metadata/payload assertions, at the absent retained fallback image
  owner. The corrected rejecting-loader case was forced individually and
  failed at its declared live decoded-byte-size assertion.
- The unchanged focused pass rows and remaining 16 expected-red rows retain the
  bodies and adjudication covered by the prior independent full review.
- Repository correspondence: 157 files / 1,404 pinned Catch cases, green.
  Correspondence checker unit suite: 24/24 green.
- Candidate diff whitespace validation: green. The correction contains only
  test code, a `#[cfg(test)]` module hook, ledger updates, and review docs; it
  makes no production behavior change.

Every relied-on Cargo invocation used `CARGO_INCREMENTAL=0` and
`CARGO_PROFILE_TEST_INCREMENTAL=false`. Existing user and other-wave workspace
changes were preserved and are not part of this receipt.
