# Data Binding Graph Formula Random Object Fallback Explicit Target-To-Source Runtime Contract

## Purpose

Pin explicit `advanceDataContext()` target-to-source scheduling for random
object-like `DataConverterFormula` fallback sources.

This slice proves the main-`ToSource | TwoWay` path for asset, artboard, and
view-model pointer sources feeding number targets through `FunctionType::random`
formula tokens. C++ evaluates the random formula while converting the edited
number target, preserves the unchanged object-like source, and later reapplies
that source through the normal early fallback path.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceAssetImage.propertyValue`,
  `ViewModelInstanceArtboard.propertyValue`, and
  `ViewModelInstanceViewModel.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters with `FunctionType::random` output
  tokens.
- `randomModeValue` values `0`, `1`, and `2`.
- Explicit `advanceDataContext()` target-to-source behavior for
  `ToSource | TwoWay` binds.
- C++ probe coverage comparing number targets and counted random-provider
  calls.

## Out Of Scope

- Source-to-target random object fallback, covered by
  `data-binding-graph-formula-random-object-fallback-runtime-contract.md`.
- Public `updateDataBinds(true)` and main `ToTarget | TwoWay` target-dirty
  random scheduling for these object-like sources.
- Imported and owned runtime contexts for this converter/source combination.
- Source mutation APIs and public object-handle APIs for these formula
  object-source binds.
- Formula converter groups beyond any composition already admitted by the
  generic graph group executor.
- Relative-path, parent-path, nested-path, listener-owned, nested-artboard, and
  full dirty-list scheduler behavior.

## Completion Checks

- Explicit target-to-source application preserves the original asset,
  artboard, or view-model pointer source after a manual number-target edit.
- C++ and Rust both report one random-provider call during the edited
  target-to-source conversion and no extra calls during source reapplication.
- The resulting number target reports match C++ for all three object-like
  source kinds and random modes `0`, `1`, and `2`.
