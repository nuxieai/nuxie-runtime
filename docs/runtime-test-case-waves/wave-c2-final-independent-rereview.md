# Wave C2 final independent semantic rereview

Status: **REJECT**

Candidate: `0b3b7ad4b0d42c9743e36a092c44971318239dcc`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Prior rejection receipt: `d8aa0506adbec39e4eb2f37d3cb2c69cef0f619b`

Scope: all 73 cases in `layout_stack_test.cpp` through
`listener_align_target_test.cpp`.

This was a review-only adjudication. It did not change candidate tests,
ledgers, fixtures, manifests, or production behavior. The `implement` and
`tdd` skills were explicitly excluded.

## Exact census

- Candidate ledger: 73 cases; 53 pass / 16 expected-red / four pending.
- Accepted executable evidence: **63 cases**; 48 pass / 15 expected-red.
- Truthful unavailable-owner rows retained as pending: **four cases**.
- Rejected semantic evidence: **six cases**; five declared-pass / one
  declared expected-red.
- Unreviewed: **zero**.

The four pending rows are `layout_test.cpp#10` and
`line_break_test.cpp#7/#11/#12`. The candidate correctly stopped counting the
former narrowed observables as parity proof: the runtime has no retained Text
align owner or retained Paragraph/run/base-direction owner corresponding to
the pinned calls. No existing direct surface was found during this rereview.

The six rejected rows are listed below. Every other executable row was
individually rechecked against the pinned fixture, action order, production
owner, and assertion stream, including all original rejects corrected by the
candidate.

## Rejected rows

1. `layout_test.cpp#23` — The bounds are now read from the retained live
   layout owner, but the test still selects that owner by walking immutable
   `ArtboardGraph` ancestry. The pinned loop selects the live
   `LayoutComponent` whose live parent is a non-Artboard LayoutComponent and
   whose style is present. A stale or incorrect runtime parent relation can
   therefore pass this test. The expected-red row must select and prove the
   exact live parent/child owner before asserting the four retained bounds.
2. `library_asset_test.cpp#2` — The corrected test reaches the named live
   NestedArtboard and its exact nested simple-animation owner, but it omits
   independent pinned assertions: `artboardId == 1`, the source artboard has
   exactly one animation, the nested animation name is empty, and the File has
   zero assets. Looking up animation index zero and checking its name does not
   reject extra source animations or assets.
3. `library_asset_test.cpp#3` — The state-machine variant has the same
   assertion loss: it omits `artboardId == 1`, the source artboard's exact
   state-machine count, the nested animation's empty name, and the File's zero
   asset count. The new owner path is valid but the pinned assertion stream is
   incomplete.
4. `library_asset_test.cpp#6` — The new test proves a live Image-to-ImageAsset
   relation and asset name, but omits the pinned `file->assets().size() == 1`
   and exact `images[0]->assetId() == 0` assertions. An extra File asset or a
   nonzero Image asset ID can pass.
5. `library_asset_test.cpp#7` — Both live Image-to-ImageAsset relations and
   names are now proved, but the exact `file->assets().size() == 2` assertion
   remains omitted. Extra File assets can pass independently of the two
   selected Image owners.
6. `library_asset_test.cpp#10` — The ledger still points to
   `crates/nuxie/tests/upstream_library_asset.rs::library_vmtest_1_host`, whose
   graph-wide paint count and arbitrary SolidColor lookup were rejected in the
   prior receipt. The added exact first-Fill-owner unit is not the registered
   evidence and, even considered separately, stops before binding/advancing
   and omits the pinned final color assertion. Neither test alone preserves
   the complete owner/action/assertion stream.

## Mechanical and execution gates

- Pinned SHA, all 73 identities, ordinals, source lines, exact names,
  classifications, outcomes, and evidence locators: 73/73 green.
- Focused non-incremental normal suites: green; all declared pass rows selected
  by the Wave C2 filters remained green and all declared expected-red rows
  remained ignored.
- All 16 expected-red rows were forced individually with incremental
  compilation disabled and failed at their selected test assertions.
- Repository correspondence checker: 157 files / 1,404 pinned Catch cases,
  green.
- Correspondence-checker unit suite: 24/24 green.

Existing user and other-lane workspace changes were preserved and are not
part of this receipt.
