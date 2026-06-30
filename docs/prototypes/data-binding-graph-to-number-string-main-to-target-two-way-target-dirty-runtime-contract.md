# Data Binding Graph ToNumber String Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ state-machine target-dirty behavior for direct
`DataConverterToNumber` on main-`ToTarget | TwoWay` string-to-number binds.

For this state-machine bindable-property action path, C++ does not immediately
run `reverseConvert` or write the manually edited number target back to the
string source. The manual target edit survives explicit
`advancedDataContext()`, and the next normal `StateMachineInstance::advance()`
reapplies source-to-target conversion through `DataConverterToNumber::convert`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceString.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterToNumber` on a `TwoWay` number data bind without the
  `ToSource` direction flag.
- C++ `std::atof`-style numeric-prefix parsing as modeled by the binary crate.
- Initial string-to-number flushing through a normal state-machine advance.
- Explicit `advancedDataContext()` preserving the manual target edit before the
  next normal state-machine advance overwrites it.
- Exact C++ probe reporting for the mutating number bind's target value after
  each explicit runtime action.

## Out Of Scope

- Main-`ToSource | TwoWay` target-to-source behavior for `DataConverterToNumber`.
- Immediate target-to-source reverse conversion for main-`ToTarget | TwoWay`
  binds.
- `DataConverterToNumber` boolean, enum, color, and symbol-list-index dirty
  paths.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path.
- Full dirty-list scheduler parity for neighboring ordinary `ToTarget`
  bindable targets.
- Converter groups, formulas, operation converters, range mapper, rounder,
  system converters, interpolators, number-to-list, list, and scripted
  converters.
- Imported and owned view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  listener-owned data binding, nested artboards, and render/layout behavior.

## Completion Checks

- The initial normal state-machine advance writes the parsed string number to
  the bindable number target.
- Mutating the `DataConverterToNumber` target on a main-`ToTarget | TwoWay`
  bind preserves the manual number value through explicit data-context
  advancement.
- The next normal state-machine advance overwrites the target from the
  unchanged string source using C++ `DataConverterToNumber` forward conversion.
- The mutating bind's exact target value matches the C++ probe after each
  explicit runtime action.
