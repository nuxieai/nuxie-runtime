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
