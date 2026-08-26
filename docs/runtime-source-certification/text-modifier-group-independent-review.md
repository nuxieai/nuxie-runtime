# Independent review: `TextModifierGroup`

Candidate `1babcba35699d24d9228123263de80fec635ff5b` is **rejected** under
`docs/runtime-exact-parity-workflow-correction.md`.

I independently read the complete pinned
`src/text/text_modifier_group.cpp` and
`include/rive/text/text_modifier_group.hpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The denominator is confirmed as
28 out-of-line bodies plus ten executable header inlines, 38 units total.
The scale-lane contraction, inverted-opacity contraction, successive nonzero
shape-group replacement from the authored style, and variation-axis callback
corrections are consistent with their pinned owners. Rows 13 and 28 are also
honest dependent-owner reds: failed-inverse retained follow-path state belongs
to the follow-path owner, while line-unit `needsShape` and its pre-shape range
stream belong to the range/Text owners. The 4 pass / 2 red / 7 pending
13-consumer accounting and broader Text 4 / 3 / 11 topology therefore do not
move in this candidate.

## Blocking finding

Row 2 and correction 5 overclaim the lifecycle boundary. Pinned
`TextModifierGroup::onAddedDirty` returns `MissingObject` during object
addition/import when the direct parent is not a `Text`, after the superclass
call and before the artboard becomes a usable runtime occurrence. Candidate
validation instead lives in `StaticTextSlice::from_graph`. The focused test
first constructs a valid `ArtboardInstance`, mutates a cloned `ArtboardGraph`,
and then calls that late Text projection directly. Consequently
`ArtboardInstance::from_graph` still accepts the malformed graph; rejection
only happens if a later Text query/render path attempts to build a
`StaticTextSlice`. This is neither import-boundary evidence nor equivalent
failure timing, and it does not establish the pinned superclass-first contract.

Narrow correction: validate every `TextModifierGroup` direct parent at the
actual `ArtboardInstance` construction/import boundary, after the applicable
base/component validation, and replace the projection-only test with evidence
that malformed runtime construction itself fails. Keep a projection guard only
as defense in depth, not as the mapped owner. Update row 2, correction 5, and
their locators/wording; leave the other 37 units and both dependent reds frozen.

Candidate-range `git diff --check` was clean. Focused test compilation
succeeded; the attempted exact-name invocations selected zero tests because
the unit tests are module-qualified, so they are not claimed as execution
evidence here. Pre-existing user worktree changes remained unstaged.
