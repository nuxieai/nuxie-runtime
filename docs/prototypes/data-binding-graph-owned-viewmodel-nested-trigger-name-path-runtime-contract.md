# Data Binding Graph Owned ViewModel Nested Trigger Name Path Runtime Contract

## Purpose

Admit generated owned nested trigger name-path mutation as the next sibling
after the nested artboard path slice.

C++ can create an owned `ViewModelInstance`, resolve a generated child with
`ViewModelInstanceRuntime::propertyViewModel("child")`, look up the child's
`fire` value with `ViewModelInstance::propertyValue("fire")`, mutate its raw
`propertyValue`, and bind that owned instance to a state machine. For this Rust
slice, `RuntimeOwnedViewModelInstance` can store direct trigger values on
generated owned view-model children and resolve a slash-separated trigger
property path through generated view-model pointers.

## In Scope

- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Nested trigger paths whose last segment is a direct trigger property `name`
  on the generated child.
- `RuntimeOwnedViewModelInstance::set_trigger_by_property_name_path`.
- Source-to-target graph binding for trigger data-bind paths longer than the
  root scalar shape.
- A C++ probe flag that resolves the parent path with `propertyViewModel`,
  then calls `propertyValue("fire")` and mutates the resulting
  `ViewModelInstanceTrigger`.

## Out Of Scope

- Trigger firing APIs, listener-owned dispatch, callback dispatch, or trigger
  reset behavior beyond the existing raw value comparison.
- Nested list or view-model value mutation.
- Imported-intermediate nested scalar reads or writes.
- Shared imported-instance mutation, stable public object handles, reverse
  propagation, broader update queues, relative/parent/name-based lookup, and
  nested artboard propagation.

## Completion Checks

- A generated owned nested trigger can be mutated by name path before binding.
- The matching C++ probe reports the same state-machine transition behavior
  after resolving the parent path, mutating the child trigger value, and
  advancing the state machine.
- Existing owned root scalar and nested
  number/boolean/string/color/enum/symbol-list-index/asset/artboard probes
  continue to pass.
