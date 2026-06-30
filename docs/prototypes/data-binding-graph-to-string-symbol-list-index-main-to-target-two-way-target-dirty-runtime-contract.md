# Data Binding Graph ToString Symbol List Index Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ state-machine target-dirty behavior for direct
`DataConverterToString` symbol-list-index-to-string binds on
main-`ToTarget | TwoWay` data binds.

For this state-machine bindable-property action path, mutating the
`BindablePropertyString.propertyValue` target does not write the edited string
back to the symbol-list-index source. Explicit `advancedDataContext()`
preserves the manual target edit, and the next normal
`StateMachineInstance::advance()` reapplies source-to-target conversion from
the unchanged raw symbol-list-index source.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources.
- `BindablePropertyString.propertyValue` targets.
- Direct `DataConverterToString` symbol-list-index decimal text conversion.
- Main-`ToTarget | TwoWay` flags, without the `ToSource` direction flag.
- Explicit data-context advancement preserving the manual string target edit.
- Subsequent normal state-machine advancement overwriting the string target
  from the unchanged raw symbol-list-index source.
- Exact C++ `stringBindings` probe reporting for the mutating string target
  after each explicit runtime action.

## Out Of Scope

- `DataConverterToString` input kinds other than symbol-list index.
- Public symbol-list-index source mutation APIs and stable public source
  handles.
- Symbol-list label lookup or enum display-name behavior.
- String trim, remove-zero, pad, converter-group, number, boolean, string,
  color, trigger, and enum-to-string dirty behavior.
- Main-`ToSource | TwoWay` target-to-source behavior.
- Immediate reverse conversion for main-`ToTarget | TwoWay` binds.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path.
- Broader dirty/update queues, relative paths, parent paths, nested paths,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- A `TwoWay` direct symbol-list-index-to-string bind applies the raw source
  index as decimal text before target mutation.
- Mutating the string target does not write back to the symbol-list-index
  source.
- Explicit data-context advancement preserves the manual string target edit.
- Normal state-machine advancement after the explicit data-context step
  overwrites the target from the unchanged source, including when elapsed time
  is zero.
- The mutating bind's exact string target value matches the C++ probe after
  each explicit runtime action.
