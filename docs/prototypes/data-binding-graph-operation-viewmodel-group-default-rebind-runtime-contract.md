# Operation-ViewModel Group Default Rebind Runtime Contract

## Scope

This slice covers default-context rebinding after a grouped
`DataConverterOperationViewModel` secondary operand has been recomputed from a
non-default runtime view-model context.

The covered graph shape is:

- a state-machine number bind whose converter group first applies
  `DataConverterOperationValue`;
- the same group then applies `DataConverterOperationViewModel`;
- the operation-viewmodel secondary operand is `ViewModelInstanceNumber.factor`;
- the state machine is first bound to an imported or owned runtime view-model
  context, then rebound to the default view-model context.

## C++ Parity Points

- After imported-context binding changes the nested operation-viewmodel
  secondary operand, rebinding the default context restores the stored default
  operand.
- After owned-context binding changes the nested operation-viewmodel secondary
  operand, rebinding the default context restores the stored default operand.
- Direct source binds in the same fixture continue to report the same C++
  source values before and after the default rebind.

## Out Of Scope

- Arbitrary converter group orders beyond
  `DataConverterGroup<OperationValue, OperationViewModel>`.
- Relative/name converter paths and non-number secondary operands.
- Mutation-driven recompute, covered by the grouped imported mutation slice.
- Broader dirty-list scheduling, listener-owned data binding, nested artboard
  propagation, and update queues.

## Tests

- `operation_viewmodel_group_imported_context_default_rebind_matches_cpp_probe`
- `operation_viewmodel_group_owned_context_default_rebind_matches_cpp_probe`
- `operation_viewmodel`
