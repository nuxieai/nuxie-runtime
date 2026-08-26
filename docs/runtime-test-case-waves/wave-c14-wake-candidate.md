# Wave C14 wake-advance exact test-port candidate

Status: **CANDIDATE; PENDING INDEPENDENT REVIEW**

This narrow Wave C14 slice covers both active Catch cases in pinned
`scripting_wake_advance_test.cpp` at upstream SHA
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`. The pinned source blob is
`dd704d375bb3c651c9b33e73213898cc22fe93e1f3316a1ec9493e0c7f5d5901`.
It adds no production behavior and does not declare the slice accepted.

## Exact census

- two of two cases are distinct discoverable direct executable ports;
- outcomes: two live-owner passes, zero expected-red, zero pending;
- the former local `WakeHarness` scheduler and counter mirrors are removed.

## Body-level evidence

Both cases compile the literal typed `WAKE_SCRIPT`, instantiate its real Lua
table, call its real initializer, and attach the resulting `ScriptInstance` to
a production `ArtboardInstance` scripted occurrence using the exact pinned
implemented-method bitmask. The fixture uses a real `ScriptedDrawable`,
`FocusData`, `SemanticData`, and `StateMachine`, not a fake
`AdvanceScriptInstance` or a duplicated wake scheduler.

The shared parking helper records the script module's own `advanceCount`, calls
the production artboard advance with `0.016`, observes exactly one increment,
calls the same production advance again, and observes that the false-returning
script remains parked. Case 1 then dispatches pointer down through the real
state-machine hit path with `(1.0, 1.0)`, timestamp `0.0`, and pointer id `0`.
Case 2 dispatches the focused keyboard event through the real state-machine
path with `Key::a`, no modifiers, pressed, and not repeated. Each case reads
the Lua chunk's own getter closure, observes the event counter at 1, performs
the next production advance, and observes `advanceCount == 2`.

The sole new harness declaration is feature-gated to `upstream-test-seams` and
allows a test to call a named numeric getter in the compiled program's private
Lua environment. It does not expose or reproduce scheduling, input dispatch,
wake behavior, or a host-side counter.

## Gates

- focused non-incremental suite: 2/2 green;
- both terminal assertions were forced red independently and each reported
  the script-owned production result `left: 2, right: 3`;
- strict pinned identity, ordinal, source-line, exact-name, classification,
  outcome, and evidence-locator validation: 2/2 green;
- repository correspondence checker: 157 files / 1,404 cases, green;
- correspondence-checker unit suite: 24/24 green;
- scoped rustfmt, JSON parsing, diff checks, and default non-test artifact
  checks: green.

Every relied-on Cargo invocation uses `CARGO_INCREMENTAL=0` and disables
incremental compilation for the invoked test or release profile.
