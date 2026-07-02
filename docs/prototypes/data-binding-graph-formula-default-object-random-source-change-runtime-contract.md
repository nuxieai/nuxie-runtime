# Data Binding Graph Formula Default Object Random Source-Change Runtime Contract

## Goal

Pin the C++ parity boundary for default-context object sources flowing through a
direct `DataConverterFormula` random fallback converter when the object source is
mutated after binding.

This slice covers the observed C++ behavior that `FormulaTokenFunction(random)`
inside an object-source formula fallback is random-inert: even with
`DataConverterFormula.randomModeValue == 2`, default asset, artboard, and
view-model pointer source mutation/relink do not pull from the runtime random
provider.

## In Scope

- Default view-model state-machine context binding.
- Direct graph-owned `DataConverterFormula` converters with
  `FormulaTokenFunction(random)` fallback tokens and `randomModeValue == 2`.
- Default root asset source mutation through
  `StateMachineInstance::set_default_view_model_asset_source_for_data_bind`.
- Default root artboard source mutation through
  `StateMachineInstance::set_default_view_model_artboard_source_for_data_bind`.
- Default root view-model pointer relink through
  `StateMachineInstance::relink_default_view_model_view_model_source_for_data_bind`
  followed by the explicit data-context advance required by the existing pointer
  relink path.
- Same-path direct object observer rows for asset, artboard, and view-model
  pointer sources.
- C++ counted runtime random provider comparison through
  `runtimeStateMachineAdvances[].randomTotalCalls`.

## Out Of Scope

- Number and symbol-list-index formula random source-change behavior; those are
  covered by their scalar formula random contracts.
- Imported and owned context post-bind object mutation random-inert checks.
- Grouped object-source formula fallback random permutations.
- Real random generation, random seeding, platform RNG behavior, or parity with
  C++ `std::rand()`.
- Relative, parent, name-based, or nested source path admission.
- Listener-owned data binding, nested artboard propagation, and full dirty-list
  scheduler parity.

## Required Behavior

- A default object source bound through direct `DataConverterFormula(random)`
  fallback keeps `data_bind_formula_random_call_count() == 0` after the initial
  source-to-target advance.
- Mutating the default asset or artboard source updates the same-path object
  observer and keeps the formula random call count at zero on the next
  source-to-target advance.
- Relinking the default view-model pointer updates the same-path pointer
  observer after the explicit data-context advance and keeps the formula random
  call count at zero.
- Later state-machine advances keep reusing the same random-inert object
  fallback result and do not pull from the host-supplied random stream.
- Rust and C++ `randomTotalCalls` remain equal for every reported action.

## Validation

- `state_machine_default_viewmodel_asset_formula_random_source_change_mutation_matches_cpp_probe`
- `state_machine_default_viewmodel_artboard_formula_random_source_change_mutation_matches_cpp_probe`
- `state_machine_default_viewmodel_viewmodel_formula_random_source_change_relink_matches_cpp_probe`
