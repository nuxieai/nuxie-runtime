# Data Binding Graph Formula Artboard Context Runtime Contract

## Purpose

Close the narrow imported/owned context path for artboard sources feeding
`DataConverterFormula` number targets.

The default-context object fallback slice proved that an artboard source can
enter the formula converter and take C++'s non-number fallback branch, writing
`0.0` to a number target. This slice proves that imported and owned
view-model context rebinding refreshes that artboard source before conversion.

## In Scope

- Root `ViewModelPropertyArtboard` sources with absolute
  `DataBindContext.sourcePathIds` of `[0, 0]`.
- Imported view-model context binding to a non-default serialized instance.
- Owned view-model context binding after mutating the owned artboard slot by
  property index.
- A formula-bound `BindablePropertyNumber.propertyValue` target using direct
  `DataConverterFormula` with `FormulaTokenInput`.
- A same-path `BindablePropertyArtboard.propertyValue` observer bind proving
  the rebound artboard value is visible in the graph.
- C++ probe parity for normal state-machine advancement after context binding.

## Out Of Scope

- View-model pointer sources under imported or owned contexts.
- Random/function-token formula artboard context behavior.
- Target-to-source, public-update, target-dirty, and reverse propagation for
  this context/converter pairing.
- Source mutation APIs after a formula artboard context bind.
- Relative, parent, nested, and name-path lookup.
- Listener-owned data binding, nested artboard propagation, pending add/remove
  behavior, and full dirty-list scheduler parity.

## Completion Checks

- Binding an imported context to the alternate serialized view-model instance
  refreshes the formula source and the same-path artboard observer source.
- Binding an owned context with a mutated artboard value refreshes the formula
  source and the same-path artboard observer source.
- The formula-bound number target still follows C++'s object fallback behavior
  and writes `0.0`.
- The observer artboard bind reports the rebound artboard id on both Rust and
  C++.
