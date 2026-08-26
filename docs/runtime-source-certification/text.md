# Text source-pair correspondence candidate

Status: **author candidate; pending independent semantic review**

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

This is an atomic source-pair audit under
`docs/runtime-exact-parity-workflow-correction.md`. It does not inherit the
older file-level `mapped` or `faithful` verdict. No production code or test was
changed during this audit.

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
| 1 | `TextValueRunProperty::TextValueRunProperty` (23) | Retain core, listener, instance value, property key, and symbol type. | `text/text.rs:15::RuntimeTextValueRunListener::new`; `RuntimeCoreObjectListener` | **adapted/packed**: no per-property dependent retaining the key/type tuple. |
| 2 | `TextValueRunProperty::writeValue` (36) | Text content writes through `CoreRegistry`; style scans paints in order, installs the first as fallback, then the named match; unknown symbol is inert. | `viewmodel/viewmodel_instance_list.rs:132::text_runs`; `text.rs:2935::resolved_runs` | **incorrect/incomplete**: named-style/first-style fallback exists, but items without `textContent` are dropped by `filter_map`; C++ still creates an empty run. Writes are polled during reconstruction, not performed by the listener in source order. |
| 3 | `TextValueRunListener` constructor (75) | Initialize base and `m_text`, then immediately create properties. | `text/text.rs:15::new` | **adapted candidate**: immediate creation exists under Rust-owned handles; object identity differs. |
| 4 | `TextValueRunListener::markDirty` (83) | Directly call owning Text `markShapeDirty`. | `data_bind/data_bind_context.rs:7310::flush_runtime_text_run_listener_changes`; `text/text.rs:57::mark_shape_dirty` | **adapted/order pending**: dirty cells are polled/flushed later rather than synchronously forwarding the callback. |
| 5 | `createProperties` (85) | Base cleanup first, then style listener, then content listener. | `text/text.rs:39::create_properties` | **incomplete**: cleanup is retained, but Rust gathers `textStyle`, then `textContent`, and installs them in one aggregate call; missing cells are silently omitted. Listener/write ordering needs direct proof. |
| 6 | `createSinglePropertyListener` (92) | Map only style/content to generated property keys; require a value property; otherwise null. | `text/text.rs:39::create_properties` | **adapted**: name-based typed-cell lookup replaces explicit key/symbol dependent construction. |
| 7 | `createPropertyListener` (120) | Create only supported listener; if present, write value before pushing it. | `text/text.rs:39::create_properties`; `data_bind_context.rs:3496::remap_source` | **incomplete**: there is no literal initial `writeValue`-then-retain operation; rendering polls shared cells. |
| 8 | `Text::~Text` (139; disabled 1412) | Delete every dynamic run listener; disabled build is empty. | Rust `Vec`/`Rc` drop for `RuntimeArtboardTextListBindingInstance::listeners` | **exact Rust lifetime adaptation, pending teardown evidence**. |
| 9 | `measureLayout` (147; disabled 1439) | Map undefined axes to float max, call `measure`, return result; disabled returns zero. | `text/text_engine.rs:54::static_text_layout_measure_bounds`; `draw.rs:13847::measure_layout_component`, `14073::measure_layout_participant` | **adapted (Taffy), incomplete**: max constraints are projected through Taffy, but there is no disabled-feature branch and row 41 discrepancies flow here. |
| 10 | `controlSize` (160; disabled 1446) | On any width/height/scale/direction change, retain all five values then `markShapeDirty(false)`; disabled is inert. | `draw.rs:7500::runtime_text_layout_constraint`; `components.rs:1008::retain_layout_scale_types`; retained layout bounds | **missing direct owner/order**: Rust does not retain the complete five-field state on Text or perform one changed-check followed by no-layout shape dirt. |
| 11 | `effectiveSizing` (179) | For active layout: no boxed axes -> authored; boxed width plus hug height -> autoHeight; all other boxed combinations -> fixed. Legacy max/hug sentinel path otherwise returns authored. | `text.rs:1352::RuntimeTextLayoutConstraint::effective_sizing`; `components.rs:1012::RuntimeTextState::effective_sizing`; `text.rs:3777::StaticTextSlice::effective_sizing` | **incorrect**: fixed/fill width plus hug height returns authored in Rust, while pinned C++ returns `autoHeight`. |
| 12 | `clearRenderStyles` (211) | Rewind every style path; clear render styles and draw commands; reset hit state on every run, in order. | `draw.rs:10605::RuntimeTextDrawOwner::retained_or_build`; `text.rs:2521::render_data_from_layout` | **adapted/incomplete**: retained backend replacement models path clearing, but no TextValueRun hit-state reset is owned here and exact rewind/clear order is distributed. |
| 13 | static `computeVerticalTrim` (230) | Zero outputs; early return; first nonempty line cap/x-height max; last nonempty line alphabetic/text descent calculation. | `text.rs:3173::StaticTextSlice::static_vertical_trim` | **adapted candidate**: structure is close, but Rust derives cap/x height from `H`/`x` glyph bounds with ascent fallback rather than consuming the pinned font metrics fields. Differential evidence required. |
| 14 | `computeBoundsInfo` (304) | Accumulate paragraph lines, widths, baseline origin, paragraph spacing, ellipsis line/height, and non-fixed trim in source order. | `text.rs:4322::static_layout_info`; `3070::static_line_metrics`; `3173::static_vertical_trim` | **incorrect**: the static Text pipeline never reads `Text.paragraphSpacing`; bounds/ellipsis height omit it. |
| 15 | `fitFontScale` (389) | Find max nonempty styled run size; early return; binary-search integer top size; each probe makes styled text, shapes, breaks, includes paragraph spacing, and checks width/height. | `text.rs:3330::fit_font_scale`; `3361::max_authored_font_size`; `3381::fit_font_scale_for_max_size` | **incorrect**: the binary search is present, but fit height omits paragraph spacing; shaping/line-break representation is adapted. |
| 16 | `shouldDrawLine` (481) | Hidden/clipped and top/middle/bottom use distinct top/bottom comparisons yielding draw/skip/stop. | `text/line_breaker.rs:50::static_text_line_iteration` | **exact algorithm candidate, pending all nine branch comparisons**. |
| 17 | `buildRenderStyles` (558) | Eight ordered phases: clear; empty-shape bounds/return; bounds+ellipsis; modifier coverage; bounds/clip; ordered glyph paths/color commands/hit rects; fit/vertical transform plus layout dirt; hit contours. | `text.rs:2238::layout_from_shaped_topology`; `2521::render_data_from_layout`; `4322::static_layout_info`; `4432::static_render_transform`; `draw.rs:17772::runtime_build_text_draw_frame` | **incomplete/packed**: render order and fit are represented, but paragraph spacing is absent, TextValueRun hit rectangles/contours have no retained source-equivalent owner, and phase/order is split across immutable reconstruction and retained draw-owner publication. |
| 18 | `styleFromShaperId` (863; disabled 1429) | Assert ID in `m_runs`, return its style; disabled returns null. | `text.rs:4528::style_index_for_local` and indexed `styles` reads | **adapted/incomplete**: local-ID lookup replaces shaper index assertion; no disabled-feature counterpart. |
| 19 | `draw` (869; disabled 1413) | Conditional outer save; optional clipped path; replay style/color commands in stored order; conditional restore. Disabled draw is inert. | `draw.rs:18306::runtime_draw_live_text_family`; `17948::runtime_text_replay_order` | **adapted candidate**: retained replay and save/clip/restore exist; exact renderer stream remains required. |
| 20 | `drawColorGlyph` (901) | Get layers or return; save+transform; cache/decode bitmap and apply extent transform, or build nonzero path/paint; restore. | `draw.rs:18222::runtime_draw_integrated_color_glyph`; `RuntimeTextBackendResources::emoji_images` | **adapted candidate**: raster cache/order is close; Rust also supports gradient color layers and caches vector paths. Exact COLR/SBIX/CBDT streams are pending. |
| 21 | `addRun` (964; disabled 1415) | Append the same pointer to authored runs and all runs; disabled is inert. | `text.rs:1670::StaticTextSlice::from_graph` run collection | **adapted**: immutable graph reconstruction retains one descriptor list, not two pointer vectors. |
| 22 | `addModifierGroup` (970; disabled 1416) | Append group in child/import order; disabled is inert. | `text.rs:1979::StaticTextSlice::from_graph` modifier collection | **adapted candidate**, pending order evidence. |
| 23 | `markShapeDirty()` (975; disabled 1418) | Delegate to `markShapeDirty(true)`; disabled is inert. | `text/text.rs:57::mark_shape_dirty` | **adapted**: wrapper exists, but Rust helper also publishes revision/world dirt and invalidates bounds. |
| 24 | `markShapeDirty(bool)` (977; disabled 1417) | Add Path; clear every modifier range map; mark world transform; optionally layout dirty, in order. Disabled is inert. | `text/text.rs:68::mark_shape_dirty_with_layout`; modifier caches in `text.rs` reconstruction | **incomplete/order difference**: Rust publishes revision, invalidates bounds, Path, WorldTransform, then layout; there is no direct per-group `clearRangeMaps` owner. |
| 25 | `modifierShapeDirty` (993; disabled 1427) | Add Path only. | modifier/property invalidation into `RuntimeTextDrawOwner` | **missing direct callback**: generic owner paths can dirty Text, but no concrete Path-only Text method is mapped. |
| 26 | `markPaintDirty` (995; disabled 1426) | Add Paint only. | `text/text.rs:106::mark_paint_dirty` | **exact enabled-path candidate**; no disabled-feature counterpart. |
| 27 | `alignValueChanged` (997; disabled 1421) | Shape dirty. | `text/text.rs:147::uint_property_changed` | **adapted through row 24**. |
| 28 | `sizingValueChanged` (999; disabled 1422) | Shape dirty. | `text/text.rs:147::uint_property_changed` | **adapted through row 24**. |
| 29 | `overflowValueChanged` (1001; disabled 1423) | Shape dirty only when effective sizing is not autoWidth. | `text/text.rs:147::uint_property_changed` | **incorrect for mixed fixed-width/hug-height** because its predicate consumes row 11's wrong effective sizing. |
| 30 | `widthChanged` (1009; disabled 1424) | Same conditional shape dirt as overflow. | `text/text.rs:115::double_property_changed` | **incorrect for mixed fixed-width/hug-height** through row 11. |
| 31 | `paragraphSpacingChanged` (1017; disabled 1433) | Paint dirty. | `text/text.rs:115::double_property_changed` | **callback exact but behavior incomplete**: Paint is published, yet rebuild never consumes paragraph spacing (rows 14, 15, 17, 41). |
| 32 | `heightChanged` (1019; disabled 1425) | Shape dirty only for effective fixed sizing. | `text/text.rs:115::double_property_changed` | **incorrect for mixed fixed-width/hug-height** through row 11. |
| 33 | `StyledText::clear` (1027) | Clear Unicode values then runs. | Ephemeral `Vec` reconstruction in `resolved_runs`/shaping | **missing direct retained owner/order**. |
| 34 | `StyledText::empty` (1033) | Emptiness is run-vector emptiness, not character-vector emptiness. | `render_layout` checks `resolved_runs.iter().all(text.empty)` | **adapted candidate**: because pinned `makeStyled` skips empty source runs before append, the Rust all-empty predicate appears equivalent for the current run stream, but there is no retained `StyledText` owner. |
| 35 | `StyledText::append` (1035) | Decode UTF-8 until NUL into Unicode values, count code points, append one `TextRun` with supplied style metadata. | `text.rs:2935::resolved_runs`; `3461::styled_resolved_run_glyphs` | **adapted (Rust UTF-8 safety)**: Rust strings exclude invalid UTF-8/NUL-tail semantics and do not retain a literal StyledText run vector. |
| 36 | `makeStyled` (1053) | Clear; preserve all-run indices while skipping missing style/font/empty text; append scaled runs; optionally apply modifiers; return run nonempty. Defaults: modifiers=true, scale=1. | `text.rs:2935::resolved_runs`; `2140::shaped_topology_from_resolved_runs`; modifier coverage/transform in `2238::layout_from_shaped_topology` | **incomplete/packed**: run/style resolution and modifiers exist, but empty/missing runs do not preserve the same style ID space and the StyledText lifecycle is reconstructed. |
| 37 | static `BreakLines` (1085) | Break each paragraph; no-wrap/auto width use -1; auto-width computes global max; then compute spacing/alignment for all paragraphs. | `text.rs:4180::layout_static_text_lines`; `text/line_breaker.rs` | **adapted/incomplete**: custom HarfRust/renderer annotation breaker replaces `GlyphLine::BreakLines`; paragraph spacing is outside this function but downstream line vertical positions still omit it. |
| 38 | `modifierRangesNeedShape` (1123; disabled 1428) | Return true on first modifier needing shape, else false; disabled false. | `text.rs` modifier preflight (`StaticTextModifierGroup::has_shape_modifiers`) | **adapted/packed**: no concrete Text member or separately retained pre-shape range state. |
| 39 | `onDirty` (1135; disabled 1420) | WorldTransform notifies all modifiers; Path/Paint invalidates stroke effects on current render styles. | `draw.rs:10197::RuntimeShapeList::on_component_dirty`; `RuntimeTextDrawOwner` dirt methods; modifier reconstruction | **incomplete**: no direct per-modifier `onTextWorldTransformDirty` pass and no source-ordered per-render-style stroke-effect invalidation owner. |
| 40 | `update` (1154; disabled 1419) | Super first. Path: optional premodifier shape, styled shape/lines/coverage, clear retained caches, build styles. Paint rebuilds styles. Opacity only propagates. Finally rebuild clip path/shape world for world/path/paint. | `artboard.rs:9020-9078`; `draw.rs:18038::update_runtime_text_render_styles`; `RuntimeTextDrawOwner` | **adapted/incomplete**: broad phase split is retained, but superclass/order is distributed, row 39 gaps remain, paragraph spacing is absent, and layout dirty is published outside the Text owner after rebuild. |
| 41 | `measure` (1259) | Make styled; choose width from authored sizing; choose wrap from max constraint; shape/break; baseline/ellipsis/paragraph spacing/trim; authored-sizing bounds; clamp to max; empty returns zero. | `text/text_engine.rs:54::static_text_layout_measure_bounds`; `text.rs:2764::measure_bounds_with_layout_constraint` | **incorrect**: paragraph spacing is omitted. The Taffy projection and fallback behavior are additional adaptations. |
| 42 | `localBounds` (1366; disabled 1434) | Shift retained `m_bounds` by normalized origin and return; disabled empty AABB. | `text/text_engine.rs:1::static_text_constraint_bounds`; `components.rs:996::RuntimeTextState::bounds` | **exact enabled retained-access candidate** after update; no disabled-feature counterpart. |
| 43 | `hitTest` (1376; disabled 1414) | If render opacity is zero return null; otherwise still return null. Disabled also null. | Generic geometry hit routing; no Text-specific public null override | **missing direct owner/evidence**: prove Text itself can never be returned by the general hit path. |
| 44 | `originValueChanged` (1386; disabled 1435) | Paint dirty, then world-transform dirty. | `text/text.rs:110::mark_origin_dirty` | **exact order candidate**. |
| 45 | `originXChanged` (1392; disabled 1436) | Paint dirty, then world-transform dirty. | `text/text.rs:115::double_property_changed` -> `mark_origin_dirty` | **exact order candidate**. |
| 46 | `originYChanged` (1397; disabled 1437) | Paint dirty, then world-transform dirty. | same as row 45 | **exact order candidate**. |
| 47 | `verticalTrimValueChanged` (1403; disabled 1438) | Shape/layout dirty through `markShapeDirty`. | `text/text.rs:147::uint_property_changed` | **adapted through row 24**. |
| 48 | `composeWorldTransform` (1453) | If participating and parent transform exists, compose parent * origin-based resolved layout translation * local transform; otherwise superclass. | `draw.rs:6692::runtime_component_world_transform_with_bounds`, especially participant branches 6739-6783 | **adapted candidate**: Taffy bounds substitute for LayoutParticipant object state; exact nested/order differential pending. |
| 49 | `layoutParticipant` (1475) | Return first direct child of LayoutParticipant type, otherwise null. | `draw.rs:8479::runtime_layout_participant_local` | **adapted candidate**: graph lookup must be checked for first-child/type ordering. |
| 50 | `isParticipatingInLayout` (1487) | `layoutParticipant()!=nullptr`. | callers of `runtime_layout_participant_local` | **adapted candidate**. |
| 51 | `align` (1492) | Inherit or authored center preserves authored; otherwise LTR->left, RTL->right. | `text.rs:1364::RuntimeTextLayoutConstraint::effective_align` | **exact algorithm candidate**, pending enum-value and mixed-layout evidence. |
| 52 | `updateList` (1504) | Build styles; reset all-runs to authored; remap/reuse listener by list index or allocate; append each valid-instance run; delete excess; shape dirty. | `data_bind_context.rs:3496::remap_source`; `viewmodel_instance_list.rs:132::text_runs`; `text.rs:2935::resolved_runs` | **incorrect/incomplete**: Rust drops list items missing `textContent`, polls rows during rendering, and does not retain/reuse concrete dynamic TextValueRun objects with source-equivalent timing. |
| 53 | `buildTextStylePaints` (1550) | Lazily cache direct TextStylePaint children once, in child order. | `text.rs:1924-1978::StaticTextSlice::from_graph`; `draw.rs:10427::topology_or_build` | **adapted candidate**: topology is lazily cached per occurrence, but construction is bundled with runs/modifiers and errors on no styles. |

Rows 8-10, 18-19, 21-32, 38-47 include the 26 explicit disabled
definition bodies at pinned lines 1412-1451. The Rust crate has no equivalent
`WITH_RIVE_TEXT` configuration, so those alternate bodies are source authority
without a current conditional owner.

## Executable header authority

| # | Pinned inline member | Concrete Rust owner | Candidate disposition |
|---:|---|---|---|
| H1 | `StyledText::unichars` (65) | no retained `StyledText`; resolved text in `StaticShapedTextTopology::text` | **adapted/missing direct accessor** |
| H2 | `StyledText::runs` (66) | `StaticShapedTextTopology::resolved_runs` plus contextual glyphs | **adapted/missing direct accessor** |
| H3 | `StyledText::swapRuns` (68) | no owner | **missing** |
| H4 | `TextValueRunListener::text` (114) | target local ID on `RuntimeArtboardTextListBindingInstance` | **adapted** |
| H5 | `TextValueRunListener::textValueRun` (115) | no concrete dynamic run object; list tuple reconstruction | **missing** |
| H6 | `ColorGlyphCacheKey::operator==` (139) | tuple key `(font_identity,glyph_id)` in `RuntimeTextBackendResources::emoji_images` | **exact key candidate** |
| H7 | `ColorGlyphCacheHash::operator()` (147) | `BTreeMap` tuple ordering | **adapted collection strategy** |
| H8 | `Text::shapeWorldTransform` (196) | `RuntimeCachedTextShapePaints`/`RuntimeRetainedColorGlyphs::shape_world` | **adapted retained owner** |
| H9 | `Text::sizing` (197) | `StaticTextSlice::authored_sizing` | **exact value candidate** |
| H10 | `Text::overflow` getter (199) | `StaticTextSlice::text_uint_property("overflowValue")` | **exact value candidate** |
| H11 | `Text::textOrigin` (200) | `text_uint_property("originValue")` | **exact value candidate** |
| H12 | `Text::verticalTrimTop` (201) | low byte in `static_vertical_trim` | **exact mask candidate** |
| H13 | `Text::verticalTrimBottom` (205) | high byte in `static_vertical_trim` | **exact mask candidate** |
| H14 | `Text::wrap` (209) | `text_uint_property("wrapValue")` | **exact value candidate** |
| H15 | `Text::verticalAlign` (210) | `text_uint_property("verticalAlignValue")` | **exact value candidate** |
| H16 | `Text::overflow(TextOverflow)` (215) | public generated uint property setter path | **adapted signature** |
| H17 | `Text::constraintBounds` (220) | `static_text_constraint_bounds` | **exact delegation candidate** |
| H18 | `Text::effectiveWidth` (230) | `StaticTextSlice::effective_width` | **adapted: constraint option replaces retained NaN sentinel** |
| H19 | `Text::effectiveHeight` (234) | `StaticTextSlice::effective_height` | **adapted: constraint option replaces retained NaN sentinel** |
| H20 | `Text::overflowAsFixed` (239) | `StaticTextSlice::overflow_as_fixed` | **adapted; inherits row 11 mixed-axis defect** |
| H21 | `Text::computedWidth` (244) | retained local bounds width | **exact value candidate** |
| H22 | `Text::computedHeight` (245) | retained local bounds height | **exact value candidate** |
| H23 | `Text::runs` (248) | `StaticTextSlice::runs` plus dynamic `resolved_runs` | **adapted; no public complete retained vector** |
| H24 | `Text::textStylePaints` (254) | `StaticTextSlice::styles` | **adapted; no public pointer vector** |
| H25 | `Text::haveModifiers` (261) | `StaticTextSlice::modifiers` / modifier predicates | **adapted; disabled false has no cfg owner** |
| H26 | testing `orderedLines` (270) | `StaticShapedTextLayout::lines`, debug report projection | **missing literal public observer** |
| H27 | testing `modifierGroups` (274) | `StaticTextSlice::modifiers`, debug report projection | **missing literal public observer** |
| H28 | testing `shape` (278) | `StaticShapedTextTopology`, debug report projection | **missing literal Paragraph/GlyphRun observer** |
| H29 | testing `unichars` (279) | retained topology text/debug projection | **missing literal public observer** |

The eight source-significant header defaults are:

| Default | Pinned value | Rust owner/disposition |
|---|---|---|
| `TextDrawCommand::style` | null | enum payload representation, **adapted safely** |
| `TextValueRunProperty::m_symbolType` | `none` | no per-property object, **missing direct state** |
| `TextValueRunListener::m_text` | null | target local stored on binding, **adapted** |
| `m_layoutWidth` | NaN | constraint option / layout bounds, **adapted** |
| `m_layoutHeight` | NaN | constraint option / layout bounds, **adapted** |
| `m_layoutWidthScaleType` | `uint8_t::max` | `RuntimeTextState::layout_scale_types=None`, **adapted** |
| `m_layoutHeightScaleType` | `uint8_t::max` | same as width, **adapted** |
| `m_layoutDirection` | `inherit` | `RuntimeTextLayoutConstraint.layout_direction`, defaulted at callers, **adapted/pending** |

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

## Blockers for the 16 pending `text_test.cpp` consumers

Pinned consumer: `tests/unit_tests/runtime/text_test.cpp`, 820 lines, 27,373
bytes, SHA-256
`d3917b4de319fbb3d2eb7d4eae1deee4f53d509b460ee2377595696b8bfd5367`.
Cases 2 and 3 are already direct/pass in Wave C7. These remaining rows stay
pending; this source audit does not port tests or substitute debug projections
for their literal observables.

| Case | Pending source blocker |
|---:|---|
| 1 `file with text loads correctly` | Literal typed `find<Text/TextStyle/TextValueRun>` count stream and no-op draw lifecycle are not exposed together; rows 19 and 40 remain adapted. |
| 4 `simple text loads` | Missing literal `Text::shape()` Paragraph/GlyphRun observer (H28), plus empty-shape retained-bounds lifecycle across mutation (rows 34-36, 40, 42). |
| 5 `vertical trim shrinks...` | H29/H28 and direct retained local-bounds observer are incomplete; row 13 uses reconstructed glyph bounds rather than pinned cap/x metrics. |
| 6 trim uint passthroughs | Generated top/bottom masked registry accessors need a literal executable owner and read/write stream; current shaping mask consumption is downstream only. |
| 7 `ellipsis is shown` | Missing literal `orderedLines`, `shape`, `unichars`, and `GlyphLookup` observables (H26/H28/H29); reconstructed line/glyph projections cannot replace them. |
| 8 `fitFontSize shrinks...` | Missing literal shaped run size and `m_transform.xx` observers; row 15 also has a known paragraph-spacing omission. |
| 9 `range mapper maps words` | Owned by `text_modifier_range.cpp`, not this pair; direct `RangeMapper::unitCount` owner/evidence remains required. |
| 10 modifier ranges select runs | Missing literal modifier group/range/run/coverage observer stream (H27) and source-pair certification for modifier owners. |
| 11 varying-size modifier runs | Same H27 blocker plus literal Unicode offset/length/text byte assertions across multiple runs. |
| 12 `double new line type works` | Missing `orderedLines()` observer (H26); custom line splitting is not a substitute for the exact retained ordered-line count. |
| 13 opacity modifiers | Requires literal fixture advance/draw stream after row 17/19/39/40 review; a no-panic proxy is insufficient. |
| 14 zero-width spaces | Requires exact serialized renderer stream and state-machine/view-model action sequence; line breaker and live renderer are still adapted. |
| 15 word joiners | Requires the full nine-frame serialized stream; current frozen divergence (`frame 2 op 262 transform.ty`) remains a real source/owner blocker. |
| 16 `Fit font size` silver | Requires full seven-frame stream; current frozen divergence (`frame 2 op 199 expected makeRenderPath, got rewind`) plus row 15 omission remains red. |
| 17 `Vertical Trim` silver | Requires full timed stream; current frozen divergence (`frame 3 op 220 expected rewind, got drawPath`) and row 13 adaptation remain red. |
| 18 layout-controlled box silver | Requires full five-frame layout/render stream; current frozen divergence (`frame 0 op 61 expected save, got frame`) and rows 10-11 are blockers. |

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

- mixed fixed-width/hug-height layout returns authored sizing in Rust instead
  of pinned `autoHeight`;
- paragraph spacing is dirtied but omitted from Text bounds, fit-font-size,
  render layout, and measurement;
- dynamic list items without `textContent` disappear instead of contributing
  an empty concrete TextValueRun;
- the per-property listener initial-write and synchronous dirt order is
  reconstructed/polled rather than directly owned;
- `controlSize` does not retain and compare the complete pinned five-field
  state in one owner;
- TextValueRun hit rectangles/contours and the direct null `Text::hitTest`
  surface have no literal owner; and
- the 26 disabled-text alternate bodies have no Rust configuration counterpart.

The 16 pending `text_test.cpp` cases are consumers of these owners, not proof
that the source rows are complete. A fresh independent reviewer must try to
falsify every mapping before any production correction begins.
