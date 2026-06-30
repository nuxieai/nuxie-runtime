# Data Binding Graph ToString Color Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ state-machine target-dirty behavior for direct
`DataConverterToString` color-to-string binds on main-`ToTarget | TwoWay` data
binds.

For this state-machine bindable-property action path, mutating the
`BindablePropertyString.propertyValue` target does not write the edited string
back to the color source. Explicit `advancedDataContext()` preserves the
manual target edit, and the next normal `StateMachineInstance::advance()`
reapplies source-to-target conversion from the unchanged color source using
the imported `DataConverterToString.colorFormat`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceColor.propertyValue` sources.
- `BindablePropertyString.propertyValue` targets.
- Direct `DataConverterToString` color formatting through imported
  `colorFormat` bytes.
- Main-`ToTarget | TwoWay` flags, without the `ToSource` direction flag.
- Explicit data-context advancement preserving the manual string target edit.
- Subsequent normal state-machine advancement overwriting the string target
  from the unchanged color source.
- Exact C++ `stringBindings` probe reporting for the mutating string target
  after each explicit runtime action.

## Out Of Scope

- `DataConverterToString` input kinds other than color.
- Color source mutation APIs beyond the existing default-context graph binding.
- Additional color format markers beyond the already admitted direct
  color-to-string formatter behavior.
- String trim, remove-zero, pad, converter-group, number, boolean, string,
  trigger, symbol-list-index, and enum-to-string dirty behavior.
- Main-`ToSource | TwoWay` target-to-source behavior.
- Immediate reverse conversion for main-`ToTarget | TwoWay` binds.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path.
- Broader dirty/update queues, relative paths, parent paths, nested paths,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- A `TwoWay` direct color-to-string bind applies the formatted color before
  target mutation.
- Mutating the string target does not write back to the color source.
- Explicit data-context advancement preserves the manual string target edit.
- Normal state-machine advancement after the explicit data-context step
  overwrites the target from the unchanged source, including when elapsed time
  is zero.
- The mutating bind's exact string target value matches the C++ probe after
  each explicit runtime action.
