# Data Binding Graph Imported ViewModel Nested Trigger Name-Path Unsupported Runtime Contract

## Purpose

Pin the imported-context nested trigger property-name path boundary.

C++ probe mutation through
`--runtime-set-view-model-instance-source-trigger-by-name` uses
`ViewModelInstance::propertyValue(name)` for trigger values. That root name
lookup can mutate `fire`, but it does not resolve slash-separated nested paths
such as `child/fire`, even after completing view-model properties. Rust mirrors
that boundary by refusing
`RuntimeImportedViewModelInstanceContext::set_trigger_by_property_name_path`
for slash paths.

## In Scope

- File-backed imported root view-model instances.
- One nested `ViewModelPropertyViewModel` segment followed by one
  `ViewModelPropertyTrigger` leaf.
- An explicit `false` result from
  `RuntimeImportedViewModelInstanceContext::set_trigger_by_property_name_path`
  for `child/fire`.
- C++ probe coverage showing the nested trigger source remains unchanged after
  a slash-path trigger-by-name mutation attempt.

## Out Of Scope

- Adding a C++ `propertyTrigger("child/fire")` probe path or a Rust equivalent.
- List and view-model pointer nested property-name paths.
- Stable public trigger object handles, reverse propagation, broader update
  queues, relative/parent/name-manifest lookup, listener-owned data binding,
  and nested artboard propagation.

## Completion Checks

- The C++ probe attempts to mutate `child/fire` and the bound state machine
  still observes the original trigger source.
- Rust returns `false` for the same imported nested trigger name path and keeps
  the original trigger source in the shared imported context.
- Trigger binding source and target reports stay equal between C++ and Rust.
