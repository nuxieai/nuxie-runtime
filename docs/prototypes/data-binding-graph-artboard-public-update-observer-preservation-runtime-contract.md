# Data Binding Graph Artboard Public Update Observer Preservation Runtime Contract

## Purpose

Admit direct artboard public `updateDataBinds(true)` target-to-source behavior
and pin the first same-path artboard observer scheduling case.

This slice extends the public-update direct value lane beyond number, boolean,
string, color, enum, trigger, and asset to
`BindablePropertyArtboard.propertyValue`.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceArtboard.propertyValue` sources.
- `BindablePropertyArtboard.propertyValue` targets.
- A dirty mutating main-`ToTarget | TwoWay` direct artboard bind.
- Public-update source writes from the edited artboard target.
- Source-to-target reapplication of the mutating bind during the same public
  update.
- A neighboring ordinary direct `ToTarget` artboard bind to the same source
  path.
- Preservation of the observer target during the public update action.
- Follow-up application of the observer target on the next normal
  state-machine advance.
- Exact C++ probe reporting for the mutating artboard bind and the optional
  observer bind after each explicit runtime action.

## Out Of Scope

- Artboard converters and converter groups.
- Multiple observers, cross-type observers, imported/owned contexts, and
  relative, parent, nested, or listener-owned source paths.
- Nested artboard instancing, host transforms, nested state-machine
  advancement, hit testing, focus, and render/layout behavior.
- Pending add/remove behavior, observer-list parity, persisting-list ordering,
  and re-entry protection beyond this direct artboard public-update path.
- Full dirty-list scheduler parity for arbitrary source/target graphs.
- Listener-owned dispatch.

## Completion Checks

- A mutated direct main-`ToTarget | TwoWay` artboard target writes the edited
  artboard id into the default artboard source during public
  `updateDataBinds(true)`.
- The same public update reapplies source-to-target for the mutating artboard
  bind so Rust and C++ report the same source and target values.
- A same-path direct observer reports the C++ source value but preserves its
  previous target value during the public update action.
- The next normal state-machine advance applies the observer's direct
  source-to-target artboard value.
- Previously admitted explicit artboard target-to-source and
  number/boolean/string/color/enum/trigger/asset observer-preservation tests
  still pass.
