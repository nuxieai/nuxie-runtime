# Formula Owned Number Mutation Runtime Contract

Purpose: prove that the owned-context number source mutation API introduced for
OperationViewModel also drives a second converter family: direct
`DataConverterFormula` number conversion.

## Scope

- A state machine can bind an owned `RuntimeOwnedViewModelInstance`.
- After binding, `StateMachineInstance::set_owned_view_model_context_number_source_for_data_bind`
  mutates an owned root number source by state-machine data-bind index.
- A direct `DataConverterFormula` number bind using that source recomputes on
  the next state-machine data-bind application.
- A direct observer bind for the same owned number source updates alongside
  the converted formula bind.
- The C++ probe uses the retained active owned number context and
  `--runtime-set-owned-view-model-source-number` for parity comparison.

## Covered Case

- Direct deterministic number formula with an owned root number source, a
  post-bind owned source mutation, a zero-second advance, and a subsequent
  positive advance.

## Out Of Scope

- Symbol-list-index, boolean, enum, color, string, trigger, list, or other
  formula input kinds.
- Random formulas, random call counts, random cache invalidation, and
  `RandomMode` scheduling.
- Formula groups, interpolator groups, target-to-source/public-update formula
  behavior, and broader dirty-list scheduler parity.
- Property-name, nested, relative, or parent owned source mutation APIs.

## Verification

- Focused C++ probe test:
  - `state_machine_owned_viewmodel_number_formula_source_mutation_matches_cpp_probe`
- Broader `formula` C++ probe filtering should continue to pass.
- Full C++ probe and workspace tests should remain green before committing.
