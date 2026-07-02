# Formula Owned Boolean Mutation Runtime Contract

Purpose: prove the first non-number owned-context source mutation path:
post-bind owned boolean mutation drives direct `DataConverterFormula` fallback
conversion and same-path direct observers.

## Scope

- A state machine can bind an owned `RuntimeOwnedViewModelInstance` whose root
  property is a boolean.
- After binding, `StateMachineInstance::set_owned_view_model_context_boolean_source_for_data_bind`
  mutates that owned root boolean source by state-machine data-bind index.
- A direct `DataConverterFormula` number bind using that boolean source
  reapplies on the next state-machine data-bind application and matches C++'s
  fallback behavior.
- A direct observer bind for the same owned boolean source updates alongside
  the formula fallback bind.
- The C++ probe retains the active owned boolean context and exposes
  `--runtime-set-owned-view-model-source-bool` for parity comparison.

## Covered Case

- Direct formula fallback with an owned root boolean source, a post-bind owned
  source mutation, a zero-second advance, and a subsequent positive advance.

## Out Of Scope

- Number and symbol-list-index owned source mutation, covered by their own
  contracts.
- Enum, color, string, trigger, list, asset, artboard, or view-model owned
  source mutation APIs.
- Random formulas, random call counts, random cache invalidation, and
  `RandomMode` scheduling.
- Formula groups, interpolator groups, target-to-source/public-update formula
  behavior, and broader dirty-list scheduler parity.
- Property-name, nested, relative, or parent owned source mutation APIs.

## Verification

- Focused C++ probe test:
  - `state_machine_owned_viewmodel_boolean_formula_source_mutation_matches_cpp_probe`
- Broader `formula` C++ probe filtering should continue to pass.
- Full C++ probe and workspace tests should remain green before committing.
