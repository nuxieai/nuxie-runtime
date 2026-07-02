# Data Binding Graph Formula Random Group Call Count Runtime Contract

## Purpose

Pin Rust's host-supplied graph formula random-source accounting for grouped
source-to-target number binds to the same observable C++ binding behavior used
by the grouped random scheduling probes.

This covers `DataConverterGroup<OperationValue, Formula(random)>` where the
nested `DataConverterFormula` consumes values from
`StateMachineInstance::set_data_bind_formula_random_values`, and where
`StateMachineInstance::data_bind_formula_random_call_count()` records the Rust
host-stream pulls needed to match C++ probe number reports.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Source-to-target state-machine advancement.
- Source-change cache clearing through
  `set_default_view_model_number_source_for_data_bind`.
- C++ probe comparisons for the observable binding values around those Rust
  call-count assertions.

## Out Of Scope

- Probe-visible C++ `RandomProvider::totalCalls()`. The current C++ probe links
  the non-`TESTING` runtime build, where that API is not available.
- A real Rust random generator, random seeding, platform RNG behavior, or
  parity with C++ `std::rand()`.
- Queue-content parity beyond values supplied by
  `set_data_bind_formula_random_values`.
- Grouped explicit target-to-source call counts are covered separately by
  `data-binding-graph-formula-random-group-target-to-source-call-count-runtime-contract.md`.
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
- Default random mode consumes one random value across repeated grouped
  source-to-target advances while the nested formula cache remains valid.
- Always random mode consumes one random value per grouped source-to-target
  formula evaluation; the second pull is covered by a source mutation that
  schedules a later evaluation.
- Source-change random mode consumes once initially, consumes again after the
  source changes, and then reuses the refreshed grouped formula cache.
- The same grouped fixtures continue to match C++ probe binding values.
