# Data Binding Graph Enum Public Update Observer Preservation Runtime Contract

## Purpose

Admit direct enum public `updateDataBinds(true)` target-to-source behavior and
pin the first same-path enum observer scheduling case.

This slice extends the public-update direct value lane beyond number, boolean,
string, color, and trigger to `BindablePropertyEnum.propertyValue`.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceEnum.propertyValue` sources.
- `BindablePropertyEnum.propertyValue` targets.
- A dirty mutating main-`ToTarget | TwoWay` direct enum bind.
- Public-update source writes from the edited enum target.
- Source-to-target reapplication of the mutating bind during the same public
  update.
- A neighboring ordinary direct `ToTarget` enum bind to the same source path.
- Preservation of the observer target during the public update action.
- Follow-up application of the observer target on the next normal
  state-machine advance.
- Exact C++ probe reporting for the mutating enum bind and the optional
  observer bind after each explicit runtime action.

## Out Of Scope

- Enum converters and enum converter groups.
- Multiple observers, cross-type observers, imported/owned contexts, and
  relative, parent, nested, or listener-owned source paths.
- Pending add/remove behavior, observer-list parity, persisting-list ordering,
  and re-entry protection beyond this direct enum public-update path.
- Full dirty-list scheduler parity for arbitrary source/target graphs.
- Nested artboards, render/layout behavior, hit testing, and listener-owned
  dispatch.

## Completion Checks

- A mutated direct main-`ToTarget | TwoWay` enum target writes the edited enum
  index into the default enum source during public `updateDataBinds(true)`.
- The same public update reapplies source-to-target for the mutating enum bind
  so Rust and C++ report the same source and target values.
- A same-path direct observer reports the C++ source value but preserves its
  previous target value during the public update action.
- The next normal state-machine advance applies the observer's direct
  source-to-target enum value.
- Previously admitted explicit enum target-to-source and
  number/boolean/string/color observer-preservation tests still pass.
