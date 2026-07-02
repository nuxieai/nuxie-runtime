# Data Binding Graph Formula Random Group Always Target-Dirty Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to grouped
`RandomMode::always` main-`ToTarget | TwoWay` target-dirty behavior for
default-context number binds.

This covers the C++ behavior where the initial grouped source-to-target pass
consumes a fresh random value, a manual target edit is preserved through
explicit data-context advancement, and later normal state-machine advances
consume fresh random values when reapplying the unchanged source.

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
- Explicit data-context advancement preserving a manual target edit.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Grouped explicit target-to-source `RandomMode::always` scheduling is covered
  separately by
  `data-binding-graph-formula-random-group-always-target-to-source-runtime-contract.md`.
- Grouped public update target-to-source `RandomMode::always` scheduling is
  covered separately by
  `data-binding-graph-formula-random-group-always-public-update-target-to-source-runtime-contract.md`.
- Grouped `RandomMode::sourceChange` target-to-source/public-update/
  target-dirty scheduling.
- List formula, symbol-list-index, and non-number random formula scheduling.
- Stateful grouped converters mixed with random formulas.
- Random call-count parity outside the observed grouped bind.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- The initial source-to-target advance consumes the first supplied random
  value.
- Explicit data-context advancement preserves the manual target edit without
  consuming another random value.
- Later normal state-machine advances consume fresh supplied random values.
- Existing direct always, grouped default-mode, grouped always source-to-target,
  grouped always explicit target-to-source, and grouped always public-update
  random tests still pass.
