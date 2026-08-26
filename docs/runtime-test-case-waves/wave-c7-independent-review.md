# Wave C7 text/input independent adversarial review

Verdict: **REJECTED — four adapted rows omit causally observable shaping
authority**

Reviewed candidate: `fa9fc4841c9d9c202bc890adfd07d61325dc5a6d`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Frozen denominator: 58 cases: 20 `text_input_test.cpp`, 17
`raw_text_input_test.cpp`, 18 `text_test.cpp`, two
`text_modifier_test.cpp`, and one `nested_text_run_test.cpp`.

## Blocking finding

Raw-text-input cases 9, 11, 14, and 16 cannot be credited by the four
owner-local Rust tests currently mapped to them. In the pinned owner,
`RawTextInput::cursorHorizontal(..., CursorBoundary::character, ...)` calls
`ensureShape()` and then consults `m_shape.glyphLookup()` to choose the next
glyph boundary. `RawTextInput::backspace()` likewise uses the shaped glyph
lookup to choose the deleted cluster.

That shaping path is causally responsible for pinned observables in all four
rows:

- case 9 includes four character-boundary moves between its word-boundary
  actions;
- case 11 asserts that the cursor skips the interior codepoint of the shaped
  decomposed glyph;
- case 14 reaches and deletes that shaped multi-codepoint glyph; and
- case 16 performs six character-boundary moves whose endpoints feed the
  journal and replacement assertions.

The Rust tests never install the pinned font, build or retain
`TextInputGeometry`, or perform the corresponding live update. They instead
exercise `RawTextInput::cluster_start` / `cluster_end`, which infer clusters
from Unicode combining-mark categories. Matching the selected literals does
not make the omitted shaping owner inapplicable, and replacing a
font-dependent glyph lookup with a character-category algorithm is not a
Rust-ownership or allocator-safety adaptation. The ledger rationales are also
factually too broad: the upstream font is an `rcp<Font>`, not a raw pointer,
and shaping/update is read indirectly through every affected cursor or delete
result.

Correction is ledger/documentation-only unless exact live evidence is added.
Demote raw cases 9, 11, 14, and 16 to strict `pending` / `unverified` rows with
empty evidence and no adaptation or note, and set `ratchet.max_pending` to 52;
or replace each locator with distinct evidence that executes the pinned font
and live shaping/geometry owner while preserving the complete case stream.
Do not relabel the current category-based tests as another adaptation.

## Accepted semantic evidence

Six executable rows survive body-level review:

- raw case 1 is an exact `cxx-language-only` spelling adaptation over the
  retained `CursorPosition` fields and saturation operation;
- raw case 10 preserves every word/subword action and assertion. Its omitted
  shape refresh has no role in the pinned word classifier's result stream;
- raw case 15 preserves all four selection actions and all eight endpoint
  assertions. `selectWord()` is buffer/classification-owned in the pin;
- raw case 17 directly preserves both `clearSelection` calls, collapse check,
  and four endpoint assertions; and
- text cases 2 and 3 each load the exact fixture and query the live
  `ArtboardInstance`. Case 2 counts live instance slots. Case 3 uses the graph
  only to resolve the fixture-local id, then reads the asserted text bytes from
  the live Artboard property owner.

The corrected semantic topology is therefore **six pass (three direct, three
adapted), zero expected-red, and 52 pending**. Wave-level acceptance remains
**0/58** while the frozen ledger overclaims the four rejected rows.

## Pending and forbidden-evidence audit

The candidate's original 48 pending rows are strict and honest. Every row is
`unverified`, has empty evidence, and carries no note or adaptation. The
available nearby tests do not promote any of them:

- raw geometry, hit, bidi, measurement, vertical movement, and remaining
  cluster cases either lack the exact combined font/shaping/buffer owner or
  occur in parameterized/collapsed probes;
- text-input probes merge multiple upstream cases and omit or add fixture
  advances, actions, return checks, or assertions;
- remaining text probes collapse literal parameter streams, weaken exact
  assertions, or observe debug/model/string projections rather than the
  complete pinned live owner;
- modifier probes inspect static graph topology or only the repro structure,
  not the live modifier-group/paint/animation and SRIV streams; and
- the nested-text-run substitute defines get/set helpers whose first behavior
  is unconditional panic.

No `cpp_probe` aggregate, static graph modifier proxy, model/string proxy,
collapsed loop, manifest action runner, or unconditional panic is counted as
Wave C7 evidence. The two accepted text-query functions happen to live in
`cpp_probe.rs`, but they are distinct one-case tests with live Artboard
observables, not aggregate probe evidence.

## Gates

- Focused non-incremental execution: eight `wave_c7_` unit tests and the two
  distinct live-Artboard text-query tests passed; zero failed or ignored.
- Strict shard identity, ordinal, source line, exact name, structured
  adaptation, outcome, and locator validation accepts the ledger's declared
  shape mechanically: 58/58 rows; direct 3, adapted 7, pending 48; pass 10,
  unverified 48. The semantic review above rejects four of those mechanically
  valid adaptations.
- Repository correspondence checker: 157 files / 1,404 pinned cases, green.
- Correspondence-checker unit suite: 24/24 green.
- All five pinned source SHA-256 values match the candidate receipt; JSON
  parsing, exact evidence locators, and candidate `git diff --check` are green.
- Default non-test release LLVM IR contains none of the ten Wave C7 test
  symbols or their case-specific test literals.
- The candidate changes only test code and correspondence documentation; no
  production behavior is changed.

Every relied-on Cargo invocation disabled incremental compilation. This
receipt changes no candidate test, production source, or machine ledger.
