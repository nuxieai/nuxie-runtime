# Data Binding Graph Imported ViewModel Number Shared Mutation Runtime Contract

## Purpose

Close the first scalar gap in imported view-model instance mutation sharing.

C++ mutates the imported `ViewModelInstanceNumber` object when a runtime caller
sets a number source on a file-backed view-model instance. If one state machine
mutates that imported source and another state machine later binds the same
imported view-model instance, the second state machine observes the changed
number. Rust should model that fact without making `RuntimeFile` mutable.

## In Scope

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Direct `ViewModelInstanceNumber.propertyValue` source mutation by
  state-machine data-bind index.
- Sharing the mutated number source across two authored state-machine runtime
  instances bound through the same imported context.
- C++ probe comparison through existing number binding reports.

## Out Of Scope

- Other imported scalar kinds: boolean, string, color, enum,
  symbol-list-index, asset, artboard, trigger, and list values.
- Property-name APIs for imported scalar mutation.
- Stable public object handles exposing imported `propertyValue` indexes.
- Reverse propagation, broader update queues, relative/parent/nested lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- Mutating an imported number source through one state machine is visible when
  a second state machine binds the same imported view-model instance.
- The source and target number binding reports match C++ after ordinary
  state-machine advancement for both state machines.
- Existing imported view-model pointer relink context probes continue to pass.
