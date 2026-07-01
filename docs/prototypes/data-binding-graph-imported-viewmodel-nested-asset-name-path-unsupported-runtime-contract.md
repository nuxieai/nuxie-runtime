# Data Binding Graph Imported ViewModel Nested Asset Name-Path Unsupported Runtime Contract

## Purpose

Pin the imported-context nested asset property-name path boundary.

C++ probe mutation through
`--runtime-set-view-model-instance-source-asset-by-name` uses
`ViewModelInstance::propertyValue(name)` for asset-image values. That root name
lookup can mutate `image`, but it does not resolve slash-separated nested paths
such as `child/image`, even after completing view-model properties. Rust mirrors
that boundary by refusing
`RuntimeImportedViewModelInstanceContext::set_asset_by_property_name_path` for
slash paths.

## In Scope

- File-backed imported root view-model instances.
- One nested `ViewModelPropertyViewModel` segment followed by one
  `ViewModelPropertyAssetImage` leaf.
- An explicit `false` result from
  `RuntimeImportedViewModelInstanceContext::set_asset_by_property_name_path`
  for `child/image`.
- C++ probe coverage showing the nested asset source remains unchanged after a
  slash-path asset-by-name mutation attempt.

## Out Of Scope

- Adding a C++ `propertyImage("child/image")` probe path or a Rust equivalent.
- Artboard, trigger, list, and view-model pointer nested property-name paths.
- Stable public asset object handles, reverse propagation, broader update
  queues, relative/parent/name-manifest lookup, listener-owned data binding,
  and nested artboard propagation.

## Completion Checks

- The C++ probe attempts to mutate `child/image` and both bound state machines
  still observe the original asset source.
- Rust returns `false` for the same imported nested asset name path and keeps
  the original asset source in the shared imported context.
- Asset binding source and target reports stay equal between C++ and Rust for
  both state machines.
