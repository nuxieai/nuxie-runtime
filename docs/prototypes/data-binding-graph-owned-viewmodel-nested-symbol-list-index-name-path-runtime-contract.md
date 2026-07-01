# Data Binding Graph Owned ViewModel Nested Symbol List Index Name Path Runtime Contract

## Purpose

Admit generated owned nested symbol-list-index name-path mutation as the next
sibling of the nested number, boolean, string, color, and enum slices.

C++ can create an owned `ViewModelInstance`, resolve the generated child with
`ViewModelInstanceRuntime::propertyViewModel("child")`, look up the child's
`symbol` value with `ViewModelInstance::propertyValue("symbol")`, mutate its
`propertyValue`, and bind that owned instance to a state machine. For this Rust
slice, `RuntimeOwnedViewModelInstance` can store direct symbol-list-index values
on generated owned view-model children and resolve a slash-separated
symbol-list-index property path through generated view-model pointers.

## In Scope

- Generated owned view-model children reached through
  `ViewModelPropertyViewModel` properties.
- Nested symbol-list-index paths whose last segment is a direct
  `ViewModelPropertySymbolListIndex.name` on the generated child.
- `RuntimeOwnedViewModelInstance::set_symbol_list_index_by_property_name_path`.
- Source-to-target graph binding for symbol-list-index data-bind paths longer
  than the root scalar shape, including the existing to-string converter path.
- A C++ probe flag that resolves the parent path with `propertyViewModel`, then
  calls `propertyValue("symbol")` and mutates the resulting
  `ViewModelInstanceSymbolListIndex`.

## Out Of Scope

- Nested asset, artboard, trigger, list, or view-model scalar value mutation.
- Imported-intermediate nested scalar reads or writes.
- Shared imported-instance mutation, stable public object handles, reverse
  propagation, broader update queues, relative/parent/name-based lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- A generated owned nested symbol-list-index can be mutated by name path before
  binding.
- The matching C++ probe reports the same state-machine transition behavior
  after resolving the parent path, mutating the child symbol value, and
  advancing the state machine.
- Existing owned root scalar and nested number/boolean/string/color/enum probes
  continue to pass.
