# Data Binding Graph Imported ViewModel Name Path Runtime Contract

## Purpose

Admit the first property-name relink API for imported view-model contexts
without changing imported file storage.

C++ `ViewModelInstanceRuntime::replaceViewModel` accepts a slash-separated
property-name path and mutates the matching `ViewModelInstanceViewModel`
reference. For this Rust slice, the state-machine runtime resolves a property
name path against the currently bound imported view-model context, then records
the relink through the existing state-machine-local imported pointer overlay.
The C++ probe runs with completed view-model property links because the public
runtime name API depends on `ViewModelInstanceValue::viewModelProperty()`.

## In Scope

- State-machine contexts bound with `bind_view_model_instance_context`.
- Slash-separated property-name paths whose segments map to
  `ViewModelPropertyViewModel` properties on imported view-model definitions.
- C++ public name-path replacement after `completeViewModelProperties` has
  linked imported instance values back to their `ViewModelProperty` records.
- Existing graph source paths that resolve to `ViewModelInstanceViewModel`
  sources.
- Relinking the currently bound imported-context source by referenced imported
  instance index.
- Rebinding the same imported context and observing the relinked source and
  target pointer values with C++ parity.

## Out Of Scope

- Sharing imported-instance mutations across independent
  `StateMachineInstance` values.
- Mutating `RuntimeFile` or `nuxie-binary` imported object storage.
- Stable public object handles exposing `referenceViewModelInstance` pointers.
- Scalar, list, symbol, asset, artboard, or trigger imported mutation.
- Name-path mutation through owned imported intermediates beyond already pinned
  unsupported boundaries.
- Relative paths, parent paths, listener-owned data binding, reverse
  propagation, and nested artboard propagation.

## Completion Checks

- Imported-context relink by data-bind index continues to pass.
- Imported-context relink by property-name path updates the currently bound
  graph source and target with C++ parity.
- Rebinding the same imported context after a name-path relink preserves the
  relinked pointer with C++ parity.
