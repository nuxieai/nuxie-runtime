# Data Binding Graph Formula Random Always Target-To-Source Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to direct
`RandomMode::always` explicit target-to-source scheduling for default-context
number binds.

This covers the C++ behavior where a `DataConverterFormula` random function
with `randomModeValue == 1` consumes a fresh random value for the explicit
target-to-source source write, then consumes fresh values for the same-bind
source-to-target reapplication. Later normal advances in this direct fixture do
not pull additional random values unless another formula evaluation is
scheduled.

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
- Explicit target-to-source scheduling through `advance_data_context`.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Direct public update target-to-source `RandomMode::always` scheduling is
  covered separately by
  `data-binding-graph-formula-random-always-public-update-target-to-source-runtime-contract.md`.
- Target-dirty, grouped, list, symbol-list-index, and non-number
  `RandomMode::always` target-to-source scheduling.
- Direct explicit target-to-source call counts for random modes `0`, `1`, and
  `2` are covered separately by
  `data-binding-graph-formula-random-target-to-source-call-count-runtime-contract.md`.
- `RandomMode::sourceChange`, random cache invalidation, and formula `addDirt`
  random-cache behavior.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- Explicit target-to-source conversion after a target mutation consumes the
  first supplied random value for the source write.
- Same-bind source-to-target reapplication consumes the next supplied random
  value instead of reusing the first one.
- Later normal advances in this direct fixture preserve the already-applied
  value without consuming more supplied random values.
- Existing default-mode and source-to-target always-mode random tests still
  pass.
