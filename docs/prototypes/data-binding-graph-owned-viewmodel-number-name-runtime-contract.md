# Data Binding Graph Owned ViewModel Number Name Runtime Contract

## Purpose

Admit the first owned scalar property-name mutation API while keeping the
owned view-model runtime context small and explicit.

C++ `ViewModelInstanceRuntime::propertyNumber(name)->value(...)` mutates a
number property by `ViewModelPropertyNumber.name` before binding the owned
view-model instance to a state machine. For this Rust slice,
`RuntimeOwnedViewModelInstance` records root view-model property names and
allows `set_number_by_property_name` to route to the existing property-index
number storage.

## In Scope

- Root owned `RuntimeOwnedViewModelInstance` contexts created from a
  `RuntimeFile` view-model definition.
- `ViewModelPropertyNumber.name` lookup for root number properties.
- Mutating the owned number value by property name before binding the owned
  context to a state machine.
- Existing graph source-to-target propagation after
  `bind_owned_view_model_context`.

## Out Of Scope

- Boolean, string, color, enum, symbol-list-index, asset, artboard, trigger,
  list, or view-model scalar name APIs.
- Nested scalar property paths.
- Imported-instance mutation, stable public object handles, and shared mutable
  imported storage.
- Reverse propagation, broader update queues, relative/parent/nested lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- The existing property-index owned number setter continues to work.
- The new property-name owned number setter matches the existing C++ public
  `propertyNumber(name)` probe behavior.
- Existing owned view-model context probes continue to pass.
