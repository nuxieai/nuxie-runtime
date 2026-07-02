# Data Binding Graph Formula Random Symbol-List-Index Target-To-Source Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to target-to-source
scheduling for default-context symbol-list-index sources feeding number targets.

For a number target fed by a `ViewModelInstanceSymbolListIndex` source through
a direct `DataConverterFormula` random function, C++ evaluates the formula
during target-to-source conversion but does not write the produced number back
into the symbol-list-index source. The unchanged source is then reapplied
through the same random formula scheduling path.

## In Scope

- `StateMachineInstance::advanceDataContext()` for main-`ToSource | TwoWay`
  number binds.
- `StateMachineInstance::updateDataBinds(true)` for main-`ToTarget | TwoWay`
  number binds.
- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from `DataBind.converterId`.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Type-mismatch source preservation when target-to-source formula conversion
  produces a number for the symbol-list-index source.
- Same-pass source-to-target reapplication for explicit data-context
  advancement and public update.
- Default-mode and source-change-mode cache reuse when the symbol-list-index
  source is preserved rather than changed.
- Always-mode fresh random consumption, including the hidden target-to-source
  formula evaluation whose produced number is not observable as a typed source
  value in the C++ probe.
- Exact C++ probe reporting for the bind's target value after each explicit
  runtime action.

## Out Of Scope

- Source-to-target symbol-list-index random formula behavior, covered by
  `data-binding-graph-formula-random-symbol-list-index-runtime-contract.md`,
  `data-binding-graph-formula-random-symbol-list-index-always-runtime-contract.md`,
  and
  `data-binding-graph-formula-random-symbol-list-index-source-change-runtime-contract.md`.
- Target-dirty scheduling for symbol-list-index random formula binds.
- Grouped symbol-list-index random formula scheduling.
- List formula random scheduling and non-number fallback random scheduling.
- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Random call-count parity outside the observed direct bind actions.
- Formula `addDirt` random-cache behavior and secondary dependency
  invalidation.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- A mutated explicit main-`ToSource | TwoWay` random-formula-bound number
  target does not write its converted number into the symbol-list-index source.
- A mutated public main-`ToTarget | TwoWay` random-formula-bound number target
  does not write its reverse-converted number into the symbol-list-index
  source.
- Explicit data-context advancement and public update both reapply the
  unchanged symbol-list-index source through the random formula path.
- Random modes `0`, `1`, and `2` match the C++ probe target reports.
- Always-mode coverage accounts for the hidden target-to-source random draw by
  supplying an unobserved random value before the C++-inferred reapply value.
- Existing source-to-target symbol-list-index random formula probes continue to
  pass.
