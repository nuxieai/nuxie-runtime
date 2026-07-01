# Data Binding Graph Owned ViewModel Nested Boolean Name Path Runtime Contract

## Purpose

Admit the generated owned nested boolean name-path mutation that sits directly
beside the nested number slice.

C++ can create an owned `ViewModelInstance`, look up a nested generated boolean
with `ViewModelInstanceRuntime::propertyBoolean("child/enabled")`, mutate it,
and bind that owned instance to a state machine. For this Rust slice,
`RuntimeOwnedViewModelInstance` can store direct boolean values on generated
owned view-model children and resolve a slash-separated boolean property path
through generated view-model pointers.

## In Scope

- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Nested boolean paths whose last segment is a direct
  `ViewModelPropertyBoolean.name` on the generated child.
- `RuntimeOwnedViewModelInstance::set_boolean_by_property_name_path`.
- Source-to-target graph binding for boolean data-bind paths longer than the
  root scalar shape, such as `[rootViewModel, childProperty, enabledProperty]`.
- A C++ probe flag that calls
  `ViewModelInstanceRuntime::propertyBoolean("child/enabled")`.

## Out Of Scope

- Nested string, color, enum, symbol-list-index, asset, artboard, trigger,
  list, or view-model scalar value mutation.
- Imported-intermediate nested scalar reads or writes.
- Shared imported-instance mutation, stable public object handles, reverse
  propagation, broader update queues, relative/parent/name-based lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- A generated owned nested boolean can be mutated by name path before binding.
- The matching C++ public `propertyBoolean(namePath)` probe reports the same
  state-machine transition behavior after state-machine advance.
- Existing owned root scalar, nested number, and view-model pointer probes
  continue to pass.
