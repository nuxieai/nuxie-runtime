# Operation ViewModel Owned Asset Mutation Runtime Contract

Purpose: add the first missing owned-context post-bind asset source mutation
bridge and pin `DataConverterOperationViewModel` fallback behavior for asset
secondary operands.

## Scope

- A state machine can bind an owned `RuntimeOwnedViewModelInstance` with a
  root number source and a root asset source.
- After binding, `StateMachineInstance::set_owned_view_model_context_asset_source_for_data_bind`
  mutates the asset source by state-machine data-bind index.
- The C++ probe exposes the matching
  `--runtime-set-owned-view-model-source-asset` action against the retained
  active owned view-model instance.
- Direct `DataConverterOperationViewModel` secondary operands whose source path
  points at that asset source keep C++'s `0.0` non-number fallback.
- `DataConverterGroup<OperationValue, OperationViewModel>` keeps the same
  fallback behavior for the nested operation-viewmodel operand.
- A same-path asset observer bind updates alongside the mutation so the source
  mutation itself is still observable.

## Covered Cases

- Direct `DataConverterOperationViewModel` with an owned asset secondary source
  mutation after owned context binding.
- Grouped `DataConverterGroup<OperationValue, OperationViewModel>` with the
  same owned asset secondary source mutation after owned context binding.

## Out Of Scope

- Number, symbol-list-index, boolean, string, color, enum, trigger, and list
  owned mutation, covered by their own contracts.
- Artboard or view-model OperationViewModel secondary operand mutation.
- Making `DataConverterOperationViewModel` accept non-number secondary sources.
- Asset resolver behavior, render image handles, file-asset payload loading, or
  renderer integration.
- Relative, parent, manifest-name, or nested converter source paths.
- Broader dirty/update queues, listener-owned data binding, and nested artboard
  propagation.

## Verification

- Focused C++ probe tests:
  - `operation_viewmodel_owned_asset_source_mutation_fallback_matches_cpp_probe`
  - `operation_viewmodel_group_owned_asset_source_mutation_fallback_matches_cpp_probe`
- Full C++ probe and workspace tests should remain green before committing.
