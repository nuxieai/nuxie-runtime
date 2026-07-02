# Data Binding Graph Formula Symbol-List-Index Explicit Target-To-Source Runtime Contract

## Purpose

Admit explicit target-to-source behavior for default-context symbol-list-index
sources flowing through `DataConverterFormula` into number targets.

For a main-`ToSource | TwoWay` number target fed by a symbol-list-index source
through `DataConverterFormula`, C++ applies the edited number target through
the formula converter's main-direction `convert` path. That produces a number,
which is not written back into the symbol-list-index source, so the source
remains unchanged. The same explicit data-context advance then reapplies the
unchanged symbol-list-index source through the formula converter, restoring
the number target to the formula value.

## In Scope

- `StateMachineInstance::advanceDataContext()` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- A direct deterministic `DataConverterFormula` on a main-`ToSource | TwoWay`
  number data bind.
- Deterministic formula output-queue tokens already modeled by the
  symbol-list-index formula contract: `FormulaTokenInput`,
  `FormulaTokenValue`, and `FormulaTokenOperation`.
- Type-mismatch source preservation when main-direction formula conversion
  produces a number for the symbol-list-index source.
- Same-pass source-to-target reapplication during explicit data-context
  advancement.
- Exact C++ probe reporting for the bind's source and target values after each
  explicit runtime action.

## Out Of Scope

- Main-`ToTarget | TwoWay` public-update behavior for symbol-list-index formula
  sources, covered separately by
  `data-binding-graph-formula-symbol-list-index-public-update-target-to-source-runtime-contract.md`.
- Main-`ToTarget | TwoWay` target-dirty behavior for symbol-list-index formula
  sources, covered separately by
  `data-binding-graph-formula-symbol-list-index-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Boolean, enum, color, string, trigger, and list formula fallback explicit
  target-to-source behavior, covered separately by the fallback explicit
  contracts.
- Asset, artboard, and view-model pointer formula fallback reverse behavior.
- `FormulaTokenFunction` with random formula values is covered separately by
  `data-binding-graph-formula-random-symbol-list-index-target-to-source-runtime-contract.md`.
- Other formula functions.
- Formula converter groups.
- Formula parent-source binding, source dependents, and general add-dirt
  behavior beyond the same-pass reapplication listed above.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated main-`ToSource | TwoWay` formula-bound number target does not write
  its converted number into the symbol-list-index source.
- Explicit data-context advancement reapplies the unchanged symbol-list-index
  source through the formula converter, restoring the number target to the C++
  formula value.
- The exact source and target reports match the C++ probe after initial
  data-context advancement, mutation/data-context advancement, and later
  state-machine advances.
- Existing symbol-list-index formula, symbol-list-index public-update, fallback
  explicit, and formula target-to-source probes continue to pass.
