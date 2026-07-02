# Data Binding Graph Formula Random Source-Change Target-Dirty Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to direct
`RandomMode::sourceChange` main-`ToTarget | TwoWay` target-dirty behavior for
default-context number binds.

This covers the C++ behavior where the initial source-to-target pass consumes
a cached-mode random value, a manual target edit is preserved through explicit
data-context advancement, and later normal state-machine advances reapply the
unchanged source through the same cached random value because the source did
not change.

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
- Explicit data-context advancement preserving a manual target edit.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Explicit target-to-source scheduling is covered separately by
  `data-binding-graph-formula-random-source-change-target-to-source-runtime-contract.md`.
- Public update target-to-source scheduling is covered separately by
  `data-binding-graph-formula-random-source-change-public-update-target-to-source-runtime-contract.md`.
- Symbol-list-index `RandomMode::sourceChange` target-dirty scheduling is
  covered separately by
  `data-binding-graph-formula-random-symbol-list-index-target-dirty-runtime-contract.md`.
- Grouped, list, and non-number `RandomMode::sourceChange` target-dirty
  scheduling.
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
- Explicit data-context advancement preserves the manual target edit without
  treating the target edit as a source change.
- Later normal state-machine advances reuse the cached supplied random value.
- Existing default-mode, always-mode, source-change source-mutation,
  source-change target-to-source, and source-change public-update tests still
  pass.
