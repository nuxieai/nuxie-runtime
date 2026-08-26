# `TextTargetModifier` source-pair certification candidate

Status: **author candidate; independent review required**.

This receipt is governed by
`docs/runtime-exact-parity-workflow-correction.md`. It does not self-accept the
pair and it keeps supporting lifecycle evidence separate from the sole pinned
consumer.

## Frozen authority and denominator

- Upstream authority: Rive runtime
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- `src/text/text_target_modifier.cpp`: 30 lines, 765 bytes, SHA-256
  `e0c6bd9f73031fb5b68a903350c808b57e49f7f88fc31fee9ef54c989a663bd4`.
- `include/rive/text/text_target_modifier.hpp`: 19 lines, 461 bytes,
  SHA-256
  `4be7851d8c1cd3ecf0b879bcb5154898e1ef4ad7ef7fd1ac64a7ace76c701ef1`.
- Complete executable denominator: **2 out-of-line cpp bodies and 0
  executable handwritten-header bodies**. The header's retained field and
  generated base behavior are state/default context, not extra executable
  authority units.

## Complete authority map

| # | Pinned body | Required behavior and ordering | Concrete Rust owner | Candidate disposition |
|---:|---|---|---|---|
| 1 | `TextTargetModifier::onAddedDirty` (cpp 9-21) | Invoke `TextModifier::onAddedDirty` first and immediately propagate a non-Ok status. Only after success, resolve the current `targetId`, cast/store it in occurrence-owned `m_Target`, and return Ok. Missing resolution stores null. A later `targetId` write does not change `m_Target`; a clone copies the generated ID into fresh state and reruns resolution. | Super prerequisite/registration: `text/text_target_modifier.rs:20::text_target_modifier_resolution`, `text/text_modifier.rs:12::StaticTextModifier::from_group_child`, and construction links in `artboard.rs:1378-1453::build_component_occurrence_relations`. Occurrence state: `components.rs:999::RuntimeTextTargetState`, allocation/clone at `:1806`/`:1884`. Import and clone resolution: `artboard.rs:2847`, `:798`, and `:2894::initialize_text_target_modifiers`. Live generated property read: `text/text_target_modifier.rs:52::text_target_modifier_target_id`; retained read: `:59::text_target_modifier_target_local`. | **corrected exact under named ownership/safety adaptations**. The previous static descriptor repeatedly resolved the authored target, so a live write followed by clone still consumed the old target. The target is now retained per occurrence, live writes freeze the current target, and clone construction resolves the copied live ID. Target-derived wrong-parent `Super` failure skips resolution and modifier participation while preserving the Artboard-level `MissingObject` continuation behavior. Rust retains a local component ID instead of a raw pointer. |
| 2 | `TextTargetModifier::textComponent` (cpp 23-30) | If and only if the direct parent is a `TextModifierGroup`, return that group's `textComponent`; otherwise return null. Group resolution in turn accepts its parent only when it is-a `Text`. | Direct group guard and delegation: `text/text_target_modifier.rs:44::text_target_modifier_text_component` -> `text/text_modifier_group.rs:386::modifier_group_text`; real occurrence observations at `text.rs:8369`, `:8381`, and `:8431`. | **corrected exact**. The direct parent is now independently required to be-a TextModifierGroup before group-to-Text traversal. A malformed `Text -> Shape -> TextFollowPathModifier` therefore returns null instead of incorrectly reaching the grandparent Text. `TextInput` is correctly excluded because pinned generated `TextInputBase` derives from `Drawable`, not `Text`; no TextInput parity claim is made. |

## Header state, generated defaults, and adaptations

The handwritten header declares the two bodies above and initializes
`TransformComponent* m_Target = nullptr`. The corresponding Rust state is
`RuntimeTextTargetState::target_local`, initialized to `None` and deliberately
reset to `None` by `clone_for_occurrence` before clone construction resolves
the copied generated property. This is the repository's established local-ID
ownership adaptation for C++ pointers; consumers resolve the retained local
against their own Artboard occurrence.

The generated base was read as necessary context. `m_TargetId` defaults to
`uint32_t(-1)`. Its setter is an equal-value no-op; otherwise it writes the ID,
invokes the empty `targetIdChanged`, then publishes the property notification.
Its `copy` copies `m_TargetId` and then calls the `TextModifier` copy. Rust's
generated property storage already preserves that write/copy behavior, while
this candidate intentionally does not connect the live write to retained
target resolution.

Pinned C++ uses an unchecked `static_cast<TransformComponent*>` after generic
context resolution. That relies on a valid-file type invariant and is unsafe
for malformed input. Rust resolves only schema `TransformComponent`
descendants and otherwise retains `None`. This is an explicit memory-safety
adaptation, not a claim that malformed wrong-type files literally execute C++
undefined behavior.

## Consumer topology

The complete pinned consumer inventory has exactly one material case:

| Pinned case | Classification | Exact evidence |
|---|---|---|
| `tests/unit_tests/runtime/follow_path_constraint_test.cpp` case #7, `Text follow path modifier` | **direct pass** | `tools/silver-corpus/tests/wave_b4.rs:61::wave_b4_text_follow_path_shape_length`; complete ten-frame stream against frozen `text_follow_path_shape_length.sriv` |

Topology is exactly **1 direct pass / 0 executable red / 0 adapted / 0
pending**. This consumer exercises the derived TextFollowPathModifier's normal
resolved-target path. It does not independently cover target lifetime,
missing/wrong targets, wrong-parent Super failure, or clone re-resolution.
Those are supporting pair evidence below and are not promoted as additional
upstream consumers.

## Source-proven corrections and supporting evidence

- `text.rs:8488::d_st_target_live_write_freezes_current_target_and_clone_reresolves`
  uses the real pinned follow-path fixture and real Artboard occurrences. It
  proves authored target A is retained, a live `targetId = B` write leaves the
  current target frozen at A, and cloning copies B then resolves the clone's
  fresh target to B.
- `text.rs:8369::d_st_target_missing_resolution_is_ok_and_retains_no_target`
  proves a successful Super path with empty generated ID returns a usable
  modifier occurrence, retains no target, and resolves its owning Text.
- `text.rs:8381::d_st_target_wrong_parent_super_failure_skips_target_resolution`
  gives the malformed target modifier a valid Transform target, proves
  Artboard construction continues on target-derived `MissingObject`, and
  proves neither target resolution nor Text registration occurs.
- `text.rs:8431::d_st_target_non_group_parent_cannot_reach_grandparent_text`
  covers the blocking counterexample from independent rejection `21e38bdc2`:
  a real `Text -> Shape -> TextFollowPathModifier` occurrence retains no target
  and no Text, and a live `start` write does not dirty the grandparent Text.
- The derived TextFollowPath owner now reads the retained target for its reset,
  world-path update, and follow-path target flags
  (`text/text_follow_path_modifier.rs:126`, `:184`; `artboard.rs:2983`). This
  removes the stale authored-descriptor route without changing the derived
  pair's numeric/path algorithm.

Focused gates:

- `cargo test -p nuxie-runtime d_st_target --lib -- --nocapture`: 4 passed.
- `cargo test -p nuxie-runtime --lib cxx_text_follow_path -- --nocapture`: 3
  passed.
- `cargo test -p nuxie-runtime --features tools --test cpp_probe d_st_target_live_cpp_missing_target_is_ok_like_rust -- --exact --nocapture`:
  1 passed.
- `cargo test -p silver-corpus --test wave_b4 wave_b4_text_follow_path_shape_length -- --exact --nocapture`:
  1 passed.
- `cargo check -p nuxie-runtime --lib`: passed with pre-existing warnings.

## Honest residual boundary

This pair corrects target-derived lifecycle timing only. The general
`TextModifier::onAddedDirty` behavior for non-target modifiers with malformed
parents belongs to the immediately subsequent complete TextModifier source
pair and remains unclaimed here. The sole upstream consumer topology is
unchanged.

## Author conclusion

Both cpp bodies and all directly necessary header/generated state are mapped.
The source-proven occurrence and clone lifecycle gap is corrected without
promoting projections or inventing a behavior from tests. Independent review
must verify construction timing, ownership, locators, and the safety-adaptation
classification before acceptance.
