# Data Binding Graph Formula Default Object Random Longer Group Target-To-Source Runtime Contract

## Goal

Pin C++ parity for default-context object sources flowing through three-child
grouped formula fallback converter chains during target-to-source application.

This slice covers explicit data-context advancement and public
`updateDataBinds(true)` behavior for these shapes:

- `DataConverterGroup<FormulaFallback, OperationValue, OperationValue>`
- `DataConverterGroup<OperationValue, FormulaFallback, OperationValue>`

## In Scope

- Default view-model state-machine context binding.
- Asset, artboard, and view-model pointer root sources.
- A number target edited through the runtime bindable-number test seam.
- Explicit target-to-source application through
  `StateMachineInstance::advance_data_context`.
- Public target-to-source application through
  `StateMachineInstance::update_data_binds_apply_target_to_source`.
- Formula fallback token variants:
  - `FormulaTokenInput`.
  - `FormulaTokenFunction(random)` with `randomModeValue` values `0`, `1`, and
    `2`.
- C++ counted runtime random provider comparison through
  `runtimeStateMachineAdvances[].randomTotalCalls`.

## Out Of Scope

- Four-child and five-or-more-child target-to-source grouped object fallback
  converter paths.
- Imported and owned target-to-source grouped object fallback contexts.
- Target-dirty grouped object fallback behavior outside the explicit and public
  update paths covered here.
- Groups with multiple formulas, no formula, or non-operation converter
  children around the formula.
- Relative, parent, name-based, or nested source path admission.
- Listener-owned data binding, nested artboard propagation, and full dirty-list
  scheduler parity.

## Required Behavior

- Three-child grouped object fallback converters with exactly one formula and
  otherwise only operation-value children must preserve object sources during
  number target-to-source application while still matching C++ target reports.
- Explicit target-to-source application must reverse-convert the edited number
  target through the longer group and reapply source-to-target with C++ parity.
- Public target-to-source update must apply the same longer group through the
  public update seam and preserve C++ report shape.
- Counted formula-random calls must match C++ for each covered object source
  kind, group order, and formula token variant.

## Validation

- `state_machine_default_viewmodel_object_formula_random_longer_group_fallbacks_explicit_target_to_source_match_cpp_probe`
- `state_machine_default_viewmodel_object_formula_random_longer_group_fallbacks_public_update_target_to_source_match_cpp_probe`
