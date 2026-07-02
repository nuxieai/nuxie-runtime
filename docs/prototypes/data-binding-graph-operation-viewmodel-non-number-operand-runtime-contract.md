# Operation-ViewModel Non-Number Operand Runtime Contract

## Scope

This slice covers the C++ fallback when `DataConverterOperationViewModel`
resolves a secondary operand path to a non-number view-model value.

The covered graph shapes are:

- a direct `DataConverterOperationViewModel` whose `sourcePathIds` point at a
  `ViewModelInstanceSymbolListIndex`;
- a `DataConverterGroup<OperationValue, OperationViewModel>` whose nested
  operation-viewmodel converter points at the same symbol-list-index operand;
- both bind a default root view-model context and feed a state-machine number
  bind.

## C++ Parity Points

- C++ `DataConverterOperationViewModel::bindFromContext` only stores
  `ViewModelInstanceNumber` secondary sources.
- If the resolved secondary value is not a number, conversion uses the `0.0`
  fallback operand.
- The grouped converter path preserves the same fallback when the
  operation-viewmodel converter is nested inside a converter group.

## Out Of Scope

- Imported or owned context recomputation for non-number secondary operands.
- Relative/name converter paths.
- Exhaustive converter-group permutations.
- Broader dirty-list scheduling, listener-owned data binding, nested artboard
  propagation, and update queues.

## Tests

- `operation_viewmodel_symbol_list_index_operand_fallback_matches_cpp_probe`
- `operation_viewmodel_group_symbol_list_index_operand_fallback_matches_cpp_probe`
- `operation_viewmodel`
