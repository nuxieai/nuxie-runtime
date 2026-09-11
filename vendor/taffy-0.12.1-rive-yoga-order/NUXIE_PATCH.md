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

## Differential evidence

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
