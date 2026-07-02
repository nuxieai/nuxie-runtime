# Data Binding Graph Formula Symbol-List-Index Main-To-Target Two-Way Target Dirty Runtime Contract

## Purpose

Pin main-`ToTarget | TwoWay` target-dirty behavior for deterministic
`DataConverterFormula` symbol-list-index sources feeding number targets.

C++ keeps the imported `ViewModelInstanceSymbolListIndex.propertyValue` source
unchanged when a formula-bound number target is manually edited. The manual
target edit survives explicit `advanceDataContext()`, and later normal
state-machine advancement reapplies source-to-target conversion, writing the
formula value derived from the unchanged symbol-list-index source.

## In Scope

- Default root view-model context bound with `bind_default_view_model_context`.
- Root-only `DataBindContext.sourcePathIds` of shape `[0, propertyIndex]`.
- `ViewModelInstanceSymbolListIndex.propertyValue` sources feeding
  `BindablePropertyNumber.propertyValue` targets.
- Direct `DataConverterFormula` converters resolved from
  `DataBind.converterId`.
- Deterministic formula output queues made only from `FormulaTokenInput`,
  `FormulaTokenValue`, and `FormulaTokenOperation`.
- Main-`ToTarget | TwoWay` data-bind flags, without the `ToSource` direction
  flag.
- Initial source-to-target flushing through a normal state-machine advance.
- Mutating the bindable number target by data-bind index.
- Explicit `advanceDataContext()` preserving the manual target edit before the
  next normal state-machine advance overwrites it.
- Existing C++ probe number binding reports:
  - source symbol-list-index remains unchanged,
  - target value remains the edited scalar after explicit data-context
    advancement,
  - target value becomes the formula value on later normal advance.

## Out Of Scope

- Source-to-target symbol-list-index formula behavior, covered by
  `data-binding-graph-formula-symbol-list-index-converter-runtime-contract.md`.
- Public `updateDataBinds(true)` target-to-source behavior, covered by
  `data-binding-graph-formula-symbol-list-index-public-update-target-to-source-runtime-contract.md`.
- Explicit target-to-source behavior, covered by
  `data-binding-graph-formula-symbol-list-index-explicit-target-to-source-runtime-contract.md`.
- Deterministic `FormulaTokenFunction` support, covered separately by
  `data-binding-graph-formula-functions-runtime-contract.md`.
- Random-function symbol-list-index formula target-dirty behavior, covered by
  `data-binding-graph-formula-random-symbol-list-index-target-dirty-runtime-contract.md`.
- Formula converter groups.
- Imported, owned, relative, parent, and nested view-model contexts.
- Secondary dependency invalidation and full dirty-list scheduler parity.

## Completion Checks

- The initial normal state-machine advance applies the symbol-list-index
  formula value to `BindablePropertyNumber.propertyValue`.
- A manual edit to the number target is preserved through explicit
  data-context advancement.
- A later normal state-machine advance reapplies the formula value from the
  unchanged symbol-list-index source.
- Existing symbol-list-index formula source-to-target and reverse probes
  continue to pass.
