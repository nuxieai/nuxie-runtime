# Data Binding Graph Imported ViewModel Trigger Shared Mutation Runtime Contract

## Purpose

Close the imported trigger mutation-sharing gap after artboard.

C++ mutates the imported `ViewModelInstanceTrigger` object when a runtime
caller sets a trigger source on a file-backed view-model instance. Trigger
counts can be consumed/reset by runtime advancement, so this slice proves the
shared mutation before any observing state machine has a chance to reset the
source. Rust should model that imported `propertyValue` count through
`RuntimeImportedViewModelInstanceContext` without making `RuntimeFile` mutable.

## In Scope

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Direct `ViewModelInstanceTrigger.propertyValue` source mutation by
  state-machine data-bind index.
- Sharing the mutated trigger source with a second authored state-machine
  runtime instance bound through the same imported context before trigger reset.
- C++ probe comparison through the observing state machine's ordinary advance.

## Out Of Scope

- Imported list values.
- Trigger firing APIs, listener/callback dispatch, and event side effects.
- Full trigger reset/update-queue parity beyond the existing ordinary advance
  behavior used by the probe.
- Property-name APIs for imported scalar mutation.
- Stable public object handles, reverse propagation, relative/parent/nested
  lookup, listener-owned data binding, and nested artboard propagation.

## Completion Checks

- Mutating an imported trigger source through one state machine is visible when
  a second state machine binds the same imported view-model instance before
  trigger reset.
- The observing state-machine advance report matches C++.
- Existing imported number, boolean, string, color, enum, symbol-list-index,
  asset, artboard, and view-model pointer context probes continue to pass.
