# FL-C State-Machine Owner-Family Translation Spec

This is the frozen pre-translation mini-map for FL-C. It binds the complete
StateMachineInstance-through-listener owner family before FL-C production
changes begin.

Pinned C++: `d788e8ec6e8b598526607d6a1e8818e8b637b60c`.

Accepted dependency: FL-A promotion
`f86d5ba0146697abc996310c62fa45e1f053144b`.

Provisional FL-B dependency:
`0b08970fccc42ff3677534d4dcece0f05f69a0bc`. Every FL-B file/member row
remains pending until independent reacceptance; FL-C may consume the
implementation without claiming that verification has occurred.

Active runtime base after the stable-Rust Apple compatibility repair:
`95eb04b7cfb847f24ba77872bd8a0ee43da1af41`.

## Finite closure

The executable ledger contains 49 FL-C C++ file rows and these eight pending
member rows:

1. `state_machine.inputs`
2. `state_machine.conditions`
3. `state_machine.transitions`
4. `state_machine.layer`
5. `state_machine.actions`
6. `state_machine.events`
7. `state_machine.collections`
8. `state_machine.advance`

The production Rust closure is:

- `crates/nuxie-runtime/src/state_machine.rs`
- `crates/nuxie-runtime/src/state_machine/instance.rs`
- `crates/nuxie-runtime/src/state_machine/transition_condition.rs`
- `crates/nuxie-runtime/src/animation.rs` only where the existing animation
  owner boundary is the direct C++ counterpart
- `crates/nuxie-runtime/src/artboard.rs` only for construction/advance owner
  integration required by the complete C++ family

Tests, C++ probes, the frame-loop ledger/checker, and evidence documents may
change with the owner family. FL-D DataBind/ViewModel settlement, FL-E live
draw, renderer backends, and Editor/product code do not.

FLR-16 applies to every lane. The default mapping is the C++ basename under
`crates/nuxie-runtime/src/state_machine/`: for example,
`state_machine_input.cpp` maps to `state_machine_input.rs`,
`listener_bool_change.cpp` maps to `listener_bool_change.rs`, and
`transition_number_condition.cpp` maps to
`transition_number_condition.rs`. Listener input types live under the matching
`state_machine/listener_types/` directory. Existing `state_machine.rs` becomes
a thin module entry point/shared-type coordinator as owners move out.
`state_machine/instance.rs` is the already-separated
StateMachineInstance owner and is accepted as the direct
`state_machine_instance.cpp` correspondence; the mapping records that explicit
name adaptation. Every lane updates both mechanical ledgers to the exact Rust
files it creates.

## Atomic production lanes

### FL-C1 — Inputs and listener-definition ownership (12 files)

C++ files:

- `src/animation/state_machine_input.cpp`
- `src/animation/state_machine_input_instance.cpp`
- `src/animation/state_machine_listener.cpp`
- `src/animation/state_machine_listener_single.cpp`
- `src/animation/listener_types/listener_input_type.cpp`
- `src/animation/listener_types/listener_input_type_gamepad.cpp`
- `src/animation/listener_types/listener_input_type_keyboard.cpp`
- `src/animation/listener_types/listener_input_type_semantic.cpp`
- `src/animation/listener_types/listener_input_type_viewmodel.cpp`
- `src/inputs/gamepad_input.cpp`
- `src/inputs/keyboard_input.cpp`
- `src/inputs/semantic_input.cpp`

Retention boundary:

- one StateMachine definition owns its authored inputs and listeners in file
  order;
- one StateMachineInstance owns one occurrence-local input slot for every
  authored input, including nullable/unsupported slots required by FLR-5;
- listener input types retain their authored source identity instead of
  rediscovering it during advance;
- concrete keyboard/gamepad/semantic constraint records attach to their
  current listener-input definition in file order, matching the three C++
  importer handoffs;
- clone/remount rebuilds occurrence state without copying live input values.

Member closed by the complete lane: `state_machine.inputs`.

Dependency correction: the original mini-map placed four occurrence dispatch
groups in this definition lane. Pinned source proves the keyboard, gamepad,
and semantic groups execute listener actions through `ListenerInvocation`, so
they belong with those owners in FL-C4. `TextInputListenerGroup::processEvent`
is entirely a client of `TextInput::{startDrag,drag,endDrag,selectWord,
selectLine}` and therefore moves with the `src/text/text_input.cpp` owner in
FL-E. No source file leaves the program; this is a topological correction that
prevents placeholder invocations or partial text-editing behavior.
The correction moves one source row from FL-C to FL-E, so the finite FL-C
closure is 49 files; the overall 341-file program closure is unchanged.

### FL-C2 — Conditions and transition definitions (12 files)

C++ files:

- `src/animation/state_transition.cpp`
- `src/animation/scripted_transition_condition.cpp`
- `src/animation/transition_bool_condition.cpp`
- `src/animation/transition_comparator.cpp`
- `src/animation/transition_condition.cpp`
- `src/animation/transition_focus_condition.cpp`
- `src/animation/transition_input_condition.cpp`
- `src/animation/transition_number_condition.cpp`
- `src/animation/transition_property_comparator.cpp`
- `src/animation/transition_property_viewmodel_comparator.cpp`
- `src/animation/transition_trigger_condition.cpp`
- `src/animation/transition_viewmodel_condition.cpp`

Read-only dependency: FL-B owns
`src/animation/blend_state_transition.cpp`; FL-C consumes its retained
definition without promoting the pending FL-B row.

Retention boundary:

- transitions own ordered condition definitions once;
- input/property/ViewModel sources retain their C++ owner identity;
- condition evaluation reads occurrence state through the retained definition
  and does not rebuild candidate vectors;
- transition eligibility, exit time, pause, duration, and allowed-state
  ordering remain literal pinned-C++ control flow.

Members closed by the complete lane: `state_machine.conditions` and
`state_machine.transitions`.

### FL-C3 — Layer and state occurrences (5 files)

C++ files:

- `src/animation/layer_state.cpp`
- `src/animation/state_instance.cpp`
- `src/animation/state_machine_layer.cpp`
- `src/animation/system_state_instance.cpp`
- `src/animation/nested_state_machine.cpp`

Retention boundary:

- each layer definition owns one stable ordered state-definition collection;
- each layer occurrence owns only C++'s dynamically created `any`, `current`,
  and transition-source (`stateFrom`) occurrences; it does not prebuild one
  occurrence per definition;
- current, source, transition, and waiting identities are retained typed
  handles rather than copied definition/action/animation descriptors;
- state entry/exit, transition interruption, reset, copy, and teardown follow
  the pinned initializer and visitation order.

Member closed by the complete lane: `state_machine.layer`.

The private `StateMachineLayerInstance` class lives inside
`state_machine_instance.cpp:140-711`, so that subsection is a read-only
supporting oracle for FL-C3 even though the whole
`state_machine_instance.cpp` file row remains in FL-C5. Rust maps that private
class to `state_machine/state_machine_layer_instance.rs`; this does not promote
the later whole-instance row.

### FL-C4 — Listener actions, events, and focus dispatch (18 files)

C++ files:

- `src/animation/focus_action_clear.cpp`
- `src/animation/focus_action_target.cpp`
- `src/animation/focus_action_traversal.cpp`
- `src/animation/focus_listener_group.cpp`
- `src/animation/gamepad_listener_group.cpp`
- `src/animation/keyboard_listener_group.cpp`
- `src/animation/listener_action.cpp`
- `src/animation/listener_align_target.cpp`
- `src/animation/listener_bool_change.cpp`
- `src/animation/listener_fire_event.cpp`
- `src/animation/listener_input_change.cpp`
- `src/animation/listener_invocation.cpp`
- `src/animation/listener_number_change.cpp`
- `src/animation/listener_trigger_change.cpp`
- `src/animation/listener_viewmodel_change.cpp`
- `src/animation/semantic_listener_group.cpp`
- `src/animation/state_machine_fire_action.cpp`
- `src/animation/state_machine_fire_trigger.cpp`

Retention boundary:

- listener/action definitions retain authored order and exact target/source
  identity;
- scheduled invocations and reported events are occurrence-owned queues;
- one apply-events call drains the pinned chained-notification loop to
  completion without rescanning authored definitions;
- focus, input, ViewModel, trigger, and event dispatch preserve C++ callback
  order and next-frame boundaries.

Members closed by the complete lane: `state_machine.actions` and
`state_machine.events`.

### FL-C5 — StateMachine definition/instance collections and advance (2 files)

C++ files:

- `src/animation/state_machine.cpp`
- `src/animation/state_machine_instance.cpp`

Retention boundary:

- the StateMachine definition owns ordered layer/input/listener definitions;
- StateMachineInstance retains all occurrence collections once at
  construction;
- `advanceAndApply`, `applyEvents`, layer advance, transition search, action
  execution, event reporting, reset, clone, and teardown follow pinned call
  order;
- steady advance performs no definition rediscovery or collection rebuild.

Members closed by the complete lane: `state_machine.collections` and
`state_machine.advance`.

## Focused correspondence matrix

- Definition order and occurrence slot count remain exact with unsupported or
  nullable inputs/listeners.
- Two instances share immutable definitions but isolate inputs, layers,
  triggers, scheduled actions, reported events, and script/listener state.
- Clone/remount preserves only the state copied by the pinned constructors.
- Condition probes cover bool, number, trigger, property, ViewModel, focus,
  input, and scripted evaluation, including duplicate and failing candidates.
- Transition probes cover exit time, zero duration, interruption, pause,
  waiting, animation-state, blend-state, and same-frame chained transitions.
- Listener/action probes cover authored FIFO, fire-event reporting,
  ViewModel/input/trigger writes, focus actions, align target, and script
  failure boundaries.
- Event probes lock `applyEvents` next-frame start and within-frame chained
  notification completion.
- Advance probes lock zero-second forcing, pending-event/listener return
  terms, layer order, reset, and keep-going propagation.
- The structural transition-search counter must move from Rust 154 to pinned
  C++ 176 for the canonical corpus by deleting rediscovery/shortcuts, not by
  adding benchmark-specific work.

## Structural deletion ratchets

The FL-C checker must reject:

- per-advance reconstruction of authored input, layer, transition, condition,
  listener, or action collections;
- cloned definition payloads beside occurrence state;
- candidate vectors or maps that replace pinned ordered owner traversal;
- event/listener rescans that coexist with retained scheduled queues;
- transition-search early exits absent from pinned C++;
- Rust-only advance return guards that omit zero-second forcing or pending
  event/listener terms.

Each negative control injects one forbidden form and proves the checker fails.

## Landing and acceptance

Each lane reads its complete pinned headers/sources before production edits,
adds lifecycle/counter evidence, runs focused runtime tests, the probe-armed
workspace, ordinary/scripted 317/317 + 647/647 zero-failure floors, and the
structural checker, then commits independently.

The complete wave additionally runs the 1,468-row pixel referee, C API, Apple
product/release/XCFramework checks, lint/format/diff, and committed-tree
9 MiB size gate. FL-B and FL-C rows remain pending until their respective
whole-wave independent verification.

No performance checkpoint runs at the FL-C boundary. Canonical timing is
deferred until every mapped FL-A-through-FL-E code row is ported and the
complete correctness/structure floor is green. FL-C acceptance is therefore
based on its source-corresponding behavioral, differential, structural, pixel,
ABI, Apple, size, and provenance evidence; timing cannot reorder FL-D/FL-E.
