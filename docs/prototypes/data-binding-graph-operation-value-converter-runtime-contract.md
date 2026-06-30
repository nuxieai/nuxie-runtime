# Data Binding Graph OperationValue Converter Runtime Contract

## Purpose

Admit the first `DataConverterOperationValue` forward-conversion path in the
runtime data-binding graph.

This slice keeps the operation lane narrow: default-context number sources
feeding number targets can use imported `operationType` and `operationValue`
without adding formula, view-model lookup, or reverse-propagation behavior.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Imported `operationType` from `DataConverterOperation`.
- Imported `operationValue` from `DataConverterOperationValue`.
- C++ `DataConverterOperation::convertValue` forward arithmetic behavior for
  number inputs.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer,
  covering every `ArithmeticOperation` discriminant.

## Out Of Scope

- Reverse conversion.
- `DataConverterOperationViewModel`.
- `DataConverterSystemNormalizer` and `DataConverterSystemDegsToRads`.
- Symbol-list-index inputs to operation converters.
- Formula, interpolator, number-to-list, and scripted converters.
- Converter group compositions involving operation converters.
- External and owned contexts for this converter.
- Update-queue, relative-path, parent-path, nested-path, and listener-owned
  behavior.

## Completion Checks

- Default number sources are converted before state-machine evaluation for
  every C++ `ArithmeticOperation` discriminant.
- The converted number drives the same blend-state report as C++.
- Existing number source, mutation, and converter probes continue to pass.
