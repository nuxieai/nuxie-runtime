# Data Binding Graph ToNumber Remaining Public Update Target-To-Source Runtime Contract

## Purpose

Admit public `updateDataBinds(true)` target-to-source behavior for the
remaining direct `DataConverterToNumber` source kinds already covered by
forward and target-dirty runtime slices: enum, color, string, and
symbol-list-index.

Like the boolean-source slice, C++ uses the base `DataConverter::reverseConvert`
for these main-`ToTarget | TwoWay` number binds because `DataConverterToNumber`
does not override `reverseConvert`. The base reverse path returns the edited
numeric target as a `DataValueNumber`. Because the resolved source expects a
non-number value type, `DataBindContextValue::calculateValueAndApply` does not
write the source. The public update still drains the dirty bind and reapplies
source-to-target, so the number target returns to the unchanged source's
`DataConverterToNumber::convert` result.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceEnum.propertyValue`,
  `ViewModelInstanceColor.propertyValue`,
  `ViewModelInstanceString.propertyValue`, and
  `ViewModelInstanceSymbolListIndex.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterToNumber` on a main-`ToTarget | TwoWay` number data
  bind.
- C++ base `DataConverter::reverseConvert` behavior for the numeric target
  value.
- No source write when the reverse value type is `DataValueNumber` but the
  source expects enum, color, string, or symbol-list-index data.
- Immediate source-to-target reapplication from the unchanged source.
- Exact C++ probe reporting for each mutating number bind's target value after
  each explicit runtime action.

## Out Of Scope

- The boolean-source public-update path, which is covered by
  `docs/prototypes/data-binding-graph-to-number-boolean-public-update-target-to-source-runtime-contract.md`.
- Number-source `DataConverterToNumber` public-update behavior.
- Main-`ToSource | TwoWay` target-to-source behavior for `DataConverterToNumber`.
- Converter groups containing `DataConverterToNumber`.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Public source handles, list and view-model bindables, imported and owned
  view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- Mutating each main-`ToTarget | TwoWay` number target and calling public
  `updateDataBinds(true)` drains the dirty bind.
- Enum, color, string, and symbol-list-index sources remain unchanged because
  the reverse value type does not match the source value type.
- The same public update reapplies source-to-target through
  `DataConverterToNumber::convert`, restoring the number target from the
  unchanged source.
- Each mutating bind's exact target value matches the C++ probe after each
  explicit runtime action.
- Previously admitted `DataConverterToNumber` forward, target-dirty, and
  boolean public-update tests still pass.
