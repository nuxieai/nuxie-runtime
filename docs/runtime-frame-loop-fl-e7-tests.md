# FL-E7 W65 test disposition

FL-E7 owns the twelve Class B cases assigned to drawable ordering, clipping,
live paint invalidation, and scripted Drawable traversal. No expectation,
tolerance, or skip was weakened.

Its direct C++ correspondence is `src/draw_rules.cpp`,
`src/draw_target.cpp`, and `src/shapes/shape_paint_container.cpp`.

| W65 source | Cases | Rust evidence |
|---|---:|---|
| `clip_test.cpp` | 5 | The retained-proxy and visibility tests in `draw.rs` cover the exact DrawOrder/Clipping dirt boundary; ordinary/scripted goldens and silver exercise `animated_clipping`, `artboardclipping`, `circle_clips`, and `clipping_and_draw_order`. |
| `draw_order_test.cpp` | 1 | `draw_order_dirt_relinks_retained_drawable_objects` covers relink and proxy reuse; `draw_order_sort_uses_the_retained_draw_rules_owner` and `draw_order_sort_uses_the_retained_draw_target_placement` adversarially prove no sort-time serialized-owner rediscovery. The golden/silver corpus retains `draw_rule_cycle`. |
| `render_test.cpp` | 1 | The live renderer path consumes retained paint state; the golden lanes retain `solid_affects_has_changed`. |
| `scripting/scripting_drawable_test.cpp` | 5 | The scripting runtime and `nuxie --lib` tests cover drawable callbacks and keyboard dispatch through the retained occurrence; both golden lanes retain `keyboard_event_to_script`. |

`ComponentHandle` is the Rust binding adaptation for a retained C++ Drawable
pointer. It is a generational occurrence handle, not a serialized local-id
lookup: live drawing dereferences the retained handle and the family enum
captured at construction. DrawRules and DrawTarget retain their resolved
target/drawable relationships, while generated setters refresh only the
retained field and publish root DrawOrder dirt.

Pinned C++ rejects an unresolved/non-Drawable `DrawTarget::drawableId` during
`onAddedDirty`. Rust's graph layer intentionally keeps unresolved references
available to diagnostics, so the runtime retains such a target as inert: it
has no drawable index and cannot detach or splice a drawable group. The
`unresolved_draw_target_is_an_inert_graph_diagnostic_adaptation` test pins this
explicit import-boundary adaptation.

- [x] Retained DrawRules target: construction resolves once, the generated setter refreshes the sidecar, and sort follows the sidecar.
- [x] Retained DrawTarget drawable and placement: construction resolves the drawable once, placement mutation refreshes the sidecar, and sort follows both retained fields.
- [x] Exact DrawOrder dirt gate: construction and root DrawOrder update are the only sort entry points.
- [x] Exact Clipping dirt gate: sort performs the pinned internal clear and root Clipping update performs the pinned explicit clear.
- [x] Stable Drawable links: relink mutates retained `prev`/`next` fields without replacing Drawable allocations.
- [x] Clone-local clipping proxies: proxy objects and clipping membership remain occurrence-owned and are rebuilt/reused within the clone.
- [x] Complete polymorphic container dispatch: `ShapePaintContainer::from` has exactly the nine pinned Shape/Layout/Text/TextInput branches; production owners retain every family and route authored-order `addPaint`, aggregate live `pathFlags`, dirt-phase stroke-effect invalidation, and opacity propagation through the direct module. A non-Shape gradient Stroke mutation proves live path selection, shader endpoints, and the configuration epoch change together.
- [x] Final live draw: traversal walks the retained first/previous link and dispatches the retained Drawable family without scene replay or draw-time type rediscovery.
- [x] Displaced refresh deletion: sort-time DrawRules/DrawTarget property reads, local-id owner map lookups, centralized Artboard callbacks, and eager unchanged text-style backend-path clears are absent under permanent structural ratchets/tests.

The E7 family is mechanism-complete, but this document does not claim FL-E
wave acceptance. The wave ledger still contains earlier E5 rows whose own
direct files explicitly record unsupported TextStyleFeature,
TextTargetModifier, TextVariationModifier, RawText, and ListPath behavior.

## Suite evidence

- `cargo test -p nuxie-runtime`: 871 passed, one ignored (832 library and
  39 integration tests).
- `cargo test -p nuxie-runtime --features tools --test cpp_probe`: 889 passed,
  five ignored, against the pinned C++ archive built under the worktree target.
- `cargo test -p nuxie --lib`: 154 / 154 passed.
- Ordinary golden: 317 / 317 entries, 647 / 647 exact segments, zero
  divergence.
- Scripted golden: 317 / 317 entries, 647 / 647 exact segments, zero
  divergence.
- Silver corpus: 238 entries, 236 provenanced, 195 runtime-selected, 95
  executed, 59 exact (49 byte-exact and 10 epsilon), 36 divergent, 100
  explicitly unsupported, zero pending. The direct opacity-bucket replacement
  and retained DashPath/effect/feather rewind lifecycles preserve the exact
  `text_stroke_test`, listener, and follow-path entries. No unsupported blocker
  became executable, no case was promoted, and the exact count remains 59;
  `animated_clipping-nodes` remains honestly divergent at expected `drawPath`,
  got `makeRenderPath`; `layout_display` remains honestly divergent at
  expected `makeRenderPath`, got `rewind`.
- Binary comparison: 70 / 70 passed.
- FL-G08 trace: canonical `draw_order_sort` C++ 24 / Rust 607 and
  `clipping_redundancy_clear` C++ 48 / Rust 1214 under the documented eager
  occurrence-local list construction adaptation; steady state is exact 0 / 0
  for both. Retained-pointer/`ComponentHandle` `drawable_owner_lookup` is
  structurally 0 / 0.
- Renderer golden: unavailable because the host reported no suitable Metal or
  other graphics adapter; no renderer expectation was relaxed.
