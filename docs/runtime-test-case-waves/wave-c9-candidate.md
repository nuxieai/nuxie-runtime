# Wave C9 state-machine candidate

Pinned upstream: `rive-runtime` `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Sources and pinned blob SHA-256 values:

- `tests/unit_tests/runtime/state_machine_test.cpp` (19 cases): `ebf4e8c241981b1ee99211e5a579f21ef1d4a2aad56cd7ec3184cd58116835d5`
- `tests/unit_tests/runtime/state_machine_event_test.cpp` (11 cases): `92372b24f0682d16afb07fbfb80cb6cc7255960c8abd9ec987e603ae71f0d225`
- `tests/unit_tests/runtime/state_machine_input_test.cpp` (one case): `7c02c756580dc4a4da8104495638dce9bb6aeb000b53426f31fcfc3622b2f2c0`
- `tests/unit_tests/runtime/semantic_state_machine_test.cpp` (15 cases): `b6f2b83272ad5df9a48169731e0f6121838beeb8dd4eb19dbfe20653d4f4b1a1`

## Candidate verdict

Candidate for fresh independent review: **25 executable passes, zero
expected-red, 21 honest pending, 46 exact identities**.

- Fifteen direct passes exercise retained live `ArtboardInstance`,
  `StateMachineInstance`, event-reporting, pointer, input, and semantic owners.
- Ten structured `rust-safety` adaptations replace C++ raw pointers, RTTI,
  typed template lookup, or `rcp` ownership spelling with retained imported
  identities, schema ancestry, `Option`, and owned VMI contexts. Every adapted
  row names its exact inapplicable observable.
- Twenty-one pending rows have no evidence, note, adaptation, proxy, or
  placeholder.

Fourteen new tests are distinct and literal. Eleven pre-existing semantic
tests were retained only after removing extra return-value assertions and
restoring the pinned label-driven list-item loops. No production code changes.

## Owner inventory and rejected substitutes

- State-machine cases 1-3 use the retained imported definition owner and a
  real live occurrence where the source does; case 4 uses the live machine
  exclusively. Cases 5-8 remain pending because the exact child
  Shape→Stroke→SolidColor stream, `AnimationResetFactory` resource pool, and
  live `layerState(0)` identity are not externally observable. The old case-8
  unconditional-panic placeholder is explicitly not evidence.
- State-machine cases 9-19 are SerializingFactory/SRIV streams. Static graph
  assertions, aggregate `cpp_probe` checks, renderer stream manifests, and
  nearby Silver corpus actions are not substitutes for their exact live
  fixture/action/draw/baseline owners, so every row remains pending.
- Event cases 1-8 are split from three legacy aggregate tests. Each new test
  preserves its own fixture, definition queries, artboard-before-machine
  initialization, pointer id/coordinates, timed advances, event order, and
  reset assertion. Event case 9 remains pending because Rust exposes no exact
  nested child state-machine/report owner. Case 10 remains pending without
  SerializingFactory/SRIV baseline authority.
- Event case 11 uses the authored imported VMI occurrence selected by pinned
  `createViewModelInstance(viewModelId, 0)`. It passes. The legacy ignored test
  selected a default-context owner instead and produced a false expected-red;
  it is not mapped.
- The input case uses the retained default-artboard local-object identities in
  exact lookup/assertion order. A global object-table scan is not counted.
- Semantic cases 1-7 and 9-12 use real Artboard, machine, semantic diff, pointer,
  action, focusable-trait, and view-model paths. Case 8 remains pending because
  Rust's public boolean manager projection cannot prove stable manager pointer
  identity. Cases 13-15 remain pending because the public integration seam
  cannot reproduce every pinned live `nodeById` state/bounds assertion; nearby
  diff-only tests omit or collapse those observables and are not evidence.

## Literal stream notes

- No accepted row is sourced from an aggregate facade or shared action loop.
  Fixture loading is shared; all case-specific values, actions, loops, and
  assertions remain in the distinct evidence function.
- Event pointer defaults are written explicitly as pointer id zero because the
  Rust API has no C++ default argument. Coordinates and call order are exact.
- Event case 8 preserves the `0`, `0.4`, `0.2` advance stream, `Half` event,
  and `0.1` seconds-delay comparison.
- Event case 11 preserves listener type count, ViewModel listener identity,
  initial zero report, `go` trigger, `0.016` advance, one `ding`, second
  `0.016` advance, and terminal zero report.
- Semantic helpers retain ten advances at `0.1`; case-local pointer/action and
  drain order remains unchanged.

## Validation

- Focused non-incremental new evidence: 14/14 passed, zero failed, zero ignored.
- Focused semantic evidence: 11 mapped cases pass through the pinned fixtures;
  the four intentionally pending nearby tests are not counted.
- Strict Wave C9 identity/status/schema/locator audit target: 46/46; direct 15,
  adapted 10, pending 21; pass 25, unverified 21.
- Repository correspondence target: 157 files / 1,404 pinned cases.
- Correspondence-checker unit target: 24/24.
- JSON parsing, source/artifact hashes, distinct locator resolution, scoped
  formatting/diff checks, default release IR containment, and exact-path
  staging are required before candidate commit.

This is candidate evidence only and does not self-accept Wave C9.
