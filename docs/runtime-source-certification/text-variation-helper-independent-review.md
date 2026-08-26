# Independent review: `TextVariationHelper`

Candidate `a70fd02abeaab0444113134a16d79901fd6af74e` is **accepted** under
`docs/runtime-exact-parity-workflow-correction.md`.

I independently read the complete pinned pair at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The cpp authority is 16 lines /
388 bytes / SHA-256
`a77b191454b98a36372762c07bc0e096ae32468e295f8cb7b64b85f8ad99bb60`;
the header is 21 lines / 476 bytes / SHA-256
`9f83c4c30e0b3302d5a1bbb4013c5a48ec0782f2bafdc123f529a1bf35c75eae`.
The strict executable denominator is four bodies: the constructor and
`style()` header inlines plus the two cpp definitions. The required non-null
retained `m_textStyle` identity is additional state context, not a fifth body.

The embedded Rust occurrence is a source-faithful representation. Construction
creates one distinct Component occurrence only for an option-bearing live
TextStyle, retains that exact style handle, attaches Component Super to the
Artboard root, and stores its then-current retained Text identity. `style()`'s
counterpart and update dispatch use the retained style handle without graph
rediscovery. Clone drops the source helper and reconstructs a fresh helper from
the cloned Style and clone-owned current Text; a live source parent write does
not retarget the source helper.

Dependency construction is exact and causal. At the TextStyle's authored slot
it inserts root -> helper and then helper -> the Style's current retained Text,
before the Style's own parent edge and before later authored root children.
The final dependent and sorted orders are observed on the real occurrence.
Imported `TextVariationHelperArtboard` and `TextVariationHelperText` edges are
explicitly skipped, so neither source nor clone can be overwritten by the
static projection.

The update owner ignores every dirt mask and always enters the accepted
two-part TextStyle variable-font update path. It computes the replacement from
the retained Style's current option state on each call, publishes the cache
only when the base font is available, and otherwise preserves the old cache.
No helper code calls a dirt API. Real `WorldTransform` and `FILTHY` scheduling
both replace the retained cache and leave the helper clean; the transform-only
retained-draw test confirms that the helper adds no Text reshape or render-path
replacement.

The nominal helper Rust file now owns only this pair. `StyledTextGlyph` and the
two general shaping helpers were moved unchanged to `text/styled_text.rs` and
included at the same lexical position, preserving visibility, call sites, and
behavior. This is an ownership split, not a replacement algorithm.

No pinned upstream test source mentions `TextVariationHelper` or
`text_variation_helper`, so literal consumer topology is **0 pass / 0 red / 0
adapted / 0 pending**. An independent recursive graph scan reproduced 378
readable fixtures plus one unrelated unreadable fixture, 133 option-bearing
files, 1,039 axes, and 572 projected embedded helpers. These fixtures are an
incidental impact surface, not literal consumers.

All five fully-qualified focused tests passed, one test each:

- exact helper identity/dependency insertion order;
- clone relinking to clone-owned Text;
- live feature update under `WorldTransform` and `FILTHY` masks;
- TextStyle helper/cache lifecycle;
- transform-only retained render-path preservation.

Candidate-range `git diff --check` passed, the delta is contained to the five
declared paths, and all 17 pre-existing user-dirty paths remained unstaged and
outside this receipt.
