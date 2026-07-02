# Data Binding Graph Formula Default ViewModel Relink Runtime Contract

## Purpose

Close the narrow default-context pointer relink path for view-model sources
feeding `DataConverterFormula` number targets.

Earlier default view-model pointer coverage proved that the default
view-model context can bind a pointer source into the graph before formula
conversion. This slice proves that default-context relink by data-bind index
refreshes that same formula source after binding, including fanout to a
same-path direct view-model observer.

## In Scope

- Root `ViewModelPropertyViewModel` sources with absolute
  `DataBindContext.sourcePathIds` of `[0, 0]`.
- The default view-model context bound to a state-machine instance.
- Post-bind relink through
  `StateMachineInstance::relink_default_view_model_view_model_source_for_data_bind`.
- Formula number source nodes retaining the referenced view-model instance id
  table needed to relink by instance index.
- A formula-bound `BindablePropertyNumber.propertyValue` target using direct
  `DataConverterFormula` with `FormulaTokenInput`.
- A same-path `BindablePropertyViewModel.propertyValue` observer bind proving
  the relink fans out to every matching view-model pointer source node in the
  active graph.
- C++ probe parity for explicit data-context advancement and normal
  state-machine advancement after the relink.

## Out Of Scope

- Random/function-token source relink variants.
- Target-to-source, public-update, target-dirty, and reverse propagation for
  this relink path.
- Relative, parent, nested, generated-child, and name-path pointer relinks.
- Listener-owned data binding, nested artboard propagation, pending add/remove
  behavior, and full dirty-list scheduler parity.

## Completion Checks

- Binding the default context initializes the formula view-model pointer
  source and same-path view-model observer source.
- Relinking the default view-model pointer source by formula data-bind index
  updates every same-path view-model pointer source node in the graph.
- The formula-bound number target still follows C++'s object fallback behavior
  and writes `0.0` after relink.
- The same-path view-model observer reports the relinked referenced instance
  index on both Rust and C++.
