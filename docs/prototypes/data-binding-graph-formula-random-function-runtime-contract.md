# Data Binding Graph Formula Random Function Runtime Contract

## Purpose

Admit the first graph-owned `FunctionType::random` formula slice with an
explicit host-supplied random stream.

This slice does not introduce real runtime random generation. It gives the
state-machine data-bind graph a narrow way to receive formula random values
from the host/test harness, then cache those values per formula converter the
way C++ default random mode caches `m_randoms`. The C++ probe comparison derives
the first random draw from the C++ number-binding report and supplies that same
draw to Rust before advancing.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceNumber.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- Formula output-queue descriptors exposed by
  `RuntimeFile::data_converter_formula_output_tokens_for_object`.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- Default `DataConverterFormula.randomModeValue == 0`.
- `StateMachineInstance::set_data_bind_formula_random_values` as a
  host-supplied graph formula random stream.
- Per-formula default-mode random caching, so later evaluations reuse the same
  random draw for the same random-token index.
- One-argument and two-argument random bounds using the same stack ordering as
  C++ `DataConverterFormula::applyFunction`.
- C++ probe coverage through an existing `BlendState1DViewModel` consumer and
  number binding reports.

## Out Of Scope

- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- `RandomMode::always`, `RandomMode::sourceChange`, random cache invalidation,
  random call-count parity, and formula `addDirt` random-cache behavior.
- Target-to-source, public update, target-dirty, grouped formula, and list
  formula scheduling with random functions.
- Random formula behavior for symbol-list-index or non-number sources.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- The runtime graph accepts a direct formula random function when
  `randomModeValue` is the default `0`.
- The same graph rejects random functions with non-default `randomModeValue`
  until cache semantics have their own contract.
- A two-bound random formula writes the same target value as C++ when the Rust
  graph receives the same first random draw C++ used.
- Later source-to-target evaluations reuse the cached random draw for the same
  formula converter.
- Existing deterministic formula-function graph tests still pass.
