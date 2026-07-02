# Data Binding Graph Formula Boolean Fallback Runtime Contract

## Purpose

Admit the first C++ `DataConverterFormula` non-number fallback path in the
runtime data-binding graph.

This slice closes one source-type gap left by deterministic formula execution:
default-context `ViewModelInstanceBoolean.propertyValue` sources can flow into a
formula converter, where C++ returns `0.0` before evaluating formula tokens
because boolean values are neither number nor symbol-list-index inputs.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- C++ `DataConverterFormula::convert` behavior that returns `0.0` for boolean
  inputs before formula token evaluation.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer with
  a non-zero imported target default, proving the bind writes `0.0` rather than
  being skipped.

## Out Of Scope

- Enum, color, string, and trigger fallback behavior is covered separately by
  `data-binding-graph-formula-remaining-fallbacks-runtime-contract.md`; list
  fallback behavior is covered separately by
  `data-binding-graph-formula-list-fallback-runtime-contract.md`.
- Asset, artboard, view-model pointer, and other non-number formula inputs.
- `FormulaTokenFunction`, random formula values, and `randomModeValue`.
- Formula parent-source binding, source dependents, and add-dirt behavior.
- Reverse conversion and target-to-source propagation.
- Formula converter groups beyond any composition already admitted by the
  generic graph group executor.
- Number-to-list, generated-list, scripted, and live context-aware converters.
- External and owned contexts for this converter/source combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- A default boolean source is admitted to the formula converter path.
- The formula converter writes `0.0` to the number target before state-machine
  evaluation.
- The converted number drives the same blend-state report as C++.
- Existing number and symbol-list-index formula probes continue to pass.
