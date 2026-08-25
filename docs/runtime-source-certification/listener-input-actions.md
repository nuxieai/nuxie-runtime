# Listener input-action source certification

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Implementing auditor: root campaign lane

Adversarial review: accepted

## `include/rive/animation/listener_action.hpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ListenerAction::matchesScheduledOccurrence` | `perform_scheduled_listener_actions` flags predicate | exact | `upstream_listener_action_flag_decode_occurrence_and_import_routing_matrix` |
| `ListenerAction::parentKind` | `nuxie_binary::importers::state_machine_layer_importer` parent-kind decode | exact | `listener_parent_kind_requires_owner_and_raw_three_falls_back_to_listener`; `upstream_listener_action_flag_decode_occurrence_and_import_routing_matrix` |

Occurrence matching reads only bit zero and compares it to the raw at-start or
at-end value. Parent-kind decoding reads bits one and two, maps `0..=2`
literally, and canonicalizes reserved raw `3` to Listener.

## `include/rive/animation/listener_input_change.hpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ListenerInputChange::validateInputType` default | `RuntimeScheduledListenerAction::validates_for_import` default branch | exact | listener-action import matrix for non-specialized actions |
| `ListenerInputChange::validateNestedInputType` default | `RuntimeScheduledListenerAction::validates_for_import` default branch | exact | listener-action import matrix for non-specialized actions |

The base virtuals accept every pointer, including null. Rust specializes the
same bool/number/trigger subclasses and returns true for all remaining action
kinds, preserving the base default rather than applying an invented type gate.

## `src/animation/listener_action.cpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ListenerAction::import` | `nuxie_binary::importers::state_machine_layer_importer` routes listener-, transition-, and state-owned actions; `RuntimeScheduledListenerAction::from_imported` builds the live occurrence | exact | `listener_parent_kind_requires_owner_and_raw_three_falls_back_to_listener`; `upstream_listener_action_flag_decode_occurrence_and_import_routing_matrix`; binary `cpp_import` action comparison |

The importer's missing-owner rejection, reserved raw parent-kind fallback,
unique ownership transfer, and superclass/import validation order are preserved
at the Rust graph-construction boundary. Rust represents ownership with graph
vectors rather than `unique_ptr`, without retaining an extra action occurrence.

## `src/animation/listener_input_change.cpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ListenerInputChange::import` | `RuntimeListenerInputTarget::from_object`; `RuntimeListenerInputTarget::validates_for_import`; `RuntimeScheduledListenerAction::validates_for_import` | exact | `import_validation_uses_nested_type_then_forward_compatible_direct_slot`; `every_wrong_typed_listener_input_action_rejects_import`; `every_out_of_range_listener_input_slot_is_retained_as_a_nullable_noop` |

The Rust validation preserves the subtle precedence exactly: an authored
`nestedInputId` controls validation only when it resolves to a `NestedInput`;
otherwise validation falls back to the direct input slot. Missing direct slots
remain valid for forward compatibility, while a resolved wrong concrete input
type rejects import.

## `src/animation/listener_bool_change.cpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ListenerBoolChange::validateInputType` | `RuntimeListenerInputTarget::validates_for_import(..., "StateMachineBool", ...)` | exact | wrong-type and nullable-slot import matrix |
| `ListenerBoolChange::validateNestedInputType` | `RuntimeListenerInputTarget::validates_for_import(..., ..., "NestedBool")` | exact | nested/direct precedence test |
| `ListenerBoolChange::perform` | `RuntimeListenerBoolChange::perform`; `StateMachineInputInstance::apply_listener_bool_change`; `ArtboardInstance::apply_listener_nested_bool_change` | exact | `ordinary_listener_actions_read_live_core_fields_at_perform_time`; `scheduled_direct_input_actions_mark_only_genuine_owner_changes`; nested-input forwarding tests |

The authored `0=false`, `1=true`, other=toggle switch is literal. A nonempty
nested id retains perform-time precedence, unresolved targets are no-ops, and
only a genuine direct-input value edge publishes the owning state-machine
advance notification, matching `SMIBool::value`/`SMIInput::valueChanged`.

## `src/animation/listener_number_change.cpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ListenerNumberChange::validateInputType` | `RuntimeListenerInputTarget::validates_for_import(..., "StateMachineNumber", ...)` | exact | wrong-type and nullable-slot import matrix |
| `ListenerNumberChange::validateNestedInputType` | `RuntimeListenerInputTarget::validates_for_import(..., ..., "NestedNumber")` | exact | nested/direct precedence test |
| `ListenerNumberChange::perform` | `RuntimeListenerNumberChange::perform`; `StateMachineInputInstance::set_number`; `ArtboardInstance::set_nested_number_value` | exact | `scheduled_direct_input_actions_mark_only_genuine_owner_changes`; nested number forwarding tests |

The live authored float is read at perform time. Direct and nested setters both
retain C++'s equal-value early return and value-change notification boundary.

## `src/animation/listener_trigger_change.cpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ListenerTriggerChange::validateInputType` | `RuntimeListenerInputTarget::validates_for_import(..., "StateMachineTrigger", ...)` | exact | wrong-type and nullable-slot import matrix |
| `ListenerTriggerChange::validateNestedInputType` | `RuntimeListenerInputTarget::validates_for_import(..., ..., "NestedTrigger")` | exact | nested/direct precedence test |
| `ListenerTriggerChange::perform` | `RuntimeListenerTriggerChange::perform`; `StateMachineInputInstance::fire_trigger`; `ArtboardInstance::fire_nested_trigger_input` | exact | `scheduled_direct_input_actions_mark_only_genuine_owner_changes`; repeated nested-trigger forwarding tests |

Rust does not materialize C++'s `CallbackData(stateMachineInstance, 0)` for the
nested branch because pinned `NestedTrigger::fire` discards the payload and
unconditionally invokes `applyValue`. Repeated firing and the child
state-machine notification boundary remain observable and exact.

## `src/animation/listener_fire_event.cpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `ListenerFireEvent::perform` | `RuntimeListenerFireEvent::perform`; `perform_scheduled_listener_actions` event append | exact | `ordinary_listener_actions_read_live_core_fields_at_perform_time`; state-machine event tests; listener event-context tests |

The event id is resolved against the live Artboard at perform time, wrong or
non-Event targets are ignored, the triggering `ListenerInvocation` context is
not copied, and a valid Event is appended to the state-machine report queue.

## Adversarial review

Accepted after independently reading all six pinned C++ translation units, the
generated defaults they consume, and the complete cited Rust import, owner,
dispatch, input-instance, nested-target, and event-report paths.

- The `.cpp` denominator contains exactly the 12 claimed definitions, with ids
  `2975600a9eb9652b`, `798fa1859dfed913`, `737a5b474044f848`,
  `f0ce5a4caa5300b2`, `8138c398a6f93ed7`, `bfa7688955cefaf5`,
  `5fecfbf3769cdd76`, `83e6402b9022a640`, `0a1c78d1a4756c05`,
  `06192b31de61ff1b`, `ea3b93b8175ae253`, and `7fcefb55549b75de`.
- A header-aware backcheck adds four executable handwritten definitions:
  `ListenerAction::matchesScheduledOccurrence`, `ListenerAction::parentKind`,
  and the two default-true `ListenerInputChange` validation virtuals. Their bit
  masking, reserved-value fallback, and permissive default are also exact.
- Parent-kind routing, owner lookup, action attachment, nested-before-direct
  validation, generated defaults, bool switch behavior, live-field reads,
  equal-value notification edges, repeated trigger behavior, and live event-id
  resolution match the pinned order and side effects. Missing owners fail;
  missing direct input slots and unresolved perform-time targets remain the
  intended no-ops.
- The Rust event-context facade can attach occurrence metadata after the source
  action reports an event. That metadata is not copied from C++
  `ListenerInvocation`, and the certified `RuntimeListenerFireEvent::perform`
  path still creates the source-equivalent context-free event. It is therefore
  an explicit host boundary, not a translation of this source symbol.
- Focused listener import, live-field, direct-input notification, nested-input,
  repeated-trigger, and event-context tests passed with
  `CARGO_INCREMENTAL=0`.

This acceptance covers the 12 out-of-line `.cpp` denominator entries and the
four handwritten header bodies listed above. It does not certify other
executable handwritten header bodies elsewhere in the runtime; those require
the campaign's header-aware denominator.

## Result

All 16 pinned executable definitions in these six source files and two headers
have concrete Rust owners. The literal pass found no missing or incorrect
translation. No production code changed. The historical `TRACKED-GAP` verdicts
on the bool/number/trigger rows therefore describe stale campaign metadata, not
missing behavior in these owners. Independent adversarial review accepted this
receipt.
