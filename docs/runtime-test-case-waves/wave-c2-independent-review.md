# Wave C2 independent semantic review

Status: **REJECT**

Candidate: `a183596b2d36a9a043832e575d1fa3f223450e43`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: all 73 cases in `layout_stack_test.cpp` through
`listener_align_target_test.cpp`.

This was a review-only adjudication. It did not change candidate tests,
ledgers, fixtures, manifests, or production behavior. The `implement` and
`tdd` skills were explicitly excluded.

## Exact census

- Candidate ledger: 73 direct cases; 52 pass / 21 expected-red.
- Accepted semantic evidence: **45 cases**; 30 pass / 15 executable
  expected-red.
- Rejected semantic evidence: **28 cases**; 22 declared-pass / six declared
  expected-red.
- Pending or unreviewed: **zero**.

The accepted rows are:

- `layout_test.cpp` cases 1-5, 7-9, 11-12, and 14-22;
- `library_asset_test.cpp` cases 1, 4, and 5;
- `line_break_test.cpp` cases 1-6 and 8-10;
- all ten `linear_animation_instance_test.cpp` cases;
- `linear_animation_test.cpp` cases 3, 4, and 6; and
- `listener_action_flags_test.cpp` case 3.

This accounts for all 45 accepts without treating a green entrypoint or an
ignored test's failure as semantic proof by itself.

## Rejected rows

### Layout stack and layout

1. `layout_stack_test.cpp#1` — After `Artboard::advance`, `Fixture::bounds`
   calls `debug_taffy_layout_bounds_report`, which runs a fresh
   `TaffyRuntimeLayoutEngine::compute_bounds`. The pinned assertions read the
   retained live `LayoutComponent::layoutX/Y/Width/Height` owners. A retained
   publication defect can therefore pass this recomputed diagnostic.
2. `layout_stack_test.cpp#2` — Every post-mutation assertion uses the same
   fresh diagnostic solve instead of the retained box LayoutComponent after
   each `Artboard::advance`. The exact nine mutations are present, but the
   queried owner is not.
3. `layout_stack_test.cpp#3` — The pinned case directly checks the attached
   style's `display()` enum after each `displayValue`/`layoutTypeValue`
   mutation. Rust instead advances the whole Artboard, recomputes layout, and
   counts non-collapsed diagnostic descendants. That aggregate can agree
   while the style getter or enum routing is wrong.
4. `layout_test.cpp#6` — **Rejected expected-red.** The test asks the fresh
   layout report for the Text local and panics because no report row exists.
   It never interrogates the pinned live `Text::localBounds` owner, so this is
   not execution to the declared missing-HiText-bounds seam.
5. `layout_test.cpp#10` — **Rejected expected-red.** The three pinned
   `LayoutComponent::actualDirection()` assertions are omitted. The test then
   reads authored `Text.alignValue`, not the computed `Text::align()` owner,
   and currently fails because that authored value is `0` rather than `2`.
   The failure is therefore on a different observable than the declared
   actual-direction-derived owner.
6. `layout_test.cpp#13` — Root width and height come from the fresh diagnostic
   report, not the retained Artboard `layoutWidth`/`layoutHeight` values read
   upstream after `advance(0)`.
7. `layout_test.cpp#23` — **Rejected expected-red.** Child selection and all
   four assertions use recomputed report rows. The pinned case selects the
   live nested LayoutComponent through its parent relation and observes its
   retained parent-relative layout values. The reported numerical divergence
   is not evidence from that owner.

### Library assets

8. `library_asset_test.cpp#2` — The nested animation is found by filtering all
   immutable host-graph objects. It is not obtained from the named live
   NestedArtboard's `nestedAnimations()` owner, so detached or incorrectly
   attached animation records can pass.
9. `library_asset_test.cpp#3` — The state-machine variant has the same detached
   global-membership proxy: it never proves that the exact named
   NestedArtboard owns the selected NestedStateMachine.
10. `library_asset_test.cpp#6` — The test follows serialized `artboardId` and
    `assetId` fields into the File tables. It does not query the live Image's
    `imageAsset()` relation, which is the linkage asserted upstream.
11. `library_asset_test.cpp#7` — Both library-image checks likewise stop at
    graph objects and File asset-table resources. Neither live Image occurrence
    proves its exact ImageAsset owner or identity.
12. `library_asset_test.cpp#8` — Event presence is proved only by immutable
    graph membership. The pinned case resolves a live Event from the Artboard
    occurrence before advancing; the data-bound string assertion alone cannot
    prove that owner exists.
13. `library_asset_test.cpp#9` — Rust advances before it discovers either
    nested Artboard occurrence, while upstream retains both exact nested
    owners first and advances afterward. The reordered traversal can pass if
    occurrences are created or replaced late.
14. `library_asset_test.cpp#10` — This row has the same ordering loss and
    substitutes graph paint-container counts plus an arbitrary live
    SolidColor lookup for `lib2Artboard->shapePaints()[0]` and its exact Fill
    and paint ownership chain.

### Line breaking

15. `line_break_test.cpp#7` — **Rejected expected-red.** Upstream first proves
    two shaped Paragraph owners, one run in each, and LTR base direction for
    both, then breaks their lines separately. Rust flattens the whole text into
    `StandaloneLine.paragraph` labels and asserts only the two line counts.
    Its `(1, 1)` failure does not certify paragraph creation, run ownership, or
    either direction assertion.
16. `line_break_test.cpp#11` — **Rejected expected-red.** The line-count
    divergence is real, but the test omits the pinned one-Paragraph owner and
    LTR base-direction assertions. Those independent upstream assertions must
    remain executable before the line-break seam.
17. `line_break_test.cpp#12` — **Rejected expected-red.** Glyph count/index
    assertions are preserved, but the shaped Paragraph count and LTR
    base-direction assertions are omitted. The flattened line owner is not a
    complete port of the pinned flow.

### Linear animation

18. `linear_animation_test.cpp#1` — The upstream `LinearAnimation::endTime()`
    assertion is replaced by local helper `end_time`, which reimplements the
    answer from the sign of speed. This test-only algorithm can pass when the
    runtime end-time owner is absent or wrong.
19. `linear_animation_test.cpp#2` — The negative-speed row uses the same
    surrogate local algorithm instead of the concrete runtime `endTime`
    surface.
20. `linear_animation_test.cpp#5` — Serialized import proves only that the
    missing keyed object was filtered and the valid sibling remained. It omits
    both exact callback invocation counts, the returned `StatusCode::Ok`, and
    the continued `onAddedDirty` call on the valid owner.

### Listener action flags and alignment

21. `listener_action_flags_test.cpp#1` — `parentKind()` is never called on a
    concrete ListenerAction. Full-file imports and final owner counts are a
    routing proxy for the exact four getter results.
22. `listener_action_flags_test.cpp#2` — Scheduled occurrence filtering is
    executable, but all three `parentKind()` assertions are again replaced by
    import-result counts. The two independent flag fields are therefore not
    both proved on the exact action owner.
23. `listener_action_flags_test.cpp#4` — Counts show one transition action,
    but omit the pinned retained-identity assertion that the stored action is
    the exact imported action and the untouched state owner remains empty.
24. `listener_action_flags_test.cpp#5` — The state route similarly proves only
    aggregate counts, not action identity or both untouched owners.
25. `listener_action_flags_test.cpp#6` — The listener route proves only a count
    and omits the exact stored-action identity.
26. `listener_action_flags_test.cpp#7` — A whole-file parse error containing
    `FocusActionClear` replaces the exact `StatusCode::MissingObject` result
    and does not prove the transition's listener-action collection stayed
    empty.
27. `listener_align_target_test.cpp#1` — Upstream performs
    `advanceAndApply(0)`, then a non-applying `advance(0)`, and repeats that
    distinction after the pointer moves. Rust calls
    `advance_state_machine_instance` for both steps, applying on both and
    changing the action order under test.
28. `listener_align_target_test.cpp#2` — The preserve-offset-on row makes the
    same extra-apply substitutions before and after pointer input.

## Mechanical and execution gates

- Pinned SHA, 73 identities, ordinals, source lines, exact names, evidence
  symbols/lines, classifications, outcomes, and ignore reasons: 73/73
  mechanically valid.
- Focused normal suites were run with `CARGO_INCREMENTAL=0` and
  `CARGO_PROFILE_TEST_INCREMENTAL=false`; all 52 declared pass rows remained
  green and all 21 declared reds remained ignored.
- All 21 expected-red ledger rows were forced individually with both
  incremental settings disabled; 21/21 failed. Rows were still rejected when
  the failing assertion used the wrong owner, omitted earlier assertions, or
  stopped at a proxy seam.
- Repository correspondence checker: 157 files / 1,404 Catch cases, green.
- Correspondence checker unit suite: 24/24 green.
- Candidate diff check and evidence locators: green.

Existing user and other-lane workspace changes were preserved and are not
part of this receipt.
