# `TextVariationModifier` source-pair certification candidate

Status: **author candidate; independent review required**.

Correction after independent rejection `618fba0b1`: the previous candidate
incorrectly attributed the phase-6 differential to Range/group reshaping. The
actual defect was the revision-keyed Variation tag cache. This correction
removes that cache, restores a live tag read at every `modify`, and fixes the
probe's phase-5 Rust observer to retain output when the empty callback requests
no update.

This receipt is governed by
`docs/runtime-exact-parity-workflow-correction.md`. It does not self-accept the
pair. The complete pinned asset inventory has no material consumer, so every
test below is supporting evidence rather than a consumer promotion.

## Frozen authority and denominator

- Upstream authority: Rive runtime
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- `src/text/text_variation_modifier.cpp`: 23 lines, 667 bytes, SHA-256
  `d263259426c410933ea60dc62953cf531956c3e451505dd224263058d91c280c`.
- `include/rive/text/text_variation_modifier.hpp`: 17 lines, 486 bytes,
  SHA-256
  `b8ab42e92b31b1c3484e381e2a95cb435043c65ec830c862ea63df3920cebc5b`.
- Complete executable denominator: **2 out-of-line cpp bodies and 0
  executable handwritten-header bodies**.

## Complete authority map

| # | Pinned body | Required behavior and ordering | Concrete Rust owner | Candidate disposition |
|---:|---|---|---|---|
| 1 | `TextVariationModifier::modify` (cpp 7-19) | Look up `axisTag` in the current variations map first; otherwise use `Font::getAxisValue` (configured font value, axis default, then zero for an unknown axis). Compute `from * (1-strength) + axisValue * strength` in pinned compiled order, overwrite only that tag, preserve every other map entry, and return `fontSize` unchanged. No clamping or finite checks. | Concrete modifier/default lookup and mutation: `text/text_variation_modifier.rs:12::text_variation_modifier_interpolate` and `:44::StaticTextVariationModifier::modify`. Ordered group call and font-size threading: `text/text_modifier_group.rs:447::StaticTextModifierGroup::variation_map`. Actual shaping consumer: `text.rs:3961::StaticTextSlice::styled_text_glyphs_for_style_with_strengths`. | **corrected exact for the supported arm64/clang build**. The prior Rust expression rounded the two products and addition separately, and the owner discarded the literal return value. The candidate uses `mul_add` for the source-proven contraction and threads the unchanged font size through the concrete modifier stream. Existing map values win; the accepted Rust font representation supplies authored/configured values before Skrifa's axis default and zero. |
| 2 | `TextVariationModifier::axisValueChanged` (cpp 21-24) | After the generated setter has stored the new value, cast the direct parent to `TextModifierGroup` and call `shapeModifierChanged`, which reaches the direct owning Text's `markShapeDirty`: Path first, range maps/coverage in order, WorldTransform last. | Callback dispatch: `artboard.rs:10542::ArtboardInstance::apply_double_property_changed` -> `text/text_variation_modifier.rs:73::text_variation_modifier_double_property_changed`. Direct group/Text safety guards: `:84-93` and `text/text_modifier_group.rs:477::modifier_group_text`. Exact dirt owner: `text/text.rs:100::mark_shape_dirty_with_layout`. Generic invalidation exclusion: `artboard.rs:7776::mark_text_changed_for_local`. | **corrected exact on valid occurrences, with named Rust safety adaptation for malformed hierarchy**. The old callback walked two generic parents and could dirty a grandparent Text through a Shape; the generic property tail also invalidated retained render styles and made the generated empty `axisTagChanged` observable. The candidate requires direct group and direct Text ownership and excludes Variation from the generic tail. Malformed casts safely become a no-op rather than invoking C++ undefined behavior. |

## Generated state, defaults, and clone lifecycle

The generated base was read as necessary context. `axisTag` defaults to zero
and `axisValue` to `0.0f`. Both generated setters are equal-value no-ops,
otherwise store before invoking their callback and property notification.
`axisTagChanged` is empty; `axisValueChanged` is the handwritten body above.
Generated copy copies both fields. Rust stores live generated properties in
the occurrence, and every concrete `modify` call reads the current tag through
`artboard/text/text_variation_helper.rs:5`; there is no revision-keyed tag
cache. A live parent write does not rerun construction, while a cold clone
reconstructs occurrence registration from the copied parent ID. Supporting
evidence proves the source keeps its old callback owner, the clone resolves
the new group/Text, and both live tag and value are copied.

## Floating-point provenance

This classification is architecture/compiler scoped, not inferred from the
Rust spelling. On arm64, Homebrew clang 22.1.8 compiling the pinned expression
at `-O2` emits `fsub`, `fmul axis*strength`, then
`fmadd from,(1-strength),(axis*strength)`. A Rust 1.97.1 `-O` probe of the
candidate helper emits `fmul axis*strength` followed by
`llvm.fma.f32(from,1-strength,axis*strength)`; the generated probe IR SHA-256
was `e40a4be28a56063ee528150ca390d9e81aaaaa7e367e9b8cf15ee57c2f069319`.
The direct owner counterexample uses `from=0xc389eceb`,
`axisValue=0xc321b678`, `strength=0x3c8a8fc1` and proves the contracted result
`0xc388f5ce`.

## Consumer topology

An exhaustive scan of the 379 pinned assets found **zero**
TextVariationModifier consumers. The topology is therefore exactly **0 direct
pass / 0 executable red / 0 adapted / 0 pending**. The FL-E8 generated
fixture, direct owner tests, and cpp probe are support only.

## Supporting evidence and honest residual

- `text.rs:8122::cxx_text_variation_modifier_preserves_map_font_size_and_pinned_contraction`
  exercises the real modifier/group owner and proves existing-map precedence,
  configured/default/unknown-axis fallback, overwrite/preservation, exact
  contracted bits, NaN propagation, and bit-preserved font-size return.
- `text.rs:8218::cxx_text_variation_modifier_axis_value_callback_requires_direct_group_and_text`
  proves valid callback order and dirt, no eager retained-render invalidation,
  empty axisTag behavior, both malformed hierarchy no-ops, live-parent freeze,
  and cold-clone callback re-registration with copied tag/value.
- `cpp_probe.rs:90357::d_st_variation_live_cpp_axis_value_mutation_matches_rust_update`
  executes the real C++ probe and the full seven-phase cycle. The immediate
  axisTag phase retains the preceding output because its generated callback is
  empty. The later legitimate Range strength reshape calls concrete
  `Variation::modify`, reads live `wdth`, and matches the C++ phase-6
  `[Some(1300.0), None, ...]` stream.

Focused gates:

- `cargo test -p nuxie-runtime --lib cxx_text_variation_modifier_ -- --nocapture`:
  2 passed.
- `cargo test -p nuxie-runtime --features tools --test cpp_probe d_st_variation_live_cpp_axis_value_mutation_matches_rust_update -- --exact --nocapture`:
  1 passed, including all seven phases.

## Author conclusion

Both cpp bodies, all handwritten-header declarations, and directly necessary
generated defaults/copy behavior are mapped. The source-proven arithmetic,
font-size stream, callback hierarchy, invalidation timing, and clone ownership
gaps are corrected. There are no upstream consumer cases to promote.
Independent review must verify the compiler-scoped arithmetic classification,
the malformed-cast safety adaptation, and the residual dependency boundary.
