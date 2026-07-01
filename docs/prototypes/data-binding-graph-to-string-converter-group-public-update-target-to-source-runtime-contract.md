# Data Binding Graph ToString Converter Group Public Update Target-To-Source Runtime Contract

## Purpose

Admit public `updateDataBinds(true)` target-to-source behavior for the
already-supported cross-type `DataConverterGroup` shape on a main-`ToTarget |
TwoWay` string bind.

The admitted group starts with `DataConverterToString` for a default-context
number source and then flows through `DataConverterStringPad`. C++ drains the
public update by running `DataConverterGroup::reverseConvert` from last child
to first. The edited string target remains a string after group reverse
conversion, so it does not write the number source. The same public update
still schedules source-to-target reapplication, causing the unchanged number
source to overwrite the edited target through forward `ToString -> Pad`
conversion.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyString.propertyValue` targets.
- A direct `DataConverterGroup` on a main-`ToTarget | TwoWay` string data bind.
- Ordered group items resolving to `DataConverterToString` then
  `DataConverterStringPad`.
- C++ `DataConverterGroup::reverseConvert` child order for string targets.
- No number-source mutation when reverse conversion yields a string value.
- Immediate source-to-target reapplication through the grouped forward
  converter pipeline.
- Exact C++ probe reporting for the mutating string bind's target value after
  each explicit runtime action.

## Out Of Scope

- Main-`ToSource | TwoWay` target-to-source behavior for this cross-type group.
- Cross-type converter groups whose first child is not `DataConverterToString`.
- Non-number sources feeding `DataConverterToString` converter groups.
- Direct `DataConverterToString` public-update behavior, covered by
  `docs/prototypes/data-binding-graph-to-string-public-update-target-to-source-runtime-contract.md`.
- Pure string converter-group public-update behavior, covered by
  `docs/prototypes/data-binding-graph-string-converter-group-public-update-target-to-source-runtime-contract.md`.
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
- Reverse group conversion does not mutate the number source when the edited
  target remains a string.
- The same public update reapplies source-to-target through the forward
  `DataConverterToString -> DataConverterStringPad` group pipeline.
- The mutating bind's exact target value matches the C++ probe after each
  explicit runtime action.
- Existing direct `DataConverterToString`, direct string converter, pure string
  converter-group, and cross-type group target-dirty tests still pass.
