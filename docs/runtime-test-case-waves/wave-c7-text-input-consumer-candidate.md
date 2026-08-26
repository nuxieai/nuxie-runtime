# Wave C7 TextInput consumer candidate

Accepted source correction: `e27e52eb3bf39b0a668aa25d76336cb1e9944f8a`

Pinned upstream: `rive-runtime`
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Pinned source: `tests/unit_tests/runtime/text_input_test.cpp`, 20 cases,
SHA-256 `e97e16bad115007bfcf5a1caae319346c225c8a098d5811a700515e1a8671808`.

## Candidate verdict

Five of the 20 previously pending TextInput cases are now literal executable
passes. Zero cases are expected-red and 15 remain pending. Across the frozen
58-row Wave C7 denominator, the candidate topology is 11 pass and 47 pending:
eight direct, three adapted, and 47 pending.

The five promoted rows are:

- case 4, backspace and delete;
- case 6, unhandled press and key release;
- case 9, native platform select-all;
- case 14, committed text insertion; and
- case 18, generated selection-radius publication.

Each row has one distinct live-Artboard test. The tests preserve the pinned
fixture, setup and advance placement, action order, handled results, and
assertion order. The existing debug entry points only expose or invoke the
retained occurrence-owned TextInput/RawTextInput state; no test-local editing,
navigation, hit, focus, or layout algorithm was added. Case 18 intentionally
asserts only the generated property value observed upstream rather than adding
a raw-radius proxy.

## Honest pending blockers

- Case 1 needs live typed child enumeration; case 2 needs the pinned serialized
  render stream and frozen silver.
- Cases 3, 7, 8, and 10-12 still cross reconstructed or non-literal shaped
  navigation ownership.
- Case 5 needs callable occurrence-level raw insertion without replacing it
  with the higher-level TextInput synchronization path.
- Case 13 needs exact state-machine TextInput hit/click routing.
- Cases 15-17 need the complete shared `updateMultiline` owner, including raw
  sizing, scroll reset, and dirt order.
- Case 19 remains blocked by focus/listener capability ownership. A discarded
  author probe selected the real authored FocusData target through
  `StateMachineInstance::set_focus`; committed text still returned unhandled,
  consistent with the unresolved `acceptsKeyboardInput` header row H3. No
  failing or weakened test is retained as evidence.
- Case 20 additionally needs the live TextInputCursor path observable across
  focus and blur; selection collapse alone cannot certify the row.

Every pending ledger row names its precise owner blocker. None has evidence,
an adaptation, or a completion claim.

## Validation

- Focused non-incremental test execution must discover and pass exactly the
  five `wave_c7_text_input_` tests, with zero failed or ignored.
- The Wave C7 ledger must retain exactly 58 identities and report direct 8,
  adapted 3, pending 47; pass 11, expected-red 0, unverified 47.
- All five evidence locators must resolve to the exact Rust symbol, the pinned
  source hash must match, JSON parsing and scoped `rustfmt --check` must pass,
  and the candidate diff/staging must contain only the test, ledger, and this
  receipt.
- Per the corrected workflow, global correspondence and release/IR gates are
  deferred to checkpoint or PR closeout; this unit adds no production or new
  debug-owner code.

This is author candidate evidence only and does not self-accept the five rows
or Wave C7.
