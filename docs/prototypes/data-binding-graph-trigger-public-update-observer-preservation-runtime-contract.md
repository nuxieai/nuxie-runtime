# Data Binding Graph Trigger Public Update Observer Preservation Runtime Contract

## Purpose

Pin the first same-path observer scheduling case for direct trigger public
`updateDataBinds(true)` target-to-source behavior.

The direct trigger public-update source write is already covered by
`docs/prototypes/data-binding-graph-trigger-public-update-target-to-source-runtime-contract.md`.
This slice adds the observer-ordering proof for a neighboring direct
`BindablePropertyTrigger.propertyValue` bind to the same trigger source.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceTrigger.propertyValue` sources.
- `BindablePropertyTrigger.propertyValue` targets.
- A dirty mutating main-`ToTarget | TwoWay` direct trigger bind.
- A neighboring ordinary direct `ToTarget` trigger bind to the same source
  path.
- Preservation of the observer target during the public update action while
  reporting the updated shared trigger source.
- Follow-up application of the observer target on the next normal
  state-machine advance.
- Exact C++ probe reporting for both trigger binds after each explicit runtime
  action.

## Out Of Scope

- `DataConverterTrigger`, trigger converter groups, trigger reset/reapply
  variants, and trigger-to-string converter paths beyond existing regression
  coverage.
- Multiple observers, cross-type observers, imported/owned contexts, and
  relative, parent, nested, or listener-owned source paths.
- Listener callback dispatch and host event delivery.
- Pending add/remove behavior, observer-list parity, persisting-list ordering,
  and re-entry protection beyond this direct trigger public-update path.
- Full dirty-list scheduler parity for arbitrary source/target graphs.
- Nested artboards, render/layout behavior, and hit testing.

## Completion Checks

- The direct main-`ToTarget | TwoWay` trigger public-update source write still
  matches C++.
- A same-path direct observer reports the C++ source value but preserves its
  previous target value during the public update action.
- The next normal state-machine advance applies the observer's direct
  source-to-target trigger value.
- Existing direct trigger public-update, converter public-update, converter
  group public-update, explicit target-to-source, and trigger reset/reapply
  tests still pass.
