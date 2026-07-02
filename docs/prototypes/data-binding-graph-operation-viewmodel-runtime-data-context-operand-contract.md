# Operation-ViewModel Runtime DataContext Operand Contract

## Scope

This slice makes direct `DataConverterOperationViewModel` secondary operand
resolution consume `RuntimeDataContext` instead of calling the binary
`DataContext` helper directly.

The behavior remains intentionally unchanged:

- the converter reads its `sourcePathIds` buffer;
- the default view-model instance is wrapped as a `RuntimeDataContext`;
- the secondary operand is resolved through absolute
  `viewModelId`/`viewModelPropertyId` lookup;
- missing, non-number, and name-path cases keep the existing C++ `0.0`
  operand fallback.

## C++ Parity Points

- The existing direct operation-view-model source-to-target, target-to-source,
  public-update, target-dirty, secondary-source mutation, and name-path
  unsupported probes continue to match C++.
- Manifest-relative converter source paths remain unsupported for
  `DataConverterOperationViewModel`, matching C++'s direct
  `DataContext::getViewModelProperty(sourcePathIds)` call.

## Out Of Scope

- Changing converter source-path semantics.
- Live relative/name converter paths.
- Imported or owned context recomputation for operation-view-model operands.
- Broader data-bind dirty queue or scheduler behavior.

## Tests

- `operation_viewmodel`
