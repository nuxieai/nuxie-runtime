# Data Binding Graph OperationValue SymbolListIndex Converter Runtime Contract

## Purpose

Admit the C++ `DataConverterOperationValue` symbol-list-index input path in the
runtime data-binding graph.

This slice closes the source-type gap left by the first OperationValue slice:
default-context `ViewModelInstanceSymbolListIndex.propertyValue` sources can be
cast to a number and then run through the already-supported operation-value
arithmetic before writing number targets.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Imported `operationType` from `DataConverterOperation`.
- Imported `operationValue` from `DataConverterOperationValue`.
- C++ `DataConverterOperation::convertValue` behavior that casts
  `DataValueSymbolListIndex` to `float` before applying the arithmetic
  operation.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer.

## Out Of Scope

- Reverse conversion.
- `DataConverterOperationViewModel`.
- `DataConverterSystemNormalizer` and `DataConverterSystemDegsToRads`.
- Formula, interpolator, number-to-list, and scripted converters.
- Converter group compositions involving operation converters.
- External and owned contexts for this converter/source combination.
- Update-queue, relative-path, parent-path, nested-path, and listener-owned
  behavior.

## Completion Checks

- A default symbol-list-index source is converted to `f32` before the
  operation-value arithmetic runs.
- The converted number drives the same blend-state report as C++.
- Existing symbol-list-index and OperationValue converter probes continue to
  pass.
