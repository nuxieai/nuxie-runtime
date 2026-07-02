# Data Binding Graph Formula Random Symbol-List-Index Group Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to the first grouped
symbol-list-index source path: a default-context symbol-list-index source
feeding a number target through
`DataConverterGroup<OperationValue, Formula(random)>`.

This covers the C++ behavior where the graph admits the symbol-list-index
source for a grouped number-producing converter, converts it through the first
group child, and then evaluates the nested random formula with the same
per-state-machine random stream and default-mode cache behavior as direct
formula binds.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- Default `DataConverterFormula.randomModeValue == 0`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Source admission for grouped number-producing converters whose first child
  can accept a symbol-list-index source.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- Direct symbol-list-index random formula behavior, covered by
  `data-binding-graph-formula-random-symbol-list-index-runtime-contract.md`,
  `data-binding-graph-formula-random-symbol-list-index-always-runtime-contract.md`,
  `data-binding-graph-formula-random-symbol-list-index-source-change-runtime-contract.md`,
  `data-binding-graph-formula-random-symbol-list-index-target-to-source-runtime-contract.md`,
  and
  `data-binding-graph-formula-random-symbol-list-index-target-dirty-runtime-contract.md`.
- Grouped symbol-list-index `RandomMode::always` and
  `RandomMode::sourceChange` source-to-target scheduling is covered by
  `data-binding-graph-formula-random-symbol-list-index-group-non-default-runtime-contract.md`.
- Grouped symbol-list-index explicit target-to-source scheduling is covered by
  `data-binding-graph-formula-random-symbol-list-index-group-target-to-source-runtime-contract.md`.
- Grouped symbol-list-index public-update target-to-source scheduling is
  covered by
  `data-binding-graph-formula-random-symbol-list-index-group-public-update-target-to-source-runtime-contract.md`.
- Grouped symbol-list-index target-dirty scheduling.
- List formula random scheduling and non-number fallback random scheduling.
- Stateful grouped converters mixed with symbol-list-index random formulas.
- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Random call-count parity outside the observed grouped bind actions.
- Formula `addDirt` random-cache behavior and secondary dependency
  invalidation.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- A grouped symbol-list-index random bind is admitted into the runtime data
  bind graph instead of being dropped as an unsupported number source.
- The grouped formula draws from the same host-supplied random stream used by
  direct formula random binds.
- The grouped formula caches that default-mode random value across subsequent
  source-to-target state-machine advancement.
- C++ probe number reports match Rust after both the initial zero-time advance
  and a later positive-time advance.
