# Data Binding Graph Formula Random Group Target-Dirty Call Count Runtime Contract

## Purpose

Extend grouped formula random call-count coverage into main-`ToTarget |
TwoWay` target-dirty scheduling.

This pins how many values Rust pulls from the host-supplied formula random
stream while matching the existing C++ probe binding reports for grouped number
binds that preserve a manual target edit through explicit data-context
advancement.

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
- Manual bindable-number target mutation followed by explicit
  `advance_data_context`, preserving the edited target.
- Later normal state-machine advancement after that target-dirty pass.
- C++ probe comparisons for the observable binding values around those Rust
  call-count assertions.

## Out Of Scope

- Probe-visible C++ `RandomProvider::totalCalls()`. The current C++ probe links
  the non-`TESTING` runtime build, where that API is not available.
- A real Rust random generator, random seeding, platform RNG behavior, or
  parity with C++ `std::rand()`.
- Queue-content parity beyond values supplied by
  `set_data_bind_formula_random_values`.
- Direct call counts, which are covered by the direct call-count contracts.
- List-source, symbol-list-index, and non-number random formula call counts.
- Imported contexts, owned contexts, and secondary converter dependency
  invalidation.
- Full dirty-list scheduler parity.

## Completion Checks

- Setting a new host random stream resets the grouped Rust call count to zero.
- Default random mode consumes one value during the initial grouped
  source-to-target advance and reuses that cached value through target-dirty
  preservation and later normal advances.
- Always random mode consumes one value during the initial grouped
  source-to-target advance, consumes one more value on the first later normal
  reapply, and does not consume an additional value on the second later normal
  advance in this grouped fixture.
- Source-change random mode consumes one value during the initial grouped
  source-to-target advance, preserves the cache through target-dirty
  advancement because the source did not change, and reuses the cached value on
  later normal advances.
- The same fixtures continue to match C++ probe binding values.
