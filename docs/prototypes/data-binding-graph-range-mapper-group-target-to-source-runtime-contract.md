# Data Binding Graph RangeMapper Group Target-To-Source Runtime Contract

## Purpose

Pin the first `DataConverterGroup` target-to-source path that includes a
`DataConverterRangeMapper`.

C++ chooses converter direction from the data bind's main direction. For the
representative main-`ToSource` fixture in this contract,
`DataConverterGroup::convert` runs children from first to last before writing
the view-model source. A grouped range mapper therefore uses ordinary forward
range mapping in item order, not `reverseConvert` simply because the data flow
is target-to-source.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterGroup` on a `ToSource | TwoWay` number data bind.
- Ordered group items resolving to `DataConverterRangeMapper` followed by
  `DataConverterOperationValue`.
- Main-direction group conversion by applying those child converters in item
  order before writing the source.
- Exact C++ probe reporting for the mutating grouped bind and a second direct
  number bind to the same source path after normal state-machine advancement.

## Out Of Scope

- Main-`ToTarget | TwoWay` reverse group scheduling beyond the public
  mutating-bind slice covered by
  `docs/prototypes/data-binding-graph-range-mapper-group-public-update-target-to-source-runtime-contract.md`.
- Broader public `DataBindContainer::updateDataBinds(true)` scheduler parity.
- Exact `advancedDataContext()` source-to-target scheduling for neighboring
  ordinary `ToTarget` observer binds.
- Stateful `DataConverterInterpolator` children.
- Resolved-interpolator range-mapper children in target-to-source groups.
- Formula, number-to-list, generated-list, list, scripted, and stateful
  converter scheduling.
- Non-number sources or targets.
- Imported and owned view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  relative/parent/nested lookup, listener-owned data binding, nested artboards,
  and render/layout behavior.

## Completion Checks

- Mutating the grouped bindable number target on a main-`ToSource` bind runs
  the target value through `RangeMapper -> OperationValue` before writing the
  C++ source value.
- A second direct number bind to the same source path observes that written
  source value after normal source-to-target application.
- The mutating and observing bind exact source/target reports match the C++
  probe after each explicit runtime action.
