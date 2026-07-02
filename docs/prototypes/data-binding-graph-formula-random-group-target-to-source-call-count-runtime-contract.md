# Data Binding Graph Formula Random Group Target-To-Source Call Count Runtime Contract

## Purpose

Extend grouped formula random call-count coverage from source-to-target
advancement into explicit target-to-source scheduling.

This pins how many values Rust pulls from the host-supplied formula random
stream while matching the existing C++ probe binding reports for
main-`ToSource | TwoWay` number binds that use
`DataConverterGroup<OperationValue, Formula(random)>`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- A main-`ToSource | TwoWay` data bind plus one same-path direct observer bind.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Explicit target-to-source scheduling through `advance_data_context` after a
  bindable number target mutation.
- Same-pass source-to-target reapplication after explicit target-to-source
  source writes.
- Later normal state-machine advancement after that explicit target-to-source
  pass, proving no additional values are pulled without another grouped
  formula evaluation in these fixtures.
- C++ probe comparisons for the observable binding values around those Rust
  call-count assertions.
- C++ probe `--runtime-random-reset`, repeated `--runtime-random-value`, and
  per-action `runtimeStateMachineAdvances[].randomTotalCalls` comparisons.

## Out Of Scope

- Upstream C++ `TESTING` builds or direct use of
  `RandomProvider::totalCalls()` outside the probe-owned counted provider
  shim.
- A real Rust random generator, random seeding, platform RNG behavior, or
  parity with C++ `std::rand()`.
- Queue-content parity beyond values supplied by
  `set_data_bind_formula_random_values`.
- Grouped public-update target-to-source call counts are covered separately by
  `data-binding-graph-formula-random-group-public-update-call-count-runtime-contract.md`.
- Grouped target-dirty call counts are covered separately by
  `data-binding-graph-formula-random-group-target-dirty-call-count-runtime-contract.md`.
- Direct call counts, which are covered by the direct call-count contracts.
- List-source, symbol-list-index, and non-number random formula call counts.
- Imported contexts, owned contexts, and secondary converter dependency
  invalidation.
- Full dirty-list scheduler parity.

## Completion Checks

- Setting a new host random stream resets the grouped Rust call count to zero.
- Default random mode consumes one value during explicit grouped
  target-to-source and reuses that cached value during same-pass
  reapplication and later advances.
- Always random mode consumes two values during explicit grouped
  target-to-source scheduling and does not consume additional values on later
  normal advances in this grouped fixture.
- Source-change random mode consumes one value for the grouped
  target-to-source source write, clears the cache for the changed source,
  consumes a second value for same-pass source-to-target reapplication, and
  does not consume additional values on later normal advances in this grouped
  fixture.
- The C++ probe's `randomTotalCalls` reports match the expected C++ grouped
  target-to-source semantics and Rust's host-stream call count for every
  grouped explicit target-to-source report.
- The same fixtures continue to match C++ probe binding values.
