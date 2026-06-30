# Data Binding Graph RangeMapper Target-To-Source Runtime Contract

## Purpose

Pin the next graph-owned number target-to-source converter family:
`DataConverterRangeMapper`.

C++ uses the data bind's main direction to choose converter direction. For a
main-`ToSource` range-mapper bind, target-to-source writes call
`DataConverterRangeMapper::convert`, not `reverseConvert`. The reverse
primitive still mirrors C++ `calculateReverseRange()` by swapping the input and
output ranges, but the full public `updateDataBinds(true)` scheduler path that
would reach it for main-`ToTarget | TwoWay` binds remains out of scope.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- `ToSource | TwoWay` number data binds with a direct
  `DataConverterRangeMapper`.
- Range-mapper fields `minInput`, `maxInput`, `minOutput`, `maxOutput`,
  `flags`, `interpolationType`, and optional resolved interpolator state as
  already admitted by the forward range-mapper contracts.
- Exact C++ probe reporting for the mutating range-mapped number bind and a
  second direct number bind to the same source path after normal
  state-machine advancement.
- Rust reverse-conversion primitive parity for the direct range-mapper numeric
  case, including preserving the `Reverse` flag after swapping ranges.

## Out Of Scope

- Public `DataBindContainer::updateDataBinds(true)` scheduler parity.
- Exact `advancedDataContext()` source-to-target scheduling for neighboring
  ordinary `ToTarget` observer binds.
- Main-`ToTarget | TwoWay` range-mapper target mutations through the public
  state-machine action path.
- Range-mapper groups in target-to-source runtime scheduling.
- Non-number range-mapper sources or targets.
- Formula, number-to-list, generated-list, scripted, and stateful converter
  scheduling.
- Imported and owned view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  relative/parent/nested lookup, listener-owned data binding, nested artboards,
  and render/layout behavior.

## Completion Checks

- Mutating the range-mapped bindable number target on a main-`ToSource` bind
  writes the C++ range-mapped source value.
- A second direct number bind to the same source path observes that written
  source value after normal source-to-target application.
- The mutating and observing bind exact source/target reports match the C++
  probe after each explicit runtime action.
- The Rust reverse primitive maps an output-domain number back into the input
  domain by swapping ranges and preserving range-mapper flags.
