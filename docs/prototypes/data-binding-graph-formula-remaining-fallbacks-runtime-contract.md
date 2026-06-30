# Data Binding Graph Formula Remaining Fallbacks Runtime Contract

## Purpose

Complete the currently represented C++ `DataConverterFormula` non-number
fallback source set in the runtime data-binding graph.

This slice extends the boolean fallback path to the other default-context graph
values that are already represented for number-target data binds: enum, color,
string, and trigger. C++ formula conversion returns `0.0` for each of these
source kinds because they are neither number nor symbol-list-index inputs.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceEnum.propertyValue`,
  `ViewModelInstanceColor.propertyValue`,
  `ViewModelInstanceString.propertyValue`, and
  `ViewModelInstanceTrigger.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- C++ `DataConverterFormula::convert` behavior that returns `0.0` for the
  admitted non-number, non-symbol-list-index input kinds before formula token
  evaluation.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer with
  a non-zero imported target default, proving each bind writes `0.0` rather
  than being skipped.

## Out Of Scope

- Asset, artboard, view-model pointer, list, and other source kinds not
  admitted by this number-target graph path.
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

- Default enum, color, string, and trigger sources are admitted to the formula
  converter path.
- The formula converter writes `0.0` to the number target for each source kind
  before state-machine evaluation.
- The converted numbers drive the same blend-state reports as C++.
- Existing number, symbol-list-index, and boolean formula probes continue to
  pass.
