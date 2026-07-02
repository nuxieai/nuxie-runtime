# Operation-ViewModel Imported Number Mutation Runtime Contract

## Scope

This slice covers live imported-context number source mutation for
`DataConverterOperationViewModel` secondary operands.

The covered graph shape is:

- a state-machine number bind whose primary source is
  `ViewModelInstanceNumber.amount`;
- a direct `DataConverterOperationViewModel` whose secondary operand source is
  `ViewModelInstanceNumber.factor`;
- a second state-machine number bind for `factor`, so the imported context can
  mutate the same resolved source path by data-bind index.

## C++ Parity Points

- Mutating the imported `factor` number source after binding an imported
  context refreshes the operation-viewmodel operand used by the `amount` bind.
- The mutated `factor` source remains visible through its own bind.
- Only the live imported operand is changed; the stored default operand remains
  available for later default-context rebinding.

## Out Of Scope

- Default-context source mutation, already covered separately.
- Owned-context source mutation, already covered by context rebinding.
- Imported mutation for converter families other than
  `DataConverterOperationViewModel`.
- Relative/name converter paths and non-number secondary operands.
- Broader dirty-list scheduling, listener-owned data binding, nested artboard
  propagation, and update queues.

## Tests

- `operation_viewmodel_imported_secondary_source_mutation_matches_cpp_probe`
- `operation_viewmodel`
