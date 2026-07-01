# Data Binding Graph String Converter Family Public Update Target-To-Source Runtime Contract

## Purpose

Admit public `updateDataBinds(true)` target-to-source behavior for the direct
string converter family on main-`ToTarget | TwoWay` string binds:
`DataConverterStringTrim`, `DataConverterStringRemoveZeros`, and
`DataConverterStringPad`.

These converters do not override C++ `reverseConvert`, so public update
receives the edited string target through the base converter identity and
writes it to the string source. The same public update then reapplies
source-to-target conversion through the direct string converter, so the target
reports the trimmed, zero-stripped, or padded value immediately.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceString.propertyValue` sources.
- `BindablePropertyString.propertyValue` targets.
- Direct `DataConverterStringTrim`, `DataConverterStringRemoveZeros`, and
  `DataConverterStringPad` on main-`ToTarget | TwoWay` string data binds.
- Imported `trimType`, `length`, `padType`, and pad `text` behavior already
  admitted by the forward converter slices.
- C++ base `DataConverter::reverseConvert` behavior for the edited string
  target value.
- String-source writes from the edited target value.
- Immediate source-to-target reapplication through the direct string
  converter.
- Exact C++ probe reporting for each mutating string bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Direct `DataConverterToString` public-update behavior, covered by
  `docs/prototypes/data-binding-graph-to-string-public-update-target-to-source-runtime-contract.md`.
- Converter groups containing string converters.
- Main-`ToSource | TwoWay` target-to-source behavior for string converters.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Public source handles, list and view-model bindables, imported and owned
  view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- Mutating each main-`ToTarget | TwoWay` string target and calling public
  `updateDataBinds(true)` drains the dirty bind.
- The edited string writes to the default view-model string source.
- The same public update reapplies source-to-target through
  `DataConverterStringTrim`, `DataConverterStringRemoveZeros`, or
  `DataConverterStringPad`.
- Each mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Previously admitted string converter-family forward and target-dirty tests
  still pass.
