# Data Binding Graph Number Converter Group Main-To-Source Target-To-Source Runtime Contract

## Purpose

Admit main-`ToSource | TwoWay` target-to-source behavior for the first
number-to-number `DataConverterGroup` runtime graph shape.

The admitted group uses `DataConverterOperationValue` followed by
`DataConverterRounder`. During explicit data-context advancement, C++ treats
main `ToSource` as the main converter direction, so the edited number target
flows through `DataConverterGroup::convert` in child order before writing the
default view-model number source. The same dirty pass refreshes the visible
target through `DataConverterGroup::reverseConvert` in reverse child order,
which means the rounded source value is exposed through the inverse
`Rounder -> OperationValue` path.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterGroup` on a main-`ToSource | TwoWay` number data bind.
- Ordered group items resolving to `DataConverterOperationValue` then
  `DataConverterRounder`.
- Explicit `advanceDataContext()` target-to-source scheduling for number
  bindables.
- Main-direction group `convert` child order before the number source write.
- Same-pass source-to-target refresh through group `reverseConvert` child
  order after the source write.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- Public `updateDataBinds(true)` behavior for this number group, covered by
  `docs/prototypes/data-binding-graph-number-converter-group-public-update-target-to-source-runtime-contract.md`.
- Operation-value-only and range-mapper group target-to-source behavior,
  covered by their dedicated contracts.
- Formula, operation-view-model, interpolator, number-to-list, list, string,
  cross-type, and scripted converter groups.
- Stateful converter advancement inside groups.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Public source handles, list and view-model bindables, imported and owned
  view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- Initial explicit data-context advancement matches C++ for the admitted
  number converter group.
- Mutating the grouped main-`ToSource | TwoWay` number target and explicitly
  advancing data context writes the number source through forward
  `OperationValue -> Rounder` group order.
- The same dirty pass refreshes the number target through reverse
  `Rounder -> OperationValue` group order.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing direct number-converter, operation-value group, range-mapper group,
  string group, cross-type group, and public-update number group tests still
  pass.
