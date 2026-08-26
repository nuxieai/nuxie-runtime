# Wave C7 text/input candidate

Pinned upstream: `rive-runtime` `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Sources and pinned blob SHA-256 values:

- `tests/unit_tests/runtime/text_input_test.cpp` (20 cases): `e97e16bad115007bfcf5a1caae319346c225c8a098d5811a700515e1a8671808`
- `tests/unit_tests/runtime/raw_text_input_test.cpp` (17 cases): `3a1c66073016d790d811219e760bb6818fd66024b7bedc113c9a0943fae3c406`
- `tests/unit_tests/runtime/text_test.cpp` (18 cases): `d3917b4de319fbb3d2eb7d4eae1deee4f53d509b460ee2377595696b8bfd5367`
- `tests/unit_tests/runtime/text_modifier_test.cpp` (2 cases): `2a05c8907b1dfee8261702cd5859f00c2f90e93150500a2dcaeafef4706b7e74`
- `tests/unit_tests/runtime/nested_text_run_test.cpp` (1 case): `dfff68cfddac3c1c6985b9247e06dc60837e2ff5039600b59834dbbb48f455a5`

## Candidate verdict

Candidate for fresh independent review: **10 executable passes, zero expected-red, 48 honest pending, 58 exact identities**.

- Three direct passes: raw-text-input case 17 and text cases 2 and 3.
- Seven adapted passes: raw-text-input case 1 is `cxx-language-only`; cases 9, 10, 11, 14, 15, and 16 declare the retained Rust ownership split between editable buffer and font geometry as `rust-safety`.
- Forty-eight pending cases have no claimed evidence. No aggregate or nearby test is counted.

Every claimed buffer pass is a distinct owner-local Rust test. The two claimed text queries are distinct live-Artboard tests that execute the exact fixture and sole pinned assertion. The candidate changes no production behavior: all new Rust code is inside existing `#[cfg(test)]` modules.

## Owner inventory and rejected substitutes

- `CursorPosition`, `Cursor`, and `RawTextInput` remain the direct owners for cursor ordering/saturation, character/word/subword movement, buffer mutation, selection, and journaling. The retained Rust design separates font shaping and visual geometry into `TextInputGeometry`/the live Artboard text slice. Six buffer-only cases therefore explicitly make the pinned raw font pointer, sizing fields, and no-op geometry refresh inapplicable; their complete buffer action/assertion streams remain literal and ordered.
- Visual cursor placement, bidi line movement, height/width measurement, and vertical cursor cases remain pending. A test-local font/shaping model or manual line table was rejected as a proxy for the missing exact combined RawTextInput authority.
- Existing `cpp_probe` text-input tests collapse multiple upstream cases, add assertions, and sometimes replace a live owner with graph topology. None is counted in the denominator.
- Existing simple-text and modifier tests collapse parameter streams or inspect static graph records instead of the complete live modifier owner. None is counted.
- The legacy nested-text-run test defines test-local get/set helpers whose only behavior is unconditional panic. It is not evidence; the row remains pending until the named live nested-run path API has a callable retained owner.

## Literal stream notes

- Raw case 1 preserves six ordered assertions. All constructed line/codepoint pairs are literal; only C++ operator spelling is adapted.
- Raw cases 9 and 10 contain no shared parameter loop. Every pinned movement action and every ordered cursor assertion is written separately.
- Raw case 11 preserves the pinned initial precomposed assignment, replacement with decomposed text, four moves, and four assertions.
- Raw cases 14 and 15 preserve the full delete/selection action and assertion streams, including both component checks represented by each upstream `CHECK_CURSOR` expansion.
- Raw case 16 preserves every insert, undo, redo, selected movement, replacement, text assertion, and both ordered endpoint assertions from every `CHECK_CURSOR` expansion.
- Raw case 17 preserves both `clearSelection` calls, the collapse check, and all four endpoint assertions.

## Validation

- Focused non-incremental execution: 10 passed, zero failed, zero ignored (eight owner-local raw-text-input tests and two live-Artboard text query tests).
- Strict Wave C7 source/identity/name/line/status/locator audit: 58/58 accepted; direct 3, adapted 7, pending 48; pass 10, unverified 48.
- Repository correspondence checker: passed for 157 files and 1,404 pinned `TEST_CASE` declarations.
- Correspondence checker unit suite: 24/24 passed.
- Pinned source hashes, JSON parse, exact Rust locators, scoped formatting, `git diff --check`, default release/IR containment, and exact-path staging are required before commit.

This is candidate evidence only and does not self-accept Wave C7.
