# Data Binding Graph Imported ViewModel Nested Symbol-List-Index Name-Path Runtime Contract

## Purpose

Admit the symbol-list-index sibling of imported-context nested property-name
path mutation.

C++ can mutate a nested imported symbol-list-index source by resolving a
slash-separated path such as `child/symbol` through
`ViewModelInstanceRuntime::propertySymbolListIndex`, then binding multiple
authored state machines to the same imported `ViewModelInstance`. Rust mirrors
that by letting `RuntimeImportedViewModelInstanceContext` resolve the same
property name path to the existing data-bind source path and store the
symbol-list-index override in the shared imported-context overlay.

## In Scope

- File-backed imported root view-model instances.
- One nested `ViewModelPropertyViewModel` segment followed by one
  `ViewModelPropertySymbolListIndex` leaf.
- `RuntimeImportedViewModelInstanceContext::
  set_symbol_list_index_by_property_name_path`.
- Sharing the mutated nested symbol-list-index source across two authored state
  machines bound through the same imported context.
- C++ probe coverage using `ViewModelInstanceRuntime::propertySymbolListIndex`
  with the `child/symbol` path after completing view-model properties.

## Out Of Scope

- Asset, artboard, trigger, list, and view-model pointer nested property-name
  paths.
- Imported-instance mutation without an explicit shared context, stable public
  object handles, reverse propagation, broader update queues,
  relative/parent/name-manifest lookup, listener-owned data binding, and nested
  artboard propagation.

## Completion Checks

- The C++ probe mutates `child/symbol` on one imported view-model instance and
  two state machines bound to that instance observe the new symbol-list-index
  source through the converted string transition condition.
- Rust resolves `child/symbol` to the same graph source path and records the
  override in `RuntimeImportedViewModelInstanceContext`.
- State-machine advance reports stay equal between C++ and Rust for both state
  machines, and Rust exposes the shared symbol-list-index source override.
