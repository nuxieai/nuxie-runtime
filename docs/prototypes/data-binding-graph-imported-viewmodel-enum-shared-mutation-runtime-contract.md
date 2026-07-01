# Data Binding Graph Imported ViewModel Enum Shared Mutation Runtime Contract

## Purpose

Close the next imported scalar mutation-sharing gap after color.

C++ mutates the imported `ViewModelInstanceEnum` object when a runtime caller
sets an enum source on a file-backed view-model instance. If one state machine
mutates that imported source and another state machine later binds the same
imported view-model instance, the second state machine observes the changed
enum index. Rust should model that import-time identity fact through
`RuntimeImportedViewModelInstanceContext` without making `RuntimeFile` mutable.

## In Scope

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Direct `ViewModelInstanceEnum.propertyValue` source mutation by
  state-machine data-bind index.
- Sharing the mutated enum source across two authored state-machine runtime
  instances bound through the same imported context.
- C++ probe comparison through state-machine advance and enum binding reports.

## Out Of Scope

- Other imported value kinds: symbol-list-index, asset, artboard, trigger, and
  list values.
- Enum label/key lookup APIs beyond the already-imported enum metadata.
- Property-name APIs for imported scalar mutation.
- Stable public object handles exposing imported `propertyValue` indexes.
- Reverse propagation, broader update queues, relative/parent/nested lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- Mutating an imported enum source through one state machine is visible when a
  second state machine binds the same imported view-model instance.
- The source and target enum binding reports match C++ after ordinary
  state-machine advancement for both state machines.
- Existing imported number, boolean, string, color, and view-model pointer
  context probes continue to pass.
