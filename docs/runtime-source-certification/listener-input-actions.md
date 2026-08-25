# Listener input-action source certification

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Implementing auditor: root campaign lane

Adversarial review: pending

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

## Result

All 12 pinned out-of-line symbols in these six files have concrete Rust owners.
The literal pass found no missing or incorrect translation. No production code
changed. The historical `TRACKED-GAP` verdicts on the bool/number/trigger rows
therefore describe stale campaign metadata, not missing behavior in these
owners; they must not be promoted until independent adversarial review accepts
this receipt.
