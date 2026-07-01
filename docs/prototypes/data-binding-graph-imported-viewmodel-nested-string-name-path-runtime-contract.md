# Data Binding Graph Imported ViewModel Nested String Name-Path Runtime Contract

## Purpose

Admit the string sibling of imported-context nested property-name path
mutation.

C++ can mutate a nested imported string source by resolving a slash-separated
path such as `child/label` through
`ViewModelInstanceRuntime::propertyString`, then binding multiple authored
state machines to the same imported `ViewModelInstance`. Rust mirrors that by
letting `RuntimeImportedViewModelInstanceContext` resolve the same property
name path to the existing data-bind source path and store the byte override in
the shared imported-context overlay.

## In Scope

- File-backed imported root view-model instances.
- One nested `ViewModelPropertyViewModel` segment followed by one
  `ViewModelPropertyString` leaf.
- `RuntimeImportedViewModelInstanceContext::set_string_by_property_name_path`.
- Sharing the mutated nested string source across two authored state machines
  bound through the same imported context.
- C++ probe coverage using `ViewModelInstanceRuntime::propertyString` with the
  `child/label` path after completing view-model properties.

## Out Of Scope

- Color, enum, symbol-list-index, asset, artboard, trigger, list, and
  view-model pointer nested property-name paths.
- Imported-instance mutation without an explicit shared context, stable public
  object handles, reverse propagation, broader update queues,
  relative/parent/name-manifest lookup, listener-owned data binding, and nested
  artboard propagation.

## Completion Checks

- The C++ probe mutates `child/label` on one imported view-model instance and
  two state machines bound to that instance observe the new string source.
- Rust resolves `child/label` to the same graph source path and records the
  override in `RuntimeImportedViewModelInstanceContext`.
- String binding source and target reports stay equal between C++ and Rust for
  both state machines.
