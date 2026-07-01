# Data Binding Graph Owned ViewModel Recursive Relink Runtime Contract

## Purpose

Generalize owned runtime view-model pointer relinking from one generated
intermediate child to recursively generated owned view-model paths.

C++ `File::createViewModelInstance(viewModel)` creates generated child
instances for nested `ViewModelPropertyViewModel` properties. A runtime-owned
root can then call `ViewModelInstanceRuntime::replaceViewModel` with a slash
path such as `child/middle/leaf`, replacing the deepest generated pointer with
a referenced imported instance before binding that owned root to a state
machine.

## In Scope

- Owned root view-model contexts bound with `bind_owned_view_model_context`.
- Absolute `DataBindContext.sourcePathIds` whose first segment is the root
  view-model index and whose remaining segments are generated owned
  `ViewModelPropertyViewModel` property indexes.
- Generated paths deeper than one intermediate child, proven by the
  `[root, child, middle, leaf]` path shape.
- Public Rust mutation by owned property path and referenced imported instance
  index.
- C++ probe mutation through
  `--runtime-bind-owned-view-model-deep-viewmodel-state-machine-context`, which
  creates an owned root instance, calls
  `ViewModelInstanceRuntime::replaceViewModel("child/middle/leaf", value)`,
  and binds the owned root to the state machine.
- C++ probe coverage through existing view-model binding reports.

## Out Of Scope

- Traversal through an imported replacement as an intermediate path segment.
- Nested scalar, list, symbol, asset, artboard, or trigger properties.
- Public property-name handles or object handles.
- Persistent mutation of imported `RuntimeFile` instances.
- Reverse target-to-source propagation.
- Broader update queues, relative paths, parent paths, listener-owned data
  binding, and nested artboard propagation.

## Completion Checks

- Owned context construction recursively records generated
  `ViewModelPropertyViewModel` children.
- `set_view_model_by_property_path(&[child, middle, leaf], index)` relinks the
  deepest generated source pointer to the selected imported instance.
- Binding the owned context resolves the deep source path and marks the source
  bound.
- Explicit data-context advance updates the
  `BindablePropertyViewModel.propertyValue` target.
- Existing root owned replacement/generated-child, one-intermediate owned
  relink, default relink, imported relink, and raw setter probes continue to
  pass.
