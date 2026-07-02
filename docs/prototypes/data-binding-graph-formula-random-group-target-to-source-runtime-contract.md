# Data Binding Graph Formula Random Group Target-To-Source Runtime Contract

## Purpose

Extend the host-supplied graph formula random group slice to explicit
main-`ToSource | TwoWay` target-to-source behavior for default-context number
binds.

This covers the C++ behavior where a manual edit to a grouped random
formula-bound `BindablePropertyNumber.propertyValue` target is applied through
forward group order during explicit data-context advancement. A neighboring
direct number bind observes the grouped write on subsequent state-machine
advancement.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- A main-`ToSource | TwoWay` data bind plus one same-path direct observer bind.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- Default `DataConverterFormula.randomModeValue == 0`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- C++ probe coverage through number binding reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- `RandomMode::always`, `RandomMode::sourceChange`, random cache invalidation,
  random call-count parity, and formula `addDirt` random-cache behavior.
- List formula, symbol-list-index, and non-number random formula scheduling.
- Stateful grouped converters mixed with random formulas.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- Explicit data-context advancement applies the grouped random formula
  target-to-source write with the supplied random value.
- The grouped formula caches that default-mode random value for subsequent
  source-to-target advancement.
- A neighboring direct observer bind reports the grouped source write after
  normal state-machine advancement, matching C++.
- Existing direct and grouped formula random public-update and target-dirty
  tests still pass.
