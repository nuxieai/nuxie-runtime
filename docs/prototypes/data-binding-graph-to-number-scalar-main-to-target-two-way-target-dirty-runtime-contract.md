# Data Binding Graph ToNumber Scalar Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ state-machine target-dirty behavior for the remaining direct
`DataConverterToNumber` scalar source kinds on main-`ToTarget | TwoWay` number
binds: boolean, enum, color, and symbol-list-index.

For this state-machine bindable-property action path, C++ does not immediately
run `reverseConvert` or write the manually edited number target back to the
source. The manual target edit survives explicit `advancedDataContext()`, and
the next normal `StateMachineInstance::advance()` reapplies source-to-target
conversion through `DataConverterToNumber::convert`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean.propertyValue` sources.
- `ViewModelInstanceEnum.propertyValue` sources.
- `ViewModelInstanceColor.propertyValue` sources.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterToNumber` on a `TwoWay` number data bind without the
  `ToSource` direction flag.
- Initial source-to-number flushing through a normal state-machine advance.
- Explicit `advancedDataContext()` preserving the manual target edit before the
  next normal state-machine advance overwrites it.
- Exact C++ probe reporting for the mutating number bind's target value after
  each explicit runtime action.

## Out Of Scope

- The string-source path, which is covered by
  `docs/prototypes/data-binding-graph-to-number-string-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Main-`ToSource | TwoWay` target-to-source behavior for `DataConverterToNumber`.
- Immediate target-to-source reverse conversion for main-`ToTarget | TwoWay`
  binds beyond the linked public-update slices.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path beyond
  `docs/prototypes/data-binding-graph-to-number-boolean-public-update-target-to-source-runtime-contract.md`
  and
  `docs/prototypes/data-binding-graph-to-number-remaining-public-update-target-to-source-runtime-contract.md`.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Converter groups, formulas, operation converters, range mapper, rounder,
  system converters, interpolators, number-to-list, list, and scripted
  converters.
- Public source handles, list and view-model bindables, imported and owned
  view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- The initial normal state-machine advance writes the converted source value to
  the bindable number target for boolean, enum, color, and symbol-list-index
  sources.
- Mutating the target on each main-`ToTarget | TwoWay` bind preserves the
  manual number value through explicit data-context advancement.
- The next normal state-machine advance overwrites the target from the
  unchanged source using C++ `DataConverterToNumber` forward conversion.
- The mutating bind's exact target value matches the C++ probe after each
  explicit runtime action.
