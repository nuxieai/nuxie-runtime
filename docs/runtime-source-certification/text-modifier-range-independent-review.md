# Independent review: `TextModifierRange`

Candidate `d197e1c67867a11b07f933c7ae726a3495621109` is **rejected** under
`docs/runtime-exact-parity-workflow-correction.md`.

I independently read the complete pinned
`src/text/text_modifier_range.cpp` and
`include/rive/text/text_modifier_range.hpp` at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The denominator is confirmed as
23 out-of-line bodies plus 14 executable header inlines, 37 units total. The
construction checks and frozen authored run local, Super child retention and
last-cubic selection, concrete sentinel RangeMap, unknown units/type/mode
switches, retained index state, ordered C++ NaN min/max/clamp operations,
falloff branches, and inert runId callback are consistent with their pinned
owners. The line-unit pre-shape routing and cross-Text run offset/length gaps
are honestly scoped dependent-owner reds rather than hidden pair corrections.

## Blocking findings

1. The claimed case-9 promotion and resulting topology are not present in the
   authoritative consumer ledger. In
   `docs/runtime-test-case-waves/wave-c7.json`,
   `tests/unit_tests/runtime/text_test.cpp#9` remains `pending` /
   `unverified` with empty evidence. Candidate `d197e1c67` changes neither that
   row nor its executable locator, while the source receipts claim 1 pass / 0
   red / 2 pending for this pair and 5 / 3 / 10 for broader Text. Until the
   ledger is mechanically updated with the literal test, the authoritative
   counts remain zero pass / three pending for these three consumers and
   broader Text remains 4 / 3 / 11.

2. The asserted real ligature pipeline lacks real-owner evidence. The focused
   `cxx_range_mapper_maps_words_and_converts_fractional_units` test directly
   injects synthetic `[3, 0, 0, 1]` counts into
   `character_range_units`. It does not shape a Text, build the unmodified
   glyph stream, pass `styled_glyph_lookup_counts` through
   `coverage_by_character`/`range_map`, or observe the resulting indivisible
   range. A separate pre-existing glyph-count test does not prove that these
   owners are connected in the live range path.

Narrow correction: mechanically promote Wave C7 case 9 to direct/pass with
the exact `upstream_range_mapper_words_body_is_ported` evidence locator and
reconcile both topology statements. Add one focused real Text occurrence whose
ligature is shaped by the production unmodified pass and whose actual range
map/coverage proves the full glyph-count handoff; any test-only observation
must be read-only and must not implement the mapping itself. Leave all 37
production classifications, cases 10/11, and the two dependent reds frozen.

Focused unit tests for word/sentinel math and construction/last-cubic behavior
passed. The existing `cpp_probe` case-9 command requires the `tools` feature,
so the invocation without that feature was a command-configuration failure,
not evidence of a semantic failure. Candidate-range `git diff --check` was
clean, and pre-existing user worktree changes remained unstaged.
