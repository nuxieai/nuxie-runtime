# Data Binding Graph Owned ViewModel Imported-Intermediate Trigger Name-Path Unsupported Runtime Contract

## Purpose

Pin the public trigger name-path mutation boundary for owned view-model
contexts after a generated child has been replaced with an imported child
instance.

C++ can bind through the imported child's existing
`ViewModelInstanceTrigger.propertyValue`, but an attempted public
`child/fire` property mutation after replacing `child` with an imported
instance leaves the observed graph source unchanged. Rust keeps the same
boundary: `RuntimeOwnedViewModelInstance::set_trigger_by_property_name_path`
returns `false` once the path crosses an imported intermediate.

## In Scope

- Owned root view-model contexts created from generated view-model metadata.
- One imported replacement intermediate reached through
  `RuntimeOwnedViewModelInstance::set_view_model_by_property_path`.
- Direct nested trigger source paths such as `child/fire`.
- The attempted C++ public path shape that resolves the owner with
  `ViewModelInstanceRuntime::propertyViewModel("child")` and writes the
  child's `ViewModelInstanceTrigger.propertyValue`.
- Verifying that both C++ and Rust preserve the imported child's existing
  trigger source value across the first bound state-machine advancement.

## Out Of Scope

- Supporting mutation through imported intermediates.
- Trigger firing APIs, listener/callback dispatch, trigger side effects,
  trigger converter groups, list and view-model pointer name-path mutation
  boundaries, stable public object handles, reverse propagation, broader
  update queues, relative/parent/name lookup, listener-owned data binding, and
  nested artboard propagation.

## Completion Checks

- The C++ probe replaces `child` with an imported child, attempts to write
  `child/fire`, binds the owned context, and still behaves as if the imported
  child's original trigger count is selected.
- Rust rejects the same mutation through
  `set_trigger_by_property_name_path("child/fire", value)` after `child` is
  imported.
- The post-bind trigger binding source and target reports stay equal between
  C++ and Rust.
