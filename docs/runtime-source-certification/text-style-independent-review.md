# Independent review: `TextStyle`

Candidate `5aba72025d04d8d0c7134df1a30acdb189965aaf` is **rejected** under
`docs/runtime-exact-parity-workflow-correction.md`.

I independently read the complete pinned pair at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The cpp authority is 185 lines /
5,090 bytes / SHA-256
`0d702cd3faf2df687f1fd467619b0b96a8ca64e44cf89bbb6153f4b0c9893850`;
the primary header is 60 lines / 1,736 bytes / SHA-256
`7c8ed1e10980fe8d75e25edb023ffebf2bbb314f5b7b74a4e03731ff1b022883`.
The handwritten denominator is exactly 16 cpp definitions, including the
defaulted constructor, plus the one executable `fontAsset` header inline: 17
authority rows.

## Blocking finding: helper refresh reads a stale feature proxy

Pinned `TextStyle::updateVariableFont` reads every retained
`TextStyleFeature::tag()` and `featureValue()` each time the helper updates.
The feature setters have empty callbacks, so a write does not immediately
dirty or rebuild Text; nevertheless a later unrelated recursive Artboard dirt
wave reaches `TextVariationHelper::update`, which must rebuild the variable
font from those then-current generated values.

Rust does not do that. `StaticTextStyle::variable_font_replacement` at
`crates/nuxie-runtime/src/text.rs:5303-5312` calls the purportedly live
`live_feature_values`, but that function calls
`StaticTextStyleFeature::option` (`text/text_style_feature.rs:52-54`). The
option owner at `artboard/text/text_style.rs:23-55` is cached by
`text_shape_revision`. A dirt-inert feature write does not change that
revision, so the real retained helper update at
`text/text_variation_helper.rs:72-95` republishes the old feature instead of
reading the occurrence property.

I exercised the exact temporal sequence through the existing real fixture and
production owners: create the variable-font cache, write the last `liga`
feature from 1 to 0, verify the immediate callback remains inert, recursively
add root `WorldTransform`, run the actual retained helper in `update_pass`, and
read the production cache through the layout observer. The assertion failed:

```text
left:  [(liga, 1), (liga, 0), (liga, 1)]
right: [(liga, 1), (liga, 0), (liga, 0)]
```

This falsifies rows 7 and 6 at their shared cache boundary. It is not a
downstream shaping approximation: the stale value is already present in the
`RuntimeTextStyleVariableFont` snapshot that all ordinary shaping consumers
read.

Narrow correction: the lazy/helper `updateVariableFont` replacement path must
read current occurrence `tag` and `featureValue` properties directly on every
invocation, bypassing the shape-revision option cache. Ordinary shaping must
continue to read the retained variable-font snapshot, preserving the intended
inert period between the feature write and a later helper update. Add retained-
owner evidence for feature write -> no immediate dirt/cache change -> root
`WorldTransform` helper update -> refreshed cache -> later shaping consumes
the refreshed cache. Do not make `TextStyleFeature` writes eagerly dirty Text
and do not broaden the correction to the separately owned helper pair.

## Other review results and gates

The remaining inspected lifecycle boundaries are consistent with the pinned
pair: occurrence-owned axis/feature insertion, pre-clean null Text state,
helper create/remove/clone and dependency insertion, re-entrant
`markShapeDirty`, inherited `TextStylePaint` callback/import/referencer paths,
stale variable-font retention when the base is unavailable, valid/same-asset
referencer movement, already-dirty suppression, empty `fontAssetIdChanged`,
and cold clone reconstruction. Literal owner topology remains **0 pass / 0
red / 0 adapted / 0 pending**; the five incidental upstream cases remain
pending rather than promoted.

The recursive fixture scan independently reproduced 379 assets, 378 readable,
one unrelated unreadable `solar-system.riv`, 138 readable style-family files,
and 697 objects (1 `TextStyle` plus 696 `TextStylePaint`). The candidate's 136
referenced plus two unreferenced fixture accounting is arithmetically
consistent and does not alter the literal consumer denominator.

- `cargo test -p nuxie-runtime --lib cxx_text_style -- --nocapture`: 4 passed.
- Exact re-entrant shape-dirt test: 1 passed.
- Exact live referencer-movement test: 1 passed.
- Exact binary Backboard/referencer test: 1 passed.
- Reviewer-only feature-write/root-wave assertion: 1 failed with the retained
  values above; the temporary assertion was removed and production/tests were
  restored byte-clean before this receipt.
- Candidate-range `git diff --check` passed; its 13 paths are contained.
- All 17 pre-existing user-dirty paths remained unstaged and outside this
  receipt.

## Narrow feature-cache correction rereview

Correction `4844ed5f0b8e51c668b49250d58634eecdbf33ec` **closes the sole
finding and is accepted**.

The lazy/helper replacement path now uses
`StaticTextStyleFeature::live_option`, which reads current occurrence `tag` and
`featureValue` properties directly on every invocation. It does not consult
the `text_shape_revision` option cache. Ordinary shaping and the report
observer instead use `feature_values`, which returns the retained
`RuntimeTextStyleVariableFont.features` snapshot whenever it exists. The
correction therefore fixes rows 6-7 without making the empty feature callbacks
eager or bypassing the source-visible variable-font cache.

The focused real-owner test reproduces the rejected sequence completely: the
initial retained snapshot is `[(liga,1),(liga,0),(liga,1)]`; the live final
write to zero changes the generated property but leaves Text, Style, helper,
cache, and shaping unchanged; recursive root `WorldTransform` reaches the
retained helper; `update_pass` replaces the cache with
`[(liga,1),(liga,0),(liga,0)]`; and later ordinary shaping consumes that
replacement. No test-local option algorithm or eager Text dirt is involved.

The exact temporal feature test, TextStyle lifecycle test, and re-entrant
shape-dirt test each passed. Correction-range `git diff --check` passed, the
delta is contained to the declared three paths, the 17-row authority and
0/0/0/0 literal-consumer topology plus all fixture counts are unchanged, and
all 17 pre-existing user-dirty paths remain unstaged.
