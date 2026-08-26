# Text source-pair correspondence candidate

Status: **author candidate; pending independent semantic review**

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Correction note: this candidate incorporates every narrow finding from the
independent rejection at `d5ea63237b7ea74331a7101394a396853bf6b549`.
The authority denominator and all three pinned file hashes remain unchanged;
no production or test behavior was changed at that audit checkpoint.

Production-correction candidate note: the first narrow source-driven unit now
corrects rows 11, 14, 15, 31, and the paragraph-spacing portions of rows 17,
37, 40, and 41. It adds direct state/layout evidence and promotes only the
complete `fit_font_size_test` Silver stream. These changes remain an author
candidate pending independent semantic review; unrelated red rows stay red.

Dynamic-list production-correction candidate note: the next narrow
source-driven unit corrects the value/topology portions of rows 2, 5, 7, 36,
52, and 53. Every valid list instance now retains an all-runs position;
missing `textContent` remains an empty run; a missing style property or an
empty style-paint list remains null-styled; and a present unmatched style name
still selects the first paint in pinned order. Direct state evidence covers
the property-presence distinction, list order, all-run positions, StyledText
offsets, fallback selection, and the valid empty-style topology. Concrete
per-property listener identity, initial-write retention, synchronous dirt,
and literal `StyledText::TextRun::styleId` ownership remain adaptations. No
`text_test.cpp` consumer moved; this remains an author candidate pending
independent semantic review.

Dynamic-list residual-correction note: the narrow rejection at `eb59504e9`
found one empty-line insertion-point consumer that could choose a retained
all-runs entry omitted by pinned `makeStyled`. `static_line_metrics` now
selects only a styled, nonempty, font-backed run before applying the existing
insertion-point order and base-style fallback. Focused real-owner evidence uses
a null-style row and an empty row before a second-style `"\nA"`, and proves the
empty paragraph retains the second style's metrics. Consumer topology remains
four pass, three executable expected-red, and 11 pending.

Empty-shape production-correction candidate note: row 17's clean Text owner
now follows the pinned `buildRenderStyles` early branch. When `makeStyled`
would produce no run, uncontrolled and every fixed/fill/hug controlled layout
combination publish zero bounds before effective-sizing retention, produce no
render topology, paths, color glyphs, draw order, or clip, and the no-authored-
run fallback also returns zero. Focused direct owner evidence covers all three
authored sizing modes and the complete 3x3 controlled scale-type matrix.
Retained path clearing, update-phase layout-dirt publication, hit rectangles
and contours, and the complete packed eight-phase order remain separate row 17
gaps. No consumer moved; topology remains four pass, three executable
expected-red, and 11 pending.

Empty-shape retained-owner residual note: the retained draw-frame owner now
counts `Text.textRunListSource` only when its data-bind direction applies the
source to the Text target. A target-to-source-only bind with no authored runs
therefore takes the pinned empty-shape return and retains zero bounds without
Text paint commands, Text clipping, path replay, or controlled scale types.
Focused evidence is
`text.rs::target_to_source_only_run_list_bind_keeps_retained_empty_text`.
Consumer topology remains four pass, three executable expected-red, and 11
pending.

This is an atomic source-pair audit under
`docs/runtime-exact-parity-workflow-correction.md`. It does not inherit the
older file-level `mapped` or `faithful` verdict. The later correction candidate
above is deliberately limited to the accepted red rows it names.

The complete pinned handwritten pair and every concrete Rust owner cited below
were read before classification:

- `src/text/text.cpp` — 1,562 lines, 52,037 bytes, SHA-256
  `a485332b6fc2e5610a59ee3e50f652814d9bc2feb19e1bcd1f9dce3499d346fb`;
- `include/rive/text/text.hpp` — 348 lines, 10,626 bytes, SHA-256
  `10688904ad16072c4f9f646775cb4512d4a5b00c057985869a9fdc3ef87cf0c8`.

The `.cpp` denominator is 79 physical definition bodies: 53 logical authority
units on the text-enabled path and 26 alternate `WITH_RIVE_TEXT`-disabled
bodies. The table consolidates each disabled body with its corresponding
logical member instead of falsely counting it as a second API. The header adds
29 executable inline methods and eight meaningful cold defaults. The stale
private declaration `updateOriginWorldTransform()` has no body in the pinned
translation unit and is recorded separately.

`exact`, `adapted`, `incorrect`, and `missing` are source-read candidate
classifications, not certification. A row remains unaccepted until an
independent reviewer rereads the complete pair and checks the named Rust symbol.

## Concrete Rust ownership boundary

The current Rust ownership is substantially packed. No whole file is evidence
for any row:

- `crates/nuxie-runtime/src/text.rs::StaticTextSlice` reconstructs run/style
  topology, shapes and breaks lines, computes bounds/trim/fit, and builds path
  and color-glyph draw data;
- `crates/nuxie-runtime/src/text/text.rs` owns Text property callbacks, dirt,
  and the reduced list-item listener;
- `crates/nuxie-runtime/src/draw.rs::RuntimeTextDrawOwner`,
  `update_runtime_text_render_styles`, and `runtime_draw_live_text_family` own
  retained update/draw state and backend resources;
- `crates/nuxie-runtime/src/components.rs::RuntimeTextState` retains only bounds
  and layout scale types from the C++ Text object;
- `crates/nuxie-runtime/src/data_bind/data_bind_context.rs::RuntimeArtboardTextListBindingInstance`
  and `viewmodel/viewmodel_instance_list.rs::RuntimeOwnedViewModelListHandle::text_runs`
  own dynamic list runs; and
- `crates/nuxie-runtime/src/text/text_engine.rs` plus Taffy callers in `draw.rs`
  own layout measurement and controlled bounds.

Line locators below name the definition start in this author candidate. Symbols,
not surrounding module attribution, are the durable identity.

## `.cpp` authority units

| # | Pinned authority | Required behavior, branch/order contract | Concrete Rust owner | Candidate disposition |
|---:|---|---|---|---|
| 1 | `TextValueRunProperty::TextValueRunProperty` (23) | Retain core, listener, instance value, property key, and symbol type. | No direct per-property Rust owner. Nearest aggregate: `text/text.rs:16::RuntimeTextValueRunListener::new`. | **adapted/packed**: no per-property dependent retaining the key/type tuple. |
| 2 | `TextValueRunProperty::writeValue` (36) | Text content writes through `CoreRegistry`; style scans paints in order, installs the first as fallback, then the named match; unknown symbol is inert. | `viewmodel/viewmodel_instance_list.rs:142::RuntimeOwnedViewModelListHandle::text_runs`; `text.rs:3050::StaticTextSlice::resolved_dynamic_runs`; evidence `viewmodel/viewmodel_instance_list.rs:616::cxx_text_run_projection_preserves_missing_properties_and_item_order`, `text.rs:5763::cxx_dynamic_text_runs_retain_all_indices_and_skip_only_null_style_or_empty_runs` | **adapted correction candidate**: absent properties remain absent, missing content retains the default empty text, present style scans in paint order with the pinned first-paint fallback, and no property/no paints leave style null. Writes are still polled during reconstruction rather than performed synchronously by a concrete listener. |
| 3 | `TextValueRunListener` constructor (75) | Initialize base and `m_text`, then immediately create properties. | `text/text.rs:15::new` | **adapted candidate**: immediate creation exists under Rust-owned handles; object identity differs. |
| 4 | `TextValueRunListener::markDirty` (83) | Directly call owning Text `markShapeDirty`. | `data_bind/data_bind_context.rs:7310::flush_runtime_text_run_listener_changes`; `text/text.rs:57::mark_shape_dirty` | **adapted/order pending**: dirty cells are polled/flushed later rather than synchronously forwarding the callback. |
| 5 | `createProperties` (85) | Base cleanup first, then style listener, then content listener. | `text/text.rs:39::create_properties`; projection `viewmodel/viewmodel_instance_list.rs:142::RuntimeOwnedViewModelListHandle::text_runs`; evidence `viewmodel/viewmodel_instance_list.rs:616::cxx_text_run_projection_preserves_missing_properties_and_item_order` | **adapted correction candidate**: cleanup and style-before-content lookup are retained, missing cells correctly create no property value, and list projection preserves item order. Rust still gathers the two cells into one aggregate listener rather than retaining concrete per-property listener objects. |
| 6 | `createSinglePropertyListener` (92) | Map only style/content to generated property keys; require a value property; otherwise null. | `text/text.rs:39::create_properties` | **adapted**: name-based typed-cell lookup replaces explicit key/symbol dependent construction. |
| 7 | `createPropertyListener` (120) | Create only supported listener; if present, write value before pushing it. | `text/text.rs:39::create_properties`; `data_bind_context.rs:3497::remap_source`; value projection `viewmodel/viewmodel_instance_list.rs:142::RuntimeOwnedViewModelListHandle::text_runs` | **adapted/incomplete**: absent and present-empty properties are now observably distinct and reads occur style-before-content, but there is no literal initial `writeValue`-then-retain operation; rendering polls retained shared cells. |
| 8 | `Text::~Text` (139; disabled 1412) | Delete every dynamic run listener; disabled build is empty. | No explicit destructor owner; implicit Rust drop of `data_bind/data_bind_context.rs:3481::RuntimeArtboardTextListBindingInstance::listeners`. | **exact Rust lifetime adaptation, pending teardown evidence**. |
| 9 | `measureLayout` (147; disabled 1439) | Map undefined axes to float max, call `measure`, return result; disabled returns zero. | `text/text_engine.rs:54::static_text_layout_measure_bounds`; `draw.rs:13847::measure_layout_component`, `14073::measure_layout_participant` | **adapted (Taffy), incomplete**: max constraints are projected through Taffy, but there is no disabled-feature branch and row 41 discrepancies flow here. |
| 10 | `controlSize` (160; disabled 1446) | On any width/height/scale/direction change, retain all five values then `markShapeDirty(false)`; disabled is inert. | No single owner. Inputs are reconstructed by `draw.rs:7500::ArtboardInstance::runtime_text_layout_constraint`; only scale types are retained by `components.rs:1008::RuntimeTextState::retain_layout_scale_types`. | **missing direct owner/order**: Rust does not retain the complete five-field state on Text or perform one changed-check followed by no-layout shape dirt. |
| 11 | `effectiveSizing` (179) | For active layout: no boxed axes -> authored; boxed width plus hug height -> autoHeight; all other boxed combinations -> fixed. Legacy max/hug sentinel path otherwise returns authored. | Shared matrix `text.rs:67::effective_layout_text_sizing`; consumers `1381::RuntimeTextLayoutConstraint::effective_sizing`, `components.rs:1012::RuntimeTextState::effective_sizing`, and `text.rs:3827::StaticTextSlice::effective_sizing`; direct matrix/callback evidence `text.rs:5460::cxx_effective_sizing_matrix_is_shared_by_constraints_state_and_callbacks` | **exact enabled-path correction candidate**: the 3x3 fixed/fill/hug matrix now returns `autoHeight` for both boxed-width/hug-height combinations, and the dependent overflow/width/height predicates consume the same retained result. The legacy no-layout sentinel path remains adapted through the Rust constraint projection. |
| 12 | `clearRenderStyles` (211) | Rewind every style path; clear render styles and draw commands; reset hit state on every run, in order. | `draw.rs:10605::RuntimeTextDrawOwner::retained_or_build`; `text.rs:2521::render_data_from_layout` | **adapted/incomplete**: retained backend replacement models path clearing, but no TextValueRun hit-state reset is owned here and exact rewind/clear order is distributed. |
| 13 | static `computeVerticalTrim` (230) | Zero outputs; early return; first nonempty line cap/x-height max; last nonempty line alphabetic/text descent calculation. | `text.rs:3173::StaticTextSlice::static_vertical_trim` | **adapted candidate**: structure is close, but Rust derives cap/x height from `H`/`x` glyph bounds with ascent fallback rather than consuming the pinned font metrics fields. Differential evidence required. |
| 14 | `computeBoundsInfo` (304) | Accumulate paragraph lines, widths, baseline origin, paragraph spacing, ellipsis line/height, and non-fixed trim in source order. | `text.rs:4466::static_layout_info`; `3158::static_line_metrics`; `3265::static_text_total_height`; `text.rs:5686::cxx_paragraph_spacing_order_covers_empty_paragraphs_trim_measure_fit_and_render` | **adapted correction candidate**: the flattened line representation now adds spacing after each paragraph, retains final spacing for fixed-height/ellipsis calculations, and removes exactly one final spacing before auto-bounds trim. Empty-line insertion ownership now excludes every source run omitted by pinned `makeStyled`; the shaped-line representation remains adapted. |
| 15 | `fitFontScale` (389) | Find max nonempty styled run size; early return; binary-search integer top size; each probe makes styled text, shapes, breaks, includes paragraph spacing, and checks width/height. | `text.rs:3383::fit_font_scale`; `3434::fit_font_scale_for_max_size`; `3189::static_text_total_height`; complete Silver stream `tools/silver-corpus/tests/silver_backfill_cases.rs:37::text_fit_font_size_source_correction_is_exact` | **adapted correction candidate with exact consumer evidence**: each binary-search height probe now includes every paragraph's spacing in pinned order; the seven-frame `fit_font_size_test` stream is exact. Shaping/line-break representation remains adapted. |
| 16 | `shouldDrawLine` (481) | Hidden/clipped and top/middle/bottom use distinct top/bottom comparisons yielding draw/skip/stop. | `text/line_breaker.rs:50::static_text_line_iteration` | **exact algorithm candidate, pending all nine branch comparisons**. |
| 17 | `buildRenderStyles` (558) | Eight ordered phases: clear; empty-shape bounds/return; bounds+ellipsis; modifier coverage; bounds/clip; ordered glyph paths/color commands/hit rects; fit/vertical transform plus layout dirt; hit contours. | `text.rs:2301::StaticTextSlice::layout_from_shaped_topology`; `2581::render_data_from_layout`; empty-shape owners `504::static_fixed_text_constraint_bounds`, `2125::render_layout`, `2706::local_bounds`, `2848::layout_bounds_with_constraint`, `3909::has_styled_text`, `4047::clip_bounds`; nonempty bounds/order `4518::static_layout_info`, `4641::static_render_transform`; retained owner `draw.rs:17772::runtime_build_text_draw_frame`; evidence `text.rs:6373::cxx_empty_shape_publishes_zero_before_controlled_box_and_render_work`, `text.rs::target_to_source_only_run_list_bind_keeps_retained_empty_text` | **adapted correction candidate/incomplete/packed**: paragraph spacing remains present in retained nonempty bounds/ellipsis/fit/render calculations. The clean owner now exactly publishes zero bounds for every no-`makeStyled`-run case before controlled sizing, render data, and clip work, including no-authored-run fallback and a target-to-source-only run-list bind. TextValueRun hit rectangles/contours still have no source-equivalent owner; retained path clearing, layout-dirt publication, and the complete phase order remain split across immutable reconstruction and the retained draw owner. |
| 18 | `styleFromShaperId` (863; disabled 1429) | Assert ID in `m_runs`, return its style; disabled returns null. | `text.rs:4528::style_index_for_local` and indexed `styles` reads | **adapted/incomplete**: local-ID lookup replaces shaper index assertion; no disabled-feature counterpart. |
| 19 | `draw` (869; disabled 1413) | Conditional outer save; optional clipped path; replay style/color commands in stored order; conditional restore. Disabled draw is inert. | `draw.rs:18306::runtime_draw_live_text_family`; `17948::runtime_text_replay_order` | **adapted candidate**: retained replay and save/clip/restore exist; exact renderer stream remains required. |
| 20 | `drawColorGlyph` (901) | Get layers or return; save+transform; cache/decode bitmap and apply extent transform, or build nonzero path/paint; restore. | `draw.rs:18222::runtime_draw_integrated_color_glyph`; caches at `draw.rs:10677::RuntimeTextBackendResources::{color_paths,emoji_images}` | **incorrect/adapted**: pinned creates a new render path and paint for every non-image layer on every draw; Rust retains vector `color_paths` by `(glyph_index,layer_index)` across draws, changing the factory stream. The raster cache/order is adapted and gradient layer support is broader. |
| 21 | `addRun` (964; disabled 1415) | Append the same pointer to authored runs and all runs; disabled is inert. | `text.rs:1670::StaticTextSlice::from_graph` run collection | **adapted**: immutable graph reconstruction retains one descriptor list, not two pointer vectors. |
| 22 | `addModifierGroup` (970; disabled 1416) | Append group in child/import order; disabled is inert. | `text.rs:1670::StaticTextSlice::from_graph`, modifier collection branch at 1979 | **adapted candidate**, pending order evidence. |
| 23 | `markShapeDirty()` (975; disabled 1418) | Delegate to `markShapeDirty(true)`; disabled is inert. | `text/text.rs:57::mark_shape_dirty` | **adapted**: wrapper exists, but Rust helper also publishes revision/world dirt and invalidates bounds. |
| 24 | `markShapeDirty(bool)` (977; disabled 1417) | Add Path; clear every modifier range map; mark world transform; optionally layout dirty, in order. Disabled is inert. | `text/text.rs:68::mark_shape_dirty_with_layout`; no direct Rust owner for per-group `clearRangeMaps`. | **incomplete/order difference**: Rust publishes revision, invalidates bounds, Path, WorldTransform, then layout; the range-map clear side effect is reconstructed during later shaping. |
| 25 | `modifierShapeDirty` (993; disabled 1427) | Add Path only. | No direct Rust callback owner. | **missing direct callback**: generic draw-owner invalidation is not this Path-only Text method. |
| 26 | `markPaintDirty` (995; disabled 1426) | Add Paint only. | `text/text.rs:106::mark_paint_dirty` | **exact enabled-path candidate**; no disabled-feature counterpart. |
| 27 | `alignValueChanged` (997; disabled 1421) | Shape dirty. | `text/text.rs:147::uint_property_changed` | **adapted through row 24**. |
| 28 | `sizingValueChanged` (999; disabled 1422) | Shape dirty. | `text/text.rs:147::uint_property_changed` | **adapted through row 24**. |
| 29 | `overflowValueChanged` (1001; disabled 1423) | Shape dirty only when effective sizing is not autoWidth. | `text/text.rs:147::uint_property_changed`; predicate evidence `text.rs:5460::cxx_effective_sizing_matrix_is_shared_by_constraints_state_and_callbacks` | **exact enabled-path correction candidate** for the mixed-axis matrix; disabled-feature counterpart remains absent. |
| 30 | `widthChanged` (1009; disabled 1424) | Same conditional shape dirt as overflow. | `text/text.rs:115::double_property_changed`; predicate evidence `text.rs:5460::cxx_effective_sizing_matrix_is_shared_by_constraints_state_and_callbacks` | **exact enabled-path correction candidate** for the mixed-axis matrix; disabled-feature counterpart remains absent. |
| 31 | `paragraphSpacingChanged` (1017; disabled 1433) | Paint dirty. | `text/text.rs:115::double_property_changed`; consumption `text.rs:3134::paragraph_spacing`, `3158::static_line_metrics`, `3265::static_text_total_height` | **exact enabled-path correction candidate**: Paint publication is unchanged and the rebuilt layout now consumes spacing in pinned bounds/fit/measure/render order. The disabled-feature counterpart remains absent. |
| 32 | `heightChanged` (1019; disabled 1425) | Shape dirty only for effective fixed sizing. | `text/text.rs:115::double_property_changed`; predicate evidence `text.rs:5460::cxx_effective_sizing_matrix_is_shared_by_constraints_state_and_callbacks` | **exact enabled-path correction candidate** for the mixed-axis matrix; disabled-feature counterpart remains absent. |
| 33 | `StyledText::clear` (1027) | Clear Unicode values then runs. | No direct retained Rust owner; `text.rs:2935::StaticTextSlice::resolved_runs` constructs a fresh vector. | **missing direct retained owner/order**. |
| 34 | `StyledText::empty` (1033) | Emptiness is run-vector emptiness, not character-vector emptiness. | No direct `StyledText` owner; nearest predicates are `text.rs:2099::StaticTextSlice::render_layout` and `2125::StaticTextSlice::render_topology`, with insertion-point consumption at `3158::static_line_metrics`. | **adapted correction candidate**: layout predicates and empty-line metric ownership now exclude null-style, font-null, and empty source runs exactly as pinned `makeStyled` does, but no retained literal `StyledText` owner exists. |
| 35 | `StyledText::append` (1035) | Decode UTF-8 until NUL into Unicode values, count code points, append one `TextRun` with supplied style metadata. | `text.rs:2935::resolved_runs`; `3461::styled_resolved_run_glyphs` | **adapted (Rust UTF-8 safety)**: Rust strings exclude invalid UTF-8/NUL-tail semantics and do not retain a literal StyledText run vector. |
| 36 | `makeStyled` (1053) | Clear; preserve all-run indices while skipping missing style/font/empty text; append scaled runs; optionally apply modifiers; return run nonempty. Defaults: modifiers=true, scale=1. | `text.rs:2993::StaticTextSlice::resolved_runs`; `3050::resolved_dynamic_runs`; `2149::shaped_layout_from_resolved_runs`; `2284::layout_from_shaped_topology`; evidence `text.rs:5763::cxx_dynamic_text_runs_retain_all_indices_and_skip_only_null_style_or_empty_runs`, `5866::cxx_empty_line_metrics_ignore_runs_omitted_by_make_styled` | **adapted correction candidate**: every dynamic list row retains its all-runs vector position, while null-style and empty runs contribute no StyledText characters or offsets; empty-line insertion also excludes font-null runs before selecting its styled owner. Rust still reconstructs the StyledText lifecycle and does not retain a literal `TextRun.styleId = allRuns index`; modifier behavior remains governed by its separate owner. |
| 37 | static `BreakLines` (1085) | Break each paragraph; no-wrap/auto width use -1; auto-width computes global max; then compute spacing/alignment for all paragraphs. | `text.rs:4318::StaticTextSlice::layout_static_text_lines`; `text/line_breaker.rs:1::split_static_text_lines`; `52::static_text_line_iteration`; downstream metrics `text.rs:3158::static_line_metrics`; focused insertion evidence `text.rs:5866::cxx_empty_line_metrics_ignore_runs_omitted_by_make_styled` | **adapted**: custom HarfRust/renderer annotation breaker replaces `GlyphLine::BreakLines`; paragraph boundaries and styled-run insertion ownership now drive downstream metrics, but the shaped line-break representation remains nonliteral. |
| 38 | `modifierRangesNeedShape` (1123; disabled 1428) | Return true on first modifier needing shape, else false; disabled false. | No direct Text member; nearest predicate is `text/text_modifier_group.rs:348::StaticTextModifierGroup::has_shape_modifiers`. | **adapted/packed**: no separately retained pre-shape range state. |
| 39 | `onDirty` (1135; disabled 1420) | WorldTransform notifies all modifiers; Path/Paint invalidates stroke effects on current render styles. | `draw.rs:10197::RuntimeShapeList::on_component_dirty`; nearest Text invalidators are `draw.rs:10477::RuntimeTextDrawOwner::mark_shape_dirty_unless_paths_retained` and `draw.rs:10504::RuntimeTextDrawOwner::mark_render_styles_dirty_unless_paths_retained`; no direct per-modifier world-dirty callback or per-current-render-style stroke-effect invalidation owner. | **incomplete**: no source-ordered per-modifier notification or per-current-render-style stroke-effect invalidation owner. |
| 40 | `update` (1154; disabled 1419) | Super first. Path: optional premodifier shape, styled shape/lines/coverage, clear retained caches, build styles. Paint rebuilds styles. Opacity only propagates. Finally rebuild clip path/shape world for world/path/paint. | Distributed across `artboard.rs:8921::ArtboardInstance::update_components_with_hook_recording` and `draw.rs:18038::ArtboardInstance::update_runtime_text_render_styles`, with concrete Text-owner phases at `draw.rs:10477::RuntimeTextDrawOwner::mark_shape_dirty_unless_paths_retained`, `draw.rs:10504::RuntimeTextDrawOwner::mark_render_styles_dirty_unless_paths_retained`, `draw.rs:10511::RuntimeTextDrawOwner::propagate_render_opacity`, `draw.rs:10544::RuntimeTextDrawOwner::propagate_world_transform`, and `draw.rs:10647::RuntimeTextDrawOwner::rebuild`; there is no single Rust owner for the complete C++ method. | **adapted/incomplete**: broad phase split is retained and the rebuilt static layout now consumes paragraph spacing, but superclass/order is distributed, row 39 gaps remain, and layout dirty is published outside the Text owner after rebuild. |
| 41 | `measure` (1259) | Make styled; choose width from authored sizing; choose wrap from max constraint; shape/break; baseline/ellipsis/paragraph spacing/trim; authored-sizing bounds; clamp to max; empty returns zero. | `text/text_engine.rs:54::static_text_layout_measure_bounds`; `text.rs:2790::measure_bounds_with_layout_constraint`; spacing evidence `text.rs:5539::cxx_paragraph_spacing_order_covers_empty_paragraphs_trim_measure_fit_and_render` | **adapted correction candidate**: measurement now includes inter-paragraph spacing and removes final trailing spacing in auto bounds. Taffy projection and fallback behavior remain adaptations. |
| 42 | `localBounds` (1366; disabled 1434) | Shift retained `m_bounds` by normalized origin and return; disabled empty AABB. | `text/text_engine.rs:1::static_text_constraint_bounds`; `components.rs:996::RuntimeTextState::bounds` | **exact enabled retained-access candidate** after update; no disabled-feature counterpart. |
| 43 | `hitTest` (1376; disabled 1414) | If render opacity is zero return null; otherwise still return null. Disabled also null. | Generic geometry hit routing; no Text-specific public null override | **missing direct owner/evidence**: prove Text itself can never be returned by the general hit path. |
| 44 | `originValueChanged` (1386; disabled 1435) | Paint dirty, then world-transform dirty. | `text/text.rs:110::mark_origin_dirty` | **exact order candidate**. |
| 45 | `originXChanged` (1392; disabled 1436) | Paint dirty, then world-transform dirty. | `text/text.rs:115::double_property_changed` -> `mark_origin_dirty` | **exact order candidate**. |
| 46 | `originYChanged` (1397; disabled 1437) | Paint dirty, then world-transform dirty. | `text/text.rs:115::double_property_changed` -> `110::mark_origin_dirty` | **exact order candidate**. |
| 47 | `verticalTrimValueChanged` (1403; disabled 1438) | Shape/layout dirty through `markShapeDirty`. | `text/text.rs:147::uint_property_changed` | **adapted through row 24**. |
| 48 | `composeWorldTransform` (1453) | If participating and parent transform exists, compose parent * origin-based resolved layout translation * local transform; otherwise superclass. | `draw.rs:6692::runtime_component_world_transform_with_bounds`, especially participant branches 6739-6783 | **adapted candidate**: Taffy bounds substitute for LayoutParticipant object state; exact nested/order differential pending. |
| 49 | `layoutParticipant` (1475) | Return first direct child of LayoutParticipant type, otherwise null. | `draw.rs:8479::runtime_layout_participant_local` | **adapted candidate**: graph lookup must be checked for first-child/type ordering. |
| 50 | `isParticipatingInLayout` (1487) | `layoutParticipant()!=nullptr`. | No direct bool method; callers query `draw.rs:8479::ArtboardInstance::runtime_layout_participant_local`. | **adapted candidate**. |
| 51 | `align` (1492) | Inherit or authored center preserves authored; otherwise LTR->left, RTL->right. | `text.rs:1364::RuntimeTextLayoutConstraint::effective_align` | **exact algorithm candidate**, pending enum-value and mixed-layout evidence. |
| 52 | `updateList` (1504) | Build styles; reset all-runs to authored; remap/reuse listener by list index or allocate; append each valid-instance run; delete excess; shape dirty. | `data_bind_context.rs:3497::remap_source`; `viewmodel_instance_list.rs:142::text_runs`; `text.rs:2993::resolved_runs`; evidence `viewmodel/viewmodel_instance_list.rs:616::cxx_text_run_projection_preserves_missing_properties_and_item_order` | **adapted correction candidate**: every valid list instance is projected in source order, including an empty run when `textContent` is absent, and the existing listener vector still remaps/reuses by list index. Rust polls aggregate cells during rendering and does not retain concrete dynamic TextValueRun/property-listener objects with source-equivalent initial-write or dirty timing. |
| 53 | `buildTextStylePaints` (1550) | Lazily cache direct TextStylePaint children once, in child order. | `text.rs:1710::StaticTextSlice::from_graph`; `draw.rs:10427::RuntimeTextDrawOwner::topology_or_build`; empty-topology evidence `text.rs:5763::cxx_dynamic_text_runs_retain_all_indices_and_skip_only_null_style_or_empty_runs` | **adapted correction candidate**: direct TextStylePaint child order and a legitimate empty style list are now accepted; a present style with no paints remains null and is skipped. Construction remains bundled with runs/modifiers and caching belongs to the retained draw occurrence rather than the Text object. |

Rows 8-10, 18-19, 21-32, 38-40, and 42-47 include the 26 explicit disabled
definition bodies at pinned lines 1412-1451. The Rust crate has no equivalent
`WITH_RIVE_TEXT` configuration, so those alternate bodies are source authority
without a current conditional owner.

## Executable header authority

| # | Pinned inline member | Concrete Rust owner | Candidate disposition |
|---:|---|---|---|
| H1 | `StyledText::unichars` (65) | No direct accessor; nearest retained field is `text/fully_shaped_text.rs:22::StaticShapedTextTopology::text`. | **adapted/missing direct accessor** |
| H2 | `StyledText::runs` (66) | No direct accessor; nearest retained fields are `text/fully_shaped_text.rs:23-24::StaticShapedTextTopology::{resolved_runs,contextual_glyphs}`. | **adapted/missing direct accessor** |
| H3 | `StyledText::swapRuns` (68) | No Rust owner. | **missing** |
| H4 | `TextValueRunListener::text` (114) | `data_bind/data_bind_context.rs:3485::RuntimeArtboardTextListBindingInstance::target_local_id` | **adapted** |
| H5 | `TextValueRunListener::textValueRun` (115) | No concrete dynamic run object; nearest projection is `viewmodel/viewmodel_instance_list.rs:132::text_runs`. | **missing** |
| H6 | `ColorGlyphCacheKey::operator==` (139) | `draw.rs:10683::RuntimeTextBackendResources::emoji_images` tuple key `(font_identity,glyph_id)` | **adapted**: C++ uses `Font*` identity; Rust uses the font-byte allocation pointer recorded by `text.rs:1038::RuntimeIntegratedColorGlyphCommand::font_identity`. |
| H7 | `ColorGlyphCacheHash::operator()` (147) | `draw.rs:10683::RuntimeTextBackendResources::emoji_images` uses `BTreeMap` tuple ordering, not a hash. | **adapted collection strategy** |
| H8 | `Text::shapeWorldTransform` (196) | No direct accessor; retained fields at `draw.rs:15837::RuntimeCachedTextShapePaints::shape_local_transform` and `15845::RuntimeRetainedColorGlyphs::shape_world`. | **adapted retained owner** |
| H9 | `Text::sizing` (197) | `text.rs:3811::StaticTextSlice::authored_sizing` | **exact value candidate** |
| H10 | `Text::overflow` getter (199) | `text.rs:4072::StaticTextSlice::text_uint_property` with `overflowValue` | **exact value candidate** |
| H11 | `Text::textOrigin` (200) | `text.rs:4072::StaticTextSlice::text_uint_property` with `originValue` | **exact value candidate** |
| H12 | `Text::verticalTrimTop` (201) | `text.rs:3173::StaticTextSlice::static_vertical_trim`, low byte | **exact mask candidate** |
| H13 | `Text::verticalTrimBottom` (205) | `text.rs:3173::StaticTextSlice::static_vertical_trim`, high byte | **exact mask candidate** |
| H14 | `Text::wrap` (209) | `text.rs:4072::StaticTextSlice::text_uint_property` with `wrapValue` | **exact value candidate** |
| H15 | `Text::verticalAlign` (210) | `text.rs:4072::StaticTextSlice::text_uint_property` with `verticalAlignValue` | **exact value candidate** |
| H16 | `Text::overflow(TextOverflow)` (215) | `artboard.rs:5758::ArtboardInstance::set_uint_property` generated-property path | **adapted signature** |
| H17 | `Text::constraintBounds` (220) | `text/text_engine.rs:1::static_text_constraint_bounds` | **exact delegation candidate** |
| H18 | `Text::effectiveWidth` (230) | `text.rs:3994::StaticTextSlice::effective_width` | **adapted: constraint option replaces retained NaN sentinel** |
| H19 | `Text::effectiveHeight` (234) | `text.rs:4011::StaticTextSlice::effective_height` | **adapted: constraint option replaces retained NaN sentinel** |
| H20 | `Text::overflowAsFixed` (239) | `text.rs:3849::StaticTextSlice::overflow_as_fixed` | **adapted**: consumes the corrected row 11 matrix, but retained property projection is nonliteral. |
| H21 | `Text::computedWidth` (244) | No direct method; nearest retained accessor is `components.rs:996::RuntimeTextState::bounds`. | **adapted value projection** |
| H22 | `Text::computedHeight` (245) | No direct method; nearest retained accessor is `components.rs:996::RuntimeTextState::bounds`. | **adapted value projection** |
| H23 | `Text::runs` (248) | No public complete vector; `text.rs:908::StaticTextSlice::runs` plus `2935::resolved_runs`. | **adapted** |
| H24 | `Text::textStylePaints` (254) | No public pointer vector; `text.rs:909::StaticTextSlice::styles`. | **adapted** |
| H25 | `Text::haveModifiers` (261) | No direct method; `text.rs:910::StaticTextSlice::modifiers` and `text/text_modifier_group.rs:348::has_shape_modifiers`. | **adapted; disabled false has no cfg owner** |
| H26 | testing `orderedLines` (270) | No literal observer; nearest projection is `text.rs:163::static_text_layout_debug_report` from `text/fully_shaped_text.rs:9::StaticShapedTextLayout::lines`. | **missing literal public observer** |
| H27 | testing `modifierGroups` (274) | No literal observer; nearest projection is `text.rs:163::static_text_layout_debug_report` from `text.rs:910::StaticTextSlice::modifiers`. | **missing literal public observer** |
| H28 | testing `shape` (278) | No literal Paragraph/GlyphRun observer; nearest projection is `text.rs:163::static_text_layout_debug_report` from `text/fully_shaped_text.rs:21::StaticShapedTextTopology`. | **missing literal observer** |
| H29 | testing `unichars` (279) | No literal observer; nearest projection is `text.rs:163::static_text_layout_debug_report` from `text/fully_shaped_text.rs:22::StaticShapedTextTopology::text`. | **missing literal public observer** |

The eight source-significant header defaults are:

| Default | Pinned value | Rust owner/disposition |
|---|---|---|
| `TextDrawCommand::style` | null | No nullable style field; `text.rs:1031::RuntimeTextDrawOrder` uses typed variants, **adapted safely**. |
| `TextValueRunProperty::m_symbolType` | `none` | No per-property Rust object, **missing direct state**. |
| `TextValueRunListener::m_text` | null | `data_bind/data_bind_context.rs:3477::RuntimeArtboardTextListBindingInstance::target_local_id`, **adapted**. |
| `m_layoutWidth` | NaN | No retained field; `text.rs:1337::RuntimeTextLayoutConstraint` is an optional reconstructed input, **adapted**. |
| `m_layoutHeight` | NaN | No retained field; `text.rs:1337::RuntimeTextLayoutConstraint` is an optional reconstructed input, **adapted**. |
| `m_layoutWidthScaleType` | `uint8_t::max` | `components.rs:981::RuntimeTextState::layout_scale_types=None`, **adapted**. |
| `m_layoutHeightScaleType` | `uint8_t::max` | `components.rs:981::RuntimeTextState::layout_scale_types=None`, **adapted**. |
| `m_layoutDirection` | `inherit` | `text.rs:1342::RuntimeTextLayoutConstraint::layout_direction`, defaulted by callers in `draw.rs:7500::runtime_text_layout_constraint`, **adapted/pending**. |

The header-only declaration `Text::updateOriginWorldTransform()` has no pinned
definition or call site. It is a stale declaration, not executable source
authority; Rust has no corresponding symbol.

## Order dependencies that must survive correction

1. A list property listener performs its initial write before being retained;
   list updates reuse/remap by index, append valid instances in list order,
   delete excess listeners, then dirty shape.
2. Shape dirt adds Path, clears modifier range maps, marks world transform, and
   only then optionally marks layout.
3. `update` runs its superclass before Text work; Path shaping/coverage precedes
   render-style construction; world/clip reconstruction follows the mutually
   exclusive Path/Paint/RenderOpacity branch.
4. `buildRenderStyles` clears retained paths/commands before the empty-shape
   return, and computes bounds before modifiers, clip, glyph paths, fit transform,
   layout publication, and hit contours.
5. Paragraph spacing contributes after every paragraph during bounds, fitting,
   and measuring, with the final spacing removed only from auto-sized bounds.
6. Color glyph drawing caches decoded images by font identity plus glyph ID and
   brackets both the whole glyph and bitmap sub-transform with save/restore.

## `text_test.cpp` consumer topology and blockers

Pinned consumer: `tests/unit_tests/runtime/text_test.cpp`, 820 lines, 27,373
bytes, SHA-256
`d3917b4de319fbb3d2eb7d4eae1deee4f53d509b460ee2377595696b8bfd5367`.
The exact 18-case topology is **four pass, three executable expected-red, and
11 pending**. Cases 2 and 3 are direct/pass in Wave C7. Cases 14 and 16 have
exact literal Silver entries and pass. Cases 15, 17, and 18 retain executable
literal Silver entries with frozen first differences; they are not
missing-owner pending work.

| Case | Current consumer status and source blocker |
|---:|---|
| 1 `file with text loads correctly` | **pending**: literal typed `find<Text/TextStyle/TextValueRun>` count stream and no-op draw lifecycle are not exposed together; rows 19 and 40 remain adapted. |
| 2 `can query for all text runs` | **pass/direct**: Wave C7 literal Rust test preserves the fixture and sole count assertion. |
| 3 `can query for a text run at a given index` | **pass/direct**: Wave C7 literal Rust test preserves the fixture, index, and text assertion. |
| 4 `simple text loads` | **pending**: missing literal `Text::shape()` Paragraph/GlyphRun observer (H28), plus empty-shape retained-bounds lifecycle across mutation (rows 17, 34-36, 40, 42). |
| 5 `vertical trim shrinks...` | **pending**: H29/H28 and direct retained local-bounds observer are incomplete; row 13 uses reconstructed glyph bounds rather than pinned cap/x metrics. |
| 6 trim uint passthroughs | **pending**: generated top/bottom masked registry accessors need a literal executable owner and read/write stream; current shaping mask consumption is downstream only. |
| 7 `ellipsis is shown` | **pending**: missing literal `orderedLines`, `shape`, `unichars`, and `GlyphLookup` observables (H26/H28/H29); reconstructed line/glyph projections cannot replace them. |
| 8 `fitFontSize shrinks...` | **pending**: missing literal shaped run size and `m_transform.xx` observers; the exact Silver stream for case 16 is not a proxy for these retained observables. |
| 9 `range mapper maps words` | **pending**: owned by `text_modifier_range.cpp`, not this pair; direct `RangeMapper::unitCount` owner/evidence remains required. |
| 10 modifier ranges select runs | **pending**: missing literal modifier group/range/run/coverage observer stream (H27) and source-pair certification for modifier owners. |
| 11 varying-size modifier runs | **pending**: same H27 blocker plus literal Unicode offset/length/text byte assertions across multiple runs. |
| 12 `double new line type works` | **pending**: missing `orderedLines()` observer (H26); custom line splitting is not a substitute for the exact retained ordered-line count. |
| 13 opacity modifiers | **pending**: requires literal fixture advance/draw stream after row 17/19/39/40 review; a no-panic proxy is insufficient. |
| 14 zero-width spaces | **pass/direct Silver**: exact fixture/action/assertion stream is executable through the existing `zero_width_space_line_break` manifest entry (`tools/silver-corpus/generate_manifest.py:189,969`). |
| 15 word joiners | **executable expected-red**: the complete **ten-rendered-frame** stream has frozen first difference `frame 2, op 262 (transform), field ty: expected -39.996094, got -15.796875` (`tools/silver-corpus/generate_manifest.py:1987`). |
| 16 `Fit font size` Silver | **pass/direct Silver**: the complete seven-rendered-frame fixture/action/assertion stream is exact after the source-driven paragraph-spacing correction (`silver-corpus.toml:1401`; focused wrapper `tools/silver-corpus/tests/silver_backfill_cases.rs:37`). |
| 17 `Vertical Trim` Silver | **executable expected-red**: frozen first difference `frame 3, op 220 (rewind): expected rewind, got drawPath` (`tools/silver-corpus/generate_manifest.py:1981`); row 13 remains adapted. |
| 18 layout-controlled box Silver | **executable expected-red**: frozen first difference `frame 0, op 61 (save): expected save, got frame` (`tools/silver-corpus/generate_manifest.py:1918`; focused wrapper `tools/silver-corpus/tests/silver_backfill_cases.rs:100`); rows 10-11 remain blockers. |

## Packed ownership and split decision

No source split is implemented in this candidate. A mechanical split is not
yet safe: `text.rs` interleaves this translation unit with TextInput, line
breaker, modifier, font, text-style, paint, and geometry-query ownership, while
`draw.rs` is currently user-dirty and owns shared renderer/backend lifetimes.
Moving those symbols before correcting the red rows would obscure whether a
behavioral change came from translation or refactoring.

The smallest later split should be source-pair driven:

1. move the `StaticTextSlice` Text-only topology/shaping/bounds functions into
   one `text/text_runtime.rs` owner while keeping shared line/font helpers in
   their accepted source-pair modules;
2. keep `RuntimeTextDrawOwner` and backend resource lifetimes in `draw.rs`, but
   expose one narrow Text update/draw interface rather than attributing the file;
3. keep Taffy adaptation at the layout boundary; and
4. keep list binding/listener ownership in data-bind modules, with an explicit
   source-order contract back to Text.

## Candidate verdict

This pair is **not at exact source parity**. The complete source read found at
least these actionable discrepancies:

- the per-property listener identity, initial-write/retention, and synchronous
  dirt order is reconstructed/polled rather than directly owned;
- dynamic all-run positions are retained by vector order, but Rust does not
  retain the literal `StyledText::TextRun::styleId` space;
- `controlSize` does not retain and compare the complete pinned five-field
  state in one owner;
- retained path clearing and update-phase layout-dirt order around the corrected
  empty-shape branch remain distributed rather than directly owned;
- non-image color-glyph layers retain vector paths across draws instead of
  creating fresh factory path/paint objects on every draw;
- TextValueRun hit rectangles/contours and the direct null `Text::hitTest`
  surface have no literal owner; and
- the 26 disabled-text alternate bodies have no Rust configuration counterpart.

The 18 `text_test.cpp` cases are consumers of these owners, not proof that the
source rows are complete. The three executable reds are honest divergences,
not coverage gaps. A fresh independent reviewer must try to falsify this
production-correction candidate before the corrected rows are accepted.
