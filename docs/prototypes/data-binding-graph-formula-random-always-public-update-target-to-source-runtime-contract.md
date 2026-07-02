# Data Binding Graph Formula Random Always Public Update Target-To-Source Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to direct
`RandomMode::always` public target-to-source scheduling for default-context
number binds.

This covers the C++ behavior where a `DataConverterFormula` random function
with `randomModeValue == 1` consumes fresh random values across the initial
source-to-target pass, public `updateDataBinds(true)` target-to-source source
write, same-update source-to-target reapplication, and later state-machine
advancement.

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
- Public target-to-source scheduling through
  `update_data_binds_apply_target_to_source`.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Explicit target-to-source scheduling is covered separately by
  `data-binding-graph-formula-random-always-target-to-source-runtime-contract.md`.
- Target-dirty, grouped, list, symbol-list-index, and non-number
  `RandomMode::always` target-to-source scheduling.
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
- Public target-to-source conversion after a target mutation consumes the next
  supplied random value for the source write.
- Same-update source-to-target reapplication consumes another supplied random
  value instead of reusing the source-write value.
- Later source-to-target advances keep consuming fresh supplied random values.
- Existing default-mode and explicit target-to-source always-mode random tests
  still pass.
