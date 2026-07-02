# Data Binding Graph Formula Random Group Non-Number Fallback Runtime Contract

## Purpose

Close the default-context source-to-target grouped random formula path for
represented non-number sources feeding number targets through
`DataConverterGroup<OperationValue, Formula(random)>`.

C++ admits boolean, enum, color, string, and trigger sources into this grouped
number-target path. The leading `OperationValue` converter falls back to a
numeric value, then the grouped `DataConverterFormula` evaluates
`FunctionType::random` using the state-machine formula random stream.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean`, `ViewModelInstanceEnum`,
  `ViewModelInstanceColor`, `ViewModelInstanceString`, and
  `ViewModelInstanceTrigger` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random` and
  constant bounds `2.0` and `6.0`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Source-to-target state-machine advancement after initial bind and, for
  `always` and `sourceChange`, after default-source mutation.
- Rust and C++ `RandomProvider::totalCalls` parity for this source-to-target
  grouped fallback path.
- Default-context trigger source mutations updating the active/default
  view-model trigger mirror when the trigger source is bound to a number
  target.

## Out Of Scope

- Direct non-number random fallback behavior, covered by
  `data-binding-graph-formula-random-non-number-fallback-runtime-contract.md`.
- Grouped target-to-source, public-update, and target-dirty behavior for these
  source kinds.
- Grouped list random formulas and generated list items.
- Other grouped converter shapes, including groups that do not start with
  `DataConverterOperationValue`.
- Imported and owned view-model contexts.
- Real Rust random generation or C++ random seeding beyond deterministic probe
  values.
- Secondary converter dependency invalidation and full dirty-list scheduler
  parity.

## Completion Checks

- Boolean, enum, color, string, and trigger default sources are admitted into
  grouped number-target data binds with `OperationValue -> Formula(random)`.
- Initial source-to-target advancement writes the C++ random formula result to
  the number target.
- Default mode consumes one random value and reuses it on later advancement.
- Always mode consumes one value initially and a fresh value after source
  mutation.
- Source-change mode consumes one value initially, clears the nested formula
  cache after source mutation, and consumes one fresh value on the next
  source-to-target pass.
- C++ `randomTotalCalls` and Rust
  `StateMachineInstance::data_bind_formula_random_call_count()` match after
  every report.
