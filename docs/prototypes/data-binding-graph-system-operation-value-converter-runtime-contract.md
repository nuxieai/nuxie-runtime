# Data Binding Graph System OperationValue Converter Runtime Contract

## Purpose

Admit direct `DataConverterSystemNormalizer` and
`DataConverterSystemDegsToRads` execution in the runtime data-binding graph for
the current source-to-target lane.

These converters inherit `DataConverterOperationValue` and choose forward or
reverse operation-value arithmetic from `DataBind.flags`. This slice models
only the cases where C++ `DataBind::toTarget()` is true.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterSystemNormalizer` converters.
- Direct `DataConverterSystemDegsToRads` converters.
- Imported `operationType` and `operationValue`.
- Source-to-target C++ direction handling for:
  - default `ToTarget` flags, using forward operation-value arithmetic;
  - `TwoWay | ToSource` flags, using reverse operation-value arithmetic.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer.

## Out Of Scope

- `ToSource`-only binds where C++ `DataBind::toTarget()` is false.
- Target-to-source propagation beyond the direct main-`ToSource | TwoWay`
  number case covered by
  `docs/prototypes/data-binding-graph-system-operation-value-target-to-source-runtime-contract.md`.
- Public update-queue scheduling.
- Reverse graph propagation as a public runtime operation.
- Symbol-list-index inputs to system converters.
- `DataConverterOperationViewModel`.
- Formula, interpolator, number-to-list, and scripted converters.
- Converter group compositions involving system converters.
- External and owned contexts for these converters.
- Relative-path, parent-path, nested-path, and listener-owned behavior.

## Completion Checks

- Direct system converters with default `ToTarget` flags use forward
  operation-value arithmetic before state-machine evaluation.
- Direct system converters with `TwoWay | ToSource` flags use reverse
  operation-value arithmetic in the source-to-target pass.
- The converted numbers drive the same blend-state reports as C++.
- Existing OperationValue and number-source converter probes continue to pass.
