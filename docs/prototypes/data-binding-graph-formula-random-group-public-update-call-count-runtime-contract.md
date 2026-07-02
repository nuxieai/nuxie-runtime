# Data Binding Graph Formula Random Group Public Update Call Count Runtime Contract

## Purpose

Extend grouped formula random call-count coverage into public
`update_data_binds_apply_target_to_source` scheduling.

This pins how many values Rust pulls from the host-supplied formula random
stream while matching the existing C++ probe binding reports for
main-`ToTarget | TwoWay` number binds that use
`DataConverterGroup<OperationValue, Formula(random)>`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- A main-`ToTarget | TwoWay` data bind.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Initial source-to-target state-machine advancement.
- Public target-to-source scheduling through
  `update_data_binds_apply_target_to_source` after a bindable number target
  mutation.
- Same-update source-to-target reapplication after public target-to-source
  source writes.
- Later normal state-machine advancement after that public update, proving no
  additional values are pulled without another grouped formula evaluation in
  these fixtures.
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
- Grouped target-dirty call counts are covered separately by
  `data-binding-graph-formula-random-group-target-dirty-call-count-runtime-contract.md`.
- Direct call counts, which are covered by the direct call-count contracts.
- List-source, symbol-list-index, and non-number random formula call counts.
- Imported contexts, owned contexts, and secondary converter dependency
  invalidation.
- Full dirty-list scheduler parity.

## Completion Checks

- Setting a new host random stream resets the grouped Rust call count to zero.
- Default random mode consumes one value during the initial grouped
  source-to-target advance and reuses that cached value through public
  target-to-source, same-update reapplication, and later normal advances.
- Always random mode consumes one value during the initial grouped
  source-to-target advance, consumes two more values during the public update,
  and does not consume additional values on later normal advances in this
  grouped fixture.
- Source-change random mode consumes one value during the initial grouped
  source-to-target advance, reuses that warmed value for the public
  target-to-source source write, consumes one refreshed value for same-update
  source-to-target reapplication, and does not consume additional values on
  later normal advances in this grouped fixture.
- The C++ probe's `randomTotalCalls` reports match the expected C++ grouped
  public-update semantics and Rust's host-stream call count for every grouped
  public-update report.
- The same fixtures continue to match C++ probe binding values.
