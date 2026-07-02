# Data Binding Graph Formula Random Remaining Fallbacks Target-To-Source Runtime Contract

## Purpose

Extend random-function formula fallback target-to-source behavior beyond the
first boolean slice.

For enum, color, string, and trigger sources flowing through
`DataConverterFormula` into number targets, C++ keeps the source unchanged when
target-to-source conversion produces a number, then reapplies the unchanged
source through the formula fallback. This remains true when the formula output
queue contains `FunctionType::random`: the final target value returns to `0.0`
for `randomModeValue` values `0`, `1`, and `2`.

## In Scope

- `StateMachineInstance::advanceDataContext()` for main-`ToSource | TwoWay`
  number binds.
- `StateMachineInstance::updateDataBinds(true)` for main-`ToTarget | TwoWay`
  number binds.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceEnum.propertyValue`,
  `ViewModelInstanceColor.propertyValue`,
  `ViewModelInstanceString.propertyValue`, and
  `ViewModelInstanceTrigger.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- A direct `DataConverterFormula` on the number data bind.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Type-mismatch source preservation when formula conversion produces a number
  for enum, color, string, and trigger sources.
- Same-pass source-to-target reapplication for explicit data-context
  advancement and public update.
- C++ probe matrix reporting each bind's source and target values after each
  explicit runtime action.

## Out Of Scope

- Boolean random fallback target-to-source behavior, covered separately by
  `data-binding-graph-formula-random-boolean-fallback-target-to-source-runtime-contract.md`.
- List random formula target-to-source behavior is covered separately by
  `data-binding-graph-formula-random-list-fallback-target-to-source-runtime-contract.md`.
- Symbol-list-index random formula target-to-source behavior.
- Number-source random formula evaluation and scheduling, covered by the
  direct and grouped random contracts.
- Asset, artboard, and view-model pointer formula fallback reverse behavior.
- Formula converter groups for non-number fallback sources.
- Formula parent-source binding, source dependents, and general add-dirt
  behavior beyond the same-pass reapplication listed above.
- Full C++ dirty-list scheduling for neighboring ordinary `ToTarget` binds.
- A real Rust random generator, C++ random seeding/queueing, or random
  call-count parity.
- Imported and owned view-model contexts.
- Relative-path, parent-path, nested-path, listener-owned data binding, nested
  artboards, and render/layout behavior.

## Completion Checks

- Mutated explicit main-`ToSource | TwoWay` random-formula-bound number targets
  do not write their converted numbers into enum, color, string, or trigger
  sources.
- Mutated public main-`ToTarget | TwoWay` random-formula-bound number targets
  do not write their reverse-converted numbers into enum, color, string, or
  trigger sources.
- Explicit data-context advancement and public update both reapply each
  unchanged source through the formula fallback, restoring the number target
  to `0.0`.
- Random modes `0`, `1`, and `2` all preserve the same observable fallback
  behavior, even when Rust is supplied non-zero random values.
- Existing random boolean fallback target-to-source, deterministic fallback,
  and formula target-to-source probes continue to pass.
