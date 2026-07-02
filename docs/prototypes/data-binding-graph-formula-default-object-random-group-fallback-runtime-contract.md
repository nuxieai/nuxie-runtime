# Data Binding Graph Formula Default Object Random Group Fallback Runtime Contract

## Goal

Pin C++ parity for default-context object sources flowing through a grouped
formula fallback converter in source-to-target application.

This slice covers a `DataConverterGroup` whose first child is a direct
`DataConverterFormula` fallback converter and whose second child is a simple
`DataConverterOperationValue`. C++ admits asset, artboard, and view-model
pointer sources through this group shape: the formula fallback converts the
object source to the numeric fallback value, then the operation-value child
continues the numeric pipeline.

## In Scope

- Default view-model state-machine context binding.
- Asset, artboard, and view-model pointer root sources.
- `DataConverterGroup<FormulaFallback, OperationValue>` source-to-target
  application into a `BindablePropertyNumber` target.
- Formula fallback token variants:
  - `FormulaTokenInput`.
  - `FormulaTokenFunction(random)` with `randomModeValue` values `0`, `1`, and
    `2`.
- C++ counted runtime random provider comparison through
  `runtimeStateMachineAdvances[].randomTotalCalls`.

## Out Of Scope

- Imported and owned context grouped object fallback behavior.
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

- Runtime graph admission must preserve asset, artboard, and view-model pointer
  default source values for a number target when the converter group starts
  with a formula fallback and continues with an operation-value converter.
- Source-to-target application must match C++ target values for each object
  source kind and token variant.
- Random-function fallback tokens remain random-inert for object sources inside
  this group shape; Rust and C++ random call counts stay at zero for every
  reported action.

## Validation

- `state_machine_default_viewmodel_object_formula_random_group_fallbacks_match_cpp_probe`
