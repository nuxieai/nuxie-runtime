# `TextVariationHelper` source-pair certification candidate

Status: **author candidate; independent review required**.

This receipt is governed by
`docs/runtime-exact-parity-workflow-correction.md`. The complete frozen pair
was read before adjudication. This candidate does not self-accept.

## Frozen authority and denominator

- Upstream authority: Rive runtime
  `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
- `src/text/text_variation_helper.cpp`: 16 lines, 388 bytes, SHA-256
  `a77b191454b98a36372762c07bc0e096ae32468e295f8cb7b64b85f8ad99bb60`.
- `include/rive/text/text_variation_helper.hpp`: 21 lines, 476 bytes,
  SHA-256
  `9f83c4c30e0b3302d5a1bbb4013c5a48ec0782f2bafdc123f529a1bf35c75eae`.
- Strict denominator: **4 executable bodies**: two out-of-line cpp
  definitions and two executable header inlines. The retained `m_textStyle`
  field/default and declarations are mapped as context below, not counted as
  another authority row.

## Complete authority map

| # | Authority unit | Pinned behavior and ordering | Concrete Rust ownership | Disposition |
|---:|---|---|---|---|
| 1 | hpp 11 `TextVariationHelper(TextStyle*)` | Component Super default construction, then retain the exact non-owning `TextStyle*`; no alternative/default helper construction exists. | `objects.rs:77-80::ComponentAddress::TextVariationHelper`; `objects.rs:280-299::attach_text_variation_helper`; `artboard.rs:1960-1989` constructs the embedded occurrence at the owning TextStyle's authored slot and links Component Super to the Artboard root. | **mapped exact**. The Rust occurrence address retains the exact style handle and its then-retained Text parent. |
| 2 | hpp 12 `style() const` | Return the exact retained constructor pointer; no lookup, mutation, or fallback. | `objects.rs:83-89::ComponentAddress::object`; `artboard.rs:9433-9439` dispatches the helper update through the retained `style` handle. `artboard/tests.rs:9839::occurrence_schedule_interleaves_text_helper_before_later_root_child` reads the same occurrence address. | **mapped exact**. The update caller consumes the retained handle rather than rediscovering a style from graph topology. |
| 3 | cpp 7-12 `buildDependencies()` | Read `text = m_textStyle->parent()`, add root/artboard -> helper, then helper -> Text, in that order. | `artboard.rs:2017-2059` inserts `RetainedEdge(root, helper)` followed by `RetainedEdge(helper, text)` at the TextStyle authored slot. `artboard/tests.rs:9839` observes exact root dependent order, helper dependent identity, and sorted dependency order. | **mapped exact**. Static imported helper-edge projections are excluded; source and clone dependencies are rebuilt from occurrence-owned style/Text identity. |
| 4 | cpp 14-17 `update(ComponentDirt)` | Ignore the dirt value and invoke the retained style's `updateVariableFont()` exactly once; publish no additional dirt. | `artboard.rs:9433-9439` passes every scheduled mask to `text/text_variation_helper.rs:7-31::update_text_variation_helper`; that owner ignores `_dirt` and unconditionally enters the already-approved two-part TextStyle update owner. It computes a live replacement every time, exits before the setter when the base font is unavailable, conditionally replaces the cache otherwise, and calls no dirt API. | **adapted through the accepted TextStyle backend owner**. Rust does not contain a literal named `TextStyle::updateVariableFont` call; replacement computation plus conditional cache publication is its approved owner. The base-font-unavailable exit mirrors pinned `updateVariableFont`. Real retained evidence covers an ordinary recursive mask and `FILTHY` (all non-collapsed dirt families), cache replacement, and helper cleanliness after update. |

## Ownership and source-file correspondence

`crates/nuxie-runtime/src/text/text_variation_helper.rs` previously also held
`StyledTextGlyph` and general HarfBuzz shaping helpers, which are not owned by
this pinned pair. This candidate moves those definitions byte-for-byte in
behavior to `text/styled_text.rs` and includes that file at the same lexical
position. The nominal helper file now contains only the mapped update owner.
This is a behavior-neutral source-ownership correction, not a new shaping
algorithm or semantic change.

Rust represents the C++ heap-embedded helper as an occurrence-owned embedded
Component address rather than a separately serialized core object. That is a
named representation adaptation: constructor identity, Component parent,
dependency edges, update scheduling, and clone reconstruction remain directly
observable and source-ordered.

The header's `TextStyle* m_textStyle` field has no independent default: it is
constructor-required, used by both inline `style()` and the two cpp methods,
and is not serialized. Rust stores the corresponding style/Text handles in
the occurrence address (`objects.rs:77-80`) and creates them only from a live
style occurrence (`objects.rs:280-299`). The source occurrence remains frozen;
a cold clone recreates the helper from the cloned TextStyle and its rebuilt
retained Text parent. Immutable graph helper-edge families are explicitly
skipped during dependency construction in favor of these occurrence-owned
retained edges, preventing stale source topology from being projected into a
clone.

## Consumer topology and supporting evidence

An exhaustive source scan of the pinned upstream unit-test tree finds no test
body mentioning `TextVariationHelper` or `text_variation_helper`. Literal owner
topology is therefore exactly **0 direct pass / 0 executable expected-red / 0
adapted / 0 pending**. TextStyle/axis/feature tests exercise this embedded owner
indirectly; they are supporting evidence and are not promoted as consumers.
The frozen fixture projection is **378 readable fixtures**, with **133**
style-axis fixtures projecting **572 embedded helpers from 1,039 axes**. That
surface is incidental only: no upstream test directly asserts this owner.

Focused real-owner evidence:

- `artboard/tests.rs:9839::occurrence_schedule_interleaves_text_helper_before_later_root_child`
  observes the exact retained constructor/style identity, Component root
  parent, helper -> Text dependent, root dependent insertion order, and final
  dependency order.
- `artboard/tests.rs:10224::clone_relinks_text_variation_helper_to_clone_owned_text_parent`
  proves source identity remains frozen while the clone helper is recreated
  against its clone-owned TextStyle parent.
- `text.rs:8185::d_st_feature_preserves_order_defaults_duplicates_and_live_callback_inaction`
  reaches the real helper through recursive root dirt, proves every dirt family
  invokes the same retained update owner, observes the accepted live TextStyle
  variable-font cache replacement, and observes the helper clean after update.
- `draw.rs:27258::transform_only_wave_through_text_variation_helper_retains_text_render_paths`
  proves a WorldTransform-only wave through the retained helper does not
  reshape or replace retained Text render paths. Together with the helper
  owner's direct source containment (there is no dirt API call), this supports
  the pinned no-extra-dirt behavior; the already-dirty `FILTHY` assertion is not
  used for that claim.

Focused gates:

- `cargo test -p nuxie-runtime --lib artboard::tests::occurrence_schedule_interleaves_text_helper_before_later_root_child -- --exact --nocapture`:
  1 passed.
- `cargo test -p nuxie-runtime --lib artboard::tests::clone_relinks_text_variation_helper_to_clone_owned_text_parent -- --exact --nocapture`:
  1 passed.
- `cargo test -p nuxie-runtime --lib text::tests::d_st_feature_preserves_order_defaults_duplicates_and_live_callback_inaction -- --exact --nocapture`:
  1 passed.
- `cargo test -p nuxie-runtime --lib text::tests::cxx_text_style_rebuilds_options_helper_and_retained_text_callbacks_on_clone -- --exact --nocapture`:
  1 passed.
- `cargo test -p nuxie-runtime --lib draw::tests::transform_only_wave_through_text_variation_helper_retains_text_render_paths -- --exact --nocapture`:
  1 passed.
- Scoped `git diff --check` over the five candidate paths: passed.

## Author conclusion

All four executable bodies and the retained-field lifecycle are concretely
mapped. No TextVariationHelper production behavior discrepancy was found. The
only code movement is the behavior-neutral removal of unrelated shaping
ownership from the nominal helper file. Independent review must verify the
embedded-occurrence representation, exact edge order, every-mask update claim,
source-file split containment, and zero-consumer accounting.
