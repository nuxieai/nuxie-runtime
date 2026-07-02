# Data Binding Graph Formula Default Object Random Group Target-To-Source Runtime Contract

## Goal

Pin C++ parity for default-context object sources flowing through a grouped
formula fallback converter during target-to-source application.

This slice covers explicit data-context advancement and public
`updateDataBinds(true)` behavior for a `DataConverterGroup` whose first child
is a direct `DataConverterFormula` fallback converter and whose second child is
a simple `DataConverterOperationValue`.

## In Scope

- Default view-model state-machine context binding.
- Asset, artboard, and view-model pointer root sources.
- A number target edited through the runtime bindable-number test seam.
- Explicit target-to-source application through
  `StateMachineInstance::advance_data_context`.
- Public target-to-source application through
  `StateMachineInstance::update_data_binds_apply_target_to_source`.
- `DataConverterGroup<FormulaFallback, OperationValue>` reverse conversion and
  subsequent source-to-target reapplication into a `BindablePropertyNumber`
  target.
- Formula fallback token variants:
  - `FormulaTokenInput`.
  - `FormulaTokenFunction(random)` with `randomModeValue` values `0`, `1`, and
    `2`.
- C++ counted runtime random provider comparison through
  `runtimeStateMachineAdvances[].randomTotalCalls`.

## Out Of Scope

- Imported and owned target-to-source grouped object fallback contexts.
- Target-dirty grouped object fallback behavior outside the explicit and public
  update paths covered here.
- Longer converter groups and other group order permutations.
- Real random generation, random seeding, platform RNG behavior, or parity with
  C++ `std::rand()`.
- Relative, parent, name-based, or nested source path admission.
- Listener-owned data binding, nested artboard propagation, and full dirty-list
  scheduler parity.

## Required Behavior

- Formula-first grouped object fallback converters must preserve the object
  source while still marking target-to-source application as applied, so the
  target report follows C++ after explicit and public target-to-source actions.
- Reverse `DataConverterOperationValue` over a non-number intermediate must
  produce C++'s numeric default value before the formula child runs.
- Explicit target-to-source reapplication must run the formula-first group in
  C++ reverse order for main-to-source binds.
- Public target-to-source update must reverse-convert the edited number target,
  preserve the object source, then reapply source-to-target in C++ forward
  order.
- Random call counts must match C++; explicit `randomModeValue == 1` observes
  the formula-first reverse path's extra random evaluation, while public update
  remains at one random call for the update cycle.

## Validation

- `state_machine_default_viewmodel_object_formula_random_group_fallbacks_explicit_target_to_source_match_cpp_probe`
- `state_machine_default_viewmodel_object_formula_random_group_fallbacks_public_update_target_to_source_match_cpp_probe`
