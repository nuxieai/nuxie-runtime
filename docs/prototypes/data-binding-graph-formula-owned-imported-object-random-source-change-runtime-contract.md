# Data Binding Graph Formula Owned/Imported Object Random Source-Change Runtime Contract

## Goal

Extend the object-source formula fallback random-inert boundary to post-bind
imported and owned view-model contexts.

This slice covers C++ parity for direct `DataConverterFormula`
`FormulaTokenFunction(random)` fallback converters with `randomModeValue == 2`
when imported or owned asset, artboard, and view-model pointer sources are
mutated after the state-machine data-bind graph is already bound.

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
- C++ counted runtime random provider comparison through
  `runtimeStateMachineAdvances[].randomTotalCalls`.

## Out Of Scope

- Default-context object random-inert mutation/relink behavior, covered by
  `data-binding-graph-formula-default-object-random-source-change-runtime-contract.md`.
- Number and symbol-list-index formula random source-change behavior; those are
  covered by their scalar formula random contracts.
- Grouped object-source formula fallback random permutations.
- Real random generation, random seeding, platform RNG behavior, or parity with
  C++ `std::rand()`.
- Relative, parent, name-based, or nested source path admission.
- Listener-owned data binding, nested artboard propagation, and full dirty-list
  scheduler parity.

## Required Behavior

- Imported and owned object sources bound through direct
  `DataConverterFormula(random)` fallback keep
  `data_bind_formula_random_call_count() == 0` after the initial
  source-to-target advance.
- Mutating imported or owned asset/artboard sources updates the same-path object
  observer and keeps the formula random call count at zero on the next
  source-to-target advance.
- Relinking imported and owned view-model pointer sources updates the same-path
  pointer observer and keeps the formula random call count at zero.
- The imported pointer relink path remains tied to the explicit data-context
  advance already required for pointer relink parity.
- Later state-machine advances keep the object fallback random-inert and do not
  pull from the host-supplied random stream.
- Rust and C++ `randomTotalCalls` remain equal for every reported action.

## Validation

- `state_machine_imported_viewmodel_object_formula_random_source_change_mutations_match_cpp_probe`
- `state_machine_owned_viewmodel_object_formula_random_source_change_mutations_match_cpp_probe`
