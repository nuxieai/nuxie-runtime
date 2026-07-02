# Data Binding Graph Formula Remaining Fallbacks Explicit Target-To-Source Runtime Contract

## Purpose

Extend non-number formula fallback explicit target-to-source behavior beyond
the first boolean slice.

For main-`ToSource | TwoWay` number targets fed by enum, color, string, or
trigger sources through `DataConverterFormula`, C++ applies the edited number
target through the formula converter's main-direction `convert` path. That
produces a number, which cannot be written back into the original source type,
so the source remains unchanged. The same explicit data-context advance then
reapplies the unchanged source through the formula fallback, restoring the
number target to `0.0`.

## In Scope

- `StateMachineInstance::advanceDataContext()` reached through the C++ probe
  and mirrored by Rust's public runtime seam.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceEnum.propertyValue`,
  `ViewModelInstanceColor.propertyValue`,
  `ViewModelInstanceString.propertyValue`, and
  `ViewModelInstanceTrigger.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterFormula` on a main-`ToSource | TwoWay` number data
  bind.
- Deterministic `FormulaTokenInput` fallback behavior for the admitted source
  kinds.
- Type-mismatch source preservation when main-direction formula conversion
  produces a number for an enum, color, string, or trigger source.
- Same-pass source-to-target reapplication during explicit data-context
  advancement.
- C++ probe matrix reporting each bind's source and target values after each
  explicit runtime action.

## Out Of Scope

- Boolean fallback explicit target-to-source behavior, covered separately by
  `data-binding-graph-formula-boolean-fallback-explicit-target-to-source-runtime-contract.md`.
- Main-`ToTarget | TwoWay` public-update behavior for these source kinds,
  covered separately by
  `data-binding-graph-formula-remaining-fallbacks-public-update-target-to-source-runtime-contract.md`.
- List and symbol-list-index formula fallback explicit target-to-source
  behavior.
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

- Mutated main-`ToSource | TwoWay` formula-bound number targets do not write
  their converted numbers into enum, color, string, or trigger sources.
- Explicit data-context advancement reapplies each unchanged source through
  the formula fallback, restoring the number target to `0.0`.
- The exact source and target reports match the C++ probe after initial
  data-context advancement, mutation/data-context advancement, and later
  state-machine advances.
- Existing boolean fallback explicit, public-update, deterministic formula
  fallback, and formula target-to-source probes continue to pass.
