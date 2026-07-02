# Data Binding Graph Formula Random Source Change Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to the first direct
`RandomMode::sourceChange` source-to-target behavior for default-context
number binds.

This covers the C++ behavior where a `DataConverterFormula` random function
with `randomModeValue == 2` keeps using its cached random value until the
bound source changes, then clears that formula cache and consumes a fresh
random value on the next formula evaluation.

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
- Source mutation through
  `set_default_view_model_number_source_for_data_bind`.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Random call-count parity outside the observed direct bind.
- `RandomMode::always` scheduling beyond the already documented direct
  source-to-target slice.
- Direct explicit target-to-source `RandomMode::sourceChange` scheduling is
  covered separately by
  `data-binding-graph-formula-random-source-change-target-to-source-runtime-contract.md`.
- Public update, target-dirty, grouped, list, symbol-list-index, and non-number
  `RandomMode::sourceChange` scheduling.
- Converter dependency invalidation for secondary source paths, including
  `DataConverterOperationViewModel` dependencies.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- The runtime graph accepts a direct formula random function when
  `randomModeValue == 2`.
- The first source-to-target advance consumes the first supplied random value.
- Mutating the bound default number source clears the formula random cache.
- The next source-to-target advance consumes the next supplied random value.
- A later source-to-target advance without another source mutation reuses the
  second value.
- Existing default-mode and always-mode random formula tests still pass.
