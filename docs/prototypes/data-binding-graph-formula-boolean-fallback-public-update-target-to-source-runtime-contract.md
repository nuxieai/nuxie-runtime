# Data Binding Graph Formula Boolean Fallback Public-Update Target-To-Source Runtime Contract

## Purpose

Admit the first non-number formula fallback public-update reverse path.

For a main-`ToTarget | TwoWay` number target fed by a boolean source through
`DataConverterFormula`, C++ applies the edited number target through
`DataConverterFormula::reverseConvert`, which delegates to `convert` and
produces a number. That number cannot be written back into the boolean source,
so the source remains unchanged. The same public update then reapplies the
unchanged boolean source through the formula fallback, restoring the number
target to `0.0`.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterFormula` on a main-`ToTarget | TwoWay` number data
  bind.
- Deterministic `FormulaTokenInput` fallback behavior for the boolean source.
- Type-mismatch source preservation when reverse formula conversion produces a
  number for the boolean source.
- Immediate source-to-target reapplication during public update.
- Exact C++ probe reporting for the bind's source and target values after each
  explicit runtime action.

## Out Of Scope

- Main-`ToSource | TwoWay` explicit data-context target-to-source behavior for
  boolean formula fallback sources.
- Enum, color, string, trigger, list, symbol-list-index, asset, artboard, and
  view-model pointer formula fallback reverse behavior.
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
  its reverse-converted number into the boolean source.
- The public update reapplies the unchanged boolean source through the formula
  fallback, restoring the number target to `0.0`.
- The exact source and target reports match the C++ probe after bind,
  mutation/public-update, and later state-machine advances.
- Existing deterministic formula fallback and formula target-to-source probes
  continue to pass.
