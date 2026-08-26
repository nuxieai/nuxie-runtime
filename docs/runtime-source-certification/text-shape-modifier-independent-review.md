# Independent review: `TextShapeModifier`

Candidate `ca2e272c7047ca98bffefd06628f053f194e8c3a` is **accepted** under
`docs/runtime-exact-parity-workflow-correction.md`.

The complete pinned header at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5` is 492 bytes with SHA-256
`b5a2846184c8695cf1c70ed5f42ec1761a0e4deb3a2d8f3a75db31fbcad94f2f`.
It contains 18 newline terminators (`wc -l` convention), 19 displayed logical
lines, and no terminal newline. The executable denominator is zero bodies; the
single authority unit is the pure-virtual const `modify` contract and its
abstract inheritance/type identity.

The frozen source tree and generated C++ core registry have exactly one
constructible descendant: type 162 `TextVariationModifier`. There is no
factory case for abstract type 161. Rust's schema has the same abstract/
concrete relationship; fixture construction rejects the abstract definition
and binary import cannot construct it. `TextFollowPathModifier` descends from
`TextTargetModifier`, not `TextShapeModifier`, in both registries.

The live Rust contract is causal. `TextModifier::onAddedDirty` registration
populates the occurrence-owned shape vector in authored order.
`StaticTextModifierGroup::from_registered_locals` maps those identities to the
closed enum and retains their indices. `variation_map` dispatches those indices
in order with one shared mutable map, the same font and strength, and each
modifier's returned font size threaded into the next call. The Variation owner
uses a value/const receiver, mutates only the shared map, and returns the input
font-size bits; the actual shaping consumer invokes this stream before making
the localized font.

The C++ interface exposes a non-const `Font*`, whereas Rust splits the frozen
Variation behavior across an immutable Skrifa font plus configured axis
values. This is sufficient only because the sole pinned implementation reads
`Font::getAxisValue` and does not mutate Font. It is a closed-SHA adaptation,
not a future-general implementation of arbitrary new shape modifiers; a new
upstream subtype requires an explicit enum/owner mapping.

The 0 direct pass / 0 executable red / 0 adapted / 0 pending consumer topology
is consistent with the accepted complete Variation review at `8e251ab8d`; the
379 pinned assets contain no constructible Variation occurrence. FL-E8 and the
focused tests remain support only. This review does not rely on rejected
Variation stages `618fba0b1` or `4ee75e197`.

All three fully-qualified exact unit tests passed (one test each), and the
explicitly configured, unskipped C++ differential passed:

- `text::tests::d_st_struct_registers_generic_modifiers_and_shape_subtype_indices_in_authored_order`
- `text::tests::d_st_variation_splits_coverage_and_applies_duplicate_unclamped_interpolation`
- `text::tests::cxx_text_variation_modifier_preserves_map_font_size_and_pinned_contraction`
- `RIVE_CPP_PROBE=tools/cpp-probe/build/macosx/bin/debug/rive_cpp_probe ... d_st_variation_live_cpp_axis_value_mutation_matches_rust_update -- --exact --nocapture`

Candidate-range `git diff --check` passed. The candidate is receipt-only, and
all 17 pre-existing user-dirty paths remained unstaged.
