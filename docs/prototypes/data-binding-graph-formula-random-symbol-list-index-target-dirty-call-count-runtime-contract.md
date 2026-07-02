# Data Binding Graph Formula Random SymbolListIndex Target-Dirty Call Count Runtime Contract

## Purpose

Extend direct symbol-list-index random formula call-count coverage into
main-`ToTarget | TwoWay` target-dirty scheduling.

This pins how many values Rust pulls from the host-supplied formula random
stream while matching the existing C++ probe binding reports for direct
symbol-list-index binds that preserve a manual target edit through explicit
data-context advancement.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- Direct graph-owned `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Initial source-to-target state-machine advancement.
- Manual bindable-number target mutation followed by explicit
  `advance_data_context`, preserving the edited target.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Later normal state-machine advancement after that target-dirty pass.
- C++ probe comparisons for the observable binding values around those call
  count assertions.

## Out Of Scope

- Probe-visible C++ `RandomProvider::totalCalls()`. The current C++ probe links
  the non-`TESTING` runtime build, where that API is not available.
- A real Rust random generator, random seeding, platform RNG behavior, or
  parity with C++ `std::rand()`.
- Direct symbol-list-index source-to-target call counts, which are covered by
  `data-binding-graph-formula-random-symbol-list-index-call-count-runtime-contract.md`.
- Direct explicit target-to-source symbol-list-index call counts, which are
  covered by
  `data-binding-graph-formula-random-symbol-list-index-target-to-source-call-count-runtime-contract.md`.
- Direct public-update symbol-list-index call counts, which are covered by
  `data-binding-graph-formula-random-symbol-list-index-public-update-call-count-runtime-contract.md`.
- Grouped symbol-list-index call counts.
- Number-source call counts, which are covered by the direct number call-count
  contracts.
- List-source, remaining non-number, imported-context, and owned-context random
  formula call counts.
- Secondary converter dependency invalidation and full dirty-list scheduler
  parity.

## Completion Checks

- Setting a new host random stream resets the Rust call count to zero.
- Default random mode consumes one value during the initial source-to-target
  advance and reuses that cached value through target-dirty preservation and
  later normal advances.
- Always random mode consumes one value during the initial source-to-target
  advance, consumes one more value on the first later normal reapply, and does
  not consume an additional value on the second later normal advance in this
  direct fixture.
- Source-change random mode consumes one value during the initial
  source-to-target advance, preserves the cache through target-dirty
  advancement because the symbol-list-index source did not change, and reuses
  the cached value on later normal advances.
- The same fixtures continue to match C++ probe binding values.
