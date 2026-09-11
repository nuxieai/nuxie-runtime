# Positioned paint order: implementation requirements

L17 is not visually qualified. Its frozen initial/composition replays retain
80 pixel failures despite exact geometry. No tolerance changes are proposed.

## Confirmed ordering semantics

[CSS 2.2 Appendix E](https://www.w3.org/TR/CSS22/zindex.html), consulted
2026-09-10, places positioned auto-z descendants after ordinary content.
Positioned descendants of an auto-z positioned element still participate in the
parent stacking context; they are not sealed into an atomic stacking context.
The compiler currently excludes explicit z-index and other stacking-context
features, which retain separate backlog qualification.

`validation/positioned-order-cases.json` is an eight-scene discriminator:
positioned/static ancestor × positioned/static later sibling × clipped/unclipped
ancestor. The inner positioned teal box overlaps the green sibling. Chrome153
captures24 views. At interior pixel(120,40), teal wins against an ordinary
sibling unless the ancestor clips it; a later positioned sibling wins in both
ancestor-position modes. `positioned-order-oracle/paint-sample-receipt.json`
records all24 passing sample assertions and complete PNG hashes. This is pixel
sampling evidence, not a substitute for full visual inspection.

Three existing composition sheets were directly inspected at all three widths:
row/border-box overlap, nested-positioned and clip. All nine show ordinary green
content incorrectly occluding positioned orange or teal content. The clip case
retains its expected teal clipping boundary. Evidence and image hashes:
`relative-position-composition-native/positioned-failure-inspection.json`.

## Runtime seam and constraints

`Artboard::css_ordered_drawables` recursively orders complete layout groups.
It does not classify authored CSS positioning. Wire position type cannot supply
this classification: ordinary imported layout also defaults to Relative.
The compiler must transport an explicit positioned marker if the new traversal
requires it, including `position:relative` with all auto/zero offsets.

`LayoutComponent::draw_proxy` both paints the background and opens its clip;
`LayoutComponent::draw` closes that clip using the occurrence's ClipSaved flag.
`Artboard::draw_drawables` traverses linked occurrences and coalesces clip-only
operations. Therefore merely moving an inner positioned group out of a static
ancestor can lose clipping or unbalance save/restore. Replaying the ancestor's
existing proxy would also repaint its background, which is incorrect.

The correction needs a paint plan that separates painting order from ancestral
clip scope. Ordinary groups paint first; positioned units follow in the
appropriate tree order while retaining their ancestor clip chain. Positioned
descendants cannot be indiscriminately flattened as part of a positioned parent.
Clip-only scope operations must not repaint ancestor backgrounds. Preserve the
existing default traversal for scenes without the new explicit CSS policy.

## Required checks for integration

- Keep the frozen24-view discriminator and existing80 failures as red evidence.
- Check static/relative markers, nested auto positioning, order/reverse order,
  clipped ancestor scopes and zero-offset positioned elements.
- Verify policy installation, target validation, original/clone ownership and
  clearing; require a versioned host capability if traversal semantics change.
- Compare all original and composition pixels under unchanged gates; inspect
  changed images, then check vector rendering and native/WASM parity.
- Exercise empty clips and nested clip save/restore without leaking state to
  later siblings. Keep geometry and scene identities unchanged.

Frozen native discriminator replay:24/24 geometry passes,18/24 pixels pass.
The six failures are precisely the two unclipped/ordinary-sibling cases across
all widths; clipped and positioned-sibling controls pass. Evidence:
`output/playwright/html-to-riv/positioned-order-native/replay.json`.

No paint traversal correction has been installed yet. This document records
the evidence that rules out a sibling-only reorder as a complete solution.

## Paint-plan implementation

`crates/nuxie-runtime/src/mechanical_port/source/artboard/css_paint_plan.rs`
now builds the two-phase plan from a forward-ordered tree of drawable groups.
First it emits ordinary content, excluding positioned descendants. Then it
visits positioned nodes in preorder, giving each its ordinary subtree while
deferring further positioned descendants. Each deferred group carries the clip
handles of its ancestors; its own before/after drawables retain its own scope.
Ancestor background commands are never duplicated into deferred groups.

Seven unit regressions cover ordinary/positioned siblings, descendants escaping
ordinary groups, nested auto-positioning versus later positioned siblings,
ancestral clip chains, closure of ordinary scopes before deferred paint, input
order within phases and unchanged output for wholly ordinary trees. The initial
depth-first traversal failed six tests and passed the ordinary control; the
corrected two-phase implementation passes all seven tests (exit0), recorded in
`output/playwright/html-to-riv/positioned-order-plan-green.log`.

This planner is compiled as an internal artboard module but is not connected to
drawing yet. Remaining integration: build the tree from runtime occurrences,
transport authored positioned markers, execute clip-only ancestor scopes,
preserve invisible/empty-clip handling, and gate new behavior through the host
contract. No pixel improvement is claimed from planner tests alone.

Planner receipt: `output/playwright/html-to-riv/positioned-order-plan-receipt.json`.

## Runtime drawing integration: focused discriminator passes

The runtime now converts its existing CSS drawable ordering into the forward
paint tree and caches the resulting plan. `set_css_positioned_paint_order`
validates unique layout object IDs before replacing its marker set; empty clears
the policy. Clone definitions copy marker IDs, and plans use instance-local
handles when rebuilt. Lifecycle behavior still needs explicit regression tests.

The drawing path executes each paint group with its ancestor scopes.
`begin_css_ancestor_clip` opens a clip without background painting or touching
the ordinary proxy's ClipSaved flag. Hidden/collapsed ancestors suppress deferred
groups, and each opened scope is restored. The default linked traversal delegates
to the same occurrence executor when no positioned policy is installed.

Runtime check passes. A frozen diagnostic probe injects explicit source markers
using `NUXIE_CSS_POSITIONED_SOURCE_IDS`; the compiler does not yet publish this
new paint contract. The native-only `positioned-order-toolchain` is explicitly
experimental and must not substitute for a native/WASM qualified release.

The24-view discriminator now passes24/24 geometry and pixels, correcting all six
original failures. All eight three-width sheets were directly inspected and
recorded with an audited24/24 receipt. Positioned content paints above ordinary
siblings, later positioned siblings retain precedence, clipped ancestors hide
their overflowing positioned descendants, and no background repaint or clip
leakage is visible. A full-size390px clip control was additionally inspected.

Evidence: `positioned-order-native-experiment/{replay,visual-inspection}.json`
and `positioned-order-runtime-receipt.json`. Larger diagnostic replays are running:
composition192 views in session99145 and initial576 views in session94425.
The earlier80 failures remain preserved until corresponding corrected runs are
complete and reviewed. Public host transport, lifecycle/invalid-target checks,
expanded parity, vector rendering and full regression remain open.

## Expanded replay, lifecycle and public contract

Both larger diagnostic replays completed successfully: initial576/576 and
composition192/192 geometry/pixels pass. All80 previous failures are corrected
numerically, with their original reproducers preserved. Full visual review of
these larger cohorts remains pending; only the focused24-view discriminator has
complete direct review so far.

`tests/positioned_paint_runtime.rs` adds two passing public-runtime regressions.
Draw colors verify that cloning preserves the positioned order and clearing one
instance leaves its clone unchanged. Root, missing, duplicate and non-layout
targets reject without altering the installed order. Clip toggling true→false→true
preserves paint order, reopens exactly the applicable ancestral clip for deferred
paint, and balances every recorded save/restore without repainting backgrounds.

The compiler now emits version17, `layout-css-positioned-paint-v1`, and the
`layout_positioned` object-ID list for every computed relative-positioned layout,
including auto/zero insets. This replaces the earlier no-new-host-requirement
candidate: geometry alone did not expose the required paint semantics. Version
finalization occurs after descendant traversal so older policies cannot downgrade
it. Percentage spacing, aspect ratios and other earlier payloads can coexist.
Rust validates version/capability/payload agreement, unique non-root IDs, and
actual layout targets. The probe installs the checked policy before drawing;
its diagnostic environment injection remains for historical reproductions.

Five compiler tests (including two new contract tests), two runtime lifecycle
tests and TypeScript checks pass. Older hosts missing the capability reject.
The full module suite is running in `positioned-order-full-public.log`
(session24071). A new frozen native/WASM build and executed host-rejection/parity
checks are still required before publishing qualification evidence for version17.

Version17 full public module run completed: 317 tests across 58 groups,
no failures or ignored tests; session24071 exit0.

## Frozen version17 host and parity qualification

`positioned-v17-toolchain` successfully built and froze publisher, probe, Rust
Metal renderer and WASM. All17 host-contract tests pass, including the new
version17 test: it observes corrected draw colors at four widths with unchanged
Rive bytes, and rejects unsupported hosts plus old/new versions, missing payload,
missing capability, duplicate/root/non-layout/unknown targets before a stream is
written. The probe receives the actual compiler manifest; diagnostic marker
injection is explicitly absent from this test.

Expanded native/WASM parity passes11/11, now including both initial and
composition relative-position oracles. The public-path discriminator passes24/24
pixels, and all24 reviews transfer from the fully reviewed diagnostic baseline
using exact compiler input plus full browser/native PNG identity; transfer audit
passes. Public composition replay also passes192/192 geometry/pixels.

Larger visual audit began with12 diagnostic composition views: row/border-box
overlap, nested-positioned, clip and image at all three widths. The order fixes
visually match Chrome; the image/card case retains thin fractional horizontal
edge differences within the unchanged gates. These observations are recorded in
its partial visual receipt (12 reviewed,180 remaining). This is not a complete
visual qualification of the192-view cohort.

Public initial576-view replay remains live under session4559; composition vector
replay192 views started under session92100. See `positioned-v17-receipt.json`.
Full native regression and the remaining native/vector visual audit are open.
