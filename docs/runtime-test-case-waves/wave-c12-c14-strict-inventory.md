# Waves C12-C14 strict inventory

Status: **READ-ONLY AUTHOR DOSSIER; NOT PARITY ACCEPTANCE**

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

This dossier records the strict case-by-case inventory performed before author
lanes begin. A manifest row, fixture, source string, ignored placeholder,
aggregate corpus runner, injected fake, or test-local implementation is not an
executable port. Each accepted case must retain its upstream fixture/action/
assertion stream against the live owner, or name a narrow approved adaptation.

## Wave C12: scripting properties

Scope: 22 cases in `scripting_properties_test.cpp`.

Strict result: **8 adapted passes / 0 direct passes / 0 real reds / 14
unported**. The manifest's current `ported-direct` classification is not
supportable.

Accepted adapted cases:

- property listeners survive userdata collection while subscribed;
- scripted list round-trip and listener behavior;
- `ViewModelInstanceList::removeAllOf` duplicate identity behavior;
- stable blob, image, and font userdata identity;
- blob listener identity-change behavior;
- runtime empty-blob behavior.

Unported cases:

- the integrated `data_binding_test.riv` property scenario;
- exact color, string, boolean, and enum script/log streams;
- all eight scripted-property silver cases;
- the exact initial `{10, 20, 30}` then `abcd` blob read/write stream.

The eight silver rows are explicitly `pending-scripted`; corpus and silver
metadata do not execute them. Required fixtures are available. The missing
high-level seam is a genuine scripted state-machine plus
`SerializingFactory` replay path, including the exact six-frame sequence in
`reset_shared_viewmodel_instance_test`.

Recommended partitions:

1. exact owner tests for cases 1-8 and 17-22 under a child of
   `crates/nuxie-scripting/src/vm/view_model.rs`;
2. one genuine scripted SRIV runner for cases 9-16 under
   `tools/silver-corpus/tests/`.

## Wave C13: renderer through update guard

Canonical scope and denominator:

- `scripting_renderer_test.cpp`: 4;
- `scripting_require_check.cpp`: 11;
- `scripting_scope_test.cpp`: 5;
- `scripting_text_runs.cpp`: 1;
- `scripting_transition_condition_test.cpp`: 1;
- `scripting_update_phase_guard_test.cpp`: 3.

Strict result: **19 direct passes / 4 adapted passes / 0 real reds / 2
unported**.

The renderer, require, scope, and text-runs cases execute their exact live
owners. The text-runs SRIV now passes with `--features scripting`; its manifest
note describing a frame-one red is stale. Feature-disabled zero-test output is
vacuous and must not be counted.

Unported cases:

- scripted transition condition: the exact fixture/action/SRIV replay is
  absent; the fake `ConditionScript`, optional injected probe, and
  `pending-scripted` row are inadmissible;
- update guard #1: one test executes the literal Lua against a test-local guard
  algorithm, while another reaches the production dirt/phase owner through an
  injected fake. Neither preserves the full direct path.

Recommended partitions:

1. a high-level scripted transition SRIV port following the text-runs runner;
2. a minimal real `ScriptInstance`-to-Artboard occurrence seam and exact update
   guard #1 port;
3. bookkeeping-only correction of stale text-runs and update/transition
   manifest claims.

## Wave C14: vector, wake, and scroll

Canonical scope and denominator:

- `scripting_vector_test.cpp`: 13;
- `scripting_wake_advance_test.cpp`: 2;
- `scroll_test.cpp`: 4;
- `scroll_velocity_test.cpp`: 4.

Strict result: **10 direct passes / 6 adapted passes / 1 real expected red / 6
unported**.

Vector cases 1-12 execute the live VM/vector/buffer owner. Case 13 is an honest
red: the exact one-million-iteration benchmark reaches the production
safepoint quota and fails with `script cycle exceeds 100000 script
safepoints`. The safety quota must not be globally lowered merely to turn this
test green.

Wake cases 1-2 are unported. `WakeHarness` mirrors `advance_active`, scheduler,
and event counters locally, so it proves a duplicate implementation. The
required seam must attach a real Lua `ScriptInstance` to a production Artboard
scripted occurrence, dispatch real pointer/keyboard events, and observe the
script's own counters.

All four scroll-silver cases are unported. Their current ignored tests end in
an unconditional `missing_silver_match` panic. The fixtures and SRIV baselines
exist, so each case should execute its literal actions through
`ArtboardInstance`, `StateMachineInstance`, and `SerializingFactory`, then use
the real SRIV parser/comparator.

All four scroll-velocity cases pass through the live runtime snapshot with a
narrow host-clock/schema-setter adaptation. The manifest claim that velocity
and `scrollActive` are omitted is stale.

Recommended partitions:

1. vector bookkeeping and a separately adjudicated scoped benchmark policy;
2. real Lua/Artboard wake attachment and observation;
3. four direct scroll SRIV cases under `tools/silver-corpus/tests/`;
4. scroll-velocity ledger correction only.

## Relied-on gates

Focused owner and integration tests were run with `CARGO_INCREMENTAL=0`.
Passing exploratory proxy tests were deliberately not counted. No production
or test files were changed during inventory.
