# `TextShapeModifier` source-pair certification candidate

Status: **author candidate; independent review required**.

This receipt is governed by
`docs/runtime-exact-parity-workflow-correction.md`. The authority is an
abstract, header-only interface; this candidate makes no production change and
does not self-accept it.

## Frozen authority and denominator

- Upstream authority: Rive runtime
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- `include/rive/text/text_shape_modifier.hpp`: 18 lines, 492 bytes, SHA-256
  `b5a2846184c8695cf1c70ed5f42ec1761a0e4deb3a2d8f3a75db31fbcad94f2f`.
- Complete executable denominator: **0 bodies**. The sole authority unit is
  one pure-virtual contract plus its inheritance/type identity.

## Complete contract map

| Authority unit | Pinned contract | Concrete Rust ownership | Disposition |
|---|---|---|---|
| `TextShapeModifier : TextShapeModifierBase`; pure virtual `modify(Font*, unordered_map<uint32_t,float>&, float fontSize, float strength) const -> float` (hpp 8-16) | A concrete shape modifier receives the same font, one group-call-owned mutable variation map, the previous modifier's returned font size, and the group's current strength. Shape modifiers execute in authored registration order. Each returns the font size passed to the next modifier. Abstract interface objects cannot be constructed. | Pinned schema inheritance/type identity: `nuxie-schema/src/generated/schema.rs:31097-31124`. Occurrence subtype registration: `artboard.rs:1404-1438::build_component_occurrence_relations`. Authored-order enum/index construction: `text/text_modifier.rs:4::StaticTextModifier`, `text/text_modifier_group.rs:157-179::StaticTextModifierGroup::from_registered_locals`. Concrete dispatch and shared mutable state: `text/text_modifier_group.rs:447-469::variation_map`. Actual shaping caller supplies font/fontSize/strength at `text.rs:3987-4017::styled_text_glyphs_for_style_with_strengths`. The only pinned concrete implementation is `text/text_variation_modifier.rs:40::StaticTextVariationModifier::modify`. Abstract import exclusion: `nuxie-binary/src/lib.rs:7901-7915` and `:11590-11599`. | **mapped exact for the pinned closed subtype set**. Rust uses exhaustive enum dispatch rather than a vtable. Both pinned core registry and source tree contain exactly one constructible descendant, TextVariationModifier. The group carries one mutable map and returned font size through every authored-order Variation entry; the concrete implementation preserves font size exactly. The final group result intentionally consumes the map and not the returned font size, matching pinned `TextModifierGroup::modifyShape`, which also returns `run.size`. |

## Dispatch, abstractness, and extension boundary

The pinned C++ core registry has no factory case for abstract type key 161 and
constructs only `TextVariationModifier` for this hierarchy. Rust schema marks
TextShapeModifier abstract. Fixture construction rejects it explicitly, and
binary import filters abstract definitions before object construction. The
`StaticTextModifier::Abstract` enum arm is therefore defensive internal graph
representation, not a callable implementation of the pure virtual contract;
it cannot enter a valid imported occurrence's shape-modifier dispatch.

There is no unknown-subtype parity gap at the frozen SHA. C++'s frozen core
registry and Rust's frozen schema both require an explicit new subtype before
it can be constructed. A future upstream shape modifier would require a new
source-pair mapping and enum dispatch arm; silently treating such a future type
as the current Variation algorithm would be incorrect.

## Consumer topology and supporting evidence

The exhaustive pinned asset inventory found no constructible
TextVariationModifier, hence no material pure-virtual dispatch consumer. This
pair's topology is exactly **0 direct pass / 0 executable red / 0 adapted / 0
pending**. FL-E8 is generated supporting evidence only:

- `text.rs:7684::d_st_struct_registers_generic_modifiers_and_shape_subtype_indices_in_authored_order`
  proves two concrete shape modifiers retain authored modifier order and shape
  subtype indices `[0, 1]` through the real occurrence/group owner.
- `text.rs:8052::d_st_variation_splits_coverage_and_applies_duplicate_unclamped_interpolation`
  drives the actual group dispatch and proves the shared map/strength behavior
  across the two concrete entries.
- `text.rs:8138::cxx_text_variation_modifier_preserves_map_font_size_and_pinned_contraction`
  proves the concrete implementation mutates only its tag and returns the
  input font-size bits unchanged.
- `cpp_probe.rs:90380::d_st_variation_live_cpp_axis_value_mutation_matches_rust_update`
  exercises the real retained owner and full seven-phase C++/Rust stream; it is
  supporting interface evidence, not an upstream consumer promotion.

Focused gates:

- `cargo test -p nuxie-runtime --lib text::tests::d_st_struct_registers_generic_modifiers_and_shape_subtype_indices_in_authored_order -- --exact --nocapture`:
  1 passed.
- `cargo test -p nuxie-runtime --lib text::tests::d_st_variation_splits_coverage_and_applies_duplicate_unclamped_interpolation -- --exact --nocapture`:
  1 passed.
- `cargo test -p nuxie-runtime --lib text::tests::cxx_text_variation_modifier_preserves_map_font_size_and_pinned_contraction -- --exact --nocapture`:
  1 passed.
- `RIVE_CPP_PROBE=tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe cargo test -p nuxie-runtime --features tools --test cpp_probe d_st_variation_live_cpp_axis_value_mutation_matches_rust_update -- --exact --nocapture`:
  the executable was present and the unskipped C++ differential passed.

## Author conclusion

The complete header was read and its only pure-virtual contract is causally
mapped through registration, ordered dispatch, concrete mutation, and shaping
consumption. The closed pinned subtype set needs no production correction.
Independent review must verify the abstract-import boundary, exhaustive enum
claim, and zero-consumer accounting.
