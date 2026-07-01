# Data Binding Graph String Public Update Observer Preservation Runtime Contract

## Purpose

Pin same-path observer scheduling for public `updateDataBinds(true)`
target-to-source updates on direct string binds.

This slice extends the observer-preservation scheduler coverage beyond number
and boolean values to byte-backed string values.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceString.propertyValue` sources.
- Two `BindablePropertyString.propertyValue` targets sharing the same source
  path.
- A dirty mutating main-`ToTarget | TwoWay` direct string bind.
- A neighboring ordinary direct `ToTarget` string bind to the same source path.
- Public-update application of the mutating bind only.
- Preservation of the observer target during the public update action.
- Follow-up application of the observer target on the next normal
  state-machine advance.
- Exact C++ probe reporting for both string binds after each explicit runtime
  action.

## Out Of Scope

- Multiple observers, cross-type observers, imported/owned contexts, and
  relative, parent, nested, or listener-owned source paths.
- `DataConverterToString` observers, string converter-family observers, and
  string converter-group observers.
- Pending add/remove behavior, observer-list parity, persisting-list ordering,
  and re-entry protection beyond this same-path two-bind fixture.
- Full dirty-list scheduler parity for arbitrary source/target graphs.
- Nested artboards, render/layout behavior, hit testing, and listener-owned
  dispatch.

## Completion Checks

- The mutating string bind writes the C++ source value and reapplies its own
  source-to-target value during public update.
- The same-path direct observer reports the C++ source value but preserves its
  previous target value during the public update action.
- The next normal state-machine advance applies the observer's direct
  source-to-target value.
- Both binds' exact source and target reports match the C++ probe after each
  explicit runtime action.
- Previously admitted string public-update and number/boolean observer
  preservation tests still pass.
