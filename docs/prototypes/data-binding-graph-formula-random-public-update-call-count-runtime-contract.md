# Data Binding Graph Formula Random Public Update Call Count Runtime Contract

## Purpose

Extend formula random call-count coverage into direct public
`update_data_binds_apply_target_to_source` scheduling.

This pins how many values Rust pulls from the host-supplied formula random
stream while matching the existing C++ probe binding reports for
main-`ToTarget | TwoWay` number binds.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- Direct graph-owned `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Initial source-to-target state-machine advancement.
- Public target-to-source scheduling through
  `update_data_binds_apply_target_to_source` after a bindable number target
  mutation.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Same-update source-to-target reapplication after public target-to-source
  source writes.
- Later normal state-machine advancement after that public update, proving no
  additional random values are pulled without another formula evaluation.
- C++ probe comparisons for the observable binding values around those call
  count assertions.

## Out Of Scope

- Probe-visible C++ `RandomProvider::totalCalls()`. The current C++ probe links
  the non-`TESTING` runtime build, where that API is not available.
- A real Rust random generator, random seeding, platform RNG behavior, or
  parity with C++ `std::rand()`.
- Explicit `advance_data_context` target-to-source call counts, which are
  covered by
  `data-binding-graph-formula-random-target-to-source-call-count-runtime-contract.md`.
- Target-dirty call counts are covered by
  `data-binding-graph-formula-random-target-dirty-call-count-runtime-contract.md`.
- Grouped public-update target-to-source call counts are covered separately by
  `data-binding-graph-formula-random-group-public-update-call-count-runtime-contract.md`.
- List sources, symbol-list-index sources, non-number fallback sources,
  imported contexts, owned contexts, secondary converter dependency
  invalidation, and full dirty-list scheduler parity.

## Completion Checks

- Setting a new host random stream resets the Rust call count to zero.
- Default random mode consumes one value during the initial source-to-target
  advance and reuses that cached value through public target-to-source,
  same-update reapplication, and later normal advances.
- Always random mode consumes one value during the initial source-to-target
  advance, consumes two more values during the public update, and does not
  consume additional values on later normal advances in this direct fixture.
- Source-change random mode consumes one value during the initial
  source-to-target advance, reuses that warmed value for the public
  target-to-source source write, consumes one refreshed value for same-update
  source-to-target reapplication, and does not consume additional values on
  later normal advances in this direct fixture.
- The same fixtures continue to match C++ probe binding values.
