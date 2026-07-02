# Data Binding Graph Formula Random SymbolListIndex Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to the first non-number
formula input path: default-context symbol-list-index sources feeding number
targets through a direct `DataConverterFormula` random function.

This covers the C++ behavior where `DataConverterFormula::convert` casts
`DataValueSymbolListIndex` to `float` before formula evaluation, while the
random function still draws and caches from the formula random state like the
direct number-source default-mode path.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from `DataBind.converterId`.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- Default `DataConverterFormula.randomModeValue == 0`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Direct source-to-target `RandomMode::always` scheduling is covered
  separately by
  `data-binding-graph-formula-random-symbol-list-index-always-runtime-contract.md`.
- Direct source-to-target `RandomMode::sourceChange` scheduling is covered
  separately by
  `data-binding-graph-formula-random-symbol-list-index-source-change-runtime-contract.md`.
- Broader random cache invalidation, random call-count parity, and formula
  `addDirt` random-cache behavior for symbol-list-index sources.
- Target-to-source symbol-list-index random formula scheduling is covered
  separately by
  `data-binding-graph-formula-random-symbol-list-index-target-to-source-runtime-contract.md`.
- Target-dirty symbol-list-index random formula scheduling is covered
  separately by
  `data-binding-graph-formula-random-symbol-list-index-target-dirty-runtime-contract.md`.
- Grouped symbol-list-index default-mode source-to-target scheduling is
  covered separately by
  `data-binding-graph-formula-random-symbol-list-index-group-runtime-contract.md`.
- Grouped symbol-list-index non-default source-to-target scheduling is covered
  separately by
  `data-binding-graph-formula-random-symbol-list-index-group-non-default-runtime-contract.md`.
- Grouped symbol-list-index explicit target-to-source scheduling is covered
  separately by
  `data-binding-graph-formula-random-symbol-list-index-group-target-to-source-runtime-contract.md`.
- Grouped symbol-list-index public-update target-to-source scheduling is
  covered separately by
  `data-binding-graph-formula-random-symbol-list-index-group-public-update-target-to-source-runtime-contract.md`.
- Grouped symbol-list-index target-dirty scheduling is covered separately by
  `data-binding-graph-formula-random-symbol-list-index-group-target-dirty-runtime-contract.md`.
- List and non-symbol non-number random formula scheduling.
- Formula parent-source binding, source dependents, and add-dirt behavior.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- A default symbol-list-index source is converted to `f32` before random
  formula evaluation.
- The first source-to-target advance consumes the supplied random value.
- A later source-to-target advance reuses the cached default-mode random value.
- Existing deterministic symbol-list-index formula tests still pass.
