# Data Binding Graph Imported ViewModel Persistent Relink Runtime Contract

## Purpose

Model the first persistent imported view-model pointer relink behavior for a
bound state-machine data context.

C++ `ViewModelInstance::replaceViewModelByProperty` mutates the imported
`ViewModelInstanceViewModel` reference behind a `ViewModelInstanceRuntime`.
For this Rust runtime slice, the mutation is represented as a state-machine
data-bind graph overlay keyed by imported view-model index, imported instance
index, and source path. Rebinding the same imported context sees the relinked
pointer.

## In Scope

- State-machine `bind_view_model_instance_context` for imported
  `ViewModelInstance` contexts.
- `ViewModelInstanceViewModel` source paths that already resolve through a
  `DataBindContext.sourcePathIds` buffer.
- Relinking the currently bound imported-context view-model pointer source by
  referenced imported instance index.
- Rebinding the same imported view-model instance and observing the relinked
  source and target pointer values with C++ parity.

## Out Of Scope

- Mutating `RuntimeFile` or `rive-binary` imported object storage.
- Sharing imported-instance mutations across independent `StateMachineInstance`
  values.
- Stable public object handles exposing `referenceViewModelInstance` pointers.
- Persistent scalar, list, symbol, asset, artboard, or trigger mutation on
  imported instances.
- Relative paths, parent paths, listener-owned data binding, reverse
  propagation, and nested artboard propagation.

## Completion Checks

- Imported-context pointer relink still updates the currently bound graph
  source and target.
- Rebinding the same imported context after relink preserves the relinked
  pointer source and target with C++ parity.
- Existing default-context, owned-context, and imported-context view-model
  pointer probes continue to pass.
