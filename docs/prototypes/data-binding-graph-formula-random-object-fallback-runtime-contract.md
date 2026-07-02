# Data Binding Graph Formula Random Object Fallback Runtime Contract

## Purpose

Pin C++ `DataConverterFormula` random-function behavior for object-like source
values feeding number targets.

This slice proves that asset, artboard, and view-model pointer source values
take the same early formula fallback branch for `FunctionType::random` output
tokens as they do for deterministic input tokens. C++ writes `0.0` for these
non-number inputs and does not request a random value from the provider.

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
- C++ probe coverage comparing number targets and counted random-provider
  calls.

## Out Of Scope

- Deterministic input-token object fallback, covered by
  `data-binding-graph-formula-object-fallback-runtime-contract.md`.
- Target-to-source and target-dirty scheduling, covered by
  `data-binding-graph-formula-object-fallback-scheduling-runtime-contract.md`.
- Random-function target-to-source scheduling for these object-like sources.
- Imported and owned runtime contexts for this converter/source combination.
- Source mutation APIs and public object-handle APIs for these formula
  object-source binds.
- Formula converter groups beyond any composition already admitted by the
  generic graph group executor.
- Relative-path, parent-path, nested-path, listener-owned, nested-artboard, and
  full dirty-list scheduler behavior.

## Completion Checks

- Random-function object-like formula sources write `0.0` to the number target
  for random modes `0`, `1`, and `2`.
- C++ and Rust both report zero random-provider calls for these fallback paths.
- Existing formula fallback probes continue to pass.
