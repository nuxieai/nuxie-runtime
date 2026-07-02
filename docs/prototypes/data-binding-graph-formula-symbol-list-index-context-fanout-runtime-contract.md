# Data-Binding Graph Formula Symbol-List-Index Context Fanout Runtime Contract

## Scope

This slice covers same-path source fanout for deterministic
`DataConverterFormula` symbol-list-index contexts.

The covered graph shape has two state-machine data binds sharing
`ViewModelPropertySymbolListIndex.symbol`:

- Data bind `0` feeds a `BindablePropertyNumber.propertyValue` target through
  a direct deterministic `DataConverterFormula`.
- Data bind `1` is a direct integer observer and must see the same rebound
  symbol-list-index source value.

## C++ Parity Points

- Imported context mutation updates every same-path symbol-list-index source
  node in the bound graph, not just the source node selected by the mutating
  data-bind index.
- Owned context binding refreshes every same-path symbol-list-index source node
  from the owned runtime view-model instance.
- The formula number target, direct integer observer target, and Rust graph
  source values match the C++ probe after repeated state-machine advances.

## Out Of Scope

- Multi-state-machine imported context sharing beyond the already covered
  shared-mutation slices.
- Formula random functions and real RNG generation.
- Reverse propagation, public `updateDataBinds(true)`, and target-dirty
  scheduling for imported or owned formula contexts.
- Relative/parent/nested lookup, listener-owned data binding, nested artboard
  propagation, and full dirty-list scheduler parity.

## Tests

- `state_machine_imported_viewmodel_symbol_list_index_formula_context_matches_cpp_probe`
- `state_machine_owned_viewmodel_symbol_list_index_formula_context_matches_cpp_probe`
