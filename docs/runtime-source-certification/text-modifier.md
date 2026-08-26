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
| 1 | `TextModifier::onAddedDirty` (cpp 7-21) | Call Component Super first and immediately propagate non-Ok. After successful parent linkage, append the modifier to its direct parent only when that parent is-a `TextModifierGroup`, preserving authored callback order, then return Ok. Any other valid Container parent returns `MissingObject`: retain the Component Super linkage, do not register, and do not run registration-dependent subclass continuation. Artboard initialization continues on MissingObject. | Component validation/link and authored traversal: `artboard.rs:1362::ArtboardInstance::build_component_occurrence_relations`, especially `:1384-1430`. Occurrence-owned all/shape/follow vectors: `components.rs:1007::RuntimeTextModifierGroupState`; fresh clone state and reconstruction at `components.rs:1018`, `:1938` and `artboard.rs:1366-1374`. Causal static/render descriptor construction: `text.rs:1800::StaticTextSlice::from_instance` and `text/text_modifier_group.rs:93::StaticTextModifierGroup::from_instance`, with all/shape/follow traversal assembled only from the occurrence vectors at `:115::from_registered_locals`. `draw.rs:10508::RuntimeTextDrawOwner::clone` leaves retained topology cold, and live layout, geometry, retained draw, on-dirty, and semantic paths call `from_instance` (`text.rs:145-626`, `text/text_engine.rs:22-67`, `text/raw_text.rs:1031`, `draw.rs:11162-11182`, `:18057`, `:18365-18383`). | **corrected exact under retained-local-ID/immutable-descriptor adaptations**. A real occurrence owns the three vectors populated by the successful callback boundary, and those vectors now causally own production modifier traversal. Live parent-property writes leave the source's registration/topology frozen; cold and already-materialized clones rebuild from copied generated fields rather than inheriting the source topology Arc. Wrong-parent concrete subclasses retain their Component parent but leave all group vectors and production topology untouched, and Artboard construction continues. The former late `StaticTextSlice::from_graph` hard rejection is removed. |

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
fresh during occurrence clone. `StaticTextModifierGroup::from_instance`
materializes the immutable rendering descriptor from those three vectors, so
coverage, shaping, variation, follow-path transform, and draw traversal share
the occurrence owner. `StaticTextSlice::from_graph` remains only the explicitly
named imported-graph bootstrap/support path for callers that have no live
occurrence; it is not used by live production consumers.

## Source-proven correction and supporting evidence

- `text.rs:8272::cxx_text_modifier_registers_valid_children_in_authored_order_across_clone`
  constructs two groups and interleaved FollowPath -> Variation -> FollowPath
  children authored in group A. It writes all three generated `parentId`
  properties to group B. The source occurrence and its actual static/render
  topology stay frozen in A with all `[6, 7, 8]`, shape index `[1]`, and
  follow-path `[6, 8]`. A cold clone builds B through its real retained owner.
  After source topology is retained in A, a second clone is driven through the
  live WorldTransform-onDirty consumer and its retained owner rebuilds all
  three production consumers in B.
- `text.rs:8419::cxx_text_modifier_missing_group_omits_concrete_subclasses_without_late_rejection`
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

## Correction after independent rejection `5ef496593`

The rejected candidate retained correct occurrence vectors but left immutable
graph children as the causal production owner. This correction routes every
live `StaticTextSlice` construction through the occurrence vectors and keeps
graph membership only as a bootstrap/support fallback. The two-group lifecycle
evidence now distinguishes frozen source ownership from clone reconstruction
through the same topology consumed by shaping and rendering.

## Correction after residual rejection `5bf9c1cdd`

`RuntimeTextDrawOwner::clone` no longer copies the source occurrence's retained
`StaticTextSlice` Arc. All other clone state remains the same fresh default
state used before this correction. Exhaustive owner search finds no second
Text draw-owner clone path or topology-copy site. The lifecycle test now
observes source A, cold-clone B, and post-materialized-clone B through the
retained owner itself; the final clone is forced through the real
WorldTransform-onDirty consumer before its retained topology is inspected.

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
