# Operation-ViewModel Non-Number Context Recompute Runtime Contract

## Scope

This slice covers C++ fallback behavior when
`DataConverterOperationViewModel` recomputes its secondary operand from a
non-default runtime context and the resolved operand is not a number.

The covered graph shapes are:

- a direct `DataConverterOperationViewModel` whose `sourcePathIds` point at a
  `ViewModelInstanceSymbolListIndex`;
- a `DataConverterGroup<OperationValue, OperationViewModel>` whose nested
  operation-viewmodel converter points at the same symbol-list-index operand;
- imported runtime view-model context binding;
- owned runtime view-model context binding.

The fixture uses additive operation-viewmodel conversion so the `0.0` fallback
operand remains observable even when an owned context's primary number starts
from its owned default value.

## C++ Parity Points

- C++ `DataConverterOperationViewModel::bindFromContext` stores a secondary
  source only when the resolved runtime value is `ViewModelInstanceNumber`.
- Imported contexts that resolve the operand path to
  `ViewModelInstanceSymbolListIndex` keep the `0.0` operand fallback.
- Owned contexts that resolve the operand path to
  `ViewModelInstanceSymbolListIndex` keep the `0.0` operand fallback.
- The same fallback applies when the operation-viewmodel converter is nested
  inside `DataConverterGroup<OperationValue, OperationViewModel>`.

## Out Of Scope

- Default-context non-number fallback, covered by
  `data-binding-graph-operation-viewmodel-non-number-operand-runtime-contract.md`.
- Relative/name converter paths.
- Mutation-driven recompute for non-number operands.
- Exhaustive converter group permutations.
- Broader dirty-list scheduling, listener-owned data binding, nested artboard
  propagation, and update queues.

## Tests

- `operation_viewmodel_imported_symbol_list_index_operand_fallback_matches_cpp_probe`
- `operation_viewmodel_owned_symbol_list_index_operand_fallback_matches_cpp_probe`
- `operation_viewmodel_group_imported_symbol_list_index_operand_fallback_matches_cpp_probe`
- `operation_viewmodel_group_owned_symbol_list_index_operand_fallback_matches_cpp_probe`
- `operation_viewmodel`
