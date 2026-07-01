# Data Binding Graph Imported ViewModel Boolean Shared Mutation Runtime Contract

## Purpose

Close the second scalar gap in imported view-model instance mutation sharing.

C++ mutates the imported `ViewModelInstanceBoolean` object when a runtime
caller sets a boolean source on a file-backed view-model instance. If one state
machine mutates that imported source and another state machine later binds the
same imported view-model instance, the second state machine observes the
changed boolean. Rust should model that fact through the existing imported
context object without making `RuntimeFile` mutable.

## In Scope

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Direct `ViewModelInstanceBoolean.propertyValue` source mutation by
  state-machine data-bind index.
- Sharing the mutated boolean source across two authored state-machine runtime
  instances bound through the same imported context.
- C++ probe comparison through state-machine advance reports.

## Out Of Scope

- Other imported scalar kinds: string, color, enum, symbol-list-index, asset,
  artboard, trigger, and list values.
- Property-name APIs for imported scalar mutation.
- Stable public object handles exposing imported `propertyValue` indexes.
- Reverse propagation, broader update queues, relative/parent/nested lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- Mutating an imported boolean source through one state machine is visible when
  a second state machine binds the same imported view-model instance.
- State-machine transition/advance reports match C++ for both state machines
  after ordinary state-machine advancement.
- Existing imported number and view-model pointer context probes continue to
  pass.
