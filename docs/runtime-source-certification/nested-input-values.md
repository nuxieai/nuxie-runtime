# Nested input-value source certification

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Implementing auditor: root campaign lane

Adversarial review: accepted

## `src/animation/nested_bool.cpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `NestedBool::applyValue` | `RuntimeNestedStateMachineInstance::apply_authored_values` | exact | `nested_state_machine_retains_authored_input_slots_and_skips_initial_trigger` and nested instance fixture coverage |
| `NestedBool::nestedValue(bool)` | `ArtboardInstance::set_nested_bool_value` and `set_nested_state_machine_bool` | exact | `upstream_runtime_nested_inputs_fixture_aliases_share_live_occurrences` |
| `NestedBool::nestedValue() const` | `ArtboardInstance::nested_bool_value`; virtual property read in `ArtboardInstance` | exact | live virtual-alias assertions in `artboard::tests` |

The serialized base value is consumed once during nested-state-machine
construction. Subsequent reads and writes address the live child `SMIBool` and
do not rewrite the parent Core property. Missing/wrong input occurrences return
false, and equal values retain `SMIBool::value`'s no-notification edge.

## `src/animation/nested_number.cpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `NestedNumber::applyValue` | `RuntimeNestedStateMachineInstance::apply_authored_values` | exact | `nested_state_machine_retains_authored_input_slots_and_skips_initial_trigger` |
| `NestedNumber::nestedValue(float)` | `ArtboardInstance::set_nested_number_value` and `set_nested_state_machine_number` | exact | live setter/storage assertions in `artboard::tests` |
| `NestedNumber::nestedValue() const` | `ArtboardInstance::nested_number_value`; virtual property read in `ArtboardInstance` | exact | live virtual-alias assertions in `artboard::tests` |

The initialization-only authored float, live child occurrence, equal-value
short circuit, missing-input zero fallback, and parent-storage boundary match
the pinned implementation.

## `src/animation/nested_trigger.cpp`

| C++ symbol | Rust owner | disposition | evidence |
|---|---|---|---|
| `NestedTrigger::fire` | `ArtboardInstance::fire_nested_trigger_input`; callback-property dispatch in `ArtboardInstance` | exact | nested callback forwarding assertions in `artboard::tests` |
| `NestedTrigger::applyValue` | `ArtboardInstance::fire_nested_state_machine_trigger`; `StateMachineInputInstance::fire_trigger` | exact | `nested_state_machine_retains_authored_input_slots_and_skips_initial_trigger`; repeated fire/notification tests |

The callback payload is intentionally discarded, every accepted call reaches
the live child trigger, and construction deliberately does not fire an authored
NestedTrigger. Rust's direct callback dispatch is an ownership adaptation, not
a behavior change.

## Adversarial review

Accepted after independently reading all three pinned C++ translation units,
their generated defaults, and the complete cited Rust nested-instance,
Artboard, input-instance, virtual-property, and clone paths.

- The denominator contains exactly the eight claimed definitions, with ids
  `fa3a144a594ad3b1`, `97181a72fe0c6e52`, `b32cb3e33d68310c`,
  `3f67850810b3bebc`, `3ae30acecdaacb4e`, `565ffb249860caa7`,
  `e7ae25dbd4921cc0`, and `12039581e3974c50`.
- Authored bool/number values are applied once, in authored slot order, during
  child state-machine construction; triggers are skipped. Later getters and
  setters use the live child occurrence, preserve the false/zero fallback and
  equal-value early return, and notify only the child state machine on a real
  value edge. Every trigger call fires the live child trigger and ignores the
  callback payload as pinned C++ does.
- Public cold clone reconstruction reapplies authored bool/number values, while
  transient live clone preserves the child occurrence's current values and
  trigger state. The cited clone and alias tests observe both boundaries rather
  than only inspecting serialized parent properties.
- Focused authored-initialization, fixture-alias, public/transient clone,
  repeated-trigger, missing-target, and notification tests passed with
  `CARGO_INCREMENTAL=0`.

This acceptance covers the eight out-of-line `.cpp` denominator entries only.
It does not certify executable handwritten header bodies, which require their
own denominator coverage.

## Result

All eight pinned out-of-line symbols in these three files have concrete Rust
owners and direct lifecycle evidence. The literal pass found no missing or
incorrect translation. No production code changed. Independent adversarial
review accepted this receipt.
