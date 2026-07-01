# Data Binding Graph Imported ViewModel Asset Shared Mutation Runtime Contract

## Purpose

Close the imported asset mutation-sharing gap after symbol-list-index.

C++ mutates the imported `ViewModelInstanceAssetImage` object when a runtime
caller sets an asset source on a file-backed view-model instance. If one state
machine mutates that imported source and another state machine later binds the
same imported view-model instance, the second state machine observes the
changed asset id. Rust should model that fact through
`RuntimeImportedViewModelInstanceContext` without making `RuntimeFile` mutable.

## In Scope

- File-backed imported view-model contexts represented by
  `RuntimeImportedViewModelInstanceContext`.
- Direct `ViewModelInstanceAssetImage.propertyValue` source mutation by
  state-machine data-bind index.
- Sharing the mutated asset source across two authored state-machine runtime
  instances bound through the same imported context.
- C++ probe comparison through state-machine advance and asset binding reports
  for `BindablePropertyAsset.propertyValue` targets.

## Out Of Scope

- Other imported value kinds: artboard, trigger, and list values.
- Asset loading, replacement, decoding, renderer hooks, or stable public asset
  object handles.
- Property-name APIs for imported scalar mutation.
- Reverse propagation, broader update queues, relative/parent/nested lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- Mutating an imported asset source through one state machine is visible when a
  second state machine binds the same imported view-model instance.
- The source and target asset binding reports match C++ after ordinary
  state-machine advancement for both state machines.
- Existing imported number, boolean, string, color, enum, symbol-list-index,
  and view-model pointer context probes continue to pass.
