# Data Binding Graph Formula Random Group Non-Number Public Update Runtime Contract

## Purpose

Close public `updateDataBinds(true)` target-to-source parity for represented
non-number sources feeding number targets through
`DataConverterGroup<OperationValue, Formula(random)>`.

This is the public-update sibling of the explicit
`advanceDataContext()` target-to-source slice. C++ first warms the grouped
source-to-target formula random path, then applies a mutated number target back
to the preserved non-number source during `updateDataBinds(true)`, and finally
reapplies the source-to-target binding in the same update.

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
- Main `ToTarget | TwoWay` flags, initial source-to-target advancement, target
  mutation, and public `updateDataBinds(true)` application.
- Rust and C++ `RandomProvider::totalCalls` parity after the initial advance,
  public update, and two later state-machine advances.

## Out Of Scope

- Initial source-to-target scheduling, covered by
  `data-binding-graph-formula-random-group-non-number-fallback-runtime-contract.md`.
- Explicit `advanceDataContext()` target-to-source behavior, covered by
  `data-binding-graph-formula-random-group-non-number-target-to-source-runtime-contract.md`.
- Main `ToTarget | TwoWay` target-dirty behavior after explicit data-context
  advancement.
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
- Public target-to-source update preserves the non-number source when grouped
  number conversion produces a number.
- Same-update source-to-target reapply matches C++ number target reports.
- Default and source-change modes keep the initial random call count at
  `[1, 1, 1, 1]`.
- Always mode consumes two additional public-update values for
  `[1, 3, 3, 3]`.
- C++ `randomTotalCalls` and Rust
  `StateMachineInstance::data_bind_formula_random_call_count()` match after
  every report.
