# Data Binding Graph Symbol List Index Public Update Observer Preservation Runtime Contract

## Purpose

Admit direct symbol-list-index public `updateDataBinds(true)`
target-to-source behavior and pin the first same-path observer scheduling case.

Rive schema exposes symbol-list-index targets through
`BindablePropertyInteger.propertyValue`, so this slice is direct
symbol-list-index source parity through the integer target path. It does not
create a separate symbol-list-index bindable target type.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources.
- `BindablePropertyInteger.propertyValue` targets.
- A dirty mutating main-`ToTarget | TwoWay` direct integer bind.
- Public-update source writes from the edited integer target into the
  symbol-list-index source.
- Source-to-target reapplication of the mutating bind during the same public
  update.
- A neighboring ordinary direct `ToTarget` integer bind to the same
  symbol-list-index source path.
- Preservation of the observer target during the public update action.
- Follow-up application of the observer target on the next normal
  state-machine advance.
- Exact C++ probe reporting for the mutating symbol-list-index bind and the
  optional observer bind after each explicit runtime action.

## Out Of Scope

- `DataConverterToString`, `DataConverterToNumber`, operation-value, formula,
  and converter-group symbol-list-index paths.
- Multiple observers, cross-type observers, imported/owned contexts, and
  relative, parent, nested, or listener-owned source paths.
- Pending add/remove behavior, observer-list parity, persisting-list ordering,
  and re-entry protection beyond this direct symbol-list-index public-update
  path.
- Full dirty-list scheduler parity for arbitrary source/target graphs.
- Nested artboards, render/layout behavior, hit testing, and listener-owned
  dispatch.

## Completion Checks

- A mutated direct main-`ToTarget | TwoWay` integer target writes the edited
  integer value into the default symbol-list-index source during public
  `updateDataBinds(true)`.
- The same public update reapplies source-to-target for the mutating
  symbol-list-index bind so Rust and C++ report the same source and target
  values.
- A same-path direct observer reports the C++ source value but preserves its
  previous target value during the public update action.
- The next normal state-machine advance applies the observer's direct
  source-to-target symbol-list-index value.
- Previously admitted explicit symbol-list-index target-to-source and
  number/boolean/string/color/enum/trigger/asset/artboard observer-preservation
  tests still pass.
