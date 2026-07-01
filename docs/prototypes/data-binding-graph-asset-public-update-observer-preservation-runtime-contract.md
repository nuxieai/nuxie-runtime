# Data Binding Graph Asset Public Update Observer Preservation Runtime Contract

## Purpose

Admit direct asset public `updateDataBinds(true)` target-to-source behavior and
pin the first same-path asset observer scheduling case.

This slice extends the public-update direct value lane beyond number, boolean,
string, color, enum, and trigger to `BindablePropertyAsset.propertyValue`.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceAssetImage.propertyValue` sources.
- `BindablePropertyAsset.propertyValue` targets.
- A dirty mutating main-`ToTarget | TwoWay` direct asset bind.
- Public-update source writes from the edited asset target.
- Source-to-target reapplication of the mutating bind during the same public
  update.
- A neighboring ordinary direct `ToTarget` asset bind to the same source path.
- Preservation of the observer target during the public update action.
- Follow-up application of the observer target on the next normal
  state-machine advance.
- Exact C++ probe reporting for the mutating asset bind and the optional
  observer bind after each explicit runtime action.

## Out Of Scope

- Asset converters and converter groups.
- Multiple observers, cross-type observers, imported/owned contexts, and
  relative, parent, nested, or listener-owned source paths.
- Asset loading, render asset resolution, image replacement, and host asset
  callback behavior.
- Pending add/remove behavior, observer-list parity, persisting-list ordering,
  and re-entry protection beyond this direct asset public-update path.
- Full dirty-list scheduler parity for arbitrary source/target graphs.
- Nested artboards, render/layout behavior, hit testing, and listener-owned
  dispatch.

## Completion Checks

- A mutated direct main-`ToTarget | TwoWay` asset target writes the edited asset
  id into the default asset source during public `updateDataBinds(true)`.
- The same public update reapplies source-to-target for the mutating asset bind
  so Rust and C++ report the same source and target values.
- A same-path direct observer reports the C++ source value but preserves its
  previous target value during the public update action.
- The next normal state-machine advance applies the observer's direct
  source-to-target asset value.
- Previously admitted explicit asset target-to-source and
  number/boolean/string/color/enum/trigger observer-preservation tests still
  pass.
