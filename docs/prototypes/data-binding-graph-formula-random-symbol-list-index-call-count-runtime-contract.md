# Data Binding Graph Formula Random SymbolListIndex Call Count Runtime Contract

## Purpose

Extend formula random call-count coverage from direct number sources to direct
symbol-list-index sources feeding number targets.

This pins Rust's host-supplied random-stream accounting to the same
source-to-target actions already compared against the C++ probe for
`DataValueSymbolListIndex` random formula conversion.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct graph-owned `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Source-to-target state-machine advancement.
- Source-change cache clearing through
  `set_default_view_model_symbol_list_index_source_for_data_bind`.
- `StateMachineInstance::data_bind_formula_random_call_count()` as the Rust
  observable for pulls from the host-supplied random stream.
- C++ probe comparisons for the observable binding values around those call
  count assertions.

## Out Of Scope

- Probe-visible C++ `RandomProvider::totalCalls()`. The current C++ probe links
  the non-`TESTING` runtime build, where that API is not available.
- A real Rust random generator, random seeding, platform RNG behavior, or
  parity with C++ `std::rand()`.
- Queue-content parity beyond values supplied by
  `set_data_bind_formula_random_values`.
- Direct explicit target-to-source symbol-list-index call counts are covered by
  `data-binding-graph-formula-random-symbol-list-index-target-to-source-call-count-runtime-contract.md`.
- Direct public-update symbol-list-index call counts are covered by
  `data-binding-graph-formula-random-symbol-list-index-public-update-call-count-runtime-contract.md`.
- Direct target-dirty symbol-list-index call counts.
- Grouped symbol-list-index call counts.
- Number-source call counts, which are covered by the direct number call-count
  contracts.
- List-source, remaining non-number, imported-context, and owned-context random
  formula call counts.
- Secondary converter dependency invalidation and full dirty-list scheduler
  parity.

## Completion Checks

- Setting a new host random stream resets the Rust call count to zero.
- Default random mode consumes one random value across repeated source-to-target
  advances while the formula cache remains valid.
- Always random mode consumes a fresh value when a source mutation schedules a
  second formula evaluation, then does not pull again on the later no-op
  advance in this fixture.
- Source-change random mode consumes once initially, consumes again after the
  symbol-list-index source changes, and then reuses the refreshed cache.
- The same fixtures continue to match C++ probe binding values.
