# Data Binding Graph Formula Random Always Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to direct
`RandomMode::always` source-to-target behavior for default-context number
binds.

This covers the C++ behavior where a `DataConverterFormula` random function
with `randomModeValue == 1` consumes a fresh random value on each formula
evaluation instead of reusing the default-mode cached value.

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
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- `RandomMode::sourceChange`, random cache invalidation, random call-count
  parity outside the observed direct bind, and formula `addDirt`
  random-cache behavior.
- Target-to-source, public update, target-dirty, grouped, list,
  symbol-list-index, and non-number `RandomMode::always` scheduling.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- The runtime graph accepts a direct formula random function when
  `randomModeValue == 1`.
- The first source-to-target advance consumes the first supplied random value.
- A later source-to-target advance consumes the next supplied random value
  instead of reusing the first one.
- Existing default-mode random formula tests still pass.
