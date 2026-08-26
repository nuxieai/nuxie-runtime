# Independent review: `TextStyleFeature`

Candidate `98cd12e507f3851606e805b88cf79d4ebd841225` is **accepted** under
`docs/runtime-exact-parity-workflow-correction.md`.

I independently read the complete pinned handwritten pair and its directly
required generated base at Rive
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The cpp authority is 19 lines /
511 bytes / SHA-256
`171921dfed01fa935ee6208299d20ddfd6b3549d22e056322228f51acec52ed6`;
the primary header is 13 lines / 324 bytes / SHA-256
`69fad9679eb4414fa2e8b244f2584527f1dc51cafda6c24661384d11b274df01`.
The strict handwritten denominator is exactly one executable body,
`TextStyleFeature::onAddedDirty`; the primary header has no executable body.
The generated base is 90 lines / 2,294 bytes / SHA-256
`e21acb58fd85d7449f535ec3332f300d163dbe7ac1cf7070c498885db4b6619e`
and supplies type/default/setter/copy/deserialize context, not another
handwritten denominator row.

The sole body is mapped completely. Occurrence construction performs Component
Super parent linking first and stops on failure. Only after that succeeds does
the exact direct-parent is-a `TextStyle` guard run; it accepts inherited
`TextStylePaint`, rejects a Shape parent as `InvalidObject`, and appends the
feature local once to the parent occurrence's vector. Authored traversal
preserves order and same-tag duplicate features. The production
`StaticTextStyle` consumer reads that occurrence vector. Live parent writes do
not rerun registration; a cold clone starts with an empty vector and rebuilds
registration from its copied parent IDs, which the retained shaping route
observes.

The generated callback correction is also source-faithful and narrowly
contained. The two generated setters still perform equality suppression,
backing-store mutation, and the ordinary property-notification call in the
pinned store -> empty callback -> notify order. The new dispatcher recognizes
only `tag` and `featureValue` on an is-a `TextStyleFeature` occurrence. It
suppresses the previously invented generic Artboard cache, prepared-frame, and
ancestor Text invalidation tails; unrelated uint properties retain their prior
paths. The remaining generic shape/focus hooks have no Feature ownership and
produce no side effect. Retained-owner evidence proves both writes leave Text,
TextStyle, and helper dirt clean and preserve cache/prepared epochs and the
retained frame. A later genuine root `WorldTransform` helper update rereads the
live feature tag/value and replaces the accepted TextStyle variable-font
snapshot, so callback inaction does not hide the legitimate downstream refresh.

No pinned upstream unit-test source mentions `TextStyleFeature`,
`text_style_feature`, or `featureValue`; literal consumer topology is therefore
**0 pass / 0 red / 0 adapted / 0 pending**. An independent recursive scan of
all 379 pinned unit-test fixtures reproduced **378 readable / 1 unrelated
unreadable / 0 files / 0 objects** containing `TextStyleFeature`.

Focused verification passed:

- retained occurrence order/default/duplicate and callback-inaction test: one
  passed;
- TextStyle option/helper clone reconstruction test: one passed;
- invalid direct-parent import test: one passed;
- `cargo check -p nuxie-runtime --lib`: passed;
- candidate-range `git diff --check`: passed.

The candidate delta is contained to the three declared Rust paths plus its
candidate receipt. All 17 pre-existing user-dirty paths remained outside the
candidate and this review commit.
