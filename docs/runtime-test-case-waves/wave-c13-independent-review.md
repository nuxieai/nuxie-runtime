# Wave C13 independent adversarial review

Status: **REJECTED; CORRECTION REQUIRED**

Reviewed frozen candidate commit
`6a1f2c6463f86a02e74f2d9031da7cdf3ce86809` against pinned upstream
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. This is a receipt-only review:
it changes no candidate test, ledger row, expectation, or runtime behavior and
does not accept Wave C13.

## Verdict

The 25-row census and all machine locators are structurally valid, and the
focused tests reproduce the candidate's 24 green outcomes plus its one genuine
transition SRIV red. Wave C13 is nevertheless not exact. Two rows omit pinned
assertions that the local owners can expose. Structural checker success cannot
substitute for those missing body-level observables.

## Blocking finding 1: renderer case 1 discards the pinned balance result

Row:
`tests/unit_tests/runtime/scripting/scripting_renderer_test.cpp#1`,
`can call renderer`.

Pinned C++ calls the literal `render`, then immediately asserts
`scriptedRenderer->end()` is true before attempting the retained renderer use.
The Rust evidence at
`crates/nuxie-scripting/tests/upstream_scripting_renderer.rs:32` calls
`ScriptInstance::call_draw` and unwraps only its unit result. The live owner at
`crates/nuxie-scripting/src/vm/lua_renderer.rs:33` does call
`call_draw_with_balance`, but then deliberately discards the returned boolean
with `map(|_| ())`. The later invalid-lifetime assertion proves cleanup, not
that the callback's save stack was balanced. Thus one of the pinned case's two
post-callback assertions is absent.

This omission is not covered by the row's native-scripting adaptation, which
only declares the `luaur_rt::Error` display prefix inapplicable. An exact
feature-gated balance seam already exists as
`ScriptVm::upstream_test_call_draw_with_balance`.

Required correction:

1. Execute the same literal source and draw table through that exact balance
   seam under `upstream-test-seams`.
2. Assert the returned value is true immediately after the draw and before
   calling `afterwards`, preserving the pinned assertion order.
3. Keep the adaptation limited to the raw-stack versus Rust error-display
   prefix.
4. Force the new balance assertion red independently and record the observed
   live value.

## Blocking finding 2: update-guard case 1 omits both phase assertions

Row:
`tests/unit_tests/runtime/scripting/scripting_update_phase_guard_test.cpp#1`,
`markNeedsUpdate is ignored during scriptUpdate`.

Pinned C++ explicitly asserts `isInUpdatePhase() == false` immediately before
`scriptUpdate()` and again immediately after it, before checking suppression.
The real-Lua Rust evidence at
`tools/silver-corpus/tests/upstream_scripting_update_phase_guard.rs:117`
observes only `ScriptUpdate` dirt: it infers the pre/post phase indirectly from
successful outside-phase dirt requests. It never reads either pinned boolean.

The booleans are not inherently unavailable. The authoritative private field
is `RuntimeComponent`'s `in_update_phase`, and existing internal tests at
`crates/nuxie-runtime/src/artboard/tests.rs:3524` directly assert both its
default and restored values. The C13-added case-2 test at line 3570 also reads
the same owner field directly. Case 1's adaptation declares only the C++
subclass dirt counter and protected direct `scriptUpdate` entry inapplicable;
it neither names nor justifies dropping the two phase assertions.

Required correction:

1. Add a narrow `tools`/test-only read seam for the retained occurrence's
   authoritative `in_update_phase` field; do not calculate or mirror it.
2. In the existing real-Lua, real-`ScriptedDrawable` case, assert false before
   the production update, invoke the update once, then assert false again
   before the dirt-suppression assertion.
3. Preserve the current literal generator, live `Context.markNeedsUpdate`, and
   production update owner. Do not substitute the existing fake
   `MarkNeedsUpdateScriptInstance` internal test.
4. Force both phase assertions red independently. The structured adaptation
   may continue to cover only the subclass dirt-call counter and protected C++
   entry point.

## Audited remainder

The other 23 rows preserve their pinned programs/fixtures, setup, action and
assertion streams through live owners. In particular:

- renderer cases 2-4 preserve the unbalanced result, literal oval programs,
  one/1,000 draw streams, frame boundaries, stack checks, collection cadence,
  and exact SRIV baselines;
- all 11 require and five scope cases preserve their module names, literal
  programs, cache actions, ordered values, error observations, scoped ranks,
  and the byte-identical `scope_probe.riv` fixture;
- text-runs preserves the exact named artboard, bind, initial draw, seven
  mutation frames, trigger multiplicities, 0.1-second advances, draws, and
  pinned SRIV comparison with scripting enabled;
- transition preserves the default artboard, ViewModel bind, 0.1 initial
  advance/draw, ordered `timelineBool` and `anyStateBool` mutations, both 0.016
  advances/draws, and real comparator. Forcing it red reproduces exactly
  `frame 1, op 30 (color): expected color, got save`;
- update cases 2 and 3 read the authoritative default phase field and preserve
  the two outside-phase requests with the declared idempotent-dirt adaptation.

## Verification receipt

- strict C13 shard: 25/25 locators resolved; 20 direct / five adapted;
  24 pass / one expected-red / zero pending;
- focused evidence: all 24 declared pass rows green; transition normally
  ignored and forced individually to the documented real comparator red;
- repository correspondence: 157 files / 1,404 pinned cases, green;
- correspondence-checker unit suite: 24/24 green;
- pinned checkout and all six source blobs verified; relied-on RIV fixtures
  and all four SRIV baselines are present and hashable, with the local
  `scope_probe.riv` byte-identical to pinned upstream;
- candidate JSON parsing, seven-file scoped diff, and diff whitespace checks:
  green;
- default release `nuxie-runtime` LLVM IR contains no C13 test-owner symbols or
  expected-red message.

Every relied-on Cargo invocation used `CARGO_INCREMENTAL=0` and disabled
incremental compilation for the invoked test or release profile.
