# Data Binding Graph System OperationValue Target-To-Source Runtime Contract

## Purpose

Admit direct `DataConverterSystemNormalizer` and
`DataConverterSystemDegsToRads` in the graph-owned target-to-source path for
main-`ToSource | TwoWay` number binds.

These converters inherit operation-value arithmetic but choose forward or
reverse math from the authored data-bind direction inside the converter
method. For a main-`ToSource | TwoWay` bind, C++ target-to-source dispatch calls
`convert`, and these system converters' `convert` methods then delegate to
`DataConverterOperationValue::reverseConvert`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` source values.
- `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterSystemNormalizer` converters.
- Direct `DataConverterSystemDegsToRads` converters.
- Imported `operationType` and `operationValue`.
- Main-`ToSource | TwoWay` target-to-source dispatch through `convert`.
- C++ system-converter direction handling that turns `convert` into
  operation-value reverse arithmetic when the authored direction is
  `ToSource`.
- Immediate same-bind source-to-target refresh after the target-to-source write
  changes the source.
- Exact C++ probe reporting for the mutating number bind's source and target
  values after each explicit runtime action.

## Out Of Scope

- `ToSource`-only binds without `TwoWay`.
- Main-`ToTarget | TwoWay` system-converter dirty behavior.
- Public `DataBindContainer::updateDataBinds(true)` scheduler parity and
  public-queue `reverseConvert` behavior.
- Symbol-list-index inputs to system converters.
- `DataConverterOperationViewModel`.
- Formula, interpolator, number-to-list, and scripted converters.
- Converter group compositions involving system converters.
- Imported and owned view-model contexts.
- Pending add/remove behavior, observer-list parity, re-entry protection,
  relative/parent/nested lookup, listener-owned data binding, nested artboards,
  and render/layout behavior.

## Completion Checks

- A mutated numeric target on a main-`ToSource | TwoWay`
  `DataConverterSystemNormalizer` bind is converted with C++ system-converter
  reverse operation-value arithmetic before writing the default view-model
  number source.
- The same behavior is pinned for `DataConverterSystemDegsToRads`.
- The same system-converter bindable target is refreshed from the changed
  source during explicit data-context advancement.
- The mutating bind's exact source and target values match the C++ probe after
  each explicit runtime action.
- Existing direct system converter source-to-target tests continue to pass.
