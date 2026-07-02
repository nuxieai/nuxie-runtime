# Data Binding Graph Formula Random List Fallback Target-To-Source Runtime Contract

## Purpose

Admit random-function formula fallback target-to-source behavior for
default-context list sources flowing into number targets.

For list sources flowing through `DataConverterFormula` into number targets,
C++ keeps the imported list source unchanged when target-to-source conversion
produces a number, then reapplies the unchanged list source through the
formula fallback. This remains true when the formula output queue contains
`FunctionType::random`: the final target value returns to `0.0` for
`randomModeValue` values `0`, `1`, and `2`.

## In Scope

- `StateMachineInstance::advanceDataContext()` for main-`ToSource | TwoWay`
  number binds.
- `StateMachineInstance::updateDataBinds(true)` for main-`ToTarget | TwoWay`
  number binds.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceList` sources with imported `ViewModelInstanceListItem`
  children feeding `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterFormula` on the number data bind.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Type-mismatch source preservation when formula conversion produces a number
  for the list source.
- Same-pass source-to-target reapplication for explicit data-context
  advancement and public update.
- Exact C++ probe reporting for the bind's source and target values after each
  explicit runtime action.

## Out Of Scope

- Boolean, enum, color, string, and trigger random fallback target-to-source
  behavior, covered separately by the non-list random fallback contracts.
- Symbol-list-index random formula target-to-source behavior.
- Source-to-target `BindablePropertyList` list-target behavior is covered by
  `data-binding-graph-formula-random-list-fallback-bindable-list-target-runtime-contract.md`.
- Explicit target-to-source `BindablePropertyList` list-target behavior is
  covered by
  `data-binding-graph-formula-random-list-fallback-bindable-list-explicit-target-to-source-runtime-contract.md`.
- Public-update `BindablePropertyList` list-target target-to-source behavior
  is covered by
  `data-binding-graph-formula-random-list-fallback-bindable-list-public-update-target-to-source-runtime-contract.md`.
- Main-`ToTarget | TwoWay` target-dirty behavior for random-function formula
  list sources is covered by
  `data-binding-graph-formula-random-list-fallback-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Generated list item creation, artboard component-list instancing, map-rule
  selection, layout, and virtualization.
- Number-source random formula evaluation and scheduling, covered by the
  direct and grouped random contracts.
- Asset, artboard, and view-model pointer formula fallback reverse behavior.
- Formula converter groups for list fallback sources.
- Formula parent-source binding, source dependents, and general add-dirt
  behavior beyond the same-pass reapplication listed above.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- A real Rust random generator, C++ random seeding/queueing, or random
  call-count parity.
- Imported and owned view-model contexts beyond the imported list items that
  make up the default list source.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated explicit main-`ToSource | TwoWay` random-formula-bound number
  target does not write its converted number into the list source.
- A mutated public main-`ToTarget | TwoWay` random-formula-bound number target
  does not write its reverse-converted number into the list source.
- Explicit data-context advancement and public update both reapply the
  unchanged list source through the formula fallback, restoring the number
  target to `0.0`.
- Random modes `0`, `1`, and `2` all preserve the same observable fallback
  behavior, even when Rust is supplied non-zero random values.
- Existing list random fallback, list fallback target-to-source, and random
  non-number fallback probes continue to pass.
