# Data Binding Graph Formula Random Asset Context Runtime Contract

## Purpose

Close the narrow imported/owned context path for asset sources feeding
`DataConverterFormula` random-function number targets.

The deterministic asset context slice proved that imported and owned
view-model context rebinding refreshes an asset source before formula
conversion. This slice proves the same context rebinding behavior when the
formula output token is `FunctionType::random`, including C++'s early
non-number fallback that skips random generation for object-like source values.

## In Scope

- Root `ViewModelPropertyAssetImage` sources with absolute
  `DataBindContext.sourcePathIds` of `[0, 0]`.
- Imported view-model context binding to a non-default serialized instance.
- Owned view-model context binding after mutating the owned asset slot by
  property index.
- `DataConverterFormula` output tokens using `FunctionType::random` with
  `randomModeValue` values `0`, `1`, and `2`.
- A formula-bound `BindablePropertyNumber.propertyValue` target and same-path
  `BindablePropertyAsset.propertyValue` observer bind.
- C++ probe parity for normal state-machine advancement after context binding,
  including zero random-provider calls on both sides.

## Out Of Scope

- Random/function-token artboard and view-model pointer contexts.
- Target-to-source, public-update, target-dirty, and reverse propagation for
  random asset context binds.
- Source mutation APIs after a formula random asset context bind.
- Relative, parent, nested, and name-path lookup.
- Listener-owned data binding, nested artboard propagation, pending add/remove
  behavior, real random generation beyond the seeded probe path, and full
  dirty-list scheduler parity.

## Completion Checks

- Imported context rebinding refreshes the formula asset source and the
  same-path asset observer source for all three random modes.
- Owned context rebinding refreshes the formula asset source and the same-path
  asset observer source for all three random modes.
- The formula-bound number target still follows C++'s object fallback behavior
  and writes `0.0`.
- C++ and Rust both report zero random-provider calls because the object
  fallback happens before random generation.
