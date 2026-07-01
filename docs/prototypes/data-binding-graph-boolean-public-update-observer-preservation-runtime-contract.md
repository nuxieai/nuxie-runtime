# Data Binding Graph Boolean Public Update Observer Preservation Runtime Contract

## Purpose

Pin the first non-number same-path observer scheduling behavior for public
`updateDataBinds(true)` target-to-source updates.

This slice extends the existing number observer-preservation contract to a
direct boolean bind pair while keeping the scope to one source path and two
targets.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean.propertyValue` source nodes.
- Two `BindablePropertyBoolean.propertyValue` targets sharing the same source
  path.
- A dirty mutating main-`ToTarget | TwoWay` direct boolean bind.
- A neighboring ordinary direct `ToTarget` boolean bind to the same source
  path.
- Public-update application of the mutating bind only.
- Preservation of the observer target during the public update action.
- Follow-up application of the observer target on the next normal
  state-machine advance.
- Exact C++ probe reporting for both boolean binds after each explicit runtime
  action.

## Out Of Scope

- Multiple observers, cross-type observers, imported/owned contexts, and
  relative, parent, nested, or listener-owned source paths.
- BooleanNegate observer ordering and boolean converter groups.
- Pending add/remove behavior, observer-list parity, persisting-list ordering,
  and re-entry protection beyond this same-path two-bind fixture.
- Full dirty-list scheduler parity for arbitrary source/target graphs.
- Nested artboards, render/layout behavior, hit testing, and listener-owned
  dispatch.

## Completion Checks

- The mutating boolean bind writes the C++ source value and reapplies its own
  source-to-target value during public update.
- The same-path direct observer reports the C++ source value but preserves its
  previous target value during the public update action.
- The next normal state-machine advance applies the observer's direct
  source-to-target value.
- Both binds' exact source and target reports match the C++ probe after each
  explicit runtime action.
- Previously admitted boolean public-update and number observer-preservation
  tests still pass.
