# Formula Owned Symbol Mutation Runtime Contract

Purpose: prove that post-bind owned-context symbol-list-index mutation drives
direct deterministic `DataConverterFormula` conversion and same-path direct
observers.

## Scope

- A state machine can bind an owned `RuntimeOwnedViewModelInstance` whose root
  property is a symbol-list-index.
- After binding, `StateMachineInstance::set_owned_view_model_context_symbol_list_index_source_for_data_bind`
  mutates that owned root symbol-list-index source by state-machine data-bind
  index.
- A direct `DataConverterFormula` number bind using that source recomputes on
  the next state-machine data-bind application, casting the symbol-list-index
  value to `f32` before deterministic formula evaluation.
- A direct observer bind for the same owned symbol-list-index source updates
  alongside the converted formula bind.
- The C++ probe uses the retained active owned symbol-list-index context and
  `--runtime-set-owned-view-model-source-symbol-list-index` for parity
  comparison.

## Covered Case

- Direct deterministic symbol-list-index formula with an owned root
  symbol-list-index source, a post-bind owned source mutation, a zero-second
  advance, and a subsequent positive advance.

## Out Of Scope

- Number owned source mutation, covered separately by
  `data-binding-graph-formula-owned-number-mutation-runtime-contract.md`.
- Boolean, enum, color, string, trigger, list, or other formula input kinds.
- Random formulas, random call counts, random cache invalidation, and
  `RandomMode` scheduling.
- Formula groups, interpolator groups, target-to-source/public-update formula
  behavior, and broader dirty-list scheduler parity.
- Property-name, nested, relative, or parent owned source mutation APIs.

## Verification

- Focused C++ probe test:
  - `state_machine_owned_viewmodel_symbol_list_index_formula_source_mutation_matches_cpp_probe`
- Broader `formula` C++ probe filtering should continue to pass.
- Full C++ probe and workspace tests should remain green before committing.
