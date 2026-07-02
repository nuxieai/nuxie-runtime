# Data Binding Graph Formula Random SymbolListIndex Always Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to direct
`RandomMode::always` behavior for default-context symbol-list-index sources
feeding number targets.

This covers the C++ behavior where a `DataConverterFormula` random function
with `randomModeValue == 1` consumes a fresh random value on each formula
evaluation, after the symbol-list-index source has been cast to `float`.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from `DataBind.converterId`.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue == 1`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
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
- Grouped symbol-list-index `RandomMode::always` source-to-target scheduling
  is covered separately by
  `data-binding-graph-formula-random-symbol-list-index-group-non-default-runtime-contract.md`.
- Grouped target-to-source, grouped target-dirty, list, and non-symbol
  non-number random formula scheduling.
- Formula parent-source binding, source dependents, and add-dirt behavior.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- A default symbol-list-index source is converted to `f32` before random
  formula evaluation.
- The first source-to-target advance consumes the first supplied random value.
- A later source-to-target advance consumes the next supplied random value.
- Existing default-mode symbol-list-index random formula tests still pass.
