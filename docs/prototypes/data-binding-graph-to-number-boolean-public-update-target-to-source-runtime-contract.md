# Data Binding Graph ToNumber Boolean Public Update Target-To-Source Runtime Contract

## Purpose

Admit the first public `updateDataBinds(true)` target-to-source slice for
`DataConverterToNumber` on a main-`ToTarget | TwoWay` number bind.

The representative source is `ViewModelInstanceBoolean.propertyValue`. C++
`DataConverterToNumber` does not override `reverseConvert`, so public update
uses the base converter reverse path and gets the edited numeric target value
back as a `DataValueNumber`. Because the resolved source expects a
`DataValueBoolean`, `DataBindContextValue::calculateValueAndApply` does not
write the source. The public update still drains the dirty bind and reapplies
source-to-target, so the number target returns to `1.0` for the unchanged
`true` boolean source.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean.propertyValue` source.
- `BindablePropertyNumber.propertyValue` target.
- A direct `DataConverterToNumber` on a main-`ToTarget | TwoWay` number data
  bind.
- C++ base `DataConverter::reverseConvert` behavior for the numeric target
  value.
- No source write when the reverse value type is `DataValueNumber` but the
  source expects `DataValueBoolean`.
- Immediate source-to-target reapplication from the unchanged boolean source.
- Exact C++ probe reporting for the mutating number bind's target value after
  each explicit runtime action.

## Out Of Scope

- Public-update `DataConverterToNumber` behavior for enum, color, string,
  symbol-list-index, number, list, or view-model source kinds.
- Main-`ToSource | TwoWay` target-to-source behavior for `DataConverterToNumber`.
- Converter groups containing `DataConverterToNumber`.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Public source handles, list and view-model bindables, imported and owned
  view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- Mutating the main-`ToTarget | TwoWay` number target and calling public
  `updateDataBinds(true)` drains the dirty bind.
- The boolean source remains unchanged because the reverse value type does not
  match the source value type.
- The same public update reapplies source-to-target through
  `DataConverterToNumber::convert`, restoring the number target from the
  unchanged boolean source.
- The mutating bind's exact target value matches the C++ probe after each
  explicit runtime action.
- Previously admitted public-update target-to-source tests still pass.
