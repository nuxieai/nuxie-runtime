# Data Binding Graph Public Update Observer Preservation Runtime Contract

## Purpose

Pin the first same-path observer scheduling behavior for public
`updateDataBinds(true)` target-to-source updates.

When C++ drains a dirty main-`ToTarget | TwoWay` number bind through public
`updateDataBinds(true)`, it reapplies source-to-target for the mutating bind in
that public update. A neighboring ordinary `ToTarget` bind to the same source
path reports the newly written source value, but its bindable target keeps the
value from the previous source-to-target application until the next normal
state-machine advance.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source nodes.
- Two `BindablePropertyNumber.propertyValue` targets sharing the same source
  path.
- A dirty mutating main-`ToTarget | TwoWay` grouped number bind using
  `DataConverterRangeMapper -> DataConverterOperationValue`.
- A neighboring ordinary direct `ToTarget` number bind to the same source path.
- Public-update application of the mutating bind only.
- Preservation of the observer target during the public update action.
- Follow-up application of the observer target on the next normal
  state-machine advance.
- Exact C++ probe reporting for both number binds after each explicit runtime
  action.

## Out Of Scope

- Multiple observers, cross-type observers, imported/owned contexts, and
  relative, parent, nested, or listener-owned source paths.
- Public-update observer ordering for non-number targets.
- Pending add/remove behavior, observer-list parity, persisting-list ordering,
  and re-entry protection beyond this same-path two-bind fixture.
- Full dirty-list scheduler parity for arbitrary source/target graphs.
- Stateful converter child scheduling beyond already admitted direct
  interpolator public-update behavior.
- Nested artboards, render/layout behavior, hit testing, and listener-owned
  dispatch.

## Completion Checks

- The mutating grouped number bind writes the C++ source value and reapplies
  its own grouped source-to-target value during public update.
- The same-path direct observer reports the C++ source value but preserves its
  previous target value during the public update action.
- The next normal state-machine advance applies the observer's direct
  source-to-target value.
- Both binds' exact source and target reports match the C++ probe after each
  explicit runtime action.
- Previously admitted public-update target-to-source tests still pass.
