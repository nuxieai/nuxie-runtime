# Data Binding Graph Imported ViewModel List Shared Mutation Runtime Contract

## Purpose

Close the imported list mutation-sharing gap after trigger while keeping list
runtime identity out of scope.

C++ mutates the imported `ViewModelInstanceList` object when runtime code edits
the file-backed view-model instance. For this slice, Rust should model only the
observable list item count through `RuntimeImportedViewModelInstanceContext`.
That lets multiple state-machine instances bound through the same imported
context observe the same list-size fact without making `RuntimeFile` mutable.

## In Scope

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Direct `ViewModelInstanceList` source item-count mutation by state-machine
  data-bind index.
- Sharing the mutated item count with a second authored state-machine runtime
  instance bound through the same imported context.
- C++ probe comparison through existing `BindablePropertyList` source-size and
  target-value reports.

## Out Of Scope

- List item identity, generated list item runtime instances, item-level
  traversal, map-rule selection, layout, and virtualization.
- Reverse conversion for generated lists.
- Property-name APIs for imported list mutation.
- Stable public object handles, reverse propagation, relative/parent/nested
  lookup, listener-owned data binding, and nested artboard propagation.

## Completion Checks

- Mutating an imported list source item count through one state machine is
  visible when a second state machine binds the same imported view-model
  instance.
- The observing state-machine bindable-list report matches C++.
- Existing imported number, boolean, string, color, enum, symbol-list-index,
  asset, artboard, trigger, and view-model pointer context probes continue to
  pass.
