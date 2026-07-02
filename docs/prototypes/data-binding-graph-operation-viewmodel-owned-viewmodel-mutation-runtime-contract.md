# Data Binding Graph OperationViewModel Owned ViewModel Mutation Runtime Contract

## Purpose

Admit owned runtime view-model pointer source mutation for
`DataConverterOperationViewModel` without expanding the data-binding runtime
surface beyond the existing owned source mutation family.

## In Scope

- Root owned view-model pointer sources reached through a `DataBindContext`
  source path such as `[rootViewModelIndex, propertyIndex]`.
- `StateMachineInstance::set_owned_view_model_context_view_model_source_for_data_bind`
  mutates the active owned context by referenced instance index, updates
  same-path graph-owned view-model source nodes, and marks the default
  view-model bindings dirty.
- Direct `DataConverterOperationViewModel` with an owned view-model pointer
  secondary source continues to use C++'s non-number `0.0` fallback after the
  pointer is relinked.
- `DataConverterGroup<OperationValue, OperationViewModel>` preserves the same
  fallback behavior while same-path view-model observer binds update to the
  selected referenced instance.
- The C++ probe exposes `--runtime-set-owned-view-model-source-viewmodel` using
  the active owned context retained by the existing owned bind commands.

## Out Of Scope

- Slash-separated owned view-model pointer name paths.
- Nested, relative, parent, imported-intermediate, and generated-child pointer
  source relinks.
- Reverse propagation, target-to-source writes, dirty queue broadening,
  listener-owned data binding, nested artboard propagation, cloning, and
  runtime graph scheduling beyond the existing state-machine advance hook.
- Numeric `DataConverterOperationViewModel` recomputation for pointer sources;
  this slice only preserves the non-number fallback parity.

## Completion Condition

- Direct and grouped owned `DataConverterOperationViewModel` pointer source
  mutation probes compare Rust against C++ for state-machine advance, number
  binding fallback, and same-path view-model source/target instance reports.
- Existing owned `OperationViewModel` scalar, list, asset, and artboard source
  mutation probes continue to pass.
