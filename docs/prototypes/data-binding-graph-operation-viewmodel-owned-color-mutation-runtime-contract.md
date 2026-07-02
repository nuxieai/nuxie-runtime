# Operation ViewModel Owned Color Mutation Runtime Contract

Purpose: pin owned-context color source mutation for
`DataConverterOperationViewModel` secondary operands without expanding the
converter's accepted operand types.

## Scope

- A state machine can bind an owned `RuntimeOwnedViewModelInstance` with a
  root number source and a root color source.
- After binding, `StateMachineInstance::set_owned_view_model_context_color_source_for_data_bind`
  mutates the color source by state-machine data-bind index.
- Direct `DataConverterOperationViewModel` secondary operands whose source path
  points at that color source keep C++'s `0.0` non-number fallback.
- `DataConverterGroup<OperationValue, OperationViewModel>` keeps the same
  fallback behavior for the nested operation-viewmodel operand.
- A same-path color observer bind updates alongside the mutation so the source
  mutation itself is still observable.

## Covered Cases

- Direct `DataConverterOperationViewModel` with an owned color secondary source
  mutation after owned context binding.
- Grouped `DataConverterGroup<OperationValue, OperationViewModel>` with the
  same owned color secondary source mutation after owned context binding.

## Out Of Scope

- Number, symbol-list-index, boolean, and string owned mutation, covered by
  their own contracts.
- Enum, asset, artboard, trigger, list, or view-model OperationViewModel
  secondary operand mutation.
- Making `DataConverterOperationViewModel` accept non-number secondary sources.
- Relative, parent, manifest-name, or nested converter source paths.
- Broader dirty/update queues, listener-owned data binding, and nested artboard
  propagation.

## Verification

- Focused C++ probe tests:
  - `operation_viewmodel_owned_color_source_mutation_fallback_matches_cpp_probe`
  - `operation_viewmodel_group_owned_color_source_mutation_fallback_matches_cpp_probe`
- Full C++ probe and workspace tests should remain green before committing.
