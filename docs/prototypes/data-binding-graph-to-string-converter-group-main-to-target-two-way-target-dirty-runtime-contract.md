# Data Binding Graph ToString Converter Group Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ state-machine target-dirty behavior for the admitted cross-type string
converter group: `DataConverterToString` followed by an already-supported
direct string converter.

This slice proves that group source admission through `DataConverterToString`
also participates in the main-`ToTarget | TwoWay` dirty replay path, without
admitting broader group families, reverse conversion, or data-bind scheduler
parity.

## In Scope

- State-machine-owned `DataBindContext` objects whose target property is
  `BindablePropertyString.propertyValue`.
- Default root view-model contexts resolved through serialized
  `DataBindContext.sourcePathIds`.
- Root-only source paths of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources.
- `BindablePropertyString.propertyValue` targets.
- A direct `DataConverterGroup` on the data bind whose first effective child is
  `DataConverterToString`.
- Ordered group conversion from imported `DataConverterGroupItem.converterId`
  records, specifically `DataConverterToString` followed by
  `DataConverterStringPad`.
- Main-`ToTarget | TwoWay` flags, without the `ToSource` direction flag.
- Explicit `advancedDataContext()` preserving a manual target edit before the
  next normal state-machine advance overwrites it.
- Exact C++ probe reporting for source and target string binding values after
  each explicit runtime action.

## Out Of Scope

- Non-number source admission through converter groups.
- Groups whose first effective converter is not `DataConverterToString`.
- Boolean, trigger, color, enum, asset, artboard, list, or view-model sources
  through converter groups.
- Number, boolean, trigger, color, enum, asset, artboard, list, or view-model
  targets.
- Formula, operation, range, rounder, interpolator, number-to-list, list, and
  scripted converter groups.
- Stateful converter advancement and converter reset/dirty scheduling.
- Main-`ToSource | TwoWay` target-to-source behavior.
- Immediate reverse conversion for main-`ToTarget | TwoWay` binds.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity.
- Dirty/update queue parity beyond this state-machine target-dirty path.
- Owned or external view-model converter-group contexts.
- Relative, parent, and nested source path lookup.
- Listener-owned data binding and nested artboard propagation.

## Completion Checks

- A `TwoWay` number-to-string converter-group bind applies source-to-target
  through ordered `DataConverterToString -> DataConverterStringPad` conversion
  before the manual target edit.
- Mutating the grouped string target preserves the manual edit through explicit
  data-context advancement.
- The next normal `StateMachineInstance::advance()` reapplies the unchanged
  number source through the imported group in C++ child order, including when
  elapsed time is zero.
- The mutating bind's exact source and target string values match the C++
  probe after each explicit runtime action.
- Existing direct `DataConverterToString`, direct string-converter,
  string-source converter-group, and ordinary number-to-string converter-group
  probes continue to pass.
