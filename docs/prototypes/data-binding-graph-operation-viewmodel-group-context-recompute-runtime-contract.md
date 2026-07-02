# Operation-ViewModel Group Context Recompute Runtime Contract

## Scope

This slice covers context-specific recomputation for
`DataConverterOperationViewModel` number operands nested inside
`DataConverterGroup<OperationValue, OperationViewModel>`.

The covered graph shape is:

- a state-machine number bind whose converter group first applies
  `DataConverterOperationValue`;
- the same group then applies `DataConverterOperationViewModel`;
- the operation-viewmodel secondary operand is `ViewModelInstanceNumber.factor`;
- imported and owned runtime view-model context binding can change the operand
  value seen by the nested operation-viewmodel converter.

## C++ Parity Points

- Binding an imported view-model instance refreshes the nested
  operation-viewmodel secondary number operand from that imported instance.
- Binding an owned runtime view-model instance refreshes the nested
  operation-viewmodel secondary number operand from owned runtime storage.
- Direct source binds in the same fixture still report the imported or owned
  source values that C++ reports.

## Out Of Scope

- Arbitrary converter group orders beyond
  `DataConverterGroup<OperationValue, OperationViewModel>`.
- Relative/name converter paths and non-number secondary operands.
- Mutation-driven recompute, covered by the grouped imported mutation slice.
- Broader dirty-list scheduling, listener-owned data binding, nested artboard
  propagation, and update queues.

## Tests

- `operation_viewmodel_group_imported_context_matches_cpp_probe`
- `operation_viewmodel_group_owned_context_matches_cpp_probe`
- `operation_viewmodel`
