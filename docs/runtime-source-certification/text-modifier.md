# `TextModifier` source-pair certification candidate

Status: **author candidate; independent review required**.

This receipt is governed by
`docs/runtime-exact-parity-workflow-correction.md`. It maps the complete pair,
keeps supporting lifecycle evidence separate from upstream consumers, and does
not self-accept the candidate.

## Frozen authority and denominator

- Upstream authority: Rive runtime
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- `src/text/text_modifier.cpp`: 21 lines, 529 bytes, SHA-256
  `52659ea3db48f7acfc4a31458a620f57c3f29e6783cfe691d5ac9527e103eda5`.
- `include/rive/text/text_modifier.hpp`: 13 lines, 283 bytes, SHA-256
  `ba29e20cfcaea03e58138b0ed25956942d87ac215d0fd9978d5df929483ddcbe`.
- Complete executable denominator: **1 out-of-line cpp body and 0 executable
  primary-header bodies**. The primary header only declares the override.

## Complete authority map

| # | Pinned body | Required behavior and ordering | Concrete Rust owner | Candidate disposition |
|---:|---|---|---|---|
| 1 | `TextModifier::onAddedDirty` (cpp 7-21) | Call Component Super first and immediately propagate non-Ok. After successful parent linkage, append the modifier to its direct parent only when that parent is-a `TextModifierGroup`, preserving authored callback order, then return Ok. Any other valid Container parent returns `MissingObject`: retain the Component Super linkage, do not register, and do not run registration-dependent subclass continuation. Artboard initialization continues on MissingObject. | Component validation/link and authored traversal: `artboard.rs:1362::ArtboardInstance::build_component_occurrence_relations`, especially `:1384-1411`. Direct is-a group registration: `:1412-1430`. Occurrence-owned all/shape/follow vectors: `components.rs:1007::RuntimeTextModifierGroupState`; fresh clone state and reconstruction at `components.rs:1018`, `:1938` and `artboard.rs:1366-1374`. Immutable rendering descriptors consume the same authored group order at `text/text_modifier_group.rs:49::StaticTextModifierGroup::from_graph` and `text/text_modifier.rs:15::StaticTextModifier::from_group_child`. | **corrected exact under retained-local-ID/immutable-descriptor adaptations**. A real occurrence now owns the three vectors populated by the successful callback boundary; cold and already-materialized clones start empty and rebuild them once in authored order. Wrong-parent concrete subclasses retain their Component parent but leave all group vectors untouched and Artboard construction continues. The former late `StaticTextSlice::from_graph` hard rejection is removed. |

## Super status, subclass, and clone adjudication

Pinned `Component::validate` guarantees a resolvable `ContainerComponent`
parent before `onAddedDirty`; Rust's construction boundary likewise rejects a
missing/non-Component/non-container parent before linking. Once linked, the
TextModifier-specific branch is represented without turning `MissingObject`
into an Artboard error. This matches pinned `Artboard::canContinue`, which
stops only on `InvalidObject`.

The pinned schema has exactly two instantiable descendants:
`TextVariationModifier` (through abstract `TextShapeModifier`) and
`TextFollowPathModifier` (through abstract `TextTargetModifier`). The abstract
`TextModifier`, `TextShapeModifier`, and `TextTargetModifier` definitions are
rejected by fixture/import construction and therefore have no occurrence test.
Registration itself uses schema is-a checks, so both concrete inheritance
paths take the same base owner without duplicating subclass algorithms.

`RuntimeTextModifierGroupState` retains local IDs instead of raw pointers.
Its all-modifier, shape-modifier, and follow-path-modifier vectors are cleared
before reconstruction, appended only once in authored order, and initialized
fresh during occurrence clone. `StaticTextModifierGroup` remains the immutable
rendering descriptor for the same imported topology; this is the accepted
graph/occurrence ownership adaptation rather than a test-local vector proxy.

## Source-proven correction and supporting evidence

- `text.rs:8252::cxx_text_modifier_registers_valid_children_in_authored_order_across_clone`
  constructs interleaved FollowPath -> Variation -> FollowPath children. It
  observes the real occurrence owner as all `[5, 6, 7]`, shape `[6]`, and
  follow-path `[5, 7]` on the source occurrence, a cold clone, and a clone after
  static Text materialization. The immutable consumer descriptor has the same
  all-modifier order.
- `text.rs:8351::cxx_text_modifier_missing_group_omits_concrete_subclasses_without_late_rejection`
  constructs both instantiable subclass paths under a non-group Shape. For
  each, Artboard construction succeeds, Component parent linkage remains, the
  real sibling TextModifierGroup registration vectors stay empty, and static
  Text materialization no longer invents a fatal validation. The target-derived
  occurrence also retains no target/Text owner.

The correction removes the old whole-graph validation in
`StaticTextSlice::from_graph` that rejected non-target wrong-parent modifiers
late. That validation contradicted source timing and also treated target and
non-target subclasses differently even though both inherit this one body.

## Consumer topology

A complete scan of all 329 pinned `tests/unit_tests/assets/*.riv` fixtures
finds only `text_follow_path_shape_length.riv` with an instantiable
TextModifier descendant. Accordingly this pair has exactly one material pinned
consumer:

| Pinned case | Classification | Exact evidence |
|---|---|---|
| `tests/unit_tests/runtime/follow_path_constraint_test.cpp` case #7, `Text follow path modifier` | **direct pass** | `tools/silver-corpus/tests/wave_b4.rs:61::wave_b4_text_follow_path_shape_length`; complete ten-frame stream against frozen `text_follow_path_shape_length.sriv` |

Topology is exactly **1 direct pass / 0 executable red / 0 adapted / 0
pending**. `text_modifier_test.cpp`, the range-oriented `text_test.cpp` cases,
and the other TextModifierGroup consumers contain groups/ranges but no concrete
TextModifier descendant, so they are not attributed to this base pair.
Synthetic lifecycle evidence above is not promoted as additional consumers.

## Focused gates

- Exact valid registration/order/subtype/clone owner test: 1 passed.
- Exact malformed concrete-subclass omission/no-late-rejection test: 1 passed.
- `cargo test -p nuxie-runtime d_st_target --lib -- --nocapture`: 4 passed.
- `cargo test -p nuxie-runtime --lib cxx_text_follow_path -- --nocapture`:
  3 passed.
- Exact Wave B4 sole consumer: 1 passed.
- `cargo check -p nuxie-runtime --lib`: passed with pre-existing warnings.

## Honest adjacent residual

The accepted TextModifierGroup construction owner currently turns that pair's
wrong-parent `MissingObject` into a hard Artboard construction error at
`artboard.rs:1435-1444`. The same pinned `Artboard::canContinue` rule proves
that is an adjacent TextModifierGroup lifecycle discrepancy. It is not changed
or hidden in this atomic TextModifier pair and must be the immediate next
source correction after this candidate is accepted.

## Author conclusion

The sole cpp body and all directly necessary header/Super/subclass lifecycle
context are mapped. This candidate corrects the real occurrence registration,
clone reconstruction, malformed-parent continuation, and late-validation
divergence without changing consumer accounting. Independent review must
verify the construction timing, three-vector ownership, locators, and topology
before acceptance.
