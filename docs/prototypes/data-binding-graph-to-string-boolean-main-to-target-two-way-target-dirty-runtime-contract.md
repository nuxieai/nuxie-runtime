# Data Binding Graph ToString Boolean Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ state-machine target-dirty behavior for direct
`DataConverterToString` boolean-to-string binds on main-`ToTarget | TwoWay`
data binds.

For this state-machine bindable-property action path, mutating the
`BindablePropertyString.propertyValue` target does not write the edited string
back to the boolean source. Explicit `advancedDataContext()` preserves the
manual target edit, and the next normal `StateMachineInstance::advance()`
reapplies source-to-target conversion from the unchanged boolean source.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean.propertyValue` sources.
- `BindablePropertyString.propertyValue` targets.
- Direct `DataConverterToString` boolean conversion.
- Main-`ToTarget | TwoWay` flags, without the `ToSource` direction flag.
- Explicit data-context advancement preserving the manual string target edit.
- Subsequent normal state-machine advancement overwriting the string target
  from the unchanged boolean source through C++ boolean-to-string conversion.
- Exact C++ `stringBindings` probe reporting for the mutating string target
  after each explicit runtime action.

## Out Of Scope

- `DataConverterToString` input kinds other than boolean.
- String trim, remove-zero, pad, converter-group, number, color, trigger,
  string, symbol-list-index, and enum-to-string dirty behavior.
- Main-`ToSource | TwoWay` target-to-source behavior.
- Immediate reverse conversion for main-`ToTarget | TwoWay` binds.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path.
- Broader dirty/update queues, relative paths, parent paths, nested paths,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- A `TwoWay` direct boolean-to-string bind applies the converted boolean value
  before target mutation.
- Mutating the string target does not write back to the boolean source.
- Explicit data-context advancement preserves the manual string target edit.
- Normal state-machine advancement after the explicit data-context step
  overwrites the target from the unchanged source, including when elapsed time
  is zero.
- The mutating bind's exact string target value matches the C++ probe after
  each explicit runtime action.
