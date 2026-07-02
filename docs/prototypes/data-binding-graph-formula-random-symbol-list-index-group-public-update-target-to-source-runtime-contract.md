# Data Binding Graph Formula Random Symbol-List-Index Group Public-Update Target-To-Source Runtime Contract

## Purpose

Pin public target-to-source scheduling for grouped symbol-list-index random
formula binds.

This extends the explicit grouped target-to-source contract by proving that a
main-`ToTarget | TwoWay` number target can be manually edited, applied through
`update_data_binds_apply_target_to_source`, preserve the unchanged
symbol-list-index source, and reapply that source in the same public update
with C++ parity.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Main-`ToTarget | TwoWay` data-bind flags with public
  `StateMachineInstance::update_data_binds_apply_target_to_source`.
- `DataConverterGroup` with a direct `DataConverterOperationValue` child
  followed by a direct `DataConverterFormula` child.
- `FormulaTokenFunction` with `functionType == FunctionType::random`.
- `DataConverterFormula.randomModeValue` values `0`, `1`, and `2`.
- Preservation of the unchanged symbol-list-index source when grouped reverse
  conversion produces a number.
- Same-update source-to-target reapplication after the public target-to-source
  pass.
- `StateMachineInstance::set_data_bind_formula_random_values` as the
  host-supplied graph formula random stream.

## Out Of Scope

- Grouped symbol-list-index source-to-target behavior, covered by
  `data-binding-graph-formula-random-symbol-list-index-group-runtime-contract.md`
  and
  `data-binding-graph-formula-random-symbol-list-index-group-non-default-runtime-contract.md`.
- Grouped symbol-list-index explicit target-to-source scheduling, covered by
  `data-binding-graph-formula-random-symbol-list-index-group-target-to-source-runtime-contract.md`.
- Grouped symbol-list-index target-dirty scheduling.
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

- Public `update_data_binds_apply_target_to_source` applies grouped
  symbol-list-index random formula target-to-source for random modes `0`, `1`,
  and `2`.
- The symbol-list-index source remains unchanged when grouped reverse
  conversion produces a number.
- The same public update reapplies the preserved source to the number target.
- C++ probe number reports match Rust for the initial source-to-target report,
  the public-update report, and the later normal state-machine advances.
