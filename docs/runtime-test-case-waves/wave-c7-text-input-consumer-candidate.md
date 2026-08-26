# Wave C7 TextInput consumer candidate

Accepted source correction: `e27e52eb3bf39b0a668aa25d76336cb1e9944f8a`

Pinned upstream: `rive-runtime`
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Pinned source: `tests/unit_tests/runtime/text_input_test.cpp`, 20 cases,
SHA-256 `e97e16bad115007bfcf5a1caae319346c225c8a098d5811a700515e1a8671808`.

## Corrected candidate verdict

This correction incorporates every finding from the independent rejection at
`d36afa15c`. Five of the 20 previously pending TextInput cases are literal
executable passes, one is an executable expected-red, and 14 remain pending.
Across the frozen 58-row Wave C7 denominator, the corrected candidate topology
is 11 pass, one expected-red, and 46 pending: nine direct, three adapted, and
46 pending.

The five promoted rows are:

- case 4, backspace and delete;
- case 6, unhandled press and key release;
- case 9, native platform select-all;
- case 14, committed text insertion; and
- case 18, generated selection-radius publication.

Case 2 is a distinct direct executable expected-red. Its Silver test consumes
the existing literal `text_input` corpus entry: pinned `text_input.riv`, named
`Text Input - Multiline` artboard, zero advance, draw, and frozen
`text_input.sriv` oracle. The retained first divergence is frame 0, operation
25 (`transform`), field `xy`: expected `-0.0` (`0x80000000`), got `0`.

Each row has one distinct live-Artboard test. The tests preserve the pinned
fixture, setup and advance placement, action order, handled results, and
assertion order. The existing debug entry points only expose or invoke the
retained occurrence-owned TextInput/RawTextInput state; no test-local editing,
navigation, hit, focus, or layout algorithm was added. Case 18 intentionally
asserts only the generated property value observed upstream rather than adding
a raw-radius proxy.

## Honest pending blockers

- Case 1 needs live typed child enumeration.
- Cases 3, 7, 8, and 10-12 still cross reconstructed or non-literal shaped
  navigation ownership.
- Case 5 needs callable occurrence-level raw insertion without replacing it
  with the higher-level TextInput synchronization path.
- Case 13 needs exact state-machine TextInput hit/click routing.
- Cases 15-17 need the complete shared `updateMultiline` owner, including raw
  sizing, scroll reset, and dirt order.
- Case 19 remains blocked by the missing live retained forwarding owner.
  Pinned dispatch walks `FocusNode` to its `FocusData` focusable and then
  `m_textInputListeners` to `TextInput`; Rust instead searches authored
  `keyboard_listener_groups`, so it never reaches the focused fixture
  TextInput. `acceptsKeyboardInput` is not the causal dispatch gate.
- Case 20 needs the complete live focus/selection/blur evidence stream,
  specifically the retained `TextInputCursor::localClockwisePath()`
  null/non-null/null observations. The accepted focus/blur callbacks and
  selection collapse cover only part of the case.

Every pending ledger row names its precise owner blocker. None has evidence,
an adaptation, or a completion claim.

## Validation

- Focused non-incremental `cpp_probe` execution must discover and pass exactly
  the five promoted live-owner tests, with zero failed or ignored.
- The distinct ignored Silver expected-red must execute the full replay and
  fail at exactly frame 0, operation 25 (`transform`), field `xy`, expected
  `-0.0` (`0x80000000`), got `0`.
- The Wave C7 ledger must retain exactly 58 identities and report direct 9,
  adapted 3, pending 46; pass 11, expected-red 1, unverified 46.
- All six evidence locators must resolve to the exact Rust symbol, the pinned
  source hash must match, JSON parsing and scoped `rustfmt --check` must pass,
  and the candidate diff/staging must contain only the test, ledger, and this
  receipt.
- Per the corrected workflow, global correspondence and release/IR gates are
  deferred to checkpoint or PR closeout; this unit adds no production or new
  debug-owner code.

The direct `silver-corpus validate --id text_input` command remains stopped
before case selection by the unrelated pre-existing `global_variables_test`
manifest validation error: its divergent entry does not record a first
difference. This correction does not change that row or claim corpus-wide
validation success. The distinct test follows the existing focused-test
convention: it selects the parsed literal `text_input` entry and executes its
complete action stream through the same `Execution` owner used by the corpus.

This is author correction evidence only and does not self-accept the corrected
TextInput rows or Wave C7.
