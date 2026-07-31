# FL-D5–D7 test ledger

Pinned upstream: `/Users/levi/dev/oss/rive-runtime` at
`d788e8ec6e8b598526607d6a1e8818e8b637b60c`.

## Assigned upstream classes

- Class A, `component_list_test.cpp`: 29 / 29 cases absorbed. The port keeps
  the upstream case split for list counts, mounted Artboard/state-machine
  ownership, layout-node/bounds transfer, row DataContext and labels,
  listener routing, NumberToList rows, virtualization/manual scroll,
  horizontal/vertical overrides, reset, non-layout positioning, follow-path
  and distance, hit order, nested data-bound virtual rows, map rules,
  stateful rows, child-origin offset, default order, and draw-index override.
  Owner-level assertions live in `artboard.rs`,
  `artboard_component_list.rs`, `nested_artboard*.rs`, and `cpp_probe.rs`;
  renderer-observable cases remain represented by their named silver entries.
- Class A, `component_test.cpp`: 8 / 8 active cases absorbed (the ninth
  `TEST_CASE` in the source is inside the upstream block comment). Coverage
  retains ViewModel-instance identity, multi-instance and multi-property
  independence, nested stateful ownership, list input/output bridges,
  Artboard swap, borrowed stateful nested sources/source switching, bridge
  cleanup, and keyed triggers. The corresponding named silver entries remain
  honest where the action interpreter cannot encode the upstream mutation.
- Class C addendum, `default_state_machine_test.cpp`: 1 / 1. The `entry.riv`
  fixture now asserts the public `default_state_machine_index`,
  `default_state_machine`, and `default_scene` API. Selection matches pinned
  C++: only an explicit in-range ordinal is the default state machine;
  `defaultScene` falls back to state machine 0, then animation 0, then none.
- Direct owner regressions: five ListenerGroup tests cover group-local pointer
  history/pooling, click/drag/Down-to-Out phases, disable/enable reset
  behavior, exact non-finite payload retention, and the C++ hover/click/direct/
  drag overwrite order. Pointer capture and its safe host event context now
  live in each group record; the duplicate StateMachineInstance side vector
  was deleted. Integration tests retain exact non-finite history and
  Exit-to-release wiring.
- Component-list virtualization now exercises remove-to-pool/remount reuse:
  the child allocation identity is retained, authored child/state-machine
  state is restored from a cold source clone (the safe Rust projection of C++
  property recorders), and the new row context is rebound before advance.
- Event callback tests mutate an imported live Event before a keyed trigger
  and verify that `event.rs::trigger_event` projects the mutation at delivery,
  not the stale import snapshot.
- Artboard tests cover every default-scene branch, the exact DataBind polls
  before late joysticks/final component settlement, and the root `advance`
  return term for retained Components dirt.

## Silver disposition

The D5/D7 port removes no action-interpreter blocker by itself. The remaining
unsupported assigned entries are blocked by `view-model-mutation`,
`runtime-object-mutation`, or `pointer-expression-encoding`, not by a missing
component-list or nested-Artboard runtime owner, so none is falsely promoted.
Newly executable cases, if any, are accepted only from the generated runner
result. `cpp-rust-exact` is never allowed below its incoming floor of 32.

## Required gates

Final command tallies and silver movement are recorded in the landing report
after running:

1. `cargo test -p nuxie-runtime`
2. `cargo test -p nuxie-runtime --features tools --test cpp_probe -- --nocapture`
3. `cargo test -p nuxie --lib`
4. ordinary and scripted golden compares
5. silver corpus
6. C++/Rust binary compare
7. runtime frame-loop port checker

Pinned-C++ harness output is redirected into this worktree's `target/`
subdirectories; the pinned checkout remains read-only.
