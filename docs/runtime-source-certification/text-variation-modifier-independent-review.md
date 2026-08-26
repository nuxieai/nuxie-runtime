# Independent review: `TextVariationModifier`

Candidate `35739d0f098bcee240ab9d674b173104352fe061` is **rejected** under
`docs/runtime-exact-parity-workflow-correction.md`.

I independently read the complete pinned
`src/text/text_variation_modifier.cpp` and
`include/rive/text/text_variation_modifier.hpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. Their hashes and denominator are
confirmed: 23 lines / 667 bytes / `d2632594...` and 17 lines / 486 bytes /
`b8ab42e9...`, with two cpp bodies and no executable primary-header bodies.
The configured/default/zero fallback, map precedence and single-tag overwrite,
unchanged font-size return, compiler-scoped contraction, valid callback dirt
order, malformed-hierarchy safety adaptation, clone registration, and zero
actual pinned-consumer accounting have no separate blocker in this review.

## Blocking finding

The documented phase-6 differential is a `TextVariationModifier` tag-authority
failure, not a demonstrated downstream Range/group-only residual. Pinned
`modify` reads `axisTag()` directly on every invocation. Rust instead routes
`StaticTextVariationModifier::tag` through
`ArtboardInstance::text_variation_modifier_tag`, which caches the tag under the
Artboard-wide `text_shape_revision`.

The `axisTag` write correctly stores the new field without dirt, so the
already-shaped phase remains unchanged. The later range-strength write then
calls `rangeChanged`; because the group contains a shape modifier, it reaches
`Text::modifierShapeDirty` and legitimately requests a fresh shape. Rust's
`modifier_shape_dirty`, however, adds Path dirt without advancing
`text_shape_revision`. The next real shaping call therefore reuses the cached
old `wght` tag instead of reading the live `wdth` field. This exactly matches
the failing differential: at phase 6 C++ reports one `wdth=1300.0` glyph while
Rust reports 13 absent `wdth` values.

Narrow correction: make every concrete Variation `modify` invocation observe
the current occurrence-owned `axisTag`, as the pinned getter does. Reading the
live generated field directly is the source-faithful solution; alternatively,
any cache must be invalidated by every path that causes actual reshaping,
including `modifierShapeDirty`. Evidence must preserve the inert immediate
axisTag phase and then prove that the subsequent range-strength reshape uses
the new tag. The exact seven-phase C++ differential must pass through phase 6;
the failure cannot remain classified as outside these two bodies.

Focused results:

- `cargo test -p nuxie-runtime --lib cxx_text_variation_modifier_ -- --nocapture`:
  2 passed.
- `cargo test -p nuxie-runtime --features tools --test cpp_probe d_st_variation_live_cpp_axis_value_mutation_matches_rust_update -- --exact --nocapture`:
  failed at phase 6 with Rust 0 versus C++ 1 present `wdth` value.

Candidate-range `git diff --check` passed. The candidate contains the declared
five paths, and all 17 pre-existing user-dirty paths remained outside it.
