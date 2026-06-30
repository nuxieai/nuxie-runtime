# Data Binding Graph String Converter Group Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ state-machine target-dirty behavior for a direct
`DataConverterGroup` whose children are already-supported string converters.

This slice proves that grouped string conversion participates in the same
main-`ToTarget | TwoWay` dirty path as direct string converters, without
admitting broader data-bind scheduler behavior or reverse group conversion.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- Root-only source paths of shape `[0, propertyIndex]`.
- `ViewModelInstanceString.propertyValue` sources.
- `BindablePropertyString.propertyValue` targets.
- A direct `DataConverterGroup` on the data bind whose children resolve
  through imported `DataConverterGroupItem.converterId`.
- The already-admitted string group shape: `DataConverterStringTrim` followed
  by `DataConverterStringPad`.
- Main-`ToTarget | TwoWay` flags, without the `ToSource` direction flag.
- Explicit `advancedDataContext()` preserving a manual target edit before the
  next normal state-machine advance overwrites it.
- Exact C++ probe reporting for source and target string binding values after
  each explicit runtime action.

## Out Of Scope

- Cross-type converter groups such as `DataConverterToString` followed by
  string converters.
- Number-to-number, list, formula, operation, range, rounder, interpolator,
  number-to-list, and scripted converter groups.
- Stateful converter advancement and converter reset/dirty scheduling.
- Main-`ToSource | TwoWay` target-to-source behavior.
- Immediate reverse conversion for main-`ToTarget | TwoWay` binds.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity.
- Dirty/update queue parity beyond this state-machine target-dirty path.
- Owned or external view-model converter-group contexts.
- Relative, parent, and nested source path lookup.
- Listener-owned data binding and nested artboard propagation.

## Completion Checks

- A `TwoWay` string converter-group bind applies source-to-target through the
  ordered trim-then-pad group before the manual target edit.
- Mutating the grouped string target preserves the manual edit through explicit
  data-context advancement.
- The next normal `StateMachineInstance::advance()` reapplies the unchanged
  source through the imported group in C++ child order, including when elapsed
  time is zero.
- The mutating bind's exact source and target string values match the C++
  probe after each explicit runtime action.
- Existing direct string converter and ordinary string converter-group probes
  continue to pass.
