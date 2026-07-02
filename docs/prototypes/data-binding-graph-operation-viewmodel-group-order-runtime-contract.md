# Operation-ViewModel Group Order Runtime Contract

## Scope

This slice covers the first observable non-default converter order for
`DataConverterOperationViewModel` nested inside a `DataConverterGroup`.

The covered graph shape is:

- a state-machine number bind whose converter group applies
  `DataConverterOperationViewModel` before `DataConverterOperationValue`;
- the operation-viewmodel converter multiplies by
  `ViewModelInstanceNumber.factor`;
- the operation-value converter then subtracts a literal value;
- an imported runtime view-model context changes the factor operand.

The mixed multiply/subtract operations make group order observable: C++ should
produce `operation_value(operation_view_model(source))`, not the old
`operation_view_model(operation_value(source))` path.

## C++ Parity Points

- `DataConverterGroup` honors imported `DataConverterGroupItem.converterId`
  ordering when `DataConverterOperationViewModel` appears before
  `DataConverterOperationValue`.
- Binding an imported view-model instance refreshes the nested
  operation-viewmodel secondary number operand before the ordered group runs.
- Direct source binds in the same fixture still report the imported source
  values that C++ reports.

## Out Of Scope

- Exhaustive converter-group permutations beyond the observed
  `OperationViewModel, OperationValue` order.
- Relative/name converter paths and non-number secondary operands.
- Mutation-driven recompute for non-default group orders.
- Broader dirty-list scheduling, listener-owned data binding, nested artboard
  propagation, and update queues.

## Tests

- `operation_viewmodel_group_reverse_order_imported_context_matches_cpp_probe`
- `operation_viewmodel`
