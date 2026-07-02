# Formula Owned Enum Mutation Runtime Contract

Purpose: extend the non-number owned-context source mutation path to enum
sources feeding direct `DataConverterFormula` fallback conversion and same-path
direct observers.

## Scope

- A state machine can bind an owned `RuntimeOwnedViewModelInstance` whose root
  property is an enum.
- After binding, `StateMachineInstance::set_owned_view_model_context_enum_source_for_data_bind`
  mutates that owned root enum source by state-machine data-bind index.
- A direct `DataConverterFormula` number bind using that enum source reapplies
  on the next state-machine data-bind application and matches C++'s fallback
  behavior.
- A direct observer bind for the same owned enum source updates alongside the
  formula fallback bind.
- The C++ probe retains the active owned enum context and exposes
  `--runtime-set-owned-view-model-source-enum` for parity comparison.

## Covered Case

- Direct formula fallback with an owned root enum source, a post-bind owned
  source mutation, a zero-second advance, and a subsequent positive advance.

## Out Of Scope

- Number, symbol-list-index, and boolean owned source mutation, covered by
  their own contracts.
- Color, string, trigger, list, asset, artboard, or view-model owned source
  mutation APIs.
- Random formulas, random call counts, random cache invalidation, and
  `RandomMode` scheduling.
- Formula groups, interpolator groups, target-to-source/public-update formula
  behavior, and broader dirty-list scheduler parity.
- Property-name, nested, relative, or parent owned source mutation APIs.

## Verification

- Focused C++ probe test:
  - `state_machine_owned_viewmodel_enum_formula_source_mutation_matches_cpp_probe`
- Broader `formula` C++ probe filtering should continue to pass.
- Full C++ probe and workspace tests should remain green before committing.
