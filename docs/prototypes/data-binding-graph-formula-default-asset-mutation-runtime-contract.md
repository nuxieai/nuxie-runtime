# Data Binding Graph Formula Default Asset Mutation Runtime Contract

## Purpose

Close the narrow default-context source-mutation path for asset sources feeding
`DataConverterFormula` number targets.

Earlier default asset context coverage proved that the default view-model
context can bind an asset source into the graph before formula conversion.
This slice proves that default-context mutation by data-bind index refreshes
that same formula source after binding, including fanout to a same-path direct
asset observer.

## In Scope

- Root `ViewModelPropertyAssetImage` sources with absolute
  `DataBindContext.sourcePathIds` of `[0, 0]`.
- The default view-model context bound to a state-machine instance.
- Post-bind mutation through
  `StateMachineInstance::set_default_view_model_asset_source_for_data_bind`.
- A formula-bound `BindablePropertyNumber.propertyValue` target using direct
  `DataConverterFormula` with `FormulaTokenInput`.
- A same-path `BindablePropertyAsset.propertyValue` observer bind proving the
  mutation fans out to every matching asset source node in the active graph.
- C++ probe parity for normal state-machine advancement before and after the
  mutation.

## Out Of Scope

- Default artboard and view-model pointer source mutations.
- Random/function-token source mutation variants.
- Target-to-source, public-update, target-dirty, and reverse propagation for
  this mutation path.
- Relative, parent, nested, and name-path lookup.
- Listener-owned data binding, nested artboard propagation, pending add/remove
  behavior, and full dirty-list scheduler parity.

## Completion Checks

- Binding the default context initializes the formula asset source and
  same-path asset observer source.
- Mutating the default asset source by formula data-bind index updates every
  same-path asset source node in the graph.
- The formula-bound number target still follows C++'s object fallback behavior
  and writes `0.0` after mutation.
- The same-path asset observer reports the mutated asset id on both Rust and
  C++.
