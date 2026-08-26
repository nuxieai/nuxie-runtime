# Wave C1 final-correction independent review

Status: **REJECT**

Candidate: `46a213848d758f55936baa43cc6f0cff9b9066ac`

Prior rejection: `23c16de3564d518a1255f02d44647ffb5bb23376`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: all 62 cases in upstream files 51-58, `image_mesh_test.cpp`
through `layout_scroll_test.cpp`.

This was an independent read-only adjudication of the candidate tests and
ledger. It did not correct author code, change production behavior, or use the
`implement` or `tdd` skills.

## Verdict census

- Candidate ledger: 62 cases; 51 direct / seven adapted / four pending;
  40 pass / 18 executable expected-red / four unverified.
- Strictly accepted rows: **58**; 38 executable pass / 16 executable
  expected-red / four honest pending.
- Rejected rows: **four unique cases**. Three have semantic owner/flow defects;
  two have expected-red metadata defects, with `in_band_asset_load_test.cpp#1`
  in both groups.
- The 47 rows accepted by the previous rereview retain the same semantic test
  bodies in this candidate. They were checked one by one against their ledger
  identities and the prior adjudication. Every corrected/reclassified row was
  then re-adjudicated directly against the pinned C++ body.

The retained LayoutParticipant corrections are materially sound: rows 1, 3,
4, 5, 6, 11, and 15 now read occurrence-owned post-advance
`ArtboardInstance::layout_bounds`, not a fresh Taffy diagnostic solve. The four
declared pending rows are also honest: grid-line offsets, Shape intrinsic
bounds (two cases), and the two-axis snap owner have no callable Rust owner,
and the candidate no longer substitutes narrower helpers or paint-path
proxies.

## Rejected rows

### In-band asset import

1. `in_band_asset_load_test.cpp#1` — Upstream deliberately calls
   `ReadRiveFile` with no asset loader. The candidate instead calls
   `RuntimeFileAssetOwners::import_with_loader` with an explicit loader that
   returns `false`. That is the rejecting-loader action owned by upstream case
   3, not the distinct no-loader flow owned by case 1. Both tests reach the
   same live decoded-size divergence, but collapsing the two import paths
   cannot prove that the no-loader path performs fallback decode.
2. `in_band_asset_load_test.cpp#3` — The executable flow and live-owner
   `Some(4) != Some(308)` failure are semantically accepted, but its ledger
   `expected_red_reason` does not equal the test's `#[ignore]` reason. The same
   metadata mismatch affects case 1. The ledger says “decodedByteSize reports
   decoded RGBA length instead of ...”; the tests say
   “ImageAsset::decodedByteSize is decoded RGBA length, not ...”. Exact
   expected-red reason validation therefore fails for both rows.

### Solo owner access

3. `layout_participant_test.cpp#2` — Upstream calls the live
   `Solo::activeComponent()` owner. The candidate's test-local
   `active_solo_child_owner` reads `activeComponentId` and then searches
   `cpp_local_ids` with `.position()`. Those are occurrence-owned inputs, but
   the helper reconstructs the missing getter rather than executing it. The
   retained 200x200 bounds can pass while the direct active-component owner is
   absent or wrong.
4. `layout_participant_test.cpp#7` — `set_solo_active_child_by_index` exercises
   the write owner, but both `getActiveChildIndex()` observations again come
   from the same test-local reconstruction. This is a helper bypass and a
   test-local algorithm, not evidence for the pinned getter. With no callable
   Rust getter, the row must remain pending or gain a real production owner in
   the later source-parity phase.

## Mechanical and execution gates

- Pinned SHA, all 62 identities, ordinals, source lines, exact names,
  classifications, pending shapes, and all 58 evidence path/line/symbol
  locators: green.
- Exact expected-red reason comparison: rejected for the two in-band rows
  above; the other Rust expected-red reasons match their `#[ignore]` strings.
- Focused normal execution: 24 unit-owner rows, 15 integration rows, and the
  one passing Silver row are green. Four initially non-unique name filters
  were repeated with fully qualified exact names and each selected one passing
  Wave C1 test.
- Forced expected-red execution: all 18 rows were run individually; each
  selected exactly one test and failed. The five unit-owner rows failed at
  their declared retained runtime assertions, and the 13 Silver rows failed at
  their documented complete-stream boundary. Failure alone did not promote
  the rejected owner/flow evidence above.
- Repository correspondence: 157 files / 1,404 pinned Catch cases, green.
- Correspondence checker unit suite: 24/24 green.
- A broader non-focused `nuxie-runtime --lib` diagnostic was not used as the
  Wave C1 gate: it reported 1,281 pass, 28 ignored, and eight pre-existing
  non-C1 failures. Every focused Wave C1 test selected from that target passed.

Every relied-on Cargo invocation used `CARGO_INCREMENTAL=0` and
`CARGO_PROFILE_TEST_INCREMENTAL=false`. Existing user and other-wave workspace
changes were preserved and are not part of this receipt.
