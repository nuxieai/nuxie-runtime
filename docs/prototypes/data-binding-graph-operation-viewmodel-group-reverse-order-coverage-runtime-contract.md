# Operation-ViewModel Group Reverse Order Coverage Runtime Contract

## Scope

This slice expands observable coverage for two-item converter groups where
`DataConverterOperationViewModel` runs before `DataConverterOperationValue`.

The covered graph shape is:

- a state-machine number bind whose converter group executes
  `DataConverterOperationViewModel` first;
- the same group then executes `DataConverterOperationValue`;
- the operation-viewmodel converter uses additive conversion against the
  `factor` source;
- the operation-value converter multiplies by `2.0`, making group order
  observable.

The covered runtime contexts are:

- default view-model context binding;
- owned runtime view-model context binding with an owned `factor` override;
- imported runtime view-model context binding followed by mutating the bound
  imported `factor` source.

## C++ Parity Points

- Converter group child order follows the authored `DataConverterGroupItem`
  order.
- Binding default, owned, or imported runtime contexts refreshes the nested
  operation-viewmodel operand before ordered conversion.
- Mutating the already-bound imported secondary number source refreshes the
  nested operation-viewmodel operand before ordered conversion.

## Out Of Scope

- Longer converter groups.
- Converter kinds other than `DataConverterOperationViewModel` and
  `DataConverterOperationValue`.
- Relative/name converter paths.
- Owned-context source mutation APIs.
- Broader dirty-list scheduling, listener-owned data binding, nested artboard
  propagation, and update queues.

## Tests

- `operation_viewmodel_group_reverse_order_default_context_matches_cpp_probe`
- `operation_viewmodel_group_reverse_order_owned_context_matches_cpp_probe`
- `operation_viewmodel_group_reverse_order_imported_source_mutation_matches_cpp_probe`
- `operation_viewmodel`
