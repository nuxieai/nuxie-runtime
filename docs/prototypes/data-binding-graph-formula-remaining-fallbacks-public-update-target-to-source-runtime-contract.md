# Data Binding Graph Formula Remaining Fallbacks Public-Update Target-To-Source Runtime Contract

## Purpose

Extend non-number formula fallback public-update reverse behavior beyond the
first boolean slice.

For main-`ToTarget | TwoWay` number targets fed by enum, color, string, or
trigger sources through `DataConverterFormula`, C++ reverse-converts the edited
number target through formula `convert`, producing a number. That number cannot
be written back into the original source type, so the source remains unchanged.
The same public update then reapplies the unchanged source through the formula
fallback, restoring the number target to `0.0`.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceEnum.propertyValue`,
  `ViewModelInstanceColor.propertyValue`,
  `ViewModelInstanceString.propertyValue`, and
  `ViewModelInstanceTrigger.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterFormula` on a main-`ToTarget | TwoWay` number data
  bind.
- Deterministic `FormulaTokenInput` fallback behavior for the admitted source
  kinds.
- Type-mismatch source preservation when reverse formula conversion produces a
  number for an enum, color, string, or trigger source.
- Immediate source-to-target reapplication during public update.
- C++ probe matrix reporting the bind source and target values after each
  explicit runtime action.

## Out Of Scope

- Boolean fallback public-update reverse behavior, covered separately by
  `data-binding-graph-formula-boolean-fallback-public-update-target-to-source-runtime-contract.md`.
- Main-`ToSource | TwoWay` explicit data-context target-to-source behavior for
  non-number formula fallback sources.
- List, symbol-list-index, asset, artboard, and view-model pointer formula
  fallback reverse behavior.
- `FormulaTokenFunction`, random formula values, and `randomModeValue`.
- Formula converter groups.
- Formula parent-source binding, source dependents, and general add-dirt
  behavior.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated main-`ToTarget | TwoWay` formula-bound number target does not write
  its reverse-converted number into enum, color, string, or trigger sources.
- The public update reapplies each unchanged source through the formula
  fallback, restoring the number target to `0.0`.
- The exact source and target reports match the C++ probe after bind,
  mutation/public-update, and later state-machine advances.
- Existing boolean fallback public-update, deterministic formula fallback, and
  formula target-to-source probes continue to pass.
