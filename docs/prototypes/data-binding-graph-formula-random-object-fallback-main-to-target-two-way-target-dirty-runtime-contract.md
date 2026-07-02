# Data Binding Graph Formula Random Object Fallback Main-To-Target Two-Way Target-Dirty Runtime Contract

## Purpose

Pin main `ToTarget | TwoWay` target-dirty scheduling for random object-like
`DataConverterFormula` fallback sources.

This slice proves that a manual edit to a formula-bound number target is
preserved through explicit data-context advancement, then overwritten on the
next normal state-machine advancement by reapplying the unchanged asset,
artboard, or view-model pointer source through the random formula fallback
path. C++ does not request a random value for this observable target-dirty
path.

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
- Main `ToTarget | TwoWay` target-dirty behavior after a manual target edit.
- C++ probe coverage comparing number targets and counted random-provider
  calls.

## Out Of Scope

- Source-to-target random object fallback, covered by
  `data-binding-graph-formula-random-object-fallback-runtime-contract.md`.
- Explicit target-to-source scheduling, covered by
  `data-binding-graph-formula-random-object-fallback-explicit-target-to-source-runtime-contract.md`.
- Public target-to-source scheduling, covered by
  `data-binding-graph-formula-random-object-fallback-public-update-target-to-source-runtime-contract.md`.
- Imported and owned runtime contexts for this converter/source combination.
- Source mutation APIs and public object-handle APIs for these formula
  object-source binds.
- Formula converter groups beyond any composition already admitted by the
  generic graph group executor.
- Relative-path, parent-path, nested-path, listener-owned, nested-artboard, and
  full dirty-list scheduler behavior.

## Completion Checks

- Explicit data-context advancement preserves the manually edited number target
  for all object-like random fallback source kinds.
- Later normal state-machine advancement reapplies C++'s `0.0` fallback from
  the unchanged object-like source.
- C++ and Rust both report zero random-provider calls across the target-dirty
  sequence.
- The resulting number target reports match C++ for all three object-like
  source kinds and random modes `0`, `1`, and `2`.
