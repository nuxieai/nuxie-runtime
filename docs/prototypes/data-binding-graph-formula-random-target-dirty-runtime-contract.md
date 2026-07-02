# Data Binding Graph Formula Random Target-Dirty Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to main-`ToTarget | TwoWay`
target-dirty behavior for direct number binds.

This covers the C++ behavior where a manual edit to a formula-bound
`BindablePropertyNumber.propertyValue` target is preserved through explicit
data-context advancement, then the next normal state-machine advance reapplies
the unchanged source through the same cached random formula value.

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
- Per-formula cached random values reused after a manual target edit.
- C++ probe coverage through number binding reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Direct target-dirty `RandomMode::always` scheduling is covered separately by
  `data-binding-graph-formula-random-always-target-dirty-runtime-contract.md`.
- `RandomMode::sourceChange`, random cache invalidation, random call-count
  parity, and formula `addDirt` random-cache behavior.
- Grouped source-to-target scheduling is covered separately by
  `data-binding-graph-formula-random-group-runtime-contract.md`; grouped
  public update target-to-source scheduling is covered separately by
  `data-binding-graph-formula-random-group-public-update-target-to-source-runtime-contract.md`;
  grouped target-dirty scheduling is covered separately by
  `data-binding-graph-formula-random-group-target-dirty-runtime-contract.md`.
  Symbol-list-index target-dirty scheduling is covered separately by
  `data-binding-graph-formula-random-symbol-list-index-target-dirty-runtime-contract.md`.
  List formula and non-number random formula scheduling remain out of scope.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- The initial source-to-target pass draws and caches the supplied formula
  random value.
- Explicit data-context advancement preserves a manual target edit for a
  main-`ToTarget | TwoWay` random formula bind.
- The next normal state-machine advance reapplies the unchanged source through
  the cached formula random value, matching C++.
- Existing deterministic formula target-dirty tests still pass.
