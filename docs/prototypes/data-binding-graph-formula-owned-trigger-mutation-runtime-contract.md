# Formula Owned Trigger Mutation Runtime Contract

Purpose: extend owned-context source mutation to trigger sources feeding direct
`DataConverterFormula` fallback conversion and same-path direct observers.

## Scope

- A state machine can bind an owned `RuntimeOwnedViewModelInstance` whose root
  property is a trigger.
- After binding, `StateMachineInstance::set_owned_view_model_context_trigger_source_for_data_bind`
  mutates that owned root trigger source by state-machine data-bind index.
- A direct `DataConverterFormula` number bind using that trigger source
  reapplies on the next state-machine data-bind application and matches C++'s
  fallback behavior.
- A direct observer bind for the same owned trigger source updates alongside
  the formula fallback bind.
- The C++ probe retains the active owned trigger context and exposes
  `--runtime-set-owned-view-model-source-trigger` for parity comparison.

## Covered Case

- Direct formula fallback with an owned root trigger source, a post-bind owned
  source mutation, a zero-second advance, and a subsequent positive advance.

## Out Of Scope

- Number, symbol-list-index, boolean, enum, color, and string owned source
  mutation, covered by their own contracts.
- List, asset, artboard, or view-model owned source mutation APIs for formula
  fallback paths.
- Random formulas, random call counts, random cache invalidation, and
  `RandomMode` scheduling.
- Formula groups, interpolator groups, target-to-source/public-update formula
  behavior, and broader dirty-list scheduler parity.
- Property-name, nested, relative, or parent owned source mutation APIs.

## Verification

- Focused C++ probe test:
  - `state_machine_owned_viewmodel_trigger_formula_source_mutation_matches_cpp_probe`
- Broader `formula` C++ probe filtering should continue to pass.
- Full C++ probe and workspace tests should remain green before committing.
