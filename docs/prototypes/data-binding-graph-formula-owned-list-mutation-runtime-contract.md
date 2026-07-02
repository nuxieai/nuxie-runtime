# Formula Owned List Mutation Runtime Contract

Purpose: extend owned-context source mutation to list item counts feeding direct
`DataConverterFormula` fallback conversion and same-path direct observers.

## Scope

- A state machine can bind an owned `RuntimeOwnedViewModelInstance` whose root
  property is a list.
- After binding, `StateMachineInstance::set_owned_view_model_context_list_source_item_count_for_data_bind`
  mutates that owned root list source item count by state-machine data-bind
  index.
- A direct `DataConverterFormula` number bind using that list source reapplies
  on the next state-machine data-bind application and matches C++'s fallback
  behavior.
- A direct observer bind for the same owned list source updates alongside the
  formula fallback bind.
- The C++ probe retains the active owned list context and exposes
  `--runtime-set-owned-view-model-source-list` for parity comparison.

## Covered Case

- Direct formula fallback with an owned root list source, a post-bind owned
  item-count mutation, a zero-second advance, and a subsequent positive
  advance.

## Out Of Scope

- Number, symbol-list-index, boolean, enum, color, string, and trigger owned
  source mutation, covered by their own contracts.
- Generated list item identity beyond item-count parity.
- Asset, artboard, or view-model owned source mutation APIs for formula
  fallback paths.
- Random formulas, random call counts, random cache invalidation, and
  `RandomMode` scheduling.
- Formula groups, interpolator groups, target-to-source/public-update formula
  behavior, and broader dirty-list scheduler parity.
- Property-name, nested, relative, or parent owned source mutation APIs beyond
  the existing bind setup for this root-list fixture.

## Verification

- Focused C++ probe test:
  - `state_machine_owned_viewmodel_list_formula_source_mutation_matches_cpp_probe`
- Broader `formula` C++ probe filtering should continue to pass.
- Full C++ probe and workspace tests should remain green before committing.
