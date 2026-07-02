# Data-Binding Graph Formula List Context Runtime Contract

## Scope

This slice covers deterministic `DataConverterFormula` fallback execution for
list sources that come from imported and owned view-model runtime contexts.

The covered graph shape has two state-machine data binds sharing
`ViewModelPropertyList.items`:

- Data bind `0` feeds a `BindablePropertyNumber.propertyValue` target through
  a direct deterministic `DataConverterFormula` and must produce C++'s numeric
  fallback.
- Data bind `1` is a direct list observer and must see the same rebound
  context source item count.

## C++ Parity Points

- Imported context mutation updates every same-path list source node in the
  bound graph, not just the source node selected by the mutating data-bind
  index.
- Owned context binding refreshes every same-path list source node from the
  owned runtime view-model instance.
- The formula number target, direct list observer target, and Rust graph source
  item counts match the C++ probe after repeated state-machine advances.

## Out Of Scope

- Generated list item identity beyond item-count parity.
- Formula random functions and real RNG generation.
- Reverse propagation, public `updateDataBinds(true)`, and target-dirty
  scheduling for imported or owned formula contexts.
- Relative/parent/nested lookup, listener-owned data binding, nested artboard
  propagation, and full dirty-list scheduler parity.

## Tests

- `state_machine_imported_viewmodel_list_formula_context_matches_cpp_probe`
- `state_machine_owned_viewmodel_list_formula_context_matches_cpp_probe`
