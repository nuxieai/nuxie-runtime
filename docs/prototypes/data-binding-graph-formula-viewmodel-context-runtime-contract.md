# Data Binding Graph Formula View-Model Context Runtime Contract

## Purpose

Close the narrow imported/owned context path for view-model pointer sources
feeding `DataConverterFormula` number targets.

The default-context object fallback slice proved that a view-model pointer can
enter the formula converter and take C++'s non-number fallback branch, writing
`0.0` to a number target. This slice proves that imported and owned
view-model context rebinding refreshes that pointer source before conversion.

## In Scope

- Root `ViewModelPropertyViewModel` sources with absolute
  `DataBindContext.sourcePathIds` of `[0, 0]`.
- Imported view-model context binding to a non-default serialized root
  instance whose pointer selects a different child instance.
- Owned view-model context binding after relinking the owned pointer slot by
  property index.
- A formula-bound `BindablePropertyNumber.propertyValue` target using direct
  `DataConverterFormula` with `FormulaTokenInput`.
- A same-path `BindablePropertyViewModel.propertyValue` observer bind proving
  the rebound child instance pointer is visible in the graph.
- C++ probe parity for normal state-machine advancement after context binding.

## Out Of Scope

- Random/function-token formula view-model pointer context behavior.
- Target-to-source, public-update, target-dirty, and reverse propagation for
  this context/converter pairing.
- Source relink APIs after a formula view-model context bind.
- Relative, parent, nested, and name-path lookup.
- Listener-owned data binding, nested artboard propagation, pending add/remove
  behavior, and full dirty-list scheduler parity.

## Completion Checks

- Binding an imported context to the alternate serialized root instance
  refreshes the formula source and the same-path view-model observer source.
- Binding an owned context with a relinked child pointer refreshes the formula
  source and the same-path view-model observer source.
- The formula-bound number target still follows C++'s object fallback behavior
  and writes `0.0`.
- The observer view-model bind reports the rebound child instance index on both
  Rust and C++.
