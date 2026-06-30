# Data Binding Graph Number Converter Group Runtime Contract

## Purpose

Admit the first number-to-number `DataConverterGroup` path in the runtime
data-binding graph.

This slice verifies that the existing graph group executor composes
already-supported direct number converters for default-context number sources
feeding number targets.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterGroup` converters whose children resolve through
  imported `DataConverterGroupItem.converterId`.
- Group children limited to already-supported number-output converters.
- C++ forward group order.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer,
  using an `OperationValue -> Rounder` group.

## Out Of Scope

- Reverse conversion.
- Group children requiring live context binding, converter state, random
  formula inputs, generated lists, or target-to-source queues.
- `DataConverterOperationViewModel`.
- Formula, interpolator, number-to-list, and scripted converters.
- External and owned contexts for this group shape.
- Relative-path, parent-path, nested-path, and listener-owned behavior.

## Completion Checks

- A default number source flows through an imported
  `DataConverterOperationValue` child and then an imported
  `DataConverterRounder` child before state-machine evaluation.
- The grouped number drives the same blend-state report as C++.
- Existing direct number-converter and converter-group probes continue to pass.
