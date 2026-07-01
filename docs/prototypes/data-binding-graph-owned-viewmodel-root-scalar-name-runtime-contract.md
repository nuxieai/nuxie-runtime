# Data Binding Graph Owned ViewModel Root Scalar Name Runtime Contract

## Purpose

Complete the owned root scalar property-name mutation surface that already has
Rust property-index storage and C++ probe coverage.

C++ creates an owned `ViewModelInstance` and mutates root scalar values by
`ViewModelProperty.name` before binding it to a state machine. Number, boolean,
string, color, and enum use typed `ViewModelInstanceRuntime::property*`
helpers; symbol-list-index, asset, artboard, and trigger use
`ViewModelInstance::propertyValue(name)` followed by type-specific mutation.

For this Rust slice, `RuntimeOwnedViewModelInstance` keeps the root
`ViewModelProperty.name` table introduced by the number-name slice and exposes
name-based setters for every owned root scalar kind already represented by the
existing property-index setters.

## In Scope

- Root owned `RuntimeOwnedViewModelInstance` contexts created from a
  `RuntimeFile` view-model definition.
- Root property-name lookup for number, boolean, string, color, enum,
  symbol-list-index, asset, artboard, and trigger properties.
- Mutating these owned scalar values by property name before binding the owned
  context to a state machine.
- Existing graph source-to-target propagation after
  `bind_owned_view_model_context`.

## Out Of Scope

- Nested scalar property paths.
- Imported-instance mutation, stable public object handles, and shared mutable
  imported storage.
- View-model pointer APIs beyond the already admitted property-index and
  property-name path relink slices.
- Reverse propagation, broader update queues, relative/parent/nested lookup,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- Existing property-index owned scalar setters remain unchanged and are the
  delegate path for the name setters.
- Existing owned root scalar C++ parity probes pass while Rust uses the
  matching root property-name setters.
- The number-specific name setter remains covered by its explicit C++ parity
  test from
  `docs/prototypes/data-binding-graph-owned-viewmodel-number-name-runtime-contract.md`.
