# Data Binding Graph Number Converter Group Public Update Target-To-Source Runtime Contract

## Purpose

Admit public `updateDataBinds(true)` target-to-source behavior for the first
number-to-number `DataConverterGroup` runtime graph shape.

The admitted group uses `DataConverterOperationValue` followed by
`DataConverterRounder`. C++ drains the public update by running
`DataConverterGroup::reverseConvert` from last child to first before writing
the default view-model number source. The same public update then reapplies
source-to-target through the forward `OperationValue -> Rounder` group
pipeline.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterGroup` on a main-`ToTarget | TwoWay` number data bind.
- Ordered group items resolving to `DataConverterOperationValue` then
  `DataConverterRounder`.
- C++ `DataConverterGroup::reverseConvert` child order for number targets.
- Immediate source-to-target reapplication through the grouped forward
  converter pipeline.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Public-update coverage for operation-value-only number groups, covered by
  `docs/prototypes/data-binding-graph-operation-value-group-public-update-target-to-source-runtime-contract.md`.
- Public-update coverage for range-mapper groups, covered by
  `docs/prototypes/data-binding-graph-range-mapper-group-public-update-target-to-source-runtime-contract.md`.
- Main-`ToSource | TwoWay` target-to-source behavior for this number group.
- Formula, operation-view-model, interpolator, number-to-list, list, string,
  cross-type, and scripted converter groups.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Public source handles, list and view-model bindables, imported and owned
  view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- Mutating the grouped main-`ToTarget | TwoWay` number target and calling
  public `updateDataBinds(true)` drains the dirty bind.
- Reverse group conversion writes the C++-expected number source value through
  `Rounder -> OperationValue` reverse order.
- The same public update reapplies source-to-target through the forward
  `OperationValue -> Rounder` group pipeline.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing direct number-converter, operation-value group, range-mapper group,
  and forward number-converter group tests still pass.
