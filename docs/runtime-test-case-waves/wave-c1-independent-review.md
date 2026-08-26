# Wave C1 independent semantic review

Status: **REJECT**

Candidate: `9e00823cd`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: the 62 cases in `image_mesh_test.cpp` through
`layout_scroll_test.cpp`

This was a review-only adjudication. It did not change production behavior or
test evidence. The `implement` and `tdd` skills were explicitly excluded.

## Exact census

- Candidate ledger: 62 cases; 57 direct / five adapted; 45 pass / 17
  expected-red.
- Accepted semantic evidence: **43 cases**; 41 direct / two adapted; 28 pass /
  15 executable expected-red.
- Rejected semantic evidence: **19 cases**; 16 direct / three adapted; 17
  declared-pass / two declared expected-red.
- Pending or unreviewed: **zero**.

The five in-band/instancing/Silver accepts before `layout_participant_test.cpp`
are cases 3-6 and 10-15. After that boundary, every case is accepted except
the nine cases listed below: layout-participant 2, 7, 14, 16-19 and
layout-scroll 2 and 6. This accounts for all 43 accepts without treating an
entrypoint or green assertion as semantic proof by itself.

## Rejected rows

### Image, instancing, joystick, and grid

1. `image_mesh_test.cpp#1` — The Rust test resolves `Tape body.png`, but tests
   the raw imported payload length and `graph.meshes.first()` instead of the
   live Image occurrence's decoded ImageAsset and exact Image-to-Mesh owner.
   It can pass without the instantiated Image retaining either owner.
2. `image_mesh_test.cpp#2` — Three Artboard occurrences are instantiated and
   drawn, but none is queried for its Image, ImageAsset, Mesh, or index owner.
   `std::ptr::eq` repeatedly compares the same `RuntimeFile` byte slice to
   itself, so the claimed sharing proof is vacuous and cannot detect
   per-instance index cloning.
3. `instancing_test.cpp#2` — Clipping membership and order are read from
   immutable `graph.clipping_shapes`, not from the clone-owned clipping state
   on the live Artboard occurrence. The recording draw has no clipping oracle,
   and the adapted definition/instance identity is not otherwise proved.
4. `instancing_test.cpp#3` — The test captures a pointer to
   `graph.animations[0]` and compares that same graph slot to itself before and
   after dropping one Artboard occurrence. It never queries the occurrence's
   animation catalog or proves definition-to-instance sharing/lifetime.
5. `joystick_flags_test.cpp#1` — Flag bits are exact, but each selected-axis
   mutation is followed by global `ArtboardInstance::update_pass()`, which can
   apply every eligible joystick, rather than the pinned selected
   `Joystick::apply` owner. Hoisting all four flag checks ahead of the apply
   flows also changes the pinned owner/action order.
6. `layout_grid_test.cpp#1` — `update_pass()` plus
   `debug_taffy_layout_bounds_report()` performs a fresh diagnostic solve in
   place of `Artboard::advance(0)` followed by retained live
   `LayoutComponent::layoutX/Y/Width/Height` observations. Settlement or
   retained publication can be broken while this passes.
7. `layout_grid_test.cpp#2` — The same fresh-solve diagnostic proxy replaces
   the retained live LayoutComponent owner. The compressed tuples preserve the
   numeric assertions, but not the behavior under test.
8. `layout_grid_test.cpp#3` — The track mutation is exact, but the subsequent
   fresh diagnostic solve can yield 250/50 even if the dirty/reflow path never
   publishes the retained LayoutComponent state asserted upstream.
9. `layout_grid_test.cpp#4` — Again reads a recomputed debug solve rather than
   live retained LayoutComponent widths after `Artboard::advance(0)`.
10. `layout_grid_test.cpp#5` — **Rejected expected-red.** It filters
    `RuntimeLayoutBoundsReport` for invented `GridLine` Components even though
    that report only emits runtime Components and can never contain such rows.
    The guaranteed empty-vector failure does not reach an exact Taffy/Yoga
    grid-line query owner; the Artboard parent check also does not assert the
    flex node's zero line counts.

### Layout participants

11. `layout_participant_test.cpp#2` — The test proves that one Shape under the
    Solo participates and is 200x200, but never proves the pinned assertion
    that the Solo itself is not a `LayoutNodeProvider`, nor obtains the active
    child through the Solo owner.
12. `layout_participant_test.cpp#7` — Directly writes generated
    `activeComponentId` and observes child collapse. That bypasses both pinned
    owners under test, `getActiveChildIndex()` and `updateByIndex(1)`, and a
    mismatch in either helper can pass.
13. `layout_participant_test.cpp#14` — Confirms that the Shape's parent is a
    `Node`, but omits the exact assertion that this intervening Group provides
    no layout node. Shape sizing alone cannot certify transparent-group owner
    semantics.
14. `layout_participant_test.cpp#16` — Replaces the exact post-advance
    `Artboard::isLeaf()` assertion with a provider-child discovery query plus a
    drawable-flag check. That pre-layout/provider proxy can be correct while
    the Artboard leaf state is wrong.
15. `layout_participant_test.cpp#17` — The inverse component-list row makes the
    same substitution: provider discovery and the flag do not prove the exact
    post-advance `!Artboard::isLeaf()` owner.
16. `layout_participant_test.cpp#18` — **Rejected expected-red.** The evidence
    asks the path traversal/world-bounds subsystem for geometry rather than
    calling the Shape intrinsic-bounds owner, and it does not preserve the
    pinned non-collapsed `PointsPath` filter. Its failure therefore does not
    certify the exact pre-advance `Shape::computeIntrinsicBounds` seam.
17. `layout_participant_test.cpp#19` — Likewise substitutes object world bounds
    for every pinned intrinsic-bounds assertion before checking world
    transforms. A local intrinsic-bounds defect can be masked or transformed
    by the world-bounds path.

### Layout scrolling

18. `layout_scroll_test.cpp#2` — The manual drag omits the pinned `Content`
    LayoutComponent assertion and supplies an arbitrary explicit one-second
    timestamp for the second move while the pinned action describes the
    200-pixel move across the 0.1-second advance. That changes the physics
    input/action stream even though the offset assertions happen to pass.
19. `layout_scroll_test.cpp#6` — The snap-disabled branch is a tautological
    local tuple assertion and never calls the ScrollConstraint snap owner.
    With snap enabled, the test calls the scalar helper directly rather than
    the two-axis `ScrollConstraint::nearestSnapOffsetInDirection` owner. The
    helper math may be right while the owner gate, axis routing, or passthrough
    behavior is wrong.

## Accepted adaptations

Two of the five declared Rust-safety adaptations are accepted:

- `instancing_test.cpp#1`: cloning the complete ownership-safe Artboard
  occurrence and comparing the exact `TopEllipse` Shape's x/y values is a
  truthful substitute for heap-cloning one C++ Component.
- `layout_participant_test.cpp#15`: excluding the inactive, collapsed Solo
  child from the retained Taffy solve truthfully preserves the intentional
  Taffy backend decision and proves it was not sized to the 200x200 slot, while
  active and deeply nested participants retain their exact live sizes.

The mesh-index and two remaining instancing adaptations are rejected because
their proposed substitute observables are self-comparisons or immutable-source
proxies, not stable ownership-safe observations of the corresponding live
occurrences.

## Mechanical and execution gates

- Pinned SHA, 62 identities, ordinals, source lines, exact names,
  classifications, adaptation records, evidence symbols/lines, and ignore
  reasons: 62/62 green.
- Focused normal suites: 45 pass and 17 ignored, as declared.
- Forced expected-red sweep: 17/17 primary ledger rows failed individually at
  their declared runtime/SRIV diagnostics. A failing test was not promoted to
  semantic acceptance when its owner or seam was wrong.
- Repository correspondence checker: 157 files / 1,404 Catch cases, green.
- Correspondence checker unit suite: 24/24 green.
- `CARGO_INCREMENTAL=0` non-test `nuxie-runtime` build: green; its rlib contains
  no Wave C1 test-owner symbols.

All Cargo review gates used `CARGO_INCREMENTAL=0`. Existing user and other-lane
workspace changes were preserved and were not included in this receipt.
