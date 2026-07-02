# Data Binding Graph Formula Default Object Random Group Source-Change Runtime Contract

## Goal

Pin C++ parity for default-context object sources that are mutated or relinked
after binding while flowing through a grouped formula fallback converter.

This slice extends default-context grouped object fallback coverage from initial
source-to-target binding to post-bind source changes. The covered converter
shape is a `DataConverterGroup` whose first child is a direct
`DataConverterFormula` fallback converter and whose second child is a simple
`DataConverterOperationValue`.

## In Scope

- Default view-model state-machine context binding.
- Post-bind default asset source mutation.
- Post-bind default artboard source mutation.
- Post-bind default view-model pointer relink plus the explicit data-context
  advance required by the existing pointer relink path.
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

- Imported and owned context post-bind grouped object mutation/relink behavior.
- Target-to-source/public-update behavior for grouped object fallback
  converters.
- Longer converter groups and other group order permutations.
- Real random generation, random seeding, platform RNG behavior, or parity with
  C++ `std::rand()`.
- Relative, parent, name-based, or nested source path admission.
- Listener-owned data binding, nested artboard propagation, and full dirty-list
  scheduler parity.

## Required Behavior

- Mutating a default asset or artboard source after binding must update the
  same-path object observer and reapply the grouped converter to the number
  target with C++-matching output on the next state-machine advance.
- Relinking a default view-model pointer source must update the same-path
  pointer observer after the explicit data-context advance and reapply the
  grouped converter to the number target with C++-matching output.
- Random-function fallback tokens remain random-inert for object sources inside
  this group shape before and after source changes; Rust and C++ random call
  counts stay at zero for every reported action.

## Validation

- `state_machine_default_viewmodel_object_formula_random_group_source_change_mutations_match_cpp_probe`
