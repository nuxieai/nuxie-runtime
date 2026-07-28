# Listener action, event, and dispatch owner-family closure

This is the production closure checklist for the complete pinned-C++
listener-action, event, focus, keyboard, gamepad, and semantic dispatch family
at `d788e8ec6e8b598526607d6a1e8818e8b637b60c`.

The family is eligible for whole-family review only when every source row
below has:

1. a filename-corresponding Rust owner;
2. a live C++ differential or an explicit source-cited structural proof;
3. its import, ownership, ordering, callback, queue, clone, and teardown
   behavior covered; and
4. an injected negative control for every mechanically enforceable structural
   rule, with temporal behavior covered by focused differentials.

The complete family is one publication unit. No subset is submitted for
acceptance.

Performance is deliberately absent. User direction defers all timing until
every mapped frame-loop code family has been ported.

## Source-to-Rust closure

| Pinned C++ owner | Complete semantics to retain | Focused Rust owner | Required proof |
| --- | --- | --- | --- |
| `src/animation/focus_action_clear.cpp` | ignore the invocation; null machine is a no-op; clear only through the occurrence's existing focus manager | `crates/nuxie-runtime/src/state_machine/focus_action_clear.rs` | null/empty/already-clear/live-focus differential; no manager rediscovery |
| `src/animation/focus_action_target.cpp` | resolve the exact target id; require `Node`; select the first direct `FocusData` child in authored child order; set focus through the occurrence | `crates/nuxie-runtime/src/state_machine/focus_action_target.rs` | missing/wrong target, duplicate direct focus children, and constructor-time unattached target differential |
| `src/animation/focus_action_traversal.cpp` | null machine is a no-op; traversal values 0–5 map to next, previous, up, down, left, right; every other value maps to next | `crates/nuxie-runtime/src/state_machine/focus_action_traversal.rs` | all six values plus invalid-value differential; constructor-time empty-topology case |
| `src/animation/focus_listener_group.cpp` | one occurrence retains exact `FocusData`, listener definition, and machine identities; register once at construction, unregister once at destruction; cache focus/blur membership; callbacks only queue the matching event | `crates/nuxie-runtime/src/state_machine/focus_listener_group.rs` | registration/teardown and focus/blur/duplicate callback ordering differential |
| `src/animation/gamepad_listener_group.cpp` | retain exact focus/listener/machine identities; always register/unregister on the `FocusData`; scripted drawable dispatch precedes listener constraints and reports the dispatched drawable; listener path performs FIFO changes, marks advance, and returns false | `crates/nuxie-runtime/src/state_machine/gamepad_listener_group.rs` | connected/event/disconnected payloads, scripted handled/unhandled, constraints failure, FIFO action, return value, and teardown differentials |
| `src/animation/keyboard_listener_group.cpp` | register keyboard/text channels from either listener flags or scripted-object wants flags; unregister the identical channels; delegate first to the `TextInput` owner, then listener-less scripted dispatch, then listener constraints/actions; listener paths always return false | `crates/nuxie-runtime/src/state_machine/keyboard_listener_group.rs` | key press/repeat/modifiers, owned text, TextInput call-boundary precedence, scripted precedence, constraints failure, listener return, and channel teardown differentials. This row closes the caller only: the text-enabled editing result remains explicitly pending in FL-E's `src/text/text_input.cpp` owner and is not claimed here. |
| `src/animation/listener_action.cpp` | import by `parentKind`: listener actions transfer unique ownership to the current listener importer; transition/state actions transfer to the current layer-component importer; raw kind 3 is canonicalized to Listener by the public accessor; missing/wrong importer fails; execution remains authored FIFO | `crates/nuxie-runtime/src/state_machine/listener_action.rs` | all parent kinds including raw 3, missing importers, duplicates, and failed-middle-action ordering proof |
| `src/animation/scripted_listener_action.cpp` | import/register the source definition once; clone one stateful action occurrence per state-machine instance; copy the ScriptAsset and every custom ScriptInput/DataBind/converter; perform through the occurrence map with `performAction` before legacy `perform`; ordinary protected-call failure is inert and resource failure remains the Rust safety fence; dispose occurrence-owned inputs once | `crates/nuxie-runtime/src/state_machine/scripted_listener_action.rs` | definition-versus-occurrence identity, two cold occurrences, complete cloned-input/DataBind proof below, cold `reinit` plus live-context retry, missing/non-function method behavior, FIFO dispatch, failure isolation, snapshot/cold-clone reset, and teardown |
| `src/animation/listener_align_target.cpp` | only pointer invocation supplies current/previous coordinates; every other invocation uses zeroes; resolve exact `Node`; invert parent world or no-op; preserve-offset applies the local delta, otherwise replace local position | `crates/nuxie-runtime/src/state_machine/listener_align_target.rs` | pointer/non-pointer, preserve/replace, transformed parent, noninvertible parent, missing/wrong target differentials |
| `src/animation/listener_bool_change.cpp` | null input definitions are forward-compatible; wrong concrete direct/nested types reject import; a non-empty nested id wins over the direct id; values 0/1 set false/true and every other value toggles | `crates/nuxie-runtime/src/state_machine/listener_bool_change.rs` | null/wrong/bad-index import, direct/nested precedence, 0/1/toggle, and missing occurrence differentials |
| `src/animation/listener_fire_event.cpp` | resolve the exact event id at perform time; wrong/missing targets are no-ops; report the exact retained event occurrence; ignore the C++ `ListenerInvocation` payload | `crates/nuxie-runtime/src/state_machine/listener_fire_event.rs` | wrong/missing/duplicate event ids, ignored C++ invocation payload, retained Rust facade hit-occurrence metadata, mutate-before-fire and mutate-after-fire live payload, report order, and next-frame delivery differential |
| `src/animation/listener_input_change.cpp` | import requires both current state-machine and artboard importers; validate the resolved nested input first, otherwise the direct state-machine input slot; preserve forward-compatible null slots; delegate parent attachment through `ListenerAction` | `crates/nuxie-runtime/src/state_machine/listener_input_change.rs` | missing importer, nested/direct precedence, bad index, null slot, wrong type, and parent attachment differentials |
| `src/animation/listener_invocation.cpp` | one owned tagged occurrence for pointer, keyboard, text, focus, reported event, ViewModel change, none, gamepad connected/event/disconnected, and semantic payloads; snapshots and strings are owned values | `crates/nuxie-runtime/src/state_machine/listener_invocation.rs` | all eleven variants, owned text/snapshot lifetime, exact accessors, and clone/isolation proof |
| `src/animation/listener_number_change.cpp` | null direct/nested input definitions are forward-compatible; wrong concrete types reject import; nested id wins; exact authored float is assigned | `crates/nuxie-runtime/src/state_machine/listener_number_change.rs` | null/wrong/bad-index import, direct/nested precedence, NaN/infinity/signed-zero, and missing occurrence differentials |
| `src/animation/listener_trigger_change.cpp` | null direct/nested input definitions are forward-compatible; wrong concrete types reject import; nested id wins; nested fire passes `CallbackData(machine, 0)` into `NestedTrigger::fire`, whose pinned implementation ignores the callback data and calls `applyValue`; direct fire targets the occurrence | `crates/nuxie-runtime/src/state_machine/listener_trigger_change.rs` | null/wrong/bad-index import, direct/nested precedence, source-cited ignored-callback structural proof, and repeated-trigger differentials |
| `src/animation/listener_viewmodel_change.cpp` | import takes ownership of the current bindable property; destruction deletes it once; perform resolves the occurrence-local bindable, updates only the target-to-source bind, seeds a ViewModel target from the live main context when required, then dirties the paired source-to-target bind | `crates/nuxie-runtime/src/state_machine/listener_viewmodel_change.rs` | missing importer, every bindable type, no source/target bind, live ViewModel target, exact dirt sink, duplicate occurrences, and teardown proof |
| `src/animation/semantic_listener_group.cpp` | retain exact semantic/listener/machine identities; register/unregister only a non-null semantic owner; tap/increase/decrease callbacks queue only when listener constraints pass | `crates/nuxie-runtime/src/state_machine/semantic_listener_group.rs` | null owner, each resolved-owner callback, constraints failure, FIFO queue, duplicate callback, and teardown differentials. Public `SemanticManager::nodeById` dispatch belongs to the separately pending `semantic_manager.cpp` and whole `state_machine_instance.cpp` rows; this slice does not expose an artboard-local id as that API. |
| `src/animation/state_machine_fire_action.cpp` | import requires the current layer-component importer and appends this exact fire-action occurrence in authored order | `crates/nuxie-runtime/src/state_machine/state_machine_fire_action.rs` | missing importer, state/transition ownership, duplicate occurrence order, and clone/remount proof |
| `src/generated/animation/state_machine_fire_event.cpp` | generated decode/copy retain the event id on this exact fire-action occurrence; perform resolves that id against the live Artboard and reports only a valid Event in authored action order | `crates/nuxie-runtime/src/state_machine/state_machine_fire_event.rs` | decode/copy, live id mutation, wrong/missing target, duplicate report order, and next-frame delivery proof |
| `src/animation/state_machine_fire_trigger.cpp` | import/decode/copy retain the complete DataBind path; perform resolves it only against the live DataContext and fires only a trigger property; missing context/path/property or wrong type is a no-op | `crates/nuxie-runtime/src/state_machine/state_machine_fire_trigger.rs` | nested path, relative path, missing/wrong property, duplicate occurrences, clone/copy, and same-frame trigger differential |

`src/animation/state_machine_listener.cpp` and the relevant sections of
`src/animation/state_machine_instance.cpp` are supporting oracles. Their whole
file rows remain owned by FL-C1 and FL-C5 respectively; this family may cite
and test their action/queue call sites without promoting those rows.
Likewise, `focused_input_dispatch.rs` remains supporting implementation on the
pending whole `state_machine_instance.cpp` row, and
`artboard_component_list_order.rs` remains supporting implementation on the
pending FL-D `artboard_component_list.cpp` row. The public semantic-node-id
lookup remains on the pending global `semantic_manager.cpp` row.

Generated implementation files are intentionally excluded by the mechanical
source-set globs, which enumerate hand-authored C++ owners. The generated
`state_machine_fire_event.cpp` row is therefore a source-correspondence support
row enforced by this checklist and its live differential, not a silently
missing mechanical-ledger entry.

## Scripted-listener transitive owner closure

`ScriptedListenerAction::cloneScriptedObject` is not a leaf method. It calls
the exact C++ methods below, so FL-C4 cannot claim the concrete action owner
while substituting a source-to-target-only hydration shortcut. These are
dependency rows brought forward for this closure; unrelated Artboard/DataBind
settlement remains pending in FL-D.

| Pinned C++ support owner/methods | Required FL-C4 semantics | Focused Rust owner/proof |
| --- | --- | --- |
| `src/assets/script_asset.cpp`: `ScriptAsset::initScriptedObject`; `include/rive/assets/script_asset.hpp`: `OptionalScriptedMethods` | copy the serialized optional-method bitfield to every occurrence; legacy all-bits default remains exact; absent/non-function `convert`, `reverseConvert`, `advance`, and `init` are pass-through/inert rather than rediscovered or called unconditionally | `crates/nuxie-runtime/src/script_asset.rs`, the scripted converter/action instantiation seam, method-mask differentials |
| `src/scripted/scripted_object.cpp`: `ensureScriptInitialized`, `hydrateScriptInputs`, `reinit`, `cloneProperties`, `disposeScriptInputs`, `scriptDispose` | clone every authored custom property in order; clone its complete DataBind and converter; cold construction attempts generator/hydration/init, live DataContext assignment retries immediately, and each occurrence completes a write-free prerequisite preflight before applying inputs in authored order. A later phase-two failure preserves earlier writes, stops later inputs, and skips init; teardown releases the occurrence without aliasing definition state | `crates/nuxie-runtime/src/state_machine/scripted_listener_action.rs`, facade lifecycle glue, cold/live retry and two-occurrence isolation tests |
| `src/data_bind/data_bind.cpp`: `import`, `bind`, `unbind`, `update`, `updateSourceBinding`, `reconcileDirt`, `toTarget`, `toSource`, `sourceToTargetRunsFirst`, `bindsOnce`, `advance` | retain complete flags and target value; last authored DataBind occurrence owns the ScriptInput; ToSource, TwoWay favored order, Once subscription, forward/reverse conversion, rebind reset, target/source self-notification suppression, and source-required converter advance remain occurrence-local | reuse `crates/nuxie-runtime/src/retained_data_bind.rs` inside `crates/nuxie-runtime/src/state_machine/scripted_listener_action.rs`; live direction/rebind/advance differentials |
| `src/data_bind/data_bind_context.cpp`: `copySourcePathIds`, `resolvePath`, `bindFromContext`; the immutable-membership projection of `src/data_bind/data_bind_container.cpp`: construction-time `addDataBind`, `bindDataBindsFromContext`, dirty scheduling in `updateDataBinds`, `advanceDataBinds`, `deleteDataBinds` | preserve full id/name path, main/global/parent resolution, retained source-cell identity, authored occurrence order, same-batch dirty updates, stateful advance after layers, and one-time deletion for the fixed listener/converter occurrence set | occurrence-owned ordered bindings plus retained cell/dirt sinks; nested/global/parent, duplicate, same-frame, and cold-clone proofs. FL-C4 does not promote the whole DataBindContainer row: dynamic add/remove, pending mutation flush, add-after-live-context reconciliation, and membership-changing advance remain pending in FL-D. |
| `src/scripted/scripted_data_converter.cpp`: `clone`, `bindFromContext`, `didHydrateScriptInputs`, `convert`, `reverseConvert`, `advance`, `disposeScriptInputs` | clone a fresh converter table for every converter occurrence, including its ordered custom ScriptInputs/DataBinds; bind it, complete the occurrence-local write-free preflight, then hydrate in authored order before init; gate optional methods; pass through failed/missing ordinary conversion; advance only a bound source, mark the parent bind dirty on `true`, retry failed construction when C++ retries, and never alias repeated converter occurrences | focused scripted-data-converter occurrence owner used by `scripted_listener_action.rs`; custom-input, duplicate-occurrence, method-mask, failure/retry, direction, and same-frame tests |
| `src/script_input_boolean.cpp`, `script_input_number.cpp`, `script_input_color.cpp`, `script_input_string.cpp`, `script_input_trigger.cpp`, `script_input_viewmodel_property.cpp`; the owner-null projection of `script_input_artboard.cpp` | preserve authored order/name/value, exact typed hydration and callback behavior, trigger edge semantics, ViewModel path/live occurrence, parent linkage, clone, and destruction; a later plain DataBind shadows an earlier context bind. Both FL-C4 scripted owners return `component() == nullptr`, so the reachable Artboard projection is authored-id resolution, unresolved prerequisite behavior, clone file/reference asymmetry, hydration/table projection, live numeric updates, and prior-reference retention. | focused typed ScriptInput occurrence representation and tests. Scalar and ViewModel rows move only when their whole lifecycle is proven. The whole `script_input_artboard.cpp` row remains pending with FL-D's Artboard/DataBind settlement. |

The complete transitive checklist is:

FL-C4 reaches only the immutable-membership projection of pinned
`DataBindContainer`: `ScriptedListenerAction::cloneScriptedObject` and each
`ScriptedDataConverter` clone append every authored input bind before the
first update, preserve that occurrence order unchanged, and delete the whole
set with the owner (`scripted_listener_action.cpp:143-160`;
`scripted_object.cpp:558-586`; `scripted_data_converter.cpp:235-280`;
`state_machine_instance.cpp:2072-2082,2141-2199`). Within this fixed set,
Rust's `data_bind_occurrences` is created by those same append operations and
is never mutated before teardown, so its post-layer walk is equivalent to
C++ `advanceDataBinds` visiting `m_dataBinds`.

`DataBindContainer::removeDataBind`, add/remove while `m_isProcessing`,
pending-addition/removal flush, add-after-live-context immediate
reconciliation, dynamic keyframe and ScriptedInterpolator membership, Artboard
clone membership, and advance eligibility after any membership mutation remain
pending under FL-D's `databind.owner` / `databind.queues` rows. The current
family neither implements nor promotes the whole
`src/data_bind/data_bind_container.cpp` row.

- [x] full DataBind flags and target values survive definition import and every
  fresh occurrence clone;
- [x] ToSource, target-first and source-first TwoWay, Once, forward/reverse
  conversion, source-cell rebind, converter reset, and same-value new-cell
  rebind match pinned C++ for the base DataBind and the
  ScriptedDataConverter-owned input graph;
- [x] repeated converter occurrences retain distinct table/input/DataBind
  state even when they share the same source converter global id;
- [x] a ScriptedDataConverter clones, binds, preflights without writes, hydrates
  in authored order, initializes, advances, and disposes its own ordered
  ScriptInputs/DataBinds;
- [x] optional-method bits gate `init`, `convert`, `reverseConvert`, and
  `advance`; missing/non-function methods are inert/pass-through;
- [x] ordinary generator/init/conversion failure leaves the occurrence inert
  and does not abort later authored work; the typed Rust resource fence stays
  terminal; constructor/live-context retry matches C++;
- [x] the six complete scalar/ViewModel ScriptInput owners plus FL-C4's
  owner-null ScriptInputArtboard projection cover authored defaults, typed live
  updates, trigger repetition, authored-id Artboard availability,
  ViewModel path identity, duplicate/last-bind ownership, clone isolation, and
  teardown; and
- [x] every supporting C++ method above has either a live pinned differential
  or a source-cited structural ratchet before publication.

Cross-object live ScriptedObject visitation uses FLR-19. Pinned C++ retains
one wrapper per unique source pointer, installs the live context across the
complete unordered map, and only then begins the init pass; relative key order
inside either pass is not defined. Rust determinizes those visits with the
authored-first unique definition vector while preserving the collection-wide
context barrier and one visit per occurrence. Cold clone traversal and each
per-occurrence generator/hydration sequence keep their separately defined
order (`state_machine_instance.cpp:2072-2082,2886-2913`;
`scripted_listener_action.cpp:154-160`; `scripted_object.cpp:399-437`).

Teardown follows the C++ owner boundary: cloned DataBind occurrences drop and
unbind before either ScriptedObject table-handle map. The Rust field order is
part of this ownership proof and is protected by an injected structural
negative (`state_machine_instance.cpp:2169-2198`;
`data_bind.cpp:239-249,354-369`).

The component-owned ancestry branch of `ScriptInputArtboard` is not reachable
from this family. Pinned `ScriptedListenerAction::component()` and
`ScriptedDataConverter::component()` both return null
(`include/rive/animation/scripted_listener_action.hpp:28`;
`include/rive/scripted/scripted_data_converter.hpp:66`), so
`ArtboardReferencer::findArtboard` cannot perform owner/self/ancestor checks
for either FL-C4 caller. `BindableArtboard`, `ContextValueArtboard`, live
ViewModel-artboard identity, and the full ArtboardReferencer lifecycle remain
pending in FL-D together with `src/script_input_artboard.cpp`; those later
owners can still supply an artboard-valued source to the fixed FL-C4
occurrence, but FL-C4 neither implements nor promotes their whole-file
lifecycle.

The separately pending Formula and OperationViewModel converter subclasses
also remain outside this transitive closure. Pinned C++ leaves each prior
additional source registered after a non-null A-to-B rebind; Rust currently
re-homes that sink. The observable stale-notification/random-reset behavior is
owned explicitly by FL-G04 and the pending FL-D
`data_converter_formula.cpp` / `data_converter_operation_viewmodel.cpp` rows.
FL-C4 does not call that subclass-specific lifetime faithful
(`data_converter_formula.cpp:526-552`;
`data_converter_operation_viewmodel.cpp:48-59`;
`data_bind_context.cpp:56-89`).

## Complete lifecycle and ordering closure

- [x] Every listener/state/transition retains its authored action occurrences
  once, in insertion order. Unsupported or malformed occurrences follow the
  pinned import result; Rust may not silently compact a valid nullable slot.
- [x] `StateMachineListener::performChanges` visits the retained action list
  exactly once in FIFO order for each invocation. An action failure or no-op
  does not rebuild or reorder the list.
- [x] Focus, keyboard, gamepad, and semantic group occurrences retain exact
  target/listener/machine identities and register/unregister symmetrically.
- [x] `ListenerInvocation` represents all eleven C++ alternatives with owned
  strings and snapshots. Consumers branch on the retained alternative rather
  than reconstructing payloads from ambient state.
- [x] Listener input actions validate direct and nested definition types at
  import, preserve forward-compatible null definitions, and use nested-id
  precedence at perform time.
- [x] Fire-event actions enqueue exact event identities. Rust re-resolves that
  identity against the live Artboard at listener delivery and host observation,
  preserving C++ `EventReport(Event*)` semantics without a self-referential
  Rust borrow. Public host draining does not consume the private
  listener-delivery queue.
- [x] `applyEvents` updates DataBinds, swaps both reporting queues, delivers
  events before ViewModel notifications, and drains chained notifications for
  at most 100 iterations in the same call.
- [x] Focus and semantic callbacks enqueue occurrence-owned records and mark
  the machine for advance. Processing moves and clears the current batch
  before FIFO action execution, so callbacks created during processing wait
  for the next batch/frame.
- [x] New-frame processing order remains focus events, semantic events, then
  reported event/ViewModel notifications before data-binding and layer
  advance.
- [x] `advance(0)` preserves C++'s forced keep-going behavior and the facade
  return includes pending reported-event and listener-ViewModel queues.
- [x] State/transition fire actions execute in authored order and observe
  their exact occurrence code. Fire-trigger paths resolve against the live
  DataContext at perform time.
- [x] ScriptedObject construction preserves the C++ phase boundary. Every
  occurrence first clones and reinitializes cold. If the Artboard DataContext
  was already live, the constructor-wide context barrier and deferred listener
  hydration run before converter binding; a later inherited live pass follows.
  An unbound occurrence instead receives C++'s unconditional second cold retry
  before any later context bind
  (`state_machine_instance.cpp:2072-2082`;
  `artboard.cpp:2844-2856`).
- [x] One File registration owns one shared scripting VM and program map across
  the root Artboard and ScriptInputArtboard children. The detached-ViewModel
  tail runs exactly once after each root StateMachineInstance host advance,
  never for a static/plain Artboard-only call, and its boolean result is
  discarded (`file.cpp:694-746`; `state_machine_instance.cpp:2607-2662`;
  `artboard.cpp:914-923`).
- [x] A fresh remount shares immutable definitions but reconstructs every
  group, queue, registration, callback target, and mutable invocation
  occurrence with empty pending state. Rust's explicit public `Clone`
  adaptation instead snapshots owned pending values and cursors into
  non-aliased queues; pinned C++ has no StateMachineInstance copy constructor.
- [x] Destruction unregisters groups and releases owned bindable/action
  occurrences once, in the same owner order, without leaving callback sinks
  attached.

## Adversarial publication review

- [x] Parent-kind import ownership: listener, transition, and state action
  parents attach to the correct current importer; raw 3 follows C++'s
  Listener fallback; missing/wrong importers fail without leaking or
  reparenting.
- [x] Authored FIFO with duplicates: duplicate actions, fire actions,
  listener groups, and notifications retain insertion order and occurrence
  identity; no map/set replacement is permitted.
- [x] Malformed input actions: bool, number, and trigger cover wrong type,
  bad direct index, bad nested id, forward-compatible null slots, and nested-id
  precedence.
- [x] Bool and number edge values: bool 0/1/toggle plus number NaN,
  infinities, and signed zero match the pinned setter behavior.
- [x] Trigger callback identity: pinned C++ supplies the current machine and
  value zero, while pinned `NestedTrigger::fire` ignores both; the Rust
  direct-callback path must prove the same repeated-fire effect without
  inventing observable callback state. Direct fires retain their occurrence
  semantics.
- [x] All invocation alternatives: pointer, keyboard, text, focus, event,
  ViewModel, none, three gamepad forms, and semantic payloads are live-tested.
- [x] Owned invocation payloads: text and gamepad snapshots survive caller
  mutation/drop and isolate cloned occurrences.
- [x] Align target matrix: pointer versus non-pointer, preserve versus
  replace, transformed/noninvertible parent, and missing/wrong target.
- [x] Focus target and traversal: first direct FocusData child, all six
  traversal values, invalid fallback, empty constructor topology, and later
  synchronized topology.
- [x] Keyboard dispatch precedence: the TextInput owner call boundary,
  listener-less scripted drawable, constrained listener, and false-return
  propagation remain distinct. The current text-enabled editing result is not
  claimed: full editable-text behavior remains pending in the mapped FL-E
  `text_input.cpp` owner.
- [x] Gamepad dispatch precedence: scripted drawable identity/return,
  listener constraints/FIFO actions, mark-needs-advance, and false listener
  return are distinct.
- [x] Semantic queue constraints: tap/increase/decrease, null semantic
  owner, failed constraints, duplicates, and deferred processing order.
- [x] ViewModel bind pairing: missing importer/binds, every value family,
  live main-ViewModel seeding, target-to-source update, paired target dirt,
  duplicate occurrence isolation, and one-time drop.
- [x] Reported-event ownership: wrong/missing event target, duplicate reports,
  ignored C++ ListenerInvocation payload, retained Rust facade hit-occurrence
  metadata, live payload changes on both sides of fire, host drain isolation,
  pending-return term, and next-frame start delivery.
- [x] Chained applyEvents completion: events precede ViewModel reports,
  both queues swap before callbacks, chained writes settle in one apply call,
  and the 100-iteration cap is exact.
- [x] Deferred focus/semantic batches: a callback queued while the current
  batch runs waits for the next processing batch rather than joining it.
- [x] Fire-trigger live path: nested/relative path, missing context/path,
  wrong property type, duplicate actions, and same-frame state effect.
- [x] Clone and teardown isolation: registrations, queues, owned bindable
  properties, pending events, invocation payloads, and callback sinks are not
  aliased across either Rust snapshots or cold remounts; only the Rust
  snapshot retains pending values.
- [x] Permanent structural ratchets: mechanically recognizable replacement shapes
  have checker ratchets with injected negatives; temporal queue, clone, and
  lifecycle semantics have pinned differentials or source-cited behavioral
  tests.

## Permanent enforcement required before publication

Each numbered regression below is permanently rejected. Items with a
mechanically stable syntax shape use checker ratchets and injected negatives;
the queue-order, frame-timing, and lifecycle items use focused behavioral
differentials because a regex cannot truthfully prove them:

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
    occurrence teardown; and
19. erasing a terminal Rust resource-limit error while consuming ordinary
    protected-call failures like pinned C++.
20. discarding DataBind flags or filtering a valid ToSource ScriptInput bind;
21. sharing one mutable ScriptedDataConverter table across two cloned bind
    occurrences or deduplicating repeated converter occurrences by global id;
22. calling a scripted converter method whose serialized optional-method bit
    is absent, or treating an absent/non-function method as a hard failure;
23. hydrating/initializing a ScriptedDataConverter before all of its own
    ScriptInputs and DataBinds are cloned, bound, and validated atomically; and
24. subscribing a Once bind to its primary source or failing to reset a
    stateful converter when the retained source-cell identity changes.

## Publication packet

Before the immutable candidate is pushed:

- [x] all 19 hand-authored primary filename-corresponding Rust owners plus the
  generated `state_machine_fire_event.cpp` support owner exist, and the two
  mechanical ledgers point to every in-scope hand-authored owner directly;
- [x] every transitive support method is mapped to the focused Rust path
  actually used by the action while unrelated whole FL-D rows remain pending;
- [x] every source/lifecycle/adversarial row above is checked;
- [x] every structural rule has a passing injected negative control;
- [x] all focused Rust tests and pinned-C++ differentials are green;
- [x] one fresh complete non-performance floor is green;
- [x] exact C++ citations, test names, checker counts, gate counts, and the
  trace receipt are recorded in this document and both status layers; the
  immutable candidate SHA accompanies the external review request; and
- [x] performance is not run or used to select implementation work.

## Candidate evidence

The focused proof set includes:

- `state_machine_scheduled_listener_fire_events_match_cpp_probe`,
  `state_machine_scheduled_listener_input_changes_match_cpp_probe`, and both
  state-machine fire-trigger C++ comparisons;
- the keyboard/text/gamepad leaf-to-parent precedence tests, malformed mixed
  Event/ViewModel registration test, semantic/focus duplicate and deferred
  batch tests, live-event mutation and pre-advance host-drain tests;
- the complete align-target branch matrix, nested bool/number/trigger
  precedence and edge-value tests, ViewModel bind-pairing tests, and cold
  clone/snapshot isolation tests;
- `scripted_listener_failure_is_swallowed_and_later_actions_still_run` plus
  `pointer_subcycles_reset_script_budgets_and_roll_back_overflowing_host_work`,
  proving ordinary C++ protected-call failures are consumed while Rust's
  terminal resource fence remains fail-closed;
- `prebound_constructor_hydrates_deferred_listener_before_converter_binding`
  and
  `post_constructor_context_bind_runs_converter_before_live_listener_init`
  lock the two constructor/context orders;
- `file_vm_tail_requires_a_root_state_machine_and_runs_once_per_host_frame`
  plus `shared_file_vm_contributes_one_host_frame_tail` prove shared File-VM
  identity and the single root-state-machine frame tail; and
- every FL-C4 structural ratchet with an injected negative control, including
  permanent guards against erasing a typed terminal resource error or freezing
  a retained Event payload at fire time.

Final production corrections are
`4f10f3ca081006774fb1c55dbc03a0335bf0844c` and
`97b5eefa415bfd1785f8be60ca6fd23024df515e`. They restore the prebound and
unbound constructor orders, Artboard-before-StateMachine context binding, one
shared File VM across root and child occurrences, and one ignored-result
detached-ViewModel tail at the root StateMachineInstance frame boundary. The
final read-only pinned-C++ audit found no remaining behavior or ownership
blocker in this closure.

The fresh non-performance receipt bound to production source
`97b5eefa415bfd1785f8be60ca6fd23024df515e` is:

- runtime 665 / 665 and public facade 146 / 146;
- probe-armed workspace including 759 / 759 pinned-C++ comparisons;
- ordinary and scripted golden each 317 / 317 entries and 647 / 647 segments,
  zero divergences, including `data_viz_demo` and `db_health_tracker`;
- same-runner pixels 1,468 / 1,468, 1,370 byte-exact, zero divergences, and
  zero gated rows;
- C API, native Apple, browser WebGPU-only, lint, format, and diff checks;
- committed-tree size 8,151,336 bytes without scripting and 9,252,072 bytes
  with scripting, both below 9 MiB;
- Apple XCFramework build/package/ABI/header/C/Swift checks, checksum
  `a4617edff64f19cbc353c579babb3c99e6a48a539644d04a65235a76f7913e1f`;
- a source-bound trace with all 18 landmarks and exact runner provenance; and
- structural checker 41 / 41 with every injected negative control green.

No performance measurement was run. Every FL-C4 file/member row remains
pending or pending-verification until an independent whole-family verdict.
The exact immutable evidence-commit SHA is supplied with that review request
instead of being self-referenced by this fingerprinted closure.
