# Data Binding Graph ToString Enum Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin C++ state-machine target-dirty behavior for the display-label-unsupported
`DataConverterToString` enum-to-string runtime graph path on
main-`ToTarget | TwoWay` data binds.

The direct converter API can stringify enum values when metadata is available,
but this state-machine graph path does not use that display label conversion.
Adding `TwoWay` flags and mutating the string target must not make Rust write
enum labels that C++ does not write. C++ reapplies an empty string fallback on
the next normal state-machine advance.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceEnum.propertyValue` sources with resolvable imported enum
  metadata.
- `BindablePropertyString.propertyValue` targets.
- Direct `DataConverterToString` on the data bind, kept unsupported for enum
  display labels in this runtime graph path.
- Main-`ToTarget | TwoWay` flags, without the `ToSource` direction flag.
- Explicit data-context advancement preserving the manual string target edit.
- Subsequent normal state-machine advancement overwriting the string target
  with the C++ empty-string enum fallback.
- Exact C++ `stringBindings` probe reporting for the mutating string target
  after each explicit runtime action.

## Out Of Scope

- Enum display-label conversion for string-target graph bindings.
- Carrying enum display/key tables in runtime graph source values.
- Owned or external view-model enum-to-string contexts.
- Enum source mutation APIs for string-target enum converter binds.
- String trim, remove-zero, pad, converter-group, number, boolean, string,
  color, trigger, and symbol-list-index dirty behavior.
- Main-`ToSource | TwoWay` target-to-source behavior.
- Immediate reverse conversion for main-`ToTarget | TwoWay` binds.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity outside
  the state-machine bindable-property action path.
- Broader dirty/update queues, relative paths, parent paths, nested paths,
  listener-owned data binding, and nested artboard propagation.

## Completion Checks

- A `TwoWay` direct enum-to-string bind does not apply enum metadata text
  before target mutation.
- Mutating the string target does not write back to the enum source.
- Explicit data-context advancement preserves the manual string target edit.
- Normal state-machine advancement after the explicit data-context step
  overwrites the target with the C++ empty-string fallback, including when
  elapsed time is zero.
- The mutating bind's exact string target value matches the C++ probe after
  each explicit runtime action.
