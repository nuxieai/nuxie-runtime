# Data Binding Graph Formula Owned/Imported Object Random Group Context Runtime Contract

## Goal

Pin C++ parity for imported and owned object sources flowing through a grouped
formula fallback converter in source-to-target application.

This slice extends the default-context grouped object fallback coverage to
alternate imported view-model contexts and API-owned view-model contexts. The
covered converter shape is a `DataConverterGroup` whose first child is a direct
`DataConverterFormula` fallback converter and whose second child is a simple
`DataConverterOperationValue`.

## In Scope

- Imported view-model state-machine contexts bound through
  `StateMachineInstance::bind_imported_view_model_context`.
- Owned view-model state-machine contexts bound through
  `StateMachineInstance::bind_owned_view_model_context`.
- Asset, artboard, and view-model pointer root sources.
- Same-path direct object observer rows for asset, artboard, and view-model
  pointer sources.
- `DataConverterGroup<FormulaFallback, OperationValue>` source-to-target
  application into a `BindablePropertyNumber` target.
- Formula fallback token variants:
  - `FormulaTokenInput`.
  - `FormulaTokenFunction(random)` with `randomModeValue` values `0`, `1`, and
    `2`.
- C++ counted runtime random provider comparison through
  `runtimeStateMachineAdvances[].randomTotalCalls`.

## Out Of Scope

- Default-context grouped object fallback behavior, covered by
  `data-binding-graph-formula-default-object-random-group-fallback-runtime-contract.md`.
- Post-bind source mutation/relink through grouped object fallback converters.
- Target-to-source/public-update behavior for grouped object fallback
  converters.
- Longer converter groups and other group order permutations.
- Real random generation, random seeding, platform RNG behavior, or parity with
  C++ `std::rand()`.
- Relative, parent, name-based, or nested source path admission.
- Listener-owned data binding, nested artboard propagation, and full dirty-list
  scheduler parity.

## Required Behavior

- Imported and owned object sources bound through a formula-first converter
  group must preserve the active asset, artboard, or view-model pointer source
  value while driving the number target with C++-matching source-to-target
  output.
- Imported alternate contexts must read the alternate root source selected at
  bind time for asset, artboard, and view-model pointer sources.
- Owned contexts must read the API-provided asset, artboard, or view-model
  pointer source selected before bind.
- Random-function fallback tokens remain random-inert for object sources inside
  this group shape; Rust and C++ random call counts stay at zero for every
  reported action.

## Validation

- `state_machine_imported_viewmodel_object_formula_random_group_contexts_match_cpp_probe`
- `state_machine_owned_viewmodel_object_formula_random_group_contexts_match_cpp_probe`
