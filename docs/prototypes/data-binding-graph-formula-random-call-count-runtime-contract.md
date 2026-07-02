# Data Binding Graph Formula Random Call Count Runtime Contract

## Purpose

Pin Rust's graph formula random-source accounting to the C++ call sites that
pull random values during `DataConverterFormula` runtime evaluation.

C++ counts calls at `RandomProvider::generateRandomFloat()`. The C++ probe now
exposes that as per-action `runtimeStateMachineAdvances[].randomTotalCalls`
when tests opt into its counted deterministic provider. In the Rust port, the
equivalent observable for the host-supplied test stream is
`StateMachineInstance::data_bind_formula_random_call_count()`, which counts
every pull from `StateMachineInstance::set_data_bind_formula_random_values`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- Direct graph-owned `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Source-to-target state-machine advancement.
- Source-change cache clearing through
  `set_default_view_model_number_source_for_data_bind`.
- C++ probe `--runtime-random-reset` and repeated `--runtime-random-value`
  inputs for deterministic call-count comparison.
- List-source formula fallback cases proving non-number, non-symbol-list-index
  inputs do not evaluate random tokens.
- C++ probe comparisons for the observable binding values around those call
  count assertions, including `randomTotalCalls` for the default-context
  number source-to-target fixtures.

## Out Of Scope

- Upstream C++ `TESTING` builds or direct use of
  `RandomProvider::totalCalls()` outside the probe-owned counted provider
  shim.
- A real Rust random generator, random seeding, platform RNG behavior, or
  parity with C++ `std::rand()`.
- Queue-content parity beyond values supplied by
  `set_data_bind_formula_random_values`.
- Direct explicit target-to-source call counts are covered separately by
  `data-binding-graph-formula-random-target-to-source-call-count-runtime-contract.md`.
- Direct public update target-to-source call counts are covered separately by
  `data-binding-graph-formula-random-public-update-call-count-runtime-contract.md`.
- Direct target-dirty call counts are covered separately by
  `data-binding-graph-formula-random-target-dirty-call-count-runtime-contract.md`.
- Grouped source-to-target call counts are covered separately by
  `data-binding-graph-formula-random-group-call-count-runtime-contract.md`.
- Grouped target-to-source, public-update, and target-dirty call counts,
  imported contexts, owned contexts, and secondary converter dependency
  invalidation. Those behaviors are covered or tracked by narrower scheduling
  contracts.
- Full dirty-list scheduler parity.

## Completion Checks

- Setting a new host random stream resets the Rust call count to zero.
- Default random mode consumes one random value across repeated source-to-target
  advances while the formula cache remains valid.
- Always random mode consumes one random value per source-to-target formula
  evaluation.
- Source-change random mode consumes once initially, consumes again after the
  source changes, and then reuses the refreshed cache.
- The C++ probe's `randomTotalCalls` reports match the expected C++ source
  semantics and Rust's host-stream call count for the default-context number
  source-to-target fixtures.
- List-source formula fallback with random tokens consumes zero random values
  because C++ returns the numeric fallback before evaluating formula tokens.
- The same fixtures continue to match C++ probe binding values.
