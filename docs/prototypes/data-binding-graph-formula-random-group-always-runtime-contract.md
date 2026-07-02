# Data Binding Graph Formula Random Group Always Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to grouped
`RandomMode::always` source-to-target behavior for default-context number
binds.

This covers the C++ behavior where a `DataConverterFormula` random function
inside `DataConverterGroup<OperationValue, Formula(random)>` has
`randomModeValue == 1` and consumes a fresh random value on each formula
evaluation instead of reusing the grouped formula state's cached value.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue == 1`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Source mutation through
  `set_default_view_model_number_source_for_data_bind`.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Grouped source-to-target `RandomMode::sourceChange` scheduling is covered
  separately by
  `data-binding-graph-formula-random-group-source-change-runtime-contract.md`.
- Grouped source-to-target host random call-count parity for the observed bind
  is covered separately by
  `data-binding-graph-formula-random-group-call-count-runtime-contract.md`;
  broader call-count parity and formula `addDirt` random-cache behavior remain
  out of scope here.
- Grouped explicit target-to-source `RandomMode::always` scheduling is covered
  separately by
  `data-binding-graph-formula-random-group-always-target-to-source-runtime-contract.md`.
- Grouped public update and target-dirty scheduling for non-default random
  modes.
- List formula, symbol-list-index, and non-number random formula scheduling.
- Stateful grouped converters mixed with random formulas.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- A grouped formula random bind accepts `randomModeValue == 1`.
- The first source-to-target advance consumes the first supplied random value.
- A source mutation schedules a later source-to-target formula evaluation that
  consumes the next supplied random value instead of reusing the first one.
- A later advance without another source mutation does not reschedule the
  grouped formula.
- Existing default-mode grouped random formula tests still pass.
