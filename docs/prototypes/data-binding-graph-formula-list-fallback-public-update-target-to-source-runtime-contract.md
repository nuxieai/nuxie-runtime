# Data Binding Graph Formula List Fallback Public-Update Target-To-Source Runtime Contract

## Purpose

Admit public-update reverse behavior for default-context list sources flowing
through `DataConverterFormula` into number targets.

For a main-`ToTarget | TwoWay` number target fed by a list source through
`DataConverterFormula`, C++ reverse-converts the edited number target through
formula `convert`, producing a number. That number cannot be written back into
the list source, so the imported list item count remains unchanged. The same
public update then reapplies the unchanged list source through the formula
fallback, restoring the number target to `0.0`.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceList` sources with imported `ViewModelInstanceListItem`
  children feeding `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterFormula` on a main-`ToTarget | TwoWay` number data
  bind.
- Deterministic `FormulaTokenInput` fallback behavior for the list source.
- Type-mismatch source preservation when reverse formula conversion produces a
  number for the list source.
- Immediate source-to-target reapplication during public update.
- Exact C++ probe reporting for the bind's source and target values after each
  explicit runtime action.

## Out Of Scope

- Boolean, enum, color, string, and trigger formula fallback public-update
  reverse behavior, covered separately by the non-list fallback public-update
  contracts.
- Main-`ToSource | TwoWay` explicit data-context target-to-source behavior for
  formula list sources is covered separately by
  `data-binding-graph-formula-list-fallback-explicit-target-to-source-runtime-contract.md`.
- List-target `BindablePropertyList` behavior, generated list item creation,
  artboard component-list instancing, map-rule selection, layout, and
  virtualization.
- Symbol-list-index formula public-update reverse behavior is covered
  separately by
  `data-binding-graph-formula-symbol-list-index-public-update-target-to-source-runtime-contract.md`.
- Asset, artboard, and view-model pointer formula fallback reverse behavior.
- `FormulaTokenFunction` with random formula values is covered separately by
  `data-binding-graph-formula-random-list-fallback-target-to-source-runtime-contract.md`.
- Other formula functions.
- Formula converter groups.
- Formula parent-source binding, source dependents, and general add-dirt
  behavior.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated main-`ToTarget | TwoWay` formula-bound number target does not write
  its reverse-converted number into the list source.
- The public update reapplies the unchanged list source through the formula
  fallback, restoring the number target to `0.0`.
- The exact source and target reports match the C++ probe after bind,
  mutation/public-update, and later state-machine advances.
- Existing list formula fallback, non-list fallback public-update, and formula
  target-to-source probes continue to pass.
