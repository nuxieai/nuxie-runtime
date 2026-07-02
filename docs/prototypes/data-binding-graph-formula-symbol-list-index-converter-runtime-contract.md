# Data Binding Graph Formula SymbolListIndex Converter Runtime Contract

## Purpose

Admit the C++ `DataConverterFormula` symbol-list-index input path in the
runtime data-binding graph.

This slice closes the source-type gap left by the first deterministic formula
slice: default-context `ViewModelInstanceSymbolListIndex.propertyValue` sources
can be cast to a number and then run through the already-supported deterministic
formula evaluator before writing number targets.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- Deterministic formula output queues made only from `FormulaTokenInput`,
  `FormulaTokenValue`, and `FormulaTokenOperation`.
- C++ `DataConverterFormula::convert` behavior that casts
  `DataValueSymbolListIndex` to `float` before formula evaluation.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer.

## Out Of Scope

- Deterministic `FormulaTokenFunction` support is covered separately by
  `data-binding-graph-formula-functions-runtime-contract.md`; symbol-list-index
  random formula support is covered separately by
  `data-binding-graph-formula-random-symbol-list-index-runtime-contract.md`.
- Non-number and non-symbol-list-index formula inputs.
- Formula parent-source binding, source dependents, and add-dirt behavior.
- Reverse conversion and target-to-source propagation.
- Formula converter groups beyond any composition already admitted by the
  generic graph group executor.
- Number-to-list, generated-list, scripted, and live context-aware converters.
- External and owned contexts for this converter/source combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- A default symbol-list-index source is converted to `f32` before formula
  evaluation.
- The converted number drives the same blend-state report as C++.
- Existing number-input formula and symbol-list-index converter probes continue
  to pass.
