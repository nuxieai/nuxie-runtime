# Independent review: `TextStylePaint`

Candidate `d9f40bc62` is **rejected** under
`docs/runtime-exact-parity-workflow-correction.md`.

I independently read the complete pinned pair at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`, plus the directly required
generated `TextStylePaintBase`, `ShapePaintContainer`, and `ShapePaint` draw /
registration context. The cpp authority is 134 lines / 4,006 bytes / SHA-256
`1b9fb64d3440012c7640b68dc9a2ba306f32e7b27c3465f02e597e2c324794d9`;
the primary header is 38 lines / 1,200 bytes / SHA-256
`c1cb701650a54203d0f0448ad59d96a5c481a43f96b5a02ac3e091aeaed51ed2`.
The strict denominator of eight cpp bodies plus three executable header inlines
is correct.

The path owner itself is sound within the stated dependent winding gap.
`m_hasContents` changes before the exact `opacity > 0` predicate, so NaN,
signed zero, and negative values are rejected while `+inf` is accepted; exact
positive keys merge and sort ascending. Rewind clears aggregate/buckets/state
without clearing the backend pool. Aggregate geometry feeds effects and inner
feather while bucket geometry remains the clip/draw source. Pool indices reset
per child, pooled paints are configured unconditionally, and clone backend/path
state starts cold. The separately owned `addPathClockwise` normalization remains
an honest dependent red rather than a hidden local substitute.

Two finite source discrepancies prevent acceptance:

1. **`ShapePaint::blendMode` is not preserved on every draw.** Pinned
   `TextStylePaint::draw` calls `shapePaint->blendMode(Text::blendMode())`
   before the opaque bucket and pool work for each drawable child. The called
   owner (`shape_paint.cpp:60-71`) uses the Text value only when the child's
   live `blendModeValue == 127`; otherwise it writes the child's explicit
   blend. Candidate `RuntimeTextReplay::EmptyStyle` instead writes the parent
   Text blend directly to every retained child paint, overwriting an explicit
   child blend. The ordinary opaque-bucket path also consumes
   `authored_paint` without replaying this per-draw call; only temporary
   nonopaque paints pass through `runtime_configure_text_pooled_paint` and
   receive the command blend. Consequently a live parent or child blend write
   can leave an opaque retained paint stale even though the rebuilt command
   contains the correct value.

2. **The claimed occurrence `m_ShapePaints` list registers a paint with no
   mutator.** Pinned `ShapePaint::onAddedClean` appends only when
   `m_PaintMutator != nullptr` (`shape_paint.cpp:20-25`).
   `build_component_occurrence_relations` appends every is-a `ShapePaint`
   direct child of `TextStylePaint` unconditionally. The later renderer
   reconstruction happens to discard a local absent from the graph's
   mutator-backed paint catalogue, but that makes the advertised
   `RuntimeTextStylePaintState` differ from its source owner and relies on a
   downstream projection to repair it.

The narrow correction is to model one actual per-child draw entry: invoke the
live `ShapePaint::blendMode(parent)` equivalent exactly once before that
child's opaque/nonopaque work (and on rejected-only styles), honoring explicit
child values and current parent values. Evidence must cover an explicit child
blend on `EmptyStyle`, a live inherited-parent blend on an opaque bucket, and a
live explicit-child blend on an opaque bucket, using the retained backend and
observing call order. Registration must also require the retained equivalent
of non-null `m_PaintMutator`; a no-mutator Fill under TextStylePaint must remain
linked as a Component but absent from both source and clone paint membership.

All five declared focused tests passed, as did
`cargo check -p nuxie-runtime --lib` and candidate-range `git diff --check`;
they do not exercise either rejected condition. Literal owner topology remains
**0 pass / 0 red / 0 adapted / 1 pending**: `text_modifier_test.cpp` ordinal 2
is still honestly pending, and the clockwise-winding dependent red is not
promoted. Independent fixture scanning reproduced 378 readable / one unrelated
unreadable fixture, 137 files and 696 `TextStylePaint` objects. Literal source
reference reconciliation reproduced 135 referenced files plus the two
unreferenced files `library.riv` and root `scroll_snap.riv`.

The candidate range contains exactly the declared seven paths. The post-commit
unstaged `draw.rs` delta is only the original formatting hunk in
`upstream_flagged_component_list_joins_layout_through_a_group`; all 17
pre-existing user-dirty paths remain outside this review commit.
