# Data Binding Graph Owned ViewModel Name-Path Runtime Contract

## Purpose

Expose the narrow owned view-model pointer replacement API by property name
path, matching the C++ `ViewModelInstanceRuntime::replaceViewModel("a/b", ...)`
shape for generated owned children.

This is an ergonomic/public API parity slice over behavior already modeled by
the index-path API; it does not add imported-instance mutation or stable runtime
object handles.

## In Scope

- `RuntimeOwnedViewModelInstance` view-model pointer properties only.
- Slash-separated generated child paths such as `child/middle/leaf`.
- Replacing the selected generated owned pointer with a referenced imported
  instance by instance index.
- Reusing the existing generated-child mutation rules, including rejection once
  the path would cross an imported intermediate.
- C++ probe coverage through existing owned view-model relink and unsupported
  imported-intermediate mutation cases.

## Out Of Scope

- Name-based setters for scalar, list, symbol, asset, artboard, or trigger
  owned properties.
- Persistent mutation of imported `RuntimeFile` instances across contexts.
- Stable public object handles exposing `referenceViewModelInstance` pointers.
- Mutating through imported intermediates.
- Relative paths, parent paths, listener-owned data binding, reverse
  propagation, and nested artboard propagation.

## Completion Checks

- A generated owned path like `child/middle/leaf` can relink through
  `set_view_model_by_property_name_path` with C++ parity.
- The same named path returns `false` once the root child is imported, matching
  the unsupported C++ imported-intermediate mutation boundary.
- Existing index-path owned view-model pointer probes continue to pass.
