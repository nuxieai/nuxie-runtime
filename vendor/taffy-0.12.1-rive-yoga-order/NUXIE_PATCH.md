# Nuxie Rive Yoga parity patches

This directory starts from the crates.io `taffy` 0.12.1 package. Nuxie keeps
Taffy as its Rust-native layout backend, but changes arithmetic and cache seams
needed to preserve the pinned Yoga owner's output and practical solve behavior.

## Source authority

- Rive runtime commit:
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- Rive layout call site: `src/layout_component.cpp`,
  `LayoutComponent::calculateLayoutInternal`.
- Yoga flex owner: `renderer/dependencies/rive-app_yoga_rive_changes_v2_0_1_2/yoga/Yoga.cpp`,
  where grow distribution evaluates
  `remainingFreeSpace / totalFlexGrowFactors * flexGrowFactor`.
- Yoga percentage owner: `renderer/dependencies/rive-app_yoga_rive_changes_v2_0_1_2/yoga/Utils.h`,
  where percentages resolve as `value.value * ownerSize * 0.01f`.
- Yoga measurement cache owner:
  `dependencies/rive-app_yoga_rive_changes_v2_0_1_2_grid/yoga/Yoga.cpp`,
  where `YGLayoutNodeInternal` retains and searches multiple measurements and
  accepts compatible constraints through `YGNodeCanUseCachedMeasurement`.

## Behavioral delta

Taffy 0.12.1 originally evaluates the equivalent expression as
`free_space * (child.flex_grow / sum_flex_grow)`. This fork evaluates it as
`free_space / sum_flex_grow * child.flex_grow`, matching Yoga's exact order.

Rive-authored dimension percentages use a dedicated compact-length tag that
retains their [0, 100] value and resolves it as `value * owner_size * 0.01`.
This leaves Taffy's native [0, 1] percentage representation and behavior
unchanged for every non-Rive caller.

Taffy's nine measurement entries are one direct-mapped entry per constraint
category. Deep Rive list/artboard trees can alternate many exact constraints
within a category, continually evicting the immediately reusable result. The
fork makes each category 32-way set-associative. Cache hits return the same
previously computed value and misses still execute the unchanged Taffy solve;
this changes retention only, not layout arithmetic or authored behavior.

The distinction is observable because each operation rounds to `f32`. In the
pinned `list_focus_order.riv` fixture, three growing column children produce
`137.20052` with Yoga but `137.20053` with upstream Taffy; the one-ULP
difference propagates into layout paths and transforms.

For `computed_values_test.riv`, Yoga resolves the animated `38.571426%` width
against 490 as `188.99997`; normalizing it to Taffy's [0, 1] representation
first resolves as `188.99998`. The resulting remaining flex width differs by
one ULP (`301.00003` versus `301`).

## Grid min-content at runtime 8e8492f8

Runtime `8e8492f8312c67ac54558adce2f0798baabcdce3` updates Yoga to
`rive_changes_v2_0_1_3_grid`, commit
`dc9e0e9f7c29c2a910e76f3738a800ca3c73e7fa`. Its `yoga/grid/TrackSizing.h`
adds `measureItemMinContent`: definite authored lengths use normal measurement;
otherwise a complete layout runs at zero available space on the measured axis.
The other axis and containing-block dimensions remain intact. Both min-content
and limited-min-content contributions use this helper. Flexible-track growth
also requires an intrinsic minimum; Taffy's existing track distribution already
applies that filter.

This fork retains Taffy's intrinsic constraint representation. Grid items with
indefinite authored size use full child layout under `MinContent` on the measured
axis, without a stretch-derived known size overriding that constraint. The
wrapping flex main-size path applies Yoga.cpp's overflowing AtMost-to-Exactly
rule to that zero-space probe. Its available inner main space is first bounded
by the resolved minimum and maximum, with margins and box insets removed,
matching `YGNodeCalculateAvailableInnerDim`; both line collection and intrinsic
main sizing use that bound. Layout then continues through the existing min/max and
padding/border clamps and child layout. Definite authored dimensions and nowrap
content retain their sizing paths. Unconstrained cross-axis space is MaxContent;
the runtime's native measurement bridge maps MinContent to AtMost(0) and
MaxContent to Undefined. No fixture names or expected container widths enter
the implementation.

This adaptation is awaiting source/integration review and differential
validation; the pre-change seven-case runtime test passed six cases and failed
WrapRigidGrid (600 rather than 400).

## Finite flex-basis measurement at runtime 45d4d01d

The fitted-text matrix added by runtime
`45d4d01dfd1fe70d3f9e73764538c16f63a04d07` exercises a column's
fill-width/hug-height child with a minimum height larger than available space.
Pinned `rive_changes_v2_0_1_3_grid` Yoga.cpp's
`YGNodeComputeFlexBasisForChild` forwards finite parent main-axis space as
AtMost unless the parent scrolls. Stock Taffy instead replaces that offer with
MaxContent during flex-basis measurement. This fork preserves the finite offer
for non-scroll parents at that boundary, leaving definite flex bases, scroll
containers, and intrinsic MinContent/MaxContent requests on their existing paths.
The existing leaf sizing code continues to apply child margins and min/max bounds.
No text-specific or fixture-specific size clamp is introduced.

## Prior differential evidence

`wave_b_focus_test_078_direct_port_expected_red` compares the complete native
render stream with the pinned C++ silver. It fails at frame 0, operation 78
with upstream Taffy and passes with this single expression-order change.

`data_binding_computed_root_values` fails at frame 2, operation 191 with the
normalized Taffy percentage and passes with the dedicated Rive Yoga percentage
resolution path.

The pinned `data_bind_blob_test.riv` silver builds a finite, acyclic tree of
326 native layout owners at maximum depth 16. Upstream Taffy's direct-mapped
cache exceeded 1.4 million child-layout calls without finishing the first
solve in the bounded debug test. The set-associative cache completes the exact
silver comparison in about two seconds on the same debug build.
# Virtual grid line observation (upstream dc75beed)

`DetailedGridTracksInfo::line_offsets` exposes the solver's actual track starts
and final end, including padding/border and distributed gaps. The runtime uses
these solved values for `LayoutComponent::grid_column_line_offsets`, replacing
the pinned Yoga grid-line query without recomputing tracks or switching layout
engines. This adds observation only and does not change the solver.

## Virtual grid contributions (upstream 160085c6)

Public Yoga tag `rive_yoga_fb91f6cb0f8f` resolves to commit
`9a6ac7fac43ca22d9b1e56468e5c8200ba0a07d3`. Its `YGNode.h`,
`grid/AutoPlacement.h`, `grid/TrackSizing.h`, and `grid/GridLayout.cpp`
define the nodeless contribution contract translated here. Node-owned row and
column contents normalize nonfinite values to zero, compare before dirtying,
clear when both arrays are empty, and deep-copy with the tree. Contributions
extend implicit tracks from grid line one and enter intrinsic/flexible sizing
directly. Realized children still lay out in their own cells, but do not also
participate in sizing when the complete contributions exist. Childless grids
with contributions use the grid solver, not leaf measurement.

Taffy sizes axes separately: it retains four sizing passes **per axis**, rather
than Yoga's four paired-axis records. The node-owned cache survives child
placement dirt, and checks current style, track counts and leading implicit
origin, inner/owner/known/min/max dimensions, available-space modes and the
finite-undefined offers. Contribution changes clear it. Cached base and growth
sizes are restored; percentage-track flags remain derived from the same track
definitions. No synthetic nodes or replacement grid-size solver are introduced.
Both row and column solved line offsets and the post-distribution gap are
published for the runtime's `grid_lines` query.

The contribution contract also preserves Yoga's 32-bit grid range. Grid counts,
spans, placement indexes and repetition counts use `u32`; authored/origin-zero
line coordinates and named-line occurrences use `i32`. The runtime's `YGGridLine`
adapter retains those widths through original-cell pinning. This replaces Taffy's
16-bit representations consistently, including occupancy, track sizing and
detailed results: 65,536 contributions must not wrap to an empty grid, and a
realized child beyond line 32,767 must remain in its original cell. Ordinary
in-range arithmetic and placement semantics are unchanged; counts are not
clamped and no synthetic cells or alternate sizing path are introduced.

The all-realized versus nodeless upstream matrices also exposed two existing
real-item sizing differences. `GridItem::minimum_contribution` now mirrors
Yoga's immediate min-content return for a definite point preferred size, before
the automatic-minimum fixed-track cap. Intrinsic-minimum distribution tests
the current percentage basis instead of an authored-tag crossing flag, so an
unresolved percentage minimum participates like auto, as Yoga's
`isIntrinsicSizingFunction` requires. Both corrections apply the upstream rule
to real items; virtual contributions and test expectations are not reduced to
match the previous differences.
