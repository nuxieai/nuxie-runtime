# Data Binding Graph Formula Random Always Target-Dirty Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to direct
`RandomMode::always` main-`ToTarget | TwoWay` target-dirty behavior for
default-context number binds.

This covers the C++ behavior where the initial source-to-target pass consumes
a random value, a manual target edit is preserved through explicit data-context
advancement, and later normal state-machine advances consume fresh random
values when reapplying the unchanged source.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from `DataBind.converterId`.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue == 1`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Explicit data-context advancement preserving a manual target edit.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Explicit and public target-to-source scheduling are covered separately by
  `data-binding-graph-formula-random-always-target-to-source-runtime-contract.md`
  and
  `data-binding-graph-formula-random-always-public-update-target-to-source-runtime-contract.md`.
- Symbol-list-index `RandomMode::always` target-dirty scheduling is covered
  separately by
  `data-binding-graph-formula-random-symbol-list-index-target-dirty-runtime-contract.md`.
- Grouped, list, and non-number `RandomMode::always` target-dirty scheduling.
- `RandomMode::sourceChange`, random cache invalidation, random call-count
  parity outside the observed direct bind, and formula `addDirt`
  random-cache behavior.
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
- Existing default-mode and target-to-source always-mode random tests still
  pass.
