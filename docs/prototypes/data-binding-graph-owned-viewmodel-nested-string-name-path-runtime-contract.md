# Data Binding Graph Owned ViewModel Nested String Name Path Runtime Contract

## Purpose

Admit generated owned nested string name-path mutation as the next sibling of
the nested number and boolean slices.

C++ can create an owned `ViewModelInstance`, look up a nested generated string
with `ViewModelInstanceRuntime::propertyString("child/label")`, mutate it,
and bind that owned instance to a state machine. For this Rust slice,
`RuntimeOwnedViewModelInstance` can store direct string values on generated
owned view-model children and resolve a slash-separated string property path
through generated view-model pointers.

## In Scope

- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Nested string paths whose last segment is a direct
  `ViewModelPropertyString.name` on the generated child.
- `RuntimeOwnedViewModelInstance::set_string_by_property_name_path`.
- Source-to-target graph binding for string data-bind paths longer than the
  root scalar shape, such as `[rootViewModel, childProperty, labelProperty]`.
- A C++ probe flag that calls
  `ViewModelInstanceRuntime::propertyString("child/label")`.

## Out Of Scope

- Nested color, enum, symbol-list-index, asset, artboard, trigger, list, or
  view-model scalar value mutation.
- Imported-intermediate nested scalar reads or writes.
- Shared imported-instance mutation, stable public object handles, reverse
  propagation, broader update queues, relative/parent/name-based lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- A generated owned nested string can be mutated by name path before binding.
- The matching C++ public `propertyString(namePath)` probe reports the same
  state-machine transition behavior after state-machine advance.
- Existing owned root scalar, nested number/boolean, and view-model pointer
  probes continue to pass.
