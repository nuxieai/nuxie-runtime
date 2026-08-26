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

## Empty-shape production-correction review of `5e56bf5f2`

Verdict: **RESIDUAL REJECTION — the retained draw owner misclassifies one
no-run binding direction**

The new `has_styled_text` predicate matches pinned `makeStyled` for this branch:
it requires a nonempty run, a non-null style, and a live font. The clean slice
owners consequently publish zero bounds before `effectiveSizing`, controlled
box, clipping, topology, or render-data work, and they do not retain layout
scale types. The fixed fallback also correctly distinguishes a genuine no-run
graph and a valid supported empty shape from an unsupported nonempty graph,
whose prior fallback behavior remains unchanged. The focused 3x3 matrix uses
those production owners, and the existing nonempty Taffy measure/controlled
bounds test remains green.

The actual retained rebuild is not yet exact. At
`draw.rs:17772::runtime_build_text_draw_frame`, the no-authored-run guard at
line 17795 treats every matching `Text.textRunListSource` data bind as a live
run source. It does not apply `data_bind_flags_apply_source_to_target`, unlike
both `text.rs:504::static_fixed_text_constraint_bounds` and
`StaticTextSlice::from_graph`. A Text with no authored runs and a
target-to-source-only list bind therefore bypasses the retained owner's empty
early branch; topology construction then rejects the same bind as not supplying
runs. Pinned `updateList` receives no source-to-target list in this case, so
`makeStyled` is empty and `buildRenderStyles` must publish zero bounds and
return successfully.

Narrow correction request:

1. Make the retained draw owner's run-list-source predicate use the same
   source-to-target direction rule as the two accepted Text owners.
2. Add focused retained-owner evidence for no authored run plus a
   target-to-source-only `textRunListSource`: rebuild succeeds, retains zero
   bounds, emits no commands/replay/clip, and does not retain controlled scale
   types. Keep the existing no-bind matrix and supported-empty evidence.
3. Freeze supported nonempty Taffy behavior, all other row 17 residuals, and
   the **4 pass / 3 executable expected-red / 11 pending** consumer topology.

Checks: the new empty-shape matrix and the existing nonempty Taffy test each
pass independently (one passed, zero failed, zero ignored); the candidate
delta passes `git diff --check`; production scope is `text.rs` only; and the
consumer topology remains 4/3/11. This review changed documentation only.

## Narrow retained empty-shape rereview of `8d7cabedb`

Verdict: **ACCEPTED**

The retained owner at `draw.rs:17772::runtime_build_text_draw_frame` now counts
`Text.textRunListSource` only when
`data_bind_flags_apply_source_to_target(data_bind.flags)` is true. That is the
same ownership predicate already used by `StaticTextSlice::from_graph` and
`static_fixed_text_constraint_bounds`, and it matches the pinned fact that a
target-to-source-only bind cannot supply `updateList` runs. The `draw.rs` delta
is exactly this one added predicate line.

The focused production test at
`text.rs:6518::target_to_source_only_run_list_bind_keeps_retained_empty_text`
uses no authored run plus a direction-to-source-only bind. It drives the real
update pass and retained draw command owner, then proves zero Text bounds, an
empty Text `shape_paints` list, no Text-local clip, unchanged authored sizing,
and no `drawPath` replay. Checking the Text dispatch directly keeps root-level
clip behavior separate from the Text assertion.

The focused test passes (one passed, zero failed, zero ignored), the candidate
delta passes `git diff --check`, and the `draw.rs` hunk is one insertion. The
row 17 residual wording remains honest, and the complete consumer section is
unchanged at **4 pass, 3 executable expected-red, 11 pending**. Pre-existing
user changes remain unstaged; this rereview changed documentation only.

## Text rows 24/25 production-correction review of `2c6ca226e`

Verdict: **RESIDUAL REJECTION — one pinned range callback is not dispatched**

The core correction is source-shaped. `RuntimeTextState` owns its range maps
per occurrence and `clone_for_occurrence` returns a cold state. The real
`StaticTextModifierRange::range_units` consumer reads and fills that cache;
empty maps retain the pinned recompute behavior. `markShapeDirty(bool)` walks
direct groups and their ranges in authored child order, adds Text Path first,
clears each group's maps before adding its TextCoverage dirt, then performs the
named Rust revision/bounds bookkeeping before WorldTransform and optional
layout publication. The distinct `modifierShapeDirty` owner adds Text Path
only. `unitsValue` now takes `rangeTypeChanged`, while `typeValue` takes the
ordinary `rangeChanged` route, with Text dirt before group TextCoverage.

One executable callback remains missing. Pinned
`TextModifierRange::clampChanged` calls `TextModifierGroup::rangeChanged`, but
Rust `ArtboardInstance::apply_bool_property_changed` has no TextModifierRange
owner at all. A live `TextModifierRange.clamp` write therefore updates the bool
and generic notifications without publishing the owning Text's Path/Paint dirt
or the group's TextCoverage dirt. This also makes row 25's claim that
shape-modifier-backed range changes route through the Path-only owner too
broad. Separately, row 25 must say **exact enabled-path candidate**: the pinned
disabled body at line 1427 is still absent/red, as required by the accepted
inventory convention.

Narrow correction request:

1. Route only `TextModifierRange.clamp` bool writes through the same ordinary
   `rangeChanged` owner as pinned `clampChanged`, and wire that owner into the
   bool setter callback chain.
2. Add real callback evidence for clamp with both a shape-modifier group
   (Text Path only, then group TextCoverage) and a paint-only group (Text Paint
   only, then group TextCoverage). Retained range maps must remain populated.
3. Extend the existing focused evidence to observe the authored multi-group /
   multi-range clear-and-dirt order rather than only collapsed final dirt bits,
   without duplicating the production traversal in the test.
4. Keep Rust revision/bounds bookkeeping named as an adaptation, keep the
   disabled path red, and freeze the **4 pass / 3 executable expected-red / 11
   pending** consumer topology.

Checks: the candidate's focused owner test passes (one passed, zero failed,
zero ignored), the delta passes `git diff --check`, and the complete consumer
section is unchanged at 4/3/11. This review changed documentation only.

## Narrow Text rows 24/25 residual rereview of `9e8b9c0e9`

Verdict: **ACCEPTED**

`ArtboardInstance::apply_bool_property_changed` now routes only
`TextModifierRange.clamp` to
`text_modifier_group_bool_property_changed`, which invokes the same real
`range_changed` owner as the pinned `clampChanged` callback. A group with a
shape modifier publishes Text Path but not Paint; a paint-only group publishes
Text Paint but not Path; both publish their own TextCoverage after the Text
dirt and preserve every retained range map.

The focused owner evidence now builds two authored groups and four ranges.
Test-only trace hooks surround the existing production traversal and record
each real range clear and group-publication boundary; they do not compute or
substitute the traversal. The observed stream is both ranges of the first
group, that group's publication, both ranges of the second group, then that
group's publication. The same test separately exercises both clamp routes and
verifies retained map count remains four.

The focused test passes (one passed, zero failed, zero ignored), the candidate
delta passes `git diff --check`, and its scope is limited to the bool callback,
test-only observation state, expanded owner evidence, and the source receipt.
Row 25 now says **exact enabled-path candidate; disabled path missing**, keeping
the pinned disabled body red. The complete consumer section is unchanged at
**4 pass, 3 executable expected-red, 11 pending**. This rereview changed
documentation only; pre-existing user dirt remains unstaged.

## Text row 10 `controlSize` production-correction review of `b54bedf4a`

Verdict: **RESIDUAL REJECTION — direct LayoutComponent animation omits the
Text callback**

The occurrence owner itself is source-shaped. `RuntimeTextState` clones cold,
compares all five fields (including the C++-equivalent floating-point `!=`
behavior), publishes the complete tuple before calling the existing
`markShapeDirty(false)` owner, and makes an identical call inert. The callback
adds Path/range-clear/group-coverage/World dirt without layout dirt. The real
direct-child solve and LayoutParticipant solve/animated-advance seams use the
owner; participant scale values come from its inherited sizing properties,
direction resolves from the owning layout, and retained constraints and
`effectiveSizing` consume the tuple. The focused owner and direct-child tests
both pass. The disabled body remains honestly red, and the consumer topology
remains **4 pass, 3 executable expected-red, 11 pending**.

One enabled caller is still absent. Pinned
`LayoutComponent::applyInterpolation` writes each changed interpolated layout
and calls `propagateSize()` when its width or height changed, and calls it again
after writing the terminal layout (`layout_component.cpp:1384-1407,1422-1432`).
Rust `RuntimeLayoutComponentState::advance_interpolation` reports those exact
`size_changed`/completion transitions, but
`ArtboardInstance::advance_layout_component_entry` only dirties controlled
parametric paths and forwards scripted layout size; it never invokes
`text_owner::control_size` for a direct non-participating Text child
(`components.rs:1449-1520`; `artboard.rs:6782-6825`). Because
`runtime_text_layout_constraint` prefers the retained tuple, the Text keeps its
pre-animation width and height while its owning LayoutComponent visibly
interpolates. The candidate therefore cannot classify row 10's enabled path as
exact.

Narrow correction request:

1. At the existing direct `LayoutComponent` animation boundary, propagate the
   current retained layout width/height plus the component style's two scale
   enums and actual direction to the same eligible direct, non-participating
   Text children as solve-time propagation. Preserve upstream intermediate
   size-change and terminal-call semantics; do not route through a map-derived
   constraint or duplicate the five-field algorithm.
2. Add real-caller evidence that advances one animated direct LayoutComponent
   through an intermediate size and completion and observes the Text's retained
   tuple and no-layout shape dirt. Keep the accepted owner, participant paths,
   disabled red, and **4/3/11** consumer topology frozen.

Checks: both focused candidate tests pass (one passed each, zero failed/ignored)
and the candidate delta passes `git diff --check`. Its committed `draw.rs`
changes are confined to the intended control-size seams/evidence. The separate
pre-existing formatting hunk in `draw.rs` remains unstaged, along with all
other user dirt. This review changed documentation only.

## Narrow Text row 10 animation residual rereview of `5a1173773`

Verdict: **ACCEPTED**

`ArtboardInstance::advance_layout_component_entry` now invokes the same
`propagate_runtime_layout_text_control_size` owner after
`RuntimeLayoutComponentState::advance_interpolation` has written its current
layout. Its gate matches the pinned split: an intermediate write propagates
only when width or height changed, while the terminal changed layout always
propagates. The shared owner reads the retained current width/height, the
LayoutComponent style's width/height scale enums, and its actual inherited
direction, then visits only direct, non-participating Text children before
calling the already accepted five-field `text_owner::control_size`. Solve-time
propagation now uses that same owner. The separate LayoutParticipant solve and
advance path is unchanged.

The focused test drives the real advancing-component dispatcher rather than a
test-local algorithm. It observes the retained tuple and preferred retained
constraint at the intermediate **150 x 70** write, then the terminal **200 x
90** write, with Text Path and WorldTransform dirt at both boundaries. The new
focused test, the retained-owner test, and the direct-child/Solo boundary test
each pass (one passed, zero failed or ignored).

The correction commit changes only `artboard.rs`, `draw.rs`, and the Text source
receipt, and passes `git diff --check`. The index contains no partially staged
paths; the pre-existing `draw.rs` formatting hunk and all other user dirt remain
unstaged. Row 10 still names the disabled no-op body as missing/red, and the
consumer topology remains **4 pass, 3 executable expected-red, 11 pending**.
This rereview changed documentation only.

## Text row 20 color-glyph production-correction review of `fe86937c2`

Verdict: **ACCEPTED AS THE NAMED ADAPTATION**

The retained draw owner now matches the pinned `drawColorGlyph` stream. An
empty layer vector returns before renderer state changes. A nonempty vector
performs the outer save, applies `shapeWorld * glyphTransform`, visits layers
in retained authored order, and restores once after the loop. Every vector
layer — including an empty path — creates a fresh `make_render_path` with
`NonZero`, creates a fresh paint, sets fill style and opacity-modulated color,
and draws it on every invocation. The former `(glyph, layer)` vector-path cache
is absent from both the backend resources and all call sites.

The image branch remains source-shaped: its cache key contains both retained
font identity and glyph ID, failed decodes remain cached as null, successful
images use the nested save, bearing/extent transform, linear-clamp/src-over
draw with glyph opacity, and nested restore. The outer restore remains balanced
for empty vector paths and decode failure because neither branch exits the
layer loop. Rust's pointer-derived font identity and Skrifa gradient extraction
and fallback-color representation remain explicitly named adaptations; this
correction does not overclaim them as literal.

The focused evidence invokes the production owner twice with the real
`RecordingFactory` and renderer. It proves zero layers emit no operation, two
vector layers allocate four distinct paths and paints and produce four draws,
the bitmap decodes once but draws twice, and the complete outer/vector/nested
image operation order repeats correctly. It does not reconstruct the draw
algorithm or inspect a proxy cache. The test passes (one passed, zero failed or
ignored), and the candidate delta passes `git diff --check`.

The correction commit changes only `draw.rs` and the Text source receipt. Its
committed `draw.rs` hunks are limited to removing the vector cache, correcting
the production stream, and replacing its focused evidence; the pre-existing
formatting hunk remains unstaged with all other user dirt. Consumer topology is
unchanged at **4 pass, 3 executable expected-red, 11 pending**, and the receipt
continues to state the residual adaptations honestly. This review changed
documentation only.

## Text row 39 `onDirty` production-correction review of `afcf1819a`

Verdict: **RESIDUAL REJECTION — focused evidence bypasses the Text owner and
reentrant dirt dispatcher**

The production mapping is source-shaped on inspection. `add_component_dirt`
publishes the accumulated mask before `dispatch_component_on_dirty`; Text then
visits immutable occurrence topology in authored modifier order. Only a group
with a retained follow-path modifier re-adds Path, and that recursive add sees
the accumulated WorldTransform-plus-Path mask before the outer modifier loop
resumes. Path or Paint derives distinct current styles from the retained replay
in first draw-command order and invalidates concrete occurrence-owned paint
effect chains. The former later broad scan of every imported TextStylePaint is
removed. Combined masks retain modifier-before-style order, and the accepted
Text draw owner clones with retained frame/backend state cold while immutable
import topology remains occurrence-addressed. The disabled body remains red
and consumer topology remains **4 pass, 3 executable expected-red, 11
pending**.

The claimed evidence does not execute any of those integration properties.
`cxx_text_on_dirty_visits_current_modifiers_then_current_style_effects`
constructs `RuntimeTextOnDirtyTargets` directly with hard-coded modifier and
style vectors, calls `visit` directly, and manually invokes the effect owner.
It never creates a Text occurrence, builds current retained draw commands,
calls `ArtboardInstance::add_dirt`, triggers the follow-path recursive add, or
proves that an imported-but-not-current style stays clean. It therefore proves
the small visitor and concrete effect primitive, but is a proxy for the exact
owner boundary that row 39 claims.

Narrow correction request:

1. Replace or extend the focused case with a real RuntimeFile/Graph and
   ArtboardInstance Text occurrence. Materialize its retained frame, then drive
   WorldTransform, Path, Paint, and the combined mask through `add_dirt`, not a
   manually constructed target list.
2. Observe the existing production dispatch (a read-only test trace is fine)
   to prove authored current-group order, the WorldTransform-follow-path
   recursive accumulated-mask stream, modifier-before-style combined order,
   and exactly one effect invalidation per current style at the reentrant
   boundary. Do not reproduce target selection or traversal in the test.
3. Include at least two current retained styles in draw-command order and one
   imported unused style with concrete effects; prove only the two current
   occurrence owners become dirty. Preserve the production correction,
   clone/reset ownership, disabled red, and **4/3/11** topology unless the real
   evidence falsifies them.

Checks: the current proxy-focused test passes (one passed, zero failed or
ignored), the existing stale-retained-frame dirty-draw containment test passes
(one passed, zero failed or ignored), and the candidate delta passes
`git diff --check`. The candidate changes only `artboard.rs`, `draw.rs`,
`text.rs`, and the source receipt. The separate pre-existing `draw.rs`
formatting hunk and all other user dirt remain unstaged. This review changed
documentation only.

## Final narrow Text row 39 evidence rereview of `f47e8735c`

Verdict: **ACCEPTED**

The proxy fixture is gone. The focused case imports a real RuntimeFile, builds
its GraphFile and ArtboardInstance, runs the update owner to materialize the
Text's retained frame, and then drives WorldTransform, Path, Paint, and the
combined mask only through `ArtboardInstance::add_dirt`. Two run styles become
the retained current style list in draw-command order; a third imported style
has a real stroke effect but no current run.

The WorldTransform case reaches the live follow-path group and records the
pinned recursive stream: the outer authored group traversal re-enters after
Path is accumulated, the nested traversal observes WorldTransform|Path and
invalidates current styles 2 then 6 once, and the outer traversal then resumes.
The standalone Path and Paint cases each visit those same two current styles.
The combined case visits all authored groups before styles 2 and 6 without an
extra recursive invalidation. Concrete effect state becomes dirty for styles 2
and 6 while unused imported style 10 remains clean in every case.

The `cfg(test)` additions only append the production callback's already chosen
mask and action to occurrence-owned trace storage; they neither select targets
nor replace traversal, reentrancy, or effect invalidation. Clone/reset remains
cold through the existing `RuntimeTextDrawOwner::default` clone path. The
production mapping is otherwise unchanged.

The real-owner focused test and stale-retained-frame dirty-draw containment
test both pass (one passed each, zero failed or ignored), and the candidate
delta passes `git diff --check`. Correction scope is only `artboard.rs`,
`draw.rs`, and the source receipt. The pre-existing `draw.rs` formatting hunk
and all other user dirt remain unstaged. Disabled row 39 remains red and
consumer topology remains **4 pass, 3 executable expected-red, 11 pending**.
This rereview changed documentation only.

## Independent Text row 43 hit-test review of `2efeabe65`

Verdict: **ACCEPTED AS EXACT OBSERVABLE BEHAVIOR WITH THE NAMED EDITOR
ADAPTATION**

The complete pinned enabled body returns null both before and after its
render-opacity branch, and the disabled body also returns null. The only
general drawable call chain is `Artboard::hitTest` (recursing through
`NestedArtboard::hitTest`), which invokes each drawable's virtual `hitTest` and
returns the first non-null owner. Rust's corresponding `geometry_hit_test`
creates one caller-space `HitTestArea`, carries it through nested artboards,
selects only the first traversal hit, and makes the Text catalogue branch
unreachable whenever that pinned area is present. Both runtime and public
facade `hit_test` methods delegate to this owner, so visible and zero-opacity
Text cannot escape through the general route.

The Text-inclusive `geometry_hit_test_paths`, segment, visible-catalogue, and
retained-catalogue methods do not masquerade as that virtual call. They are
separately named all-hit/editor queries covered by the pre-existing adaptation
in `runtime-source-certification/hit-test.md`; visible queries still obey
opacity and the retained catalogue intentionally does not. State-machine
targeting is also not a leak: pinned `HitExpandable` calls the distinct
inherited `Component::hitTestPoint`, while `TextValueRun` owns its separate
high-fidelity override. Rust routes those listeners through the corresponding
component owner rather than through the Artboard drawable query.

The focused case imports a real RuntimeFile, constructs its GraphFile and
ArtboardInstance, materializes a real Text occurrence, and observes the actual
owners. It proves the general route is empty at visible and zero opacity, the
named editor path/segment catalogues include visible Text, the retained
catalogue preserves zero-opacity Text, and the component route remains
hittable. It passes (one passed, zero failed or ignored), and the candidate
delta passes `git diff --check`. The candidate changes only this real-owner
test and the Text source receipt; no production behavior or topology moved.
The pre-existing `draw.rs` formatting hunk and all other user dirt remain
unstaged. Consumer topology remains **4 pass, 3 executable expected-red, 11
pending**. This review changed documentation only.

## Independent StyledText production-correction review of `721a38cc3`

Verdict: **REJECTED — the retained projection still changes defined append,
style-ID, and first-run shaping behavior**

The new inclusion bit correctly preserves the important leading-NUL case when
the whole source is valid UTF-8: a nonempty source with a live font retains a
StyledText run even when its decoded prefix and `unicharCount` are both empty.
The ordinary embedded-NUL, skipped null-style/font-null/empty-run offset, and
fresh immutable rebuild witnesses pass. The separately owned modifier-group
discrepancy is also stated honestly: Rust still layers successive group
variations, while pinned `applyShapeModifiers` restarts each replacement from
the original style and swaps the run vector.

Four narrow blockers remain:

1. `resolved_runs` and `resolved_dynamic_runs` validate the complete byte
   vector as UTF-8 before applying the C-string prefix. For source bytes
   `[0x00, 0xff]`, pinned `text.empty()` is false, `append` stops before the
   invalid suffix, appends a zero-count run, and therefore leaves
   `StyledText::empty()` false. Rust rejects the full value before constructing
   that run. Truncate at the first NUL in byte space before validating the
   StyledText prefix; retain inclusion from the original source's nonempty
   state. Invalid bytes before the first NUL may remain an explicitly approved
   Rust-safety adaptation, but an unread suffix is not unchecked upstream
   decoding.
2. `StaticResolvedRun` does not retain pinned `uint16_t styleId` at all.
   Direct `style_local` happens to recover ordinary skipped-run gaps, but it
   bypasses the all-runs index owner and the defined unsigned wrap after
   65,535. Retain the pre-increment all-runs index (including every skipped
   entry) as a wrapping `u16`, and make the corresponding paint/style lookup
   consume that ID. Add focused skipped-prefix and wrap-boundary evidence; do
   not use the modifier correction to hide the already acknowledged
   `text_modifier_group.cpp` red.
3. The real topology, bounds, measure, clip, and fit paths still seed their
   common shaper from `self.base_style()`, the first TextStylePaint, rather
   than `styledText.runs()[0].font`, the first run actually appended by
   `makeStyled`. An unrelated or font-null first paint followed by a valid run
   therefore returns no topology or uses the wrong shaper/metrics in Rust,
   while pinned code skips the invalid run and shapes with the later included
   run. Resolve the common shaping owner from the first participating retained
   run and cover a font-null/unrelated first-paint plus valid later-run case
   through the real render/measure consumers.
4. The clear/rebuild witness calls `StaticTextSlice::render_topology` directly.
   It proves a fresh local value but not that the retained
   `RuntimeTextDrawOwner::shaped_topology_or_build` invalidates the old
   publication before rebuilding. Exercise the occurrence owner through a
   real text write/update and observe that the prior Unicode/run topology is
   absent, including the leading-NUL zero-run state.

The three focused candidate tests pass individually (one passed each, zero
failed or ignored), and the candidate delta passes `git diff --check`.
Candidate scope is only `text.rs` and the Text source receipt. The pre-existing
`draw.rs` formatting hunk and all other user dirt remain unstaged; the global
working-tree check still reports the user's existing trailing whitespace in
`tools/webgpu-renderer-replay/build.sh`. Consumer topology remains **4 pass, 3
executable expected-red, 11 pending**. This review changed documentation only.
