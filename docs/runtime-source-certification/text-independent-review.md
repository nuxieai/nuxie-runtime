# Text source-pair independent review

Verdict: **REJECTED — consumer accounting and several concrete source
discrepancies require narrow correction**

Reviewed candidate: `626e68e91fcfd90babf4a5bcc0123c1b748eda71`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

The denominator is complete: 53 logical `.cpp` authority units, 79 physical
bodies including 26 text-disabled alternates, 29 executable header methods,
and eight meaningful header defaults. The pinned `text.cpp`, `text.hpp`, and
`text_test.cpp` hashes match the candidate. The major reported discrepancies
in `effectiveSizing`, paragraph spacing, dynamic empty runs, listener timing,
`controlSize`, retained hit geometry, and the absent disabled-text
configuration are confirmed.

## Required correction

1. The 16 remaining `text_test.cpp` rows are not all pending. Existing literal
   Silver entries make case 14 (`zero_width_space_line_break`) executable and
   exact, and cases 15-18 executable expected-red:
   `word_joiner_test` at frame 2/op 262 `transform.ty`,
   `fit_font_size_test` at frame 2/op 199 `makeRenderPath`,
   `text_vertical_trim_test` at frame 3/op 220 `rewind`, and
   `layout_text_match` at frame 0/op 61 `save`. Case 15 is a ten-rendered-frame
   stream, not nine. The correct 18-case consumer topology is **3 pass, 4
   expected-red, 11 pending**. Distinct Wave C7 wrappers/locators may still be
   needed, but callable literal evidence cannot be described as missing-owner
   pending work.
2. Row 2 omits a second dynamic-list discrepancy. When `textContent` exists
   but `textStyle` does not, pinned `createPropertyListener` leaves the new
   `TextValueRun::m_style` null and `makeStyled` skips it. Rust converts the
   absent style to an empty name and `resolved_runs` falls back to the first
   style, so it renders a run that pinned C++ omits.
3. Row 17 must record the empty-shape bounds difference. Pinned
   `buildRenderStyles` sets `m_bounds` to zero and returns whenever `m_shape`
   is empty. Rust's `unshaped_local_bounds` and
   `static_fixed_text_constraint_bounds` publish the controlled layout box for
   empty layout-controlled Text. This is an incorrect retained observable, not
   merely an unproven packed phase.
4. Row 20 has a known factory-stream difference. Pinned `drawColorGlyph`
   creates a new render path and paint for every non-image color layer on every
   draw. Rust retains `color_paths` by `(glyph_index, layer_index)` and reuses
   them across draws. Classify that path as incorrect rather than an adapted
   candidate. H6 must also be adapted, not exact: C++ keys its image cache by
   `Font*` identity plus glyph ID, while Rust uses the font-byte allocation
   pointer plus glyph ID.
5. Row 53 already states that Rust errors when no `TextStylePaint` child
   exists, but misclassifies the behavior as adapted. Pinned
   `buildTextStylePaints` legitimately retains an empty list and later skips
   null-style runs; the Rust topology error is an incorrect/incomplete owner.
6. Correct the disabled-body cross-reference: the stated ranges include row
   41, whose `measure` has no disabled body. The 26 alternates are rows 8-10,
   18-19, 21-32, 38-40, and 42-47.
7. Supply durable file/line/symbol locators for every claimed concrete owner or
   say explicitly that no owner exists. Examples currently lacking an exact
   locator include rows 1, 8, 10, 20, 24, 33-34, and 38-39, plus several H
   rows. A module, field description, or generic “reconstruction” is not the
   exact symbol locator promised by the candidate.

## Checks

- Complete pinned pair and every cited concrete Rust owner were reread; source
  hashes and candidate `git diff --check` pass.
- The existing focused Silver backfill test replays and confirms the exact
  `layout_text_match` frozen difference; 1 passed, 0 failed, 0 ignored.
- Manifest fixture, action, status, provenance, and frozen-difference records
  for cases 14-18 were checked directly.
- No production code, test, or source-candidate document was changed by this
  review.

## Narrow correction rereview of `d4cf30f9d`

Verdict: **RESIDUAL REJECTION — six substantive corrections are accepted;
the exact-locator correction remains incomplete**

The denominator and pinned hashes are unchanged. The corrected source receipt
now honestly records the missing-style dynamic-run behavior, empty-shape
controlled bounds, per-draw color-glyph factory stream and adapted H6 key,
no-style topology error, and the 26 disabled-body row ranges. Its consumer
topology is also correct at **3 pass, 4 executable expected-red, 11 pending**,
including the ten-rendered-frame word-joiner stream.

Only the following locator cleanup remains:

1. Row 1 promises a definition-start locator but names
   `text/text.rs:15::RuntimeTextValueRunListener::new`; line 15 is the `impl`
   block and `new` starts at line 16. Use the actual definition line.
2. Row 39 still gives the generic range
   `draw.rs:10445-10509::RuntimeTextDrawOwner` instead of naming the individual
   nearest methods and their definition lines (or stating that no direct owner
   exists). Row 40 likewise retains `artboard.rs:9020-9078` and a bare
   `RuntimeTextDrawOwner`. These do not satisfy the receipt's durable
   file/line/symbol rule.
3. H29 names “retained topology text” without its source locator. Name
   `text/fully_shaped_text.rs:22::StaticShapedTextTopology::text`, as H1
   already does, or state that no retained source field is being claimed.

Checks: candidate delta is documentation-only and passes `git diff --check`;
all three pinned SHA-256 values match; corrected Silver manifest entries and
frozen differences for cases 14-18 were rechecked. Production and tests remain
untouched.

## Final locator-only rereview of `eb768b4c8`

Verdict: **ACCEPTED**

The finite residual list is closed. Row 1 now names the actual `new` definition
at `text/text.rs:16`; rows 39-40 name each nearest split Rust method at its
verified definition line and explicitly retain the absent single-owner and
missing-callback findings; H29 now names
`text/fully_shaped_text.rs:22::StaticShapedTextTopology::text`.

The candidate delta is documentation-only and passes `git diff --check`. The
denominator remains 53 logical/79 physical definitions, 29 executable header
methods, and eight defaults; the consumer topology remains **3 pass, 4
expected-red, 11 pending**; and all three pinned hashes are unchanged. No
production or test file was modified by this review.

## Production-correction review of `a45c75a78`

Verdict: **ACCEPTED**

The mixed-axis correction is one production algorithm shared by
`RuntimeTextLayoutConstraint` and retained `RuntimeTextState`; its complete
fixed/fill/hug matrix matches pinned `Text::effectiveSizing`. Width, height,
and overflow callbacks consume that retained result and preserve the pinned
dirty/no-dirty returns. The still-missing complete `controlSize` owner remains
the pre-existing row 10 gap and is not concealed by this acceptance.

Paragraph boundaries are retained explicitly: authored and empty paragraphs
end a paragraph, while soft-wrapped intermediate lines do not. Line metrics
therefore carry spacing only into the next paragraph; measurement publishes the
last line bottom (N-1 spaces), fit/fixed height adds the final space, auto bounds
subtract it exactly once before trim, and ellipsis uses the pinned rule that
only a chosen line after line zero replaces the full height. The same owners
feed bounds, fit-font-size probes, render transforms, and measure; there is no
parallel test-local layout algorithm.

Evidence independently checked:

- the sizing-matrix/callback and paragraph-spacing owner tests each pass
  independently (one passed, zero failed, zero ignored);
- the literal `fit_font_size_test` action stream is bind + initial
  advance/draw followed by six frame/trigger/advance/draw cycles, matching all
  seven pinned rendered frames, and its full SRIV comparison passes;
- only `fit_font_size_test` changes from diverges to exact in the manifest and
  generator; direct replay preserves the other three frozen differences:
  `word_joiner_test` frame 2/op 262, `text_vertical_trim_test` frame 3/op 220,
  and `layout_text_match` frame 0/op 61; and
- the resulting consumer topology is **4 pass, 3 executable expected-red, 11
  pending**. Candidate `git diff --check` passes.

The normal filtered CLI could not reach case selection because the unchanged
manifest has an unrelated `global_variables_test` validation defect. A
read-only direct `Execution` harness was used to replay the three retained reds
instead. One documentation-only cleanup remains: row 17's first two shifted
locators should now read `text.rs:2263::layout_from_shaped_topology` and
`text.rs:2543::render_data_from_layout`. This does not affect the accepted
production behavior. No production or test file was changed by this review.

## Dynamic-list production-correction review of `90726b99f`

Verdict: **RESIDUAL REJECTION — one insertion-point consumer still loses the
pinned styled-run owner**

The value projection itself is correct. Every valid list item remains in source
order; absent content becomes the concrete run's default empty text; absent
style remains null; a present unmatched style selects the first paint; and an
empty paint list is valid. Style is read before content, and the changed
`Option` projection reaches the artboard, semantic-label, topology, bounds,
shaping, glyph, and paint-order callers. The receipt also honestly leaves the
aggregate listener, initial-write/retention, synchronous dirt, font-null, and
literal all-runs `styleId` ownership as adaptations. No `text_test.cpp`
consumer moved: topology remains **4 pass, 3 executable expected-red, 11
pending**.

One affected production consumer is not exact. In
`text.rs:3158::StaticTextSlice::static_line_metrics` (fallback at line 3182), the empty-line lookup
finds the first run whose inclusive range touches the line insertion point,
then checks whether that run has a style. A retained null-style dynamic run has
zero StyledText length, so it can win that search at the same `char_start` as a
later participating run. The code then falls back to paint zero instead of the
later run's paint. For example, a missing-style row followed by a second-style
row beginning `"\nA"` gives the empty first paragraph base-style metrics. The
pinned `makeStyled` omits the null-style row entirely, so that paragraph remains
owned by the second-style StyledText run.

Narrow correction request:

1. In the empty-line insertion-point lookup, exclude runs that did not produce
   a StyledText run before choosing the owner (at minimum null-style and empty
   source runs), rather than allowing one to trigger the base-style fallback.
2. Add focused production-owner evidence with a null-style/empty row preceding
   a differently styled run whose leading newline makes line metrics observable.
   Assert the retained run/character offsets and the selected line metric style;
   do not duplicate the selection algorithm in the test.
3. Freeze all other production behavior, source classifications, residual gaps,
   and the 4/3/11 consumer topology.

Checks: both focused candidate owner tests pass independently (one passed, zero
failed, zero ignored each); the pinned `text.cpp` and `text.hpp` SHA-256 values
remain `a485332b...d346fb` and `10688904...cf0c8`; the candidate delta passes
`git diff --check`; and no manifest or consumer-test file changed. This review
changed documentation only.

## Narrow dynamic-list residual rereview of `291fb2933`

Verdict: **ACCEPTED**

The one rejected insertion-point path is corrected at
`text.rs:3158::StaticTextSlice::static_line_metrics`. The lookup preserves
all-runs order but now considers only nonempty styled participants and then
requires a live font before selecting the insertion-point style. Null-style,
empty-source, and font-null runs therefore cannot mask the later run that
pinned `makeStyled` actually emits; the existing base-style fallback remains
unchanged when no emitted run owns the insertion point.

The focused real-owner test at
`text.rs:5866::cxx_empty_line_metrics_ignore_runs_omitted_by_make_styled`
constructs the rejected sequence exactly: a null-style row, an empty first-style
row, and a second-style `"\nA"` row. It verifies the retained zero-length and
two-character offsets, invokes the production line-metrics owner, matches the
second-style-only control, and materially differs from the paint-zero control.
The prior dynamic-run test at line 5763 is byte-unchanged apart from its shifted
locator.

Both focused tests pass independently (one passed, zero failed, zero ignored
each), the candidate delta passes `git diff --check`, and only `text.rs` plus
the source receipt changed. The complete consumer section is unchanged at
**4 pass, 3 executable expected-red, 11 pending**. All previously recorded
listener/timing, literal `styleId`, and other source-pair adaptations remain in
place. This rereview changed documentation only.
