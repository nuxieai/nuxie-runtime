# Data Binding Graph Formula Symbol-List-Index Public-Update Target-To-Source Runtime Contract

## Purpose

Admit public-update reverse behavior for default-context symbol-list-index
sources flowing through `DataConverterFormula` into number targets.

For a main-`ToTarget | TwoWay` number target fed by a symbol-list-index source
through `DataConverterFormula`, C++ reverse-converts the edited number target
through formula `convert`, producing a number. That number is not written back
into the symbol-list-index source, so the source remains unchanged. The same
public update then reapplies the unchanged symbol-list-index source through
the formula converter, restoring the number target to the formula value.

## In Scope

- `StateMachineInstance::updateDataBinds(true)` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- A direct deterministic `DataConverterFormula` on a main-`ToTarget | TwoWay`
  number data bind.
- Deterministic formula output-queue tokens already modeled by the
  symbol-list-index formula contract: `FormulaTokenInput`,
  `FormulaTokenValue`, and `FormulaTokenOperation`.
- Type-mismatch source preservation when reverse formula conversion produces a
  number for the symbol-list-index source.
- Immediate source-to-target reapplication during public update.
- Exact C++ probe reporting for the bind's source and target values after each
  explicit runtime action.

## Out Of Scope

- Main-`ToSource | TwoWay` explicit data-context target-to-source behavior for
  symbol-list-index formula sources is covered separately by
  `data-binding-graph-formula-symbol-list-index-explicit-target-to-source-runtime-contract.md`.
- Boolean, enum, color, string, trigger, and list formula fallback
  public-update reverse behavior, covered separately by the fallback
  public-update contracts.
- Asset, artboard, and view-model pointer formula fallback reverse behavior.
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
  its reverse-converted number into the symbol-list-index source.
- The public update reapplies the unchanged symbol-list-index source through
  the formula converter, restoring the number target to the C++ formula value.
- The exact source and target reports match the C++ probe after bind,
  mutation/public-update, and later state-machine advances.
- Existing symbol-list-index formula, fallback public-update, and formula
  target-to-source probes continue to pass.
