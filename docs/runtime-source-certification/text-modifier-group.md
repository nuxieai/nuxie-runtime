# `TextModifierGroup` source-pair certification candidate

Authority is pinned at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. This candidate follows
`docs/runtime-exact-parity-workflow-correction.md`; it does not inherit a
file-level mapped/faithful label and is not independently accepted.

## Frozen denominator

- `src/text/text_modifier_group.cpp`: 412 lines, 10,780 bytes, SHA-256
  `cf631d9960dcac834d8b46d705cfbdc39909d3fcecf9e05c286dfd344119dfed`.
- `include/rive/text/text_modifier_group.hpp`: 140 lines, 4,261 bytes,
  SHA-256
  `cfc7de561f45dca1e7ca1b03805751b0c8b54896ff0e553cc3a47d17001bf721`.

The `.cpp` contains 28 executable bodies: 27 class members and the local
`copyRun` static. The handwritten header contributes ten executable inlines:
the `TransformGlyphArg` constructor, `coverage`, six flag predicates, and two
`TESTING` accessors. The authority denominator is therefore 38 units. The
nine generated property defaults and callbacks were read as directly
necessary context; they are recorded under retained state rather than falsely
added to the handwritten-pair denominator.

The concrete Rust owner is
`crates/nuxie-runtime/src/text/text_modifier_group.rs`, with occurrence and
consumer seams in `text.rs`, `text/text.rs`, `text/text_modifier_range.rs`,
`text/text_variation_modifier.rs`, `text/text_follow_path_modifier.rs`,
`text/glyph_lookup.rs`, `components.rs`, and `draw.rs`. Whole-file attribution
is not evidence: every row below names its actual symbol.

Disposition terms are candidate classifications. `exact` means the live
supported path preserves the source behavior. `adapted` names the immutable
graph/occurrence representation used instead of C++ pointers. `incorrect`
means a concrete source discrepancy remains.

## Out-of-line authority map

| # | Pinned body | Source contract | Concrete Rust owner | Candidate disposition |
|---:|---|---|---|---|
| 1 | `textComponent` (13) | Return the parent only when it is a `Text`; otherwise null. | `text/text_modifier_group.rs:368::modifier_group_text` | **exact value under occurrence adaptation** |
| 2 | `onAddedDirty` (22) | Run `Super`; on success register with a direct `Text` parent, otherwise `MissingObject`. | import validation `text.rs:1995-2009`; authored collection `text.rs:2112-2118::StaticTextSlice::from_graph` | **adapted correction candidate**: graph import replaces pointer registration and now rejects a non-Text parent. |
| 3 | `addModifierRange` (39) | Append range in callback/child order. | `text/text_modifier_group.rs:49::StaticTextModifierGroup::from_graph`, range branch 84-90 | **exact order under immutable descriptor adaptation** |
| 4 | `addModifier` (44) | Append every modifier; additionally append shape and follow-path subtypes in the same order. | `text/text_modifier_group.rs:91-111::StaticTextModifierGroup::from_graph`; subtype owner `text/text_modifier.rs:12::StaticTextModifier` | **exact order under immutable descriptor adaptation** |
| 5 | `rangeTypeChanged` (57) | `Text::modifierShapeDirty`, then add group `TextCoverage`. | `text/text_modifier_group.rs:394::range_changed(path_only=true)`; dispatch `449::text_modifier_group_uint_property_changed` | **exact supported callback/order** |
| 6 | `shapeModifierChanged` (63) | Invoke `Text::markShapeDirty()` only. | `text/text_variation_modifier.rs:60::text_variation_modifier_double_property_changed` -> `text/text.rs:57::mark_shape_dirty` | **exact correction candidate**: removed the prior premature group-coverage publication. |
| 7 | `rangeChanged` (68) | Shape group -> `modifierShapeDirty`; paint-only group -> paint dirt; then group `TextCoverage`. | `text/text_modifier_group.rs:379::group_has_shape_modifier`, `394::range_changed`; double/bool/uint dispatch at 417/438/449 | **exact supported callback/order** |
| 8 | `clearRangeMaps` (86) | Clear every range map in order, then add group `TextCoverage`. | occurrence state `components.rs:1076::RuntimeTextState::clear_modifier_range_map`; ordered owner `text/text.rs:99::mark_shape_dirty_with_layout` | **adapted/exact retained state and order** |
| 9 | `computeRangeMap` (95) | Visit every range in order with current text, shape, lines, and glyph lookup. | `text/text_modifier_group.rs:332::coverage_by_character` -> `text/text_modifier_range.rs:96::apply_coverage` and `195::range_units` | **adapted/incomplete**: lazy range materialization replaces the explicit phase; line-unit ranges do not receive the pinned pre-shape wrapped-line stream (see row 28). |
| 10 | `computeCoverage` (107) | Dirt guard; clear own dirt; resize/zero retained coverage; apply every range in order. | `text/text_modifier_group.rs:332::coverage_by_character` | **adapted**: returns a freshly zeroed occurrence value on each consumer rebuild rather than retaining `m_coverage`/self-clearing non-DAG dirt; range application order is preserved. |
| 11 | `glyphCoverage` (127) | Assert at least one code point, sum retained per-character coverage, divide by count. | `text/glyph_lookup.rs:13::glyph_coverage`; call `text.rs:2600` | **exact on the live valid glyph domain under checked-slice adaptation** |
| 12 | `onTextWorldTransformDirty` (140) | A follow-path group adds `Path` dirt to its Text; other groups do nothing. | target enumeration `text.rs:1725::StaticTextSlice::world_dirty_modifier_groups`; dispatch `draw.rs:11996::RuntimeTextOnDirtyTargets::visit` | **exact under retained callback-target adaptation** |
| 13 | `resetTextFollowPath` (149) | If Text is absent or world inversion fails, return without mutating retained path measures; otherwise reset every follow-path modifier with the inverse. | `text.rs:2582` plus `text/text_follow_path_modifier.rs:177::path_measure` | **incorrect**: Rust uses `invert_or_identity` and reconstructs path measure rather than preserving the prior retained measure on inverse failure. |
| 14 | `transform` (163) | Exact early return; follow-path/translation, scale and rotation accumulation; compose; add origin, multiply, subtract origin. Scale lanes use the authored contracted expression. | `text/text_modifier_group.rs:124::StaticTextModifierGroup::transform`; composition `11::apply_text_modifier_transform`; scale `38::text_modifier_group_scale_component` | **exact correction candidate subject to row 13's follow-path input gap**: both scale lanes now retain pinned fused results. |
| 15 | `computeOpacity` (230) | Inverted: `current*(1-t)+opacity*t`; otherwise `current*opacity*t`, with pinned grouping/contraction. | `text/text_modifier_group.rs:297::opacity`; contraction `44::text_modifier_group_inverted_opacity` | **exact correction candidate** |
| 16 | `modifierFlagsChanged` (242) | Mark parent Text paint dirty. | `text/text_modifier_group.rs:449::text_modifier_group_uint_property_changed` | **exact** |
| 17 | `originXChanged` (246) | Mark parent Text paint dirty. | `text/text_modifier_group.rs:417::text_modifier_group_double_property_changed` | **exact** |
| 18 | `originYChanged` (250) | Mark parent Text paint dirty. | same owner as row 17 | **exact** |
| 19 | `opacityChanged` (254) | Mark parent Text paint dirty. | same owner as row 17 | **exact** |
| 20 | `xChanged` (258) | Mark parent Text paint dirty. | same owner as row 17 | **exact** |
| 21 | `yChanged` (259) | Mark parent Text paint dirty. | same owner as row 17 | **exact** |
| 22 | `rotationChanged` (260) | Mark parent Text paint dirty. | same owner as row 17 | **exact** |
| 23 | `scaleXChanged` (264) | Mark parent Text paint dirty. | same owner as row 17 | **exact** |
| 24 | `scaleYChanged` (268) | Mark parent Text paint dirty. | same owner as row 17 | **exact** |
| 25 | `copyRun` (273) | Copy every run field except replace `unicharCount`. | segmentation owner `text.rs:3747::StaticTextSlice::styled_resolved_run_glyphs`; run fields `text/text_variation_helper.rs:2::StyledTextGlyph` | **adapted**: immutable segment reconstruction carries source style/font metrics and replaces the character span rather than copying a mutable `TextRun`. |
| 26 | `modifyShape` (287) | Resolve the authored style by shaper ID; return unchanged without style/font; run shape modifiers in order from the style font; replace font only when coordinates are produced. | `text/text_modifier_group.rs:347::variation_map`; consumer `text.rs:3909::styled_text_glyphs_for_style_with_strengths` | **exact for the current concrete variation modifier under immutable-font adaptation** |
| 27 | `applyShapeModifiers` (332) | No-op without shape modifiers; split runs at equal coverage; zero coverage copies the incoming run, nonzero coverage calls `modifyShape`; swap replacement runs after each group. | coverage segmentation `text.rs:3747::styled_resolved_run_glyphs`; replacement `3909::styled_text_glyphs_for_style_with_strengths` | **adapted correction candidate**: every nonzero group now restarts from the authored style font and replaces the prior group's coordinates; zero strength retains the preceding result. Immutable reconstruction replaces `m_nextTextRuns`/`swapRuns`. |
| 28 | `needsShape` (399) | True for any shape modifier, otherwise true on the first range whose unit mapping needs shaped lines. | `text/text_modifier_group.rs:363::has_shape_modifiers` | **incorrect**: current predicate omits line-unit `TextModifierRange::needsShape`, and the corresponding pre-shape wrapped-line coverage pass is absent. |

## Executable header authority

| # | Pinned inline | Concrete Rust owner | Candidate disposition |
|---:|---|---|---|
| H1 | `TransformGlyphArg` constructor (28) | `text/text_modifier_group.rs:469::StaticTextGlyphContext`; construction `text.rs:2592` | **adapted/exact values**: origin includes half advance, offset starts zero per group, line index/baselines are retained. |
| H2 | `coverage` (61) | slice access inside `text/glyph_lookup.rs:13::glyph_coverage` | **exact on valid live indices under safe indexing adaptation** |
| H3 | `modifiesTransform` (70) | `text/text_modifier_group.rs:28::text_modifier_group_modifies_transform` | **exact mask** |
| H4 | `modifiesOpacity` (79) | `text/text_modifier_group.rs:283::modifies_opacity` | **exact mask** |
| H5 | `modifiesRotation` (85) | rotation branch in `text/text_modifier_group.rs:215::transform` | **exact mask** |
| H6 | `modifiesTranslation` (91) | follow-path offset and translation branches in `text/text_modifier_group.rs:164/194::transform` | **exact mask** |
| H7 | `modifiesScale` (97) | scale branch in `text/text_modifier_group.rs:226::transform` | **exact mask** |
| H8 | `modifiesOrigin` (103) | origin branch in `text/text_modifier_group.rs:255::transform` | **exact mask** |
| H9 | `ranges` TESTING accessor (114) | public count `text.rs:123::RuntimeTextModifierDebugReport::range_count`; internal descriptors `text/text_modifier_group.rs:5::StaticTextModifierGroup::ranges` | **adapted/incomplete read-only surface**: tests inside the owner can inspect descriptors, but the public debug report does not expose range identities. |
| H10 | `modifiers` TESTING accessor (115) | `text.rs:126::RuntimeTextModifierDebugReport::modifier_locals`; source descriptors at `text/text_modifier_group.rs:6` | **adapted read-only evidence surface** |

## Retained state and defaults

Generated defaults read by this pair are: flags `0`; origin, translation and
rotation `0`; opacity and both scales `1`. Runtime property reads in
`StaticTextModifierGroup::transform`, `modifies_opacity`, and `opacity` use
those same defaults. Generic setters retain the pinned no-op-on-equal contract
before dispatching the callbacks in rows 16-24.

The C++ vectors `m_ranges`, `m_modifiers`, `m_shapeModifiers`, and
`m_followPathModifiers` map to the four authored-order vectors on
`StaticTextModifierGroup`. `m_coverage` is reconstructed as an occurrence
value by `coverage_by_character`; range-map identity remains occurrence-owned
in `components.rs:982::RuntimeTextState::modifier_range_maps`. The variable
font, coordinate, and next-run scratch vectors map to the immutable shaping
stream in rows 26-27. This representation is explicit adaptation, not literal
pointer or allocation identity.

## Corrections and evidence

This candidate makes five source-proven corrections:

1. each successive nonzero shape group restarts from the authored style font
   instead of inheriting the preceding group's axes;
2. both transform scale lanes retain the pinned finite contraction;
3. inverted opacity retains its distinct pinned finite contraction;
4. `TextVariationModifier::axisValueChanged` reaches the single pinned
   `markShapeDirty` sequence without premature group dirt; and
5. a modifier group without a direct Text parent is rejected at import.

Focused actual-owner evidence:

- `text.rs::cxx_successive_modifier_groups_restart_from_the_authored_style_font`;
- `text.rs::cxx_text_modifier_group_retains_pinned_scale_and_inverted_opacity_contractions`;
- `text.rs::cxx_text_modifier_group_requires_a_direct_text_parent`;
- existing callback/state evidence
  `text.rs::d_st_variation_splits_coverage_and_applies_duplicate_unclamped_interpolation`;
- retained range/order evidence
  `text.rs::cxx_text_shape_dirty_clears_retained_range_maps_before_group_and_world_dirt`;
- transform/origin evidence
  `text.rs::opacity_only_text_modifier_returns_incoming_ctm_without_identity_multiply`
  and
  `text.rs::text_modifier_origin_preserves_pinned_three_step_ctm_composition_bits`.

## Upstream consumer topology

Thirteen material upstream cases consume this pair. They remain separated from
the focused source evidence above:

- four executable pass: `listener_align_target_test.cpp` #1/#2 incidentally,
  `follow_path_constraint_test.cpp` #7, and
  `serialized_rendering_test.cpp` #6;
- two executable expected-red: `serialized_rendering_test.cpp` #12 (rewards)
  and #15 (hunter). Both freeze at earlier blend-mode differences, so the scale
  correction does not move their first divergence;
- seven pending: `serialized_rendering_test.cpp` #11,
  `text_modifier_test.cpp` #1/#2, and `text_test.cpp` #9/#10/#11/#13. The
  inverted-opacity correction is consumed by pending modifier #2 and Text #13;
  range cases remain blocked by H27 rather than promoted through a proxy.

The complete pair remains **not exact** because rows 13 and 28 are red. No
upstream consumer moved in this atomic unit. The broader Text topology likewise
remains four pass, three executable expected-red, and 11 pending. Focused
source evidence is not counted as consumer promotion.
