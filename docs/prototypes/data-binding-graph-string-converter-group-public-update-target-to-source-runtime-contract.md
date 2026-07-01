# Data Binding Graph String Converter Group Public Update Target-To-Source Runtime Contract

## Purpose

Admit public `updateDataBinds(true)` target-to-source behavior for the
already-supported string `DataConverterGroup` shape on a main-`ToTarget |
TwoWay` string bind.

The grouped string path uses `DataConverterStringTrim` followed by
`DataConverterStringPad`. C++ drains the public update by running
`DataConverterGroup::reverseConvert` from last child to first before writing
the string source. Neither direct string child overrides `reverseConvert`, so
the edited target writes through the group as the base identity. The same
public update then reapplies source-to-target through the group in forward
trim-then-pad order.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceString.propertyValue` sources.
- `BindablePropertyString.propertyValue` targets.
- A direct `DataConverterGroup` on a main-`ToTarget | TwoWay` string data bind.
- Ordered group items resolving to `DataConverterStringTrim` then
  `DataConverterStringPad`.
- C++ `DataConverterGroup::reverseConvert` child order for string targets.
- Immediate source-to-target reapplication through the grouped forward
  converter pipeline.
- Exact C++ probe reporting for the mutating string bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Direct string converter-family public-update behavior, covered by
  `docs/prototypes/data-binding-graph-string-converter-family-public-update-target-to-source-runtime-contract.md`.
- Main-`ToSource | TwoWay` target-to-source behavior for converter groups.
- Cross-type converter groups such as `DataConverterToString` followed by
  string converters.
- Number-to-number, list, formula, operation, range, rounder, interpolator,
  number-to-list, and scripted converter groups.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Public source handles, list and view-model bindables, imported and owned
  view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- Mutating the grouped main-`ToTarget | TwoWay` string target and calling
  public `updateDataBinds(true)` drains the dirty bind.
- The edited target writes to the default view-model string source through C++
  group reverse order.
- The same public update reapplies source-to-target through the forward
  trim-then-pad group pipeline.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing direct string converter-family and string converter-group
  target-dirty tests still pass.
