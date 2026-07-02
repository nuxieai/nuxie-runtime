# Data Binding Graph Formula Random Boolean Fallback Target-To-Source Runtime Contract

## Purpose

Admit random-function formula fallback target-to-source behavior for the first
non-number source type.

For boolean sources flowing through `DataConverterFormula` into number targets,
C++ keeps the source unchanged when target-to-source conversion produces a
number, then reapplies the unchanged boolean source through the formula
fallback. This remains true when the formula output queue contains
`FunctionType::random`: the final target value returns to `0.0` for
`randomModeValue` values `0`, `1`, and `2`.

## In Scope

- `StateMachineInstance::advanceDataContext()` for main-`ToSource | TwoWay`
  number binds.
- `StateMachineInstance::updateDataBinds(true)` for main-`ToTarget | TwoWay`
  number binds.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterFormula` on the number data bind.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Type-mismatch source preservation when formula conversion produces a number
  for the boolean source.
- Same-pass source-to-target reapplication for explicit data-context
  advancement and public update.
- Exact C++ probe reporting for the bind's source and target values after each
  explicit runtime action.

## Out Of Scope

- Enum, color, string, and trigger random fallback target-to-source behavior is
  covered separately by
  `data-binding-graph-formula-random-remaining-fallbacks-target-to-source-runtime-contract.md`.
- List random formula target-to-source behavior is covered separately by
  `data-binding-graph-formula-random-list-fallback-target-to-source-runtime-contract.md`.
- Symbol-list-index random formula target-to-source behavior is covered
  separately by
  `data-binding-graph-formula-random-symbol-list-index-target-to-source-runtime-contract.md`.
- Number-source random formula evaluation and scheduling, covered by the
  direct and grouped random contracts.
- Boolean random fallback target-dirty behavior is covered by
  `data-binding-graph-formula-random-boolean-fallback-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Enum, color, string, and trigger random fallback target-dirty behavior is
  covered by
  `data-binding-graph-formula-random-remaining-fallbacks-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Asset, artboard, and view-model pointer formula fallback reverse behavior.
- Formula converter groups for non-number fallback sources.
- Formula parent-source binding, source dependents, and general add-dirt
  behavior beyond the same-pass reapplication listed above.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- A real Rust random generator or C++ random seeding/queueing.
- Boolean target-to-source random call counts are covered by
  `data-binding-graph-formula-random-boolean-fallback-target-to-source-call-count-runtime-contract.md`.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- A mutated explicit main-`ToSource | TwoWay` random-formula-bound number
  target does not write its converted number into the boolean source.
- A mutated public main-`ToTarget | TwoWay` random-formula-bound number target
  does not write its reverse-converted number into the boolean source.
- Explicit data-context advancement and public update both reapply the
  unchanged boolean source through the formula fallback, restoring the number
  target to `0.0`.
- Random modes `0`, `1`, and `2` all preserve the same observable fallback
  behavior, even when Rust is supplied non-zero random values.
- Existing random formula, deterministic fallback, and target-to-source probes
  continue to pass.
