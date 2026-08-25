# Solid-color source certification

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Implementing auditor: root campaign lane

Adversarial review: accepted

## `src/shapes/paint/solid_color.cpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `SolidColor::onAddedDirty` | `preallocate_source_artboard_render_paints_into`; `preallocate_render_paint_for_instance`; `preallocate_render_paint_for_mounted_instance`; `initialize_authored_shape_render_paint`; `ArtboardInstance::update_runtime_shape_paint_mutator` | exact | `solid_color_property_callback_settles_the_owned_paint_immediately`; source-paint import/clone allocation tests |
| `SolidColor::renderOpacityChanged` | `ArtboardInstance::settle_runtime_solid_color_callback_with_graph`; `ArtboardInstance::propagate_runtime_shape_paint_opacity`; `runtime_shape_paint_state`; `runtime_current_solid_render_color`; `ArtboardInstance::runtime_is_translucent` | exact | `solid_color_property_callback_settles_the_owned_paint_immediately`; `shape_paint_command_report_uses_retained_opacity_owner_without_graph_resolution`; `solid_color_changes_keep_prepared_topology_epoch_stable`; `solid_color_visibility_changes_invalidate_prepared_topology` |
| `SolidColor::applyTo` | `runtime_shape_paint_state`; `runtime_current_solid_render_color`; `runtime_render_paint_configuration` | exact | `shape_paint_command_report_uses_retained_opacity_owner_without_graph_resolution`; render-paint configuration tests |
| `SolidColor::colorValueChanged` | `ArtboardInstance::after_color_property_set`; `ArtboardInstance::set_keyed_solid_color_property`; `ArtboardInstance::settle_runtime_solid_color_callback` | exact | `solid_color_property_callback_settles_the_owned_paint_immediately`; `solid_color_changes_keep_prepared_topology_epoch_stable`; `nested_solid_color_changed_edge_propagates_when_mutable_visit_returns` |

The initialization traversal retains C++'s superclass-first failure boundary:
the parent `ShapePaint` is allocated and initialized before the concrete
mutator applies its authored color. The live occurrence then owns the paint;
neither ordinary nor keyed color callbacks reconstruct it.

Both direct property paths preserve the generated setter's unchanged-value
short circuit and synchronously modulate the live color by the retained render
opacity. Rust's `solid_color_paint_revisions` is only a renderer handoff
counter. It does not defer the mutation, dirty the dependency graph, or replace
the paint occurrence.

The Rust visibility catalogue has to invalidate its prepared membership when
the effective alpha crosses zero because it does not retain C++'s intrusive
drawable lists. That is a representation consequence, not an extra observable
runtime behavior. The source's unusual `if (opacity > 0) ... else if (opacity
< 1)` remains literal in `runtime_is_translucent`: every nonzero solid,
including a partially transparent solid, is classified as opaque, while zero
alpha is classified as translucent.

## Adversarial review

Accepted after independently reading the complete pinned C++ translation unit,
the generated color default, and the complete cited Rust paint allocation,
initialization, callback, retained-state, rendering, visibility, and clone
paths.

- The denominator contains exactly the four claimed definitions, with ids
  `31d2dd6ec71c21c4`, `3a97284d7fd4956a`, `64a0d93ba96f7ad3`, and
  `670c7fa9a9168258`.
- Superclass initialization and failure occur before mutator attachment;
  successful attachment immediately applies the authored color. Duplicate
  mutators remain malformed. Source and clone paint allocation tests observe
  allocation/style/color ordering rather than merely checking final state.
- Color and render-opacity callbacks synchronously update the same retained
  paint, reset and recompute the source flags, and publish the Artboard change.
  `applyTo` uses `renderOpacity * opacityModifier`. The Rust text/configuration
  path preserves that second opacity factor instead of reusing the base paint
  color unchanged.
- Generated default materialization, unchanged-value callback suppression,
  zero-alpha membership changes, nonzero partial-alpha classification, and
  clone reconstruction all match the pinned behavior. The extra prepared-list
  invalidation is confined to Rust's non-intrusive drawable representation and
  does not alter the source-visible paint state.
- Focused callback, topology, opacity, source allocation, clone allocation,
  clone reconstruction, and C++/Rust malformed-import comparison tests passed
  with `CARGO_INCREMENTAL=0`.

This acceptance covers the four out-of-line `.cpp` denominator entries only.
It does not certify executable handwritten header bodies, which require their
own denominator coverage.

## Result

All four pinned out-of-line symbols have concrete Rust owners and lifecycle
evidence. No production change was required. The historical RB5/B6 divergence
described a now-obsolete deferred-revision design; the current callback mutates
the occurrence-owned render paint immediately. Independent adversarial review
accepted this receipt.
