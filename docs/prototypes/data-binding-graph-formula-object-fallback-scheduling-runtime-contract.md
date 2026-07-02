# Data Binding Graph Formula Object Fallback Scheduling Runtime Contract

## Purpose

Close deterministic target-to-source scheduling for object-like
`DataConverterFormula` fallback sources that feed number targets.

This slice builds on
`data-binding-graph-formula-object-fallback-runtime-contract.md`. C++ keeps the
asset, artboard, or view-model pointer source unchanged when a number target is
manually edited through a formula bind, then reapplies the unchanged source
through the formula fallback according to the active scheduling path.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceAssetImage.propertyValue`,
  `ViewModelInstanceArtboard.propertyValue`, and
  `ViewModelInstanceViewModel.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct deterministic `DataConverterFormula` converters using
  `FormulaTokenInput`.
- Explicit `advanceDataContext()` target-to-source behavior for
  `ToSource | TwoWay` binds.
- Public `updateDataBinds(true)` target-to-source behavior for
  `ToTarget | TwoWay` binds.
- Main `ToTarget | TwoWay` target-dirty behavior after a manual target edit.
- C++ probe coverage comparing the resulting number target after each
  scheduling step.

## Out Of Scope

- Initial direct source-to-target object fallback, covered by
  `data-binding-graph-formula-object-fallback-runtime-contract.md`.
- Formula random functions and function-token object fallbacks.
- Imported and owned runtime contexts for this converter/source combination.
- Source mutation APIs and public object-handle APIs for these formula
  object-source binds.
- Formula converter groups beyond any composition already admitted by the
  generic graph group executor.
- Relative-path, parent-path, nested-path, listener-owned, nested-artboard, and
  full dirty-list scheduler behavior.

## Completion Checks

- Manual edits to formula-bound number targets do not replace the underlying
  asset, artboard, or view-model pointer source with a number.
- Explicit target-to-source, public-update target-to-source, and target-dirty
  paths match C++ for all three object-like source kinds.
- Existing formula fallback probes continue to pass.
