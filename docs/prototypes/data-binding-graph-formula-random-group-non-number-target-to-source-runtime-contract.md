# Data Binding Graph Formula Random Group Non-Number Target-To-Source Runtime Contract

## Purpose

Close explicit `advanceDataContext()` target-to-source parity for represented
non-number sources feeding number targets through
`DataConverterGroup<OperationValue, Formula(random)>`.

C++ preserves the boolean, enum, color, string, or trigger source when a
mutated number target is applied back through this grouped converter. The
explicit pass still schedules the same immediate source-to-target reapply, and
the grouped formula random path reports the same deterministic random call
count as C++.

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
- Main `ToSource | TwoWay` flags applied through explicit
  `advanceDataContext()` after a number target mutation.
- Rust and C++ `RandomProvider::totalCalls` parity after the explicit pass and
  two later state-machine advances.

## Out Of Scope

- Source-to-target scheduling, covered by
  `data-binding-graph-formula-random-group-non-number-fallback-runtime-contract.md`.
- Public `updateDataBinds(true)` target-to-source behavior.
- Main `ToTarget | TwoWay` target-dirty behavior.
- Grouped list random formulas and generated list items.
- Other grouped converter shapes.
- Imported and owned view-model contexts.
- Real Rust random generation or C++ random seeding beyond deterministic probe
  values.
- Secondary converter dependency invalidation and full dirty-list scheduler
  parity.

## Completion Checks

- Number target mutations apply through explicit `advanceDataContext()` for all
  five represented non-number source kinds.
- The non-number source value is preserved when grouped number conversion
  produces a number.
- The immediate source-to-target reapply matches C++ number target reports.
- Default, always, and source-change modes each consume one visible random
  value for this explicit non-number target-to-source schedule and reuse it on
  later advances.
- C++ `randomTotalCalls` and Rust
  `StateMachineInstance::data_bind_formula_random_call_count()` match after
  every report.
