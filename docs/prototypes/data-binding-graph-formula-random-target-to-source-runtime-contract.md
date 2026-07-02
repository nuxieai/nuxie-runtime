# Data Binding Graph Formula Random Target-To-Source Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to direct number
target-to-source scheduling.

This covers two direct C++ paths that use `DataConverterFormula` default
random-mode cache state: explicit `advancedDataContext()` after a target
mutation for a main-`ToSource | TwoWay` bind, and public
`updateDataBinds(true)` for a main-`ToTarget | TwoWay` bind after an initial
source-to-target evaluation.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- Default `DataConverterFormula.randomModeValue == 0`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Per-formula cached random values shared between source-to-target and
  target-to-source evaluation.
- Explicit target-to-source scheduling through `advance_data_context`.
- Public target-to-source scheduling through
  `update_data_binds_apply_target_to_source`.
- C++ probe coverage through number binding reports, including a direct
  observer bind for the explicit target-to-source case.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- `RandomMode::always`, `RandomMode::sourceChange`, random cache invalidation,
  random call-count parity, and formula `addDirt` random-cache behavior.
- Grouped formula, list formula, symbol-list-index, and non-number random
  formula scheduling. Target-dirty scheduling is covered separately by
  `data-binding-graph-formula-random-target-dirty-runtime-contract.md`.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- Explicit target-to-source conversion after a target mutation draws and caches
  the supplied formula random value.
- Later public update target-to-source conversion reuses the cached formula
  random value and reapplies source-to-target in the same update.
- Existing deterministic formula target-to-source and public update tests still
  pass.
