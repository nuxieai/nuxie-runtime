# `TextStyleFeature` source-pair certification candidate

Status: **author candidate; independent review required**.

This receipt is governed by
`docs/runtime-exact-parity-workflow-correction.md`. The complete pinned cpp and
primary handwritten header were read before adjudication. This candidate does
not self-accept.

## Frozen authority and strict denominator

- Upstream authority: Rive runtime
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- `src/text/text_style_feature.cpp`: 19 lines, 511 bytes, SHA-256
  `171921dfed01fa935ee6208299d20ddfd6b3549d22e056322228f51acec52ed6`.
- `include/rive/text/text_style_feature.hpp`: 13 lines, 324 bytes,
  SHA-256
  `69fad9679eb4414fa2e8b244f2584527f1dc51cafda6c24661384d11b274df01`.
- Strict handwritten executable denominator: **1 body**,
  `TextStyleFeature::onAddedDirty`. The primary header contains only the class
  declaration/override plus include guard and includes; it has no executable
  inline, macro body, field, default, callback, or other behavioral unit.

## Complete handwritten authority map

| # | Pinned authority | Required behavior and ordering | Concrete Rust ownership | Candidate disposition |
|---:|---|---|---|---|
| 1 | cpp 7-19 `TextStyleFeature::onAddedDirty(CoreContext*)` | Invoke Component Super first. If Super is not `Ok`, propagate that status unchanged and do nothing else. After `Ok`, require the retained direct parent to be is-a `TextStyle`; otherwise return `InvalidObject`. On success, cast the same parent and append this exact feature once to `TextStyle::m_styleFeatures`, preserving authored callback order. | Component Super retained-parent construction and status boundary: `artboard.rs:1385-1415::build_component_occurrence_relations`. Direct is-a TextStyle guard and ordered registration: `artboard.rs:1443-1456`. Occurrence list: `components.rs:1018-1056::RuntimeTextStyleState::{register_feature,feature_locals}`. Actual production consumer: `text.rs:5244-5262::StaticTextStyle::from_graph_with_occurrence`, with direct retained-parent validation in `text/text_style_feature.rs:17-50::StaticTextStyleFeature::from_graph_with_occurrence`. Cold clone state is rebuilt from copied generated parent ids via `components.rs:1038-1040::clone_for_occurrence` and the same construction loop. | **mapped exact under the existing combined Rust construction/status adaptation**. A Super failure stops before feature validation/registration; a non-TextStyle direct parent is fatal `InvalidObject`; valid TextStyle and TextStylePaint parents append occurrence-owned feature identity in authored order. Live parent writes do not rerun the callback, while clone construction rebuilds registration against the copied parent. |

## Generated context boundary

The directly necessary generated base was also read, but it is not added to
the one-body handwritten denominator. It supplies type key 164; Component
inheritance; `tag=0` and `featureValue=1` defaults; guarded setters; store ->
empty callback -> `notifyPropertyChanged` order; clone/copy/deserialization;
and empty virtual `tagChanged`/`featureValueChanged` callbacks.

The complete pair proved a previously hidden Rust discrepancy at that boundary.
The common uint setter stored and notified correctly, but then treated the two
empty callbacks as a generic Text/prepared-frame mutation. Depending on the
imported ancestor index, this could dirty the owning Text's retained render
styles, queue Paint work, and advance render cache/prepared epochs even though
pinned C++ performs no such callback side effect.

This candidate corrects only that source-proven seam:

- `text/text_style_feature.rs:74-94::text_style_feature_uint_property_changed`
  identifies exactly the two inherited generated keys for any is-a
  TextStyleFeature occurrence and reports an empty handled callback.
- `artboard.rs:5984-6024::set_uint_property` retains the generated store and
  property notification, but does not turn that notification into cache or
  prepared invalidation.
- `artboard.rs:7849-7868::mark_text_changed_for_local` excludes the same
  is-a owner from the unrelated generic Text invalidation tail.
- `artboard.rs:9944-9959::apply_uint_property_changed` dispatches through the
  pair-owned empty callback boundary. No unrelated uint owner is suppressed.

Ordinary shaping intentionally continues to consume its retained feature
snapshot. A later legitimate `TextVariationHelper` update rereads the current
feature properties through the separately accepted TextStyle owner; this pair
does not make feature writes eagerly reshape Text.

## Rust source ownership

`crates/nuxie-runtime/src/text/text_style_feature.rs` contains only this pair's
generated callback boundary and its concrete retained feature representation:
direct-parent validation, authored defaults, retained/live option reads, and
the HarfBuzz feature consumer. No unrelated text algorithm or packed upstream
owner remains in the nominal file, so no behavior-neutral split was needed.

## Consumer and recursive fixture accounting

An exhaustive scan of the pinned upstream unit-test source found no
`TextStyleFeature`, `text_style_feature`, or `featureValue` owner mention.
Literal owner topology is exactly **0 direct pass / 0 executable expected-red /
0 adapted / 0 pending**.

A recursive scan of all 379 pinned `tests/unit_tests/assets/**/*.riv` files
found **378 readable / 1 unrelated unreadable** (`solar-system.riv`) and **0
files / 0 objects** containing a `TextStyleFeature`. Therefore the frozen
fixture-consumer denominator is also zero. The repository's FL-E8
`text_style_feature.riv` is focused supporting evidence created for this port;
it is not an upstream fixture consumer.

## Supporting evidence and focused gates

- `text.rs:8186::d_st_feature_preserves_order_defaults_duplicates_and_live_callback_inaction`
  uses the real occurrence list and retained Text draw owner. Both tag and
  featureValue writes store their new values while Text, TextStyle, and helper
  dirt remain clean; retained shape identity/glyphs/pending dirt/render-style
  flags remain byte-for-byte equal; cache and prepared epochs do not advance.
  The later legitimate helper wave reads the live values and only then replaces
  the accepted variable-font snapshot.
- `text.rs:8573::cxx_text_style_rebuilds_options_helper_and_retained_text_callbacks_on_clone`
  separately proves a source feature remains registered in its original
  TextStyle after a live parent write, while a cold clone re-registers it in
  the copied new TextStyle and actual production shaping consumes that clone
  topology.
- `text.rs:8951::cxx_text_style_feature_invalid_direct_parent_stops_import`
  proves the InvalidObject boundary for a direct Shape parent.

Focused commands and results:

- `cargo test -p nuxie-runtime --features tools --lib text::tests::d_st_feature_preserves_order_defaults_duplicates_and_live_callback_inaction -- --exact --nocapture`:
  1 passed.
- `cargo test -p nuxie-runtime --lib text::tests::cxx_text_style_rebuilds_options_helper_and_retained_text_callbacks_on_clone -- --exact --nocapture`:
  1 passed.
- `cargo test -p nuxie-runtime --lib text::tests::cxx_text_style_feature_invalid_direct_parent_stops_import -- --exact --nocapture`:
  1 passed.
- `cargo check -p nuxie-runtime --lib`: passed.
- Scoped candidate `git diff --check`: passed.

## Author conclusion

The sole handwritten body is completely mapped, the primary header contains no
hidden executable denominator, and the generated property context is explicit.
Occurrence registration, direct-parent failure, live-source freeze, clone
re-registration, authored order, and callback inaction are bound to real
production owners. The candidate corrects one source-proven generic
invalidation leak. Independent review must verify that strict denominator,
combined construction adaptation, exact callback containment, clone evidence,
and both zero-consumer counts.
