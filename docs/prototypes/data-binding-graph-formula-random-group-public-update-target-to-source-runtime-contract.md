# Data Binding Graph Formula Random Group Public Update Target-To-Source Runtime Contract

## Purpose

Extend the host-supplied graph formula random group slice to public
`updateDataBinds(true)` behavior for a main-`ToTarget | TwoWay` number bind.

This covers the C++ behavior where public update applies a grouped random
formula target mutation through reverse group order, then reapplies
source-to-target in the same update using the cached default-mode random
formula value.

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
- Public `StateMachineInstance::update_data_binds_apply_target_to_source`.
- C++ probe coverage through number binding reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- `RandomMode::always`, `RandomMode::sourceChange`, random cache invalidation,
  random call-count parity, and formula `addDirt` random-cache behavior.
- Grouped explicit target-to-source and target-dirty scheduling.
- List formula, symbol-list-index, and non-number random formula scheduling.
- Stateful grouped converters mixed with random formulas.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- Initial source-to-target advancement draws and caches the supplied grouped
  formula random value.
- Public update after a target mutation runs grouped target-to-source behavior
  with the cached random value and reports the same number binding state as
  C++.
- The following normal state-machine advances continue to match C++.
- Existing direct and grouped formula random source-to-target tests still pass.
