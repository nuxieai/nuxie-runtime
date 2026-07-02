# Data Binding Graph Formula Default Object Random Longer Group Runtime Contract

## Goal

Pin C++ parity for default-context object sources flowing through longer grouped
formula fallback converter chains in source-to-target application.

This contract covers the first six longer `DataConverterGroup` shapes:

- `DataConverterGroup<FormulaFallback, OperationValue, OperationValue>`
- `DataConverterGroup<OperationValue, FormulaFallback, OperationValue>`
- `DataConverterGroup<FormulaFallback, OperationValue, OperationValue, OperationValue>`
- `DataConverterGroup<OperationValue, FormulaFallback, OperationValue, OperationValue>`
- `DataConverterGroup<OperationValue, OperationValue, FormulaFallback, OperationValue>`
- `DataConverterGroup<OperationValue, OperationValue, OperationValue, FormulaFallback>`

The purpose is to close the first longer-group hole without turning
`rive-runtime`'s data-bind graph admission rule into a general runtime
scheduler. The runtime admission rule is intentionally limited to groups with
exactly one formula fallback child and only operation-value children around it;
the C++ oracle coverage in this contract is the three- and four-child shapes
above.

## In Scope

- Default view-model state-machine context binding.
- Asset, artboard, and view-model pointer root sources.
- Source-to-target application into a `BindablePropertyNumber` target.
- Three- and four-child groups containing exactly one `DataConverterFormula`
  fallback converter and otherwise only `DataConverterOperationValue`
  converters.
- Formula fallback token variants:
  - `FormulaTokenInput`.
  - `FormulaTokenFunction(random)` with `randomModeValue` values `0`, `1`, and
    `2`.
- C++ counted runtime random provider comparison through
  `runtimeStateMachineAdvances[].randomTotalCalls`.

## Out Of Scope

- Imported and owned contexts for longer grouped object fallback converters.
- Post-bind source mutation/relink for longer grouped object fallback
  converters.
- Target-to-source/public-update behavior for longer grouped object fallback
  converters.
- Five-or-more-child group permutations not exercised by the C++ oracle in
  this contract.
- Groups with multiple formulas, no formula, or non-operation converter
  children around the formula.
- Relative, parent, name-based, or nested source path admission.
- Listener-owned data binding, nested artboard propagation, and full dirty-list
  scheduler parity.
- Real random generation, random seeding, platform RNG behavior, or parity with
  C++ `std::rand()`.

## Required Behavior

- Runtime graph admission must preserve asset, artboard, and view-model pointer
  default source values for a number target when a converter group contains
  exactly one formula fallback child and all other children are operation-value
  converters.
- Formula-first longer groups keep object-source random fallback tokens
  random-inert; Rust and C++ random call counts remain zero for every reported
  action.
- Operation-before-formula longer groups let the formula observe the numeric
  intermediate produced by the operation-value child. Random modes `0`, `1`,
  and `2` each report one C++ random provider call on the initial
  source-to-target application and no extra call on the later no-change
  advance.
- Target values and counted random calls must match C++ for each covered object
  source kind, group order, and formula token variant.

## Validation

- `state_machine_default_viewmodel_object_formula_random_longer_group_fallbacks_match_cpp_probe`
- `state_machine_default_viewmodel_object_formula_random_four_child_group_fallbacks_match_cpp_probe`
