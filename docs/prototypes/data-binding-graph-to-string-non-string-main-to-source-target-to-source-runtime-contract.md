# Data Binding Graph ToString Non-String Main-To-Source Target-To-Source Runtime Contract

## Purpose

Pin C++ behavior for direct `DataConverterToString` on
main-`ToSource | TwoWay` string binds whose sources are not strings.

For this direction, explicit data-context advancement attempts
target-to-source first. C++ applies `DataConverterToString::convert` to the
edited string target, producing a string value that does not match non-string
source types, so the source remains unchanged. The explicit data-context pass
preserves the edited target. The next normal state-machine advance refreshes
source-to-target in the reverse direction; because `DataConverterToString`
inherits the base `reverseConvert`, the non-string source value does not become
a string and the string target receives `DataValueString::defaultValue`, the
empty string.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue`,
  `ViewModelInstanceBoolean.propertyValue`,
  `ViewModelInstanceTrigger.propertyValue`,
  `ViewModelInstanceSymbolListIndex.propertyValue`,
  `ViewModelInstanceColor.propertyValue`, and
  `ViewModelInstanceEnum.propertyValue` sources.
- `BindablePropertyString.propertyValue` targets.
- Direct `DataConverterToString` on main-`ToSource | TwoWay` string data binds.
- Explicit `advancedDataContext()` target-to-source scheduling that preserves
  the edited string target when the source type does not match.
- Delayed normal-advance source-to-target refresh through C++ base
  `reverseConvert`, yielding the default empty string target value.
- Exact C++ probe reporting for each mutating string bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- `ViewModelInstanceString` sources for this main-`ToSource | TwoWay`
  `DataConverterToString` path; the current synthetic C++ probe crashes for
  that case, so this slice does not claim it.
- Main-`ToTarget | TwoWay` target-dirty behavior, covered by the direct
  `DataConverterToString` target-dirty contracts.
- Public `updateDataBinds(true)` behavior, covered by
  `docs/prototypes/data-binding-graph-to-string-public-update-target-to-source-runtime-contract.md`.
- `DataConverterStringTrim`, `DataConverterStringRemoveZeros`, and
  `DataConverterStringPad` main-`ToSource | TwoWay` behavior.
- Converter groups containing `DataConverterToString`.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Public source handles, list and view-model bindables, imported and owned
  view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- Initial explicit data-context advancement writes C++'s reverse/default empty
  string for each non-string source into the string target.
- Mutating the main-`ToSource | TwoWay` string target and explicitly advancing
  data context does not mutate the non-string source and preserves the edited
  target.
- The next normal state-machine advance refreshes the target back to the empty
  string through base reverse conversion.
- Each mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Previously admitted direct `DataConverterToString` forward, target-dirty,
  and public-update tests still pass.
