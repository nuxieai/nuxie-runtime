# Data Binding Graph Formula Random Group Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to the first grouped
converter path: a default-context number source feeding a number target through
`DataConverterGroup<OperationValue, Formula(random)>`.

This proves that runtime graph conversion passes the same per-state-machine
formula random stream through nested group converter state, and that the
grouped formula caches the default-mode random value like the direct formula
path.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- Default `DataConverterFormula.randomModeValue == 0`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- `RandomMode::always`, `RandomMode::sourceChange`, random cache invalidation,
  random call-count parity, and formula `addDirt` random-cache behavior.
- Grouped public update target-to-source scheduling is covered separately by
  `data-binding-graph-formula-random-group-public-update-target-to-source-runtime-contract.md`;
  grouped target-dirty scheduling is covered separately by
  `data-binding-graph-formula-random-group-target-dirty-runtime-contract.md`;
  grouped explicit target-to-source scheduling is covered separately by
  `data-binding-graph-formula-random-group-target-to-source-runtime-contract.md`.
- List formula, symbol-list-index, and non-number random formula scheduling.
- Stateful grouped converters mixed with random formulas.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- A grouped formula random bind draws from the same host-supplied random stream
  used by direct formula random binds.
- The grouped formula caches that default-mode random value across subsequent
  source-to-target state-machine advancement.
- C++ probe number reports match Rust after both the initial zero-time advance
  and a later positive-time advance.
- Existing direct formula random tests still pass.
