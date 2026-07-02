# Operation ViewModel Owned Artboard Mutation Runtime Contract

Purpose: add the owned-context post-bind artboard source mutation bridge and
pin `DataConverterOperationViewModel` fallback behavior for artboard secondary
operands.

## Scope

- A state machine can bind an owned `RuntimeOwnedViewModelInstance` with a
  root number source and a root artboard source.
- After binding, `StateMachineInstance::set_owned_view_model_context_artboard_source_for_data_bind`
  mutates the artboard source by state-machine data-bind index.
- The C++ probe exposes the matching
  `--runtime-set-owned-view-model-source-artboard` action against the retained
  active owned view-model instance.
- Direct `DataConverterOperationViewModel` secondary operands whose source path
  points at that artboard source keep C++'s `0.0` non-number fallback.
- `DataConverterGroup<OperationValue, OperationViewModel>` keeps the same
  fallback behavior for the nested operation-viewmodel operand.
- A same-path artboard observer bind updates alongside the mutation so the
  source mutation itself is still observable.

## Covered Cases

- Direct `DataConverterOperationViewModel` with an owned artboard secondary
  source mutation after owned context binding.
- Grouped `DataConverterGroup<OperationValue, OperationViewModel>` with the
  same owned artboard secondary source mutation after owned context binding.

## Out Of Scope

- Number, symbol-list-index, boolean, string, color, enum, trigger, list, and
  asset owned mutation, covered by their own contracts.
- View-model OperationViewModel secondary operand mutation.
- Making `DataConverterOperationViewModel` accept non-number secondary sources.
- Nested artboard instancing, host remapping, animation/state-machine remapping,
  rendering, and draw side effects.
- Relative, parent, manifest-name, or nested converter source paths.
- Broader dirty/update queues, listener-owned data binding, and nested artboard
  propagation.

## Verification

- Focused C++ probe tests:
  - `operation_viewmodel_owned_artboard_source_mutation_fallback_matches_cpp_probe`
  - `operation_viewmodel_group_owned_artboard_source_mutation_fallback_matches_cpp_probe`
- Full C++ probe and workspace tests should remain green before committing.
