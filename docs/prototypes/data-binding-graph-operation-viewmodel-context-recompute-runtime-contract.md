# Operation-ViewModel Context Recompute Runtime Contract

## Scope

This slice covers context-specific recomputation for
`DataConverterOperationViewModel` number operands.

The covered graph shapes are direct `DataConverterOperationViewModel`
converters on state-machine number binds where:

- the primary source is `ViewModelInstanceNumber.amount`;
- the converter secondary operand is `ViewModelInstanceNumber.factor`;
- binding an imported or owned runtime view-model context changes the operand
  value seen by the converter.

## C++ Parity Points

- Binding an imported view-model instance refreshes the converter's secondary
  number operand from that imported instance's `DataContext`.
- Binding an owned runtime view-model instance refreshes the converter's
  secondary number operand from owned runtime storage.
- Rebinding the default context restores the converter's original default
  operand instead of retaining a prior imported or owned operand.
- Missing, non-number, and manifest-name converter paths keep the existing
  C++ `0.0` fallback behavior.

## Out Of Scope

- Relative/name converter paths for `DataConverterOperationViewModel`.
- Imported/owned recomputation for converter kinds other than
  `DataConverterOperationViewModel`.
- Non-number secondary operands.
- Broader dirty-list scheduling, pending add/remove behavior, and re-entry
  protection.

## Tests

- `state_machine_imported_viewmodel_number_operation_viewmodel_converter_matches_cpp_probe`
- `state_machine_owned_viewmodel_number_operation_viewmodel_converter_matches_cpp_probe`
  covers owned binding and default rebinding.
- `operation_viewmodel`
