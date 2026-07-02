# Data Binding Graph Formula Random Group Non-Number Target-Dirty Runtime Contract

## Purpose

Close main `ToTarget | TwoWay` target-dirty parity for represented non-number
sources feeding number targets through
`DataConverterGroup<OperationValue, Formula(random)>`.

This is the target-dirty sibling of the grouped non-number source-to-target,
explicit target-to-source, and public-update slices. C++ first warms the grouped
source-to-target formula random path, preserves a manually edited number target
during explicit `advanceDataContext()`, and then reapplies the source-to-target
binding on later normal state-machine advancement when the graph marks it dirty.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean`, `ViewModelInstanceEnum`,
  `ViewModelInstanceColor`, `ViewModelInstanceString`, and
  `ViewModelInstanceTrigger` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- `DataConverterGroup` with exactly two children:
  `DataConverterOperationValue` followed by `DataConverterFormula`.
- `FormulaTokenFunction` with `functionType == FunctionType::random` and
  constant bounds `2.0` and `6.0`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Main `ToTarget | TwoWay` flags, initial source-to-target advancement, manual
  number target mutation, explicit data-context advancement, and later normal
  state-machine advancement.
- Rust and C++ `RandomProvider::totalCalls` parity after the initial advance,
  target-dirty data-context advance, and two later state-machine advances.

## Out Of Scope

- Initial source-to-target scheduling, covered by
  `data-binding-graph-formula-random-group-non-number-fallback-runtime-contract.md`.
- Explicit main-`ToSource | TwoWay` target-to-source behavior, covered by
  `data-binding-graph-formula-random-group-non-number-target-to-source-runtime-contract.md`.
- Public `updateDataBinds(true)` target-to-source behavior, covered by
  `data-binding-graph-formula-random-group-non-number-public-update-runtime-contract.md`.
- Grouped list random formulas and generated list items.
- Other grouped converter shapes.
- Imported and owned view-model contexts.
- Real Rust random generation or C++ random seeding beyond deterministic probe
  values.
- Secondary converter dependency invalidation and full dirty-list scheduler
  parity.

## Completion Checks

- Initial source-to-target advancement warms the grouped formula random cache
  for all five represented non-number source kinds.
- A manual number target mutation is preserved during explicit
  `advanceDataContext()`.
- Later normal state-machine advancement reapplies the grouped source-to-target
  value reported by C++.
- Default mode keeps the initial random call count at `[1, 1, 1, 1]`.
- Always mode consumes one later value for `[1, 1, 2, 2]`.
- Source-change mode keeps `[1, 1, 1, 1]` for boolean, enum, color, and string
  sources.
- Source-change mode uses `[1, 1, 2, 2]` for trigger sources because the bound
  trigger reset is a C++-visible source change for the nested grouped formula
  random cache.
- C++ `randomTotalCalls` and Rust
  `StateMachineInstance::data_bind_formula_random_call_count()` match after
  every report.
