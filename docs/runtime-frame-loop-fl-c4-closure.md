# Listener action, event, and dispatch owner-family closure

This is the pre-production closure checklist for the complete pinned-C++
listener-action, event, focus, keyboard, gamepad, and semantic dispatch family
at `d788e8ec6e8b598526607d6a1e8818e8b637b60c`.

Production is not eligible for review until every source row below has:

1. a filename-corresponding Rust owner;
2. a live C++ differential or an explicit source-cited structural proof;
3. its import, ownership, ordering, callback, queue, clone, and teardown
   behavior covered; and
4. an injected negative control for every structural rule that could otherwise
   regress silently.

The complete family is one publication unit. No subset is submitted for
acceptance.

## Source-to-Rust closure

| Pinned C++ owner | Complete semantics to retain | Focused Rust owner | Required proof |
| --- | --- | --- | --- |
| `src/animation/focus_action_clear.cpp` | ignore the invocation; null machine is a no-op; clear only through the occurrence's existing focus manager | `crates/nuxie-runtime/src/state_machine/focus_action_clear.rs` | null/empty/already-clear/live-focus differential; no manager rediscovery |
| `src/animation/focus_action_target.cpp` | resolve the exact target id; require `Node`; select the first direct `FocusData` child in authored child order; set focus through the occurrence | `crates/nuxie-runtime/src/state_machine/focus_action_target.rs` | missing/wrong target, duplicate direct focus children, and constructor-time unattached target differential |
| `src/animation/focus_action_traversal.cpp` | null machine is a no-op; traversal values 0–5 map to next, previous, up, down, left, right; every other value maps to next | `crates/nuxie-runtime/src/state_machine/focus_action_traversal.rs` | all six values plus invalid-value differential; constructor-time empty-topology case |
| `src/animation/focus_listener_group.cpp` | one occurrence retains exact `FocusData`, listener definition, and machine identities; register once at construction, unregister once at destruction; cache focus/blur membership; callbacks only queue the matching event | `crates/nuxie-runtime/src/state_machine/focus_listener_group.rs` | registration/teardown and focus/blur/duplicate callback ordering differential |
| `src/animation/gamepad_listener_group.cpp` | retain exact focus/listener/machine identities; always register/unregister on the `FocusData`; scripted drawable dispatch precedes listener constraints and reports the dispatched drawable; listener path performs FIFO changes, marks advance, and returns false | `crates/nuxie-runtime/src/state_machine/gamepad_listener_group.rs` | connected/event/disconnected payloads, scripted handled/unhandled, constraints failure, FIFO action, return value, and teardown differentials |
| `src/animation/keyboard_listener_group.cpp` | register keyboard/text channels from either listener flags or scripted-object wants flags; unregister the identical channels; `TextInput` dispatch wins, then listener-less scripted dispatch, then listener constraints/actions; listener paths always return false | `crates/nuxie-runtime/src/state_machine/keyboard_listener_group.rs` | key press/repeat/modifiers, owned text, TextInput precedence, scripted precedence, constraints failure, listener return, and channel teardown differentials |
| `src/animation/listener_action.cpp` | import by `parentKind`: listener actions transfer unique ownership to the current listener importer; transition/state actions transfer to the current layer-component importer; missing/wrong importer or unknown parent kind fails; execution remains authored FIFO | `crates/nuxie-runtime/src/state_machine/listener_action.rs` | all parent kinds, missing importers, unknown kind, duplicates, and failed-middle-action ordering proof |
| `src/animation/listener_align_target.cpp` | only pointer invocation supplies current/previous coordinates; every other invocation uses zeroes; resolve exact `Node`; invert parent world or no-op; preserve-offset applies the local delta, otherwise replace local position | `crates/nuxie-runtime/src/state_machine/listener_align_target.rs` | pointer/non-pointer, preserve/replace, transformed parent, noninvertible parent, missing/wrong target differentials |
| `src/animation/listener_bool_change.cpp` | null input definitions are forward-compatible; wrong concrete direct/nested types reject import; a non-empty nested id wins over the direct id; values 0/1 set false/true and every other value toggles | `crates/nuxie-runtime/src/state_machine/listener_bool_change.rs` | null/wrong/bad-index import, direct/nested precedence, 0/1/toggle, and missing occurrence differentials |
| `src/animation/listener_fire_event.cpp` | resolve the exact event id at perform time; wrong/missing targets are no-ops; report the exact retained event occurrence | `crates/nuxie-runtime/src/state_machine/listener_fire_event.rs` | wrong/missing/duplicate event ids, context propagation, report order, and next-frame delivery differential |
| `src/animation/listener_input_change.cpp` | import requires both current state-machine and artboard importers; validate the resolved nested input first, otherwise the direct state-machine input slot; preserve forward-compatible null slots; delegate parent attachment through `ListenerAction` | `crates/nuxie-runtime/src/state_machine/listener_input_change.rs` | missing importer, nested/direct precedence, bad index, null slot, wrong type, and parent attachment differentials |
| `src/animation/listener_invocation.cpp` | one owned tagged occurrence for pointer, keyboard, text, focus, reported event, ViewModel change, none, gamepad connected/event/disconnected, and semantic payloads; snapshots and strings are owned values | `crates/nuxie-runtime/src/state_machine/listener_invocation.rs` | all eleven variants, owned text/snapshot lifetime, exact accessors, and clone/isolation proof |
| `src/animation/listener_number_change.cpp` | null direct/nested input definitions are forward-compatible; wrong concrete types reject import; nested id wins; exact authored float is assigned | `crates/nuxie-runtime/src/state_machine/listener_number_change.rs` | null/wrong/bad-index import, direct/nested precedence, NaN/infinity/signed-zero, and missing occurrence differentials |
| `src/animation/listener_trigger_change.cpp` | null direct/nested input definitions are forward-compatible; wrong concrete types reject import; nested id wins; nested fire carries `CallbackData(machine, 0)`, direct fire targets the occurrence | `crates/nuxie-runtime/src/state_machine/listener_trigger_change.rs` | null/wrong/bad-index import, direct/nested precedence, callback identity/value, and repeated-trigger differentials |
| `src/animation/listener_viewmodel_change.cpp` | import takes ownership of the current bindable property; destruction deletes it once; perform resolves the occurrence-local bindable, updates only the target-to-source bind, seeds a ViewModel target from the live main context when required, then dirties the paired source-to-target bind | `crates/nuxie-runtime/src/state_machine/listener_viewmodel_change.rs` | missing importer, every bindable type, no source/target bind, live ViewModel target, exact dirt sink, duplicate occurrences, and teardown proof |
| `src/animation/semantic_listener_group.cpp` | retain exact semantic/listener/machine identities; register/unregister only a non-null semantic owner; tap/increase/decrease queue only when listener constraints pass | `crates/nuxie-runtime/src/state_machine/semantic_listener_group.rs` | null owner, each action, constraints failure, FIFO queue, duplicate callback, and teardown differentials |
| `src/animation/state_machine_fire_action.cpp` | import requires the current layer-component importer and appends this exact fire-action occurrence in authored order | `crates/nuxie-runtime/src/state_machine/state_machine_fire_action.rs` | missing importer, state/transition ownership, duplicate occurrence order, and clone/remount proof |
| `src/animation/state_machine_fire_trigger.cpp` | import/decode/copy retain the complete DataBind path; perform resolves it only against the live DataContext and fires only a trigger property; missing context/path/property or wrong type is a no-op | `crates/nuxie-runtime/src/state_machine/state_machine_fire_trigger.rs` | nested path, relative path, missing/wrong property, duplicate occurrences, clone/copy, and same-frame trigger differential |

`src/animation/state_machine_listener.cpp` and the relevant sections of
`src/animation/state_machine_instance.cpp` are supporting oracles. Their whole
file rows remain owned by FL-C1 and FL-C5 respectively; this family may cite
and test their action/queue call sites without promoting those rows.

## Complete lifecycle and ordering closure

- [ ] Every listener/state/transition retains its authored action occurrences
  once, in insertion order. Unsupported or malformed occurrences follow the
  pinned import result; Rust may not silently compact a valid nullable slot.
- [ ] `StateMachineListener::performChanges` visits the retained action list
  exactly once in FIFO order for each invocation. An action failure or no-op
  does not rebuild or reorder the list.
- [ ] Focus, keyboard, gamepad, and semantic group occurrences retain exact
  target/listener/machine identities and register/unregister symmetrically.
- [ ] `ListenerInvocation` represents all eleven C++ alternatives with owned
  strings and snapshots. Consumers branch on the retained alternative rather
  than reconstructing payloads from ambient state.
- [ ] Listener input actions validate direct and nested definition types at
  import, preserve forward-compatible null definitions, and use nested-id
  precedence at perform time.
- [ ] Fire-event actions enqueue exact event occurrences. Public host draining
  does not consume the private listener-delivery queue.
- [ ] `applyEvents` updates DataBinds, swaps both reporting queues, delivers
  events before ViewModel notifications, and drains chained notifications for
  at most 100 iterations in the same call.
- [ ] Focus and semantic callbacks enqueue occurrence-owned records and mark
  the machine for advance. Processing moves and clears the current batch
  before FIFO action execution, so callbacks created during processing wait
  for the next batch/frame.
- [ ] New-frame processing order remains focus events, semantic events, then
  reported event/ViewModel notifications before data-binding and layer
  advance.
- [ ] `advance(0)` preserves C++'s forced keep-going behavior and the facade
  return includes pending reported-event and listener-ViewModel queues.
- [ ] State/transition fire actions execute in authored order and observe
  their exact occurrence code. Fire-trigger paths resolve against the live
  DataContext at perform time.
- [ ] Clone/remount shares immutable definitions but reconstructs every group,
  queue, registration, callback target, and mutable invocation occurrence
  without copying live pending state.
- [ ] Destruction unregisters groups and releases owned bindable/action
  occurrences once, in the same owner order, without leaving callback sinks
  attached.

## Adversarial publication review

- [ ] Parent-kind import ownership: listener, transition, and state action
  parents attach to the correct current importer; missing/wrong/unknown parents
  fail without leaking or reparenting.
- [ ] Authored FIFO with duplicates: duplicate actions, fire actions,
  listener groups, and notifications retain insertion order and occurrence
  identity; no map/set replacement is permitted.
- [ ] Malformed input actions: bool, number, and trigger cover wrong type,
  bad direct index, bad nested id, forward-compatible null slots, and nested-id
  precedence.
- [ ] Bool and number edge values: bool 0/1/toggle plus number NaN,
  infinities, and signed zero match the pinned setter behavior.
- [ ] Trigger callback identity: nested triggers receive the current
  machine and callback value zero; direct and repeated fires retain their
  occurrence semantics.
- [ ] All invocation alternatives: pointer, keyboard, text, focus, event,
  ViewModel, none, three gamepad forms, and semantic payloads are live-tested.
- [ ] Owned invocation payloads: text and gamepad snapshots survive caller
  mutation/drop and isolate cloned occurrences.
- [ ] Align target matrix: pointer versus non-pointer, preserve versus
  replace, transformed/noninvertible parent, and missing/wrong target.
- [ ] Focus target and traversal: first direct FocusData child, all six
  traversal values, invalid fallback, empty constructor topology, and later
  synchronized topology.
- [ ] Keyboard dispatch precedence: TextInput, listener-less scripted
  drawable, constrained listener, and false-return propagation remain distinct.
- [ ] Gamepad dispatch precedence: scripted drawable identity/return,
  listener constraints/FIFO actions, mark-needs-advance, and false listener
  return are distinct.
- [ ] Semantic queue constraints: tap/increase/decrease, null semantic
  owner, failed constraints, duplicates, and deferred processing order.
- [ ] ViewModel bind pairing: missing importer/binds, every value family,
  live main-ViewModel seeding, target-to-source update, paired target dirt,
  duplicate occurrence isolation, and one-time drop.
- [ ] Reported-event ownership: wrong/missing event target, duplicate
  reports, event context, host drain isolation, pending-return term, and
  next-frame start delivery.
- [ ] Chained applyEvents completion: events precede ViewModel reports,
  both queues swap before callbacks, chained writes settle in one apply call,
  and the 100-iteration cap is exact.
- [ ] Deferred focus/semantic batches: a callback queued while the current
  batch runs waits for the next processing batch rather than joining it.
- [ ] Fire-trigger live path: nested/relative path, missing context/path,
  wrong property type, duplicate actions, and same-frame state effect.
- [ ] Clone and teardown isolation: registrations, queues, owned bindable
  properties, pending events, invocation payloads, and callback sinks are not
  aliased across cold remounts.
- [ ] Permanent structural ratchets: every forbidden replacement below has
  a checker rule and an injected negative control.

## Structural enforcement required before publication

The checker must reject:

1. filtering valid listener/fire-action occurrences through `filter_map`;
2. replacing authored action, group, fire-action, or notification order with
   a map, set, sort, or reconstructed candidate vector;
3. an invocation enum missing any pinned C++ alternative, or borrowing text
   and gamepad snapshot payloads from the caller;
4. validating bool/number/trigger actions only at perform time, or accepting a
   known wrong concrete input type during import;
5. testing the direct input before a non-empty nested input id;
6. rebuilding listener/action definitions during dispatch or advance;
7. delivering a newly queued focus/semantic callback in the batch currently
   being processed;
8. delivering listener events on the creation frame instead of the next-frame
   `applyEvents` boundary;
9. notifying ViewModel listeners before reported-event listeners or swapping
   only one reporting queue;
10. stopping chained `applyEvents` after one batch or changing its 100-iteration
    bound;
11. dropping pending event/listener queues from the advance return terms or
    removing zero-second keep-going forcing;
12. sending keyboard listener return values as handled, or skipping TextInput
    and listener-less scripted precedence;
13. returning the listener action result from gamepad dispatch, or failing to
    report the scripted drawable that consumed focus-tree dispatch;
14. resolving align-target coordinates from non-pointer invocations, replacing
    preserve-offset delta semantics, or using identity for a noninvertible
    parent;
15. applying a ViewModel action through a whole-context rebind instead of its
    exact source bind plus paired target dirt;
16. resolving a state-machine fire-trigger path at import/construction instead
    of against the live DataContext at perform time;
17. copying pending queues, registrations, callback sinks, or live invocation
    state into a cold clone/remount; and
18. leaving group registrations or the owned bindable property attached after
    occurrence teardown.

## Publication packet

Before the immutable candidate is pushed:

- [ ] all 18 filename-corresponding Rust owners exist and the two mechanical
  ledgers point to them directly;
- [ ] every source/lifecycle/adversarial row above is checked;
- [ ] every structural rule has a passing injected negative control;
- [ ] all focused Rust tests and pinned-C++ differentials are green;
- [ ] one fresh complete non-performance floor is green;
- [ ] exact C++ citations, test names, checker counts, gate counts, trace
  fingerprint, and candidate SHA are recorded in this document and both status
  layers; and
- [ ] performance is not run or used to select implementation work.
