# Wave C13 exact test-port candidate

Status: **CANDIDATE; PENDING INDEPENDENT REVIEW**

Wave C13 covers all 25 active Catch cases in the six pinned scripting files
from `scripting_renderer_test.cpp` through
`scripting_update_phase_guard_test.cpp` at upstream SHA
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. It does not change production
behavior or declare Wave C13 accepted.

## Exact census

- 25/25 distinct rows mapped;
- classifications: 20 direct and five native-scripting adaptations;
- outcomes: 24 pass and one genuine expected-red;
- zero pending, differential, proxy, aggregate, or placeholder rows.

## New direct owner evidence

The scripted-transition case imports the pinned
`scripted_transition_condition.riv`, binds the live ViewModel, performs the
initial 0.1-second advance and draw, sets `timelineBool`, advances and draws at
0.016 seconds, sets `anyStateBool`, advances and draws at 0.016 seconds, and
compares the complete stream with the pinned SRIV. It is a genuine production
red: frame 1 operation 30 expects `color` but the live Rust path emits `save`.

Update-guard case 1 compiles the literal upstream Lua generator, attaches that
real `ScriptInstance` to a concrete production `ScriptedDrawable` occurrence,
and executes it through `ArtboardInstance::update_script_instances`. The live
owner suppresses `Context.markNeedsUpdate` while its authoritative update-phase
bit is set and accepts the same dirt request after the callback. No local guard
algorithm or fake `ScriptInstance` is used.

Update-guard case 2 directly reads the authoritative private per-occurrence
phase field from a production `RuntimeComponent`. Its test-only visibility is
the Rust counterpart of the C++ test subclass's `isInUpdatePhase` accessor; it
does not calculate or mirror the result. Case 3 calls the production outside-
phase owner twice and observes its real idempotent `ComponentDirt` bit. Its
structured adaptation records why the C++ subclass-only dirt-call counter is
not a Rust production observable.

The prior renderer, require, scope, and text-run evidence was re-run at its
live owner. The text-run test was explicitly run with `--features scripting`;
feature-disabled zero-test output is not counted.

## Gates

- focused non-incremental existing owner suites: 23/23 green;
- new update-guard suite: 3/3 green;
- scripted-transition normal suite: one explicitly ignored genuine red;
- scripted-transition expected-red forced individually and reproduced the
  exact frame/operation difference;
- strict pinned identity, ordinal, source-line, exact-name, classification,
  outcome, locator, adaptation, and ignored-reason validator: 25/25 green;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence checker unit suite: 24/24 green;
- direct rustfmt on the two new standalone files, JSON parse, and scoped diff
  checks: green;
- release non-test `nuxie-runtime` LLVM IR contains no Wave C13 test-owner
  symbols or expected-red message.

Every relied-on Cargo invocation uses `CARGO_INCREMENTAL=0` and
`CARGO_PROFILE_TEST_INCREMENTAL=false` (or the release equivalent).
