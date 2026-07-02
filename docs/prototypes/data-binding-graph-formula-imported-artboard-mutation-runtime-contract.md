# Data Binding Graph Formula Imported Artboard Mutation Runtime Contract

## Purpose

Close the narrow imported-context source-mutation path for artboard sources
feeding `DataConverterFormula` number targets.

Earlier imported artboard context slices proved that a file-backed view-model
instance can bind an artboard source into the graph before formula conversion.
This slice proves that the public imported-context mutation API refreshes that
same formula source after binding, including fanout to a same-path direct
artboard observer.

## In Scope

- Root `ViewModelPropertyArtboard` sources with absolute
  `DataBindContext.sourcePathIds` of `[0, 0]`.
- An imported `RuntimeImportedViewModelInstanceContext` bound to a
  state-machine instance.
- Post-bind mutation through
  `StateMachineInstance::set_imported_view_model_context_artboard_source_for_data_bind`.
- A formula-bound `BindablePropertyNumber.propertyValue` target using direct
  `DataConverterFormula` with `FormulaTokenInput`.
- A same-path `BindablePropertyArtboard.propertyValue` observer bind proving
  the mutation fans out to every matching artboard source node in the active
  graph.
- C++ probe parity for normal state-machine advancement before and after the
  mutation.

## Out Of Scope

- Imported view-model pointer source mutations.
- Default-context object-source mutation for formula paths.
- Random/function-token source mutation variants.
- Target-to-source, public-update, target-dirty, and reverse propagation for
  this mutation path.
- Relative, parent, nested, and name-path lookup.
- Listener-owned data binding, nested artboard propagation, pending add/remove
  behavior, and full dirty-list scheduler parity.

## Completion Checks

- Binding the imported context initializes the formula artboard source and
  same-path artboard observer source.
- Mutating the imported artboard source by formula data-bind index updates the
  context override and every same-path artboard source node in the graph.
- The formula-bound number target still follows C++'s object fallback behavior
  and writes `0.0` after mutation.
- The same-path artboard observer reports the mutated artboard id on both Rust
  and C++.
