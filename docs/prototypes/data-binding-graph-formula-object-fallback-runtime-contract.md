# Data Binding Graph Formula Object Fallback Runtime Contract

## Purpose

Close the direct default-context C++ `DataConverterFormula` fallback path for
object-like source values represented by the runtime data-binding graph.

This slice extends number-target formula fallback beyond scalar and list source
values. C++ admits asset, artboard, and view-model pointer inputs into
`DataConverterFormula::convert`; because they are not number or
symbol-list-index values, direct formula conversion writes the observable
number fallback `0.0`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceAssetImage.propertyValue`,
  `ViewModelInstanceArtboard.propertyValue`, and
  `ViewModelInstanceViewModel.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- Deterministic `FormulaTokenInput` conversion.
- C++ probe coverage through a `BlendState1DViewModel` consumer with a
  non-zero imported target default, proving each object-source bind writes
  `0.0` rather than being skipped.

## Out Of Scope

- Boolean, enum, color, string, trigger, and list fallback paths covered by
  earlier contracts.
- Formula random functions and other `FormulaTokenFunction` behavior for these
  object-like sources.
- Formula target-to-source, public-update, and target-dirty scheduling for
  these source kinds.
- Imported and owned runtime contexts for this converter/source combination.
- Source mutation APIs for these formula object-source binds.
- Formula converter groups beyond any composition already admitted by the
  generic graph group executor.
- Relative-path, parent-path, nested-path, listener-owned, nested-artboard, and
  full dirty-list scheduler behavior.

## Completion Checks

- Default asset, artboard, and view-model pointer sources are admitted to the
  number-target formula converter path.
- The formula converter writes `0.0` to the number target for each object-like
  source kind before state-machine evaluation.
- The converted numbers drive the same blend-state reports as C++.
- Existing formula fallback probes continue to pass.
