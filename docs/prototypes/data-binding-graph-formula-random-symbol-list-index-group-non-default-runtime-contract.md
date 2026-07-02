# Data Binding Graph Formula Random Symbol-List-Index Group Non-Default Runtime Contract

## Purpose

Extend grouped symbol-list-index random formula source-to-target scheduling to
`RandomMode::always` and `RandomMode::sourceChange`.

This builds on the default grouped symbol-list-index contract by proving that
`DataConverterGroup<OperationValue, Formula(random)>` uses the nested formula
state for fresh always-mode draws and source-change cache invalidation after a
symbol-list-index source mutation.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `1` and `2`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Source-to-target state-machine advancement and C++ probe number reports.
- Source-change invalidation when
  `set_default_view_model_symbol_list_index_source_for_data_bind` mutates the
  bound default symbol-list-index source.

## Out Of Scope

- Grouped symbol-list-index default-mode source-to-target behavior, covered by
  `data-binding-graph-formula-random-symbol-list-index-group-runtime-contract.md`.
- Grouped symbol-list-index explicit target-to-source scheduling is covered by
  `data-binding-graph-formula-random-symbol-list-index-group-target-to-source-runtime-contract.md`.
- Grouped symbol-list-index public-update target-to-source scheduling is
  covered by
  `data-binding-graph-formula-random-symbol-list-index-group-public-update-target-to-source-runtime-contract.md`.
- Grouped symbol-list-index target-dirty scheduling is covered by
  `data-binding-graph-formula-random-symbol-list-index-group-target-dirty-runtime-contract.md`.
- Direct symbol-list-index random formula behavior, covered by the direct
  symbol-list-index random contracts.
- List formula random scheduling and non-number fallback random scheduling.
- Stateful grouped converters mixed with symbol-list-index random formulas.
- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Grouped symbol-list-index source-to-target call counts are covered by
  `data-binding-graph-formula-random-symbol-list-index-group-call-count-runtime-contract.md`.
- Formula `addDirt` random-cache behavior and secondary dependency
  invalidation.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- Grouped `RandomMode::always` consumes fresh supplied random values on each
  source-to-target state-machine advancement.
- Grouped `RandomMode::sourceChange` caches the first supplied random value,
  clears the nested formula cache when the bound symbol-list-index source
  mutates, and consumes the next supplied random value on the next advance.
- C++ probe number reports match Rust for both non-default random modes.
- Existing grouped symbol-list-index default-mode random probes continue to
  pass.
