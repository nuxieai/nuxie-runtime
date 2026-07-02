# Data Binding Graph Formula Random Symbol-List-Index Group Target-Dirty Call Count Runtime Contract

## Purpose

Extend grouped symbol-list-index random call-count coverage into
main-`ToTarget | TwoWay` target-dirty scheduling.

This pins how many values Rust pulls from the host-supplied formula random
stream while matching the existing C++ probe binding reports for grouped
symbol-list-index binds that preserve a manual target edit through explicit
data-context advancement.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Main-`ToTarget | TwoWay` data-bind flags with explicit
  `StateMachineInstance::advance_data_context` after a manual target edit.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Initial source-to-target state-machine advancement.
- Manual bindable-number target mutation followed by explicit
  `advance_data_context`, preserving the edited target.
- Later normal state-machine advancement after that target-dirty pass.
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
- Grouped symbol-list-index source-to-target call counts are covered by
  `data-binding-graph-formula-random-symbol-list-index-group-call-count-runtime-contract.md`.
- Grouped explicit target-to-source symbol-list-index call counts are covered
  by
  `data-binding-graph-formula-random-symbol-list-index-group-target-to-source-call-count-runtime-contract.md`.
- Grouped public-update symbol-list-index call counts are covered by
  `data-binding-graph-formula-random-symbol-list-index-group-public-update-call-count-runtime-contract.md`.
- Direct symbol-list-index call counts, which are covered by the direct
  symbol-list-index call-count contracts.
- Number-source grouped call counts, which are covered by the grouped number
  call-count contracts.
- List-source and remaining non-number random formula call counts.
- Imported contexts, owned contexts, and secondary converter dependency
  invalidation.
- Full dirty-list scheduler parity.

## Completion Checks

- Setting a new host random stream resets the grouped symbol-list-index Rust
  call count to zero.
- Default random mode consumes one value during the initial grouped
  source-to-target advance and reuses that cached value through target-dirty
  preservation and later normal advances.
- Always random mode consumes one value during the initial grouped
  source-to-target advance, consumes one more value on the first later normal
  reapply, and does not consume an additional value on the second later normal
  advance in this grouped fixture.
- Source-change random mode consumes one value during the initial grouped
  source-to-target advance, preserves the cache through target-dirty
  advancement because the symbol-list-index source did not change, and reuses
  the cached value on later normal advances.
- The C++ probe's `randomTotalCalls` reports match the expected C++ source
  semantics and Rust's host-stream call count for every grouped target-dirty
  report.
- The same grouped symbol-list-index fixtures continue to match C++ probe
  binding values.
