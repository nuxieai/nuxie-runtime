# `TextModifierRange` source-pair certification candidate

Status: **author candidate; pending independent semantic review**

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Frozen authority

- `src/text/text_modifier_range.cpp`: 386 lines, 11,288 bytes, SHA-256
  `85c892fd95a7e82536e0d8ffacc139636ae04011262844cfa198ed632952e8f2`.
- `include/rive/text/text_modifier_range.hpp`: 161 lines, 4,616 bytes,
  SHA-256
  `1ab54caee20ef33d8f7c94efb2e885b863cae5ba2b2b40173585a234b8b3d606`.

The denominator is **37 executable authority units**: 23 out-of-line bodies
and 14 header inlines. Generated property storage is not added to that
denominator, but its defaults and callback order were read and are recorded
below. No blanket file-level mapping is used.

## Out-of-line authority map

| # | Pinned body | Concrete Rust owner | Candidate disposition |
|---:|---|---|---|
| 1 | `TextModifierRange::onAddedDirty` (10) | construction validation `artboard.rs:1417-1451::ArtboardInstance::build_component_occurrence_relations`; frozen run resolution `text/text_modifier_range.rs:114::from_graph`; group child-order collection `text/text_modifier_group.rs:49::from_graph` | **adapted correction candidate**: Component parent linking occurs first; construction then rejects a non-group parent or unresolved/non-TextValueRun local `runId`, and the resolved local is frozen before instance publication. Pointer registration is immutable graph ownership. |
| 2 | `addChild` (38) | `text/text_modifier_range.rs:135-150::from_graph` | **adapted correction candidate**: all children remain owned by the graph/Super path; every cubic is visited in child order and the last cubic replaces the prior interpolator. The former non-cubic/duplicate rejection is removed. |
| 3 | `clearRangeMap` (48) | `components.rs:1078::RuntimeTextState::clear_modifier_range_map`; ordered caller `text/text.rs:139::mark_shape_dirty_with_layout` | **exact retained occurrence effect under map-storage adaptation**. |
| 4 | `computeRange` (50) | lazy map owner `text/text_modifier_range.rs:280::range_map`; cache `components.rs:1059::modifier_range_units` | **adapted/incomplete**: lazy nonempty retention, run clipping, and unit dispatch are present; line units still receive the wrong pre-shape line owner (row 23). |
| 5 | `coverageAt` (82) | `text/text_modifier_range.rs:391::coverage_at` | **exact correction candidate**: ordering, falloff branches, and cubic invocation are preserved, including cubic calls for falloff-branch endpoint values. |
| 6 | `computeCoverage` (123) | `text/text_modifier_range.rs:164::apply_coverage`; cached indices `components.rs:1082::modifier_range_indices` | **adapted correction candidate/incomplete**: retained index state, ordered modes, spacing zeroing, and clamp are exact; a valid run owned by another Text lacks a concrete retained `TextValueRun::offset/length` owner (row 1 residual). |
| 7 | `modifyFromChanged` (190) | `text/text_modifier_group.rs:426::text_modifier_group_double_property_changed` -> `403::range_changed` | **exact enabled callback/order**. |
| 8 | `modifyToChanged` (194) | same as row 7 | **exact**. |
| 9 | `strengthChanged` (198) | same as row 7 | **exact**. |
| 10 | `unitsValueChanged` (202) | `text/text_modifier_group.rs:458::text_modifier_group_uint_property_changed`, units branch 474 | **exact enabled callback/order**: Text Path-only dirt precedes group TextCoverage. |
| 11 | `typeValueChanged` (206) | same owner, recognized ordinary branch 477-480 | **exact enabled callback/order**. |
| 12 | `modeValueChanged` (210) | same owner as row 11 | **exact enabled callback/order**. |
| 13 | `clampChanged` (216) | `text/text_modifier_group.rs:447::text_modifier_group_bool_property_changed` | **exact enabled callback/order**, accepted by the prior Text rows 24/25 review. |
| 14 | `falloffFromChanged` (220) | double callback owner in row 7 | **exact**. |
| 15 | `falloffToChanged` (224) | double callback owner in row 7 | **exact**. |
| 16 | `offsetChanged` (228) | double callback owner in row 7 | **exact**. |
| 17 | `needsShape` (233) | `text/text_modifier_range.rs:160::needs_shape` | **exact pair owner, dependent consumer red**: it returns true only for line units, but `TextModifierGroup::needsShape` still omits this range scan. |
| 18 | `RangeMapper::clear` (238) | occurrence removal `components.rs:1078::clear_modifier_range_map`; cold clone `components.rs:1004::RuntimeTextState::clone_for_occurrence` | **exact effect under occurrence map adaptation**. |
| 19 | `unitToCharacterRange` (244) | `text/text_modifier_range.rs:55::StaticRangeMap::unit_to_character_range` | **exact correction candidate** with direct fractional/clamped evidence. |
| 20 | `addRange` (264) | `text/text_modifier_range.rs:71::add_range_unit` | **exact intersection/order**. |
| 21 | `fromWords` (283) | `text/text_modifier_range.rs:21::StaticRangeMap::from_words`, scanner at 355 | **exact correction candidate**: word units and terminal sentinel are the live RangeMap owner. |
| 22 | `fromCharacters` (325) | `text/text_modifier_range.rs:322::character_range_units`; real unmodified-shape counts `text.rs:3722::styled_resolved_run_glyphs`, `5311::styled_glyph_lookup_counts` | **exact correction candidate for current shaped Text**: character and excluding-space units advance by `GlyphLookup` span, keeping ligatures indivisible; unknown units take the pinned default character branch. |
| 23 | `fromLines` (350) | `text/text_modifier_range.rs:373::line_range_units`; caller lines in `text.rs:2420-2455::layout_from_shaped_topology` | **incorrect/dependent red**: the range pair clips lines correctly, but shape-modifier coverage is still first requested with source-newline lines rather than pinned wrapped glyph-line endpoints. |

## Executable header authority

| # | Pinned inline | Concrete Rust owner | Candidate disposition |
|---:|---|---|---|
| H1 | `unitCount` | `text/text_modifier_range.rs:28::unit_count` | **exact**. |
| H2 | `unitCharacterIndexCount` | `text/text_modifier_range.rs:32::unit_character_index_count` | **exact**, including terminal sentinel. |
| H3 | `empty` | `text/text_modifier_range.rs:36::empty` | **exact**: keyed by unit lengths, not sentinel presence. |
| H4 | `unitCharacterIndex` | `text/text_modifier_range.rs:40::unit_character_index` | **exact valid-domain checked-index adaptation**. |
| H5 | `unitLength` | `text/text_modifier_range.rs:48::unit_length` | **exact valid-domain checked-index adaptation**. |
| H6 | `units` | units dispatch `text/text_modifier_range.rs:280::range_map` | **exact enum behavior**, including unknown/default -> characters. |
| H7 | `type` | `text/text_modifier_range.rs:193-216::apply_coverage` | **exact correction candidate**: unknown values retain the occurrence's prior four indices. |
| H8 | `mode` | `text/text_modifier_range.rs:228-238::apply_coverage` | **exact correction candidate**: unknown values leave current coverage unchanged. |
| H9 | TESTING `interpolator` | `text/text_modifier_range.rs:137-150::from_graph` | **adapted read-only identity**: last authored cubic local/global pair. |
| H10 | `offsetModifyFrom` | `text/text_modifier_range.rs:192-207::apply_coverage` | **exact addition before type scaling**. |
| H11 | `offsetModifyTo` | same owner as H10 | **exact**. |
| H12 | `offsetFalloffFrom` | same owner as H10 | **exact**. |
| H13 | `offsetFalloffTo` | same owner as H10 | **exact**. |
| H14 | TESTING `run` | frozen `text/text_modifier_range.rs:5::run_local`; retained observer `text.rs:210-226::static_text_layout_debug_report` | **adapted/incomplete**: local identity is exact and no longer re-read after `runId` mutation; cross-Text run offset/length state remains absent. |

## Defaults, state, and corrections

Pinned generated defaults are `modifyFrom/falloffFrom/offset = 0`,
`modifyTo/strength/falloffTo = 1`, units/type/mode `0`, clamp `false`, and
`runId = Core::emptyId`. Rust property reads use those values. The four cached
coverage indices start at zero per occurrence and survive invalid type writes;
range-map clearing deliberately does not clear them. A `runId` write retains
the generic property notification but no longer calls `rangeChanged`, matching
the empty generated `runIdChanged` callback and the frozen import-time pointer.

This candidate corrects: construction timing and frozen local run resolution;
last-cubic/Super child behavior; unknown units/type/mode behavior; retained
coverage indices; ordered C++ NaN behavior for min/max/clamp; cubic falloff
endpoint invocation; ligature-aware character units; and the missing concrete
RangeMap/sentinel/unit-to-character owner. The two-pass shape seam uses the
real unmodified glyph stream before applying shape-modifier coverage; it is not
a test-local shaping algorithm.

Focused evidence:

- `text.rs::cxx_range_mapper_maps_words_and_converts_fractional_units` covers
  the complete upstream case 9 assertion, sentinel/fractional conversion, and
  a synthetic real-owner ligature span;
- `text.rs::cxx_text_modifier_range_constructs_with_frozen_run_and_last_cubic_child`
  covers valid construction, frozen run identity, last-cubic wins, and real
  construction rejection for wrong parent/run targets;
- the accepted Text rows 24/25 evidence continues to cover every range
  callback's Text-before-group dirt order and retained map clearing.

## Consumers and residuals

The material pinned consumers in `text_test.cpp` are cases 9-11:

- case 9 `range mapper maps words`: **pass/direct**. The literal Rust assertion
  now calls `debug_text_word_unit_count`, which delegates to the production
  `StaticRangeMap::from_words::unit_count` owner rather than the prior line-break
  proxy;
- case 10 `run modifier ranges select runs`: **pending**. Its existing port
  recomputes coverage through `RuntimeTextLayoutDebugReport`; it is not a
  literal observer of retained group/range/run/coverage state;
- case 11 varying-size runs: **pending** for the same observer gap plus the
  cross-run Unicode offset/length/text-byte stream.

Accordingly this pair's consumer topology is **1 pass / 0 executable red / 2
pending**. The broader 18-case Text topology becomes **5 pass / 3 executable
expected-red / 10 pending**. No projection was promoted.

Two dependent source reds remain explicit:

1. `TextModifierGroup::needsShape` and `Text::update` do not yet route line-unit
   ranges through the pinned pre-shape wrapped-line pass (pair row 17/23 and
   TextModifierGroup row 28).
2. A valid `runId` may point to a `TextValueRun` outside the current Text. Rust
   validates and freezes that identity but lacks the other occurrence's
   retained `offset/length` owner, so coverage cannot yet consume it literally.

No source split is attempted in this candidate. The pair is already isolated
in `text/text_modifier_range.rs`; the remaining state lives on the Text
occurrence and the construction/callback boundaries necessarily remain in
their owning modules.
