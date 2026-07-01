# Data Binding Graph Owned ViewModel Nested Number Name Path Runtime Contract

## Purpose

Admit the first generated owned nested scalar name-path mutation without
opening the full nested scalar storage model.

C++ can create an owned `ViewModelInstance`, look up a nested generated number
with `ViewModelInstanceRuntime::propertyNumber("child/amount")`, mutate it,
and bind that owned instance to a state machine. For this Rust slice,
`RuntimeOwnedViewModelInstance` can store direct number values on generated
owned view-model children and resolve a slash-separated number property path
through generated view-model pointers.

## In Scope

- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Nested number paths whose last segment is a direct
  `ViewModelPropertyNumber.name` on the generated child.
- `RuntimeOwnedViewModelInstance::set_number_by_property_name_path`.
- Source-to-target graph binding for number data-bind paths longer than the
  root scalar shape, such as `[rootViewModel, childProperty, amountProperty]`.
- A C++ probe flag that calls
  `ViewModelInstanceRuntime::propertyNumber("child/amount")`.

## Out Of Scope

- Nested boolean, string, color, enum, symbol-list-index, asset, artboard,
  trigger, list, or view-model scalar value mutation.
- Imported-intermediate nested scalar reads or writes.
- Shared imported-instance mutation, stable public object handles, reverse
  propagation, broader update queues, relative/parent/name-based lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- A generated owned nested number can be mutated by name path before binding.
- The matching C++ public `propertyNumber(namePath)` probe reports the same
  source and target values after state-machine advance.
- Existing owned root scalar and view-model pointer probes continue to pass.
