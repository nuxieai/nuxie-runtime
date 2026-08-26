# `TextFollowPathModifier` source-pair certification candidate

Status: **author candidate; independent review required**.

This receipt is governed by
`docs/runtime-exact-parity-workflow-correction.md`. It does not self-accept the
pair and it does not infer completeness from the existing Silver pass.

## Frozen authority and denominator

- Upstream authority: Rive runtime
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- `src/text/text_follow_path_modifier.cpp`: 206 lines, 6,267 bytes,
  SHA-256 `8fa10143705bc1ac46fa8560642ec64a1b2d820637070e461bb316919405eae9`.
- `include/rive/text/text_follow_path_modifier.hpp`: 36 lines, 1,061 bytes,
  SHA-256 `92d142840608e3e6fbd755fcc3091d08ebc6c08a3ad618094ac5409f370324f6`.
- Complete executable denominator: 12 out-of-line cpp bodies and 0
  executable primary-header bodies. The header declarations and its three
  retained fields are recorded below as state/ownership context.

## Complete cpp body map

| # | Pinned body | Required behavior and ordering | Concrete Rust owner | Candidate disposition |
|---:|---|---|---|---|
| 1 | `buildDependencies` (13) | If target is Shape, composer -> modifier; else if Path, path -> modifier; then modifier -> owning Text when present. | `nuxie-graph/src/lib.rs:4160::text_follow_path_modifier_text_dependencies`; `:4181::text_follow_path_modifier_target_node_dependencies`; insertion into dependency construction at `:3592`, `:3764` | **adapted exact**: graph construction retains the same three conditional edges without C++ mutable dependent lists. Direct cpp-probe evidence at `nuxie-graph/tests/cpp_probe.rs:4077`, `:4190`. |
| 2 | `onAddedClean` (34) | For non-null Shape/Path target, add its follow-path flag, then call `Super::onAddedClean`. Unsupported/null target adds no flag. | `nuxie-runtime/src/artboard.rs:2892::initialize_path_target_flags`, TextFollow branch `:2943-2973`; target resolution in `text/text_follow_path_modifier.rs:15::StaticTextFollowPathModifier::from_graph` | **adapted exact**: occurrence construction publishes the same target flag before normal update traversal. Runtime target cpp-probe covers missing and resolved targets. |
| 3 | `update` (52) | Collect Shape paths in authored order or the single Path; rewind `m_worldPath`; append each raw path transformed by that path's `pathTransform`. Empty/null/unsupported target leaves an empty retained world path. | `text/text_follow_path_modifier.rs:183::world_path_commands`; `:253::update_text_follow_path_world_path`; `artboard.rs:9363-9382`; occurrence state `components.rs:994::RuntimeTextFollowPathState` | **corrected exact**: world commands are now retained on the modifier occurrence at its component update boundary instead of reconstructed per glyph. Rust path geometry applies the retained path transform before ordered append. |
| 4 | `radialChanged` (75) | Call `modifierShapeDirty`, then generated notification tail. | `text/text_follow_path_modifier.rs:277::text_follow_path_modifier_bool_property_changed`; dispatch `artboard.rs:9854`; generic-tail exclusion `artboard.rs:7679` | **corrected exact**: direct Path-only owner replaces the former broad generic Text invalidation. |
| 5 | `orientChanged` (76) | Same direct Path-only callback and notification order. | Same owner as row 4. | **corrected exact**. |
| 6 | `startChanged` (77) | Same direct Path-only callback and notification order. | `text/text_follow_path_modifier.rs:264::text_follow_path_modifier_double_property_changed`; dispatch `artboard.rs:10449`; generic-tail exclusion `artboard.rs:7679` | **corrected exact**. |
| 7 | `endChanged` (78) | Same direct Path-only callback and notification order. | Same owner as row 6. | **corrected exact**. |
| 8 | `offsetChanged` (79) | Same direct Path-only callback and notification order. | Same owner as row 6. | **corrected exact**. |
| 9 | `strengthChanged` (80) | Same direct Path-only callback and notification order. | Same owner as row 6. | **corrected exact**. |
| 10 | `modifierShapeDirty` (81) | Resolve the owning Text through the group and add Path dirt only when Text exists. | `text/text_follow_path_modifier.rs:290::text_follow_path_modifier_shape_dirty` -> `text/text.rs:193::modifier_shape_dirty` | **exact**. Focused real-occurrence evidence proves all six property callbacks leave exactly Text Path dirt. |
| 11 | `reset` (90) | Null target clears `m_pathMeasure` and returns; otherwise rewind local path, append retained world path through Text inverse, then replace measure at tolerance 0.1. The caller must not invoke this body after failed Text inversion, so prior local path/measure survives that failure. | `text/text_follow_path_modifier.rs:198::reset`; `text/text_modifier_group.rs:285::reset_text_follow_path`; `text.rs:2557-2560` | **corrected exact**: occurrence-owned local measure is replaced only after successful inversion; missing inverse returns before every modifier reset. Null resolved target clears it. Real pinned-fixture evidence preserves the previous measure bit-for-bit after a singular Text transform. |
| 12 | `transformGlyph` (102) | Zero-length no-op; ordered `std::min`/`std::max` then Rive clamp; closed-path full-range wrap decision; nested `fmod` offset; before/start-equal, after-end, and in-range samples; normalized tangent; previous/current paragraph baselines; radial/non-radial translation; optional tangent rotation; Rive-clamped strength interpolation into a fresh x/y/rotation result. | `text/text_follow_path_modifier.rs:81::transform_glyph`; retained measure `components.rs:994`; numeric owners `text/text_follow_path_modifier.rs:302-317`; group order in `text/text_modifier_group.rs` | **corrected exact** for the audited pair: retained measure consumption replaces per-glyph reconstruction; ordered min/max, NaN-safe Rive clamp, nested fmod, signed zero, and nonfinite behavior are source-shaped. Existing sampling, baseline, radial/orient, and strength order maps branch-for-branch. |

## Header state, defaults, and callbacks

The primary handwritten header contains declarations only. Its private
`m_worldPath`, `m_localPath`, and `m_pathMeasure` are occurrence-owned by
`components.rs:994::RuntimeTextFollowPathState`; clone construction creates
fresh empty state (`components.rs:1007::clone_for_occurrence`). `m_worldPath`
is replaced by row 3, while `m_localPath` is represented by the command stream
used to replace `m_pathMeasure` in row 11. This is an ownership adaptation, not
shared file-level attribution.

Generated property context was read because the six callbacks depend on it:
`radial=false`, `orient=true`, `start=0`, `end=1`, `strength=1`, and `offset=0`.
Generated setters no-op on equal values; otherwise they write the backing
field, call the corresponding changed override, then publish property
notification. Rust retains those defaults through
`StaticTextFollowPathModifier::{bool_property,double_property}` and routes the
changed callback before the existing property notification tail.

## Consumer topology

The complete pinned consumer inventory has exactly one material case:

| Pinned case | Classification | Exact evidence |
|---|---|---|
| `tests/unit_tests/runtime/follow_path_constraint_test.cpp` case #7, `Text follow path modifier` | **direct pass** | `tools/silver-corpus/tests/wave_b4.rs:61::wave_b4_text_follow_path_shape_length`; complete 10-frame action/render stream against frozen `text_follow_path_shape_length.sriv` |

Topology is therefore exactly **1 direct pass / 0 executable red / 0 adapted /
0 pending**. The case covers five Shape-target modifier groups (the fifth has
start/end data binds), but it does not independently cover a Path target,
invalid target, singular inverse, or retained-state lifetime. The focused
tests below are supporting source-pair evidence and are not promoted or counted
as additional upstream consumers.

## Supporting evidence and gates

- `text.rs:7467::cxx_text_follow_path_retains_local_measure_when_text_inverse_fails`
  uses the real pinned fixture and occurrence-owned state to prove world-path
  retention, successful local measure publication, and failed-inverse
  preservation.
- `text.rs:7510::cxx_text_follow_path_callbacks_publish_path_only` drives all
  six live generated property write routes and observes exact Text Path dirt.
- `text.rs:7549::cxx_text_follow_path_numeric_order_matches_min_max_clamp_and_nested_fmod`
  binds the numeric edge semantics to the production helpers, including NaN,
  infinity, and signed zero.
- `cargo test -p nuxie-runtime --lib cxx_text_follow_path -- --nocapture`: 3
  passed.
- `cargo test -p silver-corpus --test wave_b4 wave_b4_text_follow_path_shape_length -- --exact --nocapture`:
  1 passed.
- The two exact graph cpp-probe tests at lines 4077/4190 passed.
- `cargo test -p nuxie-runtime --features tools --test cpp_probe d_st_target_live_cpp_missing_target_is_ok_like_rust -- --exact`:
  1 passed, covering both missing and valid target topology.
- `cargo check -p nuxie-runtime --lib`: passed (pre-existing warnings only).

## Author conclusion

All 12 cpp bodies and all executable/header state context are concretely
mapped. This author correction closes the source-proven numeric, callback, and
retained-state discrepancies without changing the one-consumer topology.
Independent review must verify ownership timing, exact locators, and the
candidate classifications before this pair can be accepted.
