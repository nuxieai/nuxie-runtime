# Data-Binding Graph Formula Number Context Runtime Contract

## Scope

This slice covers deterministic `DataConverterFormula` execution for number
sources that come from imported and owned view-model runtime contexts.

The covered graph shape is a state-machine `DataBindContext` from
`ViewModelPropertyNumber.amount` to a `BindablePropertyNumber.propertyValue`
target through a direct `DataConverterFormula` containing:

- `FormulaTokenInput`
- `FormulaTokenOperation`
- `FormulaTokenValue`

Rust must bind the selected runtime context first, then evaluate the formula
from the rebound context source value on normal state-machine advancement.

## C++ Parity Points

- Imported context: `StateMachineInstance::bindViewModelInstance(...)` binds
  the file-backed `ViewModelInstance`, then mutating the bound number source
  updates the value seen by formula conversion.
- Owned context: `File::createViewModelInstance(...)` creates a runtime-owned
  instance; mutating its number property before binding updates the value seen
  by formula conversion.
- The formula target value and source value reported through the number
  binding probe must match C++ for repeated state-machine advances.

## Out Of Scope

- Formula random functions and real RNG generation.
- Symbol-list-index and non-number context formula paths.
- Reverse propagation, public `updateDataBinds(true)`, and target-dirty
  scheduling for imported or owned formula contexts.
- Imported-context sharing across multiple state machines. Scalar sharing is
  covered by the existing imported number shared-mutation slices; this slice
  only proves the shared value is fed through formula conversion.
- Generated list item identity, relative/parent/nested lookup, listener-owned
  data binding, nested artboard propagation, and full dirty-list scheduler
  parity.

## Tests

- `state_machine_imported_viewmodel_number_formula_context_matches_cpp_probe`
- `state_machine_owned_viewmodel_number_formula_context_matches_cpp_probe`
