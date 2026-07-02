# Data Binding Graph Formula Boolean Fallback Explicit Target-To-Source Runtime Contract

## Purpose

Admit the first non-number formula fallback explicit target-to-source path.

For a main-`ToSource | TwoWay` number target fed by a boolean source through
`DataConverterFormula`, C++ applies the edited number target through the
formula converter's main-direction `convert` path. That produces a number,
which cannot be written back into the boolean source, so the source remains
unchanged. The same explicit data-context advance then reapplies the unchanged
boolean source through the formula fallback, restoring the number target to
`0.0`.

## In Scope

- `StateMachineInstance::advanceDataContext()` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterFormula` on a main-`ToSource | TwoWay` number data
  bind.
- Deterministic `FormulaTokenInput` fallback behavior for the boolean source.
- Type-mismatch source preservation when main-direction formula conversion
  produces a number for the boolean source.
- Same-pass source-to-target reapplication during explicit data-context
  advancement.
- Exact C++ probe reporting for the bind's source and target values after each
  explicit runtime action.

## Out Of Scope

- Main-`ToTarget | TwoWay` public-update behavior for boolean formula fallback
  sources, covered separately by
  `data-binding-graph-formula-boolean-fallback-public-update-target-to-source-runtime-contract.md`.
- Enum, color, string, trigger, list, and symbol-list-index formula fallback
  explicit target-to-source behavior.
- Asset, artboard, and view-model pointer formula fallback reverse behavior.
- `FormulaTokenFunction`, random formula values, and `randomModeValue`.
- Formula converter groups.
- Formula parent-source binding, source dependents, and general add-dirt
  behavior beyond the same-pass reapplication listed above.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated main-`ToSource | TwoWay` formula-bound number target does not write
  its converted number into the boolean source.
- Explicit data-context advancement reapplies the unchanged boolean source
  through the formula fallback, restoring the number target to `0.0`.
- The exact source and target reports match the C++ probe after initial
  data-context advancement, mutation/data-context advancement, and later
  state-machine advances.
- Existing deterministic formula target-to-source, public-update, and fallback
  probes continue to pass.
