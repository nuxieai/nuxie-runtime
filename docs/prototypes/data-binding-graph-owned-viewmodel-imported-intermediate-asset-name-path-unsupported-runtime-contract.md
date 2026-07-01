# Data Binding Graph Owned ViewModel Imported-Intermediate Asset Name-Path Unsupported Runtime Contract

## Purpose

Pin the public asset name-path mutation boundary for owned view-model contexts
after a generated child has been replaced with an imported child instance.

C++ can bind through the imported child's existing
`ViewModelInstanceAssetImage.propertyValue`, but an attempted public
`child/image` property mutation after replacing `child` with an imported
instance leaves the observed graph source unchanged. Rust keeps the same
boundary: `RuntimeOwnedViewModelInstance::set_asset_by_property_name_path`
returns `false` once the path crosses an imported intermediate.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested asset source paths such as `child/image`.
- The attempted C++ public path shape that resolves the owner with
  `ViewModelInstanceRuntime::propertyViewModel("child")` and writes the
  child's `ViewModelInstanceAssetImage.propertyValue`.
- Verifying that both C++ and Rust preserve the imported child's existing
  asset source value after binding and state-machine advancement.

## Out Of Scope

- Supporting mutation through imported intermediates.
- Artboard, trigger, list, and view-model pointer name-path mutation
  boundaries.
- Asset loading, image decoding, renderer integration, stable public object
  handles, reverse propagation, broader update queues, relative/parent/name
  lookup, listener-owned data binding, and nested artboard propagation.

## Completion Checks

- The C++ probe replaces `child` with an imported child, attempts to write
  `child/image`, binds the owned context, and still behaves as if the imported
  child's original asset ID is selected.
- Rust rejects the same mutation through
  `set_asset_by_property_name_path("child/image", value)` after `child` is
  imported.
- The state-machine advance reports stay equal between C++ and Rust.
