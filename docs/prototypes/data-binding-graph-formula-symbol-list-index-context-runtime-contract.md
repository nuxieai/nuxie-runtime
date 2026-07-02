# Data-Binding Graph Formula Symbol-List-Index Context Runtime Contract

## Scope

This slice covers deterministic `DataConverterFormula` execution for
symbol-list-index sources that come from imported and owned view-model runtime
contexts.

The covered graph shape is a state-machine `DataBindContext` from
`ViewModelPropertySymbolListIndex.symbol` to a
`BindablePropertyNumber.propertyValue` target through a direct
`DataConverterFormula` containing:

- `FormulaTokenInput`
- `FormulaTokenOperation`
- `FormulaTokenValue`

Rust must bind the selected runtime context first, cast the rebound
symbol-list-index source value to `f32`, and evaluate the formula from that
value on normal state-machine advancement.

## C++ Parity Points

- Imported context: `StateMachineInstance::bindViewModelInstance(...)` binds
  the file-backed `ViewModelInstance`, then mutating the bound
  symbol-list-index source updates the value seen by formula conversion.
- Owned context: `File::createViewModelInstance(...)` creates a runtime-owned
  instance; mutating its symbol-list-index property before binding updates the
  value seen by formula conversion.
- The formula target value, absent number-source value, and rebound
  symbol-list-index source value must match C++/Rust expectations for repeated
  state-machine advances.

## Out Of Scope

- Formula random functions and real RNG generation.
- Number contexts, covered separately by
  `data-binding-graph-formula-number-context-runtime-contract.md`.
- Boolean, enum, color, string, trigger, and list context formula paths.
- Reverse propagation, public `updateDataBinds(true)`, and target-dirty
  scheduling for imported or owned formula contexts.
- Imported-context sharing across multiple state machines. Scalar sharing is
  covered by the existing imported symbol-list-index shared-mutation slices;
  this slice only proves the shared value is fed through formula conversion.
- Generated list item identity, relative/parent/nested lookup, listener-owned
  data binding, nested artboard propagation, and full dirty-list scheduler
  parity.

## Tests

- `state_machine_imported_viewmodel_symbol_list_index_formula_context_matches_cpp_probe`
- `state_machine_owned_viewmodel_symbol_list_index_formula_context_matches_cpp_probe`
