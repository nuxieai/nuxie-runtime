# Operation-ViewModel Group Imported Number Mutation Runtime Contract

## Scope

This slice covers imported-context number source mutation for the grouped
`DataConverterGroup<OperationValue, OperationViewModel>` path.

The covered graph shape is:

- a state-machine number bind whose converter group first applies
  `DataConverterOperationValue`;
- the same group then applies `DataConverterOperationViewModel`;
- the operation-viewmodel secondary operand is `ViewModelInstanceNumber.factor`;
- a separate number bind exposes `factor` so the imported context can mutate
  the same resolved source path by data-bind index.

## C++ Parity Points

- Mutating the imported `factor` number source after binding an imported
  context refreshes the nested operation-viewmodel operand inside the group.
- The grouped converted `amount` source, the direct `amount` source, and the
  direct `factor` source all match C++ advance reports.
- The recursive refresh updates live imported operands only and leaves stored
  default operands intact.

## Out Of Scope

- Arbitrary converter group orders beyond
  `DataConverterGroup<OperationValue, OperationViewModel>`.
- Relative/name converter paths and non-number secondary operands.
- Mutation-driven recompute for converter families other than
  `DataConverterOperationViewModel`.
- Broader dirty-list scheduling, listener-owned data binding, nested artboard
  propagation, and update queues.

## Tests

- `operation_viewmodel_group_imported_secondary_source_mutation_matches_cpp_probe`
- `operation_viewmodel`
