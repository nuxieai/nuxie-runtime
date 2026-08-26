# Wave C7 TextInput consumer independent review

Verdict: **REJECTED — one callable expected-red row is omitted and two
pending blockers misidentify focus ownership**

Reviewed candidate: `c7d880aeb53f90e1c01dc9dcc75235529a0c0633`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

The complete pinned `text_input_test.cpp` has 20 cases and SHA-256
`e97e16bad115007bfcf5a1caae319346c225c8a098d5811a700515e1a8671808`.
The candidate preserves all 20 identities, ordinals, lines, and names.

## Accepted promotions

Cases 4, 6, 9, 14, and 18 are distinct literal live-owner tests. They preserve
the exact fixture, raw setup, advance placement, action order, handled results,
intermediate assertions, and final assertions. Their debug entry points expose
or invoke the retained occurrence-owned TextInput/RawTextInput state; they do
not implement a test-local editor algorithm. Case 18 correctly preserves the
pinned row's sole generated-property assertion rather than substituting an
unasserted raw-radius projection.

## Required correction

1. Case 2 is not pending. `silver-corpus.toml` already contains the callable
   literal `text_input` replay with the pinned `text_input.riv`, named
   `Text Input - Multiline` artboard, zero advance, draw, and frozen
   `text_input.sriv` oracle. It is classified `diverges` with the recorded
   first difference `frame 0, op 25 (transform), field xy: expected -0.0
   (0x80000000), got 0`. Add one distinct Silver test locator, retain the full
   replay, and classify this row `direct` / `expected-red`; do not ignore this
   existing owner as pending.
2. Case 19's blocker must name the missing retained focus forwarding owner.
   Pinned dispatch is `FocusManager::textInput` -> live `FocusNode` -> its
   `Focusable` (`FocusData`) -> `m_textInputListeners` -> `TextInput`. Rust's
   `text_input_at_focus_data` instead searches authored
   `keyboard_listener_groups`, so the fixture's focused TextInput is never
   reached. `acceptsKeyboardInput()` is a host capability hint and does not
   gate pinned `keyInput`/`textInput` dispatch; header row H3 is not the causal
   blocker.
3. Case 20 should not claim incomplete generic "focus-capability dispatch."
   The accepted production correction already invokes TextInput focus/blur
   owners. The remaining consumer-evidence blocker is the complete live
   focus/selection/blur stream including the retained
   `TextInputCursor::localClockwisePath()` null/non-null/null observations.

The other 12 pending TextInput rows have precise owner blockers and empty
evidence. No static graph, proxy, collapsed assertion stream, altered fixture,
or test-local algorithm is credited.

After these corrections the TextInput subset is **5 pass, 1 expected-red, and
14 pending**. The frozen 58-row Wave C7 topology is **11 pass, 1 expected-red,
and 46 pending**: **9 direct, 3 adapted, and 46 pending**. The five accepted
promotions need no test changes.

## Checks

- Focused non-incremental `cpp_probe` execution: 5 passed, 0 failed, 0 ignored.
- All five candidate evidence locators resolve exactly; scoped
  `rustfmt --check`, JSON parsing/topology checks, and candidate
  `git diff --check` pass.
- The pinned source hash matches. No added TextInput consumer test is ignored.
- Direct `silver-corpus validate --id text_input` is currently stopped before
  selection by the unrelated pre-existing `global_variables_test` manifest
  validation error. The callable `text_input` entry and its frozen divergence
  are nevertheless present and cannot honestly be classified pending.

This review changes no production code, tests, or machine ledger.

## Narrow correction rereview

Verdict: **ACCEPTED**

Reviewed correction: `3693b5d767da6f99e8a64a3e6508a7e62a2d37ba`

The correction satisfies all three requests above without changing the five
previously accepted pass tests:

- case 2 now has one distinct direct Silver test over the existing literal
  `text_input` corpus entry. Default execution reports exactly one ignored
  expected-red. Explicit `--ignored` execution runs the complete owner and
  fails at the frozen difference: `frame 0, op 25 (transform), field xy:
  expected -0.0 (0x80000000), got 0`;
- case 19 now identifies the absent live
  `FocusNode -> FocusData::m_textInputListeners -> TextInput` forwarding path
  and no longer attributes dispatch to `acceptsKeyboardInput`; and
- case 20 now identifies only the missing complete live cursor-path
  focus/selection/blur evidence stream.

The corrected ledger has 58 unique rows and the exact topology **11 pass, 1
expected-red, 46 pending; 9 direct, 3 adapted, 46 pending**. Its TextInput
subset is **5 pass, 1 expected-red, 14 pending**, `max_pending` is 46, and all
six evidence locators resolve exactly. JSON/topology checks, targeted
`rustfmt --check`, candidate `git diff --check`, and the proof that
`cpp_probe.rs` is unchanged from the rejection checkpoint all pass.
