# Data Binding Graph Formula Random Group Source-Change Target-Dirty Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to grouped
`RandomMode::sourceChange` main-`ToTarget | TwoWay` target-dirty behavior for
default-context number binds.

This covers the C++ behavior where the initial grouped source-to-target pass
consumes a cached-mode random value, a manual target edit is preserved through
explicit data-context advancement, and later normal state-machine advances
reapply the unchanged source through the same cached random value because the
source did not change.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue == 2`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Explicit data-context advancement preserving a manual target edit.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Grouped explicit target-to-source `RandomMode::sourceChange` scheduling is
  covered separately by
  `data-binding-graph-formula-random-group-source-change-target-to-source-runtime-contract.md`.
- Grouped public update target-to-source `RandomMode::sourceChange` scheduling
  is covered separately by
  `data-binding-graph-formula-random-group-source-change-public-update-target-to-source-runtime-contract.md`.
- List formula, symbol-list-index, and non-number random formula scheduling.
- Stateful grouped converters mixed with random formulas.
- Secondary converter dependency invalidation beyond the observed grouped
  source.
- Random call-count parity outside the observed grouped bind.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- The initial source-to-target advance consumes the first supplied random
  value.
- Explicit data-context advancement preserves the manual target edit without
  treating the target edit as a source change.
- Later normal state-machine advances reuse the cached supplied random value.
- Existing grouped default-mode, grouped always, grouped source-change
  source-to-target, grouped source-change explicit target-to-source, and
  grouped source-change public-update tests still pass.
