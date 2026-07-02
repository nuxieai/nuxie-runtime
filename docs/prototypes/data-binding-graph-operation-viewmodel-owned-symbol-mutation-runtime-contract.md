# OperationViewModel Owned Symbol Mutation Runtime Contract

Purpose: close the owned-context non-number mutation sibling of the
OperationViewModel owned number mutation slice without admitting the full
owned-context mutation matrix.

## Scope

- A state machine can bind an owned `RuntimeOwnedViewModelInstance`.
- After binding, Rust exposes
  `StateMachineInstance::set_owned_view_model_context_symbol_list_index_source_for_data_bind`.
- The mutation is accepted only while an owned view-model context is active.
- The data-bind index must resolve to a graph-owned symbol-list-index source
  path in the active owned view-model instance.
- Mutating that source updates the owned context value and matching graph
  source nodes.
- A direct or grouped `DataConverterOperationViewModel` whose operand path
  points at the same symbol-list-index source keeps the C++ `0.0` numeric
  operand fallback; only the ordinary symbol-list-index binding changes.
- The C++ probe exposes
  `--runtime-set-owned-view-model-source-symbol-list-index` and retains active
  owned contexts created by the root number and symbol-list-index bind
  commands.

## Covered Cases

- Direct `DataConverterOperationViewModel` with an owned primary `amount`
  source and a post-bind owned secondary `symbol` mutation.
- `DataConverterGroup<OperationValue, OperationViewModel>` with the same
  post-bind owned secondary `symbol` mutation.
- The converted number binding and ordinary direct symbol-list-index binding
  are compared against C++ after the mutation and after a subsequent positive
  advance.

## Out Of Scope

- Owned-context mutation for boolean, string, color, enum, asset, artboard,
  trigger, list, or view-model sources.
- Property-name or nested owned source mutation through this state-machine API.
- Stable public identity handles for proving that a later Rust mutation is
  applied to the exact same owned context object that was bound.
- Owned-context mutation for converter families other than
  `DataConverterOperationViewModel`.
- Listener-owned data contexts, nested artboard propagation, and broader dirty
  scheduler parity.

## Verification

- Focused C++ probe tests:
  - `operation_viewmodel_owned_symbol_list_index_source_mutation_fallback_matches_cpp_probe`
  - `operation_viewmodel_group_owned_symbol_list_index_source_mutation_fallback_matches_cpp_probe`
- Broader `operation_viewmodel` C++ probe filtering should continue to pass.
- Full C++ probe and workspace tests should remain green before committing.
