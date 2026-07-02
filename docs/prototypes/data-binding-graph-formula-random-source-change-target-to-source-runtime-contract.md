# Data Binding Graph Formula Random Source-Change Target-To-Source Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to direct
`RandomMode::sourceChange` explicit target-to-source scheduling for
default-context number binds.

This covers the C++ behavior where a `DataConverterFormula` random function
with `randomModeValue == 2` consumes a cached-mode random value for an
explicit target-to-source source write, treats the changed source as a source
change, clears that formula cache, and consumes a fresh value for same-pass
source-to-target reapplication.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from `DataBind.converterId`.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue == 2`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Explicit target-to-source scheduling through `advance_data_context`.
- Source-to-target reapplication after the target-to-source source write.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Direct public update target-to-source `RandomMode::sourceChange` scheduling
  is covered separately by
  `data-binding-graph-formula-random-source-change-public-update-target-to-source-runtime-contract.md`.
- Target-dirty, grouped, list, symbol-list-index, and non-number
  `RandomMode::sourceChange` scheduling.
- Converter dependency invalidation for secondary source paths, including
  `DataConverterOperationViewModel` dependencies.
- Direct explicit target-to-source call counts for random modes `0`, `1`, and
  `2` are covered separately by
  `data-binding-graph-formula-random-target-to-source-call-count-runtime-contract.md`.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- Explicit target-to-source conversion after a target mutation consumes the
  first supplied random value for the source write.
- A changed source clears the source-change formula random cache.
- Same-pass source-to-target reapplication consumes the next supplied random
  value instead of reusing the target-to-source write value.
- Later normal advances in this direct fixture preserve the same-pass
  reapplication result without consuming more supplied random values unless
  another source change schedules a formula evaluation.
- Existing default-mode, always-mode, and source-change source-mutation random
  tests still pass.
