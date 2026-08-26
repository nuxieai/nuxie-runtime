# Wave C14 wake-advance final independent rereview

Verdict: **ACCEPTED — 2/2 direct executable ports; zero pending, adapted,
differential, or expected-red cases**

Original candidate: `c5d4ef22036f342b52e6b2613ab255a155f61823`

Independent rejection: `11279eb6b`

Correction candidate: `39625d105d46388192cdf1942e44fc40b2308cad`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Source: `tests/unit_tests/runtime/scripting/scripting_wake_advance_test.cpp`

Pinned source SHA-256:
`dd704d375bb3c651c9b33e73213898cc22fe93e1f3316a1ec9493e0c7f5d5901`

## Corrected fixture topology

The correction removes every setup and assertion divergence identified in the
rejection.

The pointer case's `RuntimeFile` now contains only `Backboard`, `Artboard`, the
real `ScriptedDrawable`, and the `StateMachine` owner needed to traverse the
Rust production pointer route. It contains no `FocusData` or `SemanticData`,
performs no focus selection, and observes no focus result.

The keyboard case contains the same real owners plus only the direct
`FocusData` required by the production keyboard listener group. It has no
`SemanticData`. It selects the required focus id as host setup but does not
assert or otherwise expose the selection result. The dispatched key counter is
the pinned observable that proves the setup worked.

Both fixtures instantiate a `GraphFile`, production `ArtboardInstance`, real
retained scripted occurrence, and production `StateMachineInstance`. Pointer
dispatch reaches the retained `HitScriptedDrawable`; keyboard dispatch reaches
the focused `RuntimeKeyboardListenerGroup`; each calls the real scripted
callback and `wake_script_advance_for_global`. Parking and re-advance execute
through `ArtboardInstance::advance_script_instances`. There is no test-local
scheduler, wake flag, mirrored counter, semantic topology, or proxy owner.

## Literal program and case actions

`WAKE_SCRIPT` is byte-for-byte identical to the pinned raw string: 828 bytes,
SHA-256
`0a84d4b1ca848fe275a413d814558bdba49d7a1dee45bdba4bd241d6794eb286`.

The pointer case retains the exact advances-plus-pointer-down mask, two `0.016`
parking advances and ordered counter checks, pointer down at `(1.0, 1.0)` with
pointer id `0` and the production route's zero timestamp, event counter check,
third `0.016` advance, and final advance-counter check.

The keyboard case retains the exact advances-plus-keyboard mask, the same
ordered parking sequence, and the exact `Key::a`, no-modifiers, pressed,
not-repeat input as `65, 0, true, false`, followed by the pinned key-counter and
re-advance assertions.

Each case has only its pinned initialization requirement, two parking counter
checks, event counter check, and terminal advance counter check. The shared
helper does not collapse the assertions, and there are no additional focus,
semantic, or script-counter observables.

## Observation seam and production containment

`ScriptProgram::upstream_test_module_i32_getter` is observation-only: it reads
and invokes a named getter from the retained script program's private Lua
environment. It does not dispatch events, schedule advances, mirror counters,
or mutate runtime ownership. Its `cfg(any(test, feature =
"upstream-test-seams"))` guard keeps it out of a default non-test build, and the
integration test itself requires the explicit `upstream-test-seams` feature.
The added graph/schema/fixture dependencies remain dev-only.

## Independent forced evidence

In an isolated detached worktree at the correction commit, I independently
changed only each case's terminal expected advance count from `2` to `3` and
ran that case non-incrementally. The pointer test and keyboard test each failed
through the live owner with `left: 2` and `right: 3`. Each assertion was
restored, the worktree was byte-clean against `39625d105`, and the isolated
worktree was removed before this receipt was written.

## Fresh gates

- strict Wave C14 wake resolver: both identities, ordinals, pinned source
  lines/names, evidence symbols, and refreshed line locators valid; two direct
  passes; zero pending, adapted, differential, expected-red, or
  not-applicable cases;
- focused non-incremental suite: 2 passed, zero failed, zero ignored;
- pinned source and literal script hashes: exact;
- repository correspondence census: 157 files / 1,404 pinned cases, green;
- correspondence-checker unit suite: 24/24 green;
- default non-test release LLVM IR: no wake getter, test, fixture, script, or
  counter symbol retained;
- candidate-through-correction `git diff --check`: green;
- correction commit scope: only the wake test, its correction document, and
  refreshed wake ledger locators.

Wave C14 wake advance is accepted as 2/2 exact direct executable
correspondence.
