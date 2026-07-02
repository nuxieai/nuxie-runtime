# Data Binding Graph Formula Owned/Imported Object Random Group Source-Change Runtime Contract

## Goal

Pin C++ parity for imported and owned object sources that are mutated or
relinked after binding while flowing through a grouped formula fallback
converter.

This slice extends imported and owned grouped object fallback coverage from
initial source-to-target binding to post-bind source changes. The covered
converter shape is a `DataConverterGroup` whose first child is a direct
`DataConverterFormula` fallback converter and whose second child is a simple
`DataConverterOperationValue`.

## In Scope

- Imported view-model state-machine contexts bound through
  `StateMachineInstance::bind_imported_view_model_context`.
- Owned view-model state-machine contexts bound through
  `StateMachineInstance::bind_owned_view_model_context`.
- Imported asset/artboard post-bind mutation through the imported context
  source mutation APIs.
- Imported view-model pointer relink followed by the explicit data-context
  advance required by the existing pointer relink path.
- Owned asset/artboard post-bind mutation through the owned context source
  mutation APIs.
- Owned view-model pointer post-bind mutation through the owned context pointer
  source mutation API.
- Same-path direct object observer rows for asset, artboard, and view-model
  pointer sources.
- `DataConverterGroup<FormulaFallback, OperationValue>` source-to-target
  reapplication into a `BindablePropertyNumber` target after the source change.
- Formula fallback token variants:
  - `FormulaTokenInput`.
  - `FormulaTokenFunction(random)` with `randomModeValue` values `0`, `1`, and
    `2`.
- C++ counted runtime random provider comparison through
  `runtimeStateMachineAdvances[].randomTotalCalls`.

## Out Of Scope

- Default-context post-bind grouped object mutation/relink behavior, covered by
  `data-binding-graph-formula-default-object-random-group-source-change-runtime-contract.md`.
- Target-to-source/public-update behavior for grouped object fallback
  converters.
- Longer converter groups and other group order permutations.
- Real random generation, random seeding, platform RNG behavior, or parity with
  C++ `std::rand()`.
- Relative, parent, name-based, or nested source path admission.
- Listener-owned data binding, nested artboard propagation, and full dirty-list
  scheduler parity.

## Required Behavior

- Mutating imported or owned asset/artboard sources after binding must update
  the same-path object observer and reapply the grouped converter to the number
  target with C++-matching output on the next state-machine advance.
- Relinking imported view-model pointer sources must update the same-path
  pointer observer after the explicit data-context advance and reapply the
  grouped converter to the number target with C++-matching output.
- Mutating owned view-model pointer sources must update the same-path pointer
  observer and reapply the grouped converter to the number target with
  C++-matching output on the next state-machine advance.
- Random-function fallback tokens remain random-inert for object sources inside
  this group shape before and after source changes; Rust and C++ random call
  counts stay at zero for every reported action.

## Validation

- `state_machine_imported_viewmodel_object_formula_random_group_source_change_mutations_match_cpp_probe`
- `state_machine_owned_viewmodel_object_formula_random_group_source_change_mutations_match_cpp_probe`
