# Data Binding Graph Formula Random Boolean Fallback Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin main-`ToTarget | TwoWay` target-dirty behavior for random-function
`DataConverterFormula` boolean fallback sources feeding number targets.

C++ keeps the boolean source unchanged when a random-formula-bound number
target is manually edited. The manual target edit survives explicit
`advanceDataContext()`, and later normal state-machine advancement reapplies
source-to-target conversion, writing the boolean formula fallback scalar to
the target. For this boolean fallback shape, supplied random values are not
pulled from Rust's host random stream.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceBoolean.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct graph-owned `DataConverterFormula` converters with
  `FormulaTokenFunction` output tokens.
- `FunctionType::random` and `DataConverterFormula.randomModeValue` values
  `0`, `1`, and `2`.
- Main-`ToTarget | TwoWay` data-bind flags, without the `ToSource` direction
  flag.
- Initial source-to-target flushing through a normal state-machine advance.
- Mutating the bindable number target by data-bind index.
- Explicit `advanceDataContext()` preserving the manual target edit before the
  next normal state-machine advance overwrites it.
- `StateMachineInstance::data_bind_formula_random_call_count()` as the Rust
  observable for pulls from the host-supplied formula random stream.
- C++ probe comparisons for the number target after each runtime action.
- Rust assertions that the boolean source remains at the fixture's initial
  `true` value through the target-dirty sequence.

## Out Of Scope

- Deterministic boolean formula target-dirty behavior.
- Enum, color, string, and trigger random fallback target-dirty behavior.
- List random fallback target-dirty behavior, covered by
  `data-binding-graph-formula-random-list-fallback-main-to-target-two-way-target-dirty-runtime-contract.md`.
- Symbol-list-index random formula target-dirty behavior, covered by
  `data-binding-graph-formula-random-symbol-list-index-target-dirty-runtime-contract.md`.
- Number-source random formula evaluation and scheduling, covered by the
  direct and grouped random contracts.
- Boolean target-to-source behavior, covered by
  `data-binding-graph-formula-random-boolean-fallback-target-to-source-runtime-contract.md`.
- A real Rust random generator, C++ random seeding/queueing, or
  probe-visible C++ `RandomProvider::totalCalls()`.
- Queue-content parity beyond values supplied by
  `set_data_bind_formula_random_values`.
- Imported, owned, relative, parent, and nested view-model contexts.
- Formula converter groups for non-number fallback sources.
- Secondary converter dependency invalidation and full dirty-list scheduler
  parity.

## Completion Checks

- The initial normal state-machine advance applies the random-function formula
  boolean fallback scalar to `BindablePropertyNumber.propertyValue`.
- A manual edit to the number target is preserved through explicit
  data-context advancement.
- Later normal state-machine advancement reapplies the fallback scalar to the
  number target.
- Random modes `0`, `1`, and `2` match C++ even when Rust is supplied non-zero
  random values.
- Rust's host-supplied formula random call count remains zero through the full
  target-dirty sequence.
- Existing random formula boolean source-to-target and target-to-source probes
  continue to pass.
