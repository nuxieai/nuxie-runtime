# OperationViewModel Owned Number Mutation Runtime Contract

Purpose: close the first owned-context source mutation gap for
`DataConverterOperationViewModel` without broadening the owned view-model API
surface beyond number sources.

## Scope

- A state machine can bind an owned `RuntimeOwnedViewModelInstance`.
- After binding, Rust exposes
  `StateMachineInstance::set_owned_view_model_context_number_source_for_data_bind`.
- The mutation is accepted only while an owned view-model context is active.
- The data-bind index must resolve to a graph-owned number source path in the
  active owned view-model instance.
- Mutating that source updates the owned context value, the matching graph
  source nodes, and any direct or grouped `DataConverterOperationViewModel`
  operands whose `sourcePathIds` match the mutated path.
- The C++ probe retains the active owned view-model instance for the
  number-context bind command and exposes
  `--runtime-set-owned-view-model-source-number` for parity comparison.

## Covered Cases

- Direct `DataConverterOperationViewModel` with an owned primary `amount`
  source and a post-bind owned secondary `factor` mutation.
- `DataConverterGroup<OperationValue, OperationViewModel>` with the same
  post-bind owned secondary `factor` mutation.
- The converted number binding and ordinary direct factor binding are compared
  against C++ after the mutation and after a subsequent positive advance.

## Out Of Scope

- Owned-context mutation for boolean, string, color, enum, symbol-list-index,
  asset, artboard, trigger, list, or view-model sources.
- Property-name or nested owned source mutation through this state-machine API.
- Mutating owned contexts created by probe commands other than the root number
  owned-context bind command.
- Stable public identity handles for proving that a later Rust mutation is
  applied to the exact same owned context object that was bound.
- Owned-context mutation for converter families other than
  `DataConverterOperationViewModel`.
- Listener-owned data contexts, nested artboard propagation, and broader dirty
  scheduler parity.

## Verification

- Focused C++ probe tests:
  - `operation_viewmodel_owned_secondary_source_mutation_matches_cpp_probe`
  - `operation_viewmodel_group_owned_secondary_source_mutation_matches_cpp_probe`
- Broader `operation_viewmodel` C++ probe filtering should continue to pass.
- Full C++ probe and workspace tests should remain green before committing.
