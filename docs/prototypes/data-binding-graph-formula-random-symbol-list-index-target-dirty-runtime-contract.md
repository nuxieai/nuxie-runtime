# Data Binding Graph Formula Random Symbol-List-Index Target-Dirty Runtime Contract

## Purpose

Extend the host-supplied graph formula random slice to main-`ToTarget | TwoWay`
target-dirty scheduling for default-context symbol-list-index sources feeding
number targets.

This covers the C++ behavior where the initial source-to-target pass evaluates
the symbol-list-index source through a direct random formula, a manual number
target edit is preserved through explicit data-context advancement, and later
normal state-machine advances reapply the unchanged source through the random
formula path.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from `DataBind.converterId`.
- Main-`ToTarget | TwoWay` data-bind flags.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.
- Explicit data-context advancement preserving a manual target edit.
- Default-mode and source-change-mode cache reuse when no symbol-list-index
  source mutation occurs.
- Always-mode fresh random consumption on later normal state-machine advances.
- Exact C++ probe reporting for the bind's target value after each explicit
  runtime action.

## Out Of Scope

- Source-to-target symbol-list-index random formula behavior, covered by
  `data-binding-graph-formula-random-symbol-list-index-runtime-contract.md`,
  `data-binding-graph-formula-random-symbol-list-index-always-runtime-contract.md`,
  and
  `data-binding-graph-formula-random-symbol-list-index-source-change-runtime-contract.md`.
- Target-to-source symbol-list-index random formula behavior, covered by
  `data-binding-graph-formula-random-symbol-list-index-target-to-source-runtime-contract.md`.
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

- The initial source-to-target pass consumes or caches random values according
  to `randomModeValue`.
- Explicit data-context advancement preserves the manually edited number
  target and does not treat that edit as a symbol-list-index source change.
- Later normal state-machine advances reapply the unchanged symbol-list-index
  source through the random formula path.
- Random modes `0`, `1`, and `2` match the C++ probe target reports.
- Existing source-to-target and target-to-source symbol-list-index random
  formula probes continue to pass.
