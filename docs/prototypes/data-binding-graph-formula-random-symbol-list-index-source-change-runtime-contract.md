# Data Binding Graph Formula Random SymbolListIndex Source Change Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to direct
`RandomMode::sourceChange` behavior for default-context symbol-list-index
sources feeding number targets.

This covers the C++ behavior where a `DataConverterFormula` random function
with `randomModeValue == 2` caches its random value until the bound
symbol-list-index source changes, then clears that formula cache and consumes a
fresh random value on the next formula evaluation.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from `DataBind.converterId`.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue == 2`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Source mutation through
  `set_default_view_model_symbol_list_index_source_for_data_bind`.
- Source-to-target state-machine advancement and C++ probe number reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Direct source-to-target random call counts for symbol-list-index sources are
  covered separately by
  `data-binding-graph-formula-random-symbol-list-index-call-count-runtime-contract.md`.
- Formula `addDirt` random-cache behavior for symbol-list-index sources.
- Target-to-source symbol-list-index random formula scheduling is covered
  separately by
  `data-binding-graph-formula-random-symbol-list-index-target-to-source-runtime-contract.md`.
- Target-dirty symbol-list-index random formula scheduling is covered
  separately by
  `data-binding-graph-formula-random-symbol-list-index-target-dirty-runtime-contract.md`.
- Grouped symbol-list-index default-mode source-to-target scheduling is
  covered separately by
  `data-binding-graph-formula-random-symbol-list-index-group-runtime-contract.md`.
- Grouped symbol-list-index `RandomMode::sourceChange` source-to-target
  scheduling is covered separately by
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
- Converter dependency invalidation for secondary source paths.
- Formula parent-source binding, source dependents, and add-dirt behavior.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- A default symbol-list-index source is converted to `f32` before random
  formula evaluation.
- The first source-to-target advance consumes the first supplied random value.
- Mutating the bound default symbol-list-index source clears the formula random
  cache.
- The next source-to-target advance consumes the next supplied random value.
- A later source-to-target advance without another source mutation reuses the
  second value.
- Existing default-mode and always-mode symbol-list-index random formula tests
  still pass.
