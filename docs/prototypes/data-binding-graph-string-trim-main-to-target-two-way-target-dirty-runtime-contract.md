# Data Binding Graph StringTrim Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ state-machine target-dirty behavior for direct
`DataConverterStringTrim` string-to-string binds on main-`ToTarget | TwoWay`
data binds.

For this state-machine bindable-property action path, mutating the
`BindablePropertyString.propertyValue` target does not write the edited string
back to the string source. Explicit `advancedDataContext()` preserves the
manual target edit, and the next normal `StateMachineInstance::advance()`
reapplies source-to-target conversion from the unchanged source through the
imported trim converter.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceString.propertyValue` sources.
- `BindablePropertyString.propertyValue` targets.
- Direct `DataConverterStringTrim` conversion through imported `trimType`.
- Main-`ToTarget | TwoWay` flags, without the `ToSource` direction flag.
- Explicit data-context advancement preserving the manual string target edit.
- Subsequent normal state-machine advancement overwriting the string target
  from the unchanged source through C++ trim conversion.
- Exact C++ `stringBindings` probe reporting for the mutating string source and
  target after each explicit runtime action.

## Out Of Scope

- `DataConverterStringRemoveZeros`, `DataConverterStringPad`, and converter
  groups.
- Non-string sources feeding `DataConverterStringTrim`.
- Owned or external view-model string-trim contexts.
- Main-`ToSource | TwoWay` target-to-source behavior.
- Immediate reverse conversion for main-`ToTarget | TwoWay` binds.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path.
- Broader dirty/update queues, relative paths, parent paths, nested paths,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- A `TwoWay` direct string-trim bind applies the trimmed source before target
  mutation.
- Mutating the string target does not write back to the string source.
- Explicit data-context advancement preserves the manual string target edit.
- Normal state-machine advancement after the explicit data-context step
  overwrites the target from the unchanged source through C++ trim conversion,
  including when elapsed time is zero.
- The mutating bind's exact string source and target values match the C++ probe
  after each explicit runtime action.
