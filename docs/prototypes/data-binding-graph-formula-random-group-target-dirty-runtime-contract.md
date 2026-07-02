# Data Binding Graph Formula Random Group Target-Dirty Runtime Contract

## Purpose

Extend the host-supplied graph formula random group slice to
main-`ToTarget | TwoWay` target-dirty behavior for default-context number
binds.

This covers the C++ behavior where a manual edit to a grouped random
formula-bound `BindablePropertyNumber.propertyValue` target is preserved
through explicit data-context advancement, then the next normal state-machine
advance reapplies the unchanged source through the same cached grouped formula
random value.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- A main-`ToTarget | TwoWay` data bind.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- Default `DataConverterFormula.randomModeValue == 0`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Per-formula cached random values reused after a manual target edit.
- C++ probe coverage through number binding reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- `RandomMode::always`, `RandomMode::sourceChange`, random cache invalidation,
  random call-count parity, and formula `addDirt` random-cache behavior.
- Grouped explicit target-to-source scheduling.
- List formula, symbol-list-index, and non-number random formula scheduling.
- Stateful grouped converters mixed with random formulas.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- Initial source-to-target advancement draws and caches the supplied grouped
  formula random value.
- Explicit data-context advancement preserves a manual target edit for a
  main-`ToTarget | TwoWay` grouped random formula bind.
- The next normal state-machine advance reapplies the unchanged source through
  the cached grouped formula random value, matching C++.
- Existing direct and grouped formula random public-update tests still pass.
