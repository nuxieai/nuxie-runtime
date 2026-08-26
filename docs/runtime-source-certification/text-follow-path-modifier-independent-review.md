# Independent review: `TextFollowPathModifier`

Candidate `f9d8eaa9884d8ab254ab538d1aceb0f670f68f93` is **accepted** under
`docs/runtime-exact-parity-workflow-correction.md`.

I independently read the complete pinned
`src/text/text_follow_path_modifier.cpp` and
`include/rive/text/text_follow_path_modifier.hpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`, then traced every cited concrete
Rust owner. The frozen hashes are
`8fa10143705bc1ac46fa8560642ec64a1b2d820637070e461bb316919405eae9` and
`92d142840608e3e6fbd755fcc3091d08ebc6c08a3ad618094ac5409f370324f6`.
The executable denominator is confirmed as 12 cpp bodies, zero executable
primary-header bodies, and three retained private fields plus generated
property defaults/callback context.

## Semantic verdict

No concrete blocker remains. The graph owners preserve the conditional Shape
composer/direct Path dependencies and modifier-to-owning-Text edge, while the
occurrence construction owner publishes the target flag before update
traversal. The modifier update retains authored-order world path commands with
the pinned raw-path/path-transform rule. All six generated property routes
write first, invoke the direct Path-only Text dirt owner, and then use the
existing notification tail.

Reset and glyph transformation also match the pinned ownership and ordering:
failed Text inversion skips reset and retains the prior local measure; a
resolved null/unsupported target clears it after a successful inverse; a valid
target rebuilds it at tolerance 0.1. The live transform consumes that retained
measure and preserves the zero-length return, ordered C++ min/max, Rive
`fmaxf`/`fminf` clamp behavior, nested `fmod`, signed-zero/nonfinite behavior,
closed-path wrap, before/in/after-range sampling, baseline/radial/orient
composition, and final strength interpolation. Occurrence cloning starts with
fresh empty retained state. The evidence reads or drives these production
owners; it does not implement a parallel path algorithm.

The complete material consumer topology remains exactly **1 direct pass / 0
red / 0 adapted / 0 pending**. Wave B4 case 7 retains its distinct ten-frame
Silver stream. The prior `TextModifierGroup` row 13 follow-path dependent red
is correctly refreshed by this accepted owner; row 28 and the broader Text
consumer topology are otherwise unchanged.

## Independent focused gates

- `cargo test -p nuxie-runtime --lib cxx_text_follow_path -- --nocapture`:
  3 passed, 0 failed.
- `cargo test --release -p nuxie-runtime --lib
  cxx_text_follow_path_numeric_order_matches_min_max_clamp_and_nested_fmod --
  --nocapture`: 1 passed, 0 failed.
- `cargo test -p nuxie-graph --test cpp_probe
  graph_dependency_order_includes_text_follow_path_dependencies --
  --nocapture`: 1 passed, 0 failed.
- `cargo test -p nuxie-graph --test cpp_probe
  cpp_text_follow_path_dependency_method_is_tracked_by_graph_model --
  --nocapture`: 1 passed, 0 failed.
- `cargo test -p nuxie-runtime --features tools --test cpp_probe
  d_st_target_live_cpp_missing_target_is_ok_like_rust -- --nocapture`: 1
  passed, 0 failed.
- `cargo test -p silver-corpus --test wave_b4
  wave_b4_text_follow_path_shape_length -- --exact --nocapture`: 1 passed, 0
  failed.
- `git diff --check f9d8eaa98^ f9d8eaa98`: passed.

All 17 pre-existing user-dirty paths remained unstaged and outside this review
receipt.
