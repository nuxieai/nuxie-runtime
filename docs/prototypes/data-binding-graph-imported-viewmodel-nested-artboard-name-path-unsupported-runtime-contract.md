# Data Binding Graph Imported ViewModel Nested Artboard Name-Path Unsupported Runtime Contract

## Purpose

Pin the imported-context nested artboard property-name path boundary.

C++ probe mutation through
`--runtime-set-view-model-instance-source-artboard-by-name` uses
`ViewModelInstance::propertyValue(name)` for artboard values. That root name
lookup can mutate `scene`, but it does not resolve slash-separated nested paths
such as `child/scene`, even after completing view-model properties. Rust
mirrors that boundary by refusing
`RuntimeImportedViewModelInstanceContext::set_artboard_by_property_name_path`
for slash paths.

## In Scope

- File-backed imported root view-model instances.
- One nested `ViewModelPropertyViewModel` segment followed by one
  `ViewModelPropertyArtboard` leaf.
- An explicit `false` result from
  `RuntimeImportedViewModelInstanceContext::set_artboard_by_property_name_path`
  for `child/scene`.
- C++ probe coverage showing the nested artboard source remains unchanged after
  a slash-path artboard-by-name mutation attempt.

## Out Of Scope

- Adding a C++ `propertyArtboard("child/scene")` probe path or a Rust
  equivalent.
- Trigger, list, and view-model pointer nested property-name paths.
- Stable public artboard object handles, reverse propagation, broader update
  queues, relative/parent/name-manifest lookup, listener-owned data binding,
  and nested artboard propagation.

## Completion Checks

- The C++ probe attempts to mutate `child/scene` and both bound state machines
  still observe the original artboard source.
- Rust returns `false` for the same imported nested artboard name path and
  keeps the original artboard source in the shared imported context.
- Artboard binding source and target reports stay equal between C++ and Rust
  for both state machines.
