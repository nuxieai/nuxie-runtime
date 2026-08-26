# Wave C14 wake-advance independent adversarial review

Verdict: **REJECTED — 0/2 literal ports accepted; shared fixture correction
required**

Reviewed candidate: `c5d4ef22036f342b52e6b2613ab255a155f61823`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Source: `tests/unit_tests/runtime/scripting/scripting_wake_advance_test.cpp`

## Exact evidence that is already correct

- The pinned source blob is exactly
  `dd704d375bb3c651c9b33e73213898cc22fe93e1f3316a1ec9493e0c7f5d5901`.
- `WAKE_SCRIPT` is byte-for-byte identical to the pinned raw string: 828
  bytes, SHA-256
  `0a84d4b1ca848fe275a413d814558bdba49d7a1dee45bdba4bd241d6794eb286`.
- Both implemented-method masks are exact: advances plus pointer-down for case
  1, and advances plus keyboard input for case 2.
- Both cases perform the pinned two `0.016` parking advances, preserve the two
  ordered parking counter assertions, dispatch the exact event inputs, assert
  the exact event counter, perform the third `0.016` advance, and assert the
  exact final advance counter.
- The fixture attaches the real VM instance to a real retained
  `ArtboardInstance` `ScriptedDrawable`. Advances run through
  `ArtboardInstance::advance_script_instances`; pointer input runs through the
  retained `HitScriptedDrawable`; keyboard input runs through the focused
  `RuntimeKeyboardListenerGroup`; and both event paths call the production
  `wake_script_advance_for_global`. There is no test-local scheduler, wake
  flag, or mirrored counter.
- `ScriptProgram::upstream_test_module_i32_getter` only reads and calls a named
  function in the script program's retained private environment. It does not
  reproduce dispatch or scheduling. Its `cfg(any(test, feature =
  "upstream-test-seams"))` guard excludes it from the default non-test release
  artifact.

## Rejected setup divergence

Both candidate tests use one `WakeFixture::new` that always authors
`FocusData`, `SemanticData`, and a `StateMachine`, always calls
`set_focus(Some(1))`, and always calls the scripted-object initialization
completion seam.

That shared setup is not literal for the pointer case. The pinned pointer body
constructs only its scripted drawable and `HitScriptedDrawable`; it neither
creates focus/semantic topology nor sets or asserts focus. The Rust pointer
route needs the production state-machine host to reach its retained hit
component, but it does not need `FocusData`, `SemanticData`, or a focused node.
The extra focus and semantic records therefore alter the pointer fixture
topology without serving the asserted owner path.

The `SemanticData` record is also unused by the keyboard case. Scripted
keyboard registration consults the direct `FocusData` child; semantic groups
are built only from authored listener definitions, of which this fixture has
none.

Correction should make the fixture topology case-specific: retain only the
real state-machine host required by the Rust pointer route for case 1; add
`FocusData` only for case 2's production keyboard route; remove the unused
`SemanticData` record from both.

## Rejected assertion divergence

The shared constructor executes `assert!(machine.set_focus(Some(1)))` in both
denominator cases. This is an extra asserted observable absent from both
pinned bodies. It is especially unrelated to case 1, whose pointer dispatch
does not consume focus. Case 2 needs to select the authored focus target as
Rust host setup, but its pinned assertions concern only parking, key dispatch,
and wake/re-advance counters; the later key-counter assertion already proves
that setup succeeded.

Correction should perform the required focus selection only in the keyboard
fixture and must not add its return value to the denominator assertion stream.
The existing `call_init` assertion corresponds to pinned
`REQUIRE(ensureScriptInitialized(...))` and is not rejected.

## Gates

- strict identities, ordinals, source lines, names, classifications, outcomes,
  and evidence locators: 2/2 valid;
- focused non-incremental suite: 2 passed, zero failed, zero ignored;
- each terminal counter assertion was forced independently in an isolated
  detached worktree and failed through the live owner with `left: 2, right:
  3`;
- repository correspondence census: 157 files / 1,404 cases, green;
- correspondence-checker unit suite: 24/24 green;
- default non-test release LLVM IR contains neither the getter nor either wake
  test/fixture symbol;
- candidate-scoped `git diff --check`: green;
- candidate changes no ordinary production behavior; new dependencies are
  dev-only, and the getter is feature-contained.

The two ledger rows must remain unaccepted until the unnecessary shared
topology and extra focus assertions are removed and the corrected cases are
independently rereviewed.
