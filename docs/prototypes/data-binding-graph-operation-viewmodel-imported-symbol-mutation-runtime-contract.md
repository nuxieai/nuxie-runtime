# Operation-ViewModel Imported Symbol Mutation Runtime Contract

## Scope

This slice covers imported runtime mutation of a non-number secondary operand
source used by `DataConverterOperationViewModel`.

The covered graph shapes are:

- a direct `DataConverterOperationViewModel` whose secondary source path points
  at `ViewModelInstanceSymbolListIndex`;
- a `DataConverterGroup<OperationValue, OperationViewModel>` whose nested
  operation-viewmodel converter points at the same symbol-list-index operand;
- an additional state-machine source bind for the symbol-list-index path so the
  imported source mutation targets the converter operand source path;
- imported runtime view-model context binding followed by mutating that symbol
  source.

## C++ Parity Points

- C++ `DataConverterOperationViewModel::bindFromContext` stores a secondary
  source only when the resolved runtime value is `ViewModelInstanceNumber`.
- Mutating an imported `ViewModelInstanceSymbolListIndex` source that matches
  the converter operand path updates the ordinary symbol-list-index bind, but
  the operation-viewmodel operand remains the `0.0` fallback.
- The same fallback applies when the operation-viewmodel converter is nested
  inside `DataConverterGroup<OperationValue, OperationViewModel>`.

## Out Of Scope

- Owned-context source mutation APIs.
- Number-source mutation, covered by the imported number mutation contracts.
- Relative/name converter paths.
- Exhaustive converter group permutations.
- Broader dirty-list scheduling, listener-owned data binding, nested artboard
  propagation, and update queues.

## Tests

- `operation_viewmodel_imported_symbol_list_index_source_mutation_fallback_matches_cpp_probe`
- `operation_viewmodel_group_imported_symbol_list_index_source_mutation_fallback_matches_cpp_probe`
- `operation_viewmodel`
