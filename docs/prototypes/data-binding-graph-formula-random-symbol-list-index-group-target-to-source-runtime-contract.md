# Data Binding Graph Formula Random Symbol-List-Index Group Target-To-Source Runtime Contract

## Purpose

Pin explicit target-to-source scheduling for grouped symbol-list-index random
formula binds.

This extends the grouped symbol-list-index random formula source-to-target
contracts by proving that a main-`ToSource | TwoWay` number target can be
manually edited, explicitly advanced through `advance_data_context`, preserve
the unchanged symbol-list-index source, and then reapply that source through
the grouped formula path with C++ parity.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Main-`ToSource | TwoWay` data-bind flags advanced through
  `StateMachineInstance::advance_data_context`.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Preservation of the unchanged symbol-list-index source when reverse formula
  conversion produces a number.
- Reapplication of the preserved source through grouped reverse order, including
  the operation-value reverse scale visible in C++ target reports.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.

## Out Of Scope

- Grouped symbol-list-index source-to-target behavior, covered by
  `data-binding-graph-formula-random-symbol-list-index-group-runtime-contract.md`
  and
  `data-binding-graph-formula-random-symbol-list-index-group-non-default-runtime-contract.md`.
- Grouped symbol-list-index public-update target-to-source and target-dirty
  scheduling.
- Direct symbol-list-index random formula behavior, covered by the direct
  symbol-list-index random contracts.
- List formula random scheduling and non-number fallback random scheduling.
- Stateful grouped converters mixed with symbol-list-index random formulas.
- A real Rust random generator or parity with C++ `std::rand()`.
- Probe CLI support for seeding or queuing C++ runtime random values.
- Random call-count parity outside the observed grouped bind actions.
- Formula `addDirt` random-cache behavior and secondary dependency
  invalidation.
- External, imported, and owned contexts for this converter/source
  combination.
- Relative-path, parent-path, nested-path, listener-owned, and update-queue
  behavior.

## Completion Checks

- Explicit `advance_data_context` target-to-source applies for grouped
  symbol-list-index random formula binds with random modes `0`, `1`, and `2`.
- The symbol-list-index source remains unchanged when grouped reverse
  conversion produces a number.
- The same explicit pass marks the bind dirty for source-to-target reapply.
- C++ probe number reports match Rust for the explicit target-to-source report
  and the later normal state-machine advances.
