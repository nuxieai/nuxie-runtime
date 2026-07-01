# Data Binding Graph Imported ViewModel Artboard Shared Mutation Runtime Contract

## Purpose

Close the imported artboard mutation-sharing gap after asset.

C++ mutates the imported `ViewModelInstanceArtboard` object when a runtime
caller sets an artboard source on a file-backed view-model instance. If one
state machine mutates that imported source and another state machine later
binds the same imported view-model instance, the second state machine observes
the changed artboard id. Rust should model that fact through
`RuntimeImportedViewModelInstanceContext` without making `RuntimeFile` mutable.

## In Scope

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Direct `ViewModelInstanceArtboard.propertyValue` source mutation by
  state-machine data-bind index.
- Sharing the mutated artboard source across two authored state-machine runtime
  instances bound through the same imported context.
- C++ probe comparison through state-machine advance and artboard binding
  reports for `BindablePropertyArtboard.propertyValue` targets.

## Out Of Scope

- Other imported value kinds: trigger and list values.
- Nested artboard instancing, remapping, draw propagation, or host advancement.
- Property-name APIs for imported scalar mutation.
- Stable public object handles, reverse propagation, broader update queues,
  relative/parent/nested lookup, and listener-owned data binding.

## Completion Checks

- Mutating an imported artboard source through one state machine is visible
  when a second state machine binds the same imported view-model instance.
- The source and target artboard binding reports match C++ after ordinary
  state-machine advancement for both state machines.
- Existing imported number, boolean, string, color, enum, symbol-list-index,
  asset, and view-model pointer context probes continue to pass.
