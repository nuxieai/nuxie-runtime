# `TextStyleAxis` source-pair certification candidate

Status: **author candidate; independent review required**.

This receipt is governed by
`docs/runtime-exact-parity-workflow-correction.md`. It maps the complete pair,
records the source-proven occurrence correction, and does not self-accept the
pair or attribute downstream `TextStyle` behavior to this owner.

## Frozen authority and denominator

- Upstream authority: Rive runtime
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- `src/text/text_style_axis.cpp`: 29 lines, 687 bytes, SHA-256
  `0462a2fb81a963115c3d973d4b3a794a5d8683927f6566d2173f4a3417bf97bd`.
- `include/rive/text/text_style_axis.hpp`: 15 lines, 379 bytes, SHA-256
  `57a07fb76437e0db73029891d885b48c9ab00dcb2fc533397e57d0c9730ee06a`.
- Complete executable denominator: **3 out-of-line cpp bodies and 0
  executable primary-header bodies**.

The complete generated base was also read as necessary context. It contributes
defaults, equal-value guards, setter order, copy, deserialize, type identity,
and empty virtual defaults, but it is not added to the handwritten denominator.

## Complete authority map

| # | Pinned body | Required behavior and order | Concrete Rust owner | Candidate disposition |
|---:|---|---|---|---|
| 1 | `TextStyleAxis::onAddedDirty` (cpp 7-20) | Run Component Super first. Continue only for `Ok`; require the direct parent to be is-a `TextStyle`, otherwise return `InvalidObject`; then append this axis to the parent's `m_variations` in authored construction order and return the Super status. | Generic parent link then direct type guard and registration: `artboard.rs:1360::ArtboardInstance::build_component_occurrence_relations` (axis block at 1416-1432). Occurrence state and clone-cold list: `components.rs:1018-1038::RuntimeTextStyleState`, `:1910::RuntimeConcreteComponentState::clone_for_occurrence`. Import-time malformed-parent stop: `nuxie-binary/src/lib.rs:10410-10435::validate_cpp_text_parentage`. | **corrected exact occurrence ownership**. Previously shaping inferred membership from immutable graph children, so live parent writes could not remain frozen in the current occurrence and re-register on clone. The candidate retains the list on each TextStyle occurrence and reconstructs it from copied parent ids. Rust's malformed file reader performs the same stop during its combined import validation; the runtime construction guard remains authoritative for rebuilt occurrences. |
| 2 | `TextStyleAxis::tagChanged` (cpp 22-25) | After the generated setter stores the new tag, add `TextShape` dirt to the direct retained parent with non-recursive/default-false dirt publication. | Generated typed setter dispatch: `artboard.rs:9824::apply_uint_property_changed` (axis dispatch at 9867); direct callback: `text/text_style_axis.rs:32-50::text_style_axis_uint_property_changed`; generic-tail exclusion: `artboard.rs:7791::mark_text_changed_for_local`. | **corrected exact through the Axis boundary**. The direct is-a TextStyle parent is used and no broader generic Text/render invalidation follows. The downstream TextStyle cascade is separately red below. |
| 3 | `TextStyleAxis::axisValueChanged` (cpp 27-30) | After the generated setter stores the new value, add the same `TextShape` dirt to the direct retained parent, with the same non-recursive order. | Generated typed setter dispatch: `artboard.rs:10483::apply_double_property_changed` (axis dispatch at 10550); direct callback: `text/text_style_axis.rs:11-30::text_style_axis_double_property_changed`; generic-tail exclusion: `artboard.rs:7791::mark_text_changed_for_local`. | **corrected exact through the Axis boundary**, with the same downstream boundary as row 2. |

The Rust callback's direct is-a check is a safe equivalent for valid imported
occurrences. The pinned C++ callback uses `parent()` without a second cast
because row 1 has already made malformed occurrences impossible.

## Generated defaults, setters, notification order, and clone

The pinned generated base initializes `tag=0` and `axisValue=0.0f`. Each setter
returns on equality; otherwise it writes the backing field, invokes the
handwritten callback, then calls `notifyPropertyChanged`. Generated copy copies
both values before `Component::copy`, including `parentId`.

Rust materializes the same defaults in the schema-owned object arena. Typed
setters return false and publish no dirt for equal writes, otherwise write the
occurrence property before owner dispatch and property notification. Clone
copies tag, value, and parent id, clears the occurrence-owned variation list,
and reruns construction. A valid live A-to-B parent write therefore leaves the
source registered with A while its clone registers with B. A copied invalid
parent stops clone reconstruction at the runtime construction guard; Rust's
infallible `Clone` API exposes that `InvalidObject` boundary as its existing
construction panic rather than a nullable C++ instance result. This failure
surface is an explicit Rust API adaptation, not a successful clone.

## Actual shaping ownership

`StaticTextStyle::from_graph_with_occurrence` at `text.rs:5135-5240` now reads
the occurrence-owned axis list whenever an `ArtboardInstance` exists. Both
ordinary Text construction (`text.rs:1827-2145`) and TextInput construction
(`text.rs:2153-2205`) use that route. `variation_values` at `text.rs:5081-5098`
then reads live tag/value properties from exactly those registered axes before
the HarfBuzz and Skrifa option builders consume the stream. The graph-child
scan remains only the explicitly non-occurrence bootstrap/test route.

Supporting evidence at
`text.rs:8084::cxx_text_style_axis_registers_per_occurrence_and_clone_parent`
proves authored order, equal-write no-ops, write-before-parent-dirt observables,
no generic retained-render tail, source A registration after a live write,
clone B re-registration with copied tag/value, occurrence-backed Text and
TextInput option streams, and invalid copied-parent clone failure.
`text.rs:8312::cxx_text_style_axis_invalid_direct_parent_stops_import` proves
the malformed direct-parent import stop.

## Honest downstream reds

These are not Axis bodies and are not claimed by this candidate:

- Current `TextStyle::onDirty` handling raw-adds `TextShape` to its owning Text
  rather than calling pinned `Text::markShapeDirty`, so the complete
  Path/range-map/WorldTransform cascade remains a TextStyle-pair red.
- `TextVariationHelper` construction/dependency topology is still projected
  from immutable graph style children. After an axis reparent and clone, helper
  existence/dependencies can therefore remain tied to the authored style even
  though the concrete shaping option stream now reads the occurrence list.
  That lifecycle belongs to the TextStyle/TextVariationHelper pairs.

## Consumer accounting

No pinned test body mentions `TextStyleAxis`, so the literal owner topology is
exactly **0 direct pass / 0 executable red / 0 adapted / 0 pending**. The tests
in this receipt are supporting owner evidence, not consumer promotions.

The independent recursive asset inventory found **133 assets / 1,039 axis
occurrences**: 123 direct `tests/unit_tests/assets` entries with 979 axes, plus
10 nested entries with 60 axes. Literal source parsing found 296 references,
comprising 278 fixture-to-`TEST_CASE` pairs across 272 unique cases and 18
helper/global sites. Those 272 cases currently classify as 134 pass, 61
expected-red, 3 adapted, 52 pending, and 22 untouched/unmapped; evidence tiers
are 208 accepted, 42 provisional B1, and 22 unmapped.

That large set is an **incidental/unproven initial-shaping impact surface**, not
272 consumers of any of the three handwritten Axis bodies. No case body
exercises Axis registration, a live tag/value callback, or parent/clone
lifecycle. Shared-helper expansion remains unresolved and is not used to infer
materiality or change any case outcome.

## Focused gates

- `cargo test -p nuxie-runtime --no-default-features --features tools --lib cxx_text_style_axis_registers_per_occurrence_and_clone_parent -- --nocapture`: 1 passed.
- `cargo test -p nuxie-runtime --no-default-features --features tools --lib cxx_text_style_axis_invalid_direct_parent_stops_import -- --nocapture`: 1 passed.

## Author conclusion

All three cpp bodies and all primary-header declarations are mapped. The
source-proven parent validation, authored-order occurrence registration,
clone re-registration, callback boundary, and actual Text/TextInput shaping
consumption are corrected. Independent review must verify the exact locators,
the infallible-clone failure adaptation, the consumer denominator, and the two
explicit downstream reds.
