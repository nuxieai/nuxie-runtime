# Data Binding Graph ToString Public Update Target-To-Source Runtime Contract

## Purpose

Admit public `updateDataBinds(true)` target-to-source behavior for direct
`DataConverterToString` on main-`ToTarget | TwoWay` string binds.

`DataConverterToString` does not override C++ `reverseConvert`, so public
update receives the edited string target through the base converter identity.
For non-string sources, that reverse value does not match the source value
type and the source remains unchanged. For string sources, the edited string is
written to the source. The same public update then reapplies source-to-target
conversion through `DataConverterToString`.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue`,
  `ViewModelInstanceBoolean.propertyValue`,
  `ViewModelInstanceString.propertyValue`,
  `ViewModelInstanceTrigger.propertyValue`,
  `ViewModelInstanceSymbolListIndex.propertyValue`,
  `ViewModelInstanceColor.propertyValue`, and
  `ViewModelInstanceEnum.propertyValue` sources.
- `BindablePropertyString.propertyValue` targets.
- Direct `DataConverterToString` on main-`ToTarget | TwoWay` string data binds.
- Imported `DataConverterToString` flags, decimals, and color-format behavior
  already admitted by the forward converter slices.
- C++ base `DataConverter::reverseConvert` behavior for the edited string
  target value.
- No source write when the reverse value type is string but the source expects
  number, boolean, trigger, symbol-list-index, color, or enum data.
- String-source writes when the reverse value type matches the source value
  type.
- Immediate source-to-target reapplication from the resulting source value.
- Exact C++ probe reporting for each mutating string bind's target value after
  each explicit runtime action.

## Out Of Scope

- `DataConverterStringTrim`, `DataConverterStringRemoveZeros`, and
  `DataConverterStringPad` public-update behavior.
- Converter groups containing `DataConverterToString`.
- Main-`ToSource | TwoWay` target-to-source behavior for
  `DataConverterToString`.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Public source handles, list and view-model bindables, imported and owned
  view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- Mutating each main-`ToTarget | TwoWay` string target and calling public
  `updateDataBinds(true)` drains the dirty bind.
- Number, boolean, trigger, symbol-list-index, color, and enum sources remain
  unchanged because the reverse value type does not match the source value
  type.
- String sources update to the edited target string.
- The same public update reapplies source-to-target through
  `DataConverterToString::convert`.
- Each mutating bind's exact target value matches the C++ probe after each
  explicit runtime action.
- Previously admitted `DataConverterToString` forward and target-dirty tests
  still pass.
