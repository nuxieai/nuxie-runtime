# Wave C1 final independent semantic rereview

Status: **REJECT**

Corrected candidate: `98956bb5ffc0c47d4c53486ed60242c76dd6413a`

Prior rejection: `72bbd4386`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: all 62 cases in upstream files 51-58, `image_mesh_test.cpp`
through `layout_scroll_test.cpp`.

This was an independent review-only adjudication. It did not correct candidate
tests or change production behavior. The `implement` and `tdd` skills were
explicitly excluded.

## Exact census

- Candidate ledger: 62 cases; 55 direct / seven adapted; 43 pass / 19
  expected-red.
- Accepted semantic evidence: **47 cases**; 41 direct / six adapted; 31 pass /
  16 executable expected-red.
- Rejected semantic evidence: **15 cases**; 14 direct / one adapted; 12
  declared-pass / three declared expected-red.
- Pending or unreviewed: **zero**.

The corrected candidate materially improved the previous submission. Image
and mesh occurrence ownership, clipping order, animation arena lifetime,
selected-joystick application, retained grid rectangles, transparent Group
ownership, Taffy leaf topology, the manual-scroll action stream, and the other
47 rows are accepted. The failures below remain because exact parity evidence
must observe the same runtime owner; a matching value from a fresh solver,
raw payload, reconstructed path, or unrelated snapshot is not sufficient.

## Rejected rows

### In-band ImageAsset ownership

1. `in_band_asset_load_test.cpp#1` - The test asserts the imported raw payload
   length is 308 and that decode produced a RenderImage. Upstream asserts the
   live `ImageAsset::decodedByteSize` owner. The raw byte slice can stay 308
   while the occurrence-owned decoded-size field is absent or wrong.
2. `in_band_asset_load_test.cpp#3` - Loader rejection reaches fallback decode,
   but the test again proves only attempted raw bytes plus RenderImage
   presence. It never observes the live `decodedByteSize == 308` owner asserted
   upstream.

### Grid-line addressability

3. `layout_grid_test.cpp#5` - **Rejected expected-red.** The test compares the
   live grid's four-value retained rectangle `[x, y, width, height]` with six
   expected column/row line offsets. That guaranteed wrong-shape comparison is
   not a grid-line API/capability seam and never executes the six offset
   queries or the two non-grid zero-count assertions.

### LayoutParticipant retained owners

4. `layout_participant_test.cpp#1` - Reads
   `debug_taffy_layout_bounds_report`, which performs a fresh Taffy solve,
   instead of the retained Shape `LayoutNodeProvider::layoutBounds` owner.
5. `layout_participant_test.cpp#2` - Solo transparency is observed directly,
   but active-child identity is inferred by a test-local search for the first
   noncollapsed `cpp_local_ids` child. That can pass while Solo's retained
   `activeComponent` mapping is wrong.
6. `layout_participant_test.cpp#3` - Uses the same fresh diagnostic Taffy solve
   instead of retained participant bounds.
7. `layout_participant_test.cpp#4` - Uses the same fresh diagnostic Taffy solve
   instead of retained participant bounds.
8. `layout_participant_test.cpp#5` - Collapse count is preserved, but the shown
   participant's 200x200 result comes from the fresh solver rather than its
   retained provider bounds.
9. `layout_participant_test.cpp#6` - Min/max values come from the fresh solver
   rather than retained provider bounds.
10. `layout_participant_test.cpp#7` - `updateByIndex` reaches the runtime setter,
    but both `getActiveChildIndex` assertions use the same test-local
    noncollapsed-child inference. The production active-id mapping can be
    wrong while this passes.
11. `layout_participant_test.cpp#11` - Both grid-participant slot results come
    from the fresh diagnostic solve rather than retained provider bounds.
12. `layout_participant_test.cpp#15` - **Rejected adaptation.** Inactive
    collapse/solve exclusion is valid, but active and deep participant sizes
    are read from the fresh diagnostic solve. The exact retained bounds can be
    broken while the adaptation passes.
13. `layout_participant_test.cpp#18` - **Rejected expected-red.** The test asks
    the pre-advance Shape's world paint-path cache to exist, then measures that
    cache. `Shape::computeIntrinsicBounds` is a different owner, and the test
    also omits the upstream nonnegative bounds assertions for every Shape
    before narrowing to noncollapsed `PointsPath` Shapes.
14. `layout_participant_test.cpp#19` - Reconstructs local bounds from world-path
    traversal and an inverse transform rather than executing the Shape
    intrinsic-bounds owner. That proxy can mask an inverted or empty
    `computeIntrinsicBounds` result while the world-transform assertions pass.

### Scroll snap owner

15. `layout_scroll_test.cpp#6` - **Rejected expected-red.** The complete pinned
    call table is inert data. The executable failure compares two
    `ScrollConstraint` snapshots before and after setting `snap`; the snapshot
    does not retain that property, so unrelated equality guarantees failure.
    None of the disabled passthrough, directional, no-op, on-snap, or two-axis
    owner calls executes.

## Mechanical and execution gates

- Strict pinned SHA, 62 identities, ordinals, source lines, exact names,
  classifications, adaptation records, and 62 evidence locators: 62/62 green.
- All 43 declared-pass rows ran non-incrementally and passed; ignored rows
  remained excluded from normal runs.
- Forced expected-red sweep: 19/19 primary ledger rows failed individually.
  Three of those failures are semantically rejected above rather than promoted
  merely because they failed.
- Repository correspondence checker: 157 files / 1,404 Catch cases, green.
- Correspondence checker unit suite: 24/24 green.
- Non-test `nuxie-runtime` build: green. Global-symbol inspection of the rlib
  found no Wave C1 or test-accessor symbols; debug source-name strings were not
  treated as compiled behavior.

Every relied-on Cargo gate used `CARGO_INCREMENTAL=0` with dev/test profile
incremental compilation disabled. Existing user and other-lane workspace dirt
was preserved and is not part of this receipt.
