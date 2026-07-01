# Data Binding Graph Imported ViewModel Nested Number Name-Path Runtime Contract

## Purpose

Admit the first imported-context nested property-name path mutation for a
scalar source.

C++ can mutate a nested imported number source by resolving a slash-separated
path such as `child/amount` through
`ViewModelInstanceRuntime::propertyNumber`, then binding multiple authored
state machines to the same imported `ViewModelInstance`. Rust mirrors that by
letting `RuntimeImportedViewModelInstanceContext` resolve the same property
name path to the existing data-bind source path and store the override in the
shared imported-context overlay.

## In Scope

- File-backed imported root view-model instances.
- One nested `ViewModelPropertyViewModel` segment followed by one
  `ViewModelPropertyNumber` leaf.
- `RuntimeImportedViewModelInstanceContext::set_number_by_property_name_path`.
- Sharing the mutated nested number source across two authored state machines
  bound through the same imported context.
- C++ probe coverage using `ViewModelInstanceRuntime::propertyNumber` with the
  `child/amount` path after completing view-model properties.

## Out Of Scope

- Boolean, string, color, enum, symbol-list-index, asset, artboard, trigger,
  list, and view-model pointer nested property-name paths.
- Imported-instance mutation without an explicit shared context, stable public
  object handles, reverse propagation, broader update queues,
  relative/parent/name-manifest lookup, listener-owned data binding, and nested
  artboard propagation.

## Completion Checks

- The C++ probe mutates `child/amount` on one imported view-model instance and
  two state machines bound to that instance observe the new number source.
- Rust resolves `child/amount` to the same graph source path and records the
  override in `RuntimeImportedViewModelInstanceContext`.
- Number binding source and target reports stay equal between C++ and Rust for
  both state machines.
