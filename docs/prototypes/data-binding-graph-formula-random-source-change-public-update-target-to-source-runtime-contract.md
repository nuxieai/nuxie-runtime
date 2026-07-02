# Data Binding Graph Formula Random Source-Change Public Update Target-To-Source Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to direct
`RandomMode::sourceChange` public target-to-source scheduling for
default-context number binds.

This covers the C++ behavior where a `DataConverterFormula` random function
with `randomModeValue == 2` warms its cached random value during the initial
source-to-target pass, reuses that value for the public target-to-source
source write, treats the changed source as a source change, clears that
formula cache, and consumes a fresh value for same-update source-to-target
reapplication.

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
- Public target-to-source scheduling through
  `update_data_binds_apply_target_to_source`.
- Same-update source-to-target reapplication after the public target-to-source
  source write.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Explicit target-to-source scheduling is covered separately by
  `data-binding-graph-formula-random-source-change-target-to-source-runtime-contract.md`.
- Target-dirty, grouped, list, symbol-list-index, and non-number
  `RandomMode::sourceChange` scheduling.
- Converter dependency invalidation for secondary source paths, including
  `DataConverterOperationViewModel` dependencies.
- Random call-count parity outside the observed direct bind.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- The initial source-to-target advance consumes the first supplied random
  value.
- Public target-to-source conversion after a target mutation reuses the warmed
  source-change random value for the source write.
- A changed source clears the source-change formula random cache.
- Same-update source-to-target reapplication consumes the next supplied random
  value instead of reusing the source-write value.
- Later source-to-target advances reuse that second value until another source
  change.
- Existing default-mode, always-mode, and explicit source-change
  target-to-source random tests still pass.
