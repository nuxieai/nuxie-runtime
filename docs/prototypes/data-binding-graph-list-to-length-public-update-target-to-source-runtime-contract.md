# Data Binding Graph List To Length Public Update Target-To-Source Runtime Contract

## Purpose

Admit public `updateDataBinds(true)` target-to-source behavior for direct
`DataConverterListToLength` on main-`ToTarget | TwoWay` list-to-number binds.

`DataConverterListToLength` does not override C++ `reverseConvert`, so the
public update path receives the edited number target through the base converter
identity. The resolved source is an imported list source represented by Rust as
its finite `ListLength` fact, so the numeric reverse value does not write the
source. The same public update still drains the dirty bind and immediately
reapplies source-to-target conversion from the unchanged imported list length.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- Imported `ViewModelInstanceList` sources represented by their imported item
  count.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterListToLength` on a main-`ToTarget | TwoWay` number
  data bind.
- C++ base `DataConverter::reverseConvert` behavior for the numeric target
  value.
- No source write when the reverse value type is numeric but the source is the
  list-length fact.
- Immediate source-to-target reapplication from the unchanged list length.
- Exact C++ probe reporting for the mutating number bind's target value after
  each explicit runtime action.

## Out Of Scope

- List targets and `BindablePropertyList` behavior.
- List mutation APIs and update-queue propagation from list edits.
- `DataConverterNumberToList` and generated runtime list items.
- Main-`ToSource | TwoWay` target-to-source behavior for
  `DataConverterListToLength`, covered by
  `docs/prototypes/data-binding-graph-list-to-length-main-to-source-target-to-source-runtime-contract.md`.
- Writing numeric target values back into list sources.
- Converter groups containing `DataConverterListToLength`.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Public source handles, imported and owned view-model contexts beyond the
  default root binding.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  relative/parent/nested lookup, listener-owned data binding, nested artboards,
  and render/layout behavior.

## Completion Checks

- Mutating the main-`ToTarget | TwoWay` number target and calling public
  `updateDataBinds(true)` drains the dirty bind.
- The imported list-length source remains unchanged because the reverse value
  type does not match a writable list source.
- The same public update reapplies source-to-target through
  `DataConverterListToLength::convert`, restoring the number target from the
  unchanged list length.
- The mutating bind's exact target value matches the C++ probe after each
  explicit runtime action.
- Previously admitted `DataConverterListToLength` forward and target-dirty
  tests still pass.
