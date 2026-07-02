# Data Binding Graph Formula Owned Artboard Mutation Runtime Contract

## Purpose

Close the narrow post-bind owned source-mutation path for artboard sources
feeding `DataConverterFormula` number targets.

Earlier artboard context slices proved that an owned view-model context can
bind an artboard source into the graph before formula conversion. This slice
proves that the public owned-context source mutation API refreshes the same
graph source after binding, including fanout to a same-path direct artboard
observer.

## In Scope

- Root `ViewModelPropertyArtboard` sources with absolute
  `DataBindContext.sourcePathIds` of `[0, 0]`.
- An owned view-model context bound to a state-machine instance.
- Post-bind mutation through
  `StateMachineInstance::set_owned_view_model_context_artboard_source_for_data_bind`.
- A formula-bound `BindablePropertyNumber.propertyValue` target using direct
  `DataConverterFormula` with `FormulaTokenInput`.
- A same-path `BindablePropertyArtboard.propertyValue` observer bind proving
  the mutation fans out to every matching artboard source node in the active
  graph.
- C++ probe parity for normal state-machine advancement before and after the
  mutation.

## Out Of Scope

- View-model pointer owned source mutations.
- Imported/default context source mutation APIs for object-like formula
  sources.
- Random/function-token source mutation variants.
- Target-to-source, public-update, target-dirty, and reverse propagation for
  this mutation path.
- Relative, parent, nested, and name-path lookup.
- Listener-owned data binding, nested artboard propagation, pending add/remove
  behavior, and full dirty-list scheduler parity.

## Completion Checks

- Binding the owned context initializes the formula artboard source and
  same-path artboard observer source.
- Mutating the owned artboard source by data-bind index updates the owned
  context and every same-path artboard source node in the graph.
- The formula-bound number target still follows C++'s object fallback behavior
  and writes `0.0` after mutation.
- The same-path artboard observer reports the mutated artboard id on both Rust
  and C++.
