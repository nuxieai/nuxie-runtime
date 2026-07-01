# Data Binding Graph Imported ViewModel Symbol List Index Shared Mutation Runtime Contract

## Purpose

Close the imported symbol-list-index mutation-sharing gap after enum.

C++ mutates the imported `ViewModelInstanceSymbolListIndex` object when a
runtime caller sets a symbol-list-index source on a file-backed view-model
instance. If one state machine mutates that imported source and another state
machine later binds the same imported view-model instance, the second state
machine observes the changed symbol index. Rust should model that fact through
`RuntimeImportedViewModelInstanceContext` without making `RuntimeFile` mutable.

## In Scope

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Direct `ViewModelInstanceSymbolListIndex.propertyValue` source mutation by
  state-machine data-bind index.
- Sharing the mutated symbol-list-index source across two authored
  state-machine runtime instances bound through the same imported context.
- C++ probe comparison through state-machine advance and symbol-list-index
  binding reports for `BindablePropertyInteger.propertyValue` targets.

## Out Of Scope

- Other imported value kinds: asset, artboard, trigger, and list values.
- Symbol label/key lookup APIs beyond the already imported value index.
- Property-name APIs for imported scalar mutation.
- Stable public object handles exposing imported `propertyValue` indexes.
- Reverse propagation, broader update queues, relative/parent/nested lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- Mutating an imported symbol-list-index source through one state machine is
  visible when a second state machine binds the same imported view-model
  instance.
- The source and target symbol-list-index binding reports match C++ after
  ordinary state-machine advancement for both state machines.
- Existing imported number, boolean, string, color, enum, and view-model
  pointer context probes continue to pass.
