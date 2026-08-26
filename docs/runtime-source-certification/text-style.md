# `TextStyle` source-pair certification candidate

Status: **author candidate; independent review required**.

Correction after independent rejection
`fd13ab53f1b20dc5d8d5cb9f668150cb9ca93228`: the first candidate's
`updateVariableFont` path called a feature observer cached by
`text_shape_revision`. Because `TextStyleFeature` tag/value callbacks are
intentionally empty, a later helper update could rebuild the retained variable
font with stale feature values. This correction makes only the lazy/helper
replacement path read the current occurrence properties directly; ordinary
shaping remains bound to the retained variable-font snapshot until the helper
runs.

This receipt is governed by
`docs/runtime-exact-parity-workflow-correction.md`. It maps the complete pinned
handwritten pair, records only source-proven corrections, and does not certify
the separately owned `TextVariationHelper` method pair.

## Frozen authority and denominator

- Upstream authority: Rive runtime
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- `src/text/text_style.cpp`: 185 lines, 5,090 bytes, SHA-256
  `0d702cd3faf2df687f1fd467619b0b96a8ca64e44cf89bbb6153f4b0c9893850`.
- `include/rive/text/text_style.hpp`: 60 lines, 1,736 bytes, SHA-256
  `7c8ed1e10980fe8d75e25edb023ffebf2bbb314f5b7b74a4e03731ff1b022883`.
- Complete handwritten executable denominator: **16 cpp definitions** (15
  explicit behavioral bodies plus the defaulted constructor) and **1
  executable primary-header inline**, for **17 authority rows**.

The complete generated base was read as required context. It supplies the four
field defaults/getters/guarded setters, callback-before-notify order, copy,
deserialization, clone allocation, type identity, and empty callback defaults;
those generated methods are not added to the handwritten denominator.

## Complete authority map

| # | Pinned authority | Required behavior and order | Concrete Rust owner | Candidate disposition |
|---:|---|---|---|---|
| 1 | `TextStyle::TextStyle` (cpp 14-15) | Default custom state: no helper, no variable font, empty coord/axis/feature vectors, null retained Text. | `components.rs:1023::RuntimeTextStyleState`; occurrence-cold construction at `components.rs:1038::RuntimeTextStyleState::clone_for_occurrence`. | **corrected exact retained state**. Rust's backend-neutral variable-font snapshot is described below. |
| 2 | `addVariation` (cpp 17-20) | Append the axis pointer, preserving construction order and duplicates. | `components.rs:1042::RuntimeTextStyleState::register_variation`; direct-parent registration at `artboard.rs:1425-1441::build_component_occurrence_relations`. | **corrected exact occurrence ownership**. The old graph-child projection could not represent live-parent freeze followed by clone re-registration. |
| 3 | `addFeature` (cpp 22-25) | Append the feature pointer with the same ordering rules. | `components.rs:1050::RuntimeTextStyleState::register_feature`; direct-parent registration at `artboard.rs:1443-1456`; occurrence-aware consumer at `text/text_style_feature.rs:16::from_graph_with_occurrence`. | **corrected exact occurrence ownership**. |
| 4 | `onDirty` (cpp 27-43) | If retained `m_text` is null, do nothing. For accumulated `TextShape`, call `Text::markShapeDirty` first, then dirty the optional helper. | Retained text field `components.rs:1062`; callback at `artboard.rs:8180-8245::dispatch_component_on_dirty`; exact dirt owner `text/text.rs:57::mark_shape_dirty`. | **corrected exact**. This now works with or without a helper and uses the full Path/range-map/group/WorldTransform cascade rather than raw TextShape dirt. It also preserves the pinned accumulated-mask re-entry described below. |
| 5 | `onAddedClean` (cpp 45-72) | Assign direct TextInterface before Super; propagate non-Ok; create helper iff either option list is nonempty; run helper dirty then clean; propagate either failure. | Direct TextStyle parent guard `artboard.rs:1413-1424`; retained assignment and occurrence-derived helper creation `artboard.rs:1953-1988`; retained helper address/linking `objects.rs:280::attach_text_variation_helper`. | **corrected under the existing combined Rust construction adaptation**. Assignment/helper existence/order are concrete occurrence state. Rust construction combines generated clean phases rather than exposing two public status callbacks; invalid direct parents stop at the corresponding construction boundary. |
| 6 | `font` (cpp 74-96) | Return existing variable font first; otherwise lazily update when options exist; return newly built variable font if available; otherwise return the current base asset font. | `text.rs:5307::StaticTextStyle::font_bytes`, backed by `components.rs:1070::variable_font` and `:1074::initialize_variable_font`. | **corrected backend adaptation**. The retained snapshot preserves lazy/stale-cache timing; Rust carries font bytes plus ordered options into HarfBuzz/Skrifa rather than storing C++ `rcp<Font>`. |
| 7 | `updateVariableFont` (cpp 98-126) | Resolve base font first and return without changing cache/buffers when unavailable. Otherwise clear/fill coords then features in retained order and replace the variable font; clear it when there are no options. Each invocation reads the current retained axes/features even though feature callbacks are dirt-inert. | Snapshot construction `text.rs:5295::variable_font_replacement`; direct current feature read `text.rs:5110::live_feature_values` -> `text/text_style_feature.rs:56::live_option`; replacement owner `components.rs:1084::update_variable_font`; normal helper update call `text/text_variation_helper.rs:72::update_text_variation_helper`. | **corrected backend adaptation after rejection**. Ordered live option values, dirt-inert feature writes, and the stale-cache-on-unavailable-base branch are retained. The shape-revision cache is not consulted by this update owner. Helper method certification remains separate. |
| 8 | `buildDependencies` (cpp 128-136) | Build helper dependencies first when present, add this style as a dependent of its parent Text, then run Super, all in authored insertion order. | Occurrence action schedule `artboard.rs:2009-2089::DependencyBuildAction` and application at `:2312`; occurrence-derived helper edges at `:2052-2058`; immutable helper edge families excluded at `:2334-2338`. | **corrected exact occurrence topology**. It can create and remove helper edges on clones rather than replaying stale graph edges. |
| 9 | `assetId` (cpp 138) | Return `fontAssetId`. | Schema property lookup in `nuxie-binary/src/assets/file_asset_referencer.rs:28::cpp_file_asset_referencer_index`; runtime resolution at `:9::resolved_file_asset_for_referencer`. | **corrected inherited owner**. Matching now uses is-a `TextStyle`, so `TextStylePaint` participates. |
| 10 | `setAsset` (cpp 140-154) | Ignore null/non-FontAsset. For a valid FontAsset, remove the old referencer, store and append the new referencer even for the same pointer, then add only `TextShape` dirt when retained Text exists. | Valid data-binding boundary plus storage `artboard/text/text_style.rs:58::set_text_style_font_override`; queue move `assets/font_asset.rs:54::RuntimeFontAssetReferencerQueue::reregister_style`; callback entry `artboard/text/text_style.rs:84::mark_text_style_shape_dirty`. | **corrected exact for valid Rust font values**. Equality suppression and extra direct path/layout publications were removed. The Rust live-font-bytes value is the named data-binding asset adaptation; callers reject unresolved/wrong-type file assets before this valid-owner seam. |
| 11 | `import` (cpp 156-164) | Register as a Backboard file-asset referencer first, propagate failure, then run Super import. | Backboard import resolution `nuxie-binary/src/assets/file_asset_referencer.rs:9-53`; is-a TextStyle referencer validation `nuxie-graph/src/lib.rs:5524::local_object_is_valid`. | **corrected combined-import adaptation**. TextStylePaint is no longer excluded, and wrong-type assets remain non-resolving rather than rejecting the style. Rust's parser/graph construction combines the two status phases. |
| 12 | `fontSizeChanged` (cpp 166) | After the generated write, call retained Text `markShapeDirty`, then generated notify. | Inherited callback dispatch `artboard.rs:10539::apply_double_property_changed`; is-a owner `text/text_style.rs:15::double_property_changed`; retained owner `:5::owning_text`; generic-tail suppression `artboard.rs:5947-5953`. | **corrected exact** for TextStyle and TextStylePaint. |
| 13 | `lineHeightChanged` (cpp 168) | Same callback and order. | Same owner as row 12; property discrimination at `text/text_style.rs:29::metric_property_changed`. | **corrected exact**. |
| 14 | `letterSpacingChanged` (cpp 170) | Same callback and order. | Same owner as rows 12-13. | **corrected exact**. |
| 15 | `clone` (cpp 172-181) | Generated-base clone/copy first; custom lists/helper/cache/Text start cold; if a file asset exists, call `setAsset` on the twin. Later construction rebuilds ownership from copied parent/child properties. | Cold custom state `components.rs:1038::clone_for_occurrence`; helper removal `objects.rs:114-159::clone_without_runtime_links`; occurrence reconstruction `artboard.rs:1413-1988`. | **corrected exact lifecycle with existing Rust infallible-clone adaptation**. Source retained ownership stays frozen after live writes; clone lists, Text, helper, dependency edges, and cache rebuild from copied properties. Invalid cloned topology fails Rust reconstruction rather than returning a nullable instance. |
| 16 | `validate` (cpp 183-186) | The resolved direct parent must implement TextInterface. | Parser/graph validation `nuxie-graph/src/lib.rs:5524::local_object_is_valid`; runtime occurrence guard `artboard.rs:1413-1424`. | **corrected exact valid/invalid boundary**. No ancestor fallback is accepted. |
| 17 | inline `fontAsset` (hpp 46) | Return the retained FileAsset as FontAsset; safety relies on `setAsset`'s type gate. | Typed referencer matching `nuxie-binary/src/assets/file_asset_referencer.rs:40::cpp_file_asset_matches_referencer`; static style resolution `text.rs:5202-5214` and `:5283::base_font_bytes`. | **corrected Rust-safe equivalent**. Wrong-type references resolve to no font without retaining an invalid typed pointer. |

## Generated context and ordering

The generated base defaults are `fontSize=12`, `lineHeight=-1`,
`letterSpacing=0`, and `fontAssetId=u32::MAX`. Every setter is an equality
no-op; otherwise it stores, calls the virtual callback, and then notifies. Rust
schema-owned occurrence properties preserve those defaults and guarded writes.
The three metric callbacks suppress only the old broad generic Text tail after
their concrete owner has run. The generated empty `fontAssetIdChanged` callback
also suppresses the generic Text tail while retaining normal property
notification (`artboard.rs:5988-6004`).

The retained `m_text == nullptr` phase is represented explicitly. Supporting
evidence clears the occurrence field before a direct `onDirty(TextShape)` and
proves the callback is inert, then restores it; this avoids declaring the Solo
construction branch unreachable.

## Variable-font and helper ownership boundary

`RuntimeTextStyleVariableFont` retains the source-visible cache state as font
bytes, ordered coordinates, and ordered features. `font_bytes` returns that
snapshot before consulting a replacement asset. `updateVariableFont` returns
without touching it when a replacement base font is unavailable. Actual Text
and TextInput shaping consume the occurrence lists and cached option stream via
`StaticTextStyle::from_graph_with_occurrence` (`text.rs:5153`) and
`variation_values`/`feature_values` (`text.rs:5100`, `:5122`). This is the
approved Rust font-backend shape, not a claim that an `rcp<Font>` exists.

The replacement builder intentionally differs from the ordinary shaping read:
`live_feature_values` reads current `tag`/`featureValue` occurrence properties
on every lazy/helper invocation, exactly like pinned `updateVariableFont`.
Once published, `feature_values` returns the retained snapshot, so an empty
feature callback remains invisible until a later legitimate helper update.

TextStyle owns whether its helper exists, where it is inserted, and which
occurrence it depends on. This candidate necessarily corrects those boundary
semantics, but does not certify `TextVariationHelper::update` or its other
methods as an independently complete source pair.

## Re-entrant `onDirty` order

The source order produces two complete range/group-clear passes in the retained
fixture. The first `TextStyle::onDirty(TextShape)` calls
`Text::markShapeDirty`; its recursive WorldTransform publication reaches the
dependent TextStyle while that style already retains TextShape. C++
`Component::addDirt` gives the re-entered callback the accumulated
`TextShape|WorldTransform` mask, so TextStyle calls `markShapeDirty` again.
Path/World bits are already set, but range clearing and group coverage still
run. Only after the outer call returns does it dirty the helper. The evidence
asserts both identical ordered passes instead of de-duplicating them.

## Consumer and fixture accounting

Literal owner topology is **0 direct pass / 0 executable expected-red / 0
adapted / 0 pending**: no pinned test body calls or asserts a TextStyle method.

Five upstream cases mention the style family incidentally: `text_test.cpp`
ordinals 1, 4, 7, and 12, plus `text_modifier_test.cpp` ordinal 2. All five are
Wave C7 pending/unverified and none is promoted here.

The mechanical fixture inventory contains 379 `.riv` files, 378 readable and
one unrelated unreadable `solar-system.riv`. There are 138 readable files with
697 style-family objects (1 `TextStyle`, 696 `TextStylePaint`); 136 are
referenced, while `library.riv` and the root `scroll_snap.riv` are unreferenced.
That is an incidental initial-shaping impact surface, not a consumer count.

## Supporting evidence and focused gates

- `text.rs:8510::cxx_text_style_rebuilds_options_helper_and_retained_text_callbacks_on_clone`
  proves defaults/cold cache, lazy cache creation, ordered axes/features,
  retained direct Text ownership, all inherited metric callbacks, inert
  `fontAssetId`, unavailable replacement retention, helper creation/removal,
  exact dependency order, live-parent freeze, clone rebuilding, and actual
  Text shaping through source/clone occurrence topology.
- `text.rs:7810::cxx_text_shape_dirty_clears_retained_range_maps_before_group_and_world_dirt`
  enters through real TextStyle dirt and proves the two exact ordered
  markShapeDirty passes and retained range/group effects.
- `text.rs:8806::cxx_text_style_validate_and_wrong_asset_follow_pinned_boundaries`
  proves invalid direct-parent rejection and wrong-type asset retention with no
  font.
- `assets/font_asset.rs:458::text_style_set_asset_moves_the_live_font_referencer`
  proves old-asset removal, new-asset append, and later decode notification.
- `nuxie-binary/tests/fixtures.rs:6580::runtime_file_asset_referencers_resolve_like_cpp_backboard_importer`
  proves inherited TextStylePaint resolution and wrong-type rejection.
- `text.rs:8184::d_st_feature_preserves_order_defaults_duplicates_and_live_callback_inaction`
  now proves the rejected temporal counterexample through real owners: initial
  retained cache; live feature write with no helper/Text dirt and unchanged
  shaping; recursive root WorldTransform reaching the retained helper; helper
  update rereading the current value; and later shaping consuming the replaced
  snapshot.
- Existing dependency, Axis, Feature, Variation, async-font, and data-bound font
  focused tests were rerun because the corrected owners are shared.

Focused commands and results:

- `cargo test -p nuxie-runtime --lib text::tests::cxx_text_style_rebuilds_options_helper_and_retained_text_callbacks_on_clone -- --exact --nocapture`: 1 passed.
- `cargo test -p nuxie-runtime --lib text::tests::cxx_text_shape_dirty_clears_retained_range_maps_before_group_and_world_dirt -- --exact --nocapture`: 1 passed.
- `cargo test -p nuxie-runtime --lib text::tests::cxx_text_style_validate_and_wrong_asset_follow_pinned_boundaries -- --exact --nocapture`: 1 passed.
- `cargo test -p nuxie-runtime --lib font_asset::tests::text_style_set_asset_moves_the_live_font_referencer -- --exact --nocapture`: 1 passed.
- `cargo test -p nuxie-binary --test fixtures runtime_file_asset_referencers_resolve_like_cpp_backboard_importer -- --exact --nocapture`: 1 passed.
- `cargo test -p nuxie-runtime --lib text::tests::d_st_feature_preserves_order_defaults_duplicates_and_live_callback_inaction -- --exact --nocapture`: 1 passed, including the rejected counterexample.
- Focused regressions: helper insertion/clone, Axis occurrence/clone, Feature
  callback/update, Variation shaping, async FontAsset notification, and
  data-bound font replacement: 7 passed.

## Author conclusion

All 17 handwritten authority rows and directly necessary generated state are
mapped. The candidate corrects occurrence ownership, helper lifecycle and edge
order, retained Text callbacks, full dirty order, variable-font cache timing,
referencer inheritance/movement, wrong-type handling, and clone rebuilding.
Independent review must verify the backend and combined-import adaptations,
the re-entrant dirt stream, exact locators, and consumer/fixture accounting.
