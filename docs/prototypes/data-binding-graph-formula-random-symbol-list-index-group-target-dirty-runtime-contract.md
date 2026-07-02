# Data Binding Graph Formula Random Symbol-List-Index Group Target-Dirty Runtime Contract

## Purpose

Pin target-dirty scheduling for grouped symbol-list-index random formula binds.

This completes the grouped symbol-list-index random scheduling family by
proving that a main-`ToTarget | TwoWay` number target can be manually edited,
preserved through explicit `advance_data_context`, and later restored from the
unchanged symbol-list-index source through the grouped formula path with C++
parity.

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
- Preservation of the dirty target value during explicit data-context
  advancement.
- Later normal source-to-target reapplication of the unchanged symbol-list-index
  source through the grouped random formula.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.

## Out Of Scope

- Grouped symbol-list-index source-to-target behavior, covered by
  `data-binding-graph-formula-random-symbol-list-index-group-runtime-contract.md`
  and
  `data-binding-graph-formula-random-symbol-list-index-group-non-default-runtime-contract.md`.
- Grouped symbol-list-index explicit target-to-source scheduling, covered by
  `data-binding-graph-formula-random-symbol-list-index-group-target-to-source-runtime-contract.md`.
- Grouped symbol-list-index public-update target-to-source scheduling, covered
  by
  `data-binding-graph-formula-random-symbol-list-index-group-public-update-target-to-source-runtime-contract.md`.
- Direct symbol-list-index random formula behavior, covered by the direct
  symbol-list-index random contracts.
- List formula random scheduling and non-number fallback random scheduling.
- Stateful grouped converters mixed with symbol-list-index random formulas.
- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Grouped symbol-list-index target-dirty call counts are covered by
  `data-binding-graph-formula-random-symbol-list-index-group-target-dirty-call-count-runtime-contract.md`.
- Formula `addDirt` random-cache behavior and secondary dependency
  invalidation.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- Explicit `advance_data_context` preserves a manually edited number target for
  grouped symbol-list-index random formula binds with random modes `0`, `1`,
  and `2`.
- Later normal state-machine advancement reapplies the unchanged
  symbol-list-index source through the grouped formula path.
- Default and source-change modes reuse the cached random value when the source
  does not mutate.
- Always mode consumes fresh supplied random values on later normal advances.
- C++ probe number reports match Rust for the initial source-to-target report,
  the target-dirty report, and later normal state-machine advances.
