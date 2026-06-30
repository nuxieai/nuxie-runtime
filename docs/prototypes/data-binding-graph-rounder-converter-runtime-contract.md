# Data Binding Graph Rounder Converter Runtime Contract

## Purpose

Admit `DataConverterRounder` forward conversion in the runtime data-binding
graph.

This slice keeps converter expansion narrow: default-context number sources
feeding number targets can use imported `DataConverterRounder.decimals` before
the graph writes the bindable target.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Imported `DataConverterRounder.decimals`.
- Forward conversion matching C++:
  `round(value * pow(10, decimals)) / pow(10, decimals)`.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer.

## Out Of Scope

- Reverse conversion.
- Non-number inputs beyond C++ default-output fallback.
- External and owned contexts for this converter.
- Converter group compositions involving rounder.
- Range mapper, operation, formula, system, interpolator, and list converters.
- Update-queue, relative-path, parent-path, nested-path, and listener-owned
  behavior.

## Completion Checks

- A default number source rounded to one decimal updates a number bindable
  before state-machine evaluation.
- The rounded number drives the same blend-state report as C++.
- Existing number source, mutation, and converter probes continue to pass.
