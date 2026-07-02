# Data Binding Graph Formula Owned ViewModel Mutation Runtime Contract

## Purpose

Close the narrow post-bind owned source-mutation path for view-model pointer
sources feeding `DataConverterFormula` number targets.

Earlier view-model context slices proved that an owned view-model context can
bind a pointer source into the graph before formula conversion. This slice
proves that the public owned-context pointer relink API refreshes that same
formula source after binding, including fanout to a same-path direct
view-model observer.

## In Scope

- Root `ViewModelPropertyViewModel` sources with absolute
  `DataBindContext.sourcePathIds` of `[0, 0]`.
- An owned view-model context bound to a state-machine instance.
- Post-bind mutation through
  `StateMachineInstance::set_owned_view_model_context_view_model_source_for_data_bind`.
- Formula number source nodes retaining the referenced view-model instance id
  table needed to relink by instance index.
- A formula-bound `BindablePropertyNumber.propertyValue` target using direct
  `DataConverterFormula` with `FormulaTokenInput`.
- A same-path `BindablePropertyViewModel.propertyValue` observer bind proving
  the mutation fans out to every matching view-model pointer source node in
  the active graph.
- C++ probe parity for normal state-machine advancement before and after the
  mutation.

## Out Of Scope

- Imported/default context source mutation APIs for object-like formula
  sources.
- Random/function-token source mutation variants.
- Target-to-source, public-update, target-dirty, and reverse propagation for
  this mutation path.
- Relative, parent, nested, generated-child, and name-path pointer relinks.
- Listener-owned data binding, nested artboard propagation, pending add/remove
  behavior, and full dirty-list scheduler parity.

## Completion Checks

- Binding the owned context initializes the formula view-model pointer source
  and same-path view-model observer source.
- Mutating the owned view-model pointer source by formula data-bind index
  updates the owned context and every same-path view-model pointer source node
  in the graph.
- The formula-bound number target still follows C++'s object fallback behavior
  and writes `0.0` after mutation.
- The same-path view-model observer reports the mutated referenced instance
  index on both Rust and C++.
